import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AppInfo,
  CapturePayload,
  TranslationPayload,
} from "../shared/types";
import { TARGET_LANGS, normalizeLangCode } from "../shared/options";
import "./popup.css";

export function PopupApp() {
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [capture, setCapture] = useState<CapturePayload | null>(null);
  const [translation, setTranslation] = useState<TranslationPayload | null>(
    null,
  );
  const [targetLang, setTargetLang] = useState("zh-CN");
  const [copyStatus, setCopyStatus] = useState("");

  useEffect(() => {
    invoke<AppInfo>("get_app_info")
      .then(setInfo)
      .catch(console.error);

    invoke<{ general: { target_lang: string } }>("get_config")
      .then((config) => {
        if (config.general?.target_lang) {
          setTargetLang(normalizeLangCode(config.general.target_lang) || "zh-CN");
        }
      })
      .catch(console.error);

    invoke<CapturePayload | null>("get_last_capture")
      .then((payload) => {
        if (payload) {
          setCapture(payload);
        }
      })
      .catch(console.error);

    invoke<TranslationPayload | null>("get_last_translation")
      .then((payload) => {
        if (payload) {
          setTranslation(payload);
          if (payload.targetLang) {
            setTargetLang(payload.targetLang);
          }
        }
      })
      .catch(console.error);

    const unlisteners: Array<() => void> = [];

    listen<CapturePayload>("capture-updated", (event) => {
      setCapture(event.payload);
      setTranslation(null);
      setCopyStatus("");
    })
      .then((fn) => unlisteners.push(fn))
      .catch(console.error);

    listen<TranslationPayload>("translation-updated", (event) => {
      setTranslation(event.payload);
      if (event.payload.targetLang) {
        setTargetLang(event.payload.targetLang);
      }
    })
      .then((fn) => unlisteners.push(fn))
      .catch(console.error);

    return () => {
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        void invoke("hide_popup_window");
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  const sourceText =
    capture && !capture.empty ? capture.text : translation?.sourceText;
  const canTranslate = Boolean(sourceText && sourceText.trim());
  const translatedText = translation?.translatedText ?? "";

  async function runTranslate(nextTarget: string) {
    if (!sourceText?.trim()) {
      return;
    }
    setCopyStatus("");
    try {
      await invoke<TranslationPayload>("translate_text", {
        text: sourceText,
        targetLang: nextTarget,
      });
    } catch (error) {
      console.error(error);
    }
  }

  async function onTargetLangChange(next: string) {
    setTargetLang(next);
    await runTranslate(next);
  }

  async function copyTranslation() {
    if (!translatedText) {
      return;
    }
    try {
      await invoke("copy_text", { text: translatedText });
      setCopyStatus("已复制");
      window.setTimeout(() => setCopyStatus(""), 1500);
    } catch (error) {
      console.error(error);
      setCopyStatus("复制失败");
    }
  }

  return (
    <main className="popup-shell">
      <header className="popup-header">
        <span>
          {capture?.source === "ocr" ? "OCR 翻译" : "划词翻译"}
        </span>
        <button
          type="button"
          className="popup-icon-btn"
          title="关闭 (Esc)"
          onClick={() => void invoke("hide_popup_window")}
        >
          ×
        </button>
      </header>

      <div className="popup-toolbar">
        <label className="popup-lang">
          <span>目标语</span>
          <select
            value={targetLang}
            disabled={!canTranslate || translation?.status === "loading"}
            onChange={(event) => void onTargetLangChange(event.target.value)}
          >
            {TARGET_LANGS.map((lang) => (
              <option key={lang.value} value={lang.value}>
                {lang.label}
              </option>
            ))}
            {!TARGET_LANGS.some((lang) => lang.value === targetLang) ? (
              <option value={targetLang}>{targetLang}</option>
            ) : null}
          </select>
        </label>
        <button
          type="button"
          className="popup-btn"
          disabled={!translatedText || translation?.status !== "ok"}
          onClick={() => void copyTranslation()}
        >
          {copyStatus || "复制译文"}
        </button>
      </div>

      {capture?.error ? <p className="popup-error">{capture.error}</p> : null}

      {sourceText ? (
        <section className="popup-source">
          <h2>原文</h2>
          <p>{sourceText}</p>
        </section>
      ) : (
        <p className="popup-placeholder">
          选中文本后按划词热键，或将指针移到文字上按 OCR 热键。Esc
          或点击外部可关闭。
        </p>
      )}

      {translation?.status === "loading" ? (
        <p className="popup-hint popup-loading">翻译中…</p>
      ) : null}

      {translation?.status === "ok" && translatedText ? (
        <section className="popup-result">
          <h2>
            译文
            {translation.engine ? ` · ${translation.engine}` : ""}
            {translation.detectedSourceLang
              ? ` · 检测 ${translation.detectedSourceLang}`
              : ""}
            {translation.cached ? " · 缓存" : ""}
          </h2>
          <p>{translatedText}</p>
        </section>
      ) : null}

      {translation?.status === "error" && translation.error ? (
        <div className="popup-translate-error">
          <p className="popup-error">{translation.error}</p>
          <button
            type="button"
            className="popup-btn"
            onClick={() => void runTranslate(targetLang)}
          >
            重试
          </button>
        </div>
      ) : null}

      {translation?.dictionaryText ? (
        <section className="popup-dict">
          <h2>
            词典
            {translation.dictionarySource
              ? ` · ${translation.dictionarySource}`
              : ""}
          </h2>
          <p>{translation.dictionaryText}</p>
        </section>
      ) : null}

      {info ? (
        <footer className="popup-footer">
          {info.name} · {info.version} · {info.phase}
        </footer>
      ) : null}
    </main>
  );
}
