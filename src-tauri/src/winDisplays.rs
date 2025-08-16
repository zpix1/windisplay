use crate::displays::{BrightnessInfo, DisplayInfo, Displays, Resolution, ScaleInfo};
use serde::Deserialize;
use serde_json::Value;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::Command;

pub struct WinDisplays;

impl WinDisplays {
    pub fn new() -> Self {
        Self
    }
}

impl Displays for WinDisplays {
    fn get_all_monitors(&self) -> Result<Vec<DisplayInfo>, String> {
        get_all_monitors_windows()
    }

    fn set_monitor_resolution(
        &self,
        device_name: String,
        width: u32,
        height: u32,
        refresh_hz: Option<u32>,
    ) -> Result<(), String> {
        set_monitor_resolution_windows(device_name, width, height, refresh_hz)
    }

    fn get_monitor_brightness(&self, device_name: String) -> Result<BrightnessInfo, String> {
        get_monitor_brightness_windows(device_name)
    }

    fn set_monitor_brightness(&self, device_name: String, percent: u32) -> Result<(), String> {
        set_monitor_brightness_windows(device_name, percent)
    }

    fn identify_monitors(&self, app_handle: tauri::AppHandle) -> Result<(), String> {
        identify_monitors_windows(app_handle)
    }

    fn set_monitor_orientation(
        &self,
        device_name: String,
        orientation_degrees: u32,
    ) -> Result<(), String> {
        set_monitor_orientation_windows(device_name, orientation_degrees)
    }

    fn set_monitor_scale(&self, device_name: String, scale_percent: u32) -> Result<(), String> {
        set_monitor_scale_windows(&device_name, scale_percent)
    }

    fn enable_hdr(&self, device_name: String, enable: bool) -> Result<(), String> {
        // Find logical display index matching `device_name` in the same order as get_all_monitors_windows
        let monitors = get_all_monitors_windows()?;
        let index = monitors
            .iter()
            .position(|m| m.device_name == device_name)
            .ok_or_else(|| "Display not found".to_string())?;

        match crate::winHdr::set_hdr_status_by_index(index, enable) {
            Some(crate::winHdr::Status::Unsupported) => {
                Err("HDR unsupported on this display".to_string())
            }
            Some(_status) => Ok(()),
            None => Err("Failed to change HDR state".to_string()),
        }
    }
}

// Attempt to fetch a preferred/native mode using registry-stored settings for the device.
// This typically reflects the monitor's native timing chosen by Windows on first connect.
fn query_preferred_native_resolution(device_name: &str) -> Option<(u32, u32)> {
    use std::mem::{size_of, zeroed};
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::Graphics::Gdi::{EnumDisplaySettingsExW, DEVMODEW, ENUM_REGISTRY_SETTINGS};

    let mut dm: DEVMODEW = unsafe { zeroed() };
    dm.dmSize = size_of::<DEVMODEW>() as u16;
    let wide = to_wide_null_terminated(device_name);
    let ok: BOOL = unsafe {
        EnumDisplaySettingsExW(
            windows::core::PCWSTR(wide.as_ptr()),
            ENUM_REGISTRY_SETTINGS,
            &mut dm,
            windows::Win32::Graphics::Gdi::ENUM_DISPLAY_SETTINGS_FLAGS(0),
        )
    };
    if ok.as_bool() {
        let w = dm.dmPelsWidth as u32;
        let h = dm.dmPelsHeight as u32;
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }
    None
}

#[derive(Debug, Clone, Default, Deserialize)]
struct PsEdidEntry {
    #[serde(default, deserialize_with = "empty_string_if_null")]
    Manufacturer: String,
    #[serde(default, deserialize_with = "empty_string_if_null")]
    Model: String,
    #[serde(default, deserialize_with = "empty_string_if_null")]
    SerialNumber: String,
    #[serde(default, deserialize_with = "empty_string_if_null")]
    ProductCodeId: String,
    #[serde(default)]
    WeekOfManufacture: Option<u32>,
    #[serde(default)]
    YearOfManufacture: Option<u32>,
    #[serde(default)]
    InstanceName: Option<String>,
    #[serde(default)]
    VideoOutputTechnology: Option<u32>,
    #[serde(default)]
    Active: Option<bool>,
}

fn empty_string_if_null<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

fn to_lower(s: &str) -> String {
    s.to_ascii_lowercase()
}

// Runs a PowerShell script invisibly and returns stdout when successful
fn run_powershell_hidden(script: &str) -> Option<String> {
    let candidates: &[&str] = &[
        "pwsh",
        "powershell",
        r"C:\\Windows\\Sysnative\\WindowsPowerShell\\v1.0\\powershell.exe",
        r"C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe",
    ];
    // Ensure PowerShell writes UTF-8 (no BOM) to stdout even on Windows PowerShell 5.1
    // Prepend a tiny prolog to set the output encoding, then run the provided script
    let wrapped_script = format!(
        "$ErrorActionPreference='SilentlyContinue'; try {{ [Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false); $global:OutputEncoding = [Console]::OutputEncoding }} catch {{}}; {}",
        script
    );
    for exe in candidates {
        log::debug!("run_powershell_hidden: trying '{}'", exe);
        let mut cmd = Command::new(exe);
        cmd.args([
            "-NoProfile",
            "-NoLogo",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &wrapped_script,
        ]);
        #[cfg(windows)]
        {
            // CREATE_NO_WINDOW
            cmd.creation_flags(0x08000000);
        }
        match cmd.output() {
            Ok(out) => {
                let success = out.status.success();
                let code = out.status.code();
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                log::debug!(
                    "run_powershell_hidden: exe='{}' success={} code={:?} stdout_len={} stderr_len={}",
                    exe,
                    success,
                    code,
                    out.stdout.len(),
                    out.stderr.len()
                );
                // Log full outputs as requested
                if !stdout.is_empty() {
                    log::debug!(
                        "run_powershell_hidden: FULL STDOUT for '{}':\n{}",
                        exe,
                        stdout
                    );
                } else {
                    log::debug!("run_powershell_hidden: STDOUT empty for '{}'", exe);
                }
                if !stderr.is_empty() {
                    log::warn!(
                        "run_powershell_hidden: FULL STDERR for '{}':\n{}",
                        exe,
                        stderr
                    );
                }
                if success {
                    if !stdout.trim().is_empty() {
                        return Some(stdout);
                    } else {
                        log::warn!(
                            "run_powershell_hidden: empty stdout from '{}' despite success",
                            exe
                        );
                    }
                }
            }
            Err(e) => {
                log::warn!("run_powershell_hidden: failed to spawn '{}': {}", exe, e);
            }
        }
    }
    None
}

fn fetch_edid_metadata_via_powershell() -> Vec<PsEdidEntry> {
    // PowerShell script adapted from test.py to gather EDID metadata
    let script = r#"
$toStr = {
  param([UInt16[]]$arr)
  if (-not $arr) { return $null }
  ($arr | ForEach-Object { if ($_ -gt 0 -and $_ -lt 256) { [char]$_ } }) -join ''
}

$ids = Get-CimInstance -Namespace root\wmi -Class WmiMonitorID -ErrorAction SilentlyContinue
$basic = @{}
Get-CimInstance -Namespace root\wmi -Class WmiMonitorBasicDisplayParams -ErrorAction SilentlyContinue | ForEach-Object {
  $basic[$_.InstanceName] = $_
}

$conn = @{}
Get-CimInstance -Namespace root\wmi -Class WmiMonitorConnectionParams -ErrorAction SilentlyContinue | ForEach-Object {
  $conn[$_.InstanceName] = $_
}

$results = @()
foreach ($m in $ids) {
  $inst = $m.InstanceName
  $b = $basic[$inst]
  $c = $conn[$inst]
  $obj = [pscustomobject]@{
    InstanceName          = $inst
    Manufacturer          = (& $toStr $m.ManufacturerName)
    Model                 = (& $toStr $m.UserFriendlyName)
    SerialNumber          = (& $toStr $m.SerialNumberID)
    ProductCodeId         = (& $toStr $m.ProductCodeID)
    WeekOfManufacture     = $m.WeekOfManufacture
    YearOfManufacture     = $m.YearOfManufacture
    VideoOutputTechnology = if ($c) { [uint32]$c.VideoOutputTechnology } else { $null }
    Active                = if ($c) { [bool]$c.Active } else { $null }
  }
  $results += $obj
}
if ($results) { $results | ConvertTo-Json -Depth 4 } else { '[]' }
"#;

    let Some(raw) = run_powershell_hidden(script) else {
        return Vec::new();
    };
    // Parse JSON, tolerate either array or single object
    match serde_json::from_str::<Value>(&raw) {
        Ok(Value::Array(arr)) => arr
            .into_iter()
            .filter_map(|v| serde_json::from_value::<PsEdidEntry>(v).ok())
            .collect(),
        Ok(Value::Object(_)) => serde_json::from_str::<PsEdidEntry>(&raw)
            .map(|e| vec![e])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn get_all_monitors_windows() -> Result<Vec<DisplayInfo>, String> {
    use std::mem::{size_of, zeroed};
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayDevicesW, EnumDisplaySettingsExW, DEVMODEW, DISPLAY_DEVICEW,
        DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_MIRRORING_DRIVER,
        DISPLAY_DEVICE_PRIMARY_DEVICE,
    };

    log::debug!("get_all_monitors_windows: start");
    let mut displays: Vec<DisplayInfo> = Vec::new();
    // Pre-fetch HDR display list and match by index as requested
    let hdr_displays = crate::winHdr::get_displays();
    log::debug!("HDR displays fetched: count={}", hdr_displays.len());
    let mut logical_display_index: usize = 0;

    // Fetch EDID metadata from PowerShell once; if it fails, we proceed with defaults
    let mut edid_entries: Vec<PsEdidEntry> = fetch_edid_metadata_via_powershell();
    log::debug!(
        "EDID entries fetched via PowerShell: count={}",
        edid_entries.len()
    );
    let mut used_edid: Vec<bool> = vec![false; edid_entries.len()];

    let mut device_index: u32 = 0;
    loop {
        let mut dd: DISPLAY_DEVICEW = unsafe { zeroed() };
        dd.cb = size_of::<DISPLAY_DEVICEW>() as u32;
        let ok: BOOL = unsafe { EnumDisplayDevicesW(None, device_index, &mut dd, 0) };
        if !ok.as_bool() {
            log::debug!(
                "EnumDisplayDevicesW returned false at device_index={}, stopping enumeration",
                device_index
            );
            break;
        }

        let state = dd.StateFlags;
        let is_attached = (state & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) != 0;
        let is_mirror = (state & DISPLAY_DEVICE_MIRRORING_DRIVER) != 0;
        if !is_attached || is_mirror {
            log::debug!(
                "Skipping device_index={} is_attached={} is_mirror={}",
                device_index,
                is_attached,
                is_mirror
            );
            device_index += 1;
            continue;
        }

        let device_name = widestr_to_string(&dd.DeviceName);
        let mut friendly_name = widestr_to_string(&dd.DeviceString);
        let mut dd_device_id = widestr_to_string(&dd.DeviceID);
        let is_primary = (state & DISPLAY_DEVICE_PRIMARY_DEVICE) != 0;

        let mut current_mode: DEVMODEW = unsafe { zeroed() };
        current_mode.dmSize = size_of::<DEVMODEW>() as u16;
        let device_name_wide = to_wide_null_terminated(&device_name);
        let ok_current: BOOL = unsafe {
            EnumDisplaySettingsExW(
                windows::core::PCWSTR(device_name_wide.as_ptr()),
                windows::Win32::Graphics::Gdi::ENUM_CURRENT_SETTINGS,
                &mut current_mode,
                windows::Win32::Graphics::Gdi::ENUM_DISPLAY_SETTINGS_FLAGS(0),
            )
        };
        if !ok_current.as_bool() {
            log::warn!(
                "EnumDisplaySettingsExW current failed for device_name='{}' (index={})",
                device_name,
                device_index
            );
            device_index += 1;
            continue;
        }

        let (pos_x, pos_y) = unsafe {
            (
                current_mode.Anonymous1.Anonymous2.dmPosition.x,
                current_mode.Anonymous1.Anonymous2.dmPosition.y,
            )
        };

        let current = Resolution {
            width: current_mode.dmPelsWidth as u32,
            height: current_mode.dmPelsHeight as u32,
            bits_per_pixel: current_mode.dmBitsPerPel as u32,
            refresh_hz: current_mode.dmDisplayFrequency as u32,
        };

        // Orientation is exposed by dmDisplayOrientation (0,1,2,3) which maps to 0,90,180,270 degrees.
        use windows::Win32::Graphics::Gdi::{DMDO_180, DMDO_270, DMDO_90, DMDO_DEFAULT};
        let orientation_degrees: u32 =
            match unsafe { current_mode.Anonymous1.Anonymous2.dmDisplayOrientation } {
                DMDO_DEFAULT => 0,
                DMDO_90 => 90,
                DMDO_180 => 180,
                DMDO_270 => 270,
                _ => 0,
            };

        let mut modes: Vec<Resolution> = Vec::new();
        let mut mode_index: u32 = 0;
        loop {
            let mut dm: DEVMODEW = unsafe { zeroed() };
            dm.dmSize = size_of::<DEVMODEW>() as u16;
            let device_name_wide = to_wide_null_terminated(&device_name);
            let ok_mode: BOOL = unsafe {
                EnumDisplaySettingsExW(
                    windows::core::PCWSTR(device_name_wide.as_ptr()),
                    windows::Win32::Graphics::Gdi::ENUM_DISPLAY_SETTINGS_MODE(mode_index),
                    &mut dm,
                    windows::Win32::Graphics::Gdi::ENUM_DISPLAY_SETTINGS_FLAGS(0),
                )
            };
            if !ok_mode.as_bool() {
                log::debug!(
                    "End of mode enumeration for device_name='{}' after {} modes",
                    device_name,
                    mode_index
                );
                break;
            }

            let res = Resolution {
                width: dm.dmPelsWidth as u32,
                height: dm.dmPelsHeight as u32,
                bits_per_pixel: dm.dmBitsPerPel as u32,
                refresh_hz: dm.dmDisplayFrequency as u32,
            };

            if !modes.iter().any(|m| {
                m.width == res.width
                    && m.height == res.height
                    && m.bits_per_pixel == res.bits_per_pixel
                    && m.refresh_hz == res.refresh_hz
            }) {
                modes.push(res);
            }

            mode_index += 1;
        }

        // Prefer OS-reported preferred/native mode via DisplayConfig; fall back to largest area mode
        let max_native = if let Some((nw, nh)) = query_preferred_native_resolution(&device_name) {
            log::debug!(
                "Preferred/native resolution for '{}' -> {}x{}",
                device_name,
                nw,
                nh
            );
            // Pick the highest refresh for the preferred width/height among available modes
            modes
                .iter()
                .filter(|m| m.width == nw && m.height == nh)
                .cloned()
                .max_by_key(|m| m.refresh_hz)
                .unwrap_or(Resolution {
                    width: nw,
                    height: nh,
                    bits_per_pixel: current.bits_per_pixel,
                    refresh_hz: current.refresh_hz,
                })
        } else {
            log::debug!(
                "No preferred/native resolution for '{}', falling back to max area mode",
                device_name
            );
            modes
                .iter()
                .cloned()
                .max_by_key(|m| (m.width as u64) * (m.height as u64))
                .unwrap_or_else(|| current.clone())
        };

        // Try to get the actual monitor device (e.g., \\.\n+        // DISPLAY1\Monitor0) to use its friendly name and DeviceID for EDID matching.
        // The top-level DISPLAY_DEVICE often represents the adapter, not the monitor.
        unsafe {
            let mut mon: DISPLAY_DEVICEW = zeroed();
            mon.cb = size_of::<DISPLAY_DEVICEW>() as u32;
            // Query the first monitor associated with this display device
            let ok_mon = EnumDisplayDevicesW(
                windows::core::PCWSTR(to_wide_null_terminated(&device_name).as_ptr()),
                0,
                &mut mon,
                0,
            );
            if ok_mon.as_bool() {
                let mon_name = widestr_to_string(&mon.DeviceString);
                if !mon_name.is_empty() {
                    friendly_name = mon_name;
                }
                let mon_id = widestr_to_string(&mon.DeviceID);
                if !mon_id.is_empty() {
                    dd_device_id = mon_id;
                }
            }
        }

        // Choose EDID metadata for this monitor (best-effort matching)
        let mut model: String = String::new();
        let mut manufacturer: String = String::new();
        let mut serial: String = String::new();
        let mut year_of_manufacture: u32 = 0;
        let mut week_of_manufacture: u32 = 0;
        let mut connection: String = String::new();
        let mut built_in: bool = false;
        let mut active: bool = false;

        // Try to match using stable identifiers first (WMI InstanceName vs monitor DeviceID),
        // then fall back to model/manufacturer presence.
        let friendly_l = to_lower(&friendly_name);
        let devid_l = to_lower(&dd_device_id);
        let mut chosen_idx: Option<usize> = None;
        // 1) Prefer exact-ish match using InstanceName fragment (e.g., VENDOR+PRODUCT)
        for (idx, e) in edid_entries.iter().enumerate() {
            if used_edid[idx] {
                continue;
            }
            if let Some(inst) = &e.InstanceName {
                let inst_l = to_lower(inst);
                // Some DeviceIDs start with MONITOR\\, others with DISPLAY\\. Compare loosely.
                if !inst_l.is_empty() && (devid_l.contains(&inst_l) || inst_l.contains(&devid_l)) {
                    chosen_idx = Some(idx);
                    break;
                }
                // Also try matching the vendor+product fragment (between first and second backslashes)
                if let Some(pos1) = inst_l.find('\\') {
                    if let Some(rest) = inst_l.get(pos1 + 1..) {
                        let frag = match rest.find('\\') {
                            Some(p) => &rest[..p],
                            None => rest,
                        };
                        if !frag.is_empty() && devid_l.contains(frag) {
                            chosen_idx = Some(idx);
                            break;
                        }
                    }
                }
            }
        }
        // 2) Fallback: match by model/manufacturer presence
        if chosen_idx.is_none() {
            for (idx, e) in edid_entries.iter().enumerate() {
                if used_edid[idx] {
                    continue;
                }
                let mdl = to_lower(&e.Model);
                let mfr = to_lower(&e.Manufacturer);
                if (!mdl.is_empty() && (friendly_l.contains(&mdl) || devid_l.contains(&mdl)))
                    || (!mfr.is_empty() && (friendly_l.contains(&mfr) || devid_l.contains(&mfr)))
                {
                    chosen_idx = Some(idx);
                    break;
                }
            }
        }
        // Fallback: assign by display index order
        if chosen_idx.is_none() {
            for (idx, used) in used_edid.iter().enumerate() {
                if !*used {
                    chosen_idx = Some(idx);
                    break;
                }
            }
        }
        if let Some(idx) = chosen_idx {
            used_edid[idx] = true;
            let e = &edid_entries[idx];
            log::debug!(
                "EDID matched for '{}' using entry index={} model='{}' manufacturer='{}'",
                device_name,
                idx,
                e.Model,
                e.Manufacturer
            );
            model = e.Model.clone();
            manufacturer = e.Manufacturer.clone();
            serial = e.SerialNumber.clone();
            year_of_manufacture = e.YearOfManufacture.unwrap_or(0);
            week_of_manufacture = e.WeekOfManufacture.unwrap_or(0);
            // Map VideoOutputTechnology to a friendly name similar to test.py
            if let Some(code) = e.VideoOutputTechnology {
                connection = match code as i64 {
                    -2 => "Uninitialized".to_string(),
                    -1 => "Other".to_string(),
                    0 => "VGA".to_string(),
                    1 => "S-Video".to_string(),
                    2 => "Composite".to_string(),
                    3 => "Component".to_string(),
                    4 => "DVI".to_string(),
                    5 => "HDMI".to_string(),
                    6 => "LVDS / MIPI-DSI".to_string(),
                    8 => "D-Jpn".to_string(),
                    9 => "SDI".to_string(),
                    10 => "DisplayPort (external)".to_string(),
                    11 => "DisplayPort (embedded)".to_string(),
                    12 => "UDI (external)".to_string(),
                    13 => "UDI (embedded)".to_string(),
                    14 => "SDTV dongle".to_string(),
                    15 => "Miracast (wireless)".to_string(),
                    16 => "Indirect (wired)".to_string(),
                    2147483648 => "Internal (adapter)".to_string(),
                    other => format!("Unknown ({})", other),
                };
                // Built-in detection as in test.py
                built_in = matches!(code, 6 | 11 | 13 | 2147483648);
            }
            active = e.Active.unwrap_or(false);
        }

        // Determine per-monitor scaling (DPI / 96)
        let scale_factor: f32 = get_monitor_scale_for_device(&device_name);

        // Query available scales and recommended state via DisplayConfig undocumented API
        let scales: Vec<ScaleInfo> = match get_scales_for_device(&device_name) {
            Ok(v) => v,
            Err(_) => Vec::new(),
        };

        let hdr_status: String = match hdr_displays.get(logical_display_index) {
            Some(h) => match h.status {
                crate::winHdr::Status::Unsupported => "unsupported".to_string(),
                crate::winHdr::Status::Off => "off".to_string(),
                crate::winHdr::Status::On => "on".to_string(),
            },
            None => "unsupported".to_string(),
        };

        displays.push(DisplayInfo {
            device_name,
            friendly_name,
            is_primary,
            position_x: pos_x,
            position_y: pos_y,
            orientation: orientation_degrees,
            current,
            modes,
            max_native,
            model,
            serial,
            manufacturer,
            year_of_manufacture,
            week_of_manufacture,
            connection,
            built_in,
            active,
            scale: scale_factor,
            scales,
            hdr_status,
        });

        if let Some(last) = displays.last() {
            log::info!(
                "Monitor {}: '{}' primary={} pos=({}, {}) current={}x{}@{}Hz scale={} connection='{}' hdr_status={}",
                logical_display_index,
                last.friendly_name,
                last.is_primary,
                last.position_x,
                last.position_y,
                last.current.width,
                last.current.height,
                last.current.refresh_hz,
                last.scale,
                last.connection,
                last.hdr_status
            );
        }

        logical_display_index += 1;
        device_index += 1;
    }

    log::info!(
        "get_all_monitors_windows: done, total_monitors={}",
        displays.len()
    );
    Ok(displays)
}

fn set_monitor_resolution_windows(
    device_name: String,
    width: u32,
    height: u32,
    refresh_hz: Option<u32>,
) -> Result<(), String> {
    use std::mem::{size_of, zeroed};
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::Graphics::Gdi::{
        ChangeDisplaySettingsExW, EnumDisplaySettingsExW, DEVMODEW, DISP_CHANGE_SUCCESSFUL,
    };

    let mut mode_index: u32 = 0;
    let mut chosen: Option<DEVMODEW> = None;
    loop {
        let mut dm: DEVMODEW = unsafe { zeroed() };
        dm.dmSize = size_of::<DEVMODEW>() as u16;
        let wide = to_wide_null_terminated(&device_name);
        let ok: BOOL = unsafe {
            EnumDisplaySettingsExW(
                windows::core::PCWSTR(wide.as_ptr()),
                windows::Win32::Graphics::Gdi::ENUM_DISPLAY_SETTINGS_MODE(mode_index),
                &mut dm,
                windows::Win32::Graphics::Gdi::ENUM_DISPLAY_SETTINGS_FLAGS(0),
            )
        };
        if !ok.as_bool() {
            break;
        }

        let w = dm.dmPelsWidth;
        let h = dm.dmPelsHeight;
        let hz = dm.dmDisplayFrequency;
        if w == width && h == height {
            if let Some(target_hz) = refresh_hz {
                if hz == target_hz {
                    chosen = Some(dm);
                    break;
                }
            } else {
                chosen = match chosen {
                    None => Some(dm),
                    Some(prev) => {
                        if dm.dmDisplayFrequency > prev.dmDisplayFrequency {
                            Some(dm)
                        } else {
                            Some(prev)
                        }
                    }
                };
            }
        }

        mode_index += 1;
    }

    let mut devmode = chosen.ok_or_else(|| "Requested resolution not supported".to_string())?;

    let wide = to_wide_null_terminated(&device_name);
    let status = unsafe {
        ChangeDisplaySettingsExW(
            windows::core::PCWSTR(wide.as_ptr()),
            Some(&mut devmode),
            None,
            windows::Win32::Graphics::Gdi::CDS_TYPE(0),
            None,
        )
    };

    if status == DISP_CHANGE_SUCCESSFUL {
        Ok(())
    } else {
        Err(format!(
            "ChangeDisplaySettingsExW failed with code: {:?}",
            status
        ))
    }
}

fn set_monitor_orientation_windows(
    device_name: String,
    orientation_degrees: u32,
) -> Result<(), String> {
    use std::mem::{size_of, zeroed};
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::Graphics::Gdi::{
        ChangeDisplaySettingsExW, EnumDisplaySettingsExW, DEVMODEW, DISP_CHANGE_SUCCESSFUL,
        DM_DISPLAYORIENTATION, DM_PELSHEIGHT, DM_PELSWIDTH,
    };

    let mut dm: DEVMODEW = unsafe { zeroed() };
    dm.dmSize = size_of::<DEVMODEW>() as u16;
    let wide = to_wide_null_terminated(&device_name);
    let ok: BOOL = unsafe {
        EnumDisplaySettingsExW(
            windows::core::PCWSTR(wide.as_ptr()),
            windows::Win32::Graphics::Gdi::ENUM_CURRENT_SETTINGS,
            &mut dm,
            windows::Win32::Graphics::Gdi::ENUM_DISPLAY_SETTINGS_FLAGS(0),
        )
    };
    if !ok.as_bool() {
        return Err("Failed to read current display settings".to_string());
    }

    // Map degrees -> DMDO_* (0,1,2,3)
    let dmdo_value: u32 = match orientation_degrees % 360 {
        0 => 0,
        90 => 1,
        180 => 2,
        270 => 3,
        other => {
            return Err(format!(
                "Unsupported orientation degrees: {} (must be 0/90/180/270)",
                other
            ))
        }
    };

    // Windows expects width/height swapped for 90/270 if not already
    let is_rotated = dmdo_value == 1 || dmdo_value == 3;
    use windows::Win32::Graphics::Gdi::{DMDO_270, DMDO_90};
    let cur_orient = unsafe { dm.Anonymous1.Anonymous2.dmDisplayOrientation };
    let cur_is_rotated = cur_orient == DMDO_90 || cur_orient == DMDO_270;
    if is_rotated != cur_is_rotated {
        // swap width/height to keep content visible
        let tmp = dm.dmPelsWidth;
        dm.dmPelsWidth = dm.dmPelsHeight;
        dm.dmPelsHeight = tmp;
    }

    unsafe {
        use windows::Win32::Graphics::Gdi::{DMDO_180, DMDO_270, DMDO_90, DMDO_DEFAULT};
        dm.Anonymous1.Anonymous2.dmDisplayOrientation = match dmdo_value {
            0 => DMDO_DEFAULT,
            1 => DMDO_90,
            2 => DMDO_180,
            3 => DMDO_270,
            _ => DMDO_DEFAULT,
        };
    }

    // Ensure the fields are marked as valid
    dm.dmFields |= DM_DISPLAYORIENTATION | DM_PELSWIDTH | DM_PELSHEIGHT;

    let status = unsafe {
        ChangeDisplaySettingsExW(
            windows::core::PCWSTR(wide.as_ptr()),
            Some(&mut dm),
            None,
            windows::Win32::Graphics::Gdi::CDS_TYPE(0),
            None,
        )
    };

    if status == DISP_CHANGE_SUCCESSFUL {
        Ok(())
    } else {
        Err(format!(
            "ChangeDisplaySettingsExW failed with code: {:?}",
            status
        ))
    }
}
fn find_hmonitor_by_device_name(
    device_name: &str,
) -> Option<windows::Win32::Graphics::Gdi::HMONITOR> {
    use std::mem::zeroed;
    use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW,
    };

    struct Ctx {
        target: Vec<u16>,
        found: Option<HMONITOR>,
    }

    unsafe extern "system" fn enum_proc(
        hmonitor: HMONITOR,
        _hdc: HDC,
        _rc: *mut RECT,
        lparam: LPARAM,
    ) -> BOOL {
        let ctx = &mut *(lparam.0 as *mut Ctx);
        let mut mi: MONITORINFOEXW = zeroed();
        mi.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
        if GetMonitorInfoW(hmonitor, &mut mi as *mut _ as *mut _).as_bool() {
            let name = widestr_to_string(&mi.szDevice);
            let target =
                String::from_utf16_lossy(&ctx.target[..ctx.target.len().saturating_sub(1)]);
            if name == target {
                ctx.found = Some(hmonitor);
                return BOOL(0);
            }
        }
        BOOL(1)
    }

    let mut ctx = Ctx {
        target: to_wide_null_terminated(device_name),
        found: None,
    };
    unsafe {
        let _ = EnumDisplayMonitors(
            HDC(0),
            None,
            Some(enum_proc),
            LPARAM(&mut ctx as *mut _ as isize),
        );
    }
    ctx.found
}

fn with_first_physical_monitor<
    T,
    F: FnOnce(windows::Win32::Devices::Display::PHYSICAL_MONITOR) -> Result<T, String>,
>(
    device_name: &str,
    f: F,
) -> Result<T, String> {
    use windows::Win32::Devices::Display::{
        DestroyPhysicalMonitors, GetNumberOfPhysicalMonitorsFromHMONITOR,
        GetPhysicalMonitorsFromHMONITOR, PHYSICAL_MONITOR,
    };
    use windows::Win32::Graphics::Gdi::HMONITOR;

    let hmon: HMONITOR = find_hmonitor_by_device_name(device_name)
        .ok_or_else(|| "Monitor handle not found".to_string())?;

    let mut count: u32 = 0;
    match unsafe { GetNumberOfPhysicalMonitorsFromHMONITOR(hmon, &mut count) } {
        Ok(_) => {}
        Err(_) => return Err("GetNumberOfPhysicalMonitorsFromHMONITOR failed".to_string()),
    }
    if count == 0 {
        return Err("No physical monitors found or operation unsupported".to_string());
    }

    let mut vec: Vec<PHYSICAL_MONITOR> = vec![unsafe { std::mem::zeroed() }; count as usize];
    match unsafe { GetPhysicalMonitorsFromHMONITOR(hmon, &mut vec) } {
        Ok(_) => {}
        Err(_) => return Err("GetPhysicalMonitorsFromHMONITOR failed".to_string()),
    }

    let result = f(vec[0]);

    let _ = unsafe { DestroyPhysicalMonitors(&vec) };
    result
}

fn get_monitor_brightness_windows(device_name: String) -> Result<BrightnessInfo, String> {
    use windows::Win32::Devices::Display::GetMonitorBrightness;

    match with_first_physical_monitor(&device_name, |pm| {
        let mut min = 0u32;
        let mut cur = 0u32;
        let mut max = 0u32;
        let ok = unsafe { GetMonitorBrightness(pm.hPhysicalMonitor, &mut min, &mut cur, &mut max) };
        if ok == 0 {
            return Err("GetMonitorBrightness failed (monitor may not support DDC/CI)".to_string());
        }
        Ok(BrightnessInfo {
            min,
            current: cur,
            max,
        })
    }) {
        Ok(b) => Ok(b),
        Err(_e) => {
            // Fallback: WMI (internal displays)
            if let Some(b) = wmi_get_brightness_via_powershell() {
                Ok(b)
            } else {
                Err("Failed to get brightness via DDC/CI and WMI fallback".to_string())
            }
        }
    }
}

fn set_monitor_brightness_windows(device_name: String, percent: u32) -> Result<(), String> {
    use windows::Win32::Devices::Display::{GetMonitorBrightness, SetMonitorBrightness};

    let pct = percent.min(100);
    match with_first_physical_monitor(&device_name, |pm| {
        let mut min = 0u32;
        let mut cur = 0u32;
        let mut max = 0u32;
        let ok = unsafe { GetMonitorBrightness(pm.hPhysicalMonitor, &mut min, &mut cur, &mut max) };
        if ok == 0 || max < min {
            return Err("GetMonitorBrightness failed (monitor may not support DDC/CI)".to_string());
        }
        let span = max - min;
        let value = min + ((span as u64 * pct as u64 + 50) / 100) as u32;
        let ok = unsafe { SetMonitorBrightness(pm.hPhysicalMonitor, value) };
        if ok == 0 {
            return Err("SetMonitorBrightness failed".to_string());
        }
        Ok(())
    }) {
        Ok(()) => Ok(()),
        Err(_e) => {
            // Fallback to WMI (usually internal panel only)
            if wmi_set_brightness_via_powershell(pct) {
                Ok(())
            } else {
                Err("Failed to set brightness via DDC/CI and WMI fallback".to_string())
            }
        }
    }
}

// WMI brightness fallback helpers (Windows internal displays)
fn wmi_get_brightness_via_powershell() -> Option<BrightnessInfo> {
    let script = r#"
$inst = Get-CimInstance -Namespace root/WMI -Class WmiMonitorBrightness -ErrorAction SilentlyContinue |
  Where-Object { $_.Active -eq $true } |
  Select-Object -First 1
if ($inst) {
  [pscustomobject]@{ Current = [uint32]$inst.CurrentBrightness } | ConvertTo-Json -Compress
}
"#;
    let raw = run_powershell_hidden(script)?;
    if raw.trim().is_empty() {
        return None;
    }
    #[derive(Deserialize)]
    struct Js {
        #[serde(rename = "Current")]
        current: Option<u32>,
    }
    if let Ok(js) = serde_json::from_str::<Js>(&raw) {
        if let Some(cur) = js.current {
            return Some(BrightnessInfo {
                min: 0,
                current: cur.min(100),
                max: 100,
            });
        }
    }
    None
}

fn wmi_set_brightness_via_powershell(percent: u32) -> bool {
    let pct = percent.min(100);
    let script = format!(
        r#"$b = [byte]({pct});
$inst = Get-CimInstance -Namespace root/WMI -Class WmiMonitorBrightnessMethods -ErrorAction SilentlyContinue |
  Where-Object {{ $_.Active -eq $true }} | Select-Object -First 1
if ($inst) {{
  $r = Invoke-CimMethod -InputObject $inst -MethodName WmiSetBrightness -Arguments @{{ Timeout = 0; Brightness = $b }} -ErrorAction SilentlyContinue;
  if ($r -and ($r.ReturnValue -eq 0)) {{ 'OK' }}
}}"#
    );
    if let Some(out) = run_powershell_hidden(&script) {
        return out.trim().starts_with("OK");
    }
    false
}

fn get_monitor_scale_for_device(device_name: &str) -> f32 {
    // Try modern per-monitor DPI (Windows 8.1+)
    #[allow(unused_mut)]
    let mut scale: f32 = 1.0;
    #[cfg(windows)]
    {
        use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
        if let Some(hmon) = find_hmonitor_by_device_name(device_name) {
            let mut dpi_x: u32 = 0;
            let mut dpi_y: u32 = 0;
            // SAFETY: API fills the provided pointers
            if unsafe { GetDpiForMonitor(hmon, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) }.is_ok()
                && dpi_x > 0
            {
                scale = (dpi_x as f32) / 96.0;
                return scale.max(0.5).min(4.0);
            }
        }
        // Fallback: query device context LOGPIXELSX for the specific device
        use windows::Win32::Graphics::Gdi::{CreateDCW, DeleteDC, GetDeviceCaps, LOGPIXELSX};
        let wide = to_wide_null_terminated(device_name);
        let hdc = unsafe {
            CreateDCW(
                windows::core::PCWSTR(wide.as_ptr()),
                windows::core::PCWSTR(wide.as_ptr()),
                None,
                None,
            )
        };
        if !hdc.is_invalid() {
            let dpi = unsafe { GetDeviceCaps(hdc, LOGPIXELSX) } as i32;
            unsafe { DeleteDC(hdc) };
            if dpi > 0 {
                scale = (dpi as f32) / 96.0;
                return scale.max(0.5).min(4.0);
            }
        }
    }
    scale
}

#[cfg(windows)]
mod displayconfig_ffi {
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    use windows::Win32::Foundation::{BOOL, LUID};

    pub const QDC_ONLY_ACTIVE_PATHS: u32 = 0x00000002;
    pub const DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME: i32 = 1;
    pub const DISPLAYCONFIG_DEVICE_INFO_GET_DPI_SCALE: i32 = -3; // undocumented
    pub const DISPLAYCONFIG_DEVICE_INFO_SET_DPI_SCALE: i32 = -4; // undocumented

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    pub struct DISPLAYCONFIG_DEVICE_INFO_HEADER {
        pub r#type: i32,
        pub size: u32,
        pub adapterId: LUID,
        pub id: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    pub struct DISPLAYCONFIG_SOURCE_DEVICE_NAME {
        pub header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
        pub viewGdiDeviceName: [u16; 32],
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    pub struct DISPLAYCONFIG_RATIONAL {
        pub Numerator: u32,
        pub Denominator: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    pub struct DISPLAYCONFIG_PATH_SOURCE_INFO {
        pub adapterId: LUID,
        pub id: u32,
        pub modeInfoIdx: u32,
        pub statusFlags: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    pub struct DISPLAYCONFIG_PATH_TARGET_INFO {
        pub adapterId: LUID,
        pub id: u32,
        pub modeInfoIdx: u32,
        pub outputTechnology: u32,
        pub rotation: u32,
        pub scaling: u32,
        pub refreshRate: DISPLAYCONFIG_RATIONAL,
        pub scanLineOrdering: u32,
        pub targetAvailable: BOOL,
        pub statusFlags: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    pub struct DISPLAYCONFIG_PATH_INFO {
        pub sourceInfo: DISPLAYCONFIG_PATH_SOURCE_INFO,
        pub targetInfo: DISPLAYCONFIG_PATH_TARGET_INFO,
        pub flags: u32,
    }

    // Oversized union payload as in test.py to satisfy API
    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct DISPLAYCONFIG_MODE_INFO {
        pub infoType: u32,
        pub id: u32,
        pub adapterId: LUID,
        pub _blob: [u8; 128],
    }

    // Intentionally no Default impl: we construct instances using `zeroed()`

    #[link(name = "user32")]
    extern "system" {
        pub fn GetDisplayConfigBufferSizes(
            flags: u32,
            num_paths: *mut u32,
            num_modes: *mut u32,
        ) -> i32;

        pub fn QueryDisplayConfig(
            flags: u32,
            num_paths: *mut u32,
            path_info_array: *mut DISPLAYCONFIG_PATH_INFO,
            mode_info_array_count: *mut u32,
            mode_info_array: *mut DISPLAYCONFIG_MODE_INFO,
            current_topology_id: *mut core::ffi::c_void,
        ) -> i32;

        pub fn DisplayConfigGetDeviceInfo(packet: *mut DISPLAYCONFIG_DEVICE_INFO_HEADER) -> i32;
        pub fn DisplayConfigSetDeviceInfo(packet: *mut DISPLAYCONFIG_DEVICE_INFO_HEADER) -> i32;
    }
}

#[allow(dead_code)]
fn get_scales_for_device(device_name: &str) -> Result<Vec<ScaleInfo>, String> {
    use crate::winDisplays::displayconfig_ffi::*;
    use std::mem::{size_of, zeroed};
    use windows::Win32::Foundation::LUID;

    #[repr(C)]
    struct DpiScaleGetStruct {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
        min_scale_rel: i32,
        cur_scale_rel: i32,
        max_scale_rel: i32,
    }

    // Common Windows scaling steps
    const DPI_VALS: [i32; 12] = [100, 125, 150, 175, 200, 225, 250, 300, 350, 400, 450, 500];

    // Resolve adapterId + sourceId for the given device name
    let (adapter_id, source_id): (LUID, u32) = {
        let mut num_paths: u32 = 0;
        let mut num_modes: u32 = 0;
        unsafe {
            let rc =
                GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut num_paths, &mut num_modes);
            if rc != 0 {
                return Err(format!("GetDisplayConfigBufferSizes failed: {}", rc));
            }
        }

        let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> = vec![unsafe { zeroed() }; num_paths as usize];
        let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> = vec![unsafe { zeroed() }; num_modes as usize];
        unsafe {
            let rc = QueryDisplayConfig(
                QDC_ONLY_ACTIVE_PATHS,
                &mut num_paths,
                paths.as_mut_ptr(),
                &mut num_modes,
                modes.as_mut_ptr(),
                core::ptr::null_mut(),
            );
            if rc != 0 {
                return Err(format!("QueryDisplayConfig failed: {}", rc));
            }
        }

        let mut found: Option<(LUID, u32)> = None;
        for p in &paths {
            let mut src_name: DISPLAYCONFIG_SOURCE_DEVICE_NAME = unsafe { zeroed() };
            src_name.header.size = size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
            src_name.header.adapterId = p.sourceInfo.adapterId;
            src_name.header.id = p.sourceInfo.id;
            // SAFETY: using documented GET_SOURCE_NAME
            src_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
            let rc = unsafe { DisplayConfigGetDeviceInfo(&mut src_name.header) };
            if rc != 0 {
                continue;
            }
            let name = widestr_to_string(&src_name.viewGdiDeviceName);
            if name == device_name {
                found = Some((p.sourceInfo.adapterId, p.sourceInfo.id));
                break;
            }
        }
        found.ok_or_else(|| "Could not map device name to DisplayConfig source".to_string())?
    };

    // Build GET packet
    let mut pkt = DpiScaleGetStruct {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
            r#type: DISPLAYCONFIG_DEVICE_INFO_GET_DPI_SCALE, // undocumented
            size: size_of::<DpiScaleGetStruct>() as u32,
            adapterId: adapter_id,
            id: source_id,
        },
        min_scale_rel: 0,
        cur_scale_rel: 0,
        max_scale_rel: 0,
    };

    let rc = unsafe { DisplayConfigGetDeviceInfo(&mut pkt.header) };
    if rc != 0 {
        return Err(format!(
            "DisplayConfigGetDeviceInfo(GET_DPI) failed: {}",
            rc
        ));
    }

    // Relative semantics:
    // - min_scale_rel: steps DOWN from recommended to reach minimum (100%).
    // - cur_scale_rel: steps from recommended to current.
    // - max_scale_rel: steps UP from recommended to maximum.
    // So:
    // recommended_idx = -min_scale_rel
    // min_idx = recommended_idx + min_scale_rel (usually 0)
    // max_idx = recommended_idx + max_scale_rel
    let recommended_idx_i32 = -pkt.min_scale_rel;
    let recommended_idx = recommended_idx_i32.clamp(0, (DPI_VALS.len() as i32) - 1) as usize;

    let mut min_idx = (recommended_idx_i32 + pkt.min_scale_rel).max(0) as usize;
    let mut max_idx =
        (recommended_idx_i32 + pkt.max_scale_rel).min((DPI_VALS.len() as i32) - 1) as usize;
    if min_idx > max_idx {
        core::mem::swap(&mut min_idx, &mut max_idx);
    }

    let mut out: Vec<ScaleInfo> = Vec::new();
    for idx in min_idx..=max_idx {
        let scale = (DPI_VALS[idx] as f32) / 100.0;
        out.push(ScaleInfo {
            scale,
            is_recommended: idx == recommended_idx,
        });
    }
    Ok(out)
}

fn set_monitor_scale_windows(device_name: &str, scale_percent: u32) -> Result<(), String> {
    use crate::winDisplays::displayconfig_ffi::*;
    use std::mem::{size_of, zeroed};
    use windows::Win32::Foundation::LUID;

    #[repr(C)]
    struct DpiScaleGetStruct {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
        min_scale_rel: i32,
        cur_scale_rel: i32,
        max_scale_rel: i32,
    }

    #[repr(C)]
    struct DpiScaleSetStruct {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
        scale_rel: i32,
    }

    const DPI_VALS: [i32; 12] = [100, 125, 150, 175, 200, 225, 250, 300, 350, 400, 450, 500];

    // Validate input
    let target_idx = DPI_VALS
        .iter()
        .position(|v| *v == scale_percent as i32)
        .ok_or_else(|| {
            format!(
                "Unsupported DPI {}%. Supported: {:?}",
                scale_percent, DPI_VALS
            )
        })? as i32;

    // Resolve adapterId + sourceId for the given device name
    let (adapter_id, source_id): (LUID, u32) = {
        let mut num_paths: u32 = 0;
        let mut num_modes: u32 = 0;
        unsafe {
            let rc =
                GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut num_paths, &mut num_modes);
            if rc != 0 {
                return Err(format!("GetDisplayConfigBufferSizes failed: {}", rc));
            }
        }

        let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> = vec![unsafe { zeroed() }; num_paths as usize];
        let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> = vec![unsafe { zeroed() }; num_modes as usize];
        unsafe {
            let rc = QueryDisplayConfig(
                QDC_ONLY_ACTIVE_PATHS,
                &mut num_paths,
                paths.as_mut_ptr(),
                &mut num_modes,
                modes.as_mut_ptr(),
                core::ptr::null_mut(),
            );
            if rc != 0 {
                return Err(format!("QueryDisplayConfig failed: {}", rc));
            }
        }

        let mut found: Option<(LUID, u32)> = None;
        for p in &paths {
            let mut src_name: DISPLAYCONFIG_SOURCE_DEVICE_NAME = unsafe { zeroed() };
            src_name.header.size = size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
            src_name.header.adapterId = p.sourceInfo.adapterId;
            src_name.header.id = p.sourceInfo.id;
            src_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
            let rc = unsafe { DisplayConfigGetDeviceInfo(&mut src_name.header) };
            if rc != 0 {
                continue;
            }
            let name = widestr_to_string(&src_name.viewGdiDeviceName);
            if name == device_name {
                found = Some((p.sourceInfo.adapterId, p.sourceInfo.id));
                break;
            }
        }
        found.ok_or_else(|| "Could not map device name to DisplayConfig source".to_string())?
    };

    // Fetch recommended index via GET packet
    let mut get_pkt = DpiScaleGetStruct {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
            r#type: DISPLAYCONFIG_DEVICE_INFO_GET_DPI_SCALE,
            size: size_of::<DpiScaleGetStruct>() as u32,
            adapterId: adapter_id,
            id: source_id,
        },
        min_scale_rel: 0,
        cur_scale_rel: 0,
        max_scale_rel: 0,
    };
    let rc = unsafe { DisplayConfigGetDeviceInfo(&mut get_pkt.header) };
    if rc != 0 {
        return Err(format!(
            "DisplayConfigGetDeviceInfo(GET_DPI) failed: {}",
            rc
        ));
    }
    // Compute recommended and bounds from relative values
    let recommended_idx: i32 = -get_pkt.min_scale_rel;
    let min_idx: i32 = (recommended_idx + get_pkt.min_scale_rel).max(0);
    let max_idx: i32 = (recommended_idx + get_pkt.max_scale_rel).min((DPI_VALS.len() as i32) - 1);

    if target_idx < min_idx || target_idx > max_idx {
        return Err(format!(
            "Unsupported DPI {}% for this display. Supported range: {}%-{}%",
            scale_percent, DPI_VALS[min_idx as usize], DPI_VALS[max_idx as usize]
        ));
    }

    let rel = target_idx - recommended_idx;

    // Build SET packet
    let mut set_pkt = DpiScaleSetStruct {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
            r#type: DISPLAYCONFIG_DEVICE_INFO_SET_DPI_SCALE, // undocumented
            size: size_of::<DpiScaleSetStruct>() as u32,
            adapterId: adapter_id,
            id: source_id,
        },
        scale_rel: rel,
    };
    let rc = unsafe { DisplayConfigSetDeviceInfo(&mut set_pkt.header) };
    if rc != 0 {
        return Err(format!(
            "DisplayConfigSetDeviceInfo(SET_DPI) failed: {}",
            rc
        ));
    }
    Ok(())
}

fn widestr_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

fn to_wide_null_terminated(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn identify_monitors_windows(_app_handle: tauri::AppHandle) -> Result<(), String> {
    use std::mem::zeroed;
    use windows::Win32::Foundation::{COLORREF, HINSTANCE, HWND, LRESULT, RECT};
    use windows::Win32::Graphics::Gdi::{
        BeginPaint, CreateFontW, DeleteObject, EndPaint, SelectObject, SetBkMode, SetTextColor,
        TextOutW, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DEFAULT_QUALITY, FW_BOLD,
        HFONT, OUT_TT_PRECIS, PAINTSTRUCT, TRANSPARENT,
    };
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, DispatchMessageW, GetClientRect, IsWindow, LoadCursorW,
        PeekMessageW, RegisterClassW, SetLayeredWindowAttributes, SetTimer, SetWindowLongPtrW,
        SetWindowPos, ShowWindow, TranslateMessage, UnregisterClassW, CS_HREDRAW, CS_VREDRAW,
        GWLP_USERDATA, HWND_TOPMOST, IDC_ARROW, LWA_ALPHA, MSG, PM_REMOVE, SWP_NOACTIVATE,
        SWP_NOMOVE, SWP_NOSIZE, SW_SHOW, WM_CREATE, WM_DESTROY, WM_PAINT, WM_QUIT, WM_TIMER,
        WNDCLASSW, WS_EX_LAYERED, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
    };

    let monitors = get_all_monitors_windows()?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let class_name = to_wide_null_terminated(&format!("MonitorIdentifierOverlay_{}", timestamp));
    let mut wc: WNDCLASSW = unsafe { zeroed() };
    wc.style = CS_HREDRAW | CS_VREDRAW;
    wc.lpfnWndProc = Some(overlay_window_proc);
    wc.hInstance = HINSTANCE(unsafe { GetModuleHandleW(None).unwrap_or_default().0 });
    wc.hCursor = unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() };
    wc.lpszClassName = windows::core::PCWSTR(class_name.as_ptr());

    unsafe {
        if RegisterClassW(&wc) == 0 {
            // Could not register the class; behave as a no-op
            return Ok(());
        }
    }

    let mut overlay_windows = Vec::new();

    for (index, monitor) in monitors.iter().enumerate() {
        let monitor_number = index + 1;
        let window_title = to_wide_null_terminated(&format!("Monitor {} Overlay", monitor_number));

        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW,
                windows::core::PCWSTR(class_name.as_ptr()),
                windows::core::PCWSTR(window_title.as_ptr()),
                WS_POPUP,
                monitor.position_x,
                monitor.position_y,
                monitor.current.width as i32,
                monitor.current.height as i32,
                HWND(0),
                None,
                GetModuleHandleW(None).unwrap_or_default(),
                Some(Box::into_raw(Box::new(monitor_number)) as *const core::ffi::c_void),
            )
        };

        if hwnd.0 == 0 {
            continue;
        }

        unsafe {
            SetLayeredWindowAttributes(hwnd, COLORREF(0), 150, LWA_ALPHA);
            SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
            ShowWindow(hwnd, SW_SHOW);
            SetTimer(hwnd, 1, 2000, None);
        }

        overlay_windows.push(hwnd);
    }

    let mut msg: MSG = unsafe { zeroed() };
    let mut windows_active = true;
    while windows_active {
        if unsafe { PeekMessageW(&mut msg, HWND(0), 0, 0, PM_REMOVE) }.as_bool() {
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            if msg.message == WM_QUIT {
                windows_active = false;
            }
        } else {
            let mut any_alive = false;
            for &window in &overlay_windows {
                if unsafe { IsWindow(window) }.as_bool() {
                    any_alive = true;
                    break;
                }
            }
            if !any_alive {
                windows_active = false;
            } else {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    }

    unsafe {
        UnregisterClassW(
            windows::core::PCWSTR(class_name.as_ptr()),
            GetModuleHandleW(None).unwrap_or_default(),
        );
    }

    Ok(())
}

unsafe extern "system" fn overlay_window_proc(
    hwnd: windows::Win32::Foundation::HWND,
    msg: u32,
    _wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::Foundation::{COLORREF, LRESULT, RECT};
    use windows::Win32::Graphics::Gdi::{
        BeginPaint, CreateFontW, DeleteObject, EndPaint, SelectObject, SetBkMode, SetTextColor,
        TextOutW, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET, DEFAULT_PITCH, DEFAULT_QUALITY, FW_BOLD,
        HFONT, OUT_TT_PRECIS, PAINTSTRUCT, TRANSPARENT,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        DestroyWindow, GetClientRect, GetWindowLongPtrW, KillTimer, SetWindowLongPtrW,
        GWLP_USERDATA, WM_CREATE, WM_DESTROY, WM_PAINT, WM_TIMER,
    };

    match msg {
        WM_CREATE => {
            let create_struct =
                lparam.0 as *const windows::Win32::UI::WindowsAndMessaging::CREATESTRUCTW;
            if !create_struct.is_null() {
                let monitor_number_ptr = unsafe { (*create_struct).lpCreateParams as *mut i32 };
                if !monitor_number_ptr.is_null() {
                    let monitor_number = unsafe { *monitor_number_ptr };
                    unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, monitor_number as isize) };
                    let _ = unsafe { Box::from_raw(monitor_number_ptr) };
                }
            }
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps: PAINTSTRUCT = std::mem::zeroed();
            let hdc = unsafe { BeginPaint(hwnd, &mut ps) };
            if !hdc.is_invalid() {
                let monitor_number = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as i32 };
                let font_name = to_wide_null_terminated("Arial");
                let hfont: HFONT = unsafe {
                    CreateFontW(
                        120,
                        0,
                        0,
                        0,
                        FW_BOLD.0 as i32,
                        0,
                        0,
                        0,
                        DEFAULT_CHARSET.0 as u32,
                        OUT_TT_PRECIS.0 as u32,
                        CLIP_DEFAULT_PRECIS.0 as u32,
                        DEFAULT_QUALITY.0 as u32,
                        DEFAULT_PITCH.0 as u32,
                        windows::core::PCWSTR(font_name.as_ptr()),
                    )
                };
                let old_font = unsafe { SelectObject(hdc, hfont) };
                unsafe { SetBkMode(hdc, TRANSPARENT) };
                unsafe { SetTextColor(hdc, COLORREF(0x00FFFFFF)) };
                let mut rect: RECT = std::mem::zeroed();
                unsafe { GetClientRect(hwnd, &mut rect) };
                let text = format!("Monitor {}", monitor_number);
                let text_wide = to_wide_null_terminated(&text);
                let text_width = text.len() as i32 * 60;
                let text_height = 120;
                let x = (rect.right - text_width) / 2;
                let y = (rect.bottom - text_height) / 2;
                unsafe { TextOutW(hdc, x, y, &text_wide[..text_wide.len() - 1]) };
                unsafe { SelectObject(hdc, old_font) };
                unsafe { DeleteObject(hfont) };
                unsafe { EndPaint(hwnd, &ps) };
            }
            LRESULT(0)
        }
        WM_TIMER => {
            unsafe { KillTimer(hwnd, 1) };
            unsafe { DestroyWindow(hwnd) };
            LRESULT(0)
        }
        WM_DESTROY => LRESULT(0),
        _ => unsafe {
            windows::Win32::UI::WindowsAndMessaging::DefWindowProcW(
                hwnd,
                msg,
                windows::Win32::Foundation::WPARAM(0),
                lparam,
            )
        },
    }
}
