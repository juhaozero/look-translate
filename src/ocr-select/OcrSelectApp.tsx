import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { recognizeWithTesseract } from "./tesseractOcr";
import "./ocr-select.css";

type Region = {
  left: number;
  top: number;
  width: number;
  height: number;
};

type DragState = {
  originX: number;
  originY: number;
  currentX: number;
  currentY: number;
};

type OcrConfirmResult =
  | { kind: "done" }
  | { kind: "tesseract"; imageDataUrl: string };

const MIN_SELECT_PX = 4;

function normalizeDrag(drag: DragState): Region {
  const left = Math.min(drag.originX, drag.currentX);
  const top = Math.min(drag.originY, drag.currentY);
  const width = Math.abs(drag.currentX - drag.originX);
  const height = Math.abs(drag.currentY - drag.originY);
  return { left, top, width, height };
}

export function OcrSelectApp() {
  const [hint, setHint] = useState<Region | null>(null);
  const [drag, setDrag] = useState<DragState | null>(null);
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const confirming = useRef(false);

  const refreshHint = useCallback(async () => {
    confirming.current = false;
    setBusy(false);
    setDrag(null);
    setStatus(null);
    try {
      const win = getCurrentWindow();
      const scale = await win.scaleFactor();
      const pos = await win.outerPosition();
      const physical = await invoke<Region>("get_ocr_region_hint");
      setHint({
        left: (physical.left - pos.x) / scale,
        top: (physical.top - pos.y) / scale,
        width: physical.width / scale,
        height: physical.height / scale,
      });
    } catch (err) {
      console.error(err);
      setHint(null);
    }
  }, []);

  const cancel = useCallback(async () => {
    if (confirming.current) return;
    try {
      await invoke("cancel_ocr_select");
    } catch (err) {
      console.error(err);
    }
  }, []);

  const confirm = useCallback(async (region: Region) => {
    if (confirming.current) return;
    if (region.width < MIN_SELECT_PX || region.height < MIN_SELECT_PX) {
      setStatus("选区太小，请拖大一点或按 Enter 使用建议框");
      return;
    }
    confirming.current = true;
    setBusy(true);
    setStatus("识别中…");
    try {
      const win = getCurrentWindow();
      const scale = await win.scaleFactor();
      const pos = await win.outerPosition();
      const result = await invoke<OcrConfirmResult>("confirm_ocr_region", {
        region: {
          left: Math.round(pos.x + region.left * scale),
          top: Math.round(pos.y + region.top * scale),
          width: Math.max(1, Math.round(region.width * scale)),
          height: Math.max(1, Math.round(region.height * scale)),
        },
      });

      switch (result.kind) {
        case "done":
          break;
        case "tesseract": {
          const imageDataUrl =
            result.imageDataUrl ??
            (result as { image_data_url?: string }).image_data_url;
          if (!imageDataUrl) {
            throw new Error("未收到截图数据，请重试");
          }
          setStatus("Tesseract 识别中…");
          const text = await recognizeWithTesseract(imageDataUrl, (message) =>
            setStatus(message),
          );
          await invoke("submit_ocr_text", { text });
          break;
        }
        default: {
          const _exhaustive: never = result;
          return _exhaustive;
        }
      }
    } catch (err) {
      console.error(err);
      confirming.current = false;
      setBusy(false);
      const message =
        err instanceof Error && err.message.trim()
          ? err.message
          : "识别失败，请按 OCR 热键重试";
      setStatus(message);
    }
  }, []);

  useEffect(() => {
    void refreshHint();
    let unlisten: (() => void) | undefined;
    void listen("ocr-select-opened", () => {
      void refreshHint();
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, [refreshHint]);

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        void cancel();
        return;
      }
      if (event.key === "Enter") {
        event.preventDefault();
        const region = drag ? normalizeDrag(drag) : hint;
        if (region) void confirm(region);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [cancel, confirm, drag, hint]);

  const active = drag ? normalizeDrag(drag) : hint;

  return (
    <div
      className={`ocr-select-root${busy ? " is-busy" : ""}`}
      onMouseDown={(event) => {
        if (busy || event.button !== 0) return;
        setDrag({
          originX: event.clientX,
          originY: event.clientY,
          currentX: event.clientX,
          currentY: event.clientY,
        });
      }}
      onMouseMove={(event) => {
        if (!drag || busy) return;
        setDrag({ ...drag, currentX: event.clientX, currentY: event.clientY });
      }}
      onMouseUp={() => {
        if (!drag || busy) return;
        const region = normalizeDrag(drag);
        setDrag(null);
        if (region.width >= MIN_SELECT_PX && region.height >= MIN_SELECT_PX) {
          setHint(region);
          void confirm(region);
        } else {
          setStatus("选区太小，请拖大一点或按 Enter 使用建议框");
        }
      }}
      onContextMenu={(event) => {
        event.preventDefault();
        void cancel();
      }}
    >
      <div className="ocr-select-hint">
        拖拽选择识别区域 · Enter 确认建议框 · Esc 取消
        {busy || status ? ` · ${status ?? "识别中…"}` : null}
      </div>
      {active ? (
        <div
          className="ocr-select-rect"
          style={{
            left: active.left,
            top: active.top,
            width: active.width,
            height: active.height,
          }}
        />
      ) : null}
    </div>
  );
}
