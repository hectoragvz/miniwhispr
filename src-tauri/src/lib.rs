mod cleanup;
mod clipboard;
mod dictation;
mod driver;
mod groq;
mod key_store;
mod key_window;
mod launch_at_login;
mod menu_bar;
mod microphone;
mod paste;
mod shortcut;
mod sounds;
mod toast;
mod voice_commands;

use tauri::{ActivationPolicy, Manager};

pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            toast::toast_hidden,
            key_window::has_groq_key,
            key_window::save_groq_key,
            key_window::show_key_window,
        ])
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

            let copy_last_dictation = menu_bar::create(app.handle())?;
            toast::create(app.handle())?;
            driver::start(app.handle(), copy_last_dictation);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Miniwhispr");
}
