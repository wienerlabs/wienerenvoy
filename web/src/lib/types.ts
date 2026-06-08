// Wire types mirroring the daemon's serde structs (camelCase on the wire).

export type Presence = "online" | "standby" | "offline";
export type ServerLevel = "active" | "server_off";
export type PowerAction = "sleep" | "restart" | "shutdown";

export interface Health {
  status: string;
  version: string;
}

export interface ServerState {
  level: ServerLevel;
  presence: Presence;
  since: string;
  recoverableVia: string[];
}

export interface CpuUsage {
  usagePct: number;
  cores: number;
  perCore: number[];
}

export interface MemoryUsage {
  usedBytes: number;
  totalBytes: number;
  availableBytes: number;
  swapUsedBytes: number;
  swapTotalBytes: number;
}

export interface DiskUsage {
  mount: string;
  usedBytes: number;
  totalBytes: number;
}

export interface NetworkRate {
  rxBytesPerSec: number;
  txBytesPerSec: number;
}

export interface TempSensor {
  label: string;
  celsius: number;
}

export interface SystemSnapshot {
  capturedAt: string;
  presence: Presence;
  uptimeSecs: number;
  cpu: CpuUsage;
  memory: MemoryUsage;
  disks: DiskUsage[];
  network: NetworkRate;
  loadAvg: [number, number, number];
  temperatures: TempSensor[];
}

export interface PresenceState {
  keepAwake: boolean;
  backend: string;
  holderPid: number | null;
  since: string;
}
