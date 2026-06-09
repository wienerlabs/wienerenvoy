// Minimal fetch client for the daemon API. The base URL is empty when the
// daemon serves this dashboard (same-origin), or set via NEXT_PUBLIC_DAEMON_URL
// in dev. The bearer token lives in localStorage under `wienerenvoy:token`.

import type {
  ControlResult,
  Health,
  LogLines,
  PowerAction,
  PresenceState,
  ServerState,
  ServiceAction,
  ServicesView,
  SystemSnapshot,
} from "./types";

const BASE = process.env.NEXT_PUBLIC_DAEMON_URL ?? "";

export class ApiError extends Error {
  status: number;
  constructor(message: string, status: number) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

const TOKEN_KEY = "wienerenvoy:token";

export function getToken(): string | null {
  if (typeof window === "undefined") return null;
  return window.localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string): void {
  if (typeof window !== "undefined") window.localStorage.setItem(TOKEN_KEY, token);
}

export function hasToken(): boolean {
  return getToken() !== null;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const token = getToken();
  const res = await fetch(`${BASE}${path}`, {
    ...init,
    headers: {
      "content-type": "application/json",
      ...(token ? { authorization: `Bearer ${token}` } : {}),
      ...(init?.headers ?? {}),
    },
  });
  if (res.status === 401) throw new ApiError("unauthorized", 401);
  if (!res.ok) {
    const body = await res.text().catch(() => res.statusText);
    throw new ApiError(body || res.statusText, res.status);
  }
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

export const api = {
  health: () => request<Health>("/health"),
  state: () => request<ServerState>("/api/v1/state"),
  metrics: () => request<SystemSnapshot>("/api/v1/system/metrics"),
  presence: () => request<PresenceState>("/api/v1/presence"),
  setKeepAwake: (enabled: boolean) =>
    request<ServerState>("/api/v1/presence/keep-awake", {
      method: "POST",
      body: JSON.stringify({ enabled }),
    }),
  power: (action: PowerAction, confirm = false) =>
    request<unknown>(`/api/v1/power/${action}`, {
      method: "POST",
      body: JSON.stringify({ confirm }),
    }),
  serverLevel: (on: boolean) =>
    request<ServerState>(`/api/v1/power/server/${on ? "on" : "off"}`, {
      method: "POST",
    }),
  scheduleWake: (relativeSecs: number) =>
    request<unknown>("/api/v1/power/wake-schedule", {
      method: "POST",
      body: JSON.stringify({ relativeSecs }),
    }),
  services: () => request<ServicesView>("/api/v1/services"),
  serviceControl: (kind: "stack" | "container", id: string, action: ServiceAction) =>
    request<ControlResult>("/api/v1/services/control", {
      method: "POST",
      body: JSON.stringify({ kind, id, action }),
    }),
  serviceLogs: (id: string, tail = 200) =>
    request<LogLines>(`/api/v1/services/logs?id=${encodeURIComponent(id)}&tail=${tail}`),
};
