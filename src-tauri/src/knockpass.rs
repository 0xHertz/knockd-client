use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
struct AuthPacket { version: i32, site_id: String, timestamp: i64, nonce: String, user: String, target: String, signature: String }

#[derive(Debug, Deserialize)]
pub struct BrowserRequest { pub action: String, pub site_id: String, #[serde(default)] pub name: String, #[serde(default)] pub url: String, #[serde(default)] pub target: String, #[serde(default)] pub secret: String }

#[derive(Debug, Serialize)]
pub struct BrowserResponse { pub success: bool, pub message: String }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteMapping { pub site_id: String, pub user: String, pub target: String, pub credential: String, pub public_key: String, #[serde(default)] pub x25519_public_key: String }

const KR: &str = "knockpass";

pub fn keyring_get(key: &str) -> Result<String, String> {
    keyring::Entry::new(KR, key).map_err(|e| format!("keyring: {}", e))?.get_password().map_err(|e| format!("keyring: {}", e))
}

pub fn keyring_set(key: &str, value: &str) -> Result<(), String> {
    keyring::Entry::new(KR, key).map_err(|e| format!("keyring: {}", e))?.set_password(value).map_err(|e| format!("keyring: {}", e))
}

pub fn keyring_delete(key: &str) -> Result<(), String> {
    keyring::Entry::new(KR, key).map_err(|e| format!("keyring: {}", e))?.delete_credential().map_err(|e| format!("keyring: {}", e))
}

fn sign_hmac(msg: &str, secret: &str) -> String {
    type H = hmac::Hmac<Sha256>;
    let mut mac = <H as KeyInit>::new_from_slice(secret.as_bytes()).expect("HMAC");
    <H as hmac::Mac>::update(&mut mac, msg.as_bytes());
    hex::encode(<H as hmac::Mac>::finalize(mac).into_bytes())
}

fn aes_enc(plain: &[u8], mat: &str) -> Result<Vec<u8>, String> {
    let key: [u8; 32] = Sha256::digest(mat.as_bytes()).into();
    let c = Aes256Gcm::new_from_slice(&key).map_err(|_| "AES".to_string())?;
    let mut iv = [0u8; 12]; OsRng.fill_bytes(&mut iv);
    let ct: Vec<u8> = c.encrypt(Nonce::from_slice(&iv), plain).map_err(|e| format!("encrypt: {}", e))?;
    let mut o = vec![]; o.extend_from_slice(&iv); o.extend_from_slice(&ct); Ok(o)
}

fn dyn_port(site_id: &str, secret: &str) -> u16 {
    let slot = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64 / 60;
    let msg = format!("{}:{}", site_id, slot);
    type H = hmac::Hmac<Sha256>;
    let d = Sha256::digest(secret.as_bytes());
    let mut mac = <H as KeyInit>::new_from_slice(&d).unwrap();
    <H as hmac::Mac>::update(&mut mac, msg.as_bytes());
    let r = <H as hmac::Mac>::finalize(mac).into_bytes();
    (u32::from_be_bytes([r[0], r[1], r[2], r[3]]) % 40000 + 20000) as u16
}

fn sites_path() -> PathBuf {
    dirs::home_dir().unwrap_or_default().join(".knockpass").join("sites.json")
}

pub fn load_site_store() -> Result<HashMap<String, SiteMapping>, String> {
    let p = sites_path();
    if !p.exists() { return Ok(HashMap::new()); }
    serde_json::from_str(&std::fs::read_to_string(&p).map_err(|e| format!("read: {}", e))?).map_err(|e| format!("json: {}", e))
}

pub fn save_site_store(s: &HashMap<String, SiteMapping>) -> Result<(), String> {
    let p = sites_path();
    if let Some(d) = p.parent() { std::fs::create_dir_all(d).map_err(|e| format!("mkdir: {}", e))?; }
    std::fs::write(&p, serde_json::to_string_pretty(s).map_err(|e| format!("json: {}", e))?).map_err(|e| format!("write: {}", e))
}

pub fn device_fingerprint() -> Result<String, String> {
    let mut parts: Vec<String> = Vec::new();
    #[cfg(target_os = "linux")] {
        for p in &["/etc/machine-id", "/var/lib/dbus/machine-id"] {
            if let Ok(d) = std::fs::read_to_string(p) { parts.push(d.trim().into()); break; }
        }
    }
    #[cfg(target_os = "windows")] {
        if let Ok(o) = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", "(Get-CimInstance -Class Win32_ComputerSystemProduct).UUID"]).output()
            { parts.push(String::from_utf8_lossy(&o.stdout).trim().into()); }
    }
    if let Ok(h) = hostname::get() { parts.push(h.to_string_lossy().into()); }
    if parts.is_empty() { return Err("no hardware ids".into()); }
    Ok(hex::encode(Sha256::digest(parts.join("|").as_bytes())))
}

pub fn authenticate_site(site_id: &str) -> Result<String, String> {
    let s = load_site_store()?;
    let m = s.get(site_id).ok_or_else(|| format!("site {} not found", site_id))?.clone();
    let secret = keyring_get(&format!("{}_priv", m.credential))?;
    let mut nonce = [0u8; 16]; OsRng.fill_bytes(&mut nonce);
    let nonce_hex = hex::encode(nonce);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let msg = format!("{}{}{}{}", now, nonce_hex, m.user, m.target);
    let sig = sign_hmac(&msg, &secret);
    let pkt = AuthPacket { version: 1, site_id: site_id.into(), timestamp: now, nonce: nonce_hex, user: m.user, target: m.target.clone(), signature: sig };
    let pt = serde_json::to_vec(&pkt).map_err(|e| format!("json: {}", e))?;
    let enc = aes_enc(&pt, &secret)?;
    let port = dyn_port(site_id, &secret);
    let host = m.target.trim_start_matches("https://").trim_start_matches("http://").split(':').next().unwrap_or(&m.target);
    let sck = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("bind: {}", e))?;
    sck.set_write_timeout(Some(Duration::from_secs(5))).ok();
    sck.send_to(&enc, &format!("{}:{}", host, port)).map_err(|e| format!("send: {}", e))?;
    Ok(format!("SPA sent to {}:{} for {}", host, port, site_id))
}

pub fn dispatch(req: &BrowserRequest) -> BrowserResponse {
    match req.action.as_str() {
        "auth" => match authenticate_site(&req.site_id) {
            Ok(m) => BrowserResponse { success: true, message: m },
            Err(e) => BrowserResponse { success: false, message: e },
        },
        _ => BrowserResponse { success: false, message: format!("Unknown: {}", req.action) },
    }
}

pub fn spa_knock(host: &str, udp_port: u16, site_id: &str, credential: &str, user: &str, target: &str) -> Result<String, String> {
    let mut nonce = [0u8; 16]; OsRng.fill_bytes(&mut nonce);
    let nonce_h = hex::encode(nonce);
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let secret = keyring_get(&format!("{}_priv", credential)).unwrap_or_else(|_| credential.to_string());
    let msg = format!("{}{}{}{}", now, nonce_h, user, target);
    let sig = sign_hmac(&msg, &secret);
    let pkt = AuthPacket { version: 1, site_id: site_id.into(), timestamp: now, nonce: nonce_h, user: user.into(), target: target.into(), signature: sig };
    let pt = serde_json::to_vec(&pkt).map_err(|e| format!("json: {}", e))?;
    let enc = aes_enc(&pt, &secret)?;
    let port = if udp_port > 0 { udp_port } else { dyn_port(site_id, &secret) };
    let addr = format!("{}:{}", host, port);
    let sck = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("bind: {}", e))?;
    sck.set_write_timeout(Some(Duration::from_secs(5))).ok();
    sck.send_to(&enc, &addr).map_err(|e| format!("send: {}", e))?;
    Ok(format!("SPA sent to {}:{}", host, port))
}
