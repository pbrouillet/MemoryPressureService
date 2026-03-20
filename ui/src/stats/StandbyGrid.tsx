import type { MemoryStats } from "@shared/types";
import { fmtMB, fmtNum } from "./format";
import "./StandbyGrid.css";

interface Props {
  stats: MemoryStats;
}

export function StandbyGrid({ stats }: Props) {
  const sbp = stats.standby_pages_by_priority;
  const maxSb = Math.max(...sbp, 1);
  const pageSize = stats.page_size_bytes;
  const toMB = (p: number) => (p * pageSize) / 1048576;

  return (
    <div className="standby-grid">
      {sbp.map((count, i) => {
        const label =
          i === 0 ? "0 (Lowest)" : i === 7 ? "7 (Highest)" : String(i);
        const w = ((count / maxSb) * 100).toFixed(1);
        return (
          <div className="standby-row" key={i}>
            <span className="label">{label}</span>
            <div className="bar-bg bar-standby">
              <div className="bar-fill" style={{ width: `${w}%` }} />
            </div>
            <span className="pages">{fmtNum(count)}</span>
            <span className="size">{fmtMB(toMB(count))}</span>
          </div>
        );
      })}
    </div>
  );
}
