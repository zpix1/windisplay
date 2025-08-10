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
