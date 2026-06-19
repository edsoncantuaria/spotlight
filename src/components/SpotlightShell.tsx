import SearchBar, { type SearchBarHandle } from "./SearchBar";
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
  searchBarRef: React.RefObject<SearchBarHandle | null>;
  searchResetKey: number;
  onSearch: (q: string) => void;
  sections: ResultSection[];
  flatResults: SearchResult[];
  quickAnswer: QuickAnswer | null;
  selectedIndex: number;
  preview: PreviewData | null;
  visible: boolean;
  closing: boolean;
  onSelect: (result: SearchResult) => void;
  onHover: (index: number) => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  onPreviewAction: (actionId: string) => void;
  onDragStart: () => void;
  onDragEnd: () => void;
  onBackdropClick: () => void;
}

export default function SpotlightShell({
  searchBarRef,
  searchResetKey,
  onSearch,
  sections,
  flatResults,
  quickAnswer,
  selectedIndex,
  preview,
  visible,
  closing,
  onSelect,
  onHover,
  onKeyDown,
  onPreviewAction,
  onDragStart,
  onDragEnd,
  onBackdropClick,
}: SpotlightShellProps) {
  const hasPreview = preview !== null;
  const hasResults = sections.length > 0;
  const expanded = hasResults || quickAnswer !== null || hasPreview;

  if (!visible && !closing) return null;

  const handleOverlayPointerDown = (e: React.PointerEvent) => {
    if (!visible || closing) return;
    if ((e.target as HTMLElement).closest(".spotlight-shell")) return;
    onBackdropClick();
  };

  return (
    <div
      className={`overlay ${visible && !closing ? "overlay-visible" : "overlay-closing"}`}
      onPointerDown={handleOverlayPointerDown}
    >
      <div
        className={`spotlight-shell ${hasPreview ? "with-preview" : ""} ${
          visible && !closing ? "spotlight-in" : ""
        } ${expanded ? "expanded" : "compact"}`}
      >
        <SearchBar
          ref={searchBarRef}
          resetKey={searchResetKey}
          onSearch={onSearch}
          onKeyDown={onKeyDown}
          onDragStart={onDragStart}
          onDragEnd={onDragEnd}
          searchDebounceMs={150}
        />

        {expanded && (
          <div className="spotlight-expandable">
            {quickAnswer && <QuickAnswerBar answer={quickAnswer} />}

            {(hasResults || hasPreview) && (
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
            )}
          </div>
        )}
      </div>
    </div>
  );
}
