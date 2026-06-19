import { useEffect, useRef, useState, useCallback, useMemo } from "react";
import { flushSync } from "react-dom";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import SpotlightShell from "./components/SpotlightShell";
import ActionPalette from "./components/ActionPalette";
import type { SearchBarHandle } from "./components/SearchBar";
import type {
  PreviewData,
  QuickAnswer,
  ResultSection,
  SearchResponse,
  SearchResult,
} from "./types";
import "./styles/overlay.css";
import { scheduleWindowFocus } from "./lib/focusSearch";

function flattenSections(sections: ResultSection[]): SearchResult[] {
  return sections.flatMap((s) => s.results);
}

function App() {
  const [sections, setSections] = useState<ResultSection[]>([]);
  const [quickAnswer, setQuickAnswer] = useState<QuickAnswer | null>(null);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [preview, setPreview] = useState<PreviewData | null>(null);
  const [actionPaletteOpen, setActionPaletteOpen] = useState(false);
  const [theme, setTheme] = useState("auto");
  const [visible, setVisible] = useState(false);
  const [closing, setClosing] = useState(false);
  const [openSession, setOpenSession] = useState(0);

  const searchBarRef = useRef<SearchBarHandle>(null);
  const moveDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const suppressBlurRef = useRef(false);
  const openingGraceUntilRef = useRef(0);
  const previewDebounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const searchGenRef = useRef(0);
  const cancelFocusRetriesRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    invoke<{ theme: string }>("get_config")
      .then((cfg) => setTheme(cfg.theme || "auto"))
      .catch(() => {});
  }, [openSession]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  const scheduleFocus = useCallback(() => {
    cancelFocusRetriesRef.current?.();
    cancelFocusRetriesRef.current = scheduleWindowFocus(() => {
      const input = document.querySelector<HTMLInputElement>(".search-input");
      return input;
    });
  }, []);

  const performSearch = useCallback(async (q: string): Promise<SearchResponse | null> => {
    const gen = ++searchGenRef.current;
    const result = await invoke<SearchResponse>("search", { query: q });
    if (gen !== searchGenRef.current) return null;
    setSections(result.sections);
    setQuickAnswer(result.quick_answer);
    setSelectedIndex(0);
    return result;
  }, []);

  const resetHidden = useCallback(() => {
    cancelFocusRetriesRef.current?.();
    cancelFocusRetriesRef.current = null;
    setClosing(false);
    setVisible(false);
  }, []);

  const openSpotlight = useCallback(() => {
    suppressBlurRef.current = true;
    openingGraceUntilRef.current = Date.now() + 1200;
    flushSync(() => {
      setClosing(false);
      setVisible(true);
      setOpenSession((n) => n + 1);
      setSections([]);
      setQuickAnswer(null);
      setSelectedIndex(0);
      setPreview(null);
    });
    scheduleFocus();
    window.setTimeout(() => scheduleFocus(), 100);
    window.setTimeout(() => scheduleFocus(), 450);
    setTimeout(() => {
      suppressBlurRef.current = false;
    }, 1200);
  }, [scheduleFocus]);

  const flatResults = useMemo(() => flattenSections(sections), [sections]);

  useEffect(() => {
    if (!visible || closing) return;
    scheduleFocus();
    return () => cancelFocusRetriesRef.current?.();
  }, [visible, closing, openSession, scheduleFocus]);

  const hideWindow = useCallback(async () => {
    cancelFocusRetriesRef.current?.();
    cancelFocusRetriesRef.current = null;
    suppressBlurRef.current = true;
    setClosing(true);
    setSections([]);
    setQuickAnswer(null);
    setPreview(null);
    setSelectedIndex(0);
    await new Promise((r) => setTimeout(r, 80));
    await invoke("hide_window");
    setVisible(false);
    setClosing(false);
  }, []);

  const handleDragStart = useCallback(() => {
    suppressBlurRef.current = true;
  }, []);

  const handleDragEnd = useCallback(() => {
    setTimeout(() => {
      suppressBlurRef.current = false;
    }, 300);
  }, []);

  const handleBackdropClick = useCallback(() => {
    if (suppressBlurRef.current || closing) return;
    suppressBlurRef.current = true;
    hideWindow();
  }, [hideWindow, closing]);

  const loadPreview = useCallback(async (result: SearchResult | null) => {
    if (!result) {
      setPreview(null);
      return;
    }
    const data = await invoke<PreviewData | null>("get_preview", {
      id: result.id,
    });
    setPreview(data);
  }, []);

  useEffect(() => {
    const q = searchBarRef.current?.getValue()?.trim() ?? "";
    const result = flatResults[selectedIndex] ?? null;

    if (previewDebounceRef.current) clearTimeout(previewDebounceRef.current);

    if (!q && !result?.id.startsWith("app:")) {
      setPreview(null);
      return;
    }

    previewDebounceRef.current = setTimeout(() => {
      loadPreview(result);
    }, 220);
    return () => {
      if (previewDebounceRef.current) clearTimeout(previewDebounceRef.current);
    };
  }, [selectedIndex, flatResults, loadPreview, openSession]);

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

    const unlistenShown = listen("spotlight-shown", () => {
      openSpotlight();
    });

    const unlistenHidden = listen("spotlight-hidden", () => {
      resetHidden();
    });

    const unlistenMove = window.onMoved(({ payload: position }) => {
      if (moveDebounceRef.current) clearTimeout(moveDebounceRef.current);
      moveDebounceRef.current = setTimeout(() => {
        invoke("save_window_position", { x: position.x, y: position.y });
      }, 150);
    });

    return () => {
      unlistenFocus.then((fn) => fn());
      unlistenShown.then((fn) => fn());
      unlistenHidden.then((fn) => fn());
      unlistenMove.then((fn) => fn());
      if (moveDebounceRef.current) clearTimeout(moveDebounceRef.current);
    };
  }, [openSpotlight, hideWindow, closing, resetHidden, visible, scheduleFocus]);

  const handleOpen = async (result: SearchResult) => {
    const query = searchBarRef.current?.getValue() ?? "";
    await invoke("open_result", { id: result.id, query });
    await hideWindow();
  };

  const handleSubmit = async () => {
    const q = searchBarRef.current?.getValue() ?? "";
    const result = await performSearch(q);
    const answer = result?.quick_answer ?? quickAnswer;

    if (answer) {
      if (answer.value !== "Taxa indisponível" && answer.kind !== "time") {
        await navigator.clipboard.writeText(answer.value);
        await hideWindow();
      }
      return;
    }

    const results = result ? flattenSections(result.sections) : flatResults;
    const selected = results[selectedIndex];
    if (selected?.id.startsWith("recent:quick:")) {
      return;
    }
    if (selected) await handleOpen(selected);
  };

  const handlePreviewAction = async (actionId: string) => {
    const result = flatResults[selectedIndex];
    if (!result) return;

    if (actionId === "copy_path") {
      const text =
        preview?.subtitle && result.kind === "file"
          ? `${preview.subtitle}/${preview.title}`
          : result.id.split(":").slice(1).join(":");
      await navigator.clipboard.writeText(text);
      await hideWindow();
      return;
    }

    await invoke("run_preview_action", {
      id: result.id,
      action: actionId,
      query: searchBarRef.current?.getValue() ?? "",
    });
    if (actionId === "open" || actionId === "copy_url") await hideWindow();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.ctrlKey && e.key.toLowerCase() === "k") {
      e.preventDefault();
      setActionPaletteOpen((open) => !open);
      return;
    }
    if (actionPaletteOpen && e.key === "Escape") {
      e.preventDefault();
      setActionPaletteOpen(false);
      return;
    }
    switch (e.key) {
      case "Escape":
        e.preventDefault();
        hideWindow();
        break;
      case "ArrowDown":
        e.preventDefault();
        setSelectedIndex((i) => Math.min(i + 1, flatResults.length - 1));
        break;
      case "ArrowUp":
        e.preventDefault();
        setSelectedIndex((i) => Math.max(i - 1, 0));
        break;
      case "Tab": {
        e.preventDefault();
        if (sections.length === 0) return;
        const sectionStarts = sections.map((_, idx) =>
          sections.slice(0, idx).reduce((sum, s) => sum + s.results.length, 0),
        );
        const currentSection = sectionStarts.findIndex((start, idx) => {
          const end = start + sections[idx].results.length;
          return selectedIndex >= start && selectedIndex < end;
        });
        const nextSection = (currentSection + 1) % sections.length;
        setSelectedIndex(sectionStarts[nextSection] ?? 0);
        break;
      }
      case "Enter":
        e.preventDefault();
        handleSubmit();
        break;
    }
  };

  return (
    <>
      <SpotlightShell
        searchBarRef={searchBarRef}
        searchResetKey={openSession}
        onSearch={performSearch}
        sections={sections}
        flatResults={flatResults}
        quickAnswer={quickAnswer}
        selectedIndex={selectedIndex}
        preview={preview}
        visible={visible && !closing}
        closing={closing}
        onSelect={handleOpen}
        onHover={setSelectedIndex}
        onKeyDown={handleKeyDown}
        onPreviewAction={handlePreviewAction}
        onDragStart={handleDragStart}
        onDragEnd={handleDragEnd}
        onBackdropClick={handleBackdropClick}
      />
      <ActionPalette
        open={actionPaletteOpen}
        actions={preview?.actions ?? []}
        onSelect={handlePreviewAction}
        onClose={() => setActionPaletteOpen(false)}
      />
    </>
  );
}

export default App;
