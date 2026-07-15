import { useState, useEffect, FormEvent } from "react";
import type { Connection, SshClient } from "../types";
import { saveConnection, validatePortsJson } from "../api";

interface Props {
  connection: Connection | null;
  clients: SshClient[];
  onSave: () => void;
  onClose: () => void;
}

const emptyForm: Connection = {
  name: "",
  connType: "ssh",
  host: "",
  port: undefined,
  username: "",
  sshClient: "auto",
  knockPorts: '[{"protocol":"udp","port":7000}]',
  knockProtocol: "udp",
  knockDelayMs: 100,
  launchUri: "",
};

const portHints: Record<string, string> = {
  ssh: '[{"protocol":"udp","port":7000},{"protocol":"tcp","port":8000},{"protocol":"udp","port":9000}]',
  web: '[{"protocol":"tcp","port":4444}]',
};

export default function ConnectionForm({ connection, clients, onSave, onClose }: Props) {
  const [form, setForm] = useState<Connection>(connection || emptyForm);
  const [saving, setSaving] = useState(false);
  const [portError, setPortError] = useState("");
  const [showHints, setShowHints] = useState(false);

  useEffect(() => {
    if (connection) setForm(connection);
  }, [connection]);

  const set = (k: keyof Connection, v: unknown) =>
    setForm((f) => ({ ...f, [k]: v }));

  const validatePorts = async (json: string) => {
    set("knockPorts", json);
    if (!json.trim()) { setPortError(""); return; }
    try {
      await validatePortsJson(json);
      setPortError("");
    } catch (e) {
      setPortError(String(e));
    }
  };

  const handleTypeChange = (t: "ssh" | "web") => {
    set("connType", t);
    set("knockPorts", portHints[t] || emptyForm.knockPorts);
    setPortError("");
  };

  const handleSave = async (e: FormEvent) => {
    e.preventDefault();
    if (!form.name.trim() || !form.host.trim()) return;
    setSaving(true);
    try {
      await saveConnection(form);
      onSave();
    } catch (err) {
      setPortError(String(err));
    }
    setSaving(false);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <form
        onSubmit={handleSave}
        className="w-full max-w-lg max-h-[90vh] overflow-y-auto mx-4 rounded-2xl bg-slate-900 border border-slate-700/50 shadow-2xl"
      >
        <div className="flex items-center justify-between px-5 py-4 border-b border-slate-700/50">
          <h2 className="font-semibold text-lg">
            {connection ? "Edit Connection" : "New Connection"}
          </h2>
          <button type="button" onClick={onClose} className="text-slate-400 hover:text-white text-lg">
            ✕
          </button>
        </div>

        <div className="p-5 space-y-4">
          <div className="flex gap-2">
            <button
              type="button"
              onClick={() => handleTypeChange("ssh")}
              className={`flex-1 py-2 rounded-lg text-sm font-medium transition-colors ${
                form.connType === "ssh"
                  ? "bg-blue-600 text-white"
                  : "bg-slate-800 text-slate-400 hover:bg-slate-700"
              }`}
            >
              🖥️ SSH
            </button>
            <button
              type="button"
              onClick={() => handleTypeChange("web")}
              className={`flex-1 py-2 rounded-lg text-sm font-medium transition-colors ${
                form.connType === "web"
                  ? "bg-purple-600 text-white"
                  : "bg-slate-800 text-slate-400 hover:bg-slate-700"
              }`}
            >
              🌐 Web
            </button>
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-400 mb-1">Name</label>
            <input
              type="text"
              value={form.name}
              onChange={(e) => set("name", e.target.value)}
              placeholder="My Server"
              className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm
                         focus:outline-none focus:border-emerald-500/50"
              required
            />
          </div>

          <div className="grid grid-cols-3 gap-3">
            <div className="col-span-2">
              <label className="block text-xs font-medium text-slate-400 mb-1">Host / IP</label>
              <input
                type="text"
                value={form.host}
                onChange={(e) => set("host", e.target.value)}
                placeholder="192.168.1.1 or server.example.com"
                className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm
                           focus:outline-none focus:border-emerald-500/50 font-mono"
                required
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-slate-400 mb-1">Port</label>
              <input
                type="number"
                value={form.port ?? ""}
                onChange={(e) => set("port", e.target.value ? Number(e.target.value) : undefined)}
                placeholder={form.connType === "ssh" ? "22" : "443"}
                className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm
                           focus:outline-none focus:border-emerald-500/50"
              />
            </div>
          </div>

          {form.connType === "ssh" && (
            <>
              <div>
                <label className="block text-xs font-medium text-slate-400 mb-1">Username</label>
                <input
                  type="text"
                  value={form.username || ""}
                  onChange={(e) => set("username", e.target.value || undefined)}
                  placeholder="root"
                  className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm
                             focus:outline-none focus:border-emerald-500/50"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-slate-400 mb-1">SSH Client</label>
                <select
                  value={form.sshClient || "auto"}
                  onChange={(e) => set("sshClient", e.target.value)}
                  className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm
                             focus:outline-none focus:border-emerald-500/50"
                >
                  <option value="auto">Auto-detect</option>
                  {clients
                    .filter((c) => c.installed)
                    .map((c) => (
                      <option key={c.name} value={c.name}>
                        {c.name}
                      </option>
                    ))}
                </select>
              </div>
            </>
          )}

          {form.connType === "web" && (
            <div>
              <label className="block text-xs font-medium text-slate-400 mb-1">Launch URL</label>
              <input
                type="url"
                value={form.launchUri || ""}
                onChange={(e) => set("launchUri", e.target.value || undefined)}
                placeholder={`https://${form.host || "example.com"}`}
                className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm
                           focus:outline-none focus:border-emerald-500/50 font-mono"
              />
            </div>
          )}

          <div>
            <div className="flex items-center justify-between mb-1">
              <label className="text-xs font-medium text-slate-400">Knock Ports (JSON)</label>
              <button
                type="button"
                onClick={() => setShowHints(!showHints)}
                className="text-[10px] text-emerald-400 hover:text-emerald-300"
              >
                {showHints ? "Hide" : "Show"} examples
              </button>
            </div>
            {showHints && (
              <div className="mb-2 space-y-1">
                {[
                  {
                    label: "3-step UDP",
                    json: '[{"protocol":"udp","port":7000},{"protocol":"udp","port":8000},{"protocol":"udp","port":9000}]',
                  },
                  {
                    label: "TCP + UDP mix",
                    json: '[{"protocol":"tcp","port":4444},{"protocol":"udp","port":5555}]',
                  },
                  {
                    label: "Simple single",
                    json: '[{"protocol":"udp","port":12345}]',
                  },
                ].map((h) => (
                  <button
                    key={h.label}
                    type="button"
                    onClick={() => validatePorts(h.json)}
                    className="block w-full text-left text-[10px] px-2 py-1 rounded bg-slate-800
                               text-slate-400 hover:bg-slate-700 hover:text-slate-200 font-mono"
                  >
                    {h.label}: {h.json}
                  </button>
                ))}
              </div>
            )}
            <textarea
              value={form.knockPorts}
              onChange={(e) => validatePorts(e.target.value)}
              rows={3}
              className={`w-full px-3 py-2 rounded-lg bg-slate-800 border text-sm font-mono
                         focus:outline-none transition-colors resize-none ${
                           portError
                             ? "border-red-500/50"
                             : "border-slate-600/50 focus:border-emerald-500/50"
                         }`}
              spellCheck={false}
            />
            {portError && <p className="text-xs text-red-400 mt-1">{portError}</p>}
          </div>

          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="block text-xs font-medium text-slate-400 mb-1">Protocol</label>
              <select
                value={form.knockProtocol}
                onChange={(e) => set("knockProtocol", e.target.value)}
                className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm
                           focus:outline-none focus:border-emerald-500/50"
              >
                <option value="udp">UDP</option>
                <option value="tcp">TCP</option>
              </select>
            </div>
            <div>
              <label className="block text-xs font-medium text-slate-400 mb-1">Delay (ms)</label>
              <input
                type="number"
                value={form.knockDelayMs}
                onChange={(e) => set("knockDelayMs", Math.max(10, Number(e.target.value) || 100))}
                min={10}
                step={10}
                className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm
                           focus:outline-none focus:border-emerald-500/50"
              />
            </div>
          </div>
        </div>

        <div className="flex items-center justify-end gap-2 px-5 py-4 border-t border-slate-700/50">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-sm transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={saving || !!portError}
            className="px-6 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-sm font-medium
                       transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {saving ? "Saving..." : connection ? "Update" : "Save"}
          </button>
        </div>
      </form>
    </div>
  );
}
