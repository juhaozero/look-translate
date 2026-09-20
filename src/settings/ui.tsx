import { useId } from "react";

export function ToggleRow({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  const id = useId();
  return (
    <div className="settings-row">
      <label className="settings-row-label" htmlFor={id}>
        {label}
      </label>
      <button
        id={id}
        type="button"
        role="switch"
        aria-checked={checked}
        className={checked ? "settings-switch is-on" : "settings-switch"}
        onClick={() => onChange(!checked)}
      >
        <span className="settings-switch-thumb" />
      </button>
    </div>
  );
}

export function SelectRow({
  label,
  value,
  options,
  onChange,
}: {
  label: string;
  value: string;
  options: ReadonlyArray<{ value: string; label: string }>;
  onChange: (value: string) => void;
}) {
  const id = useId();
  const known = options.some((item) => item.value === value);
  return (
    <div className="settings-row">
      <label className="settings-row-label" htmlFor={id}>
        {label}
      </label>
      <select
        id={id}
        className="settings-select"
        value={value}
        onChange={(event) => onChange(event.target.value)}
      >
        {options.map((item) => (
          <option key={item.value} value={item.value}>
            {item.label}
          </option>
        ))}
        {!known ? <option value={value}>{value}</option> : null}
      </select>
    </div>
  );
}

export function RowAction({
  label,
  actionLabel,
  onClick,
}: {
  label: string;
  actionLabel: string;
  onClick: () => void;
}) {
  return (
    <div className="settings-row">
      <span className="settings-row-label">{label}</span>
      <button type="button" className="settings-btn" onClick={onClick}>
        {actionLabel}
      </button>
    </div>
  );
}
