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
  onSelect: (result: SearchResult) => void;
  onHover: (index: number) => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  onPreviewAction: (actionId: string) => void;
  onDragStart: () => void;
  onDragEnd: () => void;
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
  onSelect,
  onHover,
  onKeyDown,
  onPreviewAction,
  onDragStart,
  onDragEnd,
}: SpotlightShellProps) {
  const hasPreview = preview !== null;
  const hasResults = sections.length > 0;
  const expanded = hasResults || quickAnswer !== null || hasPreview;

  if (!visible) return null;

  return (
    <div className="overlay">
      <div
        className={`spotlight-shell ${hasPreview ? "with-preview" : ""} ${
          expanded ? "expanded" : "compact"
        }`}
      >
        <SearchBar
          ref={searchBarRef}
          resetKey={searchResetKey}
          onSearch={onSearch}
          onKeyDown={onKeyDown}
          onDragStart={onDragStart}
          onDragEnd={onDragEnd}
          searchDebounceMs={200}
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
