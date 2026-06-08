import type { Presence } from "@/lib/types";

export function StatusPill({
  status,
  label,
}: {
  status: Presence;
  label?: string;
}) {
  return (
    <span className="pill" data-status={status}>
      <span className="dot" />
      {label ?? status}
    </span>
  );
}
