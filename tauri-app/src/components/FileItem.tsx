import { useState } from "react";
import type { FileItem, FileCategory } from "../types";

interface FileItemRowProps {
  item: FileItem;
  isSelected: boolean;
  maxSize: number;
  isCalculating: boolean;
  onToggleSelect: () => void;
  onClick: () => void;
  onDelete: () => void;
  onOpenInExplorer: () => void;
}

function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const KB = 1024;
  const MB = 1024 * 1024;
  const GB = 1024 * 1024 * 1024;
  const TB = 1024 * 1024 * 1024 * 1024;
  if (bytes < KB) return `${bytes} B`;
  if (bytes < MB) return `${(bytes / KB).toFixed(1)} KB`;
  if (bytes < GB) return `${(bytes / MB).toFixed(1)} MB`;
  if (bytes < TB) return `${(bytes / GB).toFixed(2)} GB`;
  return `${(bytes / TB).toFixed(2)} TB`;
}

function getCategoryStyle(cat: FileCategory): { text: string; color: string; borderColor: string } {
  switch (cat) {
    case "MustKeep": return { text: "Must Keep", color: "text-cyber-neon-green", borderColor: "border-cyber-neon-green" };
    case "System": return { text: "System", color: "text-purple-400", borderColor: "border-purple-400" };
    case "Regular": return { text: "Regular", color: "text-cyber-electric-blue", borderColor: "border-cyber-electric-blue" };
    case "Useless": return { text: "Useless", color: "text-cyber-neon-red", borderColor: "border-cyber-neon-red" };
    case "Unknown": return { text: "Unknown", color: "text-cyber-dim-purple", borderColor: "border-cyber-dim-purple" };
  }
}

function getUsefulnessColor(value: number): string {
  if (value < 20) return "text-cyber-neon-red";
  if (value < 50) return "text-cyber-neon-orange";
  if (value < 80) return "text-cyber-cyan";
  return "text-cyber-neon-green";
}

export function FileItemRow({
  item,
  isSelected,
  maxSize,
  isCalculating,
  onToggleSelect,
  onClick,
  onDelete,
  onOpenInExplorer,
}: FileItemRowProps) {
  const [showContext, setShowContext] = useState(false);
  const [hovered, setHovered] = useState(false);

  const isEmptyFolder = item.is_dir && item.child_count === 0;
  const sizeRatio = item.size > 0 && maxSize > 0 ? Math.min(item.size / maxSize, 1) : 0;
  const catStyle = getCategoryStyle(item.category);

  const sizeStr = item.is_dir
    ? item.size > 0
      ? formatSize(item.size)
      : isCalculating
        ? "⏳"
        : item.child_count === 0
          ? "Empty"
          : item.child_count !== null
            ? `${item.child_count} items`
            : "—"
    : formatSize(item.size);

  const nameColor = item.is_dir
    ? isEmptyFolder
      ? "text-[#503c64]"
      : "text-cyber-cyan"
    : "text-cyber-light-purple";

  const bgClass = isSelected
    ? "bg-[#3c1450]"
    : isEmptyFolder
      ? "bg-[#0f0c14]"
      : "bg-[#12101a]";

  const hoverBgClass = isSelected
    ? "hover:bg-[#501e64]"
    : isEmptyFolder
      ? "hover:bg-[#191423]"
      : "hover:bg-[#1e1932]";

  const borderClass = isSelected
    ? hovered ? "border-cyber-magenta" : "border-[#b400b4]"
    : hovered
      ? isEmptyFolder ? "border-[#3c2850]" : "border-cyber-cyan"
      : "border-[#281e3c]";

  // Size bar color
  const barColor = sizeRatio > 0.8
    ? "rgba(255, 51, 102, 0.15)"
    : sizeRatio > 0.5
      ? "rgba(255, 136, 0, 0.13)"
      : "rgba(0, 255, 255, 0.12)";

  return (
    <div
      className={`relative flex items-center px-2.5 py-2 rounded-md mb-1 border transition-all cursor-pointer ${bgClass} ${hoverBgClass} ${borderClass}`}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => { setHovered(false); setShowContext(false); }}
      onContextMenu={(e) => {
        e.preventDefault();
        setShowContext(true);
      }}
    >
      {/* Size bar background */}
      {sizeRatio > 0 && (
        <div
          className="absolute inset-y-1 left-1 rounded"
          style={{ width: `${sizeRatio * 100}%`, backgroundColor: barColor }}
        />
      )}

      {/* Content */}
      <div className="relative flex items-center w-full z-10">
        {/* Checkbox */}
        <input
          type="checkbox"
          checked={isSelected}
          onChange={(e) => { e.stopPropagation(); onToggleSelect(); }}
          className="w-4 h-4 mr-2 accent-cyber-magenta cursor-pointer"
        />

        {/* Icon + Name (clickable) */}
        <div className="flex items-center flex-1 min-w-0" onClick={onClick}>
          <span className="text-lg mr-3 flex-shrink-0">{item.icon}</span>
          <span className={`text-[13px] truncate ${nameColor}`}>{item.name}</span>
        </div>

        {/* Size */}
        <div className="w-20 text-center text-[11px] text-gray-400 flex-shrink-0">{sizeStr}</div>

        {/* Category badge */}
        <div className="w-24 flex-shrink-0">
          <span className={`text-[9px] px-1.5 py-0.5 rounded border ${catStyle.color} ${catStyle.borderColor} bg-[#191923]`}>
            {catStyle.text}
          </span>
        </div>

        {/* Usefulness */}
        <div className={`w-16 text-center text-[11px] font-semibold flex-shrink-0 ${getUsefulnessColor(item.usefulness)}`}>
          {item.usefulness.toFixed(0)}%
        </div>
      </div>

      {/* Context menu */}
      {showContext && (
        <div className="absolute right-2 top-full z-50 bg-[#1a1428] border border-cyber-cyan rounded-lg shadow-lg py-1 min-w-[180px]">
          <button
            onClick={() => { onOpenInExplorer(); setShowContext(false); }}
            className="w-full text-left px-3 py-1.5 text-sm hover:bg-[#281e3c] transition-colors"
          >
            📂 Open in Explorer
          </button>
          <button
            onClick={() => {
              navigator.clipboard.writeText(item.path);
              setShowContext(false);
            }}
            className="w-full text-left px-3 py-1.5 text-sm hover:bg-[#281e3c] transition-colors"
          >
            📋 Copy Path
          </button>
          {item.category !== "MustKeep" && item.category !== "System" ? (
            <>
              <div className="border-t border-[#2d3037] my-1" />
              <button
                onClick={() => { onDelete(); setShowContext(false); }}
                className="w-full text-left px-3 py-1.5 text-sm text-red-400 hover:bg-[#281e3c] transition-colors"
              >
                🗑️ Delete {item.is_dir ? "Folder" : "File"}
              </button>
            </>
          ) : (
            <>
              <div className="border-t border-[#2d3037] my-1" />
              <span className="px-3 py-1.5 text-[11px] text-gray-500">🔒 Protected</span>
            </>
          )}
        </div>
      )}
    </div>
  );
}
