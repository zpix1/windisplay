use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
    pub bits_per_pixel: u32,
    pub refresh_hz: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct ScaleInfo {
    pub scale: f32,
    pub is_recommended: bool,
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
    pub model: String,
    pub serial: String,
    pub manufacturer: String,
    pub year_of_manufacture: u32,
    pub week_of_manufacture: u32,
    pub connection: String,
    pub built_in: bool,
    pub active: bool,
    // Whether the monitor is powered on
    pub enabled: bool,

    pub scale: f32,
    pub scales: Vec<ScaleInfo>,
    // HDR status: "unsupported", "on", "off"
    pub hdr_status: String,
    // Whether DDC/CI input switch (VCP 0x60) appears supported for this monitor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports_input_switch: Option<bool>,
}

#[derive(Debug, Serialize, Clone)]
pub struct BrightnessInfo {
    pub min: u32,
    pub current: u32,
    pub max: u32,
}

pub trait Displays {
    fn get_all_monitors(&self) -> Result<Vec<DisplayInfo>, String>;
    fn get_all_monitors_short(&self) -> Result<Vec<String>, String>;
    fn set_monitor_resolution(
        &self,
        device_name: String,
        width: u32,
        height: u32,
        refresh_hz: Option<u32>,
    ) -> Result<(), String>;
    fn set_monitor_orientation(
        &self,
        device_name: String,
        orientation_degrees: u32,
    ) -> Result<(), String>;
    fn get_monitor_brightness(&self, device_name: String) -> Result<BrightnessInfo, String>;
    fn set_monitor_brightness(&self, device_name: String, percent: u32) -> Result<(), String>;
    fn identify_monitors(&self, app_handle: tauri::AppHandle) -> Result<(), String>;
    fn set_monitor_scale(&self, device_name: String, scale_percent: u32) -> Result<(), String>;
    fn enable_hdr(&self, device_name: String, enable: bool) -> Result<(), String>;
    fn set_monitor_input_source(&self, device_name: String, input: String) -> Result<(), String>;
    fn get_monitor_input_source(&self, device_name: String) -> Result<String, String>;
    fn get_monitor_ddc_caps(&self, device_name: String) -> Result<String, String>;
    fn set_monitor_power(&self, device_name: String, power_on: bool) -> Result<(), String>;
}

pub fn active_provider() -> Box<dyn Displays> {
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

async fn run_display_task<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|e| format!("Display task failed: {e}"))?
}

// Tauri commands delegate to the selected provider
#[tauri::command]
pub async fn get_all_monitors() -> Result<Vec<DisplayInfo>, String> {
    run_display_task(|| active_provider().get_all_monitors()).await
}

#[tauri::command]
pub async fn set_monitor_resolution(
    device_name: String,
    width: u32,
    height: u32,
    refresh_hz: Option<u32>,
) -> Result<(), String> {
    run_display_task(move || {
        active_provider().set_monitor_resolution(device_name, width, height, refresh_hz)
    })
    .await
}

#[tauri::command]
pub async fn set_monitor_orientation(
    device_name: String,
    orientation_degrees: u32,
) -> Result<(), String> {
    run_display_task(move || {
        active_provider().set_monitor_orientation(device_name, orientation_degrees)
    })
    .await
}

#[tauri::command]
pub async fn get_monitor_brightness(device_name: String) -> Result<BrightnessInfo, String> {
    run_display_task(move || active_provider().get_monitor_brightness(device_name)).await
}

#[tauri::command]
pub async fn set_monitor_brightness(device_name: String, percent: u32) -> Result<(), String> {
    run_display_task(move || active_provider().set_monitor_brightness(device_name, percent)).await
}

#[tauri::command]
pub async fn identify_monitors(app_handle: tauri::AppHandle) -> Result<(), String> {
    run_display_task(move || active_provider().identify_monitors(app_handle)).await
}

#[tauri::command]
pub async fn set_monitor_scale(device_name: String, scale_percent: u32) -> Result<(), String> {
    run_display_task(move || active_provider().set_monitor_scale(device_name, scale_percent)).await
}

#[tauri::command]
pub async fn enable_hdr(device_name: String, enable: bool) -> Result<(), String> {
    run_display_task(move || active_provider().enable_hdr(device_name, enable)).await
}

#[tauri::command]
pub async fn set_monitor_input_source(device_name: String, input: String) -> Result<(), String> {
    run_display_task(move || active_provider().set_monitor_input_source(device_name, input)).await
}

#[tauri::command]
pub async fn get_monitor_input_source(device_name: String) -> Result<String, String> {
    run_display_task(move || active_provider().get_monitor_input_source(device_name)).await
}

#[tauri::command]
pub async fn get_monitor_ddc_caps(device_name: String) -> Result<String, String> {
    run_display_task(move || active_provider().get_monitor_ddc_caps(device_name)).await
}

#[tauri::command]
pub async fn set_monitor_power(device_name: String, power_on: bool) -> Result<(), String> {
    run_display_task(move || active_provider().set_monitor_power(device_name, power_on)).await
}
