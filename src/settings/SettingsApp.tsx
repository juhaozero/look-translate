import {
  useEffect,
  useId,
  useMemo,
  useState,
  type ReactNode,
  type ComponentType,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { AppConfig, AppInfo, AppPaths, HotkeyStatus } from "../shared/types";
import { emptyConfig } from "../shared/types";
import { ENGINES, SOURCE_LANGS, TARGET_LANGS, normalizeLangCode } from "../shared/options";
import { HotkeyRecorder } from "./HotkeyRecorder";
import {
  IconAbout,
  IconApp,
  IconDict,
  IconGeneral,
  IconHotkey,
  IconService,
  IconTranslate,
} from "./icons";
import "./settings.css";

type NavId =
  | "general"
  | "translate"
  | "hotkey"
  | "service"
  | "dictionary"
  | "about";

const NAV_ITEMS: ReadonlyArray<{
  id: NavId;
  label: string;
  Icon: ComponentType<{ className?: string }>;
}> = [
  { id: "general", label: "常规设置", Icon: IconGeneral },
  { id: "translate", label: "翻译设置", Icon: IconTranslate },
  { id: "hotkey", label: "热键设置", Icon: IconHotkey },
  { id: "service", label: "服务设置", Icon: IconService },
  { id: "dictionary", label: "词典设置", Icon: IconDict },
  { id: "about", label: "关于应用", Icon: IconAbout },
];

const PAGE_TITLE: Record<NavId, string> = {
  general: "常规设置",
  translate: "翻译设置",
  hotkey: "热键设置",
  service: "服务设置",
  dictionary: "词典设置",
  about: "关于应用",
};

export function SettingsApp() {
  const [nav, setNav] = useState<NavId>("general");
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [paths, setPaths] = useState<AppPaths | null>(null);
  const [config, setConfig] = useState<AppConfig>(emptyConfig);
  const [savedSnapshot, setSavedSnapshot] = useState("");
  const [hotkeyStatus, setHotkeyStatus] = useState<HotkeyStatus | null>(null);
  const [status, setStatus] = useState<{ tone: "ok" | "err"; text: string } | null>(
    null,
  );
  const [saving, setSaving] = useState(false);
  const [loading, setLoading] = useState(true);

  const dirty = useMemo(
    () => serializeConfig(config) !== savedSnapshot && savedSnapshot.length > 0,
    [config, savedSnapshot],
  );

  async function loadAll() {
    setLoading(true);
    setStatus(null);
    try {
      const [appInfo, appPaths, appConfig, statusInfo] = await Promise.all([
        invoke<AppInfo>("get_app_info"),
        invoke<AppPaths>("get_app_paths"),
        invoke<AppConfig>("get_config"),
        invoke<HotkeyStatus>("get_hotkey_status"),
      ]);
      setInfo(appInfo);
      setPaths(appPaths);
      setConfig(appConfig);
      setSavedSnapshot(serializeConfig(appConfig));
      setHotkeyStatus(statusInfo);
    } catch (error: unknown) {
      console.error(error);
      setStatus({ tone: "err", text: String(error) });
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void loadAll();
  }, []);

  async function openPopupPreview() {
    try {
      await invoke("show_popup_window");
    } catch (error) {
      console.error(error);
      setStatus({ tone: "err", text: String(error) });
    }
  }

  async function handleSave() {
    const validation = validateConfig(config);
    if (validation) {
      setStatus({ tone: "err", text: validation });
      return;
    }

    setSaving(true);
    setStatus(null);
    try {
      const payload = normalizeConfig(config);
      await invoke<AppConfig>("save_config", { config: payload });
      await loadAll();
    } catch (error) {
      console.error(error);
      setStatus({ tone: "err", text: String(error) });
    } finally {
      setSaving(false);
    }
  }

  async function pickDictionaryFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "MDict", extensions: ["mdx"] }],
      });
      if (typeof selected === "string" && selected) {
        setConfig({
          ...config,
          dictionary: {
            ...config.dictionary,
            paths: mergePath(config.dictionary.paths, selected),
          },
        });
      }
    } catch (error) {
      console.error(error);
      setStatus({ tone: "err", text: String(error) });
    }
  }

  async function pickDictionaryFolder() {
    try {
      const selected = await open({
        multiple: false,
        directory: true,
      });
      if (typeof selected === "string" && selected) {
        setConfig({
          ...config,
          dictionary: {
            ...config.dictionary,
            paths: mergePath(config.dictionary.paths, selected),
          },
        });
      }
    } catch (error) {
      console.error(error);
      setStatus({ tone: "err", text: String(error) });
    }
  }

  function updateGeneral<K extends keyof AppConfig["general"]>(
    key: K,
    value: AppConfig["general"][K],
  ) {
    setConfig({
      ...config,
      general: { ...config.general, [key]: value },
    });
  }

  const engineHint =
    ENGINES.find((item) => item.value === config.engine.active)?.hint ??
    "请选择已支持的翻译引擎";

  return (
    <div className="settings-layout">
      <aside className="settings-sidebar" aria-label="设置导航">
        <div className="settings-brand">
          <IconApp className="settings-brand-icon" />
        </div>
        <nav className="settings-nav">
          {NAV_ITEMS.map(({ id, label, Icon }) => (
            <button
              key={id}
              type="button"
              className={
                nav === id ? "settings-nav-item is-active" : "settings-nav-item"
              }
              aria-current={nav === id ? "page" : undefined}
              onClick={() => setNav(id)}
            >
              <Icon className="settings-nav-icon" />
              <span>{label}</span>
            </button>
          ))}
        </nav>
      </aside>

      <div className="settings-main">
        <header className="settings-main-header">
          <div>
            <h1>{PAGE_TITLE[nav]}</h1>
            {loading ? (
              <p className="settings-subtle">加载中…</p>
            ) : dirty ? (
              <p className="settings-subtle settings-subtle-warn">有未保存的更改</p>
            ) : null}
          </div>
          <div className="settings-main-actions">
            <button
              type="button"
              className="settings-btn settings-btn-primary"
              disabled={saving || loading}
              onClick={() => void handleSave()}
            >
              {saving ? "保存中…" : "保存"}
            </button>
          </div>
        </header>

        <div className="settings-content">
          {nav === "general" ? (
            <>
              <section className="settings-panel">
                <ToggleRow
                  label="启用全局热键"
                  checked={config.general.hotkey_enabled}
                  onChange={(checked) => updateGeneral("hotkey_enabled", checked)}
                />
                <ToggleRow
                  label="跟随系统代理"
                  checked={config.general.follow_system_proxy}
                  onChange={(checked) =>
                    updateGeneral("follow_system_proxy", checked)
                  }
                />
              </section>
              <section className="settings-panel">
                <RowAction
                  label="预览划词浮层"
                  actionLabel="打开浮层"
                  onClick={() => void openPopupPreview()}
                />
              </section>
            </>
          ) : null}

          {nav === "translate" ? (
            <section className="settings-panel">
              <SelectRow
                label="源语言"
                value={config.general.source_lang}
                options={SOURCE_LANGS}
                onChange={(value) => updateGeneral("source_lang", value)}
              />
              <SelectRow
                label="目标语言"
                value={config.general.target_lang}
                options={TARGET_LANGS}
                onChange={(value) => updateGeneral("target_lang", value)}
              />
              <p className="settings-panel-foot">
                划词后浮层里仍可临时改目标语并重译；此处为默认值。
              </p>
            </section>
          ) : null}

          {nav === "hotkey" ? (
            <>
              <section className="settings-panel">
                <HotkeyRecorder
                  label="划词翻译"
                  value={config.general.hotkey_translate}
                  onChange={(next) => updateGeneral("hotkey_translate", next)}
                />
                <HotkeyRecorder
                  label="文字识别"
                  value={config.general.hotkey_ocr}
                  onChange={(next) => updateGeneral("hotkey_ocr", next)}
                />
                <p className="settings-panel-foot">
                  划词热键走剪贴板 Ctrl+C；OCR 热键截取指针附近固定区域识别，二者互不兜底。
                </p>
              </section>
              <section className="settings-panel">
                <ToggleRow
                  label="启用全局热键"
                  checked={config.general.hotkey_enabled}
                  onChange={(checked) => updateGeneral("hotkey_enabled", checked)}
                />
                <p
                  className={
                    hotkeyStatus?.enabled &&
                    !(hotkeyStatus.translateRegistered ?? hotkeyStatus.registered)
                      ? "settings-panel-foot is-warn"
                      : "settings-panel-foot"
                  }
                >
                  {hotkeyStatus
                    ? hotkeyStatus.enabled
                      ? [
                          (hotkeyStatus.translateRegistered ??
                          hotkeyStatus.registered)
                            ? `划词已注册：${hotkeyStatus.translate}`
                            : `划词未注册：${hotkeyStatus.translate}`,
                          hotkeyStatus.ocrRegistered
                            ? `OCR 已注册：${hotkeyStatus.ocr ?? config.general.hotkey_ocr}`
                            : `OCR 未注册：${hotkeyStatus.ocr ?? config.general.hotkey_ocr}`,
                        ].join("；")
                      : "热键已关闭（可在托盘菜单中开关）"
                    : "读取热键状态中…"}
                </p>
              </section>
            </>
          ) : null}

          {nav === "service" ? (
            <>
              <section className="settings-panel">
                <SelectRow
                  label="翻译引擎"
                  value={config.engine.active}
                  options={ENGINES.map((engine) => ({
                    value: engine.value,
                    label: engine.label,
                  }))}
                  onChange={(value) =>
                    setConfig({
                      ...config,
                      engine: { ...config.engine, active: value },
                    })
                  }
                />
                <p className="settings-panel-foot">{engineHint}</p>
              </section>
              {config.engine.active === "microsoft" ? (
                <section className="settings-panel">
                  <FieldRow label="API Key">
                    <input
                      type="password"
                      autoComplete="off"
                      className="settings-input"
                      placeholder="Azure Translator 订阅密钥"
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
                  </FieldRow>
                  <FieldRow label="Region">
                    <input
                      className="settings-input"
                      placeholder="如 eastasia；全球资源可留空"
                      value={config.engine.microsoft_region ?? ""}
                      onChange={(event) =>
                        setConfig({
                          ...config,
                          engine: {
                            ...config.engine,
                            microsoft_region: event.target.value,
                          },
                        })
                      }
                    />
                  </FieldRow>
                </section>
              ) : null}
              {config.engine.active === "google" ? (
                <section className="settings-panel">
                  <FieldRow label="API Key">
                    <input
                      type="password"
                      autoComplete="off"
                      className="settings-input"
                      placeholder="Google Cloud Translation API Key"
                      value={config.engine.google_api_key ?? ""}
                      onChange={(event) =>
                        setConfig({
                          ...config,
                          engine: {
                            ...config.engine,
                            google_api_key: event.target.value,
                          },
                        })
                      }
                    />
                  </FieldRow>
                </section>
              ) : null}
            </>
          ) : null}

          {nav === "dictionary" ? (
            <>
              <section className="settings-panel">
                <ToggleRow
                  label="启用离线词库"
                  checked={config.dictionary.enabled}
                  onChange={(checked) =>
                    setConfig({
                      ...config,
                      dictionary: {
                        ...config.dictionary,
                        enabled: checked,
                      },
                    })
                  }
                />
                <p className="settings-panel-foot">
                  仅短词查询：≤30 字符且 ≤3 个词，且无换行。只查列表第一本。
                </p>
              </section>
              <section className="settings-panel settings-panel-stack">
                <label className="settings-stack-label" htmlFor="dict-paths">
                  词典路径
                </label>
                <textarea
                  id="dict-paths"
                  className="settings-textarea"
                  rows={5}
                  placeholder={"D:\\dicts\\oxford.mdx\n或词典所在文件夹"}
                  value={config.dictionary.paths.join("\n")}
                  onChange={(event) =>
                    setConfig({
                      ...config,
                      dictionary: {
                        ...config.dictionary,
                        paths: event.target.value
                          .split(/\r?\n/)
                          .map((line) => line.trim())
                          .filter(Boolean),
                      },
                    })
                  }
                />
                <div className="settings-inline-actions">
                  <button
                    type="button"
                    className="settings-btn"
                    onClick={() => void pickDictionaryFile()}
                  >
                    选择 .mdx
                  </button>
                  <button
                    type="button"
                    className="settings-btn"
                    onClick={() => void pickDictionaryFolder()}
                  >
                    选择文件夹
                  </button>
                  {config.dictionary.paths.length > 0 ? (
                    <button
                      type="button"
                      className="settings-btn settings-btn-ghost"
                      onClick={() =>
                        setConfig({
                          ...config,
                          dictionary: { ...config.dictionary, paths: [] },
                        })
                      }
                    >
                      清空
                    </button>
                  ) : null}
                </div>
                <p className="settings-panel-foot">
                  可为 `.mdx` 文件，或包含 `.mdx` 的文件夹。释义纯文本展示，不加载
                  `.mdd`。
                </p>
              </section>
            </>
          ) : null}

          {nav === "about" ? (
            <section className="settings-about">
              <IconApp className="settings-about-icon" />
              <h2>{info?.name ?? "Look Translate"}</h2>
              <p className="settings-about-ver">
                v{info?.version ?? "…"}
                {info?.phase ? ` · ${info.phase}` : ""}
              </p>
              <div className="settings-about-card">
                <div className="settings-about-row">
                  <span>data</span>
                  <code>{paths?.dataDir ?? "…"}</code>
                </div>
                <div className="settings-about-row">
                  <span>config</span>
                  <code>{paths?.configPath ?? "…"}</code>
                </div>
              </div>
              <p className="settings-about-note">
            
              </p>
            </section>
          ) : null}

          {status ? (
            <p
              className={
                status.tone === "ok"
                  ? "settings-toast is-ok"
                  : "settings-toast is-err"
              }
              role="status"
            >
              {status.text}
            </p>
          ) : null}
        </div>
      </div>
    </div>
  );
}

function ToggleRow({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  const id = useId();
  return (
    <div className="settings-row">
      <label className="settings-row-label" htmlFor={id}>
        {label}
      </label>
      <button
        id={id}
        type="button"
        role="switch"
        aria-checked={checked}
        className={checked ? "settings-switch is-on" : "settings-switch"}
        onClick={() => onChange(!checked)}
      >
        <span className="settings-switch-thumb" />
      </button>
    </div>
  );
}

function SelectRow({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: ReadonlyArray<{ value: string; label: string }>;
  onChange: (value: string) => void;
}) {
  const id = useId();
  const known = options.some((item) => item.value === value);
  return (
    <div className="settings-row">
      <label className="settings-row-label" htmlFor={id}>
        {label}
      </label>
      <select
        id={id}
        className="settings-select"
        value={value}
        onChange={(event) => onChange(event.target.value)}
      >
        {options.map((item) => (
          <option key={item.value} value={item.value}>
            {item.label}
          </option>
        ))}
        {!known ? <option value={value}>{value}</option> : null}
      </select>
    </div>
  );
}

function FieldRow({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <div className="settings-row settings-row-field">
      <span className="settings-row-label">{label}</span>
      <div className="settings-row-control">{children}</div>
    </div>
  );
}

function RowAction({
  label,
  actionLabel,
  onClick,
}: {
  label: string;
  actionLabel: string;
  onClick: () => void;
}) {
  return (
    <div className="settings-row">
      <span className="settings-row-label">{label}</span>
      <button type="button" className="settings-btn" onClick={onClick}>
        {actionLabel}
      </button>
    </div>
  );
}

function normalizeOptionalKey(value: string | null | undefined): string | null {
  const trimmed = (value ?? "").trim();
  return trimmed.length > 0 ? trimmed : null;
}

function normalizeConfig(config: AppConfig): AppConfig {
  return {
    ...config,
    general: {
      ...config.general,
      target_lang: normalizeLangCode(config.general.target_lang) || "zh-CN",
      source_lang: normalizeLangCode(config.general.source_lang) || "auto",
      hotkey_translate: config.general.hotkey_translate.trim() || "Ctrl+Shift+D",
    },
    engine: {
      ...config.engine,
      active: config.engine.active.trim() || "microsoft",
      microsoft_api_key: normalizeOptionalKey(config.engine.microsoft_api_key),
      microsoft_region: normalizeOptionalKey(config.engine.microsoft_region),
      google_api_key: normalizeOptionalKey(config.engine.google_api_key),
    },
    dictionary: {
      ...config.dictionary,
      paths: config.dictionary.paths.map((p) => p.trim()).filter(Boolean),
    },
  };
}

function serializeConfig(config: AppConfig): string {
  return JSON.stringify(normalizeConfig(config));
}

function validateConfig(config: AppConfig): string | null {
  if (!config.general.hotkey_translate.trim()) {
    return "划词热键不能为空";
  }
  if (!config.general.target_lang.trim()) {
    return "目标语言不能为空";
  }
  if (
    config.general.hotkey_ocr.trim() &&
    config.general.hotkey_translate.trim().toLowerCase() ===
      config.general.hotkey_ocr.trim().toLowerCase()
  ) {
    return "划词热键与 OCR 热键不能相同";
  }
  return null;
}

function mergePath(paths: string[], next: string): string[] {
  const cleaned = paths.map((p) => p.trim()).filter(Boolean);
  if (cleaned.includes(next)) {
    return [next, ...cleaned.filter((p) => p !== next)];
  }
  return [next, ...cleaned];
}
