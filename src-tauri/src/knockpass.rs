use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use hmac::{Hmac, Mac};
use rand::RngCore;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Serialize)]
struct AuthPacket {
    version: i32,
    site_id: String,
    timestamp: i64,
    nonce: String,
    user: String,
    target: String,
    signature: String,
}

pub fn spa_knock(
    host: &str,
    udp_port: u16,
    site_id: &str,
    credential: &str,
    user: &str,
    target: &str,
) -> Result<String, String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let mut nonce_bytes = [0u8; 16];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = hex::encode(nonce_bytes);

    let message = format!("{}{}{}{}", timestamp, nonce, user, target);
    let mut mac = <HmacSha256 as KeyInit>::new_from_slice(credential.as_bytes())
        .map_err(|_| "HMAC key error".to_string())?;
    mac.update(message.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    let packet = AuthPacket {
        version: 1,
        site_id: site_id.to_string(),
        timestamp,
        nonce,
        user: user.to_string(),
        target: target.to_string(),
        signature,
    };

    let plaintext = serde_json::to_vec(&packet).map_err(|e| format!("JSON: {}", e))?;

    let key: [u8; 32] = Sha256::digest(credential.as_bytes()).into();
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "AES key error".to_string())?;

    let mut iv = [0u8; 12];
    OsRng.fill_bytes(&mut iv);
    let nonce_gcm = Nonce::from_slice(&iv);

    let ciphertext = cipher
        .encrypt(nonce_gcm, plaintext.as_ref())
        .map_err(|e| format!("Encrypt: {}", e))?;

    let mut payload = Vec::with_capacity(iv.len() + ciphertext.len());
    payload.extend_from_slice(&iv);
    payload.extend_from_slice(&ciphertext);

    let addr = format!("{}:{}", host, udp_port);
    let socket = UdpSocket::bind("0.0.0.0:0").map_err(|e| format!("Bind: {}", e))?;
    socket
        .set_write_timeout(Some(Duration::from_secs(5)))
        .ok();
    socket
        .send_to(&payload, &addr)
        .map_err(|e| format!("Send: {}", e))?;

    Ok(format!(
        "SPA packet sent to {}:{} for site {}",
        host, udp_port, site_id
    ))
}
