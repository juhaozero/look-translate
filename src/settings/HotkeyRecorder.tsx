import { useEffect, useId, useState } from "react";

type HotkeyRecorderProps = {
  label: string;
  value: string;
  onChange: (next: string) => void;
  disabled?: boolean;
  hint?: string;
};

export function HotkeyRecorder({
  label,
  value,
  onChange,
  disabled = false,
  hint,
}: HotkeyRecorderProps) {
  const id = useId();
  const [recording, setRecording] = useState(false);
  const [draft, setDraft] = useState(value);

  useEffect(() => {
    if (!recording) {
      setDraft(value);
    }
  }, [value, recording]);

  useEffect(() => {
    if (!recording) {
      return;
    }

    const onKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();

      if (event.key === "Escape") {
        setDraft(value);
        setRecording(false);
        return;
      }

      if (event.key === "Backspace" || event.key === "Delete") {
        setDraft("");
        return;
      }

      const combo = formatHotkeyEvent(event);
      if (combo) {
        setDraft(combo);
      }
    };

    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [recording, value]);

  function startRecording() {
    if (disabled) {
      return;
    }
    setDraft(value);
    setRecording(true);
  }

  function confirmDraft() {
    if (!draft.trim()) {
      return;
    }
    onChange(draft.trim());
    setRecording(false);
  }

  function cancelRecording() {
    setDraft(value);
    setRecording(false);
  }

  return (
    <div className="settings-row settings-row-hotkey">
      <label className="settings-row-label" htmlFor={id}>
        {label}
      </label>
      <div className="hotkey-recorder">
        <button
          id={id}
          type="button"
          className={
            recording
              ? "hotkey-recorder-field is-recording"
              : "hotkey-recorder-field"
          }
          disabled={disabled}
          aria-pressed={recording}
          aria-label={
            recording ? "按下按键设置快捷键" : label
          }
          onClick={() => {
            if (recording) {
              return;
            }
            startRecording();
          }}
        >
          {recording ? (
            <span
              className={
                draft.trim()
                  ? "hotkey-recorder-value has-draft"
                  : "hotkey-recorder-value"
              }
            >
              {draft.trim() || "按下按键设置…"}
            </span>
          ) : (
            <span className="hotkey-recorder-value">
              {value || "点击后按下快捷键"}
            </span>
          )}
        </button>
        {recording ? (
          <div className="hotkey-recorder-actions">
            <button
              type="button"
              className="settings-btn settings-btn-primary"
              disabled={!draft.trim()}
              onClick={confirmDraft}
            >
              确定
            </button>
            <button
              type="button"
              className="settings-btn"
              onClick={cancelRecording}
            >
              取消
            </button>
          </div>
        ) : null}
      </div>
      {hint ? <p className="settings-row-hint">{hint}</p> : null}
    </div>
  );
}

/** Map a KeyboardEvent to tauri global-shortcut style, e.g. `Ctrl+Shift+D`. */
export function formatHotkeyEvent(event: KeyboardEvent): string | null {
  const key = normalizeKeyToken(event);
  if (!key) {
    return null;
  }

  const parts: string[] = [];
  if (event.ctrlKey) {
    parts.push("Ctrl");
  }
  if (event.altKey) {
    parts.push("Alt");
  }
  if (event.shiftKey) {
    parts.push("Shift");
  }
  if (event.metaKey) {
    parts.push("Super");
  }
  parts.push(key);
  return parts.join("+");
}

function normalizeKeyToken(event: KeyboardEvent): string | null {
  const { key, code } = event;

  if (
    key === "Control" ||
    key === "Shift" ||
    key === "Alt" ||
    key === "Meta" ||
    key === "OS"
  ) {
    return null;
  }

  if (/^F\d{1,2}$/i.test(key)) {
    return key.toUpperCase();
  }

  if (key.length === 1) {
    const upper = key.toUpperCase();
    if (/^[A-Z0-9]$/.test(upper)) {
      return upper;
    }
  }

  const codeMap: Record<string, string> = {
    Space: "Space",
    Enter: "Enter",
    Tab: "Tab",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
    Escape: "Esc",
    Backquote: "`",
    Minus: "-",
    Equal: "=",
    BracketLeft: "[",
    BracketRight: "]",
    Backslash: "\\",
    Semicolon: ";",
    Quote: "'",
    Comma: ",",
    Period: ".",
    Slash: "/",
  };

  if (codeMap[code]) {
    return codeMap[code];
  }

  if (code.startsWith("Digit") && code.length === 6) {
    return code.slice(5);
  }
  if (code.startsWith("Key") && code.length === 4) {
    return code.slice(3);
  }
  if (code.startsWith("Numpad")) {
    return code.replace("Numpad", "Num");
  }

  if (key.length > 0 && key !== "Unidentified") {
    return key.length === 1 ? key.toUpperCase() : key;
  }

  return null;
}
