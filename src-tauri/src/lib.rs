mod dictation;
mod driver;
mod launch_at_login;
mod menu_bar;
mod microphone;
mod shortcut;
mod sounds;
mod toast;

use tauri::{ActivationPolicy, Manager};

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![toast::toast_hidden])
        .setup(|app| {
            // Menu bar only, no Dock icon. Info.plist sets LSUIElement for the
            // bundled app; this covers `tauri dev`, which runs without it.
            app.set_activation_policy(ActivationPolicy::Accessory);

            let marker = app.path().app_data_dir()?.join("launch-at-login-defaulted");
            if let Err(e) =
                launch_at_login::apply_default(&marker, || launch_at_login::set_enabled(true))
            {
                log::warn!("Could not turn on Launch at login, will retry next launch: {e}");
            }

            menu_bar::create(app.handle())?;
            toast::create(app.handle())?;
            driver::start(app.handle());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Miniwhispr");
}
