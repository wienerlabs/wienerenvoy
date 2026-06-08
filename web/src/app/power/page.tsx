import { StatCard } from "@/components/stat-card";

export default function PowerPage() {
  return (
    <div className="container-page py-8">
      <header className="mb-8">
        <h1 className="text-3xl font-light tracking-tight">Power</h1>
        <p className="mt-1 text-sm" style={{ color: "var(--color-muted)" }}>
          Control presence and machine power. Sleep is recoverable over the tailnet;
          shutdown leaves only Wake-on-LAN and needs confirmation.
        </p>
      </header>

      <section className="mb-8">
        <div className="chip mb-4">Presence</div>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <StatCard label="Keep awake" value="--" hint="Available in M1" />
          <StatCard label="Backend" value="caffeinate" hint="Engaged in M1" />
          <StatCard label="Since" value="--" />
        </div>
      </section>

      <section>
        <div className="chip mb-4">Machine</div>
        <div className="flex flex-wrap gap-3">
          <button type="button" className="btn" disabled>
            Sleep
          </button>
          <button type="button" className="btn" disabled>
            Restart
          </button>
          <button type="button" className="btn btn-danger" disabled>
            Shutdown
          </button>
        </div>
        <p className="mt-3 text-xs" style={{ color: "var(--color-muted)" }}>
          Available in M1. Service-level and server-level controls arrive with M2.
        </p>
      </section>
    </div>
  );
}
