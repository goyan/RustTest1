interface ToastProps {
  message: string;
}

export function Toast({ message }: ToastProps) {
  return (
    <div className="fixed bottom-8 left-1/2 -translate-x-1/2 z-50"
      style={{ animation: "toast-in 0.35s cubic-bezier(0.16, 1, 0.3, 1)" }}
    >
      <div className="px-6 py-3 rounded-xl glass-strong border border-cyan-500/20 glow-cyan-soft
                      text-cyan-300 text-sm font-medium shadow-2xl tracking-wide">
        {message}
      </div>
    </div>
  );
}
