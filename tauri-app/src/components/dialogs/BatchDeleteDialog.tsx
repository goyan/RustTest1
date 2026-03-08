interface BatchDeleteDialogProps {
  selectedPaths: string[];
  onConfirm: () => void;
  onCancel: () => void;
}

function getFileName(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || path;
}

export function BatchDeleteDialog({ selectedPaths, onConfirm, onCancel }: BatchDeleteDialogProps) {
  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50" onClick={onCancel}>
      <div
        className="bg-[#140f1e] border-2 border-cyber-neon-red rounded-xl p-6 min-w-[350px] max-w-[450px] shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="text-lg font-bold text-cyber-neon-red mb-2.5">🗑️ Delete Selected Files?</h2>
        <p className="text-sm text-cyber-light-purple mb-2">
          {selectedPaths.length} items will be permanently deleted.
        </p>

        <div className="bg-[#0c0a12] rounded p-2 max-h-[200px] overflow-y-auto mb-4">
          {selectedPaths.slice(0, 10).map((path) => (
            <div key={path} className="text-[11px] text-[#b4a0dc] py-0.5 truncate">
              📄 {getFileName(path)}
            </div>
          ))}
          {selectedPaths.length > 10 && (
            <div className="text-[11px] text-cyber-dim-purple">
              ... and {selectedPaths.length - 10} more
            </div>
          )}
        </div>

        <div className="flex gap-4">
          <button
            onClick={onCancel}
            className="flex-1 py-2 bg-[#3c3250] hover:bg-[#504670] text-white rounded-lg transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            className="flex-1 py-2 bg-[#b41e3c] hover:bg-[#d4264a] text-white font-bold rounded-lg transition-colors"
          >
            🗑️ Delete All
          </button>
        </div>
      </div>
    </div>
  );
}
