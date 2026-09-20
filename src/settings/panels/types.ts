import type { Dispatch, SetStateAction } from "react";
import type { AppConfig, AppInfo, AppPaths, HotkeyStatus } from "../../shared/types";
import type { EngineListItem } from "../../shared/options";

export type UpdateGeneral = <K extends keyof AppConfig["general"]>(
  key: K,
  value: AppConfig["general"][K],
) => void;

export type GeneralPanelProps = {
  config: AppConfig;
  updateGeneral: UpdateGeneral;
  onOpenPopupPreview: () => void;
};

export type TranslatePanelProps = {
  config: AppConfig;
  updateGeneral: UpdateGeneral;
};

export type HotkeyPanelProps = {
  config: AppConfig;
  updateGeneral: UpdateGeneral;
  hotkeyStatus: HotkeyStatus | null;
};

export type ServicePanelProps = {
  config: AppConfig;
  setConfig: Dispatch<SetStateAction<AppConfig>>;
  paths: AppPaths | null;
  engineList: EngineListItem[];
  editingEngine: string | null;
  profileBusy: boolean;
  onEngineToggle: (engineId: string, turnOn: boolean) => void;
  onSelectOcrEngine: (engineId: string) => void;
  onOcrEngineToggle: (engineId: string, turnOn: boolean) => void;
  onToggleEngineEditor: (engineId: string) => void;
  onOpenConfigFile: () => void;
  onReloadConfigFromDisk: () => void;
  onAddCustomTemplate: () => void;
};

export type DictionaryPanelProps = {
  config: AppConfig;
  setConfig: Dispatch<SetStateAction<AppConfig>>;
  paths: AppPaths | null;
  installingDict: boolean;
  loading: boolean;
  onInstallRecommendedDict: () => void;
  onPickDictionaryFile: () => void;
  onPickDictionaryFolder: () => void;
};

export type AboutPanelProps = {
  info: AppInfo | null;
  paths: AppPaths | null;
  onOpenExternalUrl: (url: string) => void;
  onOpenLogDir: () => void;
  onOpenConfigFile: () => void;
  onOpenDataDir: () => void;
};
