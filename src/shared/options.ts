export const TARGET_LANGS = [
  { value: "zh-CN", label: "简体中文" },
  { value: "en", label: "English" },
  { value: "ja", label: "日本語" },
  { value: "ko", label: "한국어" },
  { value: "fr", label: "Français" },
  { value: "de", label: "Deutsch" },
  { value: "es", label: "Español" },
] as const;

export const SOURCE_LANGS = [
  { value: "auto", label: "自动检测" },
  ...TARGET_LANGS,
] as const;

export const ENGINES = [
  {
    value: "deepl",
    label: "DeepL 翻译",
    subtitle: "官方 API，支持 Free / Pro",
    hint: "填写 DeepL API Key；根据密钥自动选择 Free 或 Pro 接口",
    configurable: true,
  },
  {
    value: "microsoft",
    label: "Microsoft 翻译",
    subtitle: "需 Azure Key",
    hint: "需填写 Azure Translator Key；区域资源再填 Region",
    configurable: true,
  },
  {
    value: "microsoft_web",
    label: "必应翻译",
    subtitle: "免费网页接口",
    hint: "非官方 Bing 网页接口，无需 Key；可能限流或失效",
    configurable: false,
  },
  {
    value: "google",
    label: "Google 翻译",
    subtitle: "Cloud API，需 Key",
    hint: "需填写 Google Cloud Translation API v2 Key",
    configurable: true,
  },
  {
    value: "google_web",
    label: "Google 翻译（网页）",
    subtitle: "免费接口，可能限流",
    hint: "非官方 gtx 接口，无需 Key；可能限流或失效",
    configurable: false,
  },
  {
    value: "cloudflare",
    label: "Cloudflare 翻译",
    subtitle: "Workers / translate-api",
        hint: "兼容本仓库 work.js Worker：POST JSON + Authorization Bearer；密钥用 wrangler secret put SECRET_PASS，勿写进源码。源语言为 auto 时按正文脚本猜测（中/日/韩/英）；简繁均映射为 zh。",
    configurable: true,
  },
  {
    value: "baidu",
    label: "百度翻译",
    subtitle: "开放平台，需 App ID",
    hint: "需填写百度翻译开放平台 App ID 与密钥（通用翻译 API）",
    configurable: true,
  },
  {
    value: "youdao",
    label: "有道翻译",
    subtitle: "智云，需应用密钥",
    hint: "需填写有道智云应用 ID（appKey）与应用密钥（文本翻译 API）",
    configurable: true,
  },
] as const;

/** Prefer `list_engines` from Rust; this set is a local fallback / type helper. */
export const BUILTIN_ENGINE_IDS: ReadonlySet<string> = new Set(
  ENGINES.map((e) => e.value),
);

export type EngineListItem = {
  value: string;
  label: string;
  subtitle: string;
  hint: string;
  configurable: boolean;
  /** Supported service */
  kind: "builtin";
};

/** Map Rust `list_engines` catalog into settings list items. */
export function catalogToEngineList(
  catalog: ReadonlyArray<{
    id: string;
    label: string;
    subtitle: string;
    hint: string;
    configurable: boolean;
    kind: string;
  }>,
): EngineListItem[] {
  return catalog.map((item) => ({
    value: item.id,
    label: item.label,
    subtitle: item.subtitle,
    hint: item.hint,
    configurable: item.configurable,
    kind: "builtin",
  }));
}

/** Local fallback when the service catalog is unavailable. */
export function buildEngineList(): EngineListItem[] {
  return ENGINES.map((engine) => ({ ...engine, kind: "builtin" }));
}

export function resolveEngineLabel(engineId: string): string {
  return ENGINES.find((item) => item.value === engineId)?.label ?? engineId;
}

/** Engines configured to run in parallel (never empty). */
export function resolveActives(engine: {
  active?: string;
  actives?: string[] | null;
}): string[] {
  const fromList = (engine.actives ?? [])
    .map((id) => id.trim())
    .filter(Boolean);
  if (fromList.length > 0) {
    const seen = new Set<string>();
    const out: string[] = [];
    for (const id of fromList) {
      const key = id.toLowerCase();
      if (seen.has(key)) {
        continue;
      }
      seen.add(key);
      out.push(id);
    }
    return out;
  }
  const active = (engine.active ?? "").trim() || "microsoft";
  return [active];
}

export const OCR_ENGINES = [
  {
    value: "system",
    label: "系统 OCR",
    subtitle: "Windows.Media.Ocr",
    hint: "使用系统自带 OCR，需已安装对应语言包；启动快、体积小",
  },
  {
    value: "tesseract",
    label: "Tesseract.js",
    subtitle: "WASM 本地识别",
    hint: "浏览器端 Tesseract（eng+chi_sim）；首次使用会下载语言模型",
  },
] as const;

export type EngineId = (typeof ENGINES)[number]["value"];
export type OcrEngineId = (typeof OCR_ENGINES)[number]["value"];
export type LangOption = (typeof TARGET_LANGS)[number];

/** Map legacy Microsoft-style tags used in older configs. */
export function normalizeLangCode(code: string): string {
  const trimmed = code.trim();
  if (!trimmed) {
    return trimmed;
  }
  if (trimmed.toLowerCase() === "auto") {
    return "auto";
  }
  switch (trimmed) {
    case "zh-Hans":
    case "zh-CN":
    case "zh":
      return "zh-CN";
    case "zh-Hant":
    case "zh-TW":
      return "zh-TW";
    default:
      return trimmed;
  }
}
