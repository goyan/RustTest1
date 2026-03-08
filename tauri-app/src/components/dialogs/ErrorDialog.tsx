interface ErrorDialogProps {
  message: string;
  onClose: () => void;
}

export function ErrorDialog({ message, onClose }: ErrorDialogProps) {
  return (
    <div
      className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 animate-backdrop-in"
      onClick={onClose}
    >
      <div
        className="glass-strong rounded-2xl p-7 min-w-[340px] max-w-[420px] shadow-2xl animate-dialog-in
                   border border-white/[0.06]"
        style={{ boxShadow: "0 0 40px rgba(255, 51, 102, 0.08)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center gap-3 mb-4">
          <div className="w-10 h-10 rounded-xl bg-red-500/10 flex items-center justify-center text-lg border border-red-500/20">
            &#x2716;
          </div>
          <h2 className="text-base font-bold text-red-400 tracking-wide">Operation Failed</h2>
        </div>
        <p className="text-sm text-gray-300 mb-6 leading-relaxed bg-white/[0.02] px-3 py-2 rounded-lg">{message}</p>
        <button
          onClick={onClose}
          className="w-full py-2.5 btn-glass rounded-xl font-medium transition-all duration-200 hover:bg-white/[0.08] text-sm"
        >
          OK
        </button>
      </div>
    </div>
  );
}
