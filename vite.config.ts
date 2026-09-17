import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
// @ts-expect-error no @types/node in this project
import fs from "node:fs";
// @ts-expect-error no @types/node in this project
import path from "node:path";
// @ts-expect-error no @types/node in this project
import process from "node:process";
// @ts-expect-error no @types/node in this project
import { fileURLToPath } from "node:url";

const host = process.env.TAURI_DEV_HOST;
const rootDir = path.dirname(fileURLToPath(import.meta.url));

/** Copy Tesseract worker/core into public/ so the OCR webview loads same-origin assets. */
function copyTesseractAssets(): Plugin {
  const files = [
    ["tesseract.js/dist/worker.min.js", "worker.min.js"],
    [
      "tesseract.js-core/tesseract-core-simd-lstm.wasm.js",
      "tesseract-core-simd-lstm.wasm.js",
    ],
    [
      "tesseract.js-core/tesseract-core-simd-lstm.wasm",
      "tesseract-core-simd-lstm.wasm",
    ],
    ["tesseract.js-core/tesseract-core-lstm.wasm.js", "tesseract-core-lstm.wasm.js"],
    ["tesseract.js-core/tesseract-core-lstm.wasm", "tesseract-core-lstm.wasm"],
  ] as const;

  const sync = () => {
    const destDir = path.join(rootDir, "public", "tesseract");
    fs.mkdirSync(destDir, { recursive: true });
    for (const [from, name] of files) {
      const src = path.join(rootDir, "node_modules", from);
      if (!fs.existsSync(src)) {
        console.warn(`[tesseract] missing asset: ${from}`);
        continue;
      }
      fs.copyFileSync(src, path.join(destDir, name));
    }

    // tesseract.js setImage does `image.slice(0,500).join(" ")`.
    // In some WebViews the cloned payload is ArrayBuffer/Uint8Array-like without
    // `.join`, which throws during recognize. Normalize via Array.from.
    const workerFile = path.join(destDir, "worker.min.js");
    if (fs.existsSync(workerFile)) {
      const original = fs.readFileSync(workerFile, "utf8");
      const patched = original.replace(
        /([a-zA-Z_$][\w$]*)\.slice\(0,500\)\.join\(" "\)/g,
        "Array.from($1).slice(0,500).join(\" \")",
      );
      if (patched === original) {
        console.warn("[tesseract] worker setImage join patch not applied");
      } else {
        fs.writeFileSync(workerFile, patched);
      }
    }
  };

  return {
    name: "copy-tesseract-assets",
    buildStart: sync,
    configureServer() {
      sync();
    },
  };
}

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [react(), copyTesseractAssets()],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
