mod cli;
mod display_monitor;
mod displays;
mod fakeDisplays;
#[cfg(target_os = "windows")]
mod hotkeys;
mod positioning;
mod settings;
#[cfg(target_os = "windows")]
mod winDisplays;
pub mod winHdr;

const AUTOSTART_BASE_LABEL: &str = "Start at login";

fn autostart_label(enabled: bool) -> String {
    if enabled {
        format!("{AUTOSTART_BASE_LABEL} ✓")
    } else {
        AUTOSTART_BASE_LABEL.to_string()
    }
}

pub fn run() {
    match cli::run_cli() {
        Ok(true) => {
            return;
        }
        Ok(false) => {
            // No CLI command or explicit --ui, continue with GUI
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    use crate::positioning;
    use tauri::{
        menu::{Menu, MenuItem},
        tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
        Manager, WebviewUrl, WebviewWindowBuilder,
    };
    use tauri_plugin_notification::NotificationExt;

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            displays::get_all_monitors,
            displays::set_monitor_resolution,
            displays::set_monitor_orientation,
            displays::get_monitor_brightness,
            displays::set_monitor_brightness,
            displays::identify_monitors,
            displays::set_monitor_scale,
            displays::enable_hdr,
            displays::set_monitor_input_source,
            displays::get_monitor_input_source,
            displays::get_monitor_ddc_caps,
        ])
        .setup(|app| {
            // helper to reveal main window
            fn reveal_main_window(app_handle: &tauri::AppHandle) {
                if let Some(window) = app_handle.get_webview_window("main") {
                    let _ = window.unminimize();
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            // Enable autostart by default on first run (persist marker so user choice isn't overridden)
            #[cfg(desktop)]
            {
                use std::fs;
                use tauri_plugin_autostart::ManagerExt;
                if let Ok(mut dir) = app.path().app_config_dir() {
                    let marker = {
                        dir.push("autostart_initialized");
                        dir
                    };
                    if !marker.exists() {
                        let manager = app.autolaunch();
                        if !manager.is_enabled().unwrap_or(false) {
                            let _ = manager.enable();
                        }
                        if let Some(parent) = marker.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        let _ = fs::write(&marker, b"1");
                    }
                }
            }
            // Show a notification on startup to inform the user the app is running in the tray
            app.notification()
                .builder()
                .title("WinDisplay")
                .body("WinDisplay is running in the system tray.")
                .show()
                .unwrap();

            // Build a tray context menu
            let show_item = MenuItem::with_id(app, "show", "Show", true, Some(""))?;
            let autostart_item = {
                use tauri_plugin_autostart::ManagerExt;
                let enabled = app.autolaunch().is_enabled().unwrap_or(false);
                let label = autostart_label(enabled);
                MenuItem::with_id(app, "autostart_toggle", label.as_str(), true, Some(""))?
            };
            let about_item = MenuItem::with_id(app, "about", "About", true, Some(""))?;
            let exit_item = MenuItem::with_id(app, "exit", "Exit", true, Some(""))?;
            let menu =
                Menu::with_items(app, &[&show_item, &autostart_item, &about_item, &exit_item])?;

            // Create tray icon using default app icon
            let tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("WinDisplay");

            let autostart_item_handle = autostart_item.clone();
            let tray_builder =
                tray_builder.on_menu_event(move |app_handle, event| match event.id().as_ref() {
                    "show" => {
                        reveal_main_window(&app_handle);
                    }
                    "autostart_toggle" => {
                        use tauri_plugin_autostart::ManagerExt;
                        let manager = app_handle.autolaunch();
                        let currently_enabled = manager.is_enabled().unwrap_or(false);
                        let _ = if currently_enabled {
                            manager.disable()
                        } else {
                            manager.enable()
                        };
                        let new_label = autostart_label(!currently_enabled);
                        let _ = autostart_item_handle.set_text(new_label.as_str());
                    }
                    "about" => {
                        if let Some(window) = app_handle.get_webview_window("about") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        } else {
                            let _ = WebviewWindowBuilder::new(
                                app_handle,
                                "about",
                                WebviewUrl::App("index.html#about".into()),
                            )
                            .title("About WinDisplay")
                            .inner_size(360.0, 280.0)
                            .resizable(false)
                            .minimizable(false)
                            .maximizable(false)
                            .skip_taskbar(true)
                            .visible(true)
                            .build();
                        }
                    }
                    "exit" => {
                        app_handle.exit(0);
                    }
                    _ => {}
                });

            let tray_builder = tray_builder.on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    rect,
                    ..
                } = event
                {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        let pos = positioning::compute_window_position_for_tray_click(
                            &app,
                            &window,
                            rect.position,
                        );
                        let _ = window.set_position(pos);

                        reveal_main_window(&app);
                    }
                }
            });

            tray_builder.build(app)?;

            // Start monitoring for display changes
            if let Err(e) = display_monitor::start_display_monitor(app.handle().clone()) {
                log::warn!("Failed to start display monitor: {}", e);
            }

            // UI reveal on monitor change handled in display_monitor.rs based on settings

            // Start global hotkey service (Windows only) if enabled by settings
            #[cfg(target_os = "windows")]
            {
                if crate::settings::should_register_brightness_hotkeys_app(&app) {
                    crate::hotkeys::start_hotkey_service(app.handle().clone());
                } else {
                    log::info!(
                        "Skipping brightness hotkeys startup: keyboardBrightnessShortcut is 'system'"
                    );
                }
            }

            // Keep main window hidden until tray click (config also sets visible: false)
            if let Some(window) = app.get_webview_window("main") {
                // Hide on focus out
                let window_for_event = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::Focused(false) = event {
                        let _ = window_for_event.hide();
                    }
                });
                // Ensure the window does not appear in the taskbar
                let _ = window.set_skip_taskbar(true);
                let _ = window.hide();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
