"use client";

import { useEffect, useState } from "react";

type Pref = "light" | "dark" | "system";

function readPref(): Pref {
  if (typeof document === "undefined") return "system";
  const p = document.documentElement.getAttribute("data-theme-pref");
  if (p === "light" || p === "dark") return p;
  return "system";
}

function systemTheme(): "light" | "dark" {
  if (typeof window === "undefined") return "dark";
  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

function applyPref(pref: Pref) {
  const resolved = pref === "system" ? systemTheme() : pref;
  document.documentElement.setAttribute("data-theme", resolved);
  document.documentElement.setAttribute("data-theme-pref", pref);
  if (pref === "system") {
    localStorage.removeItem("wienerenvoy:theme");
  } else {
    localStorage.setItem("wienerenvoy:theme", pref);
  }
}

export function ThemeToggle() {
  const [pref, setPref] = useState<Pref>("system");
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setPref(readPref());
    setMounted(true);
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onSystemChange = () => {
      if (readPref() === "system") {
        applyPref("system");
      }
    };
    mq.addEventListener("change", onSystemChange);
    return () => mq.removeEventListener("change", onSystemChange);
  }, []);

  function set(p: Pref) {
    setPref(p);
    applyPref(p);
  }

  return (
    <div
      role="radiogroup"
      aria-label="Theme"
      className="inline-flex items-center gap-0.5 rounded-full border p-0.5 text-[var(--color-muted)]"
      style={{
        borderColor: "var(--color-border)",
        background: "var(--color-surface-2)",
      }}
    >
      <Btn active={mounted && pref === "light"} onClick={() => set("light")} aria-label="Light theme">
        <Sun />
      </Btn>
      <Btn active={mounted && pref === "system"} onClick={() => set("system")} aria-label="System theme">
        <Auto />
      </Btn>
      <Btn active={mounted && pref === "dark"} onClick={() => set("dark")} aria-label="Dark theme">
        <Moon />
      </Btn>
    </div>
  );
}

function Btn({
  active,
  onClick,
  children,
  "aria-label": ariaLabel,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
  "aria-label": string;
}) {
  return (
    <button
      type="button"
      role="radio"
      aria-checked={active}
      aria-label={ariaLabel}
      onClick={onClick}
      className="h-7 w-7 inline-flex items-center justify-center rounded-full transition-colors"
      style={{
        background: active ? "var(--color-text)" : "transparent",
        color: active ? "var(--color-bg)" : "currentColor",
      }}
    >
      {children}
    </button>
  );
}

function Sun() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <circle cx="12" cy="12" r="4" />
      <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
    </svg>
  );
}

function Moon() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
    </svg>
  );
}

function Auto() {
  return (
    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" aria-hidden>
      <circle cx="12" cy="12" r="8" />
      <path d="M12 4a8 8 0 0 0 0 16" fill="currentColor" />
    </svg>
  );
}
