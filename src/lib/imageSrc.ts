import { convertFileSrc } from "@tauri-apps/api/core";

export function resolveImageSrc(src: string): string {
  if (src.startsWith("data:")) {
    return src;
  }
  return convertFileSrc(src);
}
