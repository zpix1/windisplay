use std::collections::HashMap;
use std::sync::Once;
use std::time::Instant;

use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyManager,
};
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use tauri::{AppHandle, Emitter};
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, GetMessageW, TranslateMessage, MSG,
};

static START_ONCE: Once = Once::new();

pub fn start_hotkey_service(app_handle: AppHandle) {
    START_ONCE.call_once(|| {
        // Thread that owns the Win32 message loop and the hotkey manager
        std::thread::spawn(move || unsafe {
            // Create the manager on the same thread as the message loop
            let manager = match GlobalHotKeyManager::new() {
                Ok(m) => m,
                Err(e) => {
                    log::error!("Failed to create GlobalHotKeyManager: {e}");
                    return;
                }
            };

            // Register brightness keys and F14/F15 and map ids to deltas
            let mut id_to_delta: HashMap<u32, i32> = HashMap::new();
            for (code, delta) in [
                (Code::BrightnessUp, 5),
                (Code::BrightnessDown, -5),
                (Code::F15, 5),
                (Code::F14, -5),
            ] {
                let hk = HotKey::new(None::<Modifiers>, code);
                if let Err(e) = manager.register(hk) {
                    log::warn!("Failed to register hotkey for {:?}: {e}", code);
                } else {
                    id_to_delta.insert(hk.id(), delta);
                    log::info!("Registered hotkey for {:?} (id={})", code, hk.id());
                }
            }

            let id_to_delta_listener = id_to_delta.clone();
            let app_for_events = app_handle.clone();

            // Separate listener thread for hotkey events
            std::thread::spawn(move || loop {
                match GlobalHotKeyEvent::receiver().recv() {
                    Ok(event) => {
                        // Only react on key press
                        if event.state == HotKeyState::Pressed {
                            if let Some(delta) = id_to_delta_listener.get(&event.id) {
                                if adjust_brightness_all(*delta) {
                                    let _ = app_for_events.emit("brightness-changed", ());
                                }
                            }
                        }
                    }
                    Err(_) => break,
                }
            });

            // Minimal Win32 message loop to satisfy Windows requirement
            let mut message: MSG = MSG::default();
            while GetMessageW(&mut message, None, 0, 0).into() {
                let _ = TranslateMessage(&message);
                let _ = DispatchMessageW(&message);
            }
        });
    });
}

fn adjust_brightness_all(delta: i32) -> bool {
    let provider = crate::displays::active_provider();
    let mut max_times_ns: HashMap<&'static str, u128> = HashMap::new();
    let mut update_max = |label: &'static str, elapsed_ns: u128| {
        let entry = max_times_ns.entry(label).or_insert(0);
        if elapsed_ns > *entry {
            *entry = elapsed_ns;
        }
    };

    let t_names_start = Instant::now();
    let monitor_names = match provider.get_all_monitors_short() {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to fetch monitors: {e}");
            return false;
        }
    };
    update_max("get_all_monitors_short", t_names_start.elapsed().as_nanos());

    let mut any_changed = false;
    for name in monitor_names {
        let t_get_start = Instant::now();
        match provider.get_monitor_brightness(name.clone()) {
            Ok(info) => {
                update_max("get_monitor_brightness", t_get_start.elapsed().as_nanos());
                let current = info.current as i32;
                let min = info.min as i32;
                let max = info.max as i32;
                let mut next = current + delta;
                if next < min {
                    next = min;
                }
                if next > max {
                    next = max;
                }
                let next_u32 = next as u32;
                if next_u32 != info.current {
                    let t_set_start = Instant::now();
                    match provider.set_monitor_brightness(name.clone(), next_u32) {
                        Ok(()) => {
                            update_max("set_monitor_brightness", t_set_start.elapsed().as_nanos());
                            log::info!(
                                "Set brightness for {} from {} to {} (range {}-{})",
                                name,
                                info.current,
                                next_u32,
                                info.min,
                                info.max
                            );
                            any_changed = true;
                        }
                        Err(e) => log::warn!("Failed to set brightness for {}: {}", name, e),
                    }
                }
            }
            Err(e) => log::warn!("Failed to read brightness for {}: {}", name, e),
        }
    }
    if !max_times_ns.is_empty() {
        if let Some((label, ns)) = max_times_ns.into_iter().max_by_key(|(_, v)| *v) {
            let ms = (ns as f64) / 1_000_000.0;
            log::info!(
                "Hotkey brightness timing: slowest='{}' took {:.2}ms",
                label,
                ms
            );
        }
    }
    any_changed
}
