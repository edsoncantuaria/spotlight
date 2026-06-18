import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import SearchBar from "./SearchBar";
import ResultSections from "./ResultSections";
import PreviewPanel from "./PreviewPanel";
import QuickAnswerBar from "./QuickAnswerBar";
import type {
  PreviewData,
  QuickAnswer,
  ResultSection,
  SearchResult,
} from "../types";

interface SpotlightShellProps {
  query: string;
  onQueryChange: (q: string) => void;
  sections: ResultSection[];
  flatResults: SearchResult[];
  quickAnswer: QuickAnswer | null;
  selectedIndex: number;
  preview: PreviewData | null;
  visible: boolean;
  inputRef: React.RefObject<HTMLInputElement | null>;
  onSelect: (result: SearchResult) => void;
  onHover: (index: number) => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  onPreviewAction: (actionId: string) => void;
  onDragStart: () => void;
  onDragEnd: () => void;
  onBackdropClick: () => void;
}

export default function SpotlightShell({
  query,
  onQueryChange,
  sections,
  flatResults,
  quickAnswer,
  selectedIndex,
  preview,
  visible,
  inputRef,
  onSelect,
  onHover,
  onKeyDown,
  onPreviewAction,
  onDragStart,
  onDragEnd,
  onBackdropClick,
}: SpotlightShellProps) {
  const shellRef = useRef<HTMLDivElement>(null);
  const hasPreview = preview !== null;

  const handleOverlayPointerDown = (e: React.PointerEvent) => {
    if (!visible) return;
    if (shellRef.current?.contains(e.target as Node)) return;
    onBackdropClick();
  };

  useEffect(() => {
    if (!visible || !shellRef.current) return;

    let rafId = 0;
    const measure = () => {
      cancelAnimationFrame(rafId);
      rafId = requestAnimationFrame(() => {
        const el = shellRef.current;
        if (!el) return;

        const width = hasPreview ? 900 : 680;
        const height = Math.min(Math.max(el.scrollHeight + 8, 120), 720);

        invoke("resize_window", { width, height }).catch(() => {});
      });
    };

    const observer = new ResizeObserver(measure);
    observer.observe(shellRef.current);
    measure();

    return () => {
      observer.disconnect();
      cancelAnimationFrame(rafId);
    };
  }, [visible, sections, quickAnswer, hasPreview, query]);

  return (
    <div
      className={`overlay ${visible ? "overlay-visible" : ""}`}
      onPointerDown={handleOverlayPointerDown}
    >
      <div
        ref={shellRef}
        className={`spotlight-shell ${hasPreview ? "with-preview" : ""} ${
          visible ? "spotlight-in" : ""
        }`}
      >
        <SearchBar
          ref={inputRef}
          value={query}
          onChange={onQueryChange}
          onKeyDown={onKeyDown}
          onDragStart={onDragStart}
          onDragEnd={onDragEnd}
        />

        {quickAnswer && <QuickAnswerBar answer={quickAnswer} />}

        <div className="spotlight-body">
          <div className="spotlight-results">
            <ResultSections
              sections={sections}
              flatResults={flatResults}
              selectedIndex={selectedIndex}
              onSelect={onSelect}
              onHover={onHover}
            />
          </div>
          {hasPreview && (
            <PreviewPanel preview={preview} onAction={onPreviewAction} />
          )}
        </div>
      </div>
    </div>
  );
}
