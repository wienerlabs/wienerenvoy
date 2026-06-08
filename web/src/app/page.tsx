import { StatCard } from "@/components/stat-card";
import { StatusPill } from "@/components/status-pill";

export default function OverviewPage() {
  return (
    <div className="container-page py-8">
      <header className="mb-8">
        <h1 className="text-3xl font-light tracking-tight">Overview</h1>
        <p className="mt-1 text-sm" style={{ color: "var(--color-muted)" }}>
          Your Mac mini at a glance.
        </p>
      </header>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard label="Presence" value={<StatusPill status="standby" label="standby" />} />
        <StatCard label="Keep awake" value="--" hint="Available in M1" />
        <StatCard label="Uptime" value="--" hint="Available in M1" />
        <StatCard label="Load" value="--" hint="Available in M1" />
      </div>

      <div className="mt-4 grid grid-cols-1 gap-4 lg:grid-cols-2">
        <div className="surface p-5">
          <div className="chip mb-3">Power</div>
          <p className="text-sm" style={{ color: "var(--color-muted)" }}>
            Keep-awake, sleep, restart, and shutdown controls land in M1. Sleep stays
            recoverable over the tailnet; shutdown will require confirmation.
          </p>
        </div>
        <div className="surface p-5">
          <div className="chip mb-3">System</div>
          <p className="text-sm" style={{ color: "var(--color-muted)" }}>
            Live CPU, memory, disk, and network telemetry over a WebSocket lands in M1.
          </p>
        </div>
      </div>
    </div>
  );
}
