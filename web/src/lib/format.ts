// Small formatters for telemetry values. Kept dependency-free.

const UNITS = ["B", "KB", "MB", "GB", "TB", "PB"];

export function formatBytes(n: number): string {
  if (n <= 0) return "0 B";
  const exp = Math.min(Math.floor(Math.log(n) / Math.log(1024)), UNITS.length - 1);
  const value = n / 1024 ** exp;
  return `${value.toFixed(value >= 100 || exp === 0 ? 0 : 1)} ${UNITS[exp]}`;
}

export function formatBytesPerSec(n: number): string {
  return `${formatBytes(n)}/s`;
}

export function formatPct(n: number): string {
  return `${Math.round(n)}%`;
}

export function formatUptime(secs: number): string {
  if (secs <= 0) return "--";
  const d = Math.floor(secs / 86400);
  const h = Math.floor((secs % 86400) / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (d > 0) return `${d}d ${h}h`;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

export function formatSince(iso: string): string {
  const then = new Date(iso).getTime();
  if (Number.isNaN(then)) return "--";
  const secs = Math.max(0, Math.floor((Date.now() - then) / 1000));
  return `${formatUptime(secs)} ago`;
}
