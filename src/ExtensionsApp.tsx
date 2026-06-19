import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import SettingsLayout from "./components/SettingsLayout";

interface ExtensionInfo {
  id: string;
  title: string;
  enabled: boolean;
  builtin: boolean;
  keywords: string[];
  icon?: string | null;
}

type ExtFilter = "all" | "active" | "disabled" | "builtin" | "user";

const BUILTIN_DESC: Record<string, string> = {
  emoji: "Busca e copia emojis por nome.",
  notes: "Notas rápidas salvas localmente.",
  git: "Status, branches e comandos git no diretório atual.",
  docker: "Lista containers em execução.",
  systemd: "Gerencia serviços systemd do usuário.",
  calculator: "Calculadora e conversões numéricas.",
  translate: "Tradução de texto via API configurável.",
  integrations: "Atalhos para serviços e ferramentas comuns.",
  ai: "Perguntas via Ollama ou API de IA.",
};

const BUILTIN_ICON: Record<string, string> = {
  emoji: "😀",
  notes: "📝",
  git: "⎇",
  docker: "🐳",
  systemd: "⚙",
  calculator: "🧮",
  translate: "🌐",
  integrations: "🔗",
  ai: "✨",
};

const FILTER_CHIPS: { id: ExtFilter; label: string }[] = [
  { id: "all", label: "Todas" },
  { id: "active", label: "Ativas" },
  { id: "disabled", label: "Desativadas" },
  { id: "builtin", label: "Integradas" },
  { id: "user", label: "Usuário" },
];

export default function ExtensionsApp() {
  const [items, setItems] = useState<ExtensionInfo[]>([]);
  const [query, setQuery] = useState("");
  const [chip, setChip] = useState<ExtFilter>("all");
  const [loading, setLoading] = useState(true);
  const [status, setStatus] = useState("");

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<ExtensionInfo[]>("list_extensions");
      setItems(data);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const filtered = useMemo(() => {
    let list = items;
    switch (chip) {
      case "active":
        list = list.filter((e) => e.enabled);
        break;
      case "disabled":
        list = list.filter((e) => !e.enabled);
        break;
      case "builtin":
        list = list.filter((e) => e.builtin);
        break;
      case "user":
        list = list.filter((e) => !e.builtin);
        break;
      default:
        break;
    }

    const q = query.trim().toLowerCase();
    if (!q) return list;
    return list.filter(
      (ext) =>
        ext.title.toLowerCase().includes(q) ||
        ext.id.toLowerCase().includes(q) ||
        ext.keywords.some((k) => k.toLowerCase().includes(q)),
    );
  }, [items, query, chip]);

  const toggle = async (ext: ExtensionInfo) => {
    try {
      await invoke("set_extension_enabled", { id: ext.id, enabled: !ext.enabled });
      setItems((prev) =>
        prev.map((e) =>
          e.id === ext.id ? { ...e, enabled: !e.enabled } : e,
        ),
      );
      setStatus(`${ext.title} ${ext.enabled ? "desativada" : "ativada"}.`);
      setTimeout(() => setStatus(""), 2500);
    } catch (e) {
      setStatus(String(e));
    }
  };

  const openStore = async () => {
    await invoke("open_store");
  };

  const openSettings = async () => {
    await invoke("open_settings");
  };

  const nav = [
    {
      id: "all",
      label: "Instaladas",
      icon: (
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
          <path d="M12 2 2 7l10 5 10-5-10-5Z" />
          <path d="M2 17l10 5 10-5M2 12l10 5 10-5" />
        </svg>
      ),
    },
  ];

  return (
    <SettingsLayout
      title="Extensões"
      subtitle={`${items.length} extensões · ${items.filter((e) => e.enabled).length} ativas`}
      nav={nav}
      active="all"
      onNav={() => {}}
      footer={
        <>
          <button type="button" className="sl-btn sl-btn-primary" onClick={openStore}>
            Loja de extensões
          </button>
          <button type="button" className="sl-btn" onClick={load}>
            Atualizar
          </button>
          <button type="button" className="sl-btn" onClick={openSettings}>
            Configurações
          </button>
          {status && <span className="sl-status">{status}</span>}
        </>
      }
    >
      <div className="ext-toolbar">
        <div className="ext-search">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <circle cx="11" cy="11" r="7" />
            <path d="m20 20-3.5-3.5" />
          </svg>
          <input
            type="search"
            placeholder="Filtrar por nome ou keyword…"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
          />
        </div>
      </div>

      <div className="ext-filter-chips">
        {FILTER_CHIPS.map((f) => (
          <button
            key={f.id}
            type="button"
            className={`ext-chip${chip === f.id ? " active" : ""}`}
            onClick={() => setChip(f.id)}
          >
            {f.label}
          </button>
        ))}
      </div>

      {loading ? (
        <div className="sl-loading">Carregando extensões…</div>
      ) : filtered.length === 0 ? (
        <div className="ext-empty">
          <strong>Nenhuma extensão encontrada</strong>
          <span>Ajuste o filtro ou instale novas extensões na loja.</span>
        </div>
      ) : (
        <div className="ext-grid">
          {filtered.map((ext) => (
            <article
              key={ext.id}
              className={`ext-card${ext.enabled ? "" : " disabled"}`}
            >
              <div className="ext-card-head">
                <div className="ext-icon" aria-hidden>
                  {BUILTIN_ICON[ext.id] ?? "🧩"}
                </div>
                <div>
                  <h3>{ext.title}</h3>
                  <p>
                    {BUILTIN_DESC[ext.id] ??
                      (ext.builtin
                        ? "Extensão integrada ao Spotlight."
                        : "Extensão instalada pelo usuário.")}
                  </p>
                </div>
              </div>

              {ext.keywords.length > 0 && (
                <div className="ext-keywords">
                  {ext.keywords.slice(0, 4).map((kw) => (
                    <span key={kw} className="ext-kw">
                      {kw}
                    </span>
                  ))}
                </div>
              )}

              <div className="ext-card-foot">
                <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
                  <span
                    className={`sl-badge ${ext.builtin ? "sl-badge-builtin" : "sl-badge-user"}`}
                  >
                    {ext.builtin ? "Integrada" : "Usuário"}
                  </span>
                  {!ext.enabled && (
                    <span className="sl-badge sl-badge-off">Desativada</span>
                  )}
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={ext.enabled}
                  aria-label={`${ext.enabled ? "Desativar" : "Ativar"} ${ext.title}`}
                  className={`sl-switch${ext.enabled ? " on" : ""}`}
                  onClick={() => toggle(ext)}
                >
                  <span />
                </button>
              </div>
            </article>
          ))}
        </div>
      )}
    </SettingsLayout>
  );
}
