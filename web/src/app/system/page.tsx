"use client";

import { Sparkline } from "@/components/sparkline";
import { StatCard } from "@/components/stat-card";
import { formatBytes, formatBytesPerSec, formatPct } from "@/lib/format";
import { useTelemetry } from "@/lib/ws";

export default function SystemPage() {
  const { snapshot, cpuSeries, rxSeries, txSeries } = useTelemetry();
  const memPct =
    snapshot && snapshot.memory.totalBytes > 0
      ? (snapshot.memory.usedBytes / snapshot.memory.totalBytes) * 100
      : null;

  return (
    <div className="container-page py-8">
      <header className="mb-8">
        <h1 className="text-3xl font-light tracking-tight">System</h1>
        <p className="mt-1 text-sm" style={{ color: "var(--color-muted)" }}>
          Live telemetry from the Mac mini.
        </p>
      </header>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <StatCard
          label="CPU"
          value={snapshot ? formatPct(snapshot.cpu.usagePct) : "--"}
          hint={snapshot ? `${snapshot.cpu.cores} cores` : undefined}
        />
        <StatCard
          label="Memory"
          value={memPct !== null ? formatPct(memPct) : "--"}
          hint={snapshot ? `${formatBytes(snapshot.memory.usedBytes)} used` : undefined}
        />
        <StatCard label="Net rx" value={snapshot ? formatBytesPerSec(snapshot.network.rxBytesPerSec) : "--"} />
        <StatCard label="Net tx" value={snapshot ? formatBytesPerSec(snapshot.network.txBytesPerSec) : "--"} />
      </div>

      <div className="mt-4 grid grid-cols-1 gap-4 lg:grid-cols-3">
        <div className="surface p-5">
          <div className="chip mb-3">CPU %</div>
          <Sparkline data={cpuSeries} width={300} height={48} />
        </div>
        <div className="surface p-5">
          <div className="chip mb-3">Net rx</div>
          <Sparkline data={rxSeries} width={300} height={48} color="var(--color-online)" />
        </div>
        <div className="surface p-5">
          <div className="chip mb-3">Net tx</div>
          <Sparkline data={txSeries} width={300} height={48} color="var(--color-standby)" />
        </div>
      </div>

      {snapshot && snapshot.disks.length > 0 ? (
        <div className="surface mt-4 p-5">
          <div className="chip mb-3">Disks</div>
          <div className="flex flex-col gap-2">
            {snapshot.disks.map((disk) => (
              <div key={disk.mount} className="flex justify-between text-sm">
                <span className="mono">{disk.mount}</span>
                <span className="num" style={{ color: "var(--color-muted)" }}>
                  {formatBytes(disk.usedBytes)} / {formatBytes(disk.totalBytes)}
                </span>
              </div>
            ))}
          </div>
        </div>
      ) : null}
    </div>
  );
}
