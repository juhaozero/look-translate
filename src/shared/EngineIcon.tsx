import type { EngineId } from "./options";
import bingIcon from "../assets/brands/bing.png";
import googleIcon from "../assets/brands/google.png";
import microsoftIcon from "../assets/brands/microsoft.svg";

type IconProps = {
  className?: string;
  title?: string;
};

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
      <img src={src} alt="" draggable={false} />
      {web ? <span className="engine-icon-web" aria-hidden="true">网</span> : null}
    </span>
  );
}

function resolveBrandSrc(engine: string): string {
  switch (engine) {
    case "microsoft":
      return microsoftIcon;
    case "microsoft_web":
      return bingIcon;
    case "google":
    case "google_web":
      return googleIcon;
    default:
      return microsoftIcon;
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
    default:
      return engine;
  }
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
