use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Wry};

use crate::voice_commands::VOICE_COMMANDS;
use crate::{driver, key_window, launch_at_login};

const COPY_LAST_DICTATION: &str = "copy-last-dictation";
const GROQ_API_KEY: &str = "groq-api-key";
const LAUNCH_AT_LOGIN: &str = "launch-at-login";
const QUIT: &str = "quit";

/// Puts the Miniwhispr icon and its menu in the macOS menu bar. Returns the
/// "Copy last dictation" item, which starts disabled.
pub fn create(app: &AppHandle) -> tauri::Result<MenuItem<Wry>> {
    let copy_last_dictation = MenuItem::with_id(
        app,
        COPY_LAST_DICTATION,
        "Copy last dictation",
        false,
        None::<&str>,
    )?;
    let voice_commands = Submenu::new(app, "Voice Commands", true)?;
    for entry in voice_command_entries() {
        // Disabled: the entries are a reference, there is nothing to click.
        voice_commands.append(&MenuItem::new(app, entry, false, None::<&str>)?)?;
    }
    let groq_api_key = MenuItem::with_id(app, GROQ_API_KEY, "Groq API key…", true, None::<&str>)?;
    let launch_at_login_item = CheckMenuItem::with_id(
        app,
        LAUNCH_AT_LOGIN,
        "Launch at login",
        true,
        launch_at_login::is_enabled(),
        None::<&str>,
    )?;
    let quit = MenuItem::with_id(app, QUIT, "Quit Miniwhispr", true, Some("CmdOrCtrl+Q"))?;
    let menu = Menu::with_items(
        app,
        &[
            &copy_last_dictation,
            &PredefinedMenuItem::separator(app)?,
            &voice_commands,
            &PredefinedMenuItem::separator(app)?,
            &groq_api_key,
            &launch_at_login_item,
            &PredefinedMenuItem::separator(app)?,
            &quit,
        ],
    )?;

    TrayIconBuilder::with_id("main")
        .icon(Image::from_bytes(include_bytes!("../icons/tray.png"))?)
        .icon_as_template(true)
        .tooltip("Miniwhispr")
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            COPY_LAST_DICTATION => driver::copy_last_dictation(app),
            GROQ_API_KEY => {
                if let Err(e) = key_window::open(app) {
                    log::error!("Could not open the Groq API key window: {e}");
                }
            }
            LAUNCH_AT_LOGIN => {
                let wanted = !launch_at_login::is_enabled();
                if let Err(e) = launch_at_login::set_enabled(wanted) {
                    log::error!("Could not change Launch at login: {e}");
                }
                // The click already flipped the check mark; show what macOS actually did.
                if let Err(e) = launch_at_login_item.set_checked(launch_at_login::is_enabled()) {
                    log::error!("Could not update the Launch at login check mark: {e}");
                }
            }
            QUIT => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(copy_last_dictation)
}

/// One read-only line per Voice Command: both phrases and what it does.
pub fn voice_command_entries() -> Vec<String> {
    VOICE_COMMANDS
        .iter()
        .map(|command| {
            format!(
                "“{}” / “{}”: {}",
                command.english, command.spanish, command.effect
            )
        })
        .collect()
}
