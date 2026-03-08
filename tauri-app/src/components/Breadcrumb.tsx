interface BreadcrumbProps {
  path: string;
  onNavigate: (path: string) => void;
}

export function Breadcrumb({ path, onNavigate }: BreadcrumbProps) {
  const parts = path.split(/[\\/]/).filter(Boolean);

  return (
    <div className="flex items-center gap-1.5 text-sm flex-wrap">
      <span className="text-sm opacity-50">&#x1F4CD;</span>
      {parts.map((part, i) => {
        const isLast = i === parts.length - 1;
        return (
          <span key={i} className="flex items-center gap-1.5">
            {i > 0 && (
              <span className="text-gray-600 text-[10px]">&#x276F;</span>
            )}
            <button
              onClick={() => {
                const newPath = parts.slice(0, i + 1).join("\\");
                onNavigate(newPath.endsWith("\\") ? newPath : newPath + "\\");
              }}
              className={`px-2 py-0.5 rounded-md text-[13px] transition-all duration-200
                ${isLast
                  ? "text-cyan-300 bg-cyan-500/10 font-medium"
                  : "text-gray-400 hover:text-cyan-300 hover:bg-white/[0.04]"
                }
              `}
            >
              {part}
            </button>
          </span>
        );
      })}
    </div>
  );
}
