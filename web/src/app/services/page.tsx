"use client";

import { useCallback, useEffect, useState } from "react";

import { StatusPill } from "@/components/status-pill";
import { api } from "@/lib/api";
import type { Presence, ServiceAction, ServiceContainer, ServicesView } from "@/lib/types";

function containerStatus(state: string): Presence {
  if (state === "running") return "online";
  if (state === "paused" || state === "created" || state === "restarting") return "standby";
  return "offline";
}

export default function ServicesPage() {
  const [view, setView] = useState<ServicesView | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState<string | null>(null);
  const [logTarget, setLogTarget] = useState<ServiceContainer | null>(null);
  const [logLines, setLogLines] = useState<string[]>([]);
  const [logLoading, setLogLoading] = useState(false);

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

  async function control(stack: string, action: ServiceAction) {
    setBusy(`${stack}:${action}`);
    try {
      await api.serviceControl("stack", stack, action);
      await load();
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(null);
    }
  }

  async function showLogs(container: ServiceContainer) {
    setLogTarget(container);
    setLogLines([]);
    setLogLoading(true);
    try {
      const result = await api.serviceLogs(container.id);
      setLogLines(result.lines.length > 0 ? result.lines : ["(no log output)"]);
    } catch (err) {
      setLogLines([`error: ${err instanceof Error ? err.message : String(err)}`]);
    } finally {
      setLogLoading(false);
    }
  }

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
        <div className="surface mb-4 p-5" style={{ color: "var(--color-offline)" }}>
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
              <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
                <span className="text-lg font-medium">{stack.name}</span>
                <div className="flex items-center gap-2">
                  <span className="chip num">
                    {stack.running}/{stack.total} up
                  </span>
                  <button
                    type="button"
                    className="btn btn-ghost"
                    disabled={busy !== null}
                    onClick={() => void control(stack.name, "start")}
                  >
                    Start
                  </button>
                  <button
                    type="button"
                    className="btn btn-ghost"
                    disabled={busy !== null}
                    onClick={() => void control(stack.name, "restart")}
                  >
                    Restart
                  </button>
                  <button
                    type="button"
                    className="btn btn-ghost"
                    disabled={busy !== null}
                    onClick={() => void control(stack.name, "stop")}
                  >
                    Stop
                  </button>
                </div>
              </div>
              <ContainerList containers={stack.containers} onLogs={showLogs} />
            </div>
          ))}
          {view.standalone.length > 0 ? (
            <div className="surface p-5">
              <div className="chip mb-4">Standalone</div>
              <ContainerList containers={view.standalone} onLogs={showLogs} />
            </div>
          ) : null}
        </div>
      ) : null}

      {logTarget ? (
        <LogModal
          name={logTarget.name}
          lines={logLines}
          loading={logLoading}
          onClose={() => setLogTarget(null)}
        />
      ) : null}
    </div>
  );
}

function ContainerList({
  containers,
  onLogs,
}: {
  containers: ServiceContainer[];
  onLogs: (container: ServiceContainer) => void;
}) {
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
          <div className="flex items-center gap-3">
            <button type="button" className="btn btn-ghost" onClick={() => onLogs(container)}>
              Logs
            </button>
            <StatusPill status={containerStatus(container.state)} label={container.state} />
          </div>
        </div>
      ))}
    </div>
  );
}

function LogModal({
  name,
  lines,
  loading,
  onClose,
}: {
  name: string;
  lines: string[];
  loading: boolean;
  onClose: () => void;
}) {
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-6"
      style={{ background: "rgba(0,0,0,0.5)" }}
      onClick={onClose}
    >
      <div
        className="surface-elevated flex max-h-[80vh] w-full max-w-3xl flex-col p-5"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mb-3 flex items-center justify-between">
          <span className="mono text-sm">{name}</span>
          <button type="button" className="btn btn-ghost" onClick={onClose}>
            Close
          </button>
        </div>
        <pre
          className="mono flex-1 overflow-auto rounded-md p-3 text-xs"
          style={{ background: "var(--color-bg-deep)", color: "var(--color-muted)" }}
        >
          {loading ? "loading..." : lines.join("\n")}
        </pre>
      </div>
    </div>
  );
}
