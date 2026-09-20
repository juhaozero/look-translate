import { HotkeyRecorder } from "../HotkeyRecorder";
import { ToggleRow } from "../ui";
import type { HotkeyPanelProps } from "./types";

export function HotkeyPanel({
  config,
  updateGeneral,
  hotkeyStatus,
}: HotkeyPanelProps) {
  return (
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
  );
}
