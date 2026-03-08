interface ToastProps {
  message: string;
}

export function Toast({ message }: ToastProps) {
  return (
    <div className="fixed bottom-12 left-1/2 -translate-x-1/2 z-50 animate-[toast-in_0.3s_ease-out]">
      <div className="px-5 py-3 rounded-lg border border-cyber-cyan bg-[#140a23]/90 text-cyber-cyan text-sm shadow-lg">
        {message}
      </div>
    </div>
  );
}
