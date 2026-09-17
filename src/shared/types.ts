export type AppInfo = {
  name: string;
  version: string;
  phase: string;
};

export type AppPaths = {
  dataDir: string;
  configPath: string;
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

export type TranslationPayload = {
  status: "loading" | "ok" | "error" | string;
  sourceText: string;
  translatedText?: string | null;
  engine?: string | null;
  sourceLang: string;
  targetLang: string;
  detectedSourceLang?: string | null;
  error?: string | null;
  cached?: boolean;
  dictionaryText?: string | null;
  dictionarySource?: string | null;
};

export type AppConfig = {
  general: {
    target_lang: string;
    source_lang: string;
    hotkey_translate: string;
    hotkey_ocr: string;
    hotkey_enabled: boolean;
    follow_system_proxy: boolean;
  };
  engine: {
    active: string;
    microsoft_api_key?: string | null;
    microsoft_region?: string | null;
    google_api_key?: string | null;
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
    },
    engine: {
      active: "microsoft",
      microsoft_api_key: null,
      microsoft_region: null,
      google_api_key: null,
    },
    dictionary: {
      enabled: true,
      paths: [],
    },
  };
}
