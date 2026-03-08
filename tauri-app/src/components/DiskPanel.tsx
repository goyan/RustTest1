import { useMemo } from "react";
import type { DiskInfo } from "../types";
import { PieChart } from "./PieChart";

interface DiskPanelProps {
  disks: DiskInfo[];
  currentDisk: string | null;
  onSelectDisk: (mountPoint: string) => void;
}

function formatGB(bytes: number): string {
  return (bytes / 1_000_000_000).toFixed(2);
}

function getUsageColor(percent: number): string {
  if (percent > 90) return "text-cyber-neon-red";
  if (percent > 75) return "text-cyber-neon-orange";
  return "text-cyber-neon-green";
}

function getUsageBarColor(percent: number): string {
  if (percent > 90) return "from-red-500 to-red-400";
  if (percent > 75) return "from-orange-500 to-amber-400";
  return "from-emerald-500 to-cyan-400";
}

export function DiskPanel({ disks, currentDisk, onSelectDisk }: DiskPanelProps) {
  const { totalSpace, totalUsed, avgUsage } = useMemo(() => {
    const total = disks.reduce((sum, d) => sum + d.total_space, 0);
    const used = disks.reduce((sum, d) => sum + d.used_space, 0);
    const avg = total > 0 ? (used / total) * 100 : 0;
    return { totalSpace: total, totalUsed: used, avgUsage: avg };
  }, [disks]);

  return (
    <aside className="w-[290px] min-w-[240px] max-w-[400px] border-r border-white/[0.04] overflow-y-auto bg-[#0c0a14]">
      <div className="p-4">
        {/* Section title */}
        <div className="flex items-center gap-2 mb-4">
          <h2 className="text-sm font-semibold text-gray-400 uppercase tracking-[0.12em]">Disks</h2>
          <div className="flex-1 h-px bg-gradient-to-r from-white/10 to-transparent" />
          <span className="text-[10px] text-cyber-dim-purple font-mono">{disks.length}</span>
        </div>

        {/* Disk cards */}
        {disks.map((disk, index) => {
          const isSelected = currentDisk === disk.mount_point;
          const displayName = disk.name
            ? `${disk.mount_point} (${disk.name})`
            : disk.mount_point;

          return (
            <button
              key={disk.mount_point}
              onClick={() => onSelectDisk(disk.mount_point)}
              className={`w-full text-left p-4 rounded-xl mb-3 border transition-all duration-200 hover-lift animate-fade-in group ${
                isSelected
                  ? "glass border-cyan-400/40 glow-cyan-soft"
                  : "glass-subtle border-white/[0.04] hover:border-white/[0.1] hover:bg-white/[0.03]"
              }`}
              style={{ animationDelay: `${index * 0.05}s` }}
            >
              {/* Disk name row */}
              <div className="flex items-center gap-2.5 mb-3">
                <div className={`flex items-center justify-center w-8 h-8 rounded-lg ${
                  isSelected ? "bg-cyan-500/15" : "bg-white/[0.04] group-hover:bg-white/[0.06]"
                } transition-colors`}>
                  <span className="text-base">&#x1F4BF;</span>
                </div>
                <span className={`text-sm font-semibold truncate ${
                  isSelected ? "text-cyan-300" : "text-gray-200 group-hover:text-gray-100"
                } transition-colors`}>
                  {displayName}
                </span>
              </div>

              {/* Usage percentage */}
              <div className="flex items-center justify-between mb-2">
                <span className={`text-xs font-bold ${getUsageColor(disk.usage_percent)}`}>
                  {disk.usage_percent.toFixed(1)}% used
                </span>
                <span className="text-[10px] text-gray-500 font-mono">
                  {formatGB(disk.used_space)} / {formatGB(disk.total_space)} GB
                </span>
              </div>

              {/* Progress bar */}
              <div className="w-full h-1.5 bg-white/[0.05] rounded-full overflow-hidden">
                <div
                  className={`h-full rounded-full bg-gradient-to-r ${getUsageBarColor(disk.usage_percent)} animate-progress`}
                  style={{ width: `${disk.usage_percent}%` }}
                />
              </div>
            </button>
          );
        })}

        {/* Summary card */}
        {disks.length > 0 && (
          <>
            <div className="mt-5 p-4 rounded-xl glass border border-white/[0.06] animate-fade-in">
              <div className="flex items-center gap-2 mb-3">
                <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Summary</h3>
                <div className="flex-1 h-px bg-gradient-to-r from-white/10 to-transparent" />
              </div>
              <div className="grid grid-cols-3 gap-3">
                <div className="text-center p-2 rounded-lg bg-white/[0.02]">
                  <div className="text-lg font-bold text-gradient-cyan">{disks.length}</div>
                  <div className="text-[10px] text-gray-500 uppercase tracking-wider mt-0.5">Disks</div>
                </div>
                <div className="text-center p-2 rounded-lg bg-white/[0.02]">
                  <div className="text-lg font-bold text-gradient-cyan">{Math.round(totalSpace / 1_000_000_000)}</div>
                  <div className="text-[10px] text-gray-500 uppercase tracking-wider mt-0.5">GB Total</div>
                </div>
                <div className="text-center p-2 rounded-lg bg-white/[0.02]">
                  <div className={`text-lg font-bold ${getUsageColor(avgUsage)}`}>{avgUsage.toFixed(1)}%</div>
                  <div className="text-[10px] text-gray-500 uppercase tracking-wider mt-0.5">Used</div>
                </div>
              </div>
            </div>

            <div className="my-4 h-px bg-gradient-to-r from-transparent via-white/[0.06] to-transparent" />

            <PieChart
              disks={disks}
              totalSpace={totalSpace}
              totalUsed={totalUsed}
              avgUsage={avgUsage}
            />
          </>
        )}
      </div>
    </aside>
  );
}
