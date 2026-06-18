import { StrictMode, useMemo } from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import ClipboardApp from "./ClipboardApp";

function Root() {
  const label = useMemo(() => {
    try {
      return getCurrentWindow().label;
    } catch {
      return "main";
    }
  }, []);

  return label === "clipboard" ? <ClipboardApp /> : <App />;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <StrictMode>
    <Root />
  </StrictMode>,
);
