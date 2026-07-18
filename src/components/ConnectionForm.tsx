import { useState, useEffect, FormEvent } from "react";
import type { Connection, SshClient } from "../types";
import { saveConnection, validatePortsJson, generateSiteKeys, spaEncrypt, storeEncryptedKey, getX25519Identity, enrollUserImport, setSetting } from "../api";

interface Props {
  connection: Connection | null;
  clients: SshClient[];
  onSave: () => void;
  onClose: () => void;
}

const emptyForm: Connection = {
  name: "", connType: "ssh", host: "", port: undefined, username: "",
  sshClient: "auto", knockPorts: '[{"protocol":"udp","port":7000}]',
  knockProtocol: "udp", knockDelayMs: 100, launchUri: "",
  authMethod: "knockd", spaSiteId: undefined, spaCredential: undefined, spaUdpPort: undefined,
};

const portHints: Record<string, string> = {
  ssh: '[{"protocol":"udp","port":7000},{"protocol":"tcp","port":8000},{"protocol":"udp","port":9000}]',
  web: '[{"protocol":"tcp","port":4444}]',
};

export default function ConnectionForm({ connection, clients, onSave, onClose }: Props) {
  const [form, setForm] = useState<Connection>(connection || emptyForm);
  const [spaMode, setSpaMode] = useState<"admin" | "user">("admin");
  const [adminPubKey, setAdminPubKey] = useState("");
  const [adminPrivKey, setAdminPrivKey] = useState("");
  const [webProtocol, setWebProtocol] = useState<"https" | "http">("https");
  const [spaResult, setSpaResult] = useState("");
  const [userPubKey, setUserPubKey] = useState("");
  const [importBlob, setImportBlob] = useState("");
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState("");
  const [showHints, setShowHints] = useState(false);
  const isKnockPass = (form.authMethod || "knockd") === "knockpass";

  const handleEnrollAdmin = async () => {
    if (!form.spaSiteId) return;
    try {
      const r = await generateSiteKeys();
      const parsed = JSON.parse(r);
      setAdminPubKey(parsed.public_key);
      setAdminPrivKey(parsed.private_key);
      setSpaResult("Keys generated. Will be saved when you click Save.");
    } catch (e) { setSpaResult(`Error: ${e}`); }
  };

  const loadUserIdentity = async () => {
    try {
      const [xpub] = await getX25519Identity();
      setUserPubKey(xpub);
    } catch { /* ignore */ }
  };

  const handleEnrollUserGen = async () => {
    try {
      const [xpub] = await getX25519Identity();
      setUserPubKey(xpub);
      setSpaResult("Send this public key to admin.");
    } catch (e) { setSpaResult(`Error: ${e}`); }
  };

  const handleEnrollUserImport = async () => {
    if (!importBlob.trim()) { setSpaResult("Paste the encrypted blob from admin first."); return; }
    try {
      setSpaResult("Decrypting...");
      const r = await enrollUserImport(importBlob.trim());
      setAdminPrivKey(r);
      setSpaResult("Key decrypted. Will be saved when you click Save.");
      setImportBlob("");
    } catch (e) { setSpaResult("Decrypt error: " + String(e)); }
  };

  useEffect(() => { if (connection) setForm(connection); }, [connection]);

  const set = (k: keyof Connection, v: unknown) => setForm((f) => ({ ...f, [k]: v }));
  const validatePorts = async (json: string) => { set("knockPorts", json); if (!json.trim()) { setSaveError(""); return; } try { await validatePortsJson(json); setSaveError(""); } catch (e) { setSaveError(String(e)); } };
  const handleTypeChange = (t: "ssh" | "web") => { set("connType", t); set("knockPorts", portHints[t] || emptyForm.knockPorts); setSaveError(""); };
  const handleSave = async (e: FormEvent) => {
    e.preventDefault();
    if (!form.name.trim() || !form.host.trim()) return;
    setSaving(true);
    try {
      const data = { ...form };
      if (data.connType === "web" && !data.launchUri) {
        const portStr = data.port ? `:${data.port}` : "";
        data.launchUri = `${webProtocol}://${data.host}${portStr}`;
      }
      // Save site key to keyring when Save is clicked
      if (data.authMethod === "knockpass") {
        if (!adminPrivKey) {
          setSaveError("No private key! Click 'Generate Keys' first.");
          setSaving(false);
          return;
        }
        try {
          const encrypted = await spaEncrypt(adminPrivKey);
          await storeEncryptedKey(data.spaSiteId!, encrypted);
          await setSetting(`kp_${data.spaSiteId}_origin`, spaMode);
          data.spaCredential = `kp_${data.spaSiteId}_priv`;
          setSaveError("");
        } catch (e) {
          setSaveError("Keyring save failed: " + String(e));
          setSaving(false);
          return;
        }
      }
      await saveConnection(data);
      onSave();
    } catch (err) { setSaveError(String(err)); }
    setSaving(false);
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm">
      <form onSubmit={handleSave} className="w-full max-w-lg max-h-[90vh] overflow-y-auto mx-4 rounded-2xl bg-slate-900 border border-slate-700/50 shadow-2xl">
        <div className="flex items-center justify-between px-5 py-4 border-b border-slate-700/50">
          <h2 className="font-semibold text-lg">{connection ? "Edit Connection" : "New Connection"}</h2>
          <button type="button" onClick={onClose} className="text-slate-400 hover:text-white text-lg">✕</button>
        </div>

        <div className="p-5 space-y-4">
          <div className="flex gap-2">
            <button type="button" onClick={() => handleTypeChange("ssh")} className={`flex-1 py-2 rounded-lg text-sm font-medium transition-colors ${form.connType === "ssh" ? "bg-blue-600 text-white" : "bg-slate-800 text-slate-400 hover:bg-slate-700"}`}>🖥️ SSH</button>
            <button type="button" onClick={() => handleTypeChange("web")} className={`flex-1 py-2 rounded-lg text-sm font-medium transition-colors ${form.connType === "web" ? "bg-purple-600 text-white" : "bg-slate-800 text-slate-400 hover:bg-slate-700"}`}>🌐 Web</button>
          </div>

          <div><label className="block text-xs font-medium text-slate-400 mb-1">Name</label><input type="text" value={form.name} onChange={(e) => set("name", e.target.value)} placeholder="My Server" className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm focus:outline-none focus:border-emerald-500/50" required /></div>

          <div className="grid grid-cols-3 gap-3">
            <div className="col-span-2"><label className="block text-xs font-medium text-slate-400 mb-1">Host / IP</label><input type="text" value={form.host} onChange={(e) => set("host", e.target.value)} placeholder="192.168.1.1" className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm focus:outline-none focus:border-emerald-500/50 font-mono" required /></div>
            <div><label className="block text-xs font-medium text-slate-400 mb-1">Port</label><input type="number" value={form.port ?? ""} onChange={(e) => set("port", e.target.value ? Number(e.target.value) : undefined)} placeholder={form.connType === "ssh" ? "22" : "443"} className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm focus:outline-none focus:border-emerald-500/50" /></div>
          </div>

          {form.connType === "ssh" && (<><div><label className="block text-xs font-medium text-slate-400 mb-1">Username</label><input type="text" value={form.username || ""} onChange={(e) => set("username", e.target.value || undefined)} placeholder="root" className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm focus:outline-none focus:border-emerald-500/50" /></div>
          <div><label className="block text-xs font-medium text-slate-400 mb-1">SSH Client</label><select value={form.sshClient || "auto"} onChange={(e) => set("sshClient", e.target.value)} className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm focus:outline-none focus:border-emerald-500/50"><option value="auto">Auto-detect</option>{clients.filter((c) => c.installed).map((c) => (<option key={c.name} value={c.name}>{c.name}</option>))}</select></div></>)}

          {form.connType === "web" && (<div className="flex items-end gap-2"><div className="w-20"><label className="block text-xs font-medium text-slate-400 mb-1">Protocol</label><select value={webProtocol} onChange={(e) => setWebProtocol(e.target.value as "https"|"http")} className="w-full px-2 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm focus:outline-none focus:border-purple-500/50"><option value="https">https://</option><option value="http">http://</option></select></div><div className="flex-1"><label className="block text-xs font-medium text-slate-400 mb-1">URL Preview</label><input type="text" value={`${webProtocol}://${form.host || "example.com"}${form.port ? ":" + form.port : ""}`} readOnly className="w-full px-3 py-2 rounded-lg bg-slate-800/50 border border-slate-600/50 text-sm focus:outline-none text-slate-400 font-mono" /></div></div>)}

          <div className="flex gap-2">
            <button type="button" onClick={() => { set("authMethod", "knockd"); set("knockPorts", portHints[form.connType] || emptyForm.knockPorts); }} className={`flex-1 py-2 rounded-lg text-xs font-medium transition-colors ${!isKnockPass ? "bg-emerald-600 text-white" : "bg-slate-800 text-slate-400 hover:bg-slate-700"}`}>🔓 Port Knocking</button>
            <button type="button" onClick={() => { set("authMethod", "knockpass"); set("knockPorts", ""); if (!form.spaSiteId) set("spaSiteId", form.name || ""); }} className={`flex-1 py-2 rounded-lg text-xs font-medium transition-colors ${isKnockPass ? "bg-amber-600 text-white" : "bg-slate-800 text-slate-400 hover:bg-slate-700"}`}>🔐 KnockPass SPA</button>
          </div>

          {isKnockPass ? (
            <div className="space-y-3">
              <div><label className="block text-xs font-medium text-slate-400 mb-1">Site ID</label><input type="text" value={form.spaSiteId || ""} onChange={(e) => set("spaSiteId", e.target.value || undefined)} placeholder="my-site" className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm focus:outline-none focus:border-amber-500/50" /></div>

              <div className="flex rounded-lg overflow-hidden border border-slate-600/50">
                <button type="button" onClick={() => { setSpaMode("admin"); setSpaResult(""); }} className={`flex-1 py-1.5 text-xs font-medium transition-colors ${spaMode === "admin" ? "bg-amber-600/30 text-amber-300" : "bg-slate-800 text-slate-400 hover:text-slate-300"}`}>Admin</button>
                <button type="button" onClick={() => { setSpaMode("user"); setSpaResult(""); loadUserIdentity(); }} className={`flex-1 py-1.5 text-xs font-medium transition-colors ${spaMode === "user" ? "bg-amber-600/30 text-amber-300" : "bg-slate-800 text-slate-400 hover:text-slate-300"}`}>User</button>
              </div>

              {spaMode === "admin" ? (
                <div className="space-y-2">
                  <button type="button" onClick={handleEnrollAdmin} className="w-full py-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-sm font-medium transition-colors">🔑 Generate Keys</button>
                  {spaResult && <p className="text-[11px] text-emerald-400">{spaResult}</p>}
                  {adminPubKey && (
                    <div className="space-y-2">
                      <div>
                        <div className="flex items-center justify-between mb-0.5"><label className="text-[10px] text-slate-400">Public Key → server config</label><button type="button" onClick={() => navigator.clipboard.writeText(adminPubKey)} className="text-[10px] text-amber-400 hover:text-amber-300">Copy</button></div>
                        <pre className="text-[10px] text-slate-300 bg-slate-800 p-2 rounded overflow-x-auto whitespace-pre-wrap break-all">{adminPubKey}</pre>
                      </div>
                      <div>
                        <div className="flex items-center justify-between mb-0.5"><label className="text-[10px] text-slate-400">Private Key → kept in keyring</label><button type="button" onClick={() => navigator.clipboard.writeText(adminPrivKey)} className="text-[10px] text-amber-400 hover:text-amber-300">Copy</button></div>
                        <pre className="text-[10px] text-slate-500 bg-slate-800/50 p-2 rounded overflow-x-auto whitespace-pre-wrap break-all max-h-20">{adminPrivKey.slice(0, 32)}...</pre>
                      </div>
                    </div>
                  )}
                </div>
              ) : (
                <div className="space-y-2">
                  <button type="button" onClick={handleEnrollUserGen} className="w-full py-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-sm font-medium transition-colors">🔑 Generate X25519 Keys</button>
                  {userPubKey && (
                    <div>
                      <div className="flex items-center justify-between mb-0.5"><label className="text-[10px] text-slate-400">Your X25519 Public Key → send to admin</label><button type="button" onClick={() => navigator.clipboard.writeText(userPubKey)} className="text-[10px] text-amber-400 hover:text-amber-300">Copy</button></div>
                      <pre className="text-[10px] text-slate-300 bg-slate-800 p-2 rounded overflow-x-auto whitespace-pre-wrap break-all">{userPubKey}</pre>
                    </div>
                  )}
                  <div><label className="block text-xs font-medium text-slate-400 mb-1">Paste Admin Encrypted Blob</label><textarea value={importBlob} onChange={(e) => setImportBlob(e.target.value)} rows={3} placeholder="From admin-tool output..." className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-xs font-mono focus:outline-none focus:border-amber-500/50 resize-none" spellCheck={false} /></div>
                  <button type="button" onClick={handleEnrollUserImport} className="w-full py-2 rounded-lg bg-slate-700 hover:bg-slate-600 text-sm font-medium transition-colors">🔓 Decrypt & Import</button>
                  {spaResult && <p className="text-[11px] text-emerald-400 mt-1">{spaResult}</p>}
                </div>
              )}

            </div>
          ) : (<>
            <div>
              <div className="flex items-center justify-between mb-1"><label className="text-xs font-medium text-slate-400">Knock Ports (JSON)</label><button type="button" onClick={() => setShowHints(!showHints)} className="text-[10px] text-emerald-400 hover:text-emerald-300">{showHints ? "Hide" : "Show"} examples</button></div>
              {showHints && (<div className="mb-2 space-y-1">{[{label:"3-step UDP",json:'[{"protocol":"udp","port":7000},{"protocol":"udp","port":8000},{"protocol":"udp","port":9000}]'},{label:"TCP+UDP mix",json:'[{"protocol":"tcp","port":4444},{"protocol":"udp","port":5555}]'},{label:"Simple single",json:'[{"protocol":"udp","port":12345}]'}].map((h)=>(<button key={h.label} type="button" onClick={() => validatePorts(h.json)} className="block w-full text-left text-[10px] px-2 py-1 rounded bg-slate-800 text-slate-400 hover:bg-slate-700 hover:text-slate-200 font-mono">{h.label}: {h.json}</button>))}</div>)}
              <textarea value={form.knockPorts} onChange={(e) => validatePorts(e.target.value)} rows={3} className={`w-full px-3 py-2 rounded-lg bg-slate-800 border text-sm font-mono focus:outline-none resize-none ${saveError ? "border-red-500/50" : "border-slate-600/50 focus:border-emerald-500/50"}`} spellCheck={false} />
              {saveError && <p className="text-xs text-red-400 mt-1">{saveError}</p>}
            </div>
            <div>
              <label className="block text-xs font-medium text-slate-400 mb-1">Delay (ms)</label><input type="number" value={form.knockDelayMs} onChange={(e) => set("knockDelayMs", Math.max(10, Number(e.target.value) || 100))} min={10} step={10} className="w-full px-3 py-2 rounded-lg bg-slate-800 border border-slate-600/50 text-sm focus:outline-none focus:border-emerald-500/50" />
            </div>
          </>)}
        </div>

{saveError && <div className="px-5 py-2 bg-red-900/40 border border-red-800/50 text-red-400 text-xs text-center">{saveError}</div>}
        <div className="flex items-center justify-end gap-2 px-5 py-4 border-t border-slate-700/50">
          <button type="button" onClick={onClose} className="px-4 py-2 rounded-lg bg-slate-800 hover:bg-slate-700 text-sm transition-colors">Cancel</button>
          <button type="submit" disabled={saving || !!saveError} className="px-6 py-2 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-sm font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed">{saving ? "Saving..." : connection ? "Update" : "Save"}</button>
        </div>
      </form>
    </div>
  );
}
