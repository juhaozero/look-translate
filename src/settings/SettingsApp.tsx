import {
  useEffect,
  useMemo,
  useRef,
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
import type { EngineCatalogItem } from "../shared/types";
import {
  buildEngineList,
  catalogToEngineList,
  resolveActives,
  type EngineListItem,
} from "../shared/options";
import {
  IconAbout,
  IconApp,
  IconDict,
  IconGeneral,
  IconHotkey,
  IconService,
  IconTranslate,
} from "./icons";
import {
  mergePath,
  normalizeConfig,
  serializeConfig,
  validateConfig,
} from "./configUtils";
import { GeneralPanel } from "./panels/GeneralPanel";
import { TranslatePanel } from "./panels/TranslatePanel";
import { HotkeyPanel } from "./panels/HotkeyPanel";
import { ServicePanel } from "./panels/ServicePanel";
import { DictionaryPanel } from "./panels/DictionaryPanel";
import { AboutPanel } from "./panels/AboutPanel";
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

const AUTO_SAVE_MS = 450;

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
  const [engineCatalog, setEngineCatalog] = useState<EngineListItem[] | null>(
    null,
  );

  const skipAutoSaveRef = useRef(true);
  const saveTimerRef = useRef<number | null>(null);
  const saveSeqRef = useRef(0);
  const statusClearRef = useRef<number | null>(null);
  const configRef = useRef(config);
  configRef.current = config;

  const dirty = useMemo(
    () => serializeConfig(config) !== savedSnapshot && savedSnapshot.length > 0,
    [config, savedSnapshot],
  );

  const engineList = useMemo(
    () => engineCatalog ?? buildEngineList(config.engines),
    [engineCatalog, config.engines],
  );

  async function refreshEngineCatalog(nextConfig?: AppConfig) {
    try {
      const catalog = await invoke<EngineCatalogItem[]>("list_engines");
      setEngineCatalog(catalogToEngineList(catalog));
    } catch (error: unknown) {
      console.error(error);
      setEngineCatalog(
        buildEngineList((nextConfig ?? configRef.current).engines),
      );
    }
  }

  function showTransientOk(text: string) {
    setStatus({ tone: "ok", text });
    if (statusClearRef.current !== null) {
      window.clearTimeout(statusClearRef.current);
    }
    statusClearRef.current = window.setTimeout(() => {
      setStatus((current) =>
        current?.tone === "ok" && current.text === text ? null : current,
      );
      statusClearRef.current = null;
    }, 1600);
  }

  function applyConfigFromDisk(next: AppConfig) {
    skipAutoSaveRef.current = true;
    const normalized = normalizeConfig(next);
    setConfig(normalized);
    setSavedSnapshot(serializeConfig(normalized));
    window.setTimeout(() => {
      skipAutoSaveRef.current = false;
    }, 0);
  }

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
      applyConfigFromDisk(appConfig);
      setHotkeyStatus(statusInfo);
      await refreshEngineCatalog(appConfig);
    } catch (error: unknown) {
      console.error(error);
      setStatus({ tone: "err", text: String(error) });
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void loadAll();
    return () => {
      if (saveTimerRef.current !== null) {
        window.clearTimeout(saveTimerRef.current);
      }
      if (statusClearRef.current !== null) {
        window.clearTimeout(statusClearRef.current);
      }
    };
  }, []);

  useEffect(() => {
    if (nav !== "service") {
      setEditingEngine(null);
    }
  }, [nav]);

  useEffect(() => {
    if (loading || skipAutoSaveRef.current || !dirty) {
      return;
    }

    if (saveTimerRef.current !== null) {
      window.clearTimeout(saveTimerRef.current);
    }
    saveTimerRef.current = window.setTimeout(() => {
      saveTimerRef.current = null;
      void persistConfig(configRef.current, { silentOk: true });
    }, AUTO_SAVE_MS);

    return () => {
      if (saveTimerRef.current !== null) {
        window.clearTimeout(saveTimerRef.current);
        saveTimerRef.current = null;
      }
    };
  }, [config, dirty, loading]);

  async function persistConfig(
    nextConfig: AppConfig,
    options?: { silentOk?: boolean },
  ) {
    const validation = validateConfig(nextConfig);
    if (validation) {
      setStatus({ tone: "err", text: validation });
      return false;
    }

    const seq = ++saveSeqRef.current;
    setSaving(true);
    try {
      const payload = normalizeConfig(nextConfig);
      await invoke<AppConfig>("save_config", { config: payload });
      if (seq !== saveSeqRef.current) {
        return true;
      }
      skipAutoSaveRef.current = true;
      setConfig(payload);
      setSavedSnapshot(serializeConfig(payload));
      window.setTimeout(() => {
        skipAutoSaveRef.current = false;
      }, 0);
      try {
        const statusInfo = await invoke<HotkeyStatus>("get_hotkey_status");
        if (seq === saveSeqRef.current) {
          setHotkeyStatus(statusInfo);
        }
      } catch {
        // Hotkey refresh is best-effort after save.
      }
      if (options?.silentOk) {
        showTransientOk("已自动保存");
      }
      return true;
    } catch (error) {
      console.error(error);
      if (seq === saveSeqRef.current) {
        setStatus({ tone: "err", text: String(error) });
      }
      return false;
    } finally {
      if (seq === saveSeqRef.current) {
        setSaving(false);
      }
    }
  }

  async function openPopupPreview() {
    try {
      await invoke("show_popup_window");
    } catch (error) {
      console.error(error);
      setStatus({ tone: "err", text: String(error) });
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
      applyConfigFromDisk(result.config);
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

  async function openLogDir() {
    setStatus(null);
    try {
      await invoke("open_log_dir");
      showTransientOk("已打开日志目录");
      const appPaths = await invoke<AppPaths>("get_app_paths");
      setPaths(appPaths);
    } catch (error) {
      console.error(error);
      setStatus({ tone: "err", text: String(error) });
    }
  }

  async function openDataDir() {
    setStatus(null);
    try {
      await invoke("open_data_dir");
      showTransientOk("已打开数据目录");
    } catch (error) {
      console.error(error);
      setStatus({ tone: "err", text: String(error) });
    }
  }

  async function openExternalUrl(url: string) {
    setStatus(null);
    try {
      await invoke("open_url", { url });
    } catch (error) {
      console.error(error);
      setStatus({ tone: "err", text: String(error) });
    }
  }

  async function reloadConfigFromDisk() {
    if (dirty || saving) {
      const ok = window.confirm(
        "当前有未写入磁盘的修改，重新加载将丢弃这些修改。继续？",
      );
      if (!ok) {
        return;
      }
    }
    if (saveTimerRef.current !== null) {
      window.clearTimeout(saveTimerRef.current);
      saveTimerRef.current = null;
    }
    setStatus(null);
    try {
      const next = await invoke<AppConfig>("reload_config");
      applyConfigFromDisk(next);
      await refreshEngineCatalog(next);
      showTransientOk("已从磁盘重新加载配置");
    } catch (error) {
      console.error(error);
      setStatus({ tone: "err", text: String(error) });
    }
  }

  async function addCustomTemplate() {
    if (profileBusy) return;
    if (dirty || saving) {
      const ok = window.confirm(
        "当前还有未自动保存完的修改。添加模板会写入配置文件，可能覆盖这些修改。继续？",
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
      applyConfigFromDisk(result.config);
      await refreshEngineCatalog(result.config);
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

  function updateGeneral<K extends keyof AppConfig["general"]>(
    key: K,
    value: AppConfig["general"][K],
  ) {
    setConfig({
      ...config,
      general: { ...config.general, [key]: value },
    });
  }

  function onEngineToggle(engineId: string, turnOn: boolean) {
    const current = resolveActives(config.engine);
    if (turnOn) {
      if (current.includes(engineId)) {
        return;
      }
      const actives = [...current, engineId];
      setConfig({
        ...config,
        engine: {
          ...config.engine,
          actives,
          active: actives[0] ?? engineId,
        },
      });
      setStatus(null);
      return;
    }
    if (current.length <= 1 && current.includes(engineId)) {
      setStatus({
        tone: "err",
        text: "请至少保留一个翻译服务",
      });
      return;
    }
    const actives = current.filter((id) => id !== engineId);
    setConfig({
      ...config,
      engine: {
        ...config.engine,
        actives,
        active: actives[0] ?? "microsoft",
      },
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
            ) : saving || dirty ? (
              <p className="settings-subtle">自动保存中…</p>
            ) : (
              <p className="settings-subtle">更改后自动保存</p>
            )}
          </div>
        </header>

        <div className="settings-content">
          {nav === "general" ? (
            <GeneralPanel
              config={config}
              updateGeneral={updateGeneral}
              onOpenPopupPreview={openPopupPreview}
            />
          ) : null}

          {nav === "translate" ? (
            <TranslatePanel
              config={config}
              updateGeneral={updateGeneral}
            />
          ) : null}

          {nav === "hotkey" ? (
            <HotkeyPanel
              config={config}
              updateGeneral={updateGeneral}
              hotkeyStatus={hotkeyStatus}
            />
          ) : null}

          {nav === "service" ? (
            <ServicePanel
              config={config}
              setConfig={setConfig}
              paths={paths}
              engineList={engineList}
              editingEngine={editingEngine}
              profileBusy={profileBusy}
              onEngineToggle={onEngineToggle}
              onSelectOcrEngine={selectOcrEngine}
              onOcrEngineToggle={onOcrEngineToggle}
              onToggleEngineEditor={toggleEngineEditor}
              onOpenConfigFile={openConfigFile}
              onReloadConfigFromDisk={reloadConfigFromDisk}
              onAddCustomTemplate={addCustomTemplate}
            />
          ) : null}

          {nav === "dictionary" ? (
            <DictionaryPanel
              config={config}
              setConfig={setConfig}
              paths={paths}
              installingDict={installingDict}
              loading={loading}
              onInstallRecommendedDict={installRecommendedDict}
              onPickDictionaryFile={pickDictionaryFile}
              onPickDictionaryFolder={pickDictionaryFolder}
            />
          ) : null}

          {nav === "about" ? (
            <AboutPanel
              info={info}
              paths={paths}
              onOpenExternalUrl={openExternalUrl}
              onOpenLogDir={openLogDir}
              onOpenConfigFile={openConfigFile}
              onOpenDataDir={openDataDir}
            />
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
