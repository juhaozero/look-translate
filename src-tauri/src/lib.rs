mod cache;
mod capture;
mod commands;
mod config;
mod dictionary;
mod hotkey;
mod translate;
mod tray_state;

use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, WindowEvent,
};

use crate::cache::TranslationCacheState;
use crate::capture::CaptureState;
use crate::commands::window_cmd::PopupUiState;
use crate::config::{load_or_init, resolve_paths, ConfigState};
use crate::translate::TranslationState;
use crate::tray_state::TrayHotkeyToggle;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::app_info::get_app_info,
            commands::window_cmd::show_settings_window,
            commands::window_cmd::show_popup_window,
            commands::window_cmd::hide_popup_window,
            commands::window_cmd::copy_text,
            commands::config_cmd::get_app_paths,
            commands::config_cmd::get_config,
            commands::config_cmd::save_config,
            commands::hotkey_cmd::get_hotkey_status,
            commands::capture_cmd::get_last_capture,
            commands::translate_cmd::get_last_translation,
            commands::translate_cmd::translate_text,
        ])
        .setup(|app| {
            let paths = resolve_paths().map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, e)
            })?;
            let config = load_or_init(&paths).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, e)
            })?;
            let hotkey_enabled = config.general.hotkey_enabled;
            app.manage(ConfigState::new(paths, config));
            app.manage(CaptureState::default());
            app.manage(TranslationState::default());
            app.manage(TranslationCacheState::default());
            app.manage(PopupUiState::default());

            if let Err(err) = hotkey::apply_from_state(app.handle()) {
                eprintln!("[look-translate] hotkey apply failed on startup: {err}");
            }

            let settings_item =
                MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
            let hotkey_item = CheckMenuItem::with_id(
                app,
                "toggle_hotkey",
                "启用热键",
                true,
                hotkey_enabled,
                None::<&str>,
            )?;
            let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &hotkey_item, &quit_item])?;
            app.manage(TrayHotkeyToggle {
                item: hotkey_item.clone(),
            });

            let _tray = TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().expect("missing default window icon").clone())
                .menu(&menu)
                .tooltip("Look Translate")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        let _ = commands::window_cmd::open_settings(app);
                    }
                    "toggle_hotkey" => {
                        let _ = commands::hotkey_cmd::toggle_hotkey_enabled(app);
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                // Keep process alive via tray; hide windows instead of destroying them.
                let _ = window.hide();
                api.prevent_close();
            }
            WindowEvent::Focused(false) if window.label() == "popup" => {
                commands::window_cmd::maybe_hide_popup_on_blur(window.app_handle());
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running Look Translate");
}
