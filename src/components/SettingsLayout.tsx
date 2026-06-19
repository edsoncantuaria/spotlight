import type { ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "../styles/settings.css";

export interface SettingsNavItem {
  id: string;
  label: string;
  icon: ReactNode;
}

interface SettingsLayoutProps {
  title: string;
  subtitle?: string;
  nav: SettingsNavItem[];
  active: string;
  onNav: (id: string) => void;
  children: ReactNode;
  footer?: ReactNode;
  theme?: string;
}

export default function SettingsLayout({
  title,
  subtitle,
  nav,
  active,
  onNav,
  children,
  footer,
  theme = "auto",
}: SettingsLayoutProps) {
  return (
    <div className="sl-root" data-theme={theme}>
      <aside className="sl-sidebar">
        <div className="sl-brand">
          <div className="sl-brand-icon" aria-hidden>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
              <circle cx="11" cy="11" r="7" />
              <path d="m20 20-3.5-3.5" />
            </svg>
          </div>
          <div>
            <strong>Spotlight</strong>
            <span>{title}</span>
          </div>
        </div>

        <nav className="sl-nav">
          {nav.map((item) => (
            <button
              key={item.id}
              type="button"
              className={`sl-nav-item${active === item.id ? " active" : ""}`}
              onClick={() => onNav(item.id)}
            >
              <span className="sl-nav-icon">{item.icon}</span>
              {item.label}
            </button>
          ))}
        </nav>
      </aside>

      <main className="sl-main">
        <header className="sl-header">
          <div>
            <h1>{nav.find((n) => n.id === active)?.label ?? title}</h1>
            {subtitle && <p>{subtitle}</p>}
          </div>
          <button
            type="button"
            className="sl-btn sl-btn-ghost"
            onClick={() => getCurrentWindow().close()}
          >
            Fechar
          </button>
        </header>

        <div className="sl-content">{children}</div>

        {footer && <footer className="sl-footer">{footer}</footer>}
      </main>
    </div>
  );
}

export function SlField({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <label className="sl-field">
      <span className="sl-field-label">{label}</span>
      {children}
      {hint && <span className="sl-field-hint">{hint}</span>}
    </label>
  );
}

export function SlCard({
  title,
  description,
  children,
}: {
  title?: string;
  description?: string;
  children: ReactNode;
}) {
  return (
    <section className="sl-card">
      {(title || description) && (
        <div className="sl-card-head">
          {title && <h2>{title}</h2>}
          {description && <p>{description}</p>}
        </div>
      )}
      <div className="sl-card-body">{children}</div>
    </section>
  );
}

export function SlToggle({
  checked,
  onChange,
  label,
  description,
  disabled,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
  label: string;
  description?: string;
  disabled?: boolean;
}) {
  return (
    <div className={`sl-toggle-row${disabled ? " disabled" : ""}`}>
      <div>
        <strong>{label}</strong>
        {description && <p>{description}</p>}
      </div>
      <button
        type="button"
        role="switch"
        aria-checked={checked}
        disabled={disabled}
        className={`sl-switch${checked ? " on" : ""}`}
        onClick={() => !disabled && onChange(!checked)}
      >
        <span />
      </button>
    </div>
  );
}
