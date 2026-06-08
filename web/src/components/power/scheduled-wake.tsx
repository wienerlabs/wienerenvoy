"use client";

import { useState } from "react";

import { api } from "@/lib/api";

export function ScheduledWake() {
  const [minutes, setMinutes] = useState("60");
  const [message, setMessage] = useState<string | null>(null);

  async function schedule() {
    const mins = Number.parseInt(minutes, 10);
    if (!Number.isFinite(mins) || mins <= 0) {
      setMessage("enter a positive number of minutes");
      return;
    }
    try {
      await api.scheduleWake(mins * 60);
      setMessage(`wake scheduled in ${mins} min`);
    } catch (err) {
      setMessage(`failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  return (
    <div className="flex flex-wrap items-center gap-3">
      <div className="input flex items-center gap-2 px-3 py-2">
        <input
          className="num w-16 bg-transparent outline-none"
          type="number"
          min="1"
          value={minutes}
          onChange={(e) => setMinutes(e.target.value)}
        />
        <span className="text-xs" style={{ color: "var(--color-muted)" }}>
          min
        </span>
      </div>
      <button type="button" className="btn" onClick={schedule}>
        Schedule wake
      </button>
      {message ? (
        <span className="text-xs" style={{ color: "var(--color-muted)" }}>
          {message}
        </span>
      ) : null}
    </div>
  );
}
