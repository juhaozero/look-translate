use tauri::menu::CheckMenuItem;

/// Tray "启用热键" checkbox item, kept so save/toggle can update checked state.
pub struct TrayHotkeyToggle {
    pub item: CheckMenuItem<tauri::Wry>,
}
