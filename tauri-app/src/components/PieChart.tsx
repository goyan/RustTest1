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
    <div className="p-3 rounded-lg bg-cyber-panel border border-[#323741]">
      <h3 className="text-base font-semibold text-[#b4c8ff] mb-2.5">Disk Usage Breakdown</h3>

      <svg width={size} height={size} className="mx-auto block">
        <polygon points={usedPath} fill={usedColor} />
        <polygon points={freePath} fill="#32c832" />
      </svg>

      <div className="flex items-center gap-4 mt-2.5 text-sm justify-center">
        <div className="flex items-center gap-1">
          <span style={{ color: usedColor }} className="text-base">●</span>
          Used: {avgUsage.toFixed(1)}%
        </div>
        <div className="flex items-center gap-1">
          <span style={{ color: "#32c832" }} className="text-base">●</span>
          Available: {(100 - avgUsage).toFixed(1)}%
        </div>
      </div>

      {disks.length > 1 && (
        <div className="mt-2.5 pt-2 border-t border-[#323741]">
          <p className="font-semibold text-sm mb-1">By Disk:</p>
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
              <div key={disk.mount_point} className="flex items-center gap-1 text-xs">
                <span style={{ color }} className="text-xs">●</span>
                {display}: {diskPercent.toFixed(1)}% ({spacePercent.toFixed(1)}% of total)
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
