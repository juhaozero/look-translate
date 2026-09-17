use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub phase: String,
}

#[tauri::command]
pub fn get_app_info() -> AppInfo {
    AppInfo {
        name: "Look Translate".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        phase: "Phase 1+2 / capture+ocr".into(),
    }
}
