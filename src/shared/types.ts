import type { EngineProfileConfig } from "./options";

export type AppInfo = {
  name: string;
  version: string;
  description: string;
};

export type AppPaths = {
  dataDir: string;
  configPath: string;
  logDir: string;
  dictsDir: string;
  recommendedDictPath: string;
  recommendedDictRelative: string;
  recommendedDictPresent: boolean;
};

export type HotkeyStatus = {
  enabled: boolean;
  translate: string;
  ocr?: string;
  translateRegistered?: boolean;
  ocrRegistered?: boolean;
  /** @deprecated alias of translateRegistered */
  registered: boolean;
};

export type CapturePayload = {
  text: string;
  empty: boolean;
  error?: string | null;
  /** `clipboard` | `ocr` */
  source?: string;
  capturedAtMs: number;
};

export type EngineTranslationResult = {
  engine: string;
  status: "loading" | "ok" | "error" | string;
  text?: string | null;
  error?: string | null;
  cached?: boolean;
  detectedSourceLang?: string | null;
};

/**
 * Translation update payload.
 * `results` is the source of truth (ordered like `engine.actives`).
 * Top-level `status` / `translatedText` / `engine` / `cached` / `error` are
 * derived summaries for convenience (first ok / overall status).
 */
export type TranslationPayload = {
  status: "loading" | "ok" | "error" | string;
  sourceText: string;
  /** Derived from first ok entry in `results`. */
  translatedText?: string | null;
  /** Derived from first ok entry in `results`. */
  engine?: string | null;
  sourceLang: string;
  targetLang: string;
  detectedSourceLang?: string | null;
  error?: string | null;
  cached?: boolean;
  dictionaryText?: string | null;
  dictionarySource?: string | null;
  /** Per-engine progressive results — always present for new translates. */
  results: EngineTranslationResult[];
};

export type EngineCatalogItem = {
  id: string;
  label: string;
  subtitle: string;
  hint: string;
  configurable: boolean;
  kind: "builtin" | "profile" | string;
};

export type InstallRecommendedDictResult = {
  status: "already_present" | "installed" | string;
  relativePath: string;
  absolutePath: string;
  config: AppConfig;
};

export type EnsureProfileResult = {
  created: boolean;
  message: string;
  config: AppConfig;
};

export type AppConfig = {
  general: {
    target_lang: string;
    source_lang: string;
    hotkey_translate: string;
    hotkey_ocr: string;
    hotkey_enabled: boolean;
    follow_system_proxy: boolean;
    launch_at_startup: boolean;
  };
  engine: {
    active: string;
    /** Engines that run in parallel (order preserved). */
    actives?: string[];
    microsoft_api_key?: string | null;
    microsoft_region?: string | null;
    google_api_key?: string | null;
    cloudflare_endpoint?: string | null;
    cloudflare_secret?: string | null;
    baidu_app_id?: string | null;
    baidu_secret?: string | null;
    youdao_app_key?: string | null;
    youdao_app_secret?: string | null;
  };
  /** Config-driven profiles (`[engines.<id>]`). */
  engines?: Record<string, EngineProfileConfig>;
  ocr: {
    /** `system` | `tesseract` */
    engine: string;
  };
  dictionary: {
    enabled: boolean;
    paths: string[];
  };
};

export function emptyConfig(): AppConfig {
  return {
    general: {
      target_lang: "zh-CN",
      source_lang: "auto",
      hotkey_translate: "Ctrl+Shift+D",
      hotkey_ocr: "Ctrl+Shift+S",
      hotkey_enabled: true,
      follow_system_proxy: true,
      launch_at_startup: false,
    },
    engine: {
      active: "microsoft",
      actives: ["microsoft"],
      microsoft_api_key: null,
      microsoft_region: null,
      google_api_key: null,
      cloudflare_endpoint: null,
      cloudflare_secret: null,
      baidu_app_id: null,
      baidu_secret: null,
      youdao_app_key: null,
      youdao_app_secret: null,
    },
    engines: {},
    ocr: {
      engine: "system",
    },
    dictionary: {
      enabled: true,
      paths: [],
    },
  };
}
