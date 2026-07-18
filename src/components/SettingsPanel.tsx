import { useState, useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { getSetting, setSetting, detectClients, adminEncryptBlob, listConnections } from "../api";
import type { SshClient, Connection } from "../types";

interface Props { onClose: () => void; }
interface CustomEntry { name: string; path: string; }

export default function SettingsPanel({ onClose }: Props) {
  const [defaultClient, setDefaultClient] = useState("auto");
  const [defaultDelay, setDefaultDelay] = useState("100");
  const [clients, setClients] = useState<SshClient[]>([]);
  const [customEntries, setCustomEntries] = useState<CustomEntry[]>([]);
  const [saved, setSaved] = useState(false);
  const [spaSites, setSpaSites] = useState<Connection[]>([]);
  const [adminSiteId, setAdminSiteId] = useState("");
  const [adminUserPub, setAdminUserPub] = useState("");
  const [adminOutput, setAdminOutput] = useState("");

  useEffect(() => {
    (async () => {
      const [dc, dd, cp, cli, conns] = await Promise.all([
        getSetting("default_ssh_client"), getSetting("default_knock_delay"),
        getSetting("custom_ssh_paths"), detectClients(), listConnections(),
      ]);
      if (dc) setDefaultClient(dc);
      if (dd) setDefaultDelay(dd);
      setClients(cli);
      if (cp) { try { setCustomEntries(JSON.parse(cp)); } catch { /* ignore */ } }
      setSpaSites(conns.filter((c: Connection) => c.authMethod === "knockpass"));
    })();
  }, []);

  const save = async () => {
    const valid = customEntries.filter((e) => e.name.trim() && e.path.trim());
    await Promise.all([
      setSetting("default_ssh_client", defaultClient), setSetting("default_knock_delay", defaultDelay),
      setSetting("custom_ssh_paths", JSON.stringify(valid)),
    ]);
    setSaved(true); setTimeout(() => setSaved(false), 2000);
  };

  const addEntry = () => setCustomEntries([...customEntries, { name: "", path: "" }]);
  const removeEntry = (idx: number) => setCustomEntries(customEntries.filter((_, i) => i !== idx));
  const updateEntry = (idx: number, field: keyof CustomEntry, value: string) => {
    const updated = [...customEntries]; updated[idx] = { ...updated[idx], [field]: value }; setCustomEntries(updated);
  };
  const browseFile = async (idx: number) => {
    const selected = await open({ multiple: false, filters: [{ name: "Executable", extensions: ["exe", "bat", "cmd", "com"] }] });
    if (selected) {
      const fp = typeof selected === "string" ? selected : (selected as { path: string }).path;
      const autoName = fp.replace(/^.*[\\/]/, "").replace(/\.[^.]+$/, "");
      updateEntry(idx, "path", fp);
      if (!customEntries[idx].name) updateEntry(idx, "name", autoName);
    }
  };

  const handleAdminEncrypt = async () => {
    if (!adminSiteId || !adminUserPub.trim()) return;
    try { const blob = await adminEncryptBlob(adminSiteId, adminUserPub.trim()); setAdminOutput(blob); }
    catch (e) { setAdminOutput("Error: " + e); }
  };

  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-black/40 backdrop-blur-sm">
      <div className="w-full max-w-md mx-4 rounded-2xl bg-slate-900 border border-slate-700/50 shadow-2xl max-h-[90vh] overflow-y-auto">
        <div className="flex items-center justify-between px-5 py-4 border-b border-slate-700/50">
          <h2 className="font-semibold">Settings</h2>
          <button onClick={onClose} className="text-slate-400 hover:text-white text-lg">✕</button>
        </div>
        <div className="p-5 space-y-4">
          <div><label className="block text-xs font-medium text-slate-400 mb-1">Default SSH Client</label>
            <select value={defaultClient} onChange={(e) => setDefaultClient(e.target.value)} className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm focus:outline-none focus:border-emerald-500/50">
              <option value="auto">Auto-detect</option>
              {clients.filter((c) => c.installed).map((c) => (<option key={c.name} value={c.name}>{c.name}</option>))}
            </select></div>
          <div><label className="block text-xs font-medium text-slate-400 mb-1">Default Knock Delay (ms)</label>
            <input type="number" value={defaultDelay} onChange={(e) => setDefaultDelay(e.target.value)} min={10} step={10} className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm focus:outline-none focus:border-emerald-500/50" /></div>
          <div><label className="block text-xs font-medium text-slate-400 mb-1">Detected SSH Clients</label>
            <div className="space-y-1 max-h-32 overflow-y-auto">
              {clients.map((c) => (<div key={c.name} className="flex items-center justify-between px-2 py-1 rounded text-xs"><span className="text-slate-300">{c.name}</span><span className={c.installed ? "text-emerald-400" : "text-slate-600"}>{c.installed ? "✓" : "✗"}</span></div>))}
            </div></div>
          <div><div className="flex items-center justify-between mb-2"><label className="text-xs font-medium text-slate-400">Custom SSH Clients</label>
            <button onClick={addEntry} className="text-xs px-2 py-1 rounded bg-slate-700 hover:bg-slate-600 text-emerald-400 transition-colors">+ Add</button></div>
            <div className="space-y-2">
              {customEntries.map((entry, i) => (<div key={i} className="flex items-center gap-2">
                <input value={entry.name} onChange={(e) => updateEntry(i, "name", e.target.value)} placeholder="Name" className="w-28 px-2 py-1.5 rounded bg-slate-800 border border-slate-600/50 text-xs focus:outline-none focus:border-emerald-500/50" />
                <input value={entry.path} readOnly placeholder="Click 📂 to select" className="flex-1 px-2 py-1.5 rounded bg-slate-800 border border-slate-600/50 text-xs font-mono focus:outline-none truncate" />
                <button onClick={() => browseFile(i)} className="px-2 py-1.5 rounded bg-slate-700 hover:bg-slate-600 text-xs transition-colors shrink-0">📂</button>
                <button onClick={() => removeEntry(i)} className="px-2 py-1.5 rounded bg-slate-700 hover:bg-red-900/50 text-xs transition-colors shrink-0">✕</button>
              </div>))}
            </div></div>
          <div className="border-t border-slate-700/50 pt-4">
            <label className="text-xs font-medium text-slate-400 mb-2 block">🔐 Admin: Encrypt Site Key for User</label>
            <div className="space-y-2">
              <select value={adminSiteId} onChange={e => setAdminSiteId(e.target.value)} className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-xs focus:outline-none focus:border-amber-500/50">
                <option value="">Select SPA site...</option>
                {spaSites.map(s => (<option key={s.id} value={s.spaSiteId || s.name}>{s.name} ({s.spaSiteId})</option>))}
              </select>
              <textarea value={adminUserPub} onChange={e => setAdminUserPub(e.target.value)} rows={2} placeholder="User X25519 Public Key (64 hex)" className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-xs font-mono focus:outline-none focus:border-amber-500/50 resize-none" spellCheck={false} />
              <button onClick={handleAdminEncrypt} className="w-full py-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-xs font-medium transition-colors">Encrypt for User</button>
              {adminOutput && <textarea value={adminOutput} readOnly rows={3} className="w-full px-3 py-2 rounded-lg bg-slate-800/50 border border-slate-600/50 text-xs font-mono text-slate-300 resize-none" />}
            </div>
          </div>
        </div>
        <div className="flex items-center justify-end gap-2 px-5 py-4 border-t border-slate-700/50">
          <button onClick={onClose} className="px-4 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-sm transition-colors">Close</button>
          <button onClick={save} className="px-4 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-sm font-medium transition-colors">{saved ? "✓ Saved" : "Save"}</button>
        </div>
      </div>
    </div>
  );
}
