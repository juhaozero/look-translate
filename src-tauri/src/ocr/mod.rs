//! Local OCR capture via a **dedicated hotkey**.
//!
//! This path never falls back to clipboard selection capture, and the
//! clipboard path never falls back here.
//!
//! Flow: hotkey → snapshot monitor (before overlay) → user picks region →
//! crop from snapshot → OCR. Never BitBlt after the overlay is shown.

use std::sync::Mutex;

use serde::Serialize;

use crate::capture::{normalize_captured_text, CapturePayload};

/// Default suggested capture rectangle around the cursor (physical pixels).
pub const OCR_WIDTH: i32 = 480;
pub const OCR_HEIGHT: i32 = 160;

/// Suggested OCR region in physical screen pixels (for the select overlay).
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcrRegionHint {
    pub left: i32,
    pub top: i32,
    pub width: i32,
    pub height: i32,
}

/// One OCR pick session: full-monitor snapshot taken before the overlay appears.
#[derive(Debug, Clone)]
pub struct OcrSession {
    pub monitor_left: i32,
    pub monitor_top: i32,
    pub monitor_width: i32,
    pub monitor_height: i32,
    pub pixels: Vec<u8>,
    pub hint: OcrRegionHint,
}

#[derive(Default)]
pub struct OcrSessionState {
    inner: Mutex<Option<OcrSession>>,
}

impl OcrSessionState {
    pub fn store(&self, session: OcrSession) {
        if let Ok(mut guard) = self.inner.lock() {
            *guard = Some(session);
        }
    }

    pub fn hint(&self) -> Option<OcrRegionHint> {
        self.inner
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|s| s.hint))
    }

    pub fn take(&self) -> Option<OcrSession> {
        self.inner.lock().ok().and_then(|mut guard| guard.take())
    }

    pub fn clear(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            *guard = None;
        }
    }
}

/// Capture the monitor under the cursor and build a session (call before overlay).
pub fn begin_ocr_session(
    monitor_left: i32,
    monitor_top: i32,
    monitor_width: u32,
    monitor_height: u32,
) -> Result<OcrSession, String> {
    #[cfg(windows)]
    {
        begin_ocr_session_windows(monitor_left, monitor_top, monitor_width, monitor_height)
    }
    #[cfg(not(windows))]
    {
        let _ = (monitor_left, monitor_top, monitor_width, monitor_height);
        Err("当前仅支持 Windows OCR".into())
    }
}

#[cfg(windows)]
fn begin_ocr_session_windows(
    monitor_left: i32,
    monitor_top: i32,
    monitor_width: u32,
    monitor_height: u32,
) -> Result<OcrSession, String> {
    let width = monitor_width as i32;
    let height = monitor_height as i32;
    if width <= 0 || height <= 0 {
        return Err("显示器尺寸无效".into());
    }

    let pixels = capture_bgra_region(monitor_left, monitor_top, width, height)?;
    let (cx, cy) = cursor_pos()?;
    let (left, top, rw, rh) = clamp_rect_around(cx, cy, OCR_WIDTH, OCR_HEIGHT);
    // Keep hint inside this monitor (overlay only covers one display).
    let (left, top, rw, rh) =
        clamp_rect_to_monitor(left, top, rw, rh, monitor_left, monitor_top, width, height);

    Ok(OcrSession {
        monitor_left,
        monitor_top,
        monitor_width: width,
        monitor_height: height,
        pixels,
        hint: OcrRegionHint {
            left,
            top,
            width: rw,
            height: rh,
        },
    })
}

/// Crop (+ pad + upscale) from session for OCR backends.
pub fn prepare_region_image(
    session: &OcrSession,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
) -> Result<(Vec<u8>, u32, u32), String> {
    let (left, top, width, height) = clamp_rect_to_monitor(
        left,
        top,
        width,
        height,
        session.monitor_left,
        session.monitor_top,
        session.monitor_width,
        session.monitor_height,
    );
    if width < 4 || height < 4 {
        return Err("选区太小，请拖出更大范围".into());
    }

    let (left, top, width, height) = expand_rect_in_monitor(
        left,
        top,
        width,
        height,
        session.monitor_left,
        session.monitor_top,
        session.monitor_width,
        session.monitor_height,
        6,
    );

    let crop = crop_bgra(
        &session.pixels,
        session.monitor_width,
        session.monitor_height,
        left - session.monitor_left,
        top - session.monitor_top,
        width,
        height,
    )?;
    Ok(upscale_bgra_for_ocr(&crop, width as u32, height as u32))
}

/// Encode BGRA8 as a PNG data URL (`data:image/png;base64,...`).
pub fn bgra_to_png_data_url(pixels: &[u8], width: u32, height: u32) -> Result<String, String> {
    use base64::Engine as _;
    use image::ImageEncoder;

    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|n| n.checked_mul(4))
        .ok_or_else(|| "图像尺寸无效".to_string())?;
    if pixels.len() != expected {
        return Err("像素缓冲与尺寸不匹配".into());
    }

    // PNG expects RGBA; our capture is BGRA.
    let mut rgba = Vec::with_capacity(pixels.len());
    for chunk in pixels.chunks_exact(4) {
        rgba.extend_from_slice(&[chunk[2], chunk[1], chunk[0], chunk[3]]);
    }

    let mut png = Vec::new();
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(
            &rgba,
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| format!("PNG 编码失败: {e}"))?;

    let b64 = base64::engine::general_purpose::STANDARD.encode(png);
    Ok(format!("data:image/png;base64,{b64}"))
}

/// Crop the session snapshot to `region` (physical screen coords) and OCR (system).
pub fn capture_ocr_payload_from_session(
    session: &OcrSession,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
) -> CapturePayload {
    match ocr_from_session(session, left, top, width, height) {
        Ok(text) => CapturePayload::ok(text, "ocr"),
        Err(err) => CapturePayload::fail(err, "ocr"),
    }
}

fn ocr_from_session(
    session: &OcrSession,
    left: i32,
    top: i32,
    width: i32,
    height: i32,
) -> Result<String, String> {
    let (pixels, ow, oh) = prepare_region_image(session, left, top, width, height)?;

    #[cfg(windows)]
    {
        let text = recognize_bgra(&pixels, ow, oh)?;
        normalize_captured_text(&text).map_err(|_| {
            "OCR 未识别到文字（可调整选区后重试）".to_string()
        })
    }
    #[cfg(not(windows))]
    {
        let _ = (pixels, ow, oh);
        Err("当前仅支持 Windows OCR".into())
    }
}

fn expand_rect_in_monitor(
    left: i32,
    top: i32,
    width: i32,
    height: i32,
    ml: i32,
    mt: i32,
    mw: i32,
    mh: i32,
    pad: i32,
) -> (i32, i32, i32, i32) {
    clamp_rect_to_monitor(
        left - pad,
        top - pad,
        width + pad * 2,
        height + pad * 2,
        ml,
        mt,
        mw,
        mh,
    )
}

/// Nearest-neighbor upscale so short sides meet Windows OCR's practical minimum.
fn upscale_bgra_for_ocr(src: &[u8], w: u32, h: u32) -> (Vec<u8>, u32, u32) {
    const MIN_SIDE: u32 = 64;
    const MIN_LONG: u32 = 240;
    const MAX_DIM: u32 = 2048;

    if w == 0 || h == 0 {
        return (src.to_vec(), w, h);
    }

    let needs =
        w < MIN_SIDE || h < MIN_SIDE || w.max(h) < MIN_LONG;
    if !needs {
        return (src.to_vec(), w, h);
    }

    let scale = (MIN_SIDE as f64 / f64::from(w.min(h)))
        .max(MIN_LONG as f64 / f64::from(w.max(h)))
        .max(1.0);
    let mut nw = (f64::from(w) * scale).ceil() as u32;
    let mut nh = (f64::from(h) * scale).ceil() as u32;
    if nw > MAX_DIM || nh > MAX_DIM {
        let shrink = f64::from(MAX_DIM) / f64::from(nw.max(nh));
        nw = (f64::from(nw) * shrink).floor().max(1.0) as u32;
        nh = (f64::from(nh) * shrink).floor().max(1.0) as u32;
    }
    if nw == w && nh == h {
        return (src.to_vec(), w, h);
    }

    let src_stride = (w * 4) as usize;
    let dst_stride = (nw * 4) as usize;
    let mut out = vec![0u8; dst_stride * nh as usize];
    for y in 0..nh as usize {
        let sy = (y as u64 * h as u64 / nh as u64) as usize;
        let src_row = sy.min(h as usize - 1) * src_stride;
        let dst_row = y * dst_stride;
        for x in 0..nw as usize {
            let sx = (x as u64 * w as u64 / nw as u64) as usize;
            let s = src_row + sx.min(w as usize - 1) * 4;
            let d = dst_row + x * 4;
            out[d..d + 4].copy_from_slice(&src[s..s + 4]);
        }
    }
    (out, nw, nh)
}

fn crop_bgra(
    src: &[u8],
    src_w: i32,
    src_h: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> Result<Vec<u8>, String> {
    if x < 0 || y < 0 || w <= 0 || h <= 0 || x + w > src_w || y + h > src_h {
        return Err("选区超出截屏范围".into());
    }
    let src_stride = (src_w * 4) as usize;
    let dst_stride = (w * 4) as usize;
    let mut out = vec![0u8; dst_stride * h as usize];
    for row in 0..h as usize {
        let src_off = (y as usize + row) * src_stride + (x as usize) * 4;
        let dst_off = row * dst_stride;
        out[dst_off..dst_off + dst_stride]
            .copy_from_slice(&src[src_off..src_off + dst_stride]);
    }
    Ok(out)
}

fn clamp_rect_to_monitor(
    left: i32,
    top: i32,
    width: i32,
    height: i32,
    ml: i32,
    mt: i32,
    mw: i32,
    mh: i32,
) -> (i32, i32, i32, i32) {
    let right = (left + width).clamp(ml, ml + mw);
    let bottom = (top + height).clamp(mt, mt + mh);
    let left = left.clamp(ml, ml + mw);
    let top = top.clamp(mt, mt + mh);
    (left, top, (right - left).max(0), (bottom - top).max(0))
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
fn virtual_desktop() -> (i32, i32, i32, i32) {
    use windows::Win32::UI::WindowsAndMessaging::{
        GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
        SM_YVIRTUALSCREEN,
    };

    let vx = unsafe { GetSystemMetrics(SM_XVIRTUALSCREEN) };
    let vy = unsafe { GetSystemMetrics(SM_YVIRTUALSCREEN) };
    let vw = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) }.max(1);
    let vh = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) }.max(1);
    (vx, vy, vw, vh)
}

#[cfg(windows)]
fn clamp_rect_around(cx: i32, cy: i32, w: i32, h: i32) -> (i32, i32, i32, i32) {
    let (vx, vy, vw, vh) = virtual_desktop();
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
        BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
        GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        HGDIOBJ, SRCCOPY,
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

    #[test]
    fn crop_bgra_copies_sub_rect() {
        // 2x2 BGRA, crop bottom-right pixel
        let src = vec![
            1, 0, 0, 255, 2, 0, 0, 255, //
            3, 0, 0, 255, 4, 0, 0, 255,
        ];
        let out = crop_bgra(&src, 2, 2, 1, 1, 1, 1).unwrap();
        assert_eq!(out, vec![4, 0, 0, 255]);
    }

    #[test]
    fn upscale_bgra_grows_small_image() {
        let src = vec![10u8, 20, 30, 255]; // 1x1
        let (out, w, h) = upscale_bgra_for_ocr(&src, 1, 1);
        assert!(w >= 64);
        assert!(h >= 64);
        assert_eq!(out.len(), (w * h * 4) as usize);
        assert_eq!(&out[0..4], &[10, 20, 30, 255]);
    }

    #[test]
    fn clamp_rect_to_monitor_keeps_inside() {
        let (l, t, w, h) = clamp_rect_to_monitor(-10, -10, 100, 50, 0, 0, 80, 40);
        assert_eq!((l, t, w, h), (0, 0, 80, 40));
    }
}
