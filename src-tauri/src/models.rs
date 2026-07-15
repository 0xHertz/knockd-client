use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: Option<i64>,
    pub name: String,
    #[serde(rename = "connType")]
    pub conn_type: String,
    pub host: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    #[serde(rename = "sshClient")]
    pub ssh_client: Option<String>,
    #[serde(rename = "knockPorts")]
    pub knock_ports: String,
    #[serde(rename = "knockProtocol")]
    pub knock_protocol: String,
    #[serde(rename = "knockDelayMs")]
    pub knock_delay_ms: i64,
    #[serde(rename = "launchUri")]
    pub launch_uri: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnockStep {
    pub protocol: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshClient {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub installed: bool,
}
