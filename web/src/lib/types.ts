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

export type RecoveryPath =
  | "tailnet_wake"
  | "wol"
  | "scheduled_wake"
  | "launchd_relaunch"
  | "wol_only";

export interface ActionAccepted {
  accepted: boolean;
  action: string;
  recoverableVia: RecoveryPath[];
  warning: string | null;
}

// WebSocket frames are internally tagged by `type`; the Metrics/StateChanged/
// PresenceChanged variants flatten their payload alongside the tag.
export type WsFrame =
  | { type: "hello"; version: string; sampleIntervalMs: number }
  | ({ type: "metrics" } & SystemSnapshot)
  | ({ type: "state_changed" } & ServerState)
  | ({ type: "presence_changed" } & PresenceState);

export interface DockerStatus {
  available: boolean;
  version: string | null;
}

export interface ServiceContainer {
  id: string;
  name: string;
  image: string;
  state: string;
  status: string;
}

export interface StackSummary {
  name: string;
  running: number;
  total: number;
  containers: ServiceContainer[];
}

export interface ServicesView {
  docker: DockerStatus;
  stacks: StackSummary[];
  standalone: ServiceContainer[];
}
