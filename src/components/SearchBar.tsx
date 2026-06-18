import { forwardRef, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  onDragStart: () => void;
  onDragEnd: () => void;
}

const SearchBar = forwardRef<HTMLInputElement, SearchBarProps>(
  ({ value, onChange, onKeyDown, onDragStart, onDragEnd }, ref) => {
    const beginDrag = (e: React.PointerEvent) => {
      if (e.button !== 0) return;
      const target = e.target as HTMLElement;
      if (target.closest("input, button, a, textarea")) return;
      onDragStart();
      void getCurrentWindow().startDragging();
    };

    useEffect(() => {
      const endDrag = () => onDragEnd();
      window.addEventListener("pointerup", endDrag);
      window.addEventListener("pointercancel", endDrag);
      return () => {
        window.removeEventListener("pointerup", endDrag);
        window.removeEventListener("pointercancel", endDrag);
      };
    }, [onDragEnd]);

    return (
      <div
        className="search-bar"
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
          <circle cx="11" cy="11" r="8" />
          <path d="m21 21-4.35-4.35" />
        </svg>
        <input
          ref={ref}
          type="text"
          className="search-input"
          placeholder="Spotlight Search"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onKeyDown={onKeyDown}
          onPointerDown={(e) => e.stopPropagation()}
          spellCheck={false}
          autoComplete="off"
        />
      </div>
    );
  },
);

SearchBar.displayName = "SearchBar";

export default SearchBar;
