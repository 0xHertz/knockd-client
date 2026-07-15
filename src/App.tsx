import { useState, useEffect, useCallback } from "react";
import type { Connection, SshClient } from "./types";
import { listConnections, deleteConnection, knockAndConnect, detectClients } from "./api";
import ConnectionCard from "./components/ConnectionCard";
import ConnectionForm from "./components/ConnectionForm";
import SettingsPanel from "./components/SettingsPanel";

export default function App() {
  const [connections, setConnections] = useState<Connection[]>([]);
  const [clients, setClients] = useState<SshClient[]>([]);
  const [search, setSearch] = useState("");
  const [filter, setFilter] = useState<"all" | "ssh" | "web">("all");
  const [editing, setEditing] = useState<Connection | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [status, setStatus] = useState<{ msg: string; ok: boolean } | null>(null);
  const [connecting, setConnecting] = useState<number | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [conns, clis] = await Promise.all([listConnections(), detectClients()]);
      setConnections(conns);
      setClients(clis);
    } catch (e) {
      showStatus(`Error: ${e}`, false);
    }
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  const showStatus = (msg: string, ok: boolean) => {
    setStatus({ msg, ok });
    setTimeout(() => setStatus(null), 4000);
  };

  const handleConnect = async (id: number) => {
    setConnecting(id);
    try {
      const msg = await knockAndConnect(id);
      showStatus(msg, true);
    } catch (e) {
      showStatus(String(e), false);
    }
    setConnecting(null);
  };

  const handleDelete = async (id: number) => {
    try {
      await deleteConnection(id);
      setConnections((p) => p.filter((c) => c.id !== id));
      showStatus("Deleted", true);
    } catch (e) {
      showStatus(String(e), false);
    }
  };

  const handleSave = async () => {
    setShowForm(false);
    setEditing(null);
    await refresh();
    showStatus("Saved", true);
  };

  const filtered = connections.filter((c) => {
    if (filter !== "all" && c.connType !== filter) return false;
    if (search) {
      const q = search.toLowerCase();
      return (
        c.name.toLowerCase().includes(q) ||
        c.host.toLowerCase().includes(q) ||
        (c.username || "").toLowerCase().includes(q)
      );
    }
    return true;
  });

  return (
    <div className="h-screen flex flex-col">
      <header className="flex items-center justify-between px-5 py-3 border-b border-slate-700/50 bg-slate-900/80">
        <div className="flex items-center gap-3">
          <span className="text-xl">🔐</span>
          <h1 className="text-lg font-semibold tracking-tight">Knockd Client</h1>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => setShowSettings(!showSettings)}
            className="px-3 py-1.5 text-sm rounded-lg bg-slate-700/50 hover:bg-slate-600/50 transition-colors"
          >
            ⚙️ Settings
          </button>
          <button
            onClick={() => { setEditing(null); setShowForm(true); }}
            className="px-4 py-1.5 text-sm rounded-lg bg-emerald-600 hover:bg-emerald-500 transition-colors font-medium"
          >
            + New Connection
          </button>
        </div>
      </header>

      {status && (
        <div
          className={`px-5 py-2 text-sm font-medium text-center transition-opacity ${
            status.ok ? "bg-emerald-900/60 text-emerald-300" : "bg-red-900/60 text-red-300"
          }`}
        >
          {status.msg}
        </div>
      )}

      {showSettings && <SettingsPanel onClose={() => { setShowSettings(false); refresh(); }} />}

      <div className="flex items-center gap-3 px-5 py-3 border-b border-slate-700/30">
        <input
          type="text"
          placeholder="Search connections..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="flex-1 px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm
                     placeholder-slate-500 focus:outline-none focus:border-emerald-500/50 transition-colors"
        />
        <div className="flex rounded-lg overflow-hidden border border-slate-600/50">
          {(["all", "ssh", "web"] as const).map((f) => (
            <button
              key={f}
              onClick={() => setFilter(f)}
              className={`px-3 py-2 text-xs font-medium uppercase transition-colors ${
                filter === f
                  ? "bg-emerald-600 text-white"
                  : "bg-slate-800 text-slate-400 hover:bg-slate-700"
              }`}
            >
              {f}
            </button>
          ))}
        </div>
      </div>

      <main className="flex-1 overflow-y-auto p-4">
        {filtered.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-slate-500 gap-3">
            <span className="text-4xl">📡</span>
            <p className="text-sm">
              {connections.length === 0
                ? "No connections yet. Add your first one!"
                : "No matching connections."}
            </p>
            {connections.length === 0 && (
              <button
                onClick={() => { setEditing(null); setShowForm(true); }}
                className="px-4 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-sm font-medium transition-colors"
              >
                + Add Connection
              </button>
            )}
          </div>
        ) : (
          <div className="grid gap-3 grid-cols-1 md:grid-cols-2 xl:grid-cols-3">
            {filtered.map((c) => (
              <ConnectionCard
                key={c.id}
                connection={c}
                onConnect={handleConnect}
                onEdit={(conn) => { setEditing(conn); setShowForm(true); }}
                onDelete={handleDelete}
                connecting={connecting === c.id}
              />
            ))}
          </div>
        )}
      </main>

      {showForm && (
        <ConnectionForm
          connection={editing}
          clients={clients}
          onSave={handleSave}
          onClose={() => { setShowForm(false); setEditing(null); }}
        />
      )}
    </div>
  );
}
