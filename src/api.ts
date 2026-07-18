import { invoke } from "@tauri-apps/api/core";
import type { Connection, SshClient, KnockStep } from "./types";


export async function listConnections(): Promise<Connection[]> { return invoke("list_connections"); }
export async function saveConnection(conn: Connection): Promise<number> { return invoke("save_connection", { connection: conn }); }
export async function deleteConnection(id: number): Promise<void> { return invoke("delete_connection", { id }); }
export async function knockAndConnect(id: number): Promise<string> { return invoke("knock_and_connect", { connectionId: id }); }
export async function validatePortsJson(json: string): Promise<KnockStep[]> { return invoke("validate_ports_json", { portsJson: json }); }
export async function detectClients(): Promise<SshClient[]> { return invoke("detect_clients"); }
export async function getSetting(key: string): Promise<string | null> { return invoke("get_setting", { key }); }
export async function setSetting(key: string, value: string): Promise<void> { return invoke("set_setting", { key, value }); }
export async function generateSiteKeys(siteId: string): Promise<string> { return invoke("generate_site_keys", { siteId }); }
export async function saveSiteKey(siteId: string, privateKey: string): Promise<void> { return invoke("save_site_key", { siteId, privateKey }); }
export async function enrollUserStart(siteId: string, name: string): Promise<string> { return invoke("enroll_user_start", { siteId, name }); }
export async function enrollUserImport(siteId: string, name: string, url: string, encryptedBlob: string): Promise<string> { return invoke("enroll_user_import", { siteId, name, url, encryptedBlob }); }
