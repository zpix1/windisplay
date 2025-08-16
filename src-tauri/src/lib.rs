mod displays;
mod fakeDisplays;
mod positioning;
#[cfg(target_os = "windows")]
mod winDisplays;
pub mod winHdr;

pub fn run() {
    use crate::positioning;
    use tauri::{
        menu::{Menu, MenuItem},
        tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
        Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent,
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            displays::get_all_monitors,
            displays::set_monitor_resolution,
            displays::set_monitor_orientation,
            displays::get_monitor_brightness,
            displays::set_monitor_brightness,
            displays::identify_monitors,
            displays::set_monitor_scale,
            displays::enable_hdr,
        ])
        .setup(|app| {
            // Build a tray context menu
            let show_item = MenuItem::with_id(app, "show", "Show", true, Some(""))?;
            let about_item = MenuItem::with_id(app, "about", "About", true, Some(""))?;
            let exit_item = MenuItem::with_id(app, "exit", "Exit", true, Some(""))?;
            let menu = Menu::with_items(app, &[&show_item, &about_item, &exit_item])?;

            // Create tray icon using default app icon
            TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("WinDisplay")
                .on_menu_event(|app_handle, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
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
                })
                .on_tray_icon_event(|tray, event| {
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

                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Keep main window hidden until tray click (config also sets visible: false)
            if let Some(window) = app.get_webview_window("main") {
                // Apply vibrancy/blur effects based on platform

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
