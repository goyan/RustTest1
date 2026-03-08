interface BreadcrumbProps {
  path: string;
  onNavigate: (path: string) => void;
}

export function Breadcrumb({ path, onNavigate }: BreadcrumbProps) {
  const parts = path.split(/[\\/]/).filter(Boolean);

  return (
    <div className="flex items-center gap-1 text-sm flex-wrap">
      <span className="text-base text-[#6496ff]">📍</span>
      {parts.map((part, i) => (
        <span key={i} className="flex items-center gap-1">
          {i > 0 && <span className="text-gray-500">/</span>}
          <button
            onClick={() => {
              const newPath = parts.slice(0, i + 1).join("\\");
              onNavigate(newPath.endsWith("\\") ? newPath : newPath + "\\");
            }}
            className="text-[#96c8ff] hover:text-cyber-cyan hover:underline transition-colors"
          >
            {part}
          </button>
        </span>
      ))}
    </div>
  );
}
