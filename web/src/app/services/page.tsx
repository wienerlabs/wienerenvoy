"use client";

import { useCallback, useEffect, useState } from "react";

import { StatusPill } from "@/components/status-pill";
import { api } from "@/lib/api";
import type { Presence, ServiceContainer, ServicesView } from "@/lib/types";

function containerStatus(state: string): Presence {
  if (state === "running") return "online";
  if (state === "paused" || state === "created" || state === "restarting") return "standby";
  return "offline";
}

export default function ServicesPage() {
  const [view, setView] = useState<ServicesView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      setView(await api.services());
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const empty =
    view && view.docker.available && view.stacks.length === 0 && view.standalone.length === 0;

  return (
    <div className="container-page py-8">
      <header className="mb-8 flex items-start justify-between gap-4">
        <div>
          <h1 className="text-3xl font-light tracking-tight">Services</h1>
          <p className="mt-1 text-sm" style={{ color: "var(--color-muted)" }}>
            Docker Compose stacks and containers.
          </p>
        </div>
        <button type="button" className="btn" onClick={() => void load()} disabled={loading}>
          Refresh
        </button>
      </header>

      {error ? (
        <div className="surface p-5" style={{ color: "var(--color-offline)" }}>
          {error}
        </div>
      ) : null}

      {view && !view.docker.available ? (
        <div className="surface p-6">
          <div className="chip mb-3">Docker</div>
          <p className="text-sm" style={{ color: "var(--color-muted)" }}>
            Docker is not detected. Install Docker Desktop, OrbStack, or Colima on the Mac mini to
            manage Compose stacks here.
          </p>
        </div>
      ) : null}

      {empty ? (
        <div className="surface p-6">
          <p className="text-sm" style={{ color: "var(--color-muted)" }}>
            Docker is running ({view?.docker.version ?? "unknown"}), but no containers were found.
          </p>
        </div>
      ) : null}

      {view && view.docker.available ? (
        <div className="flex flex-col gap-4">
          {view.stacks.map((stack) => (
            <div key={stack.name} className="surface p-5">
              <div className="mb-4 flex items-center justify-between gap-4">
                <span className="text-lg font-medium">{stack.name}</span>
                <span className="chip num">
                  {stack.running}/{stack.total} up
                </span>
              </div>
              <ContainerList containers={stack.containers} />
            </div>
          ))}
          {view.standalone.length > 0 ? (
            <div className="surface p-5">
              <div className="chip mb-4">Standalone</div>
              <ContainerList containers={view.standalone} />
            </div>
          ) : null}
        </div>
      ) : null}
    </div>
  );
}

function ContainerList({ containers }: { containers: ServiceContainer[] }) {
  return (
    <div className="flex flex-col gap-2">
      {containers.map((container) => (
        <div key={container.id} className="flex items-center justify-between gap-4">
          <div className="min-w-0">
            <div className="truncate font-medium">{container.name}</div>
            <div className="mono truncate text-xs" style={{ color: "var(--color-muted-2)" }}>
              {container.image}
            </div>
          </div>
          <StatusPill status={containerStatus(container.state)} label={container.state} />
        </div>
      ))}
    </div>
  );
}
