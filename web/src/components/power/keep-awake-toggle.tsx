"use client";

import { useState } from "react";

import { api } from "@/lib/api";
import { useTelemetry } from "@/lib/ws";

export function KeepAwakeToggle() {
  const { presence } = useTelemetry();
  const [busy, setBusy] = useState(false);
  const engaged = presence?.keepAwake ?? false;

  async function toggle() {
    setBusy(true);
    try {
      await api.setKeepAwake(!engaged);
    } catch {
      // The next presence frame will reconcile the displayed state.
    } finally {
      setBusy(false);
    }
  }

  return (
    <button
      type="button"
      className={engaged ? "btn btn-primary" : "btn"}
      disabled={busy}
      onClick={toggle}
    >
      Keep awake: {engaged ? "on" : "off"}
    </button>
  );
}
