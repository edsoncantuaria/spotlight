import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ShortcutValidation {
  shortcut: string;
  valid_format: boolean;
  registrable: boolean;
  available: boolean;
  gnome_conflict?: string | null;
  message: string;
}

function formatShortcut(e: KeyboardEvent): string | null {
  if (e.key === "Escape" || e.key === "Tab") return null;
  if (["Control", "Alt", "Shift", "Meta"].includes(e.key)) return null;

  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Super");

  let key = e.key;
  if (key === " ") key = "Space";
  else if (key.length === 1) key = key.toUpperCase();
  parts.push(key);

  if (parts.length < 2) return null;
  return parts.join("+");
}

interface ShortcutFieldProps {
  label: string;
  hint?: string;
  value: string;
  onChange: (value: string) => void;
}

export default function ShortcutField({
  label,
  hint,
  value,
  onChange,
}: ShortcutFieldProps) {
  const [capturing, setCapturing] = useState(false);
  const [checking, setChecking] = useState(false);
  const [result, setResult] = useState<ShortcutValidation | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const test = useCallback(async (shortcut: string) => {
    const trimmed = shortcut.trim();
    if (!trimmed) {
      setResult(null);
      return;
    }
    setChecking(true);
    try {
      const res = await invoke<ShortcutValidation>("validate_shortcut", {
        shortcut: trimmed,
      });
      setResult(res);
    } catch (e) {
      setResult({
        shortcut: trimmed,
        valid_format: false,
        registrable: false,
        available: false,
        message: String(e),
      });
    } finally {
      setChecking(false);
    }
  }, []);

  useEffect(() => {
    if (!capturing) return;

    const onKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      e.stopPropagation();
      const formatted = formatShortcut(e);
      if (formatted) {
        onChange(formatted);
        setCapturing(false);
        void test(formatted);
      }
    };

    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [capturing, onChange, test]);

  const statusClass =
    result == null
      ? ""
      : result.available && result.valid_format
        ? " ok"
        : " err";

  return (
    <label className="sl-field sl-shortcut-field">
      <span className="sl-field-label">{label}</span>
      <div className="sl-shortcut-row">
        <input
          ref={inputRef}
          value={value}
          onChange={(e) => {
            onChange(e.target.value);
            setResult(null);
          }}
          onBlur={() => {
            if (value.trim()) void test(value);
          }}
          placeholder="Ctrl+Alt+Space"
        />
        <button
          type="button"
          className={`sl-btn${capturing ? " sl-btn-primary" : ""}`}
          onClick={() => setCapturing((c) => !c)}
        >
          {capturing ? "Pressione…" : "Capturar"}
        </button>
        <button
          type="button"
          className="sl-btn"
          disabled={checking || !value.trim()}
          onClick={() => test(value)}
        >
          {checking ? "…" : "Testar"}
        </button>
      </div>
      {hint && <span className="sl-field-hint">{hint}</span>}
      {result && (
        <span className={`sl-shortcut-status${statusClass}`}>{result.message}</span>
      )}
    </label>
  );
}
