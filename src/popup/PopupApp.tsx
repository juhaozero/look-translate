import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppConfig, CapturePayload, TranslationPayload } from "../shared/types";
import { TARGET_LANGS, normalizeLangCode, resolveEngineLabel } from "../shared/options";
import { EngineIcon } from "../shared/EngineIcon";
import "./popup.css";

export function PopupApp() {
  const [capture, setCapture] = useState<CapturePayload | null>(null);
  const [translation, setTranslation] = useState<TranslationPayload | null>(
    null,
  );
  const [targetLang, setTargetLang] = useState("zh-CN");
  const [copyStatus, setCopyStatus] = useState("");
  const [engineProfiles, setEngineProfiles] = useState<AppConfig["engines"]>(
    {},
  );

  useEffect(() => {
    invoke<AppConfig>("get_config")
      .then((config) => {
        if (config.general?.target_lang) {
          setTargetLang(normalizeLangCode(config.general.target_lang) || "zh-CN");
        }
        setEngineProfiles(config.engines ?? {});
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

  const captureFailed = Boolean(capture?.error);
  const sourceText = captureFailed
    ? undefined
    : capture && !capture.empty
      ? capture.text
      : translation?.sourceText;
  const canTranslate = Boolean(sourceText && sourceText.trim());
  const translatedText =
    captureFailed || translation?.status !== "ok"
      ? ""
      : (translation?.translatedText ?? "");
  const isOcr = capture?.source === "ocr";
  const isLoading = !captureFailed && translation?.status === "loading";

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
      <div className="popup-accent" aria-hidden="true" />

      <header className="popup-header">
        <h1 className={isOcr ? "popup-title is-ocr" : "popup-title"}>
          {isOcr ? "OCR 翻译" : "划词翻译"}
        </h1>
        <button
          type="button"
          className="popup-icon-btn"
          title="关闭 (Esc)"
          aria-label="关闭"
          onClick={() => void invoke("hide_popup_window")}
        >
          <IconClose />
        </button>
      </header>

      <div className="popup-toolbar">
        <label className="popup-lang">
          <span>目标语</span>
          <select
            value={targetLang}
            disabled={!canTranslate || isLoading}
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
          className="popup-btn popup-btn-primary"
          disabled={!translatedText || translation?.status !== "ok"}
          onClick={() => void copyTranslation()}
        >
          {copyStatus || "复制译文"}
        </button>
      </div>

      {capture?.error ? (
        <p className="popup-error" role="alert">
          {capture.error}
        </p>
      ) : null}

      {sourceText ? (
        <section className="popup-block popup-source">
          <div className="popup-block-head">
            <h2>原文</h2>
          </div>
          <p>{sourceText}</p>
        </section>
      ) : (
        <div className="popup-empty">
          <p>选中文本后按划词热键</p>
          <span>或将指针移到文字上按 OCR 热键。Esc / 点外部可关闭。</span>
        </div>
      )}

      {isLoading ? (
        <section className="popup-block popup-skeleton" aria-live="polite">
          <div className="popup-block-head">
            <h2>译文</h2>
            <span className="popup-chip">翻译中</span>
          </div>
          <div className="popup-skeleton-lines">
            <span />
            <span />
            <span />
          </div>
        </section>
      ) : null}

      {translation?.status === "ok" && translatedText ? (
        <section className="popup-block popup-result">
          <div className="popup-block-head">
            <h2>译文</h2>
            <div className="popup-meta">
              {translation.engine ? (
                <span className="popup-chip popup-chip-engine">
                  <EngineIcon engine={translation.engine} size="sm" />
                  <span>
                    {resolveEngineLabel(
                      translation.engine,
                      engineProfiles,
                    )}
                  </span>
                </span>
              ) : null}
              {translation.detectedSourceLang ? (
                <span className="popup-chip">
                  检测 {translation.detectedSourceLang}
                </span>
              ) : null}
              {translation.cached ? (
                <span className="popup-chip is-soft">缓存</span>
              ) : null}
            </div>
          </div>
          <p>{translatedText}</p>
        </section>
      ) : null}

      {translation?.status === "error" && translation.error ? (
        <div className="popup-translate-error" role="alert">
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

      {translation?.dictionaryText && !captureFailed ? (
        <section className="popup-block popup-dict">
          <div className="popup-block-head">
            <h2>词典</h2>
            {translation.dictionarySource ? (
              <span className="popup-chip is-soft">
                {translation.dictionarySource}
              </span>
            ) : null}
          </div>
          <p>{translation.dictionaryText}</p>
        </section>
      ) : null}
    </main>
  );
}

function IconClose() {
  return (
    <svg viewBox="0 0 20 20" width="14" height="14" aria-hidden="true">
      <path
        d="M5 5l10 10M15 5 5 15"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.75"
        strokeLinecap="round"
      />
    </svg>
  );
}
