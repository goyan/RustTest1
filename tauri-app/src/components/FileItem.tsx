import React, { useState, useRef, useEffect, useLayoutEffect } from "react";
import { createPortal } from "react-dom";
import { invoke } from "@tauri-apps/api/core";
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
  animationDelay?: number;
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

function getCategoryStyle(cat: FileCategory): { text: string; color: string; bg: string; border: string } {
  switch (cat) {
    case "MustKeep": return { text: "Must Keep", color: "text-emerald-400", bg: "bg-emerald-500/10", border: "border-emerald-500/30" };
    case "System": return { text: "System", color: "text-purple-400", bg: "bg-purple-500/10", border: "border-purple-500/30" };
    case "Regular": return { text: "Regular", color: "text-cyan-400", bg: "bg-cyan-500/10", border: "border-cyan-500/30" };
    case "Useless": return { text: "Useless", color: "text-red-400", bg: "bg-red-500/10", border: "border-red-500/30" };
    case "Unknown": return { text: "Unknown", color: "text-gray-500", bg: "bg-gray-500/10", border: "border-gray-500/20" };
  }
}

function getUsefulnessColor(value: number): string {
  if (value < 20) return "text-red-400";
  if (value < 50) return "text-orange-400";
  if (value < 80) return "text-cyan-400";
  return "text-emerald-400";
}

function getUsefulnessGlow(value: number): string {
  if (value < 20) return "rgba(255, 51, 102, 0.15)";
  if (value < 50) return "rgba(255, 136, 0, 0.12)";
  if (value < 80) return "rgba(0, 255, 255, 0.1)";
  return "rgba(0, 255, 136, 0.12)";
}

interface ContextMenuProps {
  x: number;
  y: number;
  item: FileItem;
  onOpenInExplorer: () => void;
  onCopyPath: () => void;
  onDelete: () => void;
  onClose: () => void;
}

interface FileDescription {
  file_type: string;
  details: string[];
  project: string | null;
  safety_tip: string | null;
}

const ContextMenu = React.forwardRef<HTMLDivElement, ContextMenuProps>(
  ({ x, y, item, onOpenInExplorer, onCopyPath, onDelete, onClose }, _ref) => {
    const menuRef = useRef<HTMLDivElement>(null);
    const [pos, setPos] = useState<{ left: number; top: number } | null>(null);
    const [desc, setDesc] = useState<FileDescription | null>(null);

    useEffect(() => {
      invoke<FileDescription>("describe_file", { name: item.name, isDir: item.is_dir, path: item.path })
        .then(setDesc)
        .catch(() => setDesc(null));
    }, [item]);

    // Recalculate position synchronously before paint
    useLayoutEffect(() => {
      if (!menuRef.current) return;
      const rect = menuRef.current.getBoundingClientRect();
      const vw = window.innerWidth;
      const vh = window.innerHeight;
      let left = x;
      let top = y;
      if (x + rect.width > vw - 8) left = vw - rect.width - 8;
      if (y + rect.height > vh - 8) top = y - rect.height;
      if (left < 8) left = 8;
      if (top < 8) top = 8;
      setPos({ left, top });
    }, [x, y, desc]);

    useEffect(() => {
      const handleClickOutside = (e: MouseEvent) => {
        if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
          onClose();
        }
      };
      window.addEventListener("mousedown", handleClickOutside);
      return () => window.removeEventListener("mousedown", handleClickOutside);
    }, [onClose]);

    return (
      <div
        ref={menuRef}
        className="fixed z-[9999] glass-strong rounded-xl shadow-2xl py-1.5 min-w-[190px] border border-white/[0.08] animate-dialog-in"
        style={{
          left: pos ? pos.left : x,
          top: pos ? pos.top : y,
          visibility: pos ? "visible" : "hidden",
        }}
      >
        {desc && (
          <div className="px-4 py-2.5 border-b border-white/[0.06] bg-white/[0.02] max-w-[320px]">
            {/* File type header */}
            <div className="text-[12px] font-semibold text-cyan-300 mb-1.5">{desc.file_type}</div>
            {/* Detail lines */}
            {desc.details.map((line, i) => (
              <div key={i} className="text-[10px] text-gray-400 leading-relaxed truncate" title={line}>
                {line}
              </div>
            ))}
            {/* Project context */}
            {desc.project && (
              <div className="text-[10px] text-purple-400 mt-1">📁 {desc.project}</div>
            )}
            {/* Safety tip */}
            {desc.safety_tip && (
              <div className={`text-[10px] mt-1.5 px-2 py-1 rounded ${
                desc.safety_tip.includes("⚠️") || desc.safety_tip.includes("CRITICAL")
                  ? "bg-red-500/10 text-red-400 border border-red-500/20"
                  : desc.safety_tip.includes("Safe") || desc.safety_tip.includes("safe")
                    ? "bg-emerald-500/10 text-emerald-400 border border-emerald-500/20"
                    : "bg-yellow-500/10 text-yellow-400 border border-yellow-500/20"
              }`}>
                {desc.safety_tip}
              </div>
            )}
          </div>
        )}
        <button
          onClick={onOpenInExplorer}
          className="w-full text-left px-4 py-2 text-xs hover:bg-white/[0.05] transition-colors duration-150 text-gray-300 hover:text-white"
        >
          📂 Open in Explorer
        </button>
        <button
          onClick={onCopyPath}
          className="w-full text-left px-4 py-2 text-xs hover:bg-white/[0.05] transition-colors duration-150 text-gray-300 hover:text-white"
        >
          📋 Copy Path
        </button>
        {item.category !== "MustKeep" && item.category !== "System" ? (
          <>
            <div className="my-1 h-px bg-white/[0.06] mx-3" />
            <button
              onClick={onDelete}
              className="w-full text-left px-4 py-2 text-xs text-red-400 hover:bg-red-500/10 transition-colors duration-150"
            >
              🗑️ Delete {item.is_dir ? "Folder" : "File"}
            </button>
          </>
        ) : (
          <>
            <div className="my-1 h-px bg-white/[0.06] mx-3" />
            <span className="px-4 py-2 text-[10px] text-gray-600 block">🔒 Protected</span>
          </>
        )}
      </div>
    );
  }
);

function FileItemRowInner({
  item,
  isSelected,
  maxSize,
  isCalculating,
  onToggleSelect,
  onClick,
  onDelete,
  onOpenInExplorer,
  animationDelay = 0,
}: FileItemRowProps) {
  const [showContext, setShowContext] = useState(false);
  const [contextPos, setContextPos] = useState<{ x: number; y: number } | null>(null);
  const [hovered, setHovered] = useState(false);
  const contextRef = useRef<HTMLDivElement>(null);

  const isEmptyFolder = item.is_dir && item.child_count === 0;
  const sizeRatio = item.size > 0 && maxSize > 0 ? Math.min(item.size / maxSize, 1) : 0;
  const catStyle = getCategoryStyle(item.category);

  const sizeStr = item.is_dir
    ? item.size > 0
      ? formatSize(item.size)
      : isCalculating
        ? "\u23F3"
        : item.child_count === 0
          ? "Empty"
          : item.child_count !== null
            ? `${item.child_count} items`
            : "\u2014"
    : formatSize(item.size);

  const nameColor = item.is_dir
    ? isEmptyFolder
      ? "text-gray-600"
      : "text-cyan-300"
    : "text-gray-200";

  // Size bar gradient color
  const barGradient = sizeRatio > 0.8
    ? "linear-gradient(90deg, rgba(255, 51, 102, 0.06), rgba(255, 51, 102, 0.12))"
    : sizeRatio > 0.5
      ? "linear-gradient(90deg, rgba(255, 136, 0, 0.05), rgba(255, 136, 0, 0.1))"
      : "linear-gradient(90deg, rgba(0, 255, 255, 0.04), rgba(0, 255, 255, 0.08))";

  return (
    <div
      className={`relative flex items-center px-3 py-2 rounded-lg mb-1 border transition-all duration-200 cursor-pointer group
        ${isSelected
          ? "glass border-magenta-400/30 bg-purple-500/[0.08]"
          : isEmptyFolder
            ? "border-transparent bg-transparent hover:bg-white/[0.02]"
            : "border-transparent bg-transparent hover:bg-white/[0.03]"
        }
        ${isSelected && hovered ? "glow-magenta border-[rgba(255,0,255,0.35)]" : ""}
        ${!isSelected && hovered && !isEmptyFolder ? "border-cyan-500/20" : ""}
      `}
      style={{
        animation: `fade-in 0.3s ease-out ${animationDelay}s forwards`,
        opacity: 0,
        borderColor: isSelected
          ? hovered ? "rgba(255, 0, 255, 0.35)" : "rgba(255, 0, 255, 0.2)"
          : hovered && !isEmptyFolder
            ? "rgba(0, 255, 255, 0.15)"
            : "transparent",
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => { setHovered(false); setShowContext(false); }}
      onContextMenu={(e) => {
        e.preventDefault();
        setContextPos({ x: e.clientX, y: e.clientY });
        setShowContext(true);
      }}
    >
      {/* Size bar background - gradient overlay */}
      {sizeRatio > 0 && (
        <div
          className="absolute inset-y-0.5 left-0.5 rounded-lg transition-all duration-300"
          style={{
            width: `${sizeRatio * 100}%`,
            background: barGradient,
          }}
        />
      )}

      {/* Content */}
      <div className="relative flex items-center w-full z-10">
        {/* Checkbox */}
        <input
          type="checkbox"
          checked={isSelected}
          onChange={(e) => { e.stopPropagation(); onToggleSelect(); }}
          className="w-3.5 h-3.5 mr-2.5 accent-cyber-magenta cursor-pointer rounded"
        />

        {/* Icon + Name (clickable) */}
        <div className="flex items-center flex-1 min-w-0" onClick={onClick}>
          <span className="text-base mr-2.5 flex-shrink-0 opacity-80 group-hover:opacity-100 transition-opacity">
            {item.icon}
          </span>
          <span className={`text-[13px] truncate font-medium ${nameColor} group-hover:text-white transition-colors duration-200`}>
            {item.name}
          </span>
        </div>

        {/* Size */}
        <div className="w-20 text-center text-[11px] text-gray-500 flex-shrink-0 font-mono">
          {isCalculating ? (
            <span className="inline-block animate-pulse">{sizeStr}</span>
          ) : sizeStr}
        </div>

        {/* Category badge */}
        <div className="w-24 flex-shrink-0">
          <span className={`text-[9px] font-medium px-2 py-0.5 rounded-full border ${catStyle.color} ${catStyle.bg} ${catStyle.border}`}>
            {catStyle.text}
          </span>
        </div>

        {/* Usefulness */}
        <div
          className={`w-16 text-center text-[11px] font-bold flex-shrink-0 ${getUsefulnessColor(item.usefulness)}`}
          style={{ textShadow: `0 0 8px ${getUsefulnessGlow(item.usefulness)}` }}
        >
          {item.usefulness.toFixed(0)}%
        </div>
      </div>

      {/* Context menu - rendered via portal to escape transform/filter containing blocks */}
      {showContext && contextPos && createPortal(
        <ContextMenu
          ref={contextRef}
          x={contextPos.x}
          y={contextPos.y}
          item={item}
          onOpenInExplorer={() => { onOpenInExplorer(); setShowContext(false); }}
          onCopyPath={() => { navigator.clipboard.writeText(item.path); setShowContext(false); }}
          onDelete={() => { onDelete(); setShowContext(false); }}
          onClose={() => setShowContext(false)}
        />,
        document.body
      )}
    </div>
  );
}

export const FileItemRow = React.memo(FileItemRowInner);
