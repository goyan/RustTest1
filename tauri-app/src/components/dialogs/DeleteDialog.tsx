import type { FileItem } from "../../types";

interface DeleteDialogProps {
  item: FileItem;
  onConfirm: () => void;
  onCancel: () => void;
}

export function DeleteDialog({ item, onConfirm, onCancel }: DeleteDialogProps) {
  const isProtected = item.category === "MustKeep" || item.category === "System";

  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50" onClick={onCancel}>
      <div
        className="bg-[#14102a] border-2 border-cyber-neon-red rounded-xl p-6 min-w-[350px] max-w-[450px] shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        {isProtected ? (
          <>
            <h2 className="text-lg font-bold text-yellow-400 mb-4">🔒 Protected System Item</h2>
            <p className="text-sm text-gray-200 font-semibold mb-4">{item.name}</p>
            <p className="text-sm text-orange-300 mb-1">This is a protected Windows system item.</p>
            <p className="text-sm text-orange-300 mb-4">Deleting it could damage your system.</p>
            <button
              onClick={onCancel}
              className="w-full py-2 bg-[#281e3c] hover:bg-[#3c2860] rounded-lg transition-colors"
            >
              OK
            </button>
          </>
        ) : (
          <>
            <h2 className="text-lg font-bold text-red-400 mb-4">
              🗑️ Delete {item.is_dir ? "Folder" : "File"}?
            </h2>
            <p className="text-sm text-gray-300 mb-1">Are you sure you want to delete:</p>
            <p className="text-sm font-bold text-[#ffc864] mb-4">{item.name}</p>
            {item.is_dir && (
              <p className="text-sm text-orange-400 mb-4">
                ⚠️ This will delete the folder and ALL its contents!
              </p>
            )}
            <div className="flex gap-4">
              <button
                onClick={onCancel}
                className="flex-1 py-2 bg-[#281e3c] hover:bg-[#3c2860] rounded-lg transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={onConfirm}
                className="flex-1 py-2 bg-[#b43232] hover:bg-[#d43c3c] text-white font-semibold rounded-lg transition-colors"
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
