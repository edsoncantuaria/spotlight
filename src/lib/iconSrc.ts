import { convertFileSrc } from "@tauri-apps/api/core";
import { resolveImageSrc } from "./imageSrc";

export function isFilePath(value: string): boolean {
  return value.startsWith("/") || value.startsWith("file://");
}

export function resolveIconSrc(icon: string | null | undefined): string | null {
  if (!icon) return null;
  if (isFilePath(icon)) return convertFileSrc(icon);
  return null;
}

export function iconEmoji(kind: string, icon?: string | null): string {
  if (icon && !isFilePath(icon)) {
    const themed: Record<string, string> = {
      "web-browser": "🌐",
      "accessories-calculator": "🔢",
      "edit-copy": "📋",
      "image-x-generic": "🖼️",
      "text-x-generic": "📝",
      "utilities-terminal": "⌨️",
      "preferences-system": "⚙️",
      "contact-new": "👤",
      "docker": "🐳",
      git: "📦",
      "face-smile": "😀",
      window: "🪟",
    };
    if (themed[icon]) return themed[icon];
  }
  switch (kind) {
    case "web":
      return "🌐";
    case "file":
      return "📄";
    case "setting":
      return "⚙️";
    case "app":
      return "▢";
    case "clipboard":
      return "📋";
    case "extension":
      return "🧩";
    default:
      return "▢";
  }
}

export { resolveImageSrc };
