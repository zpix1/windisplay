#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]

// Rust implementation of HDR control functionality analogous to src-tauri/src/hdr.cpp
// Uses DisplayConfig device info packets to query and toggle advanced color (HDR) state.

use std::mem::{size_of, zeroed};

#[cfg(windows)]
mod displayconfig_ffi_hdr {
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    use windows::Win32::Foundation::LUID;

    pub const QDC_ONLY_ACTIVE_PATHS: u32 = 0x00000002;

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
    pub struct DISPLAYCONFIG_PATH_SOURCE_INFO {
        pub adapterId: LUID,
        pub id: u32,
        pub modeInfoIdx: u32,
        pub statusFlags: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    pub struct DISPLAYCONFIG_RATIONAL {
        pub Numerator: u32,
        pub Denominator: u32,
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
        pub targetAvailable: windows::Win32::Foundation::BOOL,
        pub statusFlags: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    pub struct DISPLAYCONFIG_PATH_INFO {
        pub sourceInfo: DISPLAYCONFIG_PATH_SOURCE_INFO,
        pub targetInfo: DISPLAYCONFIG_PATH_TARGET_INFO,
        pub flags: u32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct DISPLAYCONFIG_MODE_INFO {
        pub infoType: u32,
        pub id: u32,
        pub adapterId: LUID,
        pub _blob: [u8; 128],
    }

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

use displayconfig_ffi_hdr::{
    DisplayConfigGetDeviceInfo, DisplayConfigSetDeviceInfo, GetDisplayConfigBufferSizes,
    QueryDisplayConfig, DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_MODE_INFO,
    DISPLAYCONFIG_PATH_INFO, QDC_ONLY_ACTIVE_PATHS,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Unsupported = 0,
    Off = 1,
    On = 2,
}

#[derive(Clone, Debug)]
pub struct Display {
    pub name: String,
    pub status: Status,
}

// DISPLAYCONFIG_DEVICE_INFO_TYPE constants (subset)
const DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME: i32 = 2;
const DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_BASE_TYPE: i32 = 6;
const DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO: i32 = 9;
const DISPLAYCONFIG_DEVICE_INFO_SET_ADVANCED_COLOR_STATE: i32 = 10;

// DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY constants (subset)
const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_OTHER: u32 = 0xFFFFFFFF; // (uint32)-1
const DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL: u32 = 0x80000000; // bit-flag for internal

#[repr(C)]
struct DISPLAYCONFIG_TARGET_DEVICE_NAME {
    header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
    flags: u32,
    outputTechnology: u32,
    edidManufactureId: u16,
    edidProductCodeId: u16,
    connectorInstance: u32,
    monitorFriendlyDeviceName: [u16; 64],
    monitorDevicePath: [u16; 128],
}

#[repr(C)]
struct DISPLAYCONFIG_TARGET_BASE_TYPE {
    header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
    baseOutputTechnology: u32,
}

// Per Windows headers, this struct contains a union with bitfields. We map it as a u32 bitmask.
// Bit 0 = advancedColorSupported, Bit 1 = advancedColorEnabled.
#[repr(C)]
struct DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO_RUST {
    header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
    colorInfo: u32,
    colorEncoding: u32,
    bitsPerColorChannel: u32,
}

#[repr(C)]
struct DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE_RUST {
    header: DISPLAYCONFIG_DEVICE_INFO_HEADER,
    enableAdvancedColor: u32,
}

fn widestr_to_string(buf: &[u16]) -> String {
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

fn for_each_display<F: FnMut(&DISPLAYCONFIG_MODE_INFO)>(mut f: F) {
    let mut path_count: u32 = 0;
    let mut mode_count: u32 = 0;
    unsafe {
        if GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count) != 0
        {
            return;
        }
    }

    let mut paths: Vec<DISPLAYCONFIG_PATH_INFO> = vec![unsafe { zeroed() }; path_count as usize];
    let mut modes: Vec<DISPLAYCONFIG_MODE_INFO> = vec![unsafe { zeroed() }; mode_count as usize];
    unsafe {
        if QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_count,
            paths.as_mut_ptr(),
            &mut mode_count,
            modes.as_mut_ptr(),
            core::ptr::null_mut(),
        ) != 0
        {
            return;
        }
    }

    for path in &paths {
        let mode_idx = path.targetInfo.modeInfoIdx as usize;
        if mode_idx < modes.len() {
            f(&modes[mode_idx]);
        }
    }
}

fn get_display_hdr_status(mode: &DISPLAYCONFIG_MODE_INFO) -> Status {
    let mut get_info: DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO_RUST = unsafe { zeroed() };
    get_info.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_ADVANCED_COLOR_INFO;
    get_info.header.size = size_of::<DISPLAYCONFIG_GET_ADVANCED_COLOR_INFO_RUST>() as u32;
    get_info.header.adapterId = mode.adapterId;
    get_info.header.id = mode.id;

    unsafe {
        if DisplayConfigGetDeviceInfo(&mut get_info.header) != 0 {
            return Status::Unsupported;
        }
    }

    let supported = (get_info.colorInfo & 0x1) != 0;
    if !supported {
        return Status::Unsupported;
    }
    let enabled = (get_info.colorInfo & 0x2) != 0;
    if enabled {
        Status::On
    } else {
        Status::Off
    }
}

pub fn get_windows_hdr_status() -> Status {
    let mut any_supported = false;
    let mut any_enabled = false;

    for_each_display(|mode| {
        let s = get_display_hdr_status(mode);
        any_supported |= s != Status::Unsupported;
        any_enabled |= s == Status::On;
    });

    if any_supported {
        if any_enabled {
            Status::On
        } else {
            Status::Off
        }
    } else {
        Status::Unsupported
    }
}

fn set_display_hdr_status(mode: &DISPLAYCONFIG_MODE_INFO, enable: bool) -> Option<Status> {
    // If not supported, skip
    if get_display_hdr_status(mode) == Status::Unsupported {
        return None;
    }

    let mut set_info: DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE_RUST = unsafe { zeroed() };
    set_info.header.r#type = DISPLAYCONFIG_DEVICE_INFO_SET_ADVANCED_COLOR_STATE;
    set_info.header.size = size_of::<DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE_RUST>() as u32;
    set_info.header.adapterId = mode.adapterId;
    set_info.header.id = mode.id;
    set_info.enableAdvancedColor = if enable { 1 } else { 0 };

    unsafe {
        if DisplayConfigSetDeviceInfo(&mut set_info.header) != 0 {
            return None;
        }
    }

    Some(get_display_hdr_status(mode))
}

pub fn set_windows_hdr_status(enable: bool) -> Option<Status> {
    let mut result: Option<Status> = None;
    for_each_display(|mode| {
        if let Some(new_status) = set_display_hdr_status(mode, enable) {
            result = match result {
                None => Some(new_status),
                Some(prev) => Some(if (prev as i32) > (new_status as i32) {
                    prev
                } else {
                    new_status
                }),
            };
        }
    });
    result
}

pub fn toggle_hdr_status() -> Option<Status> {
    match get_windows_hdr_status() {
        Status::Unsupported => Some(Status::Unsupported),
        Status::Off => set_windows_hdr_status(true),
        Status::On => set_windows_hdr_status(false),
    }
}

pub fn set_hdr_status_by_index(target_index: usize, enable: bool) -> Option<Status> {
    let mut idx: usize = 0;
    let mut result: Option<Status> = None;
    for_each_display(|mode| {
        if idx == target_index {
            result = set_display_hdr_status(mode, enable);
        }
        idx += 1;
    });
    result
}

fn get_fallback_display_name(mode: &DISPLAYCONFIG_MODE_INFO) -> String {
    let mut base: DISPLAYCONFIG_TARGET_BASE_TYPE = unsafe { zeroed() };
    base.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_BASE_TYPE;
    base.header.size = size_of::<DISPLAYCONFIG_TARGET_BASE_TYPE>() as u32;
    base.header.adapterId = mode.adapterId;
    base.header.id = mode.id;
    unsafe {
        if DisplayConfigGetDeviceInfo(&mut base.header) == 0 {
            if base.baseOutputTechnology != DISPLAYCONFIG_OUTPUT_TECHNOLOGY_OTHER
                && (base.baseOutputTechnology & DISPLAYCONFIG_OUTPUT_TECHNOLOGY_INTERNAL) != 0
            {
                return "Internal Display".to_string();
            }
        }
    }
    "Unnamed".to_string()
}

pub fn get_displays() -> Vec<Display> {
    let mut out: Vec<Display> = Vec::new();
    for_each_display(|mode| {
        let status = get_display_hdr_status(mode);

        let mut dev_name: DISPLAYCONFIG_TARGET_DEVICE_NAME = unsafe { zeroed() };
        dev_name.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
        dev_name.header.size = size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
        dev_name.header.adapterId = mode.adapterId;
        dev_name.header.id = mode.id;
        let name = unsafe {
            if DisplayConfigGetDeviceInfo(&mut dev_name.header) == 0 {
                let friendly = widestr_to_string(&dev_name.monitorFriendlyDeviceName);
                if !friendly.is_empty() {
                    friendly
                } else {
                    get_fallback_display_name(mode)
                }
            } else {
                get_fallback_display_name(mode)
            }
        };

        out.push(Display { name, status });
    });
    out
}
