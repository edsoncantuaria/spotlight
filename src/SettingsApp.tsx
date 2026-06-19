import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import SettingsLayout, { SlCard, SlField, SlToggle } from "./components/SettingsLayout";
import ShortcutField from "./components/ShortcutField";

interface AppConfig {
  shortcuts: string[];
  clipboard_shortcut: string;
  web_search_engine: string;
  theme: string;
  clipboard_limit: number;
  file_roots: string[];
  exclude_patterns: string[];
  max_index_files: number;
  extension_dirs: string[];
  translate_api_url: string | null;
  translate_target: string;
  ai_enabled: boolean;
  ai_model: string;
  ai_ollama_url: string | null;
  ai_api_url: string | null;
  extension_store_url: string | null;
  launch_at_login: boolean;
}

type Section = "general" | "shortcuts" | "search" | "advanced";

const NAV = [
  {
    id: "general" as const,
    label: "Geral",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <circle cx="12" cy="12" r="3" />
        <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42" />
      </svg>
    ),
  },
  {
    id: "shortcuts" as const,
    label: "Atalhos",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <rect x="2" y="6" width="20" height="12" rx="2" />
        <path d="M6 10h.01M10 10h.01M14 10h.01M18 10h.01M8 14h8" />
      </svg>
    ),
  },
  {
    id: "search" as const,
    label: "Busca",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <circle cx="11" cy="11" r="7" />
        <path d="m20 20-3.5-3.5" />
      </svg>
    ),
  },
  {
    id: "advanced" as const,
    label: "Avançado",
    icon: (
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.75">
        <path d="M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z" />
        <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1Z" />
      </svg>
    ),
  },
];

const SECTION_HINT: Record<Section, string> = {
  general: "Inicialização, aparência e tradução",
  shortcuts: "Teclas globais do Spotlight e clipboard",
  search: "Indexação de arquivos e motor web",
  advanced: "IA, loja de extensões e backup",
};

export default function SettingsApp() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [section, setSection] = useState<Section>("general");
  const [status, setStatus] = useState("");
  const [error, setError] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    invoke<AppConfig>("get_config").then(setConfig);
  }, []);

  const patch = useCallback((partial: Partial<AppConfig>) => {
    setConfig((c) => (c ? { ...c, ...partial } : c));
  }, []);

  const save = useCallback(async () => {
    if (!config) return;
    setSaving(true);
    setError(false);
    try {
      const checks = await invoke<
        { valid_format: boolean; available: boolean; message: string }[]
      >("validate_shortcuts", {
        shortcuts: config.shortcuts,
        clipboard_shortcut: config.clipboard_shortcut,
      });

      const blocked = checks.find((c) => !c.valid_format || !c.available);
      if (blocked) {
        setStatus(blocked.message);
        setError(true);
        return;
      }

      await invoke("save_config", { config });
      setStatus("Configurações salvas. Atalhos e autostart atualizados.");
      await invoke("reload_config");
      setTimeout(() => setStatus(""), 3000);
    } catch (e) {
      setStatus(String(e));
      setError(true);
    } finally {
      setSaving(false);
    }
  }, [config]);

  const backup = useCallback(async () => {
    try {
      const path = await invoke<string>("backup_spotlight");
      setStatus(`Backup criado em ${path}`);
      setError(false);
    } catch (e) {
      setStatus(String(e));
      setError(true);
    }
  }, []);

  const openExtensions = useCallback(async () => {
    await invoke("open_extensions");
  }, []);

  const navItems = useMemo(() => NAV.map((n) => ({ ...n })), []);

  if (!config) {
    return <div className="sl-loading">Carregando configurações…</div>;
  }

  return (
    <SettingsLayout
      title="Configurações"
      subtitle={SECTION_HINT[section]}
      nav={navItems}
      active={section}
      onNav={(id) => setSection(id as Section)}
      theme={config.theme}
      footer={
        <>
          <button
            type="button"
            className="sl-btn sl-btn-primary"
            disabled={saving}
            onClick={save}
          >
            {saving ? "Salvando…" : "Salvar alterações"}
          </button>
          <button type="button" className="sl-btn" onClick={backup}>
            Backup
          </button>
          <button type="button" className="sl-btn" onClick={openExtensions}>
            Extensões
          </button>
          {status && (
            <span className={`sl-status${error ? " error" : ""}`}>{status}</span>
          )}
        </>
      }
    >
      {section === "general" && (
        <>
          <SlCard title="Sistema" description="Comportamento ao iniciar o Linux.">
            <SlToggle
              checked={config.launch_at_login ?? true}
              onChange={(v) => patch({ launch_at_login: v })}
              label="Abrir com o sistema"
              description="Inicia automaticamente após instalar o .deb ou ao fazer login. Desmarque para não subir com o sistema."
            />
          </SlCard>

          <SlCard title="Aparência">
            <SlField label="Tema">
              <select
                value={config.theme}
                onChange={(e) => patch({ theme: e.target.value })}
              >
                <option value="auto">Automático (segue o sistema)</option>
                <option value="dark">Escuro</option>
                <option value="light">Claro</option>
              </select>
            </SlField>
          </SlCard>

          <SlCard title="Tradução" description="Extensão de tradução integrada.">
            <div className="sl-field-row">
              <SlField label="Idioma alvo">
                <input
                  value={config.translate_target}
                  onChange={(e) => patch({ translate_target: e.target.value })}
                  placeholder="pt"
                />
              </SlField>
              <SlField label="URL da API (opcional)">
                <input
                  value={config.translate_api_url ?? ""}
                  onChange={(e) =>
                    patch({ translate_api_url: e.target.value || null })
                  }
                  placeholder="https://…"
                />
              </SlField>
            </div>
          </SlCard>
        </>
      )}

      {section === "shortcuts" && (
        <SlCard
          title="Atalhos globais"
          description="Capture a combinação ou digite manualmente. Teste antes de salvar."
        >
          <ShortcutField
            label="Abrir Spotlight"
            hint="Padrão: Ctrl+Alt+Space"
            value={config.shortcuts[0] ?? ""}
            onChange={(v) =>
              patch({
                shortcuts: [v, ...config.shortcuts.slice(1)].filter(Boolean),
              })
            }
          />
          <SlField label="Atalhos alternativos" hint="Separados por vírgula (opcional)">
            <input
              value={config.shortcuts.slice(1).join(", ")}
              onChange={(e) => {
                const rest = e.target.value
                  .split(",")
                  .map((s) => s.trim())
                  .filter(Boolean);
                const primary = config.shortcuts[0];
                patch({
                  shortcuts: primary ? [primary, ...rest] : rest,
                });
              }}
            />
          </SlField>
          <ShortcutField
            label="Histórico de clipboard"
            hint="Padrão: Ctrl+Alt+C"
            value={config.clipboard_shortcut}
            onChange={(v) => patch({ clipboard_shortcut: v })}
          />
        </SlCard>
      )}

      {section === "search" && (
        <>
          <SlCard title="Web e clipboard">
            <SlField label="Motor de busca web">
              <input
                value={config.web_search_engine}
                onChange={(e) => patch({ web_search_engine: e.target.value })}
                placeholder="google"
              />
            </SlField>
            <div className="sl-field-row">
              <SlField label="Limite do clipboard" hint="Entre 10 e 500">
                <input
                  type="number"
                  min={10}
                  max={500}
                  value={config.clipboard_limit}
                  onChange={(e) =>
                    patch({ clipboard_limit: Number(e.target.value) || 50 })
                  }
                />
              </SlField>
              <SlField label="Máx. arquivos indexados">
                <input
                  type="number"
                  value={config.max_index_files}
                  onChange={(e) =>
                    patch({ max_index_files: Number(e.target.value) || 50000 })
                  }
                />
              </SlField>
            </div>
          </SlCard>

          <SlCard
            title="Indexação de arquivos"
            description="Pastas monitoradas e padrões ignorados."
          >
            <SlField label="Pastas para indexar" hint="Uma por linha">
              <textarea
                rows={4}
                value={config.file_roots.join("\n")}
                onChange={(e) =>
                  patch({
                    file_roots: e.target.value
                      .split("\n")
                      .map((s) => s.trim())
                      .filter(Boolean),
                  })
                }
                placeholder="~/Documents&#10;~/Downloads"
              />
            </SlField>
            <SlField label="Pastas excluídas" hint="Uma por linha">
              <textarea
                rows={3}
                value={config.exclude_patterns.join("\n")}
                onChange={(e) =>
                  patch({
                    exclude_patterns: e.target.value
                      .split("\n")
                      .map((s) => s.trim())
                      .filter(Boolean),
                  })
                }
              />
            </SlField>
          </SlCard>
        </>
      )}

      {section === "advanced" && (
        <>
          <SlCard title="Inteligência artificial">
            <SlToggle
              checked={config.ai_enabled}
              onChange={(v) => patch({ ai_enabled: v })}
              label="Extensão de IA"
              description="Requer Ollama ou API compatível configurada."
            />
            <div className="sl-field-row">
              <SlField label="Modelo">
                <input
                  value={config.ai_model}
                  onChange={(e) => patch({ ai_model: e.target.value })}
                />
              </SlField>
              <SlField label="URL Ollama">
                <input
                  value={config.ai_ollama_url ?? ""}
                  onChange={(e) =>
                    patch({ ai_ollama_url: e.target.value || null })
                  }
                  placeholder="http://127.0.0.1:11434"
                />
              </SlField>
            </div>
            <SlField label="URL API alternativa">
              <input
                value={config.ai_api_url ?? ""}
                onChange={(e) => patch({ ai_api_url: e.target.value || null })}
              />
            </SlField>
          </SlCard>

          <SlCard title="Extensões e loja">
            <SlField
              label="URL do catálogo"
              hint="JSON público no GitHub com extensões disponíveis"
            >
              <input
                value={config.extension_store_url ?? ""}
                onChange={(e) =>
                  patch({ extension_store_url: e.target.value || null })
                }
                placeholder="https://raw.githubusercontent.com/…/catalog.json"
              />
            </SlField>
            <SlField label="Pastas adicionais de extensões" hint="Uma por linha">
              <textarea
                rows={2}
                value={config.extension_dirs.join("\n")}
                onChange={(e) =>
                  patch({
                    extension_dirs: e.target.value
                      .split("\n")
                      .map((s) => s.trim())
                      .filter(Boolean),
                  })
                }
              />
            </SlField>
          </SlCard>
        </>
      )}
    </SettingsLayout>
  );
}
