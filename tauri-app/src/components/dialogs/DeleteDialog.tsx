import type { FileItem } from "../../types";

interface DeleteDialogProps {
  item: FileItem;
  onConfirm: () => void;
  onCancel: () => void;
}

export function DeleteDialog({ item, onConfirm, onCancel }: DeleteDialogProps) {
  const isProtected = item.category === "MustKeep" || item.category === "System";

  return (
    <div
      className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 animate-backdrop-in"
      onClick={onCancel}
    >
      <div
        className="glass-strong rounded-2xl p-7 min-w-[380px] max-w-[460px] shadow-2xl animate-dialog-in
                   border border-white/[0.06]"
        style={{ boxShadow: isProtected ? "0 0 40px rgba(255, 200, 0, 0.08)" : "0 0 40px rgba(255, 51, 102, 0.1)" }}
        onClick={(e) => e.stopPropagation()}
      >
        {isProtected ? (
          <>
            <div className="flex items-center gap-3 mb-5">
              <div className="w-10 h-10 rounded-xl bg-yellow-500/10 flex items-center justify-center text-lg border border-yellow-500/20">
                &#x1F512;
              </div>
              <h2 className="text-lg font-bold text-yellow-400 tracking-wide">Protected Item</h2>
            </div>
            <p className="text-sm text-gray-200 font-semibold mb-3 bg-white/[0.03] px-3 py-2 rounded-lg">{item.name}</p>
            <p className="text-sm text-orange-300/80 mb-1 leading-relaxed">This is a protected Windows system item.</p>
            <p className="text-sm text-orange-300/80 mb-6 leading-relaxed">Deleting it could damage your system.</p>
            <button
              onClick={onCancel}
              className="w-full py-2.5 btn-glass rounded-xl font-medium transition-all duration-200 hover:bg-white/[0.08] text-sm"
            >
              OK
            </button>
          </>
        ) : (
          <>
            <div className="flex items-center gap-3 mb-5">
              <div className="w-10 h-10 rounded-xl bg-red-500/10 flex items-center justify-center text-lg border border-red-500/20">
                &#x1F5D1;&#xFE0F;
              </div>
              <h2 className="text-lg font-bold text-red-400 tracking-wide">
                Delete {item.is_dir ? "Folder" : "File"}?
              </h2>
            </div>
            <p className="text-sm text-gray-400 mb-2">Are you sure you want to delete:</p>
            <p className="text-sm font-bold text-amber-300 mb-5 bg-white/[0.03] px-3 py-2 rounded-lg truncate">{item.name}</p>
            {item.is_dir && (
              <div className="flex items-center gap-2 mb-5 px-3 py-2 rounded-lg bg-orange-500/[0.06] border border-orange-500/20">
                <span className="text-sm">&#x26A0;&#xFE0F;</span>
                <p className="text-xs text-orange-300/80">
                  This will delete the folder and ALL its contents!
                </p>
              </div>
            )}
            <div className="flex gap-3">
              <button
                onClick={onCancel}
                className="flex-1 py-2.5 btn-glass rounded-xl font-medium transition-all duration-200 hover:bg-white/[0.08] text-sm"
              >
                Cancel
              </button>
              <button
                onClick={onConfirm}
                className="flex-1 py-2.5 bg-red-500/20 hover:bg-red-500/30 text-red-300 font-semibold rounded-xl
                           border border-red-500/30 hover:border-red-500/50 transition-all duration-200 text-sm
                           hover:shadow-[0_0_20px_rgba(255,51,102,0.15)]"
              >
                Delete
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
