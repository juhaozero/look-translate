//! Clipboard selection capture for the translate hotkey.
//!
//! OCR is intentionally **not** used as a fallback here — it has its own
//! hotkey and module (`crate::ocr`).

use std::sync::RwLock;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;

pub const MAX_CAPTURE_CHARS: usize = 8_000;
/// Apps often deliver clipboard text asynchronously after Ctrl+C / Ctrl+Insert.
pub const CAPTURE_TIMEOUT: Duration = Duration::from_millis(1_500);
/// Poll sequence number without opening the clipboard (avoids lock races).
pub const CAPTURE_POLL: Duration = Duration::from_millis(15);
/// Wait for Shift/Alt/Win to lift — leftover Shift turns Ctrl+C into Chrome DevTools.
pub const CONFLICTING_MODIFIER_WAIT: Duration = Duration::from_millis(1_500);
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

/// Prefer UI Automation (Chrome/Edge/VS Code); fall back to clipboard Ctrl+Insert/Ctrl+C.
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
    // UIA reads the selection without synthesizing keys — avoids Chrome treating
    // leftover Shift+Ctrl+C as DevTools when the hotkey is Ctrl+Shift+*.
    if let Some(text) = capture_selection_uia() {
        return normalize_captured_text(&text);
    }

    capture_selection_clipboard()
}

/// Read selected text via UI Automation TextPattern (focused element + ancestors).
#[cfg(windows)]
fn capture_selection_uia() -> Option<String> {
    use uiautomation::UIAutomation;

    let automation = UIAutomation::new().ok()?;
    let focused = automation.get_focused_element().ok()?;

    if let Some(text) = try_uia_selection(&focused) {
        return Some(text);
    }

    // Web pages often focus a wrapper; selection lives on an ancestor Document.
    let walker = automation.create_tree_walker().ok()?;
    let mut current = focused;
    for _ in 0..8 {
        current = walker.get_parent(&current).ok()?;
        if let Some(text) = try_uia_selection(&current) {
            return Some(text);
        }
    }
    None
}

#[cfg(windows)]
fn try_uia_selection(element: &uiautomation::UIElement) -> Option<String> {
    use uiautomation::patterns::UITextPattern;

    let pattern: UITextPattern = element.get_pattern().ok()?;
    let ranges = pattern.get_selection().ok()?;
    let mut combined = String::new();
    for range in &ranges {
        if let Ok(text) = range.get_text(-1) {
            combined.push_str(&text);
        }
    }
    let trimmed = combined.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(windows)]
fn capture_selection_clipboard() -> Result<String, String> {
    use arboard::Clipboard;
    use enigo::{Enigo, Settings};

    let mut clipboard =
        Clipboard::new().map_err(|e| format!("打开剪贴板失败: {e}"))?;

    // Text-only backup (non-text clipboard content may not survive restore).
    let previous_text = clipboard.get_text().ok();

    let marker = format!("{MARKER_PREFIX}{}", now_ms());
    clipboard
        .set_text(marker.clone())
        .map_err(|e| format!("写入剪贴板标记失败: {e}"))?;

    // Settle so the marker is committed before we send copy shortcuts.
    thread::sleep(Duration::from_millis(40));
    let baseline_seq = clipboard_sequence_number();

    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| format!("初始化按键模拟失败: {e}"))?;

    // Detach from any console so synthesized Ctrl+C is not delivered as
    // CTRL_C_EVENT to this process (debug `cargo run` / `tauri:dev` console flash).
    let _console_guard = DetachConsole::detach();
    let _suppress_ctrl_c = SuppressConsoleCtrlC::install();

    // Default hotkey is Ctrl+Shift+D. Released often fires while Shift is still
    // held; Ctrl+Shift+C in Chrome/Edge opens DevTools instead of copying.
    // Control may stay held — that is fine for a subsequent Ctrl+C chord.
    release_conflicting_modifiers(&mut enigo);
    let modifiers_clear = wait_for_conflicting_modifiers_up(CONFLICTING_MODIFIER_WAIT);
    release_conflicting_modifiers(&mut enigo);
    thread::sleep(Duration::from_millis(30));

    if !modifiers_clear && conflicting_modifiers_down() {
        eprintln!(
            "[look-translate] Shift/Alt/Win still down after wait; copy may fail in browsers"
        );
    }

    // Prefer Ctrl+Insert (less likely to collide with browser chords), then Ctrl+C.
    send_ctrl_insert(&mut enigo)?;
    let mut changed = wait_for_clipboard_seq_change(baseline_seq, CAPTURE_TIMEOUT / 2);

    if !changed {
        release_conflicting_modifiers(&mut enigo);
        if conflicting_modifiers_down() {
            // One more short wait — critical for Chrome DevTools collision.
            let _ = wait_for_conflicting_modifiers_up(Duration::from_millis(500));
            release_conflicting_modifiers(&mut enigo);
        }
        thread::sleep(Duration::from_millis(40));
        let retry_baseline = clipboard_sequence_number();
        send_ctrl_c(&mut enigo)?;
        changed = wait_for_clipboard_seq_change(retry_baseline, CAPTURE_TIMEOUT / 2);
    }

    let raw = match read_clipboard_after_copy(&mut clipboard, &marker, changed) {
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
fn release_conflicting_modifiers(enigo: &mut enigo::Enigo) {
    use enigo::{Direction::Release, Key, Keyboard};

    // Do NOT release Control here: user may still hold it from the hotkey, and
    // a synthetic Control-up while physically held confuses later Ctrl+C.
    for key in [Key::LShift, Key::RShift, Key::Alt, Key::Meta] {
        let _ = enigo.key(key, Release);
    }
}

#[cfg(windows)]
fn conflicting_modifiers_down() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_MENU, VK_RMENU, VK_RSHIFT, VK_RWIN,
        VK_SHIFT,
    };

    [
        VK_SHIFT, VK_LSHIFT, VK_RSHIFT, VK_MENU, VK_LMENU, VK_RMENU, VK_LWIN, VK_RWIN,
    ]
    .iter()
    .any(|vk| unsafe { GetAsyncKeyState(vk.0 as i32) } < 0)
}

/// Wait until Shift/Alt/Win are physically up (Control may remain held).
#[cfg(windows)]
fn wait_for_conflicting_modifiers_up(timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if !conflicting_modifiers_down() {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    !conflicting_modifiers_down()
}

#[cfg(windows)]
fn send_ctrl_insert(enigo: &mut enigo::Enigo) -> Result<(), String> {
    use enigo::{
        Direction::{Click, Press, Release},
        Key, Keyboard,
    };

    enigo
        .key(Key::LControl, Press)
        .or_else(|_| enigo.key(Key::Control, Press))
        .map_err(|e| format!("模拟 Ctrl 按下失败: {e}"))?;
    thread::sleep(Duration::from_millis(10));
    enigo
        .key(Key::Insert, Click)
        .map_err(|e| format!("模拟 Insert 失败: {e}"))?;
    thread::sleep(Duration::from_millis(10));
    enigo
        .key(Key::LControl, Release)
        .or_else(|_| enigo.key(Key::Control, Release))
        .map_err(|e| format!("模拟 Ctrl 松开失败: {e}"))?;
    Ok(())
}

#[cfg(windows)]
fn send_ctrl_c(enigo: &mut enigo::Enigo) -> Result<(), String> {
    use enigo::{
        Direction::{Click, Press, Release},
        Key, Keyboard,
    };

    // IMPORTANT: do NOT use Key::Unicode('c') — on Windows that becomes
    // KEYEVENTF_UNICODE / VK_PACKET and does not trigger Ctrl+C copy.
    enigo
        .key(Key::LControl, Press)
        .or_else(|_| enigo.key(Key::Control, Press))
        .map_err(|e| format!("模拟 Ctrl 按下失败: {e}"))?;
    thread::sleep(Duration::from_millis(10));
    enigo
        .key(Key::C, Click)
        .map_err(|e| format!("模拟 C 失败: {e}"))?;
    thread::sleep(Duration::from_millis(10));
    enigo
        .key(Key::LControl, Release)
        .or_else(|_| enigo.key(Key::Control, Release))
        .map_err(|e| format!("模拟 Ctrl 松开失败: {e}"))?;
    Ok(())
}

#[cfg(windows)]
fn clipboard_sequence_number() -> u32 {
    unsafe { windows::Win32::System::DataExchange::GetClipboardSequenceNumber() }
}

/// Poll `GetClipboardSequenceNumber` only — never opens the clipboard.
#[cfg(windows)]
fn wait_for_clipboard_seq_change(baseline: u32, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    thread::sleep(Duration::from_millis(50));
    while Instant::now() < deadline {
        let seq = clipboard_sequence_number();
        if seq != 0 && seq != baseline {
            thread::sleep(Duration::from_millis(20));
            return true;
        }
        thread::sleep(CAPTURE_POLL);
    }
    false
}

#[cfg(windows)]
fn read_clipboard_after_copy(
    clipboard: &mut arboard::Clipboard,
    marker: &str,
    seq_changed: bool,
) -> Result<String, String> {
    if !seq_changed {
        return Err(
            "取词超时：未检测到新的剪贴板内容（请确认已选中文本，且目标窗口支持 Ctrl+C；浏览器请先松开 Shift）"
                .into(),
        );
    }

    let deadline = Instant::now() + Duration::from_millis(400);
    let mut saw_marker = false;
    while Instant::now() < deadline {
        match clipboard.get_text() {
            Ok(text) if text != marker => {
                if text.trim().is_empty() {
                    thread::sleep(CAPTURE_POLL);
                    continue;
                }
                return Ok(text);
            }
            Ok(_) => {
                saw_marker = true;
            }
            Err(_) => {}
        }
        thread::sleep(CAPTURE_POLL);
    }

    if saw_marker {
        Err(
            "取词超时：未检测到新的剪贴板内容（请确认已选中文本，且目标窗口支持 Ctrl+C；浏览器请先松开 Shift）"
                .into(),
        )
    } else {
        Err("取词超时：剪贴板无响应".into())
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
        use windows::Win32::Foundation::HWND;
        use windows::Win32::System::Console::{FreeConsole, GetConsoleWindow};

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
        let ok = CapturePayload::ok("hi".into(), "clipboard");
        let fail = CapturePayload::fail("x", "clipboard");
        assert_eq!(ok.source, "clipboard");
        assert_eq!(fail.source, "clipboard");
        assert_ne!(ok.source, "ocr");
    }
}
