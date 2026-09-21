import type { EngineId } from "./options";
import { BUILTIN_ENGINE_IDS } from "./options";
import bingIcon from "../assets/brands/bing.png";
import cloudflareIcon from "../assets/brands/cloudflare.png";
import googleIcon from "../assets/brands/google.png";
import microsoftIcon from "../assets/brands/microsoft.png";

type IconProps = {
  className?: string;
  title?: string;
};

function isBuiltinBrand(engine: string): boolean {
  return BUILTIN_ENGINE_IDS.has(engine);
}

function resolveBrandSrc(engine: string): string | null {
  switch (engine) {
    case "microsoft":
      return microsoftIcon;
    case "microsoft_web":
      return bingIcon;
    case "google":
    case "google_web":
      return googleIcon;
    case "cloudflare":
      return cloudflareIcon;
    default:
      return null;
  }
}

function resolveBrandLabel(engine: string): string {
  switch (engine) {
    case "microsoft":
      return "Microsoft";
    case "microsoft_web":
      return "Bing";
    case "google":
      return "Google";
    case "google_web":
      return "Google 网页";
    case "cloudflare":
      return "Cloudflare";
    case "baidu":
      return "百度翻译";
    case "youdao":
      return "有道翻译";
    case "system":
      return "系统 OCR";
    case "tesseract":
      return "Tesseract.js";
    default:
      return engine;
  }
}

function CustomEngineGlyph() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <rect x="3" y="3" width="18" height="18" rx="4" fill="#475569" />
      <path
        d="M8 12h8M12 8v8"
        stroke="#F8FAFC"
        strokeWidth="1.75"
        strokeLinecap="round"
      />
    </svg>
  );
}

/** Brand marks for engines; web variants share a unified 「网」badge. */
export function EngineIcon({
  engine,
  className,
  size = "md",
}: {
  engine: EngineId | string;
  className?: string;
  size?: "sm" | "md";
}) {
  const web = engine === "microsoft_web" || engine === "google_web";
  const src = resolveBrandSrc(engine);
  const label = resolveBrandLabel(engine);
  const custom = !isBuiltinBrand(engine);

  return (
    <span
      className={
        size === "sm"
          ? `engine-icon engine-icon-sm${web ? " has-web" : ""}${className ? ` ${className}` : ""}`
          : `engine-icon${web ? " has-web" : ""}${className ? ` ${className}` : ""}`
      }
      title={label}
      aria-label={label}
      role="img"
    >
      {custom || !src ? (
        <CustomEngineGlyph />
      ) : (
        <img src={src} alt="" draggable={false} />
      )}
      {web ? <span className="engine-icon-web" aria-hidden="true">网</span> : null}
    </span>
  );
}

/** Simple marks for OCR backends (no external brand assets). */
export function OcrEngineIcon({
  engine,
  className,
}: {
  engine: string;
  className?: string;
}) {
  const label = resolveBrandLabel(engine);
  return (
    <span
      className={`engine-icon${className ? ` ${className}` : ""}`}
      title={label}
      aria-label={label}
      role="img"
    >
      {engine === "tesseract" ? (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <rect x="3" y="3" width="18" height="18" rx="4" fill="#0F766E" />
          <path
            d="M7 8h10M12 8v9M9.5 17h5"
            stroke="#ECFDF5"
            strokeWidth="1.75"
            strokeLinecap="round"
          />
        </svg>
      ) : (
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <rect x="3" y="3" width="18" height="18" rx="4" fill="#1D4ED8" />
          <path
            d="M7 9h10M7 12h7M7 15h9"
            stroke="#EFF6FF"
            strokeWidth="1.75"
            strokeLinecap="round"
          />
        </svg>
      )}
    </span>
  );
}

export function IconEdit({ className }: IconProps) {
  return (
    <svg className={className} viewBox="0 0 24 24" fill="none" aria-hidden="true">
      <path
        d="M5 19h3.2L18.5 8.7a1.8 1.8 0 0 0 0-2.5l-.7-.7a1.8 1.8 0 0 0-2.5 0L5 15.8V19Z"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinejoin="round"
      />
      <path
        d="M13.5 7.5 16.5 10.5"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinecap="round"
      />
    </svg>
  );
}
