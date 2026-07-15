use tauri::State;

use crate::db::Database;
use crate::knock;
use crate::launcher;
use crate::models::{Connection, KnockStep, SshClient};

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
            launcher::launch_ssh(
                &client,
                &conn.host,
                conn.port.unwrap_or(22),
                &conn.username.unwrap_or_default(),
            )
            .map(|msg| format!("{} | {}", result.message, msg))
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
