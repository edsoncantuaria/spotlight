import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

export default function GuideApp() {
  const [guide, setGuide] = useState("");

  useEffect(() => {
    invoke<string>("get_extensions_guide").then(setGuide);
  }, []);

  return (
    <div className="settings-page">
      <header className="settings-header">
        <h1>Como criar extensões</h1>
        <button type="button" onClick={() => getCurrentWindow().close()}>
          Fechar
        </button>
      </header>
      <pre className="guide-body">{guide}</pre>
    </div>
  );
}
