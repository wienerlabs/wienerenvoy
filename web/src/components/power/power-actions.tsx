"use client";

import { useState } from "react";

import { api } from "@/lib/api";
import type { PowerAction } from "@/lib/types";

import { ConfirmModal } from "../confirm-modal";

export function PowerActions() {
  const [pending, setPending] = useState<PowerAction | null>(null);
  const [message, setMessage] = useState<string | null>(null);

  async function run(action: PowerAction) {
    setPending(null);
    try {
      // Sleep needs no confirm flag; restart and shutdown do.
      await api.power(action, action !== "sleep");
      setMessage(`${action} command sent`);
    } catch (err) {
      setMessage(`${action} failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  return (
    <div>
      <div className="flex flex-wrap gap-3">
        <button type="button" className="btn" onClick={() => setPending("sleep")}>
          Sleep
        </button>
        <button type="button" className="btn" onClick={() => setPending("restart")}>
          Restart
        </button>
        <button type="button" className="btn btn-danger" onClick={() => setPending("shutdown")}>
          Shutdown
        </button>
      </div>
      {message ? (
        <p className="mt-3 text-xs" style={{ color: "var(--color-muted)" }}>
          {message}
        </p>
      ) : null}

      <ConfirmModal
        open={pending === "sleep"}
        title="Sleep the machine?"
        body="The machine sleeps now. You can wake it over the tailnet, by Wake-on-LAN, or by a scheduled wake."
        confirmLabel="Sleep"
        onConfirm={() => run("sleep")}
        onCancel={() => setPending(null)}
      />
      <ConfirmModal
        open={pending === "restart"}
        title="Restart the machine?"
        body="The machine restarts. The daemon comes back on its own after boot."
        confirmLabel="Restart"
        onConfirm={() => run("restart")}
        onCancel={() => setPending(null)}
      />
      <ConfirmModal
        open={pending === "shutdown"}
        danger
        title="Shut down the machine?"
        body="The tailnet goes down on shutdown. The only remote recovery is Wake-on-LAN from a device on the same network. Are you sure?"
        confirmLabel="Shut down"
        onConfirm={() => run("shutdown")}
        onCancel={() => setPending(null)}
      />
    </div>
  );
}
