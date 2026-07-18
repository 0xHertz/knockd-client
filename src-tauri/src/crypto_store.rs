use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use sha2::{Digest, Sha256};

fn derive_key() -> Result<[u8; 32], String> {
    let fp = crate::knockpass::device_fingerprint()?;
    let pepper = "knockd-client-v1-master-key";
    let material = format!("{}|{}", fp, pepper);
    Ok(Sha256::digest(material.as_bytes()).into())
}

pub fn derive_db_key() -> String {
    let fp = crate::knockpass::device_fingerprint().unwrap_or_default();
    let pepper = "knockd-sqlcipher-v1";
    let material = format!("{}|{}", fp, pepper);
    hex::encode(Sha256::digest(material.as_bytes()))
}

pub fn encrypt_value(plaintext: &str) -> Result<String, String> {
    let key = derive_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "AES".to_string())?;
    let mut iv = [0u8; 12]; OsRng.fill_bytes(&mut iv);
    let ct = cipher.encrypt(Nonce::from_slice(&iv), plaintext.as_bytes())
        .map_err(|e| format!("encrypt: {}", e))?;
    let mut out = vec![]; out.extend_from_slice(&iv); out.extend_from_slice(&ct);
    Ok(hex::encode(out))
}

pub fn decrypt_value(encrypted_hex: &str) -> Result<String, String> {
    let key = derive_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| "AES".to_string())?;
    let data = hex::decode(encrypted_hex).map_err(|e| format!("hex: {}", e))?;
    if data.len() < 13 { return Err("too short".into()); }
    let iv = &data[..12];
    let ct = &data[12..];
    let pt = cipher.decrypt(Nonce::from_slice(iv), ct)
        .map_err(|e| format!("decrypt: {}", e))?;
    String::from_utf8(pt).map_err(|e| format!("utf8: {}", e))
}
