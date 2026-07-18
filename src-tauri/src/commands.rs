use tauri::State;

use crate::db::Database;
use crate::knock;
use crate::launcher;
use crate::models::{Connection, KnockStep, SshClient};
use std::process::Command;

pub struct AppState {
    pub db: Database,
}

#[tauri::command]
pub fn list_connections(state: State<AppState>) -> Result<Vec<Connection>, String> {
    state.db.list_connections().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_connection(state: State<AppState>, connection: Connection) -> Result<i64, String> {
    if let Some(id) = connection.id {
        state
            .db
            .update_connection(&connection)
            .map_err(|e| e.to_string())?;
        Ok(id)
    } else {
        state
            .db
            .insert_connection(&connection)
            .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub fn delete_connection(state: State<AppState>, id: i64) -> Result<(), String> {
    state.db.delete_connection(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn knock_and_connect(state: State<AppState>, connection_id: i64) -> Result<String, String> {
    let conn = state
        .db
        .get_connection(connection_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Connection not found".to_string())?;

    if conn.auth_method == "knockpass" {
        let site_id = conn.spa_site_id.as_deref().unwrap_or("");
        let credential = conn.spa_credential.as_deref().unwrap_or("");
        let udp_port = conn.spa_udp_port.unwrap_or(0);
        if site_id.is_empty() || credential.is_empty() {
            return Err("KnockPass SPA requires site_id and credential".into());
        }
        let msg = crate::knockpass::spa_knock(
            &conn.host,
            udp_port,
            site_id,
            credential,
            conn.username.as_deref().unwrap_or(""),
            &conn.host,
        )?;
        if conn.conn_type == "web" {
            let url = conn.launch_uri.as_deref().unwrap_or(&conn.host);
            return launcher::launch_url(url).map(|m| format!("{} | {}", msg, m));
        }
        return Ok(msg);
    }

    let result = knock::perform_knock(
        &conn.host,
        &conn.knock_ports,
        &conn.knock_protocol,
        conn.knock_delay_ms as u64,
    );

    if !result.success {
        return Err(result.message);
    }

    match conn.conn_type.as_str() {
        "ssh" => {
            let client = conn.ssh_client.clone().unwrap_or_else(|| "auto".into());
            let result = launch_ssh_or_custom(&client, &state, &conn, result.message);
            result
        }
        "web" => {
            let url = conn.launch_uri.clone().unwrap_or_else(|| {
                format!(
                    "https://{}",
                    if let Some(p) = conn.port {
                        format!("{}:{}", conn.host, p)
                    } else {
                        conn.host.clone()
                    }
                )
            });
            launcher::launch_url(&url).map(|msg| format!("{} | {}", result.message, msg))
        }
        _ => Err(format!("Unknown connection type: {}", conn.conn_type)),
    }
}

#[tauri::command]
pub fn test_knock(
    host: String,
    ports_json: String,
    protocol: String,
    delay_ms: u64,
) -> Result<String, String> {
    let result = knock::perform_knock(&host, &ports_json, &protocol, delay_ms);
    if result.success {
        Ok(result.message)
    } else {
        Err(result.message)
    }
}

#[tauri::command]
pub fn validate_ports_json(ports_json: String) -> Result<Vec<KnockStep>, String> {
    let steps: Vec<KnockStep> =
        serde_json::from_str(&ports_json).map_err(|e| format!("Invalid JSON: {}", e))?;
    if steps.is_empty() {
        return Err("Port list cannot be empty".into());
    }
    for (i, step) in steps.iter().enumerate() {
        if step.port == 0 {
            return Err(format!("Step {}: port cannot be 0", i + 1));
        }
        let proto = step.protocol.to_lowercase();
        if !proto.is_empty() && proto != "udp" && proto != "tcp" {
            return Err(format!(
                "Step {}: protocol must be 'udp' or 'tcp', got '{}'",
                i + 1,
                step.protocol
            ));
        }
    }
    Ok(steps)
}

#[tauri::command]
pub fn detect_clients(state: State<AppState>) -> Vec<SshClient> {
    let mut clients = launcher::detect_ssh_clients();
    if let Ok(Some(json)) = state.db.get_setting("custom_ssh_paths") {
        if let Ok(custom) = serde_json::from_str::<Vec<SshClient>>(&json) {
            for c in custom {
                let installed = !c.path.is_empty()
                    && std::path::Path::new(&c.path).exists();
                clients.push(SshClient {
                    installed,
                    ..c
                });
            }
        }
    }
    clients
}

#[tauri::command]
pub fn get_setting(state: State<AppState>, key: String) -> Result<Option<String>, String> {
    state.db.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_setting(state: State<AppState>, key: String, value: String) -> Result<(), String> {
    state
        .db
        .set_setting(&key, &value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn generate_site_keys(site_id: String) -> Result<String, String> {
    crate::knockpass::generate_site_keys(&site_id)
}

#[tauri::command]
pub fn save_site_key(site_id: String, private_key: String) -> Result<(), String> {
    crate::knockpass::save_site_key(&site_id, &private_key)
}

#[tauri::command]
pub fn enroll_user_start(site_id: String, name: String) -> Result<String, String> {
    crate::knockpass::enroll_user_start(&site_id, &name)
}

#[tauri::command]
pub fn enroll_user_import(site_id: String, name: String, url: String, encrypted_blob: String) -> Result<String, String> {
    crate::knockpass::enroll_user_import(&site_id, &name, &url, &encrypted_blob)
}

#[tauri::command]

fn launch_ssh_or_custom(
    client: &str,
    state: &State<AppState>,
    conn: &Connection,
    knock_msg: String,
) -> Result<String, String> {
    let custom_path = if let Ok(Some(json)) = state.db.get_setting("custom_ssh_paths") {
        if let Ok(custom) = serde_json::from_str::<Vec<SshClient>>(&json) {
            custom.iter().find(|c| c.name == client).map(|c| c.path.clone())
        } else { None }
    } else { None };

    let user_host = if let Some(u) = &conn.username {
        format!("{}@{}", u, conn.host)
    } else {
        conn.host.clone()
    };
    let port = conn.port.unwrap_or(22);

    let launch_msg = if let Some(path) = custom_path {
        let name = std::path::Path::new(&path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        let url = format!("ssh://{}:{}", user_host, port);
        let args: &[&str] = match name.as_str() {
            n if n.contains("putty") || n.contains("kitty") =>
                &["-ssh", &user_host, "-P", &port.to_string()],
            n if n.contains("xshell") || n.contains("nxshell") =>
                &["-url", &url],
            n if n.contains("mobaxterm") =>
                &["-newtab", &url],
            n if n.contains("securecrt") =>
                &["/SSH2", "/L", conn.username.as_deref().unwrap_or(""), &conn.host, "/P", &port.to_string()],
            n if n.contains("bitvise") =>
                &["-host", &conn.host, "-port", &port.to_string(), "-user", conn.username.as_deref().unwrap_or("")],
            n if n.contains("termius") =>
                &["ssh", &user_host],
            _ => &[url.as_str()],
        };
        Command::new(&path)
            .args(args)
            .spawn()
            .map(|_| format!("Launched {} (custom)", client))
            .map_err(|e| format!("Failed to launch {}: {}", client, e))?
    } else {
        launcher::launch_ssh(client, &conn.host, port, conn.username.as_deref().unwrap_or(""))?
    };

    Ok(format!("{} | {}", knock_msg, launch_msg))
}
