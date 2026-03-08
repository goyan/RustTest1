interface ErrorDialogProps {
  message: string;
  onClose: () => void;
}

export function ErrorDialog({ message, onClose }: ErrorDialogProps) {
  return (
    <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50" onClick={onClose}>
      <div
        className="bg-[#14102a] border border-red-500 rounded-xl p-6 min-w-[300px] max-w-[400px] shadow-2xl"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="text-base font-bold text-red-400 mb-2.5">❌ Deletion Failed</h2>
        <p className="text-sm text-gray-300 mb-4">{message}</p>
        <button
          onClick={onClose}
          className="w-full py-2 bg-[#281e3c] hover:bg-[#3c2860] rounded-lg transition-colors"
        >
          OK
        </button>
      </div>
    </div>
  );
}
