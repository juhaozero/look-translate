import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppInfo } from "../shared/types";
import "./settings.css";

export function SettingsApp() {
  const [info, setInfo] = useState<AppInfo | null>(null);

  useEffect(() => {
    invoke<AppInfo>("get_app_info")
      .then(setInfo)
      .catch((error: unknown) => {
        console.error(error);
      });
  }, []);

  async function openPopupPreview() {
    try {
      await invoke("show_popup_window");
    } catch (error) {
      console.error(error);
    }
  }

  return (
    <main className="settings-shell">
      <h1>Look Translate 设置</h1>
      <p className="settings-lead">
        脚手架阶段：托盘菜单可打开本页。引擎、热键、词典等将在后续步骤接入。
      </p>

      <section className="settings-card">
        <h2>开发预览</h2>
        <button type="button" onClick={openPopupPreview}>
          显示浮层窗口
        </button>
      </section>

      {info ? (
        <p className="settings-meta">
          {info.name} · v{info.version} · {info.phase}
        </p>
      ) : null}
    </main>
  );
}
