import type { MemoryStats } from "@shared/types";
import { fmtMB, fmtNum } from "./format";
import "./PageListTable.css";

interface Props {
  stats: MemoryStats;
}

type BarColor = "zeroed" | "free" | "modified" | "standby" | "bad";

export function PageListTable({ stats }: Props) {
  const pageSize = stats.page_size_bytes;
  const toMB = (p: number) => (p * pageSize) / 1048576;
  const totalPages =
    stats.zeroed_pages +
    stats.free_pages +
    stats.modified_pages +
    stats.total_standby_pages +
    (stats.bad_pages || 0);
  const pctOf = (p: number) => (totalPages > 0 ? (p / totalPages) * 100 : 0);

  const rows: [string, number, number, BarColor][] = [
    ["Zeroed", stats.zeroed_pages, toMB(stats.zeroed_pages), "zeroed"],
    ["Free", stats.free_pages, toMB(stats.free_pages), "free"],
    ["Modified", stats.modified_pages, toMB(stats.modified_pages), "modified"],
    [
      "Standby",
      stats.total_standby_pages,
      toMB(stats.total_standby_pages),
      "standby",
    ],
  ];
  if (stats.bad_pages > 0) {
    rows.push(["Bad", stats.bad_pages, toMB(stats.bad_pages), "bad"]);
  }

  return (
    <table>
      <thead>
        <tr>
          <th>List</th>
          <th className="num">Pages</th>
          <th className="num">Size</th>
          <th className="bar-cell">Distribution</th>
        </tr>
      </thead>
      <tbody>
        {rows.map(([label, pages, mb, color]) => (
          <tr key={label}>
            <td>{label}</td>
            <td className="num">{fmtNum(pages)}</td>
            <td className="num">{fmtMB(mb)}</td>
            <td className="bar-cell">
              <div className={`bar-bg bar-${color}`}>
                <div
                  className="bar-fill"
                  style={{ width: `${pctOf(pages).toFixed(1)}%` }}
                />
              </div>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
