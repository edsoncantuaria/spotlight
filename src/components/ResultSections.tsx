import { convertFileSrc } from "@tauri-apps/api/core";
import HighlightText from "./HighlightText";
import { useScrollSelectedItem } from "../lib/useScrollSelectedItem";
import type { ResultSection, SearchResult } from "../types";

interface ResultSectionsProps {
  sections: ResultSection[];
  flatResults: SearchResult[];
  selectedIndex: number;
  onSelect: (result: SearchResult) => void;
  onHover: (index: number) => void;
}

function ResultIcon({ result }: { result: SearchResult }) {
  if (result.icon && result.kind !== "file") {
    return (
      <img
        className="result-icon"
        src={convertFileSrc(result.icon)}
        alt=""
        onError={(e) => {
          (e.target as HTMLImageElement).style.display = "none";
        }}
      />
    );
  }

  return (
    <div className="result-icon result-icon-fallback">
      {result.kind === "file" ? "📄" : result.kind === "setting" ? "⚙️" : "▢"}
    </div>
  );
}

export default function ResultSections({
  sections,
  flatResults,
  selectedIndex,
  onSelect,
  onHover,
}: ResultSectionsProps) {
  const setSelectedRef = useScrollSelectedItem<HTMLLIElement>(selectedIndex);

  if (sections.length === 0) {
    return null;
  }

  let globalIndex = 0;

  return (
    <div className="result-sections">
      {sections.map((section) => (
        <div key={section.id} className="result-section">
          <div className="section-header">{section.title}</div>
          <ul className="result-list">
            {section.results.map((result) => {
              const index = globalIndex++;
              const isSelected = index === selectedIndex;
              return (
                <li
                  key={result.id}
                  ref={isSelected ? setSelectedRef : null}
                  className={`result-item ${isSelected ? "selected" : ""}`}
                  onMouseEnter={() => onHover(index)}
                  onClick={() => onSelect(result)}
                >
                  <ResultIcon result={result} />
                  <div className="result-text">
                    <span className="result-title">
                      <HighlightText text={result.title} ranges={result.match_ranges} />
                    </span>
                    {result.subtitle && (
                      <span className="result-subtitle">{result.subtitle}</span>
                    )}
                  </div>
                </li>
              );
            })}
          </ul>
        </div>
      ))}
      {flatResults.length === 0 && (
        <div className="no-results">Nenhum resultado</div>
      )}
    </div>
  );
}
