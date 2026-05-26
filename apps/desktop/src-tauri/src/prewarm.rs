use tauri::{AppHandle, LogicalPosition, Manager, PhysicalPosition, Runtime};

pub fn show<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.show()?;
        window.set_focus()?;
    }
    Ok(())
}

pub fn hide<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide()?;
    }
    Ok(())
}

pub fn toggle<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            window.hide()?;
        } else {
            window.show()?;
            window.set_focus()?;
        }
    }
    Ok(())
}

/// Surface the pre-warmed window on the monitor under the user's cursor.
///
/// Called when a guard event arrives so the dialog appears where the user
/// is already looking. Falls back to the primary monitor if the cursor
/// position is unavailable.
pub fn show_on_event<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let Some(window) = app.get_webview_window("main") else {
        return Ok(());
    };

    if let Err(e) = center_on_cursor_monitor(&window) {
        tracing::warn!("guard dialog could not center on cursor monitor: {e}");
    }

    window.show()?;
    window.set_focus()?;
    Ok(())
}

fn center_on_cursor_monitor<R: Runtime>(window: &tauri::WebviewWindow<R>) -> tauri::Result<()> {
    let cursor = match window.cursor_position() {
        Ok(p) => p,
        Err(_) => return window.center(),
    };

    let monitor = pick_monitor(window, &cursor)?;
    let scale = monitor.scale_factor();
    let monitor_pos = monitor.position().to_logical::<f64>(scale);
    let monitor_size = monitor.size().to_logical::<f64>(scale);
    let window_size = window.outer_size()?.to_logical::<f64>(scale);

    let x = monitor_pos.x + (monitor_size.width - window_size.width) / 2.0;
    let y = monitor_pos.y + (monitor_size.height - window_size.height) / 2.0;
    window.set_position(LogicalPosition::new(x, y))?;
    Ok(())
}

fn pick_monitor<R: Runtime>(
    window: &tauri::WebviewWindow<R>,
    cursor: &PhysicalPosition<f64>,
) -> tauri::Result<tauri::Monitor> {
    for monitor in window.available_monitors()? {
        let pos = monitor.position();
        let size = monitor.size();
        let in_x = cursor.x >= pos.x as f64 && cursor.x < pos.x as f64 + size.width as f64;
        let in_y = cursor.y >= pos.y as f64 && cursor.y < pos.y as f64 + size.height as f64;
        if in_x && in_y {
            return Ok(monitor);
        }
    }
    window
        .primary_monitor()?
        .or_else(|| window.current_monitor().ok().flatten())
        .ok_or_else(|| tauri::Error::FailedToReceiveMessage)
}
