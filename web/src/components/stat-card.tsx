import type { ReactNode } from "react";

export function StatCard({
  label,
  value,
  hint,
}: {
  label: string;
  value: ReactNode;
  hint?: string;
}) {
  return (
    <div className="surface p-4">
      <div className="chip mb-3">{label}</div>
      <div className="num text-2xl font-light">{value}</div>
      {hint ? (
        <div className="mt-1 text-xs" style={{ color: "var(--color-muted)" }}>
          {hint}
        </div>
      ) : null}
    </div>
  );
}
