mod cache;
mod capture;
mod commands;
mod config;
mod dictionary;
mod hotkey;
mod translate;

use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    WindowEvent,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::app_info::get_app_info,
            commands::window_cmd::show_settings_window,
            commands::window_cmd::show_popup_window,
            commands::window_cmd::hide_popup_window,
        ])
        .setup(|app| {
            let settings_item =
                MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

            let _tray = TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().expect("missing default window icon").clone())
                .menu(&menu)
                .tooltip("Look Translate")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        let _ = commands::window_cmd::open_settings(app);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                // Keep process alive via tray; hide windows instead of destroying them.
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running Look Translate");
}
