import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface ExtensionInfo {
  id: string;
  title: string;
  enabled: boolean;
  builtin: boolean;
  keywords: string[];
}

export default function ExtensionsApp() {
  const [items, setItems] = useState<ExtensionInfo[]>([]);

  const load = useCallback(async () => {
    const data = await invoke<ExtensionInfo[]>("list_extensions");
    setItems(data);
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  return (
    <div className="settings-page">
      <header className="settings-header">
        <h1>Extensões instaladas</h1>
        <button type="button" onClick={() => getCurrentWindow().close()}>
          Fechar
        </button>
      </header>

      <ul className="settings-ext-list">
        {items.map((ext) => (
          <li key={ext.id}>
            <strong>{ext.title}</strong>{" "}
            <span className="settings-ext-meta">
              {ext.id} · {ext.builtin ? "builtin" : "usuário"} ·{" "}
              {ext.keywords.join(", ") || "sem keywords"}
            </span>
          </li>
        ))}
      </ul>

      <footer className="settings-footer">
        <button type="button" onClick={load}>
          Atualizar
        </button>
      </footer>
    </div>
  );
}
