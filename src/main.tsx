import React from "react";
import ReactDOM from "react-dom/client";
import { resolveWindowKind } from "./shared/windowKind";
import { PopupApp } from "./popup/PopupApp";
import { SettingsApp } from "./settings/SettingsApp";
import "./shared/base.css";

const kind = resolveWindowKind();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {kind === "popup" ? <PopupApp /> : <SettingsApp />}
  </React.StrictMode>,
);
