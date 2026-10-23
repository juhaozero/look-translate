// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Release builds have no console. Keep panic details available even when
    // Tauri fails before setup (and before the portable data directory exists).
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        use std::io::Write;
        let base = std::env::var_os("LOCALAPPDATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let log_dir = base.join("Look Translate").join("logs");
        if std::fs::create_dir_all(&log_dir).is_ok() {
            if let Ok(mut log) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_dir.join("startup.log"))
            {
                let _ = writeln!(log, "{:?}: {info}", std::time::SystemTime::now());
            }
        }
        default_hook(info);
    }));
    look_translate_lib::run()
}
