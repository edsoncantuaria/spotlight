export type ResultKind = "app" | "file" | "setting" | "recent";

export interface ClipboardItem {
  id: string;
  preview: string;
  subtitle: string;
}

export interface MatchRange {
  start: number;
  end: number;
}

export interface SearchResult {
  id: string;
  kind: ResultKind;
  title: string;
  subtitle: string | null;
  icon: string | null;
  score: number;
  match_ranges: MatchRange[];
}

export interface ResultSection {
  id: string;
  title: string;
  results: SearchResult[];
}

export interface QuickAnswer {
  kind: string;
  label: string;
  value: string;
  hint: string | null;
}

export interface SearchResponse {
  quick_answer: QuickAnswer | null;
  sections: ResultSection[];
}

export interface PreviewAction {
  id: string;
  label: string;
}

export interface PreviewData {
  title: string;
  subtitle: string | null;
  description: string | null;
  icon: string | null;
  preview_text: string | null;
  preview_image: string | null;
  actions: PreviewAction[];
}
