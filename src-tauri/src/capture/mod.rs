//! Clipboard capture (Phase 1) and OCR hook (Phase 2).

use std::sync::RwLock;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

pub const MAX_CAPTURE_CHARS: usize = 8_000;
pub const CAPTURE_TIMEOUT: Duration = Duration::from_millis(450);
pub const CAPTURE_POLL: Duration = Duration::from_millis(20);
pub const MARKER_PREFIX: &str = "__look_translate_marker__";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturePayload {
    pub text: String,
    pub empty: bool,
    pub error: Option<String>,
    pub captured_at_ms: u128,
}

impl CapturePayload {
    pub fn ok(text: String) -> Self {
        Self {
            text,
            empty: false,
            error: None,
            captured_at_ms: now_ms(),
        }
    }

    pub fn fail(message: impl Into<String>) -> Self {
        Self {
            text: String::new(),
            empty: true,
            error: Some(message.into()),
            captured_at_ms: now_ms(),
        }
    }
}

#[derive(Debug, Default)]
pub struct CaptureState {
    pub last: RwLock<Option<CapturePayload>>,
}

impl CaptureState {
    pub fn store(&self, payload: CapturePayload) {
        if let Ok(mut guard) = self.last.write() {
            *guard = Some(payload);
        }
    }

    pub fn latest(&self) -> Option<CapturePayload> {
        self.last.read().ok().and_then(|g| g.clone())
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Trim and validate captured text.
pub fn normalize_captured_text(raw: &str) -> Result<String, String> {
    let text = raw.trim();
    if text.is_empty() {
        return Err("未获取到选中文本（请先选中文字再按热键）".into());
    }
    if text.chars().count() > MAX_CAPTURE_CHARS {
        return Err(format!(
            "选中文本过长（最多 {MAX_CAPTURE_CHARS} 字）"
        ));
    }
    Ok(text.to_string())
}

/// Backup clipboard → simulate Ctrl+C → read → restore.
pub fn capture_selection() -> Result<String, String> {
    #[cfg(windows)]
    {
        capture_selection_windows()
    }
    #[cfg(not(windows))]
    {
        Err("当前仅支持 Windows 剪贴板取词".into())
    }
}

#[cfg(windows)]
fn capture_selection_windows() -> Result<String, String> {
    use arboard::Clipboard;
    use enigo::{
        Direction::{Click, Press, Release},
        Enigo, Key, Keyboard, Settings,
    };

    let mut clipboard =
        Clipboard::new().map_err(|e| format!("打开剪贴板失败: {e}"))?;

    // Text-only backup (non-text clipboard content may not survive restore).
    let previous_text = clipboard.get_text().ok();

    let marker = format!("{MARKER_PREFIX}{}", now_ms());
    clipboard
        .set_text(marker.clone())
        .map_err(|e| format!("写入剪贴板标记失败: {e}"))?;

    // Brief settle so the marker is visible before we send Ctrl+C.
    thread::sleep(Duration::from_millis(30));

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("初始化按键模拟失败: {e}"))?;
    enigo
        .key(Key::Control, Press)
        .map_err(|e| format!("模拟 Ctrl 按下失败: {e}"))?;
    enigo
        .key(Key::Unicode('c'), Click)
        .map_err(|e| format!("模拟 C 失败: {e}"))?;
    enigo
        .key(Key::Control, Release)
        .map_err(|e| format!("模拟 Ctrl 松开失败: {e}"))?;

    let raw = poll_clipboard_change(&mut clipboard, &marker)?;
    restore_clipboard(&mut clipboard, previous_text.as_deref());

    normalize_captured_text(&raw)
}

#[cfg(windows)]
fn poll_clipboard_change(
    clipboard: &mut arboard::Clipboard,
    marker: &str,
) -> Result<String, String> {
    let deadline = Instant::now() + CAPTURE_TIMEOUT;
    let mut last_seen = String::new();

    while Instant::now() < deadline {
        thread::sleep(CAPTURE_POLL);
        match clipboard.get_text() {
            Ok(text) if text != marker => {
                return Ok(text);
            }
            Ok(text) => {
                last_seen = text;
            }
            Err(_) => {
                // Clipboard may be locked briefly after Ctrl+C.
            }
        }
    }

    if last_seen.is_empty() || last_seen == marker {
        Err("取词超时：未检测到新的剪贴板内容".into())
    } else {
        Ok(last_seen)
    }
}

#[cfg(windows)]
fn restore_clipboard(clipboard: &mut arboard::Clipboard, previous: Option<&str>) {
    let result = match previous {
        Some(text) => clipboard.set_text(text.to_string()),
        None => clipboard.clear(),
    };
    if let Err(err) = result {
        eprintln!("[look-translate] restore clipboard failed: {err}");
    }
}

/// Run capture and always produce a payload (success or failure).
pub fn capture_payload() -> CapturePayload {
    match capture_selection() {
        Ok(text) => CapturePayload::ok(text),
        Err(err) => CapturePayload::fail(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_rejects_empty() {
        assert!(normalize_captured_text("").is_err());
        assert!(normalize_captured_text("   \n\t").is_err());
    }

    #[test]
    fn normalize_trims() {
        assert_eq!(normalize_captured_text("  hello  ").unwrap(), "hello");
    }

    #[test]
    fn normalize_rejects_too_long() {
        let long = "字".repeat(MAX_CAPTURE_CHARS + 1);
        assert!(normalize_captured_text(&long).is_err());
    }
}
