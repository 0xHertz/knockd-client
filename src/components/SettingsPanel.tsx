import { useState, useEffect } from "react";
import { getSetting, setSetting, detectClients } from "../api";
import type { SshClient } from "../types";

interface Props {
  onClose: () => void;
}

export default function SettingsPanel({ onClose }: Props) {
  const [defaultClient, setDefaultClient] = useState("auto");
  const [defaultDelay, setDefaultDelay] = useState("100");
  const [clients, setClients] = useState<SshClient[]>([]);
  const [customPaths, setCustomPaths] = useState("");
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    (async () => {
      const [dc, dd, cp, cli] = await Promise.all([
        getSetting("default_ssh_client"),
        getSetting("default_knock_delay"),
        getSetting("custom_ssh_paths"),
        detectClients(),
      ]);
      if (dc) setDefaultClient(dc);
      if (dd) setDefaultDelay(dd);
      setClients(cli);
      if (cp) {
        try {
          const arr: { name: string; path: string }[] = JSON.parse(cp);
          setCustomPaths(arr.map((e) => `${e.name}=${e.path}`).join("\n"));
        } catch { /* invalid JSON, ignore */ }
      }
    })();
  }, []);

  const save = async () => {
    const entries = customPaths
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line.includes("="))
      .map((line) => {
        const idx = line.indexOf("=");
        return { name: line.slice(0, idx).trim(), path: line.slice(idx + 1).trim() };
      })
      .filter((e) => e.name && e.path);
    await Promise.all([
      setSetting("default_ssh_client", defaultClient),
      setSetting("default_knock_delay", defaultDelay),
      setSetting("custom_ssh_paths", JSON.stringify(entries)),
    ]);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/40 backdrop-blur-sm">
      <div className="w-full max-w-md mx-4 rounded-2xl bg-slate-900 border border-slate-700/50 shadow-2xl max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between px-5 py-4 border-b border-slate-700/50">
          <h2 className="font-semibold">Settings</h2>
          <button onClick={onClose} className="text-slate-400 hover:text-white text-lg">✕</button>
        </div>

        <div className="p-5 space-y-4">
          <div>
            <label className="block text-xs font-medium text-slate-400 mb-1">
              Default SSH Client
            </label>
            <select
              value={defaultClient}
              onChange={(e) => setDefaultClient(e.target.value)}
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

          <div>
            <label className="block text-xs font-medium text-slate-400 mb-1">
              Default Knock Delay (ms)
            </label>
            <input
              type="number"
              value={defaultDelay}
              onChange={(e) => setDefaultDelay(e.target.value)}
              min={10} step={10}
              className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm
                         focus:outline-none focus:border-emerald-500/50"
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-400 mb-1">
              Detected SSH Clients
            </label>
            <div className="space-y-1 max-h-32 overflow-y-auto">
              {clients.map((c) => (
                <div key={c.name}
                  className="flex items-center justify-between px-2 py-1 rounded text-xs"
                >
                  <span className="text-slate-300">{c.name}</span>
                  <span className={c.installed ? "text-emerald-400" : "text-slate-600"}>
                    {c.installed ? "✓ Installed" : "✗ Not found"}
                  </span>
                </div>
              ))}
            </div>
          </div>

          <div>
            <label className="block text-xs font-medium text-slate-400 mb-1">
              Custom SSH Paths
            </label>
            <p className="text-[10px] text-slate-500 mb-2">
              Format: <code className="text-slate-400">Name=C:\path\to\client.exe</code>, one per line
            </p>
            <textarea
              value={customPaths}
              onChange={(e) => setCustomPaths(e.target.value)}
              rows={4}
              placeholder="MySSH=D:\Tools\ssh.exe"
              className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-xs
                         font-mono focus:outline-none focus:border-emerald-500/50 resize-none"
              spellCheck={false}
            />
          </div>
        </div>

        <div className="flex items-center justify-end gap-2 px-5 py-4 border-t border-slate-700/50">
          <button onClick={onClose}
            className="px-4 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-sm transition-colors"
          >
            Close
          </button>
          <button onClick={save}
            className="px-4 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-sm font-medium
                       transition-colors"
          >
            {saved ? "✓ Saved" : "Save"}
          </button>
        </div>
      </div>
    </div>
  );
}
