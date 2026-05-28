use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Manager, PhysicalPosition, Position, WindowEvent};

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
const PARKED_WINDOW_POSITION: PhysicalPosition<i32> = PhysicalPosition { x: -32000, y: -32000 };
static MAIN_WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

fn autostart_label(enabled: bool) -> String {
    if enabled {
        format!("{AUTOSTART_BASE_LABEL} ✓")
    } else {
        AUTOSTART_BASE_LABEL.to_string()
    }
}

pub(crate) fn reveal_main_window(app_handle: &tauri::AppHandle) -> bool {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_ignore_cursor_events(false);
        let _ = window.set_focus();
        MAIN_WINDOW_OPEN.store(true, Ordering::SeqCst);
        true
    } else {
        false
    }
}

fn park_main_window(app_handle: &tauri::AppHandle) -> bool {
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.set_ignore_cursor_events(true);
        let _ = window.set_position(Position::Physical(PARKED_WINDOW_POSITION));
        MAIN_WINDOW_OPEN.store(false, Ordering::SeqCst);
        true
    } else {
        false
    }
}

fn toggle_main_window_for_tray_click(
    app_handle: &tauri::AppHandle,
    tray_position: Position,
) -> bool {
    if MAIN_WINDOW_OPEN.load(Ordering::SeqCst) {
        park_main_window(app_handle)
    } else if let Some(window) = app_handle.get_webview_window("main") {
        let pos = positioning::compute_window_position_for_tray_click(
            app_handle,
            &window,
            tray_position,
        );
        let _ = window.set_position(pos);
        reveal_main_window(app_handle)
    } else {
        false
    }
}

fn write_smoke_report(message: String) {
    if let Some(path) = std::env::var_os("WINDISPLAY_SMOKE_OUT") {
        use std::io::Write;
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "{message}");
        }
    }
}

#[tauri::command]
fn smoke_report(message: String) {
    write_smoke_report(message);
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

    use tauri::{
        menu::{Menu, MenuItem},
        tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
        WebviewUrl, WebviewWindowBuilder,
    };
    use tauri_plugin_notification::NotificationExt;

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            displays::set_monitor_power,
            smoke_report,
        ])
        .setup(|app| {
            // Log settings file location
            if let Ok(mut settings_path) = app.path().app_data_dir() {
                settings_path.push("settings.json");
                log::info!("Settings file: {}", settings_path.display());
            }

            // Enable autostart by default on first run (persist marker so user choice isn't overridden)
            // Only do this in release builds to avoid registering a dev (console) binary on Windows.
            #[cfg(all(desktop, not(debug_assertions)))]
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
            // On Windows release builds: if autostart is already enabled (possibly from a prior dev run),
            // re-register it to ensure it points to the current GUI (windows subsystem) executable.
            #[cfg(all(target_os = "windows", not(debug_assertions)))]
            {
                use tauri_plugin_autostart::ManagerExt;
                let manager = app.autolaunch();
                if manager.is_enabled().unwrap_or(false) {
                    let _ = manager.disable();
                    let _ = manager.enable();
                }
            }
            // Show a notification on startup to inform the user the app is running in the tray
            if crate::settings::should_show_startup_notification_app(&app) {
                app.notification()
                    .builder()
                    .title("WinDisplay")
                    .body("WinDisplay is running in the system tray.")
                    .show()
                    .unwrap();
            }

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

            MAIN_WINDOW_OPEN.store(false, Ordering::SeqCst);

            let tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
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

            let tray_builder = tray_builder.on_tray_icon_event(move |tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    rect,
                    ..
                } = event
                {
                    let app = tray.app_handle();

                    toggle_main_window_for_tray_click(&app, rect.position);
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
                window.on_window_event(move |event| {
                    match event {
                        WindowEvent::Destroyed => {
                            MAIN_WINDOW_OPEN.store(false, Ordering::SeqCst);
                        }
                        _ => {}
                    }
                });
                // Ensure the window does not appear in the taskbar
                let _ = window.set_skip_taskbar(true);
                let _ = window.set_ignore_cursor_events(true);
                let _ = window.set_position(Position::Physical(PARKED_WINDOW_POSITION));
                let _ = window.show();
                MAIN_WINDOW_OPEN.store(false, Ordering::SeqCst);

                if std::env::var_os("WINDISPLAY_SMOKE").is_some() {
                    let window_for_smoke = window.clone();
                    let app_for_smoke = app.handle().clone();
                    tauri::async_runtime::spawn_blocking(move || {
                        let started = std::time::Instant::now();
                        let mut marks: Vec<String> = Vec::new();
                        let mut mark = |name: &str| {
                            marks.push(format!(
                                "{name}:{}",
                                started.elapsed().as_millis()
                            ));
                            write_smoke_report(format!("SMOKE {}", marks.join("|")));
                        };

                        park_main_window(&app_for_smoke);
                        mark("native-start");
                        let smoke_tray_position = Position::Physical(PhysicalPosition { x: 260, y: 260 });
                        toggle_main_window_for_tray_click(&app_for_smoke, smoke_tray_position);
                        mark(if MAIN_WINDOW_OPEN.load(Ordering::SeqCst) {
                            "native-first-open"
                        } else {
                            "native-first-closed"
                        });
                        toggle_main_window_for_tray_click(&app_for_smoke, smoke_tray_position);
                        mark(if MAIN_WINDOW_OPEN.load(Ordering::SeqCst) {
                            "native-second-open"
                        } else {
                            "native-second-closed"
                        });
                        toggle_main_window_for_tray_click(&app_for_smoke, smoke_tray_position);
                        mark(if MAIN_WINDOW_OPEN.load(Ordering::SeqCst) {
                            "native-third-open"
                        } else {
                            "native-third-closed"
                        });
                        let _ = window_for_smoke.set_position(Position::Physical(PhysicalPosition { x: 260, y: 260 }));
                        std::thread::sleep(std::time::Duration::from_millis(2500));
                        let _ = window_for_smoke.eval(
                            r#"
                            (() => {
                              const started = performance.now();
                              const marks = [];
                              const report = (message) => {
                                document.body.dataset.smoke = message;
                                window.__TAURI_INTERNALS__?.invoke('smoke_report', { message }).catch(() => {});
                              };
                              const mark = (name) => {
                                const value = Math.round(performance.now() - started);
                                marks.push(`${name}:${value}`);
                                report(`SMOKE ${marks.join('|')}`);
                              };
                              const waitFrame = () => new Promise((resolve) => requestAnimationFrame(resolve));
                              const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
                              const waitFor = async (predicate, timeoutMs = 5000) => {
                                const deadline = performance.now() + timeoutMs;
                                while (performance.now() < deadline) {
                                  const value = predicate();
                                  if (value) return value;
                                  await wait(50);
                                }
                                return predicate();
                              };

                              (async () => {
                                mark('start');
                                const settingsButton = await waitFor(() => {
                                  const button = document.querySelector('[aria-label="Open settings"]');
                                  return button && !button.disabled ? button : null;
                                });
                                if (!settingsButton) {
                                  mark(document.querySelector('[aria-label="Open settings"]') ? 'settings-button-disabled' : 'no-settings-button');
                                  return;
                                }
                                mark('button-ready');
                                settingsButton.click();
                                mark('open-click');
                                await waitFrame();
                                mark(document.querySelector('.settings-dialog') ? 'open-dialog-raf' : 'open-no-dialog-raf');
                                await wait(100);
                                mark(document.querySelector('.settings-dialog') ? 'open-dialog-100' : 'open-no-dialog-100');

                                const overlay = document.querySelector('.dialog-overlay');
                                if (!overlay) {
                                  mark('no-overlay');
                                  return;
                                }
                                overlay.click();
                                mark('close-click');
                                await waitFrame();
                                mark(document.querySelector('.settings-dialog') ? 'close-dialog-raf' : 'close-no-dialog-raf');
                                await wait(100);
                                mark(document.querySelector('.settings-dialog') ? 'close-dialog-100' : 'close-no-dialog-100');
                              })();
                            })();
                            "#,
                        );
                    });
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
