"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

import { useTelemetry } from "@/lib/ws";

import { StatusPill } from "./status-pill";
import { ThemeToggle } from "./theme-toggle";

const NAV = [
  { href: "/", label: "Overview" },
  { href: "/power", label: "Power" },
  { href: "/system", label: "System" },
  { href: "/services", label: "Services" },
];

export function Sidebar() {
  const pathname = usePathname();
  const { connected, snapshot } = useTelemetry();
  const status = connected ? (snapshot?.presence ?? "online") : "offline";
  const label = connected ? status : "disconnected";

  return (
    <aside
      className="flex w-60 shrink-0 flex-col justify-between border-r p-5"
      style={{ borderColor: "var(--color-border)", background: "var(--color-surface)" }}
    >
      <div>
        <div className="mb-8 flex items-center gap-2">
          <Mark />
          <span className="text-lg font-semibold tracking-tight">WienerEnvoy</span>
        </div>
        <nav className="flex flex-col gap-1">
          {NAV.map((item) => {
            const active = pathname === item.href;
            return (
              <Link
                key={item.href}
                href={item.href}
                className="rounded-md px-3 py-2 text-sm transition-colors"
                style={{
                  background: active ? "var(--color-surface-2)" : "transparent",
                  color: active ? "var(--color-text)" : "var(--color-muted)",
                }}
              >
                {item.label}
              </Link>
            );
          })}
        </nav>
      </div>

      <div className="flex flex-col gap-3">
        <StatusPill status={status} label={label} />
        <ThemeToggle />
      </div>
    </aside>
  );
}

function Mark() {
  return (
    <svg width="22" height="22" viewBox="0 0 24 24" fill="none" aria-hidden>
      <rect x="2" y="2" width="20" height="20" rx="6" stroke="var(--color-text)" strokeWidth="1.6" />
      <circle cx="12" cy="12" r="3.2" fill="var(--color-online)" />
    </svg>
  );
}
