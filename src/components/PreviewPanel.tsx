import { convertFileSrc } from "@tauri-apps/api/core";
import type { PreviewData } from "../types";

interface PreviewPanelProps {
  preview: PreviewData | null;
  onAction: (actionId: string) => void;
}

export default function PreviewPanel({ preview, onAction }: PreviewPanelProps) {
  if (!preview) {
    return null;
  }

  return (
    <aside className="preview-panel">
      {preview.preview_image ? (
        <img
          className="preview-image"
          src={convertFileSrc(preview.preview_image)}
          alt=""
        />
      ) : preview.icon ? (
        <img
          className="preview-icon-large"
          src={convertFileSrc(preview.icon)}
          alt=""
        />
      ) : null}

      <h2 className="preview-title">{preview.title}</h2>
      {preview.subtitle && (
        <p className="preview-subtitle">{preview.subtitle}</p>
      )}
      {preview.description && (
        <p className="preview-description">{preview.description}</p>
      )}
      {preview.preview_text && (
        <pre className="preview-text">{preview.preview_text}</pre>
      )}

      <div className="preview-actions">
        {preview.actions.map((action) => (
          <button
            key={action.id}
            type="button"
            className="preview-action-btn"
            onClick={() => onAction(action.id)}
          >
            {action.label}
          </button>
        ))}
      </div>
    </aside>
  );
}
