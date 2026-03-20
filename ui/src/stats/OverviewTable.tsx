import type { MemoryStats } from "@shared/types";
import { fmtMB, fmtNum } from "./format";
import "./OverviewTable.css";

interface Props {
  stats: MemoryStats;
}

export function OverviewTable({ stats }: Props) {
  const rows: [string, string][] = [
    ["Total Physical", fmtMB(stats.total_physical_mb)],
    ["Available Physical", fmtMB(stats.available_physical_mb)],
    [
      "Commit (Used / Limit)",
      `${fmtMB(stats.commit_total_mb)} / ${fmtMB(stats.commit_limit_mb)}`,
    ],
    ["System Cache", fmtMB(stats.system_cache_mb)],
    [
      "Kernel Paged / Nonpaged",
      `${fmtMB(stats.kernel_paged_mb)} / ${fmtMB(stats.kernel_nonpaged_mb)}`,
    ],
    [
      "Processes / Threads / Handles",
      `${fmtNum(stats.process_count)} / ${fmtNum(stats.thread_count)} / ${fmtNum(stats.handle_count)}`,
    ],
  ];

  return (
    <table>
      <thead>
        <tr>
          <th>Metric</th>
          <th className="num">Value</th>
        </tr>
      </thead>
      <tbody>
        {rows.map(([label, value]) => (
          <tr key={label}>
            <td>{label}</td>
            <td className="num">{value}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
