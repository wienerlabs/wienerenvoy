"use client";

// Client shell: gates on a token, then mounts the telemetry provider and the
// sidebar + main layout. Rendered from the server `layout.tsx`.

import { useEffect, useState } from "react";

import { hasToken, setToken } from "@/lib/api";
import { TelemetryProvider } from "@/lib/ws";

import { Sidebar } from "./sidebar";

export function AppShell({ children }: { children: React.ReactNode }) {
  const [ready, setReady] = useState(false);
  const [tokenPresent, setTokenPresent] = useState(false);

  useEffect(() => {
    setTokenPresent(hasToken());
    setReady(true);
  }, []);

  if (!ready) return null;

  if (!tokenPresent) {
    return <TokenGate onSaved={() => setTokenPresent(true)} />;
  }

  return (
    <TelemetryProvider>
      <div className="flex min-h-dvh">
        <Sidebar />
        <main className="min-w-0 flex-1">{children}</main>
      </div>
    </TelemetryProvider>
  );
}

function TokenGate({ onSaved }: { onSaved: () => void }) {
  const [value, setValue] = useState("");

  function save() {
    const trimmed = value.trim();
    if (trimmed) {
      setToken(trimmed);
      onSaved();
    }
  }

  return (
    <div className="flex min-h-dvh items-center justify-center p-6">
      <div className="surface w-full max-w-md p-6">
        <h1 className="text-xl font-semibold tracking-tight">WienerEnvoy</h1>
        <p className="mt-2 text-sm" style={{ color: "var(--color-muted)" }}>
          Paste the daemon token to connect. Get it on the Mac mini with{" "}
          <code className="mono">sudo wenvoy token show</code>.
        </p>
        <input
          className="input mt-4 w-full px-3 py-2"
          type="password"
          placeholder="bearer token"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") save();
          }}
        />
        <button type="button" className="btn btn-primary mt-3 w-full" onClick={save}>
          Connect
        </button>
      </div>
    </div>
  );
}
