"use client";

// Live telemetry over a single WebSocket, shared via context so the sidebar and
// every page read one connection. The token is passed as a query param (browsers
// cannot set WebSocket headers). Reconnects with exponential backoff; keeps
// bounded per-series ring buffers for the sparklines.

import { createContext, useContext, useEffect, useState } from "react";

import { getToken } from "./api";
import type { PresenceState, ServerState, SystemSnapshot, WsFrame } from "./types";

export interface TelemetryState {
  connected: boolean;
  snapshot: SystemSnapshot | null;
  server: ServerState | null;
  presence: PresenceState | null;
  cpuSeries: number[];
  rxSeries: number[];
  txSeries: number[];
}

const INITIAL: TelemetryState = {
  connected: false,
  snapshot: null,
  server: null,
  presence: null,
  cpuSeries: [],
  rxSeries: [],
  txSeries: [],
};

const TelemetryContext = createContext<TelemetryState>(INITIAL);

export function useTelemetry(): TelemetryState {
  return useContext(TelemetryContext);
}

const MAX_POINTS = 60;

function ring(series: number[], value: number): number[] {
  const next = [...series, value];
  return next.length > MAX_POINTS ? next.slice(next.length - MAX_POINTS) : next;
}

function wsUrl(token: string): string | null {
  const base =
    process.env.NEXT_PUBLIC_DAEMON_URL ||
    (typeof window !== "undefined" ? window.location.origin : "");
  if (!base) return null;
  return `${base.replace(/^http/, "ws")}/api/v1/ws?token=${encodeURIComponent(token)}`;
}

function reduce(state: TelemetryState, frame: WsFrame): TelemetryState {
  switch (frame.type) {
    case "metrics": {
      const snapshot = frame as { type: "metrics" } & SystemSnapshot;
      return {
        ...state,
        snapshot,
        cpuSeries: ring(state.cpuSeries, snapshot.cpu.usagePct),
        rxSeries: ring(state.rxSeries, snapshot.network.rxBytesPerSec),
        txSeries: ring(state.txSeries, snapshot.network.txBytesPerSec),
      };
    }
    case "state_changed":
      return { ...state, server: frame as { type: "state_changed" } & ServerState };
    case "presence_changed":
      return { ...state, presence: frame as { type: "presence_changed" } & PresenceState };
    default:
      return state;
  }
}

export function TelemetryProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<TelemetryState>(INITIAL);

  useEffect(() => {
    let stopped = false;
    let socket: WebSocket | null = null;
    let backoff = 1000;

    function connect() {
      if (stopped) return;
      const token = getToken();
      const url = token ? wsUrl(token) : null;
      if (!url) {
        setTimeout(connect, 2000);
        return;
      }
      socket = new WebSocket(url);
      socket.onopen = () => {
        backoff = 1000;
        setState((s) => ({ ...s, connected: true }));
      };
      socket.onmessage = (event) => {
        try {
          const frame = JSON.parse(event.data as string) as WsFrame;
          setState((s) => reduce(s, frame));
        } catch {
          // ignore malformed frame
        }
      };
      socket.onclose = () => {
        setState((s) => ({ ...s, connected: false }));
        if (!stopped) {
          setTimeout(connect, backoff);
          backoff = Math.min(backoff * 2, 15000);
        }
      };
      socket.onerror = () => socket?.close();
    }

    connect();
    return () => {
      stopped = true;
      socket?.close();
    };
  }, []);

  return <TelemetryContext.Provider value={state}>{children}</TelemetryContext.Provider>;
}
