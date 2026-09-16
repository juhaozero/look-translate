import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppInfo } from "../shared/types";
import "./popup.css";

export function PopupApp() {
  const [info, setInfo] = useState<AppInfo | null>(null);

  useEffect(() => {
    invoke<AppInfo>("get_app_info")
      .then(setInfo)
      .catch((error: unknown) => {
        console.error(error);
      });
  }, []);

  return (
    <main className="popup-shell">
      <header className="popup-header">划词翻译</header>
      <p className="popup-placeholder">
        按划词热键可打开本浮层（下一步将接入取词与翻译）。
      </p>
      {info ? (
        <footer className="popup-footer">
          {info.name} · {info.version} · {info.phase}
        </footer>
      ) : null}
    </main>
  );
}
