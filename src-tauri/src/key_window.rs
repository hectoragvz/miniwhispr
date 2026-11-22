//! The "Groq API key" window, where the key is entered or replaced. The
//! page (`key.html`) never sees a saved key, only whether there is one.

use objc2_app_kit::NSApplication;
use objc2_foundation::MainThreadMarker;
use tauri::{AppHandle, LogicalSize, Manager, WebviewUrl, WindowEvent};

use crate::key_store;

const LABEL: &str = "key";
/// Logical width of the window; `src/key/KeyForm.css` lays out its contents.
const WIDTH: f64 = 440.0;
/// A first guess. The window opens hidden, and the page fits the height to
/// its content before showing it (see [`show_key_window`]).
const FIRST_HEIGHT: f64 = 160.0;

/// Opens the window, or brings it to the front if it is already open. Call
/// on the main thread.
pub fn open(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(LABEL) {
        return window.set_focus();
    }
    let window = tauri::WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("key.html".into()))
        .title("Groq API key")
        .inner_size(WIDTH, FIRST_HEIGHT)
        .resizable(false)
        .minimizable(false)
        .maximizable(false)
        .visible(false)
        .build()?;
    window.on_window_event(|event| {
        // Opening the window made Miniwhispr the active app. With the window
        // gone, hand focus back to the app that had it.
        if let WindowEvent::Destroyed = event
            && let Some(mtm) = MainThreadMarker::new()
        {
            NSApplication::sharedApplication(mtm).deactivate();
        }
    });
    Ok(())
}

/// The page has laid itself out: it is `content_height` tall, and the
/// window shows `visible_height` of it. Fits the window to the content, so
/// nothing is cut off whatever the title bar takes, then shows it.
#[tauri::command]
pub async fn show_key_window(
    app: AppHandle,
    content_height: f64,
    visible_height: f64,
) -> Result<(), String> {
    let window = app
        .get_webview_window(LABEL)
        .ok_or("The Groq API key window is gone")?;
    let fit_and_show = || -> tauri::Result<()> {
        let inner = window
            .inner_size()?
            .to_logical::<f64>(window.scale_factor()?);
        let height = inner.height + content_height - visible_height;
        window.set_size(LogicalSize::new(WIDTH, height))?;
        window.center()?;
        window.show()?;
        window.set_focus()
    };
    fit_and_show().map_err(|e| {
        log::error!("Could not show the Groq API key window: {e}");
        e.to_string()
    })
}

// The Keychain commands are async so they run off the main thread: a
// Keychain access prompt must not freeze the app.

/// Whether a key is saved.
#[tauri::command]
pub async fn has_groq_key() -> Result<bool, String> {
    key_store::read().map(|key| key.is_some())
}

/// Saves `key`, replacing any saved one, and closes the window.
#[tauri::command]
pub async fn save_groq_key(app: AppHandle, key: String) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("Enter a key".to_string());
    }
    key_store::save(key).inspect_err(|e| log::error!("{e}"))?;
    if let Some(window) = app.get_webview_window(LABEL) {
        window.close().map_err(|e| e.to_string())?;
    }
    Ok(())
}
