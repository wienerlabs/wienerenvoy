"use client";

import { Sparkline } from "@/components/sparkline";
import { StatCard } from "@/components/stat-card";
import { StatusPill } from "@/components/status-pill";
import { formatBytes, formatPct, formatUptime } from "@/lib/format";
import { useTelemetry } from "@/lib/ws";

export default function OverviewPage() {
  const { connected, snapshot, presence, cpuSeries } = useTelemetry();
  const status = connected ? (snapshot?.presence ?? "online") : "offline";

  return (
    <div className="container-page py-8">
      <header className="mb-8">
        <h1 className="text-3xl font-light tracking-tight">Overview</h1>
        <p className="mt-1 text-sm" style={{ color: "var(--color-muted)" }}>
          Your Mac mini at a glance.
        </p>
      </header>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard
          label="Presence"
          value={<StatusPill status={status} label={connected ? status : "offline"} />}
        />
        <StatCard label="Keep awake" value={presence?.keepAwake ? "on" : "off"} />
        <StatCard label="Uptime" value={snapshot ? formatUptime(snapshot.uptimeSecs) : "--"} />
        <StatCard
          label="CPU"
          value={snapshot ? formatPct(snapshot.cpu.usagePct) : "--"}
          hint={snapshot ? `${snapshot.cpu.cores} cores` : undefined}
        />
      </div>

      <div className="mt-4 grid grid-cols-1 gap-4 lg:grid-cols-2">
        <div className="surface p-5">
          <div className="chip mb-3">CPU load</div>
          <Sparkline data={cpuSeries} width={340} height={48} />
        </div>
        <div className="surface p-5">
          <div className="chip mb-3">Memory</div>
          <div className="num text-2xl font-light">
            {snapshot
              ? `${formatBytes(snapshot.memory.usedBytes)} / ${formatBytes(snapshot.memory.totalBytes)}`
              : "--"}
          </div>
        </div>
      </div>
    </div>
  );
}
