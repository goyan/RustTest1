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
  if (percent > 90) return "bg-cyber-neon-red";
  if (percent > 75) return "bg-cyber-neon-orange";
  return "bg-cyber-neon-green";
}

function getUsageBorderColor(percent: number): string {
  if (percent > 90) return "border-cyber-neon-red";
  if (percent > 75) return "border-cyber-neon-orange";
  return "border-cyber-neon-green";
}

export function DiskPanel({ disks, currentDisk, onSelectDisk }: DiskPanelProps) {
  const { totalSpace, totalUsed, avgUsage } = useMemo(() => {
    const total = disks.reduce((sum, d) => sum + d.total_space, 0);
    const used = disks.reduce((sum, d) => sum + d.used_space, 0);
    const avg = total > 0 ? (used / total) * 100 : 0;
    return { totalSpace: total, totalUsed: used, avgUsage: avg };
  }, [disks]);

  return (
    <aside className="w-[280px] min-w-[220px] max-w-[400px] border-r border-[#2d3037] overflow-y-auto bg-[#12101a]">
      <div className="p-3">
        <h2 className="text-lg font-semibold text-gray-200 mb-2">Disks</h2>
        <div className="border-b border-[#2d3037] mb-3" />

        {/* Disk cards */}
        {disks.map((disk) => {
          const isSelected = currentDisk === disk.mount_point;
          const displayName = disk.name
            ? `${disk.mount_point} (${disk.name})`
            : disk.mount_point;

          return (
            <button
              key={disk.mount_point}
              onClick={() => onSelectDisk(disk.mount_point)}
              className={`w-full text-left p-3.5 rounded-lg mb-3 border transition-all ${
                isSelected
                  ? "bg-[#283b4b] border-blue-400 border-2"
                  : "bg-cyber-card hover:bg-cyber-card-hover border-[#2d3037] hover:border-current"
              } ${!isSelected ? getUsageBorderColor(disk.usage_percent).replace("border-", "hover:border-") : ""}`}
            >
              <div className="flex items-center gap-2 mb-2">
                <span className="text-xl">💿</span>
                <span className="text-base font-semibold text-[#dce6ff]">{displayName}</span>
              </div>
              <p className={`text-sm font-semibold mb-1.5 ${getUsageColor(disk.usage_percent)}`}>
                {disk.usage_percent.toFixed(1)}% used
              </p>
              <div className="w-full h-2 bg-[#12101a] rounded-full overflow-hidden mb-1">
                <div
                  className={`h-full rounded-full transition-all ${getUsageBarColor(disk.usage_percent)}`}
                  style={{ width: `${disk.usage_percent}%` }}
                />
              </div>
              <p className="text-xs text-gray-400 mt-1">
                {formatGB(disk.used_space)} GB / {formatGB(disk.total_space)} GB
              </p>
            </button>
          );
        })}

        {/* Summary */}
        {disks.length > 0 && (
          <>
            <div className="mt-4 p-3 rounded-lg bg-cyber-panel border border-[#323741]">
              <h3 className="text-base font-semibold text-[#b4c8ff] mb-2.5">Summary</h3>
              <div className="grid grid-cols-3 gap-2 text-center">
                <div>
                  <div className="text-lg font-bold text-[#64c8ff]">{disks.length}</div>
                  <div className="text-[10px] text-cyber-dim-purple">Disks</div>
                </div>
                <div>
                  <div className="text-lg font-bold text-[#64c8ff]">{Math.round(totalSpace / 1_000_000_000)} GB</div>
                  <div className="text-[10px] text-cyber-dim-purple">Total</div>
                </div>
                <div>
                  <div className={`text-lg font-bold ${getUsageColor(avgUsage)}`}>{avgUsage.toFixed(1)}%</div>
                  <div className="text-[10px] text-cyber-dim-purple">Used</div>
                </div>
              </div>
            </div>

            <div className="border-b border-[#2d3037] my-3" />

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
