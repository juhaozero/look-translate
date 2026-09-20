import type { GeneralPanelProps } from "./types";
import { ToggleRow, RowAction } from "../ui";

export function GeneralPanel({
  config,
  updateGeneral,
  onOpenPopupPreview,
}: GeneralPanelProps) {
  return (
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
          onChange={(checked) => updateGeneral("launch_at_startup", checked)}
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
          onClick={() => void onOpenPopupPreview()}
        />
      </section>
    </>
  );
}
