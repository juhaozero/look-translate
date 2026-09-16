//! Global hotkey registration (Ctrl+Shift+D by default).

#![allow(dead_code)]

pub struct HotkeyManager;

impl HotkeyManager {
    pub fn default_translate_hotkey() -> &'static str {
        "Ctrl+Shift+D"
    }
}
