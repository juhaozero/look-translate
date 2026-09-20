import { SOURCE_LANGS, TARGET_LANGS } from "../../shared/options";
import { SelectRow } from "../ui";
import type { TranslatePanelProps } from "./types";

export function TranslatePanel({
  config,
  updateGeneral,
}: TranslatePanelProps) {
  return (
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
  );
}
