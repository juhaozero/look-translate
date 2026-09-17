import { createWorker, type Worker } from "tesseract.js";

let workerPromise: Promise<Worker> | null = null;

/**
 * Serve worker/core from the same origin (Vite/Tauri), not jsDelivr.
 * Language data still downloads from CDN on first use (cached by tesseract.js).
 */
function tesseractAsset(file: string): string {
  const base = import.meta.env.BASE_URL?.replace(/\/?$/, "/") ?? "/";
  return `${base}tesseract/${file}`;
}

/** Decode a PNG/JPEG data URL into a Blob (avoids tesseract data-URL path bugs). */
async function dataUrlToBlob(dataUrl: string): Promise<Blob> {
  if (typeof dataUrl !== "string" || !dataUrl) {
    throw new Error("无效的图像数据");
  }
  const comma = dataUrl.indexOf(",");
  if (!dataUrl.startsWith("data:") || comma < 0) {
    throw new Error("无效的图像数据");
  }
  const meta = dataUrl.slice(5, comma); // e.g. image/png;base64
  const payload = dataUrl.slice(comma + 1);
  const mimeMatch = /^([^;]+)/.exec(meta);
  const mime = mimeMatch?.[1] ?? "image/png";

  if (meta.includes(";base64")) {
    const binary = atob(payload);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i += 1) {
      bytes[i] = binary.charCodeAt(i);
    }
    return new Blob([bytes], { type: mime });
  }

  return new Blob([decodeURIComponent(payload)], { type: mime });
}

async function getWorker(
  onStatus?: (message: string) => void,
): Promise<Worker> {
  if (!workerPromise) {
    workerPromise = (async () => {
      onStatus?.("加载 Tesseract 引擎…");
      const worker = await createWorker("eng+chi_sim", 1, {
        workerPath: tesseractAsset("worker.min.js"),
        // Directory URL: tesseract.js picks simd/lstm wasm.js under it.
        corePath: tesseractAsset(""),
        workerBlobURL: false,
        logger: (message) => {
          if (message.status === "loading tesseract core") {
            onStatus?.("加载识别核心…");
          } else if (message.status === "loading language traineddata") {
            onStatus?.("下载语言模型（首次较慢）…");
          } else if (message.status === "initializing api") {
            onStatus?.("初始化识别引擎…");
          } else if (message.status === "recognizing text") {
            const progress =
              typeof message.progress === "number"
                ? ` ${Math.round(message.progress * 100)}%`
                : "";
            onStatus?.(`识别中${progress}…`);
          }
        },
      });
      return worker;
    })().catch((err) => {
      workerPromise = null;
      throw err;
    });
  }
  return workerPromise;
}

function formatTesseractError(err: unknown): string {
  const raw = err instanceof Error ? err.message : String(err ?? "");
  const lower = raw.toLowerCase();
  if (lower.includes("join is not a function")) {
    return "Tesseract 图像处理失败，请更新后重试或改用系统 OCR";
  }
  if (
    lower.includes("failed to fetch") ||
    lower.includes("network") ||
    lower.includes("load") ||
    lower.includes("traineddata")
  ) {
    return "Tesseract 资源加载失败（需联网下载语言模型，请检查网络后重试）";
  }
  return raw ? `Tesseract 识别失败：${raw}` : "Tesseract 识别失败";
}

/** Recognize text from a PNG/JPEG data URL via Tesseract.js WASM. */
export async function recognizeWithTesseract(
  imageDataUrl: string,
  onStatus?: (message: string) => void,
): Promise<string> {
  try {
    const worker = await getWorker(onStatus);
    onStatus?.("识别中…");
    // Pass Blob instead of data URL — tesseract.js setImage assumes array-like
    // bytes; the data-URL path can leave a string and throw
    // `slice(...).join is not a function` in WebView.
    const blob = await dataUrlToBlob(imageDataUrl);
    const result = await worker.recognize(blob);
    return result.data.text ?? "";
  } catch (err) {
    throw new Error(formatTesseractError(err));
  }
}
