use crate::displays::{BrightnessInfo, DisplayInfo, Displays, Resolution};

pub struct FakeDisplays;

impl FakeDisplays {
    pub fn new() -> Self {
        Self
    }
}

impl Displays for FakeDisplays {
    fn get_all_monitors(&self) -> Result<Vec<DisplayInfo>, String> {
        let modes = vec![
            Resolution {
                width: 1920,
                height: 1080,
                bits_per_pixel: 32,
                refresh_hz: 60,
            },
            Resolution {
                width: 2560,
                height: 1440,
                bits_per_pixel: 32,
                refresh_hz: 60,
            },
            Resolution {
                width: 3840,
                height: 2160,
                bits_per_pixel: 32,
                refresh_hz: 60,
            },
        ];

        let displays = (0..4)
            .map(|i| DisplayInfo {
                device_name: format!("\\\\.\\DISPLAY{}", i + 1),
                friendly_name: format!("Fake Monitor {}", i + 1),
                is_primary: i == 0,
                position_x: (i as i32) * 1920,
                position_y: 0,
                orientation: 0,
                current: modes[0].clone(),
                modes: modes.clone(),
                max_native: modes
                    .iter()
                    .cloned()
                    .max_by_key(|m| (m.width as u64) * (m.height as u64))
                    .unwrap_or_else(|| modes[0].clone()),
                model: String::new(),
                serial: String::new(),
                manufacturer: String::new(),
                year_of_manufacture: 0,
                week_of_manufacture: 0,
                connection: String::new(),
                built_in: false,
                active: false,
                scale: if i == 0 { 1.25 } else { 1.0 },
                scales: vec![],
            })
            .collect();

        Ok(displays)
    }

    fn set_monitor_resolution(
        &self,
        _device_name: String,
        _width: u32,
        _height: u32,
        _refresh_hz: Option<u32>,
    ) -> Result<(), String> {
        Ok(())
    }

    fn get_monitor_brightness(&self, _device_name: String) -> Result<BrightnessInfo, String> {
        Ok(BrightnessInfo {
            min: 0,
            current: 50,
            max: 100,
        })
    }

    fn set_monitor_brightness(&self, _device_name: String, _percent: u32) -> Result<(), String> {
        Ok(())
    }

    fn identify_monitors(&self, _app_handle: tauri::AppHandle) -> Result<(), String> {
        Ok(())
    }

    fn set_monitor_orientation(
        &self,
        _device_name: String,
        _orientation_degrees: u32,
    ) -> Result<(), String> {
        Ok(())
    }

    fn set_monitor_scale(&self, _device_name: String, _scale_percent: u32) -> Result<(), String> {
        Ok(())
    }
}
