use serde_json::Value;
use tauri::{App, AppHandle};
use tauri_plugin_store::StoreExt;

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