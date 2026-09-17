export const TARGET_LANGS = [
  { value: "zh-CN", label: "简体中文" },
  { value: "zh-TW", label: "繁體中文" },
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
    value: "self_hosted",
    label: "自建翻译",
    subtitle: "Cloudflare / 自定义 API",
    hint: "兼容 translate-api（Workers + m2m100）：填写你的接口地址与密钥。源语言为 auto 时按 en 发送。",
    configurable: true,
  },
] as const;

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
