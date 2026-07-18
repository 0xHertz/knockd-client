import type { Connection } from "../types";

interface Props {
  connection: Connection;
  onConnect: (id: number) => void;
  onEdit: (conn: Connection) => void;
  onDelete: (id: number) => void;
  connecting: boolean;
}

export default function ConnectionCard({ connection, onConnect, onEdit, onDelete, connecting }: Props) {
  const isSsh = connection.connType === "ssh";
  const isSpa = (connection.authMethod || "knockd") === "knockpass";

  let ports: { protocol: string; port: number }[] = [];
  try { ports = JSON.parse(connection.knockPorts); } catch { /* ignore */ }

  const hostDisplay = connection.port && connection.port !== 22
    ? `${connection.host}:${connection.port}` : connection.host;

  return (
    <div className="rounded-xl bg-slate-800/60 border border-slate-700/50 p-4 hover:border-slate-600/50 transition-colors">
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-2.5">
          <span className="text-xl">{isSsh ? "🖥️" : "🌐"}</span>
          <div>
            <h3 className="font-medium text-sm">{connection.name}</h3>
            <p className="text-xs text-slate-400 font-mono">
              {isSsh && connection.username ? `${connection.username}@${hostDisplay}` : hostDisplay}
            </p>
          </div>
        </div>
        <div className="flex gap-1">
          {isSpa && <span className="text-[10px] px-2 py-0.5 rounded-full font-medium bg-amber-900/50 text-amber-300">SPA</span>}
          <span className={`text-[10px] px-2 py-0.5 rounded-full font-medium uppercase ${isSsh ? "bg-blue-900/50 text-blue-300" : "bg-purple-900/50 text-purple-300"}`}>
            {connection.connType}
          </span>
        </div>
      </div>

      {isSpa ? (
        <div className="flex items-center gap-2 mb-3">
          <span className="text-[10px] text-slate-500 uppercase tracking-wider">SPA Auth:</span>
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-amber-900/30 text-amber-300 font-mono">
            {connection.spaSiteId || "—"}
          </span>
          <span className="text-[10px] text-slate-500">·</span>
          <span className="text-[10px] text-slate-500 flex items-center gap-1">
            <span className="inline-block w-2 h-2 rounded-full bg-emerald-400 animate-pulse" />
            Dynamic Port
          </span>
        </div>
      ) : (
        ports.length > 0 && (
          <div className="flex items-center gap-1.5 mb-3 flex-wrap">
            <span className="text-[10px] text-slate-500 uppercase tracking-wider">Knock:</span>
            {ports.map((s, i) => (
              <span key={i} className="text-[10px] px-1.5 py-0.5 rounded bg-slate-700/50 text-slate-300 font-mono">
                {s.protocol || connection.knockProtocol}:{s.port}
              </span>
            ))}
            <span className="text-[10px] text-slate-500">{connection.knockDelayMs}ms</span>
          </div>
        )
      )}

      <div className="flex items-center gap-2">
        <button onClick={() => onConnect(connection.id!)} disabled={connecting}
          className={`flex-1 py-2 rounded-lg text-sm font-medium transition-all ${
            connecting ? "bg-slate-700 text-slate-400 cursor-wait"
            : isSpa ? "bg-amber-600 hover:bg-amber-500 text-white"
            : "bg-emerald-600 hover:bg-emerald-500 text-white"
          }`}>
          {connecting ? "Knocking..." : isSpa ? "🔐 Send SPA" : "🚀 Knock & Connect"}
        </button>
        <button onClick={() => onEdit(connection)} className="px-3 py-2 rounded-lg bg-slate-700/50 hover:bg-slate-600/50 text-slate-300 text-sm transition-colors">✏️</button>
        <button onClick={() => onDelete(connection.id!)} className="px-3 py-2 rounded-lg bg-slate-700/50 hover:bg-red-900/50 text-slate-300 hover:text-red-400 text-sm transition-colors">🗑️</button>
      </div>
    </div>
  );
}
