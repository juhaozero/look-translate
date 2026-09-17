//! Clipboard selection capture for the translate hotkey.
//!
//! OCR is intentionally **not** used as a fallback here — it has its own
//! hotkey and module (`crate::ocr`).

use std::sync::RwLock;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

pub const MAX_CAPTURE_CHARS: usize = 8_000;
/// Apps often deliver clipboard text asynchronously after Ctrl+C.
pub const CAPTURE_TIMEOUT: Duration = Duration::from_millis(1_200);
pub const CAPTURE_POLL: Duration = Duration::from_millis(25);
pub const MARKER_PREFIX: &str = "__look_translate_marker__";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CapturePayload {
    pub text: String,
    pub empty: bool,
    pub error: Option<String>,
    /// `clipboard` | `ocr`
    pub source: String,
    pub captured_at_ms: u128,
}

impl CapturePayload {
    pub fn ok(text: String, source: &str) -> Self {
        Self {
            text,
            empty: false,
            error: None,
            source: source.to_string(),
            captured_at_ms: now_ms(),
        }
    }

    pub fn fail(message: impl Into<String>, source: &str) -> Self {
        Self {
            text: String::new(),
            empty: true,
            error: Some(message.into()),
            source: source.to_string(),
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
/// Never falls back to OCR.
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

    // Settle so the marker is committed before we send Ctrl+C.
    thread::sleep(Duration::from_millis(40));

    // Give the foreground app a beat after the global hotkey is released.
    thread::sleep(Duration::from_millis(50));

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("初始化按键模拟失败: {e}"))?;

    // Detach from any console so synthesized Ctrl+C is not delivered as
    // CTRL_C_EVENT to this process (debug `cargo run` / `tauri:dev` console flash).
    let _console_guard = DetachConsole::detach();

    // Also ignore CTRL_C if we are still attached somehow.
    let _suppress_ctrl_c = SuppressConsoleCtrlC::install();

    // Ensure modifiers from the global hotkey are up (Ctrl+Shift+C would not copy).
    for key in [Key::LShift, Key::RShift, Key::Alt, Key::Meta] {
        let _ = enigo.key(key, Release);
    }

    // IMPORTANT: do NOT use Key::Unicode('c') — on Windows that becomes
    // KEYEVENTF_UNICODE / VK_PACKET and does not trigger Ctrl+C copy.
    // Key::C maps to VK_C and works as a real shortcut chord.
    enigo
        .key(Key::LControl, Press)
        .or_else(|_| enigo.key(Key::Control, Press))
        .map_err(|e| format!("模拟 Ctrl 按下失败: {e}"))?;
    enigo
        .key(Key::C, Click)
        .map_err(|e| format!("模拟 C 失败: {e}"))?;
    enigo
        .key(Key::LControl, Release)
        .or_else(|_| enigo.key(Key::Control, Release))
        .map_err(|e| format!("模拟 Ctrl 松开失败: {e}"))?;

    // Keep suppressor + detached console across settle + poll window.
    thread::sleep(Duration::from_millis(40));

    let raw = match poll_clipboard_change(&mut clipboard, &marker) {
        Ok(text) => text,
        Err(err) => {
            drop(_suppress_ctrl_c);
            drop(_console_guard);
            restore_clipboard(&mut clipboard, previous_text.as_deref());
            return Err(err);
        }
    };
    drop(_suppress_ctrl_c);
    drop(_console_guard);
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
    let mut saw_marker = false;

    while Instant::now() < deadline {
        thread::sleep(CAPTURE_POLL);
        match clipboard.get_text() {
            Ok(text) if text != marker => {
                // Ignore empty flashes some apps write during copy.
                if text.trim().is_empty() {
                    continue;
                }
                return Ok(text);
            }
            Ok(text) => {
                saw_marker = true;
                last_seen = text;
            }
            Err(_) => {
                // Clipboard may be locked briefly after Ctrl+C.
            }
        }
    }

    if last_seen.is_empty() || last_seen == marker {
        let hint = if saw_marker {
            "取词超时：未检测到新的剪贴板内容（请确认已选中文本，且目标窗口支持 Ctrl+C）"
        } else {
            "取词超时：剪贴板无响应"
        };
        Err(hint.into())
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

/// Ignore CTRL_C / CTRL_BREAK delivered to *this* process while we synthesize Ctrl+C.
#[cfg(windows)]
struct SuppressConsoleCtrlC;

#[cfg(windows)]
impl SuppressConsoleCtrlC {
    fn install() -> Self {
        unsafe {
            let _ = windows::Win32::System::Console::SetConsoleCtrlHandler(
                Some(ignore_console_ctrl),
                true,
            );
        }
        Self
    }
}

#[cfg(windows)]
impl Drop for SuppressConsoleCtrlC {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::System::Console::SetConsoleCtrlHandler(
                Some(ignore_console_ctrl),
                false,
            );
        }
    }
}

#[cfg(windows)]
unsafe extern "system" fn ignore_console_ctrl(ctrl_type: u32) -> windows::core::BOOL {
    use windows::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT};

    if ctrl_type == CTRL_C_EVENT || ctrl_type == CTRL_BREAK_EVENT {
        true.into()
    } else {
        false.into()
    }
}

/// Temporarily detach from the parent/debug console so Ctrl+C does not flash CMD.
#[cfg(windows)]
struct DetachConsole {
    had_console: bool,
}

#[cfg(windows)]
impl DetachConsole {
    fn detach() -> Self {
        use windows::Win32::System::Console::{FreeConsole, GetConsoleWindow};
        use windows::Win32::Foundation::HWND;

        let had_console = unsafe { GetConsoleWindow() != HWND::default() };
        if had_console {
            let _ = unsafe { FreeConsole() };
        }
        Self { had_console }
    }
}

#[cfg(windows)]
impl Drop for DetachConsole {
    fn drop(&mut self) {
        // Do not re-attach: re-attaching to the parent console mid-session can
        // bring the flash back on the next capture. Debug logs still go to the
        // original cargo/tauri terminal via inherited handles when present.
        let _ = self.had_console;
    }
}

/// Clipboard capture path used by the translate hotkey.
pub fn capture_payload() -> CapturePayload {
    match capture_selection() {
        Ok(text) => CapturePayload::ok(text, "clipboard"),
        Err(err) => CapturePayload::fail(err, "clipboard"),
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

    #[test]
    fn clipboard_payload_never_reports_ocr_source() {
        // Structural guard: clipboard path always tags source as clipboard.
        let ok = CapturePayload::ok("hi".into(), "clipboard");
        let fail = CapturePayload::fail("x", "clipboard");
        assert_eq!(ok.source, "clipboard");
        assert_eq!(fail.source, "clipboard");
        assert_ne!(ok.source, "ocr");
    }
}
