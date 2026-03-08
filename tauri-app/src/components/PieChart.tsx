import { useMemo } from "react";
import type { DiskInfo } from "../types";

interface PieChartProps {
  disks: DiskInfo[];
  totalSpace: number;
  totalUsed: number;
  avgUsage: number;
}

function getUsageHexColor(percent: number): string {
  if (percent > 90) return "#ff3366";
  if (percent > 75) return "#ff8800";
  return "#00ff88";
}

export function PieChart({ disks, totalSpace, totalUsed, avgUsage }: PieChartProps) {
  const size = 180;
  const center = size / 2;
  const radius = size * 0.4;

  const usedColor = getUsageHexColor(avgUsage);

  const { usedPath, freePath } = useMemo(() => {
    const usedAngle = totalSpace > 0 ? (totalUsed / totalSpace) * 360 : 0;
    const startAngle = -90;

    function describeArc(startDeg: number, endDeg: number): string {
      const startRad = (startDeg * Math.PI) / 180;
      const endRad = (endDeg * Math.PI) / 180;
      const points: string[] = [`${center},${center}`];
      const steps = 32;
      for (let i = 0; i <= steps; i++) {
        const angle = startRad + (endRad - startRad) * (i / steps);
        const x = center + radius * Math.cos(angle);
        const y = center + radius * Math.sin(angle);
        points.push(`${x.toFixed(2)},${y.toFixed(2)}`);
      }
      return points.join(" ");
    }

    return {
      usedPath: describeArc(startAngle, startAngle + usedAngle),
      freePath: describeArc(startAngle + usedAngle, startAngle + 360),
    };
  }, [totalSpace, totalUsed, center, radius]);

  const diskColors = ["#6496ff", "#ff9664", "#96ff96", "#ffc864", "#c896ff"];

  return (
    <div className="p-4 rounded-xl glass border border-white/[0.06] animate-fade-in">
      <div className="flex items-center gap-2 mb-3">
        <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Usage Breakdown</h3>
        <div className="flex-1 h-px bg-gradient-to-r from-white/10 to-transparent" />
      </div>

      {/* SVG Chart with glow filter */}
      <svg width={size} height={size} className="mx-auto block drop-shadow-lg">
        <defs>
          <filter id="chartGlow" x="-20%" y="-20%" width="140%" height="140%">
            <feGaussianBlur stdDeviation="3" result="blur" />
            <feMerge>
              <feMergeNode in="blur" />
              <feMergeNode in="SourceGraphic" />
            </feMerge>
          </filter>
          <radialGradient id="centerGlow" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor={usedColor} stopOpacity="0.08" />
            <stop offset="100%" stopColor="transparent" stopOpacity="0" />
          </radialGradient>
        </defs>

        {/* Subtle center glow */}
        <circle cx={center} cy={center} r={radius * 1.1} fill="url(#centerGlow)" />

        {/* Chart segments */}
        <polygon points={usedPath} fill={usedColor} opacity="0.85" filter="url(#chartGlow)" />
        <polygon points={freePath} fill="#32c832" opacity="0.65" />

        {/* Center label */}
        <text x={center} y={center - 4} textAnchor="middle" fill="white" fontSize="16" fontWeight="bold" opacity="0.9">
          {avgUsage.toFixed(0)}%
        </text>
        <text x={center} y={center + 12} textAnchor="middle" fill="#786496" fontSize="9" letterSpacing="1">
          USED
        </text>
      </svg>

      {/* Legend */}
      <div className="flex items-center gap-5 mt-3 text-xs justify-center">
        <div className="flex items-center gap-1.5">
          <div className="w-2 h-2 rounded-full" style={{ backgroundColor: usedColor, boxShadow: `0 0 6px ${usedColor}40` }} />
          <span className="text-gray-400">Used {avgUsage.toFixed(1)}%</span>
        </div>
        <div className="flex items-center gap-1.5">
          <div className="w-2 h-2 rounded-full bg-[#32c832]" style={{ boxShadow: "0 0 6px rgba(50, 200, 50, 0.3)" }} />
          <span className="text-gray-400">Free {(100 - avgUsage).toFixed(1)}%</span>
        </div>
      </div>

      {/* Per-disk breakdown */}
      {disks.length > 1 && (
        <div className="mt-3 pt-3 border-t border-white/[0.06]">
          <p className="font-semibold text-[11px] text-gray-500 uppercase tracking-wider mb-2">By Disk</p>
          {disks.map((disk, i) => {
            const diskPercent = disk.total_space > 0
              ? (disk.used_space / disk.total_space) * 100
              : 0;
            const spacePercent = totalSpace > 0
              ? (disk.total_space / totalSpace) * 100
              : 0;
            const color = diskColors[i % diskColors.length];
            const display = disk.name
              ? `${disk.mount_point} (${disk.name})`
              : disk.mount_point;

            return (
              <div key={disk.mount_point} className="flex items-center gap-2 text-[11px] py-0.5">
                <div className="w-1.5 h-1.5 rounded-full flex-shrink-0" style={{ backgroundColor: color }} />
                <span className="text-gray-400 truncate">{display}</span>
                <span className="text-gray-500 ml-auto flex-shrink-0 font-mono text-[10px]">
                  {diskPercent.toFixed(1)}% ({spacePercent.toFixed(0)}%)
                </span>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
