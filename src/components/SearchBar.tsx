import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useDebouncedCallbackWithFlush } from "../lib/useDebouncedCallback";

export interface SearchBarHandle {
  focus: () => void;
  getValue: () => string;
  setValue: (value: string) => void;
  clear: () => void;
}

interface SearchBarProps {
  onSearch: (value: string) => void;
  onKeyDown: (e: React.KeyboardEvent) => void;
  onDragStart: () => void;
  onDragEnd: () => void;
  resetKey: number;
  searchDebounceMs?: number;
}

const SearchBar = forwardRef<SearchBarHandle, SearchBarProps>(
  (
    {
      onSearch,
      onKeyDown,
      onDragStart,
      onDragEnd,
      resetKey,
      searchDebounceMs = 300,
    },
    ref,
  ) => {
    const inputRef = useRef<HTMLInputElement>(null);
    const [value, setValue] = useState("");
    const { debounced: debouncedSearch, flush: flushSearch } =
      useDebouncedCallbackWithFlush(onSearch, searchDebounceMs);

    useImperativeHandle(ref, () => ({
      focus: () => inputRef.current?.focus({ preventScroll: true }),
      getValue: () => inputRef.current?.value ?? "",
      setValue: (next: string) => {
        setValue(next);
        flushSearch(next);
      },
      clear: () => {
        setValue("");
        flushSearch("");
      },
    }));

    useEffect(() => {
      setValue("");
      flushSearch("");
      requestAnimationFrame(() => {
        inputRef.current?.focus({ preventScroll: true });
        requestAnimationFrame(() => {
          inputRef.current?.focus({ preventScroll: true });
        });
      });
    }, [resetKey, flushSearch]);

    useEffect(() => {
      const onSetQuery = (e: Event) => {
        const detail = (e as CustomEvent<string>).detail ?? "";
        setValue(detail);
        flushSearch(detail);
        requestAnimationFrame(() =>
          inputRef.current?.focus({ preventScroll: true }),
        );
      };
      window.addEventListener("spotlight-set-query", onSetQuery);
      return () => window.removeEventListener("spotlight-set-query", onSetQuery);
    }, [flushSearch]);

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

    const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
      const next = e.target.value;
      setValue(next);
      debouncedSearch(next);
    };

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
          ref={inputRef}
          type="text"
          className="search-input"
          placeholder="Buscar ou digite > para comandos…"
          value={value}
          onChange={handleChange}
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
