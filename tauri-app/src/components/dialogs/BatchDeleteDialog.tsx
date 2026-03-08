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
    <div
      className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 animate-backdrop-in"
      onClick={onCancel}
    >
      <div
        className="glass-strong rounded-2xl p-7 min-w-[380px] max-w-[460px] shadow-2xl animate-dialog-in
                   border border-white/[0.06]"
        style={{ boxShadow: "0 0 40px rgba(255, 51, 102, 0.1)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-3 mb-4">
          <div className="w-10 h-10 rounded-xl bg-red-500/10 flex items-center justify-center text-lg border border-red-500/20">
            &#x1F5D1;&#xFE0F;
          </div>
          <h2 className="text-lg font-bold text-red-400 tracking-wide">Delete Selected?</h2>
        </div>

        <p className="text-sm text-gray-400 mb-3">
          <span className="text-red-300 font-semibold">{selectedPaths.length}</span> items will be permanently deleted.
        </p>

        {/* File list */}
        <div className="rounded-xl bg-black/20 border border-white/[0.04] p-2.5 max-h-[200px] overflow-y-auto mb-5">
          {selectedPaths.slice(0, 10).map((path) => (
            <div key={path} className="text-[11px] text-gray-400 py-1 px-2 truncate rounded hover:bg-white/[0.02] transition-colors">
              &#x1F4C4; {getFileName(path)}
            </div>
          ))}
          {selectedPaths.length > 10 && (
            <div className="text-[11px] text-gray-600 px-2 py-1 italic">
              ... and {selectedPaths.length - 10} more
            </div>
          )}
        </div>

        <div className="flex gap-3">
          <button
            onClick={onCancel}
            className="flex-1 py-2.5 btn-glass rounded-xl font-medium transition-all duration-200 hover:bg-white/[0.08] text-sm"
          >
            Cancel
          </button>
          <button
            onClick={onConfirm}
            className="flex-1 py-2.5 bg-red-500/20 hover:bg-red-500/30 text-red-300 font-bold rounded-xl
                       border border-red-500/30 hover:border-red-500/50 transition-all duration-200 text-sm
                       hover:shadow-[0_0_20px_rgba(255,51,102,0.15)]"
          >
            &#x1F5D1;&#xFE0F; Delete All
          </button>
        </div>
      </div>
    </div>
  );
}
