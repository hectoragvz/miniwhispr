use tauri::AppHandle;
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;

use crate::launch_at_login;

const LAUNCH_AT_LOGIN: &str = "launch-at-login";
const QUIT: &str = "quit";

/// Puts the Miniwhispr icon and its menu in the macOS menu bar.
pub fn create(app: &AppHandle) -> tauri::Result<()> {
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
    Ok(())
}
