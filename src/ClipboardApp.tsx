import { resolveImageSrc } from "./lib/imageSrc";
import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useScrollSelectedItem } from "./lib/useScrollSelectedItem";
import { scheduleWindowFocus } from "./lib/focusSearch";
import type { ClipboardItem } from "./types";
import "./styles/overlay.css";

type ClipboardFilter = "all" | "text" | "image" | "pinned";

export default function ClipboardApp() {
  const [items, setItems] = useState<ClipboardItem[]>([]);
  const [filter, setFilter] = useState<ClipboardFilter>("all");
  const [stackCount, setStackCount] = useState(0);
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
    const cfg = await invoke<{ clipboard_limit: number }>("get_config");
    const [data, count] = await Promise.all([
      invoke<ClipboardItem[]>("get_clipboard_history", {
        limit: cfg.clipboard_limit,
        filter,
      }),
      invoke<number>("get_clipboard_stack_count"),
    ]);
    setItems(data);
    setStackCount(count);
    setSelectedIndex(0);
  }, [filter]);

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

  const copyItem = useCallback(
    async (index: number) => {
      const item = items[index];
      if (!item) return;
      await invoke("paste_clipboard_item", { id: item.id });
      await hideWindow();
    },
    [items, hideWindow],
  );

  const togglePin = useCallback(
    async (id: string) => {
      await invoke("toggle_clipboard_pin", { id });
      await loadItems();
    },
    [loadItems],
  );

  const addToStack = useCallback(
    async (index: number) => {
      const item = items[index];
      if (!item) return;
      const count = await invoke<number>("add_clipboard_to_stack", { id: item.id });
      setStackCount(count);
    },
    [items],
  );

  const pasteStack = useCallback(async () => {
    await invoke("paste_clipboard_stack");
    await hideWindow();
  }, [hideWindow]);

  useEffect(() => {
    const window = getCurrentWindow();

    const unlistenFocus = window.onFocusChanged(async ({ payload: focused }) => {
      if (focused) {
        if (visible) scheduleFocus();
        return;
      }
      if (suppressBlurRef.current || closing) return;
      if (Date.now() < openingGraceUntilRef.current) return;
      if (!visible) return;
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
    if (visible) loadItems();
  }, [visible, filter, loadItems]);

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
          if (e.shiftKey) {
            addToStack(selectedIndex);
          } else {
            copyItem(selectedIndex);
          }
          break;
        case "p":
          if (e.ctrlKey) {
            e.preventDefault();
            const item = items[selectedIndex];
            if (item) togglePin(item.id);
          }
          break;
      }
      if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === "v") {
        e.preventDefault();
        pasteStack();
      }
    };

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [visible, items, selectedIndex, hideWindow, copyItem, addToStack, togglePin, pasteStack]);

  const beginDrag = (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    if ((e.target as HTMLElement).closest("button, .result-item, .clipboard-filters")) return;
    suppressBlurRef.current = true;
    void getCurrentWindow().startDragging();
  };

  const handleOverlayPointerDown = (e: React.PointerEvent) => {
    if (!visible) return;
    if (shellRef.current?.contains(e.target as Node)) return;
    hideWindow();
  };

  const filters: { id: ClipboardFilter; label: string }[] = [
    { id: "all", label: "Tudo" },
    { id: "text", label: "Texto" },
    { id: "image", label: "Imagens" },
    { id: "pinned", label: "Fixados" },
  ];

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

        <div className="clipboard-filters">
          {filters.map((f) => (
            <button
              key={f.id}
              type="button"
              className={`clipboard-filter-btn ${filter === f.id ? "active" : ""}`}
              onClick={() => setFilter(f.id)}
            >
              {f.label}
            </button>
          ))}
          {stackCount > 0 && (
            <button type="button" className="clipboard-stack-btn" onClick={pasteStack}>
              Colar stack ({stackCount})
            </button>
          )}
        </div>

        <div className="clipboard-list">
          {items.length === 0 ? (
            <div className="no-results">Nenhuma cópia neste filtro</div>
          ) : (
            <ul className="result-list">
              {items.map((item, index) => (
                <li
                  key={item.id}
                  ref={index === selectedIndex ? setSelectedRef : null}
                  className={`result-item ${index === selectedIndex ? "selected" : ""}`}
                  onMouseEnter={() => setSelectedIndex(index)}
                  onClick={() => copyItem(index)}
                >
                  <div className="result-icon result-icon-fallback">
                    {item.preview_image ? (
                      <img
                        src={resolveImageSrc(item.preview_image)}
                        alt=""
                        className="clipboard-thumb"
                      />
                    ) : item.content_type === "image" ? (
                      "🖼️"
                    ) : (
                      "📋"
                    )}
                  </div>
                  <div className="result-text">
                    <span className="result-title">
                      {item.pinned && <span className="pin-badge">📌 </span>}
                      {item.preview}
                    </span>
                    <span className="result-subtitle">{item.subtitle}</span>
                  </div>
                  <button
                    type="button"
                    className="clipboard-action-btn"
                    title="Fixar"
                    onClick={(e) => {
                      e.stopPropagation();
                      togglePin(item.id);
                    }}
                  >
                    {item.pinned ? "📌" : "📍"}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>

        <div className="clipboard-footer">
          <span>Enter colar · Shift+Enter stack · Ctrl+P fixar</span>
          <span>Ctrl+Shift+V colar stack</span>
        </div>
      </div>
    </div>
  );
}
