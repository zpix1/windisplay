mod displays;
#[cfg(target_os = "windows")]
mod winDisplays;
mod fakeDisplays;
mod positioning;

pub fn run() {
    use crate::positioning;
    use tauri::{
        menu::{Menu, MenuItem},
        tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
        Manager, WindowEvent,
    };

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            displays::get_all_monitors,
            displays::set_monitor_resolution,
             displays::set_monitor_orientation,
            displays::get_monitor_brightness,
            displays::set_monitor_brightness,
            displays::identify_monitors,
        ])
        .setup(|app| {
            // Build a tray context menu
            let show_item = MenuItem::with_id(app, "show", "Show", true, Some(""))?;
            let exit_item = MenuItem::with_id(app, "exit", "Exit", true, Some(""))?;
            let menu = Menu::with_items(app, &[&show_item, &exit_item])?;

            // Create tray icon using default app icon
            TrayIconBuilder::new()
                .menu(&menu)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app_handle, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
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
                window.on_window_event(move |event| {
                    if let WindowEvent::Focused(false) = event {
                        // let _ = win_clone.hide();
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
