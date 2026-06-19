import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

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

export default function SettingsApp() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [status, setStatus] = useState("");
  const [extensions, setExtensions] = useState<
    { id: string; title: string; enabled: boolean; builtin: boolean }[]
  >([]);

  useEffect(() => {
    invoke<AppConfig>("get_config").then(setConfig);
    invoke<typeof extensions>("list_extensions").then(setExtensions);
  }, []);

  const save = useCallback(async () => {
    if (!config) return;
    await invoke("save_config", { config });
    setStatus("Salvo! Atalhos e config recarregados.");
    await invoke("reload_config");
    setTimeout(() => setStatus(""), 2500);
  }, [config]);

  const backup = useCallback(async () => {
    const path = await invoke<string>("backup_spotlight");
    setStatus(`Backup em ${path}`);
  }, []);

  if (!config) {
    return <div className="settings-page">Carregando…</div>;
  }

  return (
    <div className="settings-page" data-theme={config.theme}>
      <header className="settings-header">
        <h1>Spotlight — Configurações</h1>
        <button type="button" onClick={() => getCurrentWindow().close()}>
          Fechar
        </button>
      </header>

      <section className="settings-section">
        <h2>Sistema</h2>
        <label className="settings-checkbox">
          <input
            type="checkbox"
            checked={config.launch_at_login ?? true}
            onChange={(e) =>
              setConfig({ ...config, launch_at_login: e.target.checked })
            }
          />
          Abrir com o sistema
        </label>
      </section>

      <section className="settings-section">
        <h2>Aparência</h2>
        <label>
          Tema
          <select
            value={config.theme}
            onChange={(e) => setConfig({ ...config, theme: e.target.value })}
          >
            <option value="auto">Automático</option>
            <option value="dark">Escuro</option>
            <option value="light">Claro</option>
          </select>
        </label>
      </section>

      <section className="settings-section">
        <h2>Busca</h2>
        <label>
          Motor de busca web
          <input
            value={config.web_search_engine}
            onChange={(e) =>
              setConfig({ ...config, web_search_engine: e.target.value })
            }
          />
        </label>
        <label>
          Limite do clipboard (10–500)
          <input
            type="number"
            min={10}
            max={500}
            value={config.clipboard_limit}
            onChange={(e) =>
              setConfig({
                ...config,
                clipboard_limit: Number(e.target.value) || 50,
              })
            }
          />
        </label>
        <label>
          Máx. arquivos indexados
          <input
            type="number"
            value={config.max_index_files}
            onChange={(e) =>
              setConfig({
                ...config,
                max_index_files: Number(e.target.value) || 50000,
              })
            }
          />
        </label>
        <label>
          Pastas (file_roots, uma por linha)
          <textarea
            rows={3}
            value={config.file_roots.join("\n")}
            onChange={(e) =>
              setConfig({
                ...config,
                file_roots: e.target.value
                  .split("\n")
                  .map((s) => s.trim())
                  .filter(Boolean),
              })
            }
          />
        </label>
        <label>
          Excluir pastas (exclude_patterns, uma por linha)
          <textarea
            rows={3}
            value={config.exclude_patterns.join("\n")}
            onChange={(e) =>
              setConfig({
                ...config,
                exclude_patterns: e.target.value
                  .split("\n")
                  .map((s) => s.trim())
                  .filter(Boolean),
              })
            }
          />
        </label>
        <label>
          Diretórios de extensões (extension_dirs)
          <textarea
            rows={2}
            value={config.extension_dirs.join("\n")}
            onChange={(e) =>
              setConfig({
                ...config,
                extension_dirs: e.target.value
                  .split("\n")
                  .map((s) => s.trim())
                  .filter(Boolean),
              })
            }
          />
        </label>
      </section>

      <section className="settings-section">
        <h2>Atalhos</h2>
        <label>
          Spotlight (separados por vírgula)
          <input
            value={config.shortcuts.join(", ")}
            onChange={(e) =>
              setConfig({
                ...config,
                shortcuts: e.target.value.split(",").map((s) => s.trim()),
              })
            }
          />
        </label>
        <label>
          Clipboard
          <input
            value={config.clipboard_shortcut}
            onChange={(e) =>
              setConfig({ ...config, clipboard_shortcut: e.target.value })
            }
          />
        </label>
      </section>

      <section className="settings-section">
        <h2>Tradução</h2>
        <label>
          Idioma alvo
          <input
            value={config.translate_target}
            onChange={(e) =>
              setConfig({ ...config, translate_target: e.target.value })
            }
          />
        </label>
        <label>
          URL da API de tradução (opcional)
          <input
            value={config.translate_api_url ?? ""}
            onChange={(e) =>
              setConfig({
                ...config,
                translate_api_url: e.target.value || null,
              })
            }
          />
        </label>
      </section>

      <section className="settings-section">
        <h2>Loja de extensões</h2>
        <label>
          URL do catálogo (JSON GitHub)
          <input
            value={config.extension_store_url ?? ""}
            placeholder="https://raw.githubusercontent.com/.../catalog.json"
            onChange={(e) =>
              setConfig({
                ...config,
                extension_store_url: e.target.value || null,
              })
            }
          />
        </label>
      </section>

      <section className="settings-section">
        <h2>Extensões instaladas</h2>
        <ul className="settings-ext-list">
          {extensions.map((ext) => (
            <li key={ext.id}>
              <strong>{ext.title}</strong>{" "}
              <span className="settings-ext-meta">
                {ext.builtin ? "builtin" : "usuário"} · {ext.enabled ? "ativa" : "off"}
              </span>
            </li>
          ))}
        </ul>
        <p className="settings-hint">
          Extensões de usuário: <code>~/.config/spotlight/extensions/&lt;id&gt;/manifest.json</code>
        </p>
      </section>

      <footer className="settings-footer">
        <button type="button" className="primary" onClick={save}>
          Salvar
        </button>
        <button type="button" onClick={backup}>
          Backup config
        </button>
        {status && <span className="settings-status">{status}</span>}
      </footer>
    </div>
  );
}
