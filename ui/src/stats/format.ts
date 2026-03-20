export function fmtNum(n: number): string {
  return n.toLocaleString("en-US");
}

export function fmtMB(mb: number): string {
  if (mb >= 1024) return (mb / 1024).toFixed(1) + " GB";
  return mb.toFixed(1) + " MB";
}
