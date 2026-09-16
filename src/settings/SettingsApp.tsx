import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig, AppInfo, AppPaths, HotkeyStatus } from "../shared/types";
import { emptyConfig } from "../shared/types";
import "./settings.css";

export function SettingsApp() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [paths, setPaths] = useState<AppPaths | null>(null);
  const [config, setConfig] = useState<AppConfig>(emptyConfig);
  const [hotkeyStatus, setHotkeyStatus] = useState<HotkeyStatus | null>(null);
  const [status, setStatus] = useState<string>("");
  const [saving, setSaving] = useState(false);

  async function refreshHotkeyStatus() {
    try {
      const next = await invoke<HotkeyStatus>("get_hotkey_status");
      setHotkeyStatus(next);
    } catch (error) {
      console.error(error);
    }
  }

  useEffect(() => {
    Promise.all([
      invoke<AppInfo>("get_app_info"),
      invoke<AppPaths>("get_app_paths"),
      invoke<AppConfig>("get_config"),
      invoke<HotkeyStatus>("get_hotkey_status"),
    ])
      .then(([appInfo, appPaths, appConfig, statusInfo]) => {
        setInfo(appInfo);
        setPaths(appPaths);
        setConfig(appConfig);
        setHotkeyStatus(statusInfo);
      })
      .catch((error: unknown) => {
        console.error(error);
        setStatus(String(error));
      });
  }, []);

  async function openPopupPreview() {
    try {
      await invoke("show_popup_window");
    } catch (error) {
      console.error(error);
      setStatus(String(error));
    }
  }

  async function handleSave() {
    setSaving(true);
    setStatus("");
    try {
      const payload: AppConfig = {
        ...config,
        engine: {
          ...config.engine,
          microsoft_api_key: normalizeOptionalKey(config.engine.microsoft_api_key),
        },
      };
      const saved = await invoke<AppConfig>("save_config", { config: payload });
      setConfig(saved);
      await refreshHotkeyStatus();
      setStatus("已保存，热键已按配置重新注册");
    } catch (error) {
      console.error(error);
      setStatus(String(error));
    } finally {
      setSaving(false);
    }
  }

  return (
    <main className="settings-shell">
      <h1>Look Translate 设置</h1>
      <p className="settings-lead">
        配置保存在可执行文件旁的便携目录 <code>data/config.toml</code>
        。默认划词热键 <code>Ctrl+Shift+D</code>
        ，托盘可开关「启用热键」。
      </p>

      <section className="settings-card">
        <h2>路径</h2>
        {paths ? (
          <dl className="settings-paths">
            <div>
              <dt>data</dt>
              <dd>{paths.dataDir}</dd>
            </div>
            <div>
              <dt>config</dt>
              <dd>{paths.configPath}</dd>
            </div>
          </dl>
        ) : (
          <p className="settings-muted">读取路径中…</p>
        )}
      </section>

      <section className="settings-card">
        <h2>热键</h2>
        <label className="settings-field">
          <span>划词热键（如 Ctrl+Shift+D）</span>
          <input
            value={config.general.hotkey_translate}
            onChange={(event) =>
              setConfig({
                ...config,
                general: {
                  ...config.general,
                  hotkey_translate: event.target.value,
                },
              })
            }
          />
        </label>
        <label className="settings-check">
          <input
            type="checkbox"
            checked={config.general.hotkey_enabled}
            onChange={(event) =>
              setConfig({
                ...config,
                general: {
                  ...config.general,
                  hotkey_enabled: event.target.checked,
                },
              })
            }
          />
          <span>启用全局热键</span>
        </label>
        <p className="settings-muted">
          {hotkeyStatus
            ? hotkeyStatus.enabled
              ? hotkeyStatus.registered
                ? `当前已注册：${hotkeyStatus.translate}`
                : `已启用但未注册成功：${hotkeyStatus.translate}`
              : "热键已关闭"
            : "读取热键状态中…"}
        </p>
      </section>

      <section className="settings-card">
        <h2>通用</h2>
        <label className="settings-field">
          <span>目标语言</span>
          <input
            value={config.general.target_lang}
            onChange={(event) =>
              setConfig({
                ...config,
                general: { ...config.general, target_lang: event.target.value },
              })
            }
          />
        </label>
        <label className="settings-field">
          <span>源语言</span>
          <input
            value={config.general.source_lang}
            onChange={(event) =>
              setConfig({
                ...config,
                general: { ...config.general, source_lang: event.target.value },
              })
            }
          />
        </label>
        <label className="settings-check">
          <input
            type="checkbox"
            checked={config.general.follow_system_proxy}
            onChange={(event) =>
              setConfig({
                ...config,
                general: {
                  ...config.general,
                  follow_system_proxy: event.target.checked,
                },
              })
            }
          />
          <span>跟随系统代理</span>
        </label>
      </section>

      <section className="settings-card">
        <h2>引擎</h2>
        <label className="settings-field">
          <span>当前引擎</span>
          <input
            value={config.engine.active}
            onChange={(event) =>
              setConfig({
                ...config,
                engine: { ...config.engine, active: event.target.value },
              })
            }
          />
        </label>
        <label className="settings-field">
          <span>Microsoft API Key</span>
          <input
            type="password"
            autoComplete="off"
            value={config.engine.microsoft_api_key ?? ""}
            onChange={(event) =>
              setConfig({
                ...config,
                engine: {
                  ...config.engine,
                  microsoft_api_key: event.target.value,
                },
              })
            }
          />
        </label>
      </section>

      <section className="settings-card settings-actions">
        <button type="button" disabled={saving} onClick={handleSave}>
          {saving ? "保存中…" : "保存配置"}
        </button>
        <button type="button" onClick={openPopupPreview}>
          显示浮层窗口
        </button>
        {status ? <p className="settings-status">{status}</p> : null}
      </section>

      {info ? (
        <p className="settings-meta">
          {info.name} · v{info.version} · {info.phase}
        </p>
      ) : null}
    </main>
  );
}

function normalizeOptionalKey(value: string | null | undefined): string | null {
  const trimmed = (value ?? "").trim();
  return trimmed.length > 0 ? trimmed : null;
}
