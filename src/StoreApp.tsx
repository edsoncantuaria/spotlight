import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface StoreExtension {
  id: string;
  title: string;
  description: string;
  repo: string;
  version?: string;
  builtin?: boolean;
}

export default function StoreApp() {
  const [items, setItems] = useState<StoreExtension[]>([]);
  const [status, setStatus] = useState("");
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const data = await invoke<StoreExtension[]>("list_store_extensions");
      setItems(data);
    } catch (e) {
      setStatus(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const install = async (id: string) => {
    setStatus("Instalando…");
    try {
      const path = await invoke<string>("install_store_extension", { id });
      setStatus(`Instalado em ${path}`);
      await invoke("reload_config");
    } catch (e) {
      setStatus(String(e));
    }
  };

  return (
    <div className="settings-page">
      <header className="settings-header">
        <h1>Loja de Extensões</h1>
        <button type="button" onClick={() => getCurrentWindow().close()}>
          Fechar
        </button>
      </header>

      <p className="settings-hint">
        Catálogo via GitHub. Para publicar, abra PR em{" "}
        <code>docs/extension-store/catalog.json</code>.
      </p>

      {loading ? (
        <p>Carregando catálogo…</p>
      ) : (
        <section className="settings-section store-list">
          {items.map((ext) => (
            <article key={ext.id} className="store-card">
              <h3>{ext.title}</h3>
              <p>{ext.description}</p>
              <div className="store-meta">
                <span>{ext.repo}</span>
                {ext.version && <span>v{ext.version}</span>}
                {ext.builtin && <span>local</span>}
              </div>
              <button type="button" className="primary" onClick={() => install(ext.id)}>
                Instalar
              </button>
            </article>
          ))}
        </section>
      )}

      <footer className="settings-footer">
        <button type="button" onClick={load}>
          Atualizar
        </button>
        {status && <span className="settings-status">{status}</span>}
      </footer>
    </div>
  );
}
