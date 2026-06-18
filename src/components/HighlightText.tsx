import type { MatchRange } from "../types";

interface HighlightTextProps {
  text: string;
  ranges: MatchRange[];
}

export default function HighlightText({ text, ranges }: HighlightTextProps) {
  if (ranges.length === 0) {
    return <>{text}</>;
  }

  const sorted = [...ranges].sort((a, b) => a.start - b.start);
  const parts: React.ReactNode[] = [];
  let cursor = 0;

  sorted.forEach((range, i) => {
    if (range.start > cursor) {
      parts.push(<span key={`t-${i}`}>{text.slice(cursor, range.start)}</span>);
    }
    parts.push(
      <span key={`m-${i}`} className="match-highlight">
        {text.slice(range.start, range.end)}
      </span>,
    );
    cursor = range.end;
  });

  if (cursor < text.length) {
    parts.push(<span key="tail">{text.slice(cursor)}</span>);
  }

  return <>{parts}</>;
}
