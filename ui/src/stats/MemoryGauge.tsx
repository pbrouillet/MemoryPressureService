import type { MemoryStats } from "@shared/types";
import { fmtMB } from "./format";
import "./MemoryGauge.css";

interface Props {
  stats: MemoryStats;
}

export function MemoryGauge({ stats }: Props) {
  const pct = stats.memory_load_percent;
  const used = stats.total_physical_mb - stats.available_physical_mb;

  return (
    <div className="mem-gauge">
      <div className="gauge-bar">
        <div
          className={`gauge-fill${pct > 80 ? " high" : ""}`}
          style={{ width: `${pct}%` }}
        />
      </div>
      <div>
        <div className="gauge-label">{pct}%</div>
        <div className="gauge-text">
          {fmtMB(used)} used / {fmtMB(stats.total_physical_mb)} total
        </div>
      </div>
    </div>
  );
}
