import { Sparkline } from "@/components/sparkline";
import { StatCard } from "@/components/stat-card";

export default function SystemPage() {
  return (
    <div className="container-page py-8">
      <header className="mb-8">
        <h1 className="text-3xl font-light tracking-tight">System</h1>
        <p className="mt-1 text-sm" style={{ color: "var(--color-muted)" }}>
          Live telemetry from the Mac mini.
        </p>
      </header>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard label="CPU" value={<Sparkline data={[]} />} hint="Available in M1" />
        <StatCard label="Memory" value="--" hint="Available in M1" />
        <StatCard label="Disk" value="--" hint="Available in M1" />
        <StatCard label="Network" value="--" hint="Available in M1" />
      </div>

      <p className="mt-4 text-xs" style={{ color: "var(--color-muted)" }}>
        Telemetry streams over a WebSocket once M1 wires the sysinfo sampler.
      </p>
    </div>
  );
}
