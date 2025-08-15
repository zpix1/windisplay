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
    pub device_name: String,
    pub friendly_name: String,
    pub is_primary: bool,
    pub position_x: i32,
    pub position_y: i32,
    // Orientation in degrees: 0, 90, 180, 270
    pub orientation: u32,
    pub current: Resolution,
    pub modes: Vec<Resolution>,
    pub max_native: Resolution,
}

#[derive(Debug, Serialize, Clone)]
pub struct BrightnessInfo {
    pub min: u32,
    pub current: u32,
    pub max: u32,
}

pub trait Displays {
    fn get_all_monitors(&self) -> Result<Vec<DisplayInfo>, String>;
    fn set_monitor_resolution(
        &self,
        device_name: String,
        width: u32,
        height: u32,
        refresh_hz: Option<u32>,
    ) -> Result<(), String>;
    fn set_monitor_orientation(&self, device_name: String, orientation_degrees: u32)
        -> Result<(), String>;
    fn get_monitor_brightness(&self, device_name: String) -> Result<BrightnessInfo, String>;
    fn set_monitor_brightness(&self, device_name: String, percent: u32) -> Result<(), String>;
    fn identify_monitors(&self, app_handle: tauri::AppHandle) -> Result<(), String>;
}

fn active_provider() -> Box<dyn Displays> {
    #[cfg(feature = "fake-displays")]
    {
        return Box::new(crate::fakeDisplays::FakeDisplays::new());
    }
    #[cfg(all(not(feature = "fake-displays"), target_os = "windows"))]
    {
        return Box::new(crate::winDisplays::WinDisplays::new());
    }
    #[cfg(all(not(feature = "fake-displays"), not(target_os = "windows")))]
    {
        return Box::new(crate::fakeDisplays::FakeDisplays::new());
    }
}

// Tauri commands delegate to the selected provider
#[tauri::command]
pub fn get_all_monitors() -> Result<Vec<DisplayInfo>, String> {
    active_provider().get_all_monitors()
}

#[tauri::command]
pub fn set_monitor_resolution(
    device_name: String,
    width: u32,
    height: u32,
    refresh_hz: Option<u32>,
) -> Result<(), String> {
    active_provider().set_monitor_resolution(device_name, width, height, refresh_hz)
}

#[tauri::command]
pub fn set_monitor_orientation(device_name: String, orientation_degrees: u32) -> Result<(), String> {
    active_provider().set_monitor_orientation(device_name, orientation_degrees)
}

#[tauri::command]
pub fn get_monitor_brightness(device_name: String) -> Result<BrightnessInfo, String> {
    active_provider().get_monitor_brightness(device_name)
}

#[tauri::command]
pub fn set_monitor_brightness(device_name: String, percent: u32) -> Result<(), String> {
    active_provider().set_monitor_brightness(device_name, percent)
}

#[tauri::command]
pub async fn identify_monitors(app_handle: tauri::AppHandle) -> Result<(), String> {
    active_provider().identify_monitors(app_handle)
}
