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
          划词热键获取选中文本；OCR 热键框选识别；输入翻译热键打开文本输入框。
        </p>
        <HotkeyRecorder
          label="输入翻译"
          value={config.general.hotkey_input}
          onChange={(next) => updateGeneral("hotkey_input", next)}
        />
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
        {hotkeyStatus?.enabled ? (
          <p className={hotkeyStatus.inputRegistered ? "settings-panel-foot" : "settings-panel-foot is-warn"}>
            {hotkeyStatus.inputRegistered ? "输入翻译已注册" : "输入翻译未注册"}：{hotkeyStatus.input}
          </p>
        ) : null}
      </section>
    </>
  );
}
