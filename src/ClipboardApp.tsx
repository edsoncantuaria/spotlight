import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useScrollSelectedItem } from "./lib/useScrollSelectedItem";
import { scheduleWindowFocus } from "./lib/focusSearch";
import type { ClipboardItem } from "./types";
import "./styles/overlay.css";

export default function ClipboardApp() {
  const [items, setItems] = useState<ClipboardItem[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [visible, setVisible] = useState(false);
  const [closing, setClosing] = useState(false);
  const [openSession, setOpenSession] = useState(0);

  const shellRef = useRef<HTMLDivElement>(null);
  const suppressBlurRef = useRef(false);
  const openingGraceUntilRef = useRef(0);
  const cancelFocusRetriesRef = useRef<(() => void) | null>(null);
  const setSelectedRef = useScrollSelectedItem<HTMLLIElement>(selectedIndex);

  const scheduleFocus = useCallback(() => {
    cancelFocusRetriesRef.current?.();
    cancelFocusRetriesRef.current = scheduleWindowFocus(() => shellRef.current);
  }, []);

  const loadItems = useCallback(async () => {
    const data = await invoke<ClipboardItem[]>("get_clipboard_history", {
      limit: 10,
    });
    setItems(data);
    setSelectedIndex(0);
  }, []);

  const resetHidden = useCallback(() => {
    cancelFocusRetriesRef.current?.();
    cancelFocusRetriesRef.current = null;
    setClosing(false);
    setVisible(false);
  }, []);

  const hideWindow = useCallback(async () => {
    cancelFocusRetriesRef.current?.();
    cancelFocusRetriesRef.current = null;
    setClosing(true);
    await new Promise((r) => setTimeout(r, 120));
    await invoke("hide_clipboard_window");
    setVisible(false);
    setClosing(false);
  }, []);

  const openClipboard = useCallback(() => {
    suppressBlurRef.current = true;
    openingGraceUntilRef.current = Date.now() + 1200;
    setClosing(false);
    setVisible(true);
    setOpenSession((n) => n + 1);
    setSelectedIndex(0);
    loadItems();
    scheduleFocus();
    setTimeout(() => {
      suppressBlurRef.current = false;
    }, 1200);
  }, [loadItems, scheduleFocus]);

  useEffect(() => {
    if (!visible || closing) return;
    scheduleFocus();
    return () => cancelFocusRetriesRef.current?.();
  }, [visible, closing, openSession, scheduleFocus]);

  const selectItem = useCallback(
    async (index: number) => {
      const item = items[index];
      if (!item) return;
      await invoke("copy_clipboard_item", { id: item.id });
      await hideWindow();
    },
    [items, hideWindow],
  );

  useEffect(() => {
    const window = getCurrentWindow();

    const unlistenFocus = window.onFocusChanged(async ({ payload: focused }) => {
      if (focused) {
        if (visible) scheduleFocus();
        return;
      }
      if (suppressBlurRef.current || closing) return;
      if (Date.now() < openingGraceUntilRef.current) return;
      const isVisible = await window.isVisible();
      if (!isVisible) {
        resetHidden();
        return;
      }
      hideWindow();
    });

    const unlistenShown = listen("clipboard-shown", () => {
      openClipboard();
    });

    const unlistenHidden = listen("clipboard-hidden", () => {
      resetHidden();
    });

    return () => {
      unlistenFocus.then((fn) => fn());
      unlistenShown.then((fn) => fn());
      unlistenHidden.then((fn) => fn());
    };
  }, [openClipboard, hideWindow, closing, resetHidden, visible, scheduleFocus]);

  useEffect(() => {
    if (!visible || !shellRef.current) return;

    let rafId = 0;
    const measure = () => {
      cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(() => {
        const el = shellRef.current;
        if (!el) return;
        const height = Math.min(Math.max(el.scrollHeight + 8, 160), 620);
        invoke("resize_window", { width: 680, height }).catch(() => {});
      });
    };

    const observer = new ResizeObserver(measure);
    observer.observe(shellRef.current);
    measure();

    return () => {
      observer.disconnect();
      cancelAnimationFrame(rafId);
    };
  }, [visible, items]);

  useEffect(() => {
    if (!visible) return;

    const onKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case "Escape":
          e.preventDefault();
          hideWindow();
          break;
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((i) => Math.min(i + 1, items.length - 1));
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((i) => Math.max(i - 1, 0));
          break;
        case "Enter":
          e.preventDefault();
          selectItem(selectedIndex);
          break;
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [visible, items, selectedIndex, hideWindow, selectItem]);

  const beginDrag = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest("button, .result-item")) return;
    suppressBlurRef.current = true;
    void getCurrentWindow().startDragging();
  };

  const handleOverlayPointerDown = (e: React.PointerEvent) => {
    if (!visible) return;
    if (shellRef.current?.contains(e.target as Node)) return;
    hideWindow();
  };

  return (
    <div
      className={`overlay ${visible && !closing ? "overlay-visible" : ""}`}
      onPointerDown={handleOverlayPointerDown}
    >
      <div
        ref={shellRef}
        tabIndex={-1}
        data-focus-root
        className={`spotlight-shell ${visible && !closing ? "spotlight-in" : ""}`}
      >
        <div
          className="search-bar clipboard-header"
          data-tauri-drag-region
          onPointerDown={beginDrag}
        >
          <div className="drag-handle" data-tauri-drag-region title="Arrastar">
            <span />
            <span />
            <span />
          </div>
          <svg
            className="search-icon"
            data-tauri-drag-region
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <rect x="8" y="2" width="8" height="4" rx="1" />
            <path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" />
          </svg>
          <span className="clipboard-title" data-tauri-drag-region>
            Área de transferência
          </span>
        </div>

        <div className="clipboard-list">
          {items.length === 0 ? (
            <div className="no-results">Nenhuma cópia recente</div>
          ) : (
            <ul className="result-list">
              {items.map((item, index) => (
                <li
                  key={item.id}
                  ref={index === selectedIndex ? setSelectedRef : null}
                  className={`result-item ${index === selectedIndex ? "selected" : ""}`}
                  onMouseEnter={() => setSelectedIndex(index)}
                  onClick={() => selectItem(index)}
                >
                  <div className="result-icon result-icon-fallback">📋</div>
                  <div className="result-text">
                    <span className="result-title">{item.preview}</span>
                    <span className="result-subtitle">{item.subtitle}</span>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>

        <div className="clipboard-footer">
          <span>↑↓ navegar</span>
          <span>Enter copiar</span>
          <span>Esc fechar</span>
        </div>
      </div>
    </div>
  );
}
