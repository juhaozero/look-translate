import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { HotkeyStatus } from "../shared/types";
import "./onboarding.css";

const STEPS = 4;

export function OnboardingApp() {
  const [step, setStep] = useState(0);
  const [hotkeys, setHotkeys] = useState<{
    translate: string;
    ocr: string;
    input: string;
  }>({
    translate: "Ctrl+Shift+D",
    ocr: "Ctrl+Shift+S",
    input: "Ctrl+Shift+T",
  });

  useEffect(() => {
    void invoke<HotkeyStatus>("get_hotkey_status")
      .then((status) => {
        setHotkeys({
          translate: status.translate || "Ctrl+Shift+D",
          ocr: status.ocr || "Ctrl+Shift+S",
          input: status.input || "Ctrl+Shift+T",
        });
      })
      .catch((error) => {
        console.error(error);
      });
  }, []);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        void dismiss();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);

  async function dismiss() {
    try {
      await invoke("complete_onboarding_window");
    } catch (error) {
      console.error(error);
    }
  }

  async function finishAndOpenSettings() {
    try {
      await invoke("complete_onboarding_window");
      // Last step introduces 服务设置 — land on that panel.
      await invoke("show_settings_window", { nav: "service" });
    } catch (error) {
      console.error(error);
    }
  }

  async function openHotkeySettings() {
    try {
      await invoke("show_settings_window", { nav: "hotkey" });
    } catch (error) {
      console.error(error);
    }
  }

  async function refreshHotkeys() {
    try {
      const status = await invoke<HotkeyStatus>("get_hotkey_status");
      setHotkeys({
        translate: status.translate || "Ctrl+Shift+D",
        ocr: status.ocr || "Ctrl+Shift+S",
        input: status.input || "Ctrl+Shift+T",
      });
    } catch (error) {
      console.error(error);
    }
  }

  useEffect(() => {
    if (step === 1 || step === 2) {
      void refreshHotkeys();
    }
  }, [step]);

  const isLast = step >= STEPS - 1;

  return (
    <div className="onboarding">
      <header className="onboarding-top">
        <div className="onboarding-brand">
          <span className="onboarding-mark" aria-hidden="true">
            文
          </span>
          <div>
            <p className="onboarding-eyebrow">Look Translate</p>
            <h1>快速上手</h1>
          </div>
        </div>
        <button type="button" className="onboarding-skip" onClick={() => void dismiss()}>
          跳过
        </button>
      </header>

      <div className="onboarding-progress" aria-hidden="true">
        {Array.from({ length: STEPS }, (_, index) => (
          <span
            key={index}
            className={
              index === step
                ? "onboarding-dot is-active"
                : index < step
                  ? "onboarding-dot is-done"
                  : "onboarding-dot"
            }
          />
        ))}
      </div>

      <main className="onboarding-body" aria-live="polite">
        {step === 0 ? (
          <section className="onboarding-step">
            <div className="onboarding-icon" aria-hidden="true">
              <svg viewBox="0 0 48 48" fill="none">
                <rect x="8" y="28" width="32" height="12" rx="3" stroke="currentColor" strokeWidth="2" />
                <circle cx="14" cy="34" r="2" fill="currentColor" />
                <path d="M20 34h14" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
              </svg>
            </div>
            <h2>软件在系统托盘里</h2>
            <p>
              安装后 Look Translate 常驻托盘，不会占满任务栏。双击托盘图标或右键打开「设置」。
            </p>
          </section>
        ) : null}

        {step === 1 ? (
          <section className="onboarding-step">
            <div className="onboarding-icon" aria-hidden="true">
              <svg viewBox="0 0 48 48" fill="none">
                <path
                  d="M12 16h24v16H12z"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinejoin="round"
                />
                <path d="M16 22h10M16 26h16" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
              </svg>
            </div>
            <h2>选中文字，按划词热键</h2>
            <p>在任意窗口选中文本后按下热键，译文会在光标附近浮层显示。</p>
            <kbd className="onboarding-hotkey">{hotkeys.translate}</kbd>
            <p className="onboarding-hint">
              热键可自定义。
              <button
                type="button"
                className="onboarding-link"
                onClick={() => void openHotkeySettings()}
              >
                打开设置
              </button>
            </p>
          </section>
        ) : null}

        {step === 2 ? (
          <section className="onboarding-step">
            <div className="onboarding-icon" aria-hidden="true">
              <svg viewBox="0 0 48 48" fill="none">
                <rect x="10" y="12" width="28" height="22" rx="3" stroke="currentColor" strokeWidth="2" />
                <path d="M16 28l5-6 4 4 7-9" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
              </svg>
            </div>
            <h2>还能 OCR 与输入翻译</h2>
            <p>框选屏幕文字，或打开输入框手动翻译。</p>
            <ul className="onboarding-hotkey-list">
              <li>
                <span>OCR 识别</span>
                <kbd className="onboarding-hotkey is-compact">{hotkeys.ocr}</kbd>
              </li>
              <li>
                <span>输入翻译</span>
                <kbd className="onboarding-hotkey is-compact">{hotkeys.input}</kbd>
              </li>
            </ul>
            <p className="onboarding-hint">
              热键可自定义。
              <button
                type="button"
                className="onboarding-link"
                onClick={() => void openHotkeySettings()}
              >
                打开设置
              </button>
            </p>
          </section>
        ) : null}

        {step === 3 ? (
          <section className="onboarding-step">
            <div className="onboarding-icon" aria-hidden="true">
              <svg viewBox="0 0 48 48" fill="none">
                <path
                  d="M14 14h20v20H14z"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinejoin="round"
                />
                <path
                  d="M18 22h12M18 28h8"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                />
                <circle cx="34" cy="14" r="5" stroke="currentColor" strokeWidth="2" />
                <path
                  d="M34 12v4M32 14h4"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  strokeLinecap="round"
                />
              </svg>
            </div>
            <h2>按需开启翻译服务</h2>
            <p>
              打开「设置 → 服务设置」，可同时启用多个引擎并行出译。免 Key
              的网页引擎开箱可用；微软 / Google / DeepL 等需自行填写密钥。
            </p>
            <ul className="onboarding-tips">
              <li>默认可先用免 Key 服务开始翻译</li>
              <li>密钥只保存在本机 `data/config.toml`</li>
            </ul>
          </section>
        ) : null}
      </main>

      <footer className="onboarding-actions">
        {step > 0 ? (
          <button type="button" className="onboarding-btn is-ghost" onClick={() => setStep((s) => s - 1)}>
            上一步
          </button>
        ) : (
          <span />
        )}
        {isLast ? (
          <div className="onboarding-finish">
            <button type="button" className="onboarding-btn is-ghost" onClick={() => void finishAndOpenSettings()}>
              打开设置
            </button>
            <button type="button" className="onboarding-btn is-primary" onClick={() => void dismiss()}>
              开始使用
            </button>
          </div>
        ) : (
          <button type="button" className="onboarding-btn is-primary" onClick={() => setStep((s) => s + 1)}>
            下一步
          </button>
        )}
      </footer>
    </div>
  );
}
