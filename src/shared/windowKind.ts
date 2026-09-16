export type WindowKind = "popup" | "settings";

export function resolveWindowKind(): WindowKind {
  const value = new URLSearchParams(window.location.search).get("window");
  return value === "popup" ? "popup" : "settings";
}
