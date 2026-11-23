export type WindowKind = "popup" | "settings" | "onboarding" | "ocr-select";

export function resolveWindowKind(): WindowKind {
  const value = new URLSearchParams(window.location.search).get("window");
  if (value === "popup") return "popup";
  if (value === "ocr-select") return "ocr-select";
  if (value === "onboarding") return "onboarding";
  return "settings";
}
