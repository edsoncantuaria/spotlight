import { useMemo } from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import ClipboardApp from "./ClipboardApp";
import SettingsApp from "./SettingsApp";
import StoreApp from "./StoreApp";
import GuideApp from "./GuideApp";
import ExtensionsApp from "./ExtensionsApp";

function Root() {
  const label = useMemo(() => {
    try {
      return getCurrentWindow().label;
    } catch {
      return "main";
    }
  }, []);

  if (label === "clipboard") return <ClipboardApp />;
  if (label === "settings") return <SettingsApp />;
  if (label === "store") return <StoreApp />;
  if (label === "guide") return <GuideApp />;
  if (label === "extensions") return <ExtensionsApp />;
  return <App />;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <Root />,
);
