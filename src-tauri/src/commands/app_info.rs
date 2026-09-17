use serde::Serialize;

/// Product display name (matches `tauri.conf.json` productName / installer).
pub const APP_NAME: &str = "Look Translate";
/// One-line product description used in About, tray tooltip, and packaging copy.
pub const APP_DESCRIPTION: &str = "Windows 划词翻译小工具";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: APP_NAME.into(),
        version: env!("CARGO_PKG_VERSION").into(),
        description: APP_DESCRIPTION.into(),
    }
}
