"use client";

import { KeepAwakeToggle } from "@/components/power/keep-awake-toggle";
import { PowerActions } from "@/components/power/power-actions";
import { ScheduledWake } from "@/components/power/scheduled-wake";
import { StatCard } from "@/components/stat-card";
import { useTelemetry } from "@/lib/ws";

export default function PowerPage() {
  const { presence, server } = useTelemetry();

  return (
    <div className="container-page py-8">
      <header className="mb-8">
        <h1 className="text-3xl font-light tracking-tight">Power</h1>
        <p className="mt-1 text-sm" style={{ color: "var(--color-muted)" }}>
          Control presence and machine power. Sleep is recoverable over the tailnet; shutdown
          leaves only Wake-on-LAN and needs confirmation.
        </p>
      </header>

      <section className="mb-8">
        <div className="chip mb-4">Presence</div>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <StatCard label="Keep awake" value={presence?.keepAwake ? "on" : "off"} />
          <StatCard label="Server level" value={server?.level === "server_off" ? "off" : "active"} />
          <StatCard label="Backend" value={presence?.backend ?? "caffeinate"} />
        </div>
        <div className="mt-4 flex flex-wrap gap-3">
          <KeepAwakeToggle />
        </div>
      </section>

      <section className="mb-8">
        <div className="chip mb-4">Machine</div>
        <PowerActions />
      </section>

      <section>
        <div className="chip mb-4">Scheduled wake</div>
        <ScheduledWake />
      </section>
    </div>
  );
}
