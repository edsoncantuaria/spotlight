import type { RefObject } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getCurrentWebview } from "@tauri-apps/api/webview";

const FOCUS_DELAYS = [0, 16, 50, 100, 200, 400, 600, 800, 1000, 1200];

export async function focusWindow(element?: HTMLElement | null): Promise<void> {
  try {
    await getCurrentWindow().setFocus();
  } catch {
    // window manager may ignore focus requests
  }
  try {
    await getCurrentWebview().setFocus();
  } catch {
    // webview focus may fail on some platforms
  }
  element?.focus({ preventScroll: true });
}

export function scheduleWindowFocus(
  getElement?: () => HTMLElement | null,
  delays = FOCUS_DELAYS,
): () => void {
  const timers: number[] = [];
  let alive = true;

  const attempt = () => {
    if (!alive) return;
    void focusWindow(getElement?.() ?? null);
  };

  attempt();
  requestAnimationFrame(() => {
    attempt();
    requestAnimationFrame(attempt);
  });

  for (const delay of delays) {
    timers.push(window.setTimeout(attempt, delay));
  }

  return () => {
    alive = false;
    timers.forEach((id) => clearTimeout(id));
  };
}

export function scheduleFocusRetries(
  inputRef: RefObject<HTMLInputElement | null>,
  delays = FOCUS_DELAYS,
): () => void {
  return scheduleWindowFocus(() => inputRef.current, delays);
}

export function focusRootElement(): void {
  const root =
    document.querySelector<HTMLElement>("[data-focus-root]") ??
    document.querySelector<HTMLElement>(".search-input");
  void focusWindow(root);
}
