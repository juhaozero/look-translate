//! Local OCR capture via a **dedicated hotkey**.
//!
//! This path never falls back to clipboard selection capture, and the
//! clipboard path never falls back here.

use crate::capture::{normalize_captured_text, CapturePayload};

/// Fixed capture rectangle around the cursor (physical pixels).
pub const OCR_WIDTH: i32 = 480;
pub const OCR_HEIGHT: i32 = 160;

/// Run OCR near the cursor and wrap into a capture payload.
pub fn capture_ocr_payload() -> CapturePayload {
    match capture_near_cursor() {
        Ok(text) => CapturePayload::ok(text, "ocr"),
        Err(err) => CapturePayload::fail(err, "ocr"),
    }
}

/// Screenshot a fixed rectangle around the cursor, then OCR.
/// Does **not** touch the clipboard or simulate Ctrl+C.
pub fn capture_near_cursor() -> Result<String, String> {
    #[cfg(windows)]
    {
        capture_near_cursor_windows()
    }
    #[cfg(not(windows))]
    {
        Err("当前仅支持 Windows OCR".into())
    }
}

#[cfg(windows)]
fn capture_near_cursor_windows() -> Result<String, String> {
    let (px, py) = cursor_pos()?;
    let (left, top, width, height) = clamp_rect_around(px, py, OCR_WIDTH, OCR_HEIGHT);
    let pixels = capture_bgra_region(left, top, width, height)?;
    let text = recognize_bgra(&pixels, width as u32, height as u32)?;
    normalize_captured_text(&text).map_err(|_| {
        "OCR 未识别到文字（可调整指针位置后重试）".to_string()
    })
}

#[cfg(windows)]
fn cursor_pos() -> Result<(i32, i32), String> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point) }.map_err(|e| format!("读取光标位置失败: {e}"))?;
    Ok((point.x, point.y))
}

#[cfg(windows)]
fn clamp_rect_around(cx: i32, cy: i32, w: i32, h: i32) -> (i32, i32, i32, i32) {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    };

    // Virtual desktop bounds (multi-monitor); may start at negative coords.
    let vx = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let vy = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
    let vw = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) }.max(1);
    let vh = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) }.max(1);

    let width = w.min(vw);
    let height = h.min(vh);
    let max_left = vx + vw - width;
    let max_top = vy + vh - height;

    let left = (cx - w / 2).clamp(vx, max_left);
    let top = (cy - h / 2).clamp(vy, max_top);
    (left, top, width, height)
}

#[cfg(windows)]
fn capture_bgra_region(
    left: i32,
    top: i32,
    width: i32,
    height: i32,
) -> Result<Vec<u8>, String> {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Gdi::{
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject,
        GetDC, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        GetDIBits, HGDIOBJ, SRCCOPY,
    };

    if width <= 0 || height <= 0 {
        return Err("OCR 截取区域无效".into());
    }

    unsafe {
        let screen_dc = GetDC(Some(HWND::default()));
        if screen_dc.is_invalid() {
            return Err("获取屏幕 DC 失败".into());
        }
        let mem_dc = CreateCompatibleDC(Some(screen_dc));
        if mem_dc.is_invalid() {
            ReleaseDC(Some(HWND::default()), screen_dc);
            return Err("创建兼容 DC 失败".into());
        }
        let bitmap = CreateCompatibleBitmap(screen_dc, width, height);
        if bitmap.is_invalid() {
            let _ = DeleteDC(mem_dc);
            ReleaseDC(Some(HWND::default()), screen_dc);
            return Err("创建位图失败".into());
        }
        let old = SelectObject(mem_dc, HGDIOBJ(bitmap.0));
        let blit_ok = BitBlt(mem_dc, 0, 0, width, height, Some(screen_dc), left, top, SRCCOPY);
        if blit_ok.is_err() {
            SelectObject(mem_dc, old);
            let _ = DeleteObject(HGDIOBJ(bitmap.0));
            let _ = DeleteDC(mem_dc);
            ReleaseDC(Some(HWND::default()), screen_dc);
            return Err("截屏 BitBlt 失败".into());
        }

        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let stride = (width * 4) as usize;
        let mut pixels = vec![0u8; stride * height as usize];
        let lines = GetDIBits(
            mem_dc,
            bitmap,
            0,
            height as u32,
            Some(pixels.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        );

        SelectObject(mem_dc, old);
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(mem_dc);
        ReleaseDC(Some(HWND::default()), screen_dc);

        if lines == 0 {
            return Err("读取截图像素失败".into());
        }

        // Ensure alpha is opaque for OCR.
        for chunk in pixels.chunks_exact_mut(4) {
            chunk[3] = 255;
        }

        Ok(pixels)
    }
}

#[cfg(windows)]
fn recognize_bgra(pixels: &[u8], width: u32, height: u32) -> Result<String, String> {
    use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
    use windows::Media::Ocr::OcrEngine;
    use windows::Storage::Streams::DataWriter;

    let writer = DataWriter::new().map_err(|e| format!("DataWriter 创建失败: {e}"))?;
    writer
        .WriteBytes(pixels)
        .map_err(|e| format!("写入像素失败: {e}"))?;
    let buffer = writer
        .DetachBuffer()
        .map_err(|e| format!("DetachBuffer 失败: {e}"))?;

    let bitmap = SoftwareBitmap::CreateCopyFromBuffer(
        &buffer,
        BitmapPixelFormat::Bgra8,
        width as i32,
        height as i32,
    )
    .map_err(|e| format!("创建 SoftwareBitmap 失败: {e}"))?;

    let engine = OcrEngine::TryCreateFromUserProfileLanguages()
        .map_err(|e| format!("创建 OCR 引擎失败（请确认已安装系统 OCR 语言包）: {e}"))?;

    let result = engine
        .RecognizeAsync(&bitmap)
        .map_err(|e| format!("启动 OCR 失败: {e}"))?
        .get()
        .map_err(|e| format!("OCR 识别失败: {e}"))?;

    let text = result
        .Text()
        .map_err(|e| format!("读取 OCR 结果失败: {e}"))?
        .to_string();

    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ocr_payload_source_is_ocr() {
        let fail = CapturePayload::fail("x", "ocr");
        assert_eq!(fail.source, "ocr");
        assert_ne!(fail.source, "clipboard");
    }

    #[test]
    fn ocr_rect_constants_are_positive() {
        assert!(OCR_WIDTH > 0);
        assert!(OCR_HEIGHT > 0);
    }
}
