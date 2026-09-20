import {
  useEffect,
  useId,
  useMemo,
  useState,
  type ComponentType,
} from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  AppConfig,
  AppInfo,
  AppPaths,
  EnsureProfileResult,
  HotkeyStatus,
  InstallRecommendedDictResult,
} from "../shared/types";
import { emptyConfig } from "../shared/types";
import { OCR_ENGINES, SOURCE_LANGS, TARGET_LANGS, BUILTIN_ENGINE_IDS, buildEngineList, normalizeLangCode } from "../shared/options";
import { EngineIcon, IconEdit, OcrEngineIcon } from "../shared/EngineIcon";
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
  const [installingDict, setInstallingDict] = useState(false);
  const [editingEngine, setEditingEngine] = useState<string | null>(null);
  const [profileBusy, setProfileBusy] = useState(false);

  const dirty = useMemo(
    () => serializeConfig(config) !== savedSnapshot && savedSnapshot.length > 0,
    [config, savedSnapshot],
  );

  const engineList = useMemo(
    () => buildEngineList(config.engines),
    [config.engines],
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
      const normalized = normalizeConfig(appConfig);
      setConfig(normalized);
      setSavedSnapshot(serializeConfig(normalized));
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

  useEffect(() => {
    if (nav !== "service") {
      setEditingEngine(null);
    }
  }, [nav]);

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

  async function installRecommendedDict() {
    if (installingDict) return;
    setInstallingDict(true);
    setStatus(null);
    try {
      const result = await invoke<InstallRecommendedDictResult>(
        "install_recommended_dict",
      );
      setConfig(result.config);
      setSavedSnapshot(serializeConfig(result.config));
      const appPaths = await invoke<AppPaths>("get_app_paths");
      setPaths(appPaths);
      const msg =
        result.status === "already_present"
          ? `推荐词典已存在，已写入配置：${result.relativePath}`
          : `推荐词典已安装：${result.relativePath}（约 70MB，来源 ECDICT）`;
      setStatus({ tone: "ok", text: msg });
    } catch (error) {
      console.error(error);
      setStatus({
        tone: "err",
        text: `安装推荐词典失败：${String(error)}`,
      });
    } finally {
      setInstallingDict(false);
    }
  }

  async function openConfigFile() {
    setStatus(null);
    try {
      await invoke("open_config_file");
      setStatus({
        tone: "ok",
        text: "已用系统默认程序打开 config.toml；改完后点「重新加载配置」。",
      });
    } catch (error) {
      console.error(error);
      setStatus({ tone: "err", text: String(error) });
    }
  }

  async function reloadConfigFromDisk() {
    if (dirty) {
      const ok = window.confirm(
        "当前有未保存的修改，重新加载将丢弃这些修改。继续？",
      );
      if (!ok) {
        return;
      }
    }
    setStatus(null);
    try {
      const next = await invoke<AppConfig>("reload_config");
      setConfig(next);
      setSavedSnapshot(serializeConfig(next));
      setStatus({ tone: "ok", text: "已从磁盘重新加载配置" });
    } catch (error) {
      console.error(error);
      setStatus({ tone: "err", text: String(error) });
    }
  }

  async function addCustomTemplate() {
    if (profileBusy) return;
    if (dirty) {
      const ok = window.confirm(
        "添加模板会写入配置文件。若有未保存修改，建议先保存。继续？",
      );
      if (!ok) {
        return;
      }
    }
    setProfileBusy(true);
    setStatus(null);
    try {
      const result = await invoke<EnsureProfileResult>(
        "ensure_custom_engine_profile",
      );
      setConfig(result.config);
      setSavedSnapshot(serializeConfig(result.config));
      setStatus({
        tone: "ok",
        text: result.message,
      });
      if (result.created) {
        setEditingEngine("custom");
      }
    } catch (error) {
      console.error(error);
      setStatus({ tone: "err", text: String(error) });
    } finally {
      setProfileBusy(false);
    }
  }

  async function openEngineProfilesDoc() {
    setStatus(null);
    try {
      await invoke("open_engine_profiles_doc");
      setStatus({ tone: "ok", text: "已打开自定义引擎填写说明" });
    } catch (error) {
      console.error(error);
      setStatus({
        tone: "err",
        text: `${String(error)}（也可直接查看仓库 docs/engine-profiles.md）`,
      });
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

  function selectEngine(engineId: string) {
    if (config.engine.active === engineId) {
      return;
    }
    setConfig({
      ...config,
      engine: { ...config.engine, active: engineId },
    });
    setStatus(null);
  }

  function selectOcrEngine(engineId: string) {
    if (config.ocr.engine === engineId) {
      return;
    }
    setConfig({
      ...config,
      ocr: { ...config.ocr, engine: engineId },
    });
    setStatus(null);
  }

  function onEngineToggle(engineId: string, turnOn: boolean) {
    if (turnOn) {
      selectEngine(engineId);
      return;
    }
    if (config.engine.active === engineId) {
      setStatus({
        tone: "err",
        text: "请先打开其他引擎，不能关闭当前唯一服务",
      });
    }
  }

  function onOcrEngineToggle(engineId: string, turnOn: boolean) {
    if (turnOn) {
      selectOcrEngine(engineId);
      return;
    }
    if (config.ocr.engine === engineId) {
      setStatus({
        tone: "err",
        text: "请先打开其他 OCR，不能关闭当前唯一识别引擎",
      });
    }
  }

  function toggleEngineEditor(engineId: string) {
    setEditingEngine((current) => (current === engineId ? null : engineId));
  }

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
                  label="开机自启"
                  checked={config.general.launch_at_startup}
                  onChange={(checked) =>
                    updateGeneral("launch_at_startup", checked)
                  }
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
                  划词热键走剪贴板；OCR 热键打开框选层识别，二者互不兜底。
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
            <section className="service-list-shell" aria-label="翻译服务">
              <h2 className="service-section-title">翻译服务</h2>
              <ul className="service-list">
                {engineList.map((engine) => {
                  const active = config.engine.active === engine.value;
                  const expanded = editingEngine === engine.value;
                  return (
                    <li
                      key={engine.value}
                      className={
                        active
                          ? "service-card is-active"
                          : "service-card"
                      }
                    >
                      <div className="service-card-row">
                        <button
                          type="button"
                          className="service-card-main"
                          onClick={() => selectEngine(engine.value)}
                        >
                          <EngineIcon
                            engine={engine.value}
                            className="service-engine-icon"
                          />
                          <span className="service-engine-text">
                            <span className="service-engine-name">
                              {engine.label}
                            </span>
                            <span className="service-engine-sub">
                              {engine.subtitle}
                            </span>
                          </span>
                        </button>
                        <div className="service-card-actions">
                          <button
                            type="button"
                            className={
                              active
                                ? "settings-switch is-on"
                                : "settings-switch"
                            }
                            role="switch"
                            aria-checked={active}
                            aria-label={`将 ${engine.label} 设为当前引擎`}
                            onClick={() =>
                              onEngineToggle(engine.value, !active)
                            }
                          >
                            <span className="settings-switch-thumb" />
                          </button>
                          {engine.configurable ? (
                            <button
                              type="button"
                              className={
                                expanded
                                  ? "service-icon-btn is-active"
                                  : "service-icon-btn"
                              }
                              aria-expanded={expanded}
                              aria-label={`配置 ${engine.label}`}
                              onClick={() => toggleEngineEditor(engine.value)}
                            >
                              <IconEdit className="service-icon-btn-svg" />
                            </button>
                          ) : (
                            <span
                              className="service-icon-btn is-spacer"
                              aria-hidden="true"
                            />
                          )}
                        </div>
                      </div>
                      {expanded && engine.kind === "profile" ? (
                        <div className="service-card-editor">
                          <p className="service-card-editor-hint">
                            {engine.hint}
                          </p>
                          <div className="service-profile-actions">
                            <button
                              type="button"
                              className="settings-btn settings-btn-ghost"
                              onClick={() => void openConfigFile()}
                            >
                              打开配置文件
                            </button>
                            <button
                              type="button"
                              className="settings-btn settings-btn-ghost"
                              onClick={() => void reloadConfigFromDisk()}
                            >
                              重新加载配置
                            </button>
                          </div>
                        </div>
                      ) : null}
                      {expanded && engine.value === "microsoft" ? (
                        <div className="service-card-editor">
                          <p className="service-card-editor-hint">
                            {engine.hint}
                          </p>
                          <label className="service-field">
                            <span>API Key</span>
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
                          </label>
                          <label className="service-field">
                            <span>Region</span>
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
                          </label>
                        </div>
                      ) : null}
                      {expanded && engine.value === "google" ? (
                        <div className="service-card-editor">
                          <p className="service-card-editor-hint">
                            {engine.hint}
                          </p>
                          <label className="service-field">
                            <span>API Key</span>
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
                          </label>
                        </div>
                      ) : null}
                      {expanded && engine.value === "cloudflare" ? (
                        <div className="service-card-editor">
                          <label className="service-field">
                            <span>Worker 地址</span>
                            <input
                              type="url"
                              autoComplete="off"
                              className="settings-input"
                              placeholder="https://xxx.workers.dev/"
                              value={config.engine.cloudflare_endpoint ?? ""}
                              onChange={(event) =>
                                setConfig({
                                  ...config,
                                  engine: {
                                    ...config.engine,
                                    cloudflare_endpoint: event.target.value,
                                  },
                                })
                              }
                            />
                          </label>
                          <label className="service-field">
                            <span>访问密钥</span>
                            <input
                              type="password"
                              autoComplete="off"
                              className="settings-input"
                              placeholder="与 Worker 中 SECRET_PASS 一致"
                              value={config.engine.cloudflare_secret ?? ""}
                              onChange={(event) =>
                                setConfig({
                                  ...config,
                                  engine: {
                                    ...config.engine,
                                    cloudflare_secret: event.target.value,
                                  },
                                })
                              }
                            />
                          </label>
                        </div>
                      ) : null}
                    </li>
                  );
                })}
              </ul>
              <p className="settings-panel-foot service-list-foot">
                同一时间仅一个翻译引擎生效。打开开关即切换当前服务；关闭当前无效。
              </p>
              <div className="service-custom-engines">
                <p className="service-custom-engines-title">自定义引擎（config.toml）</p>
                <p className="service-custom-engines-desc">
                  在{" "}
                  <code>{paths?.configPath ?? "data/config.toml"}</code>{" "}
                  编写 <code>[engines.*]</code>
                  。可插入通用 HTTP 骨架，再按文档改成 DeepL 等任意厂商。
                </p>
                <div className="service-profile-actions">
                  <button
                    type="button"
                    className="settings-btn settings-btn-ghost"
                    onClick={() => void openConfigFile()}
                  >
                    打开配置文件
                  </button>
                  <button
                    type="button"
                    className="settings-btn settings-btn-ghost"
                    onClick={() => void reloadConfigFromDisk()}
                  >
                    重新加载配置
                  </button>
                  <button
                    type="button"
                    className="settings-btn settings-btn-ghost"
                    onClick={() => void openEngineProfilesDoc()}
                  >
                    填写说明
                  </button>
                  <button
                    type="button"
                    className="settings-btn"
                    disabled={profileBusy}
                    onClick={() => void addCustomTemplate()}
                  >
                    {profileBusy ? "处理中…" : "添加通用模板"}
                  </button>
                </div>
              </div>
            </section>

            <section className="service-list-shell" aria-label="OCR 识别">
              <h2 className="service-section-title">OCR 识别</h2>
              <ul className="service-list">
                {OCR_ENGINES.map((engine) => {
                  const active = config.ocr.engine === engine.value;
                  return (
                    <li
                      key={engine.value}
                      className={
                        active ? "service-card is-active" : "service-card"
                      }
                    >
                      <div className="service-card-row">
                        <button
                          type="button"
                          className="service-card-main"
                          onClick={() => selectOcrEngine(engine.value)}
                        >
                          <OcrEngineIcon
                            engine={engine.value}
                            className="service-engine-icon"
                          />
                          <span className="service-engine-text">
                            <span className="service-engine-name">
                              {engine.label}
                            </span>
                            <span className="service-engine-sub">
                              {engine.subtitle}
                            </span>
                          </span>
                        </button>
                        <div className="service-card-actions">
                          <button
                            type="button"
                            className={
                              active
                                ? "settings-switch is-on"
                                : "settings-switch"
                            }
                            role="switch"
                            aria-checked={active}
                            aria-label={`将 ${engine.label} 设为当前 OCR`}
                            onClick={() =>
                              onOcrEngineToggle(engine.value, !active)
                            }
                          >
                            <span className="settings-switch-thumb" />
                          </button>
                          <span
                            className="service-icon-btn is-spacer"
                            aria-hidden="true"
                          />
                        </div>
                      </div>
                      {active ? (
                        <div className="service-card-editor">
                          <p className="service-card-editor-hint">
                            {engine.hint}
                          </p>
                        </div>
                      ) : null}
                    </li>
                  );
                })}
              </ul>
              <p className="settings-panel-foot service-list-foot">
                OCR 与翻译引擎独立配置。Tesseract.js 首次识别会下载 eng+chi_sim
                模型（见{" "}
                <a
                  href="https://github.com/naptha/tesseract.js/"
                  target="_blank"
                  rel="noreferrer"
                >
                  tesseract.js
                </a>
                ）。
              </p>
            </section>
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
                <label className="settings-stack-label">推荐词典</label>
                <p className="settings-panel-foot">
                  ECDICT 简明英汉增强版（mdx，无音标）。安装到{" "}
                  <code>
                    {paths?.recommendedDictRelative ?? "dicts/ecdict.mdx"}
                  </code>
                  （相对 data/）。约 70MB，需联网从 GitHub Release 下载。
                </p>
                <div className="settings-inline-actions">
                  <button
                    type="button"
                    className="settings-btn settings-btn-primary"
                    disabled={installingDict || loading}
                    onClick={() => void installRecommendedDict()}
                  >
                    {installingDict
                      ? "下载安装中…"
                      : paths?.recommendedDictPresent
                        ? "重新绑定推荐词典"
                        : "安装 ECDICT 推荐词典"}
                  </button>
                </div>
                {paths?.recommendedDictPresent ? (
                  <p className="settings-panel-foot">
                    已检测到：<code>{paths.recommendedDictPath}</code>
                  </p>
                ) : null}
              </section>
              <section className="settings-panel settings-panel-stack">
                <div className="settings-stack-head">
                  <span className="settings-stack-label" id="dict-paths-label">
                    词典路径
                  </span>
                  {config.dictionary.paths.length > 0 ? (
                    <span className="settings-stack-meta">
                      {config.dictionary.paths.length} 项 · 仅查第一本
                    </span>
                  ) : null}
                </div>

                {config.dictionary.paths.length === 0 ? (
                  <div
                    className="dict-path-empty"
                    role="status"
                    aria-labelledby="dict-paths-label"
                  >
                    <IconDict className="dict-path-empty-icon" />
                    <p>尚未添加词典</p>
                    <span>选择 .mdx 文件或文件夹，或安装上方推荐词典</span>
                  </div>
                ) : (
                  <ul
                    className="dict-path-list"
                    aria-labelledby="dict-paths-label"
                  >
                    {config.dictionary.paths.map((path, index) => {
                      const kind = describeDictPath(path);
                      return (
                        <li
                          key={path}
                          className={
                            index === 0
                              ? "dict-path-item is-primary"
                              : "dict-path-item"
                          }
                        >
                          <div className="dict-path-item-main">
                            <div className="dict-path-badges">
                              {index === 0 ? (
                                <span className="dict-path-badge is-primary">
                                  优先
                                </span>
                              ) : null}
                              <span className="dict-path-badge">{kind.badge}</span>
                            </div>
                            <div className="dict-path-text">
                              <span className="dict-path-name" title={path}>
                                {kind.name}
                              </span>
                              <code className="dict-path-full" title={path}>
                                {path}
                              </code>
                            </div>
                          </div>
                          <div className="dict-path-item-actions">
                            {index > 0 ? (
                              <button
                                type="button"
                                className="settings-btn settings-btn-ghost dict-path-action"
                                onClick={() =>
                                  setConfig({
                                    ...config,
                                    dictionary: {
                                      ...config.dictionary,
                                      paths: movePathToFront(
                                        config.dictionary.paths,
                                        path,
                                      ),
                                    },
                                  })
                                }
                              >
                                设为优先
                              </button>
                            ) : null}
                            <button
                              type="button"
                              className="settings-btn settings-btn-ghost dict-path-action"
                              aria-label={`移除 ${kind.name}`}
                              onClick={() =>
                                setConfig({
                                  ...config,
                                  dictionary: {
                                    ...config.dictionary,
                                    paths: config.dictionary.paths.filter(
                                      (p) => p !== path,
                                    ),
                                  },
                                })
                              }
                            >
                              移除
                            </button>
                          </div>
                        </li>
                      );
                    })}
                  </ul>
                )}

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
                      全部清空
                    </button>
                  ) : null}
                </div>
                <p className="settings-panel-foot">
                  相对路径相对安装目录旁的 <code>data/</code>
                  。释义纯文本，不加载 <code>.mdd</code>。
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
              </p>
              <p className="settings-about-desc">
                {info?.description ?? "Windows 划词翻译小工具"}
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
                推荐离线词库来自{" "}
                <a
                  href="https://github.com/skywind3000/ECDICT"
                  target="_blank"
                  rel="noreferrer"
                >
                  skywind3000/ECDICT
                </a>
                ，按需下载至 data/dicts/
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
  const ocrEngine =
    config.ocr?.engine?.trim().toLowerCase() === "tesseract"
      ? "tesseract"
      : "system";
  return {
    ...config,
    general: {
      ...config.general,
      target_lang: normalizeLangCode(config.general.target_lang) || "zh-CN",
      source_lang: normalizeLangCode(config.general.source_lang) || "auto",
      hotkey_translate: config.general.hotkey_translate.trim() || "Ctrl+Shift+D",
      launch_at_startup: Boolean(config.general.launch_at_startup),
    },
    engine: {
      ...config.engine,
      active: config.engine.active.trim() || "microsoft",
      microsoft_api_key: normalizeOptionalKey(config.engine.microsoft_api_key),
      microsoft_region: normalizeOptionalKey(config.engine.microsoft_region),
      google_api_key: normalizeOptionalKey(config.engine.google_api_key),
      cloudflare_endpoint: normalizeOptionalKey(
        config.engine.cloudflare_endpoint,
      ),
      cloudflare_secret: normalizeOptionalKey(config.engine.cloudflare_secret),
    },
    engines: config.engines ?? {},
    ocr: {
      engine: ocrEngine,
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
  if (config.engine.active === "cloudflare") {
    const endpoint = (config.engine.cloudflare_endpoint ?? "").trim();
    if (!endpoint) {
      return "Cloudflare 翻译需填写 Worker 地址";
    }
    if (!/^https?:\/\//i.test(endpoint)) {
      return "Cloudflare 翻译地址须以 http:// 或 https:// 开头";
    }
  }
  const active = config.engine.active.trim();
  if (active && !BUILTIN_ENGINE_IDS.has(active)) {
    const profile = config.engines?.[active];
    if (!profile) {
      return `未找到自定义引擎 [engines.${active}]，请检查 config.toml`;
    }
    if (!(profile.url ?? "").trim()) {
      return `自定义引擎 ${active} 缺少 url`;
    }
    if (!(profile.text_path ?? "").trim()) {
      return `自定义引擎 ${active} 缺少 text_path`;
    }
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

function movePathToFront(paths: string[], target: string): string[] {
  return [target, ...paths.filter((p) => p !== target)];
}

function describeDictPath(path: string): { badge: string; name: string } {
  const normalized = path.replace(/\\/g, "/");
  const name =
    normalized.split("/").filter(Boolean).pop() ?? path;
  const isAbsolute =
    /^[a-zA-Z]:[\\/]/.test(path) ||
    path.startsWith("\\\\") ||
    path.startsWith("/");
  const looksFolder = !/\.mdx$/i.test(name);
  if (!isAbsolute) {
    return { badge: "相对", name };
  }
  return { badge: looksFolder ? "文件夹" : "文件", name };
}
