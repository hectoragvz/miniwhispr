//! The Toast window: a small transparent window at the bottom centre of the
//! screen, above the Dock, that never takes focus. The page inside it
//! (`toast.html`) draws the Toast from the states sent here.

use std::sync::{Arc, Mutex};

use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewUrl, WebviewWindow};

use crate::dictation::Toast;

const LABEL: &str = "toast";
/// Logical size of the window, which sits right on top of the Dock. The pill
/// is smaller, leaving room for its shadow and its gap above the Dock;
/// `src/toast/Toast.css` draws it.
const WIDTH: f64 = 120.0;
const HEIGHT: f64 = 64.0;

/// What the Toast is showing right now.
#[derive(Clone)]
pub struct CurrentToast(Arc<Mutex<Toast>>);

/// Creates the hidden Toast window. Call once, on the main thread.
pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let window = tauri::WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("toast.html".into()))
        .title("Miniwhispr")
        .inner_size(WIDTH, HEIGHT)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .visible_on_all_workspaces(true)
        .skip_taskbar(true)
        .focused(false)
        .focusable(false)
        .visible(false)
        .build()?;
    window.set_ignore_cursor_events(true)?;

    // Also show over full-screen apps, stay put in Mission Control, and stay
    // out of the Cmd + ` window cycle.
    let ns_window = window.ns_window()?.cast::<NSWindow>();
    // SAFETY: Tauri gives a valid NSWindow pointer, and setup runs on the
    // main thread, where AppKit windows must be changed.
    unsafe {
        let ns_window = &*ns_window;
        ns_window.setCollectionBehavior(
            ns_window.collectionBehavior()
                | NSWindowCollectionBehavior::FullScreenAuxiliary
                | NSWindowCollectionBehavior::Stationary
                | NSWindowCollectionBehavior::IgnoresCycle,
        );
    }

    app.manage(CurrentToast(Arc::new(Mutex::new(Toast::Hidden))));
    Ok(())
}

/// Shows `toast`. The window appears on the screen with the mouse pointer.
/// Hiding is up to the page: it plays its exit animation, then calls
/// [`toast_hidden`].
pub fn show(app: &AppHandle, toast: Toast) {
    let Some(window) = app.get_webview_window(LABEL) else {
        return;
    };
    let state = app.state::<CurrentToast>();
    // Held until the page has the new state, so `toast_hidden` cannot hide
    // a window that was just shown again.
    let mut current = state.0.lock().unwrap();
    if *current == Toast::Hidden
        && toast != Toast::Hidden
        && let Err(e) = place(&window).and_then(|()| window.show())
    {
        log::error!("Could not show the Toast: {e}");
    }
    *current = toast;
    if let Err(e) = window.emit_to(LABEL, "toast", toast) {
        log::error!("Could not update the Toast: {e}");
    }
}

/// The page finished hiding the Toast, so the window can go.
#[tauri::command]
pub async fn toast_hidden(app: AppHandle) {
    let state = app.state::<CurrentToast>();
    let current = state.0.lock().unwrap();
    if *current == Toast::Hidden
        && let Some(window) = app.get_webview_window(LABEL)
        && let Err(e) = window.hide()
    {
        log::error!("Could not hide the Toast: {e}");
    }
}

/// Moves the window to the bottom centre of the screen with the mouse
/// pointer, just above the Dock.
fn place(window: &WebviewWindow) -> tauri::Result<()> {
    let cursor = window.cursor_position()?;
    let Some(monitor) = window
        .monitor_from_point(cursor.x, cursor.y)?
        .or(window.primary_monitor()?)
    else {
        return Ok(());
    };
    // The work area is the screen minus the menu bar and the Dock.
    let area = monitor.work_area();
    let size = window.outer_size()?;
    let x = area.position.x + (area.size.width as i32 - size.width as i32) / 2;
    let y = area.position.y + area.size.height as i32 - size.height as i32;
    window.set_position(PhysicalPosition::new(x, y))
}
