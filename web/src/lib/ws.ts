"use client";

// Live telemetry hook. M0 is a stub returning a disconnected state so the
// dashboard renders without a daemon. M1 wires a native WebSocket to
// /api/v1/ws (token passed as a query param, since browsers cannot set WS
// headers), with reconnect backoff and bounded per-series ring buffers feeding
// the sparklines.

import { useEffect, useState } from "react";

import type { SystemSnapshot } from "./types";

export interface Telemetry {
  connected: boolean;
  snapshot: SystemSnapshot | null;
  cpuSeries: number[];
}

const EMPTY: Telemetry = {
  connected: false,
  snapshot: null,
  cpuSeries: [],
};

export function useTelemetry(): Telemetry {
  const [telemetry] = useState<Telemetry>(EMPTY);

  useEffect(() => {
    // M1: open the WebSocket, push frames into state, reconnect on drop.
  }, []);

  return telemetry;
}
