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
  registered: boolean;
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
  };
  dictionary: {
    enabled: boolean;
    paths: string[];
  };
};

export function emptyConfig(): AppConfig {
  return {
    general: {
      target_lang: "zh-Hans",
      source_lang: "auto",
      hotkey_translate: "Ctrl+Shift+D",
      hotkey_ocr: "Ctrl+Shift+S",
      hotkey_enabled: true,
      follow_system_proxy: true,
    },
    engine: {
      active: "microsoft",
      microsoft_api_key: null,
    },
    dictionary: {
      enabled: true,
      paths: [],
    },
  };
}
