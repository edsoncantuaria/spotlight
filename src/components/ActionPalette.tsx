import type { PreviewAction } from "../types";

interface ActionPaletteProps {
  open: boolean;
  actions: PreviewAction[];
  onSelect: (actionId: string) => void;
  onClose: () => void;
}

export default function ActionPalette({
  open,
  actions,
  onSelect,
  onClose,
}: ActionPaletteProps) {
  if (!open || actions.length === 0) return null;

  return (
    <div className="action-palette-backdrop" onClick={onClose}>
      <div
        className="action-palette"
        onClick={(e) => e.stopPropagation()}
        role="menu"
      >
        <div className="action-palette-title">Ações (Ctrl+K)</div>
        {actions.map((action) => (
          <button
            key={action.id}
            type="button"
            className="action-palette-item"
            onClick={() => {
              onSelect(action.id);
              onClose();
            }}
          >
            {action.label}
          </button>
        ))}
      </div>
    </div>
  );
}
