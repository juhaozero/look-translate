import type { AppConfig } from "../shared/types";
import {
  BUILTIN_ENGINE_IDS,
  normalizeLangCode,
  resolveActives,
} from "../shared/options";

export function normalizeOptionalKey(
  value: string | null | undefined,
): string | null {
  const trimmed = (value ?? "").trim();
  return trimmed.length > 0 ? trimmed : null;
}

export function normalizeConfig(config: AppConfig): AppConfig {
  const ocrEngine =
    config.ocr?.engine?.trim().toLowerCase() === "tesseract"
      ? "tesseract"
      : "system";
  const actives = resolveActives(config.engine);
  return {
    ...config,
    general: {
      ...config.general,
      target_lang: normalizeLangCode(config.general.target_lang) || "zh-CN",
      source_lang: normalizeLangCode(config.general.source_lang) || "auto",
      hotkey_translate: config.general.hotkey_translate.trim() || "Ctrl+Shift+D",
      hotkey_input: config.general.hotkey_input?.trim() || "Ctrl+Shift+T",
      launch_at_startup: Boolean(config.general.launch_at_startup),
    },
    engine: {
      ...config.engine,
      actives,
      active: actives[0] || "microsoft",
      microsoft_api_key: normalizeOptionalKey(config.engine.microsoft_api_key),
      microsoft_region: normalizeOptionalKey(config.engine.microsoft_region),
      google_api_key: normalizeOptionalKey(config.engine.google_api_key),
      deepl_api_key: normalizeOptionalKey(config.engine.deepl_api_key),
      cloudflare_endpoint: normalizeOptionalKey(
        config.engine.cloudflare_endpoint,
      ),
      cloudflare_secret: normalizeOptionalKey(config.engine.cloudflare_secret),
      baidu_app_id: normalizeOptionalKey(config.engine.baidu_app_id),
      baidu_secret: normalizeOptionalKey(config.engine.baidu_secret),
      youdao_app_key: normalizeOptionalKey(config.engine.youdao_app_key),
      youdao_app_secret: normalizeOptionalKey(config.engine.youdao_app_secret),
    },
    ocr: {
      engine: ocrEngine,
    },
    dictionary: {
      ...config.dictionary,
      paths: config.dictionary.paths.map((p) => p.trim()).filter(Boolean),
    },
  };
}

export function serializeConfig(config: AppConfig): string {
  return JSON.stringify(normalizeConfig(config));
}

export function validateConfig(config: AppConfig): string | null {
  if (!config.general.hotkey_input.trim()) {
    return "输入翻译热键不能为空";
  }
  const shortcuts = [config.general.hotkey_translate, config.general.hotkey_ocr,
    config.general.hotkey_input].map((s) => s.trim().toLowerCase()).filter(Boolean);
  if (new Set(shortcuts).size !== shortcuts.length) {
    return "划词、OCR 和输入翻译热键不能相同";
  }
  if (!config.general.hotkey_translate.trim()) {
    return "划词热键不能为空";
  }
  if (!config.general.target_lang.trim()) {
    return "目标语言不能为空";
  }
  const actives = resolveActives(config.engine);
  if (actives.length === 0) {
    return "请至少启用一个翻译服务";
  }
  if (actives.includes("cloudflare")) {
    const endpoint = (config.engine.cloudflare_endpoint ?? "").trim();
    if (!endpoint) {
      return "Cloudflare 翻译需填写 Worker 地址";
    }
    if (!/^https?:\/\//i.test(endpoint)) {
      return "Cloudflare 翻译地址须以 http:// 或 https:// 开头";
    }
  }
  for (const active of actives) {
    if (!BUILTIN_ENGINE_IDS.has(active)) {
      return `未知翻译服务：${active}`;
    }
  }
  return null;
}

export function mergePath(paths: string[], next: string): string[] {
  const cleaned = paths.map((p) => p.trim()).filter(Boolean);
  if (cleaned.includes(next)) {
    return [next, ...cleaned.filter((p) => p !== next)];
  }
  return [next, ...cleaned];
}

export function movePathToFront(paths: string[], target: string): string[] {
  return [target, ...paths.filter((p) => p !== target)];
}

export function describeDictPath(path: string): { badge: string; name: string } {
  const normalized = path.replace(/\\/g, "/");
  const name =
    normalized.split("/").filter(Boolean).pop() ?? path;
  const isAbsolute =
    /^[a-zA-Z]:[\\/]/.test(path) ||
    path.startsWith("\\\\") ||
    path.startsWith("/");
  const looksFolder = !/\.mdx$/i.test(name);
  if (!isAbsolute) {
    return { badge: "相对", name };
  }
  return { badge: looksFolder ? "文件夹" : "文件", name };
}
