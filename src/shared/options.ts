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
    label: "Microsoft Translator",
    hint: "需填写 Azure Translator Key；区域资源再填 Region",
  },
  {
    value: "microsoft_web",
    label: "Microsoft / Bing（免费/网页）",
    hint: "非官方 Bing 网页接口，无需 Key；可能限流或失效",
  },
  {
    value: "google",
    label: "Google Cloud Translation",
    hint: "需填写 Google Cloud Translation API v2 Key",
  },
  {
    value: "google_web",
    label: "Google（免费/网页）",
    hint: "非官方 gtx 接口，无需 Key；可能限流或失效",
  },
] as const;

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
