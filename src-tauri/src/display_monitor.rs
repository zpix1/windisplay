#[cfg(target_os = "windows")]
use std::sync::Mutex;
#[cfg(target_os = "windows")]
use tauri::{AppHandle, Emitter, Manager};
#[cfg(target_os = "windows")]
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, RegisterClassW, HMENU, WINDOW_EX_STYLE, WM_DISPLAYCHANGE,
        WNDCLASSW, WS_OVERLAPPEDWINDOW,
    },
};

#[cfg(target_os = "windows")]
static APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);

#[cfg(target_os = "windows")]
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DISPLAYCHANGE => {
            // Display configuration changed
            if let Ok(guard) = APP_HANDLE.lock() {
                if let Some(app) = guard.as_ref() {
                    let _ = app.emit("display-changed", ());
                    // Optionally reveal UI if user enabled it in settings
                    if crate::settings::should_show_ui_on_monitor_change_handle(app) {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[cfg(target_os = "windows")]
pub fn start_display_monitor(app_handle: AppHandle) -> Result<(), String> {
    // Store app handle for use in window_proc
    {
        let mut guard = APP_HANDLE.lock().map_err(|e| e.to_string())?;
        *guard = Some(app_handle.clone());
    }

    // Spawn a thread to create and run the message window
    std::thread::spawn(move || {
        unsafe {
            let class_name = windows::core::w!("WinDisplayMonitorClass");

            let wc = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                lpszClassName: class_name,
                hInstance: GetModuleHandleW(None).unwrap_or_default().into(),
                ..Default::default()
            };

            let atom = RegisterClassW(&wc);
            if atom == 0 {
                log::error!("Failed to register window class for display monitoring");
                return;
            }

            // Create a hidden message-only window
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class_name,
                windows::core::w!("WinDisplay Monitor"),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                0,
                0,
                HWND::default(),
                HMENU::default(),
                GetModuleHandleW(None).unwrap_or_default(),
                None,
            );

            if hwnd.0 == 0 {
                log::error!("Failed to create window for display monitoring");
                return;
            }

            // Message loop
            let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();
            while windows::Win32::UI::WindowsAndMessaging::GetMessageW(
                &mut msg,
                HWND::default(),
                0,
                0,
            )
            .as_bool()
            {
                windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                windows::Win32::UI::WindowsAndMessaging::DispatchMessageW(&msg);
            }
        }
    });

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn start_display_monitor(_app_handle: tauri::AppHandle) -> Result<(), String> {
    // No-op on non-Windows platforms
    Ok(())
}
