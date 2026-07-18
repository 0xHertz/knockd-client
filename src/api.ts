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
export async function generateSiteKeys(): Promise<string> { return invoke("generate_site_keys"); }
export async function spaEncrypt(plaintext: string): Promise<string> { return invoke("spa_encrypt", { plaintext }); }
export async function spaDecrypt(encrypted: string): Promise<string> { return invoke("spa_decrypt", { encrypted }); }
export async function storeEncryptedKey(siteId: string, encryptedKey: string): Promise<void> { return invoke("store_encrypted_key", { siteId, encryptedKey }); }
export async function getX25519Identity(): Promise<[string,string]> { return invoke("get_x25519_identity"); }
export async function enrollUserImport(encryptedBlob: string): Promise<string> { return invoke("enroll_user_import", { encryptedBlob }); }
export async function adminEncryptBlob(siteId: string, userX25519Pub: string): Promise<string> { return invoke("admin_encrypt_blob", { siteId, userX25519Pub }); }
export async function adminEncryptBatch(siteId: string, csvContent: string): Promise<string> { return invoke("admin_encrypt_batch", { siteId, csvContent }); }
export async function readFileContent(path: string): Promise<string> { return invoke("read_file_content", { path }); }
export async function writeFileContent(path: string, content: string): Promise<void> { return invoke("write_file_content", { path, content }); }
export async function adminEncrypt(siteId: string, userX25519Pub: string): Promise<string> { return invoke("admin_encrypt", { siteId, userX25519Pub }); }
