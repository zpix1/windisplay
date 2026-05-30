use tauri::{AppHandle, PhysicalPosition, Position, WebviewWindow};

fn window_size_px(window: &WebviewWindow) -> (i32, i32) {
    let size = window.outer_size().ok();
    let width_px = size.as_ref().map(|s| s.width as i32).unwrap_or(300);
    let height_px = size.as_ref().map(|s| s.height as i32).unwrap_or(600);

    (width_px, height_px)
}

pub fn compute_window_position_for_tray_click(
    app: &AppHandle,
    window: &WebviewWindow,
    rect_position: Position,
) -> Position {
    // Determine current window size in physical pixels
    let (width_px, height_px) = window_size_px(window);

    // Compute click position in physical pixels
    let (click_x, click_y): (i32, i32) = match rect_position {
        Position::Physical(p) => (p.x, p.y),
        Position::Logical(p) => {
            let scale = window.scale_factor().unwrap_or(1.0);
            ((p.x * scale).round() as i32, (p.y * scale).round() as i32)
        }
    };

    // Position the window so it appears above the tray area (align left edge to click)
    let mut x_px = click_x;
    let mut y_px = click_y - height_px;

    // Clamp to the monitor that contains the click to avoid jumping across screens
    if let Ok(monitors) = app.available_monitors() {
        if let Some(m) = monitors.into_iter().find(|m| {
            let mp = m.position();
            let ms = m.size();
            click_x >= mp.x
                && click_x < mp.x + ms.width as i32
                && click_y >= mp.y
                && click_y < mp.y + ms.height as i32
        }) {
            let mp = m.position();
            let ms = m.size();
            let max_x = mp.x + (ms.width as i32).saturating_sub(width_px);
            let max_y = mp.y + (ms.height as i32).saturating_sub(height_px);

            if x_px < mp.x {
                x_px = mp.x;
            }
            if y_px < mp.y {
                y_px = mp.y;
            }
            if x_px > max_x {
                x_px = max_x;
            }
            if y_px > max_y {
                y_px = max_y;
            }
        }
    }

    Position::Physical(PhysicalPosition { x: x_px, y: y_px })
}

pub fn is_position_on_any_monitor(app: &AppHandle, position: PhysicalPosition<i32>) -> bool {
    app.available_monitors()
        .ok()
        .map(|monitors| {
            monitors.into_iter().any(|monitor| {
                let monitor_position = monitor.position();
                let monitor_size = monitor.size();

                position.x >= monitor_position.x
                    && position.x < monitor_position.x + monitor_size.width as i32
                    && position.y >= monitor_position.y
                    && position.y < monitor_position.y + monitor_size.height as i32
            })
        })
        .unwrap_or(false)
}

pub fn compute_default_window_position(app: &AppHandle, window: &WebviewWindow) -> Position {
    let (width_px, height_px) = window_size_px(window);
    let margin_px = 16;

    let monitor = app.primary_monitor().ok().flatten().or_else(|| {
        app.available_monitors()
            .ok()
            .and_then(|monitors| monitors.into_iter().next())
    });

    if let Some(monitor) = monitor {
        let monitor_position = monitor.position();
        let monitor_size = monitor.size();
        let x_px = monitor_position.x + (monitor_size.width as i32 - width_px - margin_px).max(0);
        let y_px = monitor_position.y + (monitor_size.height as i32 - height_px - margin_px).max(0);

        Position::Physical(PhysicalPosition { x: x_px, y: y_px })
    } else {
        Position::Physical(PhysicalPosition { x: 0, y: 0 })
    }
}
