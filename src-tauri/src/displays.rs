use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
    pub bits_per_pixel: u32,
    pub refresh_hz: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct DisplayInfo {
    pub device_name: String,   // e.g. "\\\\.\\DISPLAY1"
    pub friendly_name: String, // e.g. monitor model
    pub is_primary: bool,
    pub position_x: i32,
    pub position_y: i32,
    pub current: Resolution,
    pub modes: Vec<Resolution>,
}

#[tauri::command]
pub fn get_all_monitors() -> Result<Vec<DisplayInfo>, String> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "windows")] {
            get_all_monitors_windows()
        } else {
            Ok(Vec::new())
        }
    }
}

#[tauri::command]
pub fn set_monitor_resolution(
    device_name: String,
    width: u32,
    height: u32,
    refresh_hz: Option<u32>,
) -> Result<(), String> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "windows")] {
            set_monitor_resolution_windows(device_name, width, height, refresh_hz)
        } else {
            Err("Unsupported platform".to_string())
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct BrightnessInfo {
    pub min: u32,
    pub current: u32,
    pub max: u32,
}

#[tauri::command]
pub fn get_monitor_brightness(device_name: String) -> Result<BrightnessInfo, String> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "windows")] {
            get_monitor_brightness_windows(device_name)
        } else {
            Err("Unsupported platform".to_string())
        }
    }
}

#[tauri::command]
pub fn set_monitor_brightness(device_name: String, percent: u32) -> Result<(), String> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "windows")] {
            set_monitor_brightness_windows(device_name, percent)
        } else {
            Err("Unsupported platform".to_string())
        }
    }
}

#[cfg(target_os = "windows")]
fn get_all_monitors_windows() -> Result<Vec<DisplayInfo>, String> {
    use std::mem::{size_of, zeroed};
    use windows::Win32::Foundation::BOOL;
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayDevicesW, EnumDisplaySettingsExW, DEVMODEW, DISPLAY_DEVICEW,
        DISPLAY_DEVICE_ATTACHED_TO_DESKTOP, DISPLAY_DEVICE_MIRRORING_DRIVER,
        DISPLAY_DEVICE_PRIMARY_DEVICE,
    };

    let mut displays: Vec<DisplayInfo> = Vec::new();

    let mut device_index: u32 = 0;
    loop {
        let mut dd: DISPLAY_DEVICEW = unsafe { zeroed() };
        dd.cb = size_of::<DISPLAY_DEVICEW>() as u32;
        let ok: BOOL = unsafe { EnumDisplayDevicesW(None, device_index, &mut dd, 0) };
        if !ok.as_bool() {
            break;
        }

        // Skip mirror drivers or devices not attached to the desktop
        let state = dd.StateFlags;
        let is_attached = (state & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP) != 0;
        let is_mirror = (state & DISPLAY_DEVICE_MIRRORING_DRIVER) != 0;
        if !is_attached || is_mirror {
            device_index += 1;
            continue;
        }

        let device_name = widestr_to_string(&dd.DeviceName);
        let friendly_name = widestr_to_string(&dd.DeviceString);
        let is_primary = (state & DISPLAY_DEVICE_PRIMARY_DEVICE) != 0;

        // Current mode
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
            device_index += 1;
            continue;
        }

        let (pos_x, pos_y) = unsafe {
            // Position is valid when DM_POSITION flag is present, but for current settings it usually is.
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

        // Enumerate all modes
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
                break;
            }

            let res = Resolution {
                width: dm.dmPelsWidth as u32,
                height: dm.dmPelsHeight as u32,
                bits_per_pixel: dm.dmBitsPerPel as u32,
                refresh_hz: dm.dmDisplayFrequency as u32,
            };

            // Deduplicate by width/height/bpp/refresh
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

        displays.push(DisplayInfo {
            device_name,
            friendly_name,
            is_primary,
            position_x: pos_x,
            position_y: pos_y,
            current,
            modes,
        });

        device_index += 1;
    }

    Ok(displays)
}

#[cfg(target_os = "windows")]
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

    // Find a matching mode
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
                // If refresh rate not specified, prefer current or highest
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

    // Apply the mode
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

#[cfg(target_os = "windows")]
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
                return BOOL(0); // FALSE to stop enumerating
            }
        }
        BOOL(1) // TRUE to continue
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

#[cfg(target_os = "windows")]
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

    // Use the first physical monitor
    let result = f(vec[0]);

    let _ = unsafe { DestroyPhysicalMonitors(&vec) };
    result
}

#[cfg(target_os = "windows")]
fn get_monitor_brightness_windows(device_name: String) -> Result<BrightnessInfo, String> {
    use windows::Win32::Devices::Display::GetMonitorBrightness;

    with_first_physical_monitor(&device_name, |pm| {
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
    })
}

#[cfg(target_os = "windows")]
fn set_monitor_brightness_windows(device_name: String, percent: u32) -> Result<(), String> {
    println!(
        "set_monitor_brightness_windows: {:?}, {:?}",
        device_name, percent
    );
    use windows::Win32::Devices::Display::{GetMonitorBrightness, SetMonitorBrightness};

    let pct = percent.min(100);
    with_first_physical_monitor(&device_name, |pm| {
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
    })
}

#[cfg(target_os = "windows")]
fn widestr_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

#[cfg(target_os = "windows")]
fn to_wide_null_terminated(s: &str) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}
