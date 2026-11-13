use serde_json::Value;
use tauri::{App, AppHandle};
use tauri_plugin_store::StoreExt;

pub fn brightness_hotkeys_app(app: &App) -> (String, String) {
    let settings = app
        .store("settings.json")
        .ok()
        .and_then(|store| store.get("settings"));
    let hotkey = |key: &str, default: &str| {
        settings
            .as_ref()
            .and_then(|value| value.get(key))
            .and_then(Value::as_str)
            .unwrap_or(default)
            .to_string()
    };
    (
        hotkey("brightnessDownHotkey", "F14"),
        hotkey("brightnessUpHotkey", "F15"),
    )
}

pub fn should_register_brightness_hotkeys_app(app: &App) -> bool {
    if let Ok(store) = app.store("settings.json") {
        if let Some(Value::Object(map)) = store.get("settings") {
            if let Some(s) = map
                .get("keyboardBrightnessShortcut")
                .and_then(|v| v.as_str())
            {
                return matches!(s, "all_screens" | "screen_with_mouse");
            }
        }
    }
    false
}

pub fn should_show_ui_on_monitor_change_handle(handle: &AppHandle) -> bool {
    if let Ok(store) = handle.store("settings.json") {
        if let Some(Value::Object(map)) = store.get("settings") {
            if let Some(v) = map.get("showUIOnMonitorChange").and_then(|v| v.as_bool()) {
                return v;
            }
        }
    }
    false
}

pub fn should_show_startup_notification_app(app: &App) -> bool {
    if let Ok(store) = app.store("settings.json") {
        if let Some(Value::Object(map)) = store.get("settings") {
            if let Some(v) = map.get("showStartupNotification").and_then(|v| v.as_bool()) {
                return v;
            }
        }
    }
    true
}

pub fn should_hide_ui_on_focus_out_handle(handle: &AppHandle) -> bool {
    if let Ok(store) = handle.store("settings.json") {
        if let Some(Value::Object(map)) = store.get("settings") {
            if let Some(v) = map.get("shouldHideUIOnFocusOut").and_then(|v| v.as_bool()) {
                return v;
            }
        }
    }
    true
}
