use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use curve25519_dalek::montgomery::MontgomeryPoint;
use curve25519_dalek::scalar::Scalar;
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::net::UdpSocket;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)] struct AuthPacket { version: i32, site_id: String, timestamp: i64, nonce: String, user: String, target: String, signature: String }
#[derive(Debug, Deserialize)]
pub struct BrowserRequest { pub action: String, #[serde(default)] pub site_id: String, #[serde(default)] pub name: String, #[serde(default)] pub url: String, #[serde(default)] pub target: String, #[serde(default)] pub secret: String }
#[derive(Debug, Serialize)] pub struct BrowserResponse { pub success: bool, pub message: String }
#[derive(Debug, Clone, Serialize, Deserialize)] pub struct SiteMapping { pub site_id: String, pub user: String, pub target: String, pub credential: String, pub public_key: String }

const KR: &str = "knockpass";
pub fn keyring_get(key: &str) -> Result<String, String> { keyring::Entry::new(KR, key).map_err(|e| format!("keyring: {}", e))?.get_password().map_err(|e| format!("keyring: {}", e)) }
pub fn keyring_set(key: &str, value: &str) -> Result<(), String> { keyring::Entry::new(KR, key).map_err(|e| format!("keyring: {}", e))?.set_password(value).map_err(|e| format!("keyring: {}", e)) }
pub fn keyring_delete(key: &str) -> Result<(), String> { keyring::Entry::new(KR, key).map_err(|e| format!("keyring: {}", e))?.delete_credential().map_err(|e| format!("keyring: {}", e)) }

fn rand32() -> [u8; 32] { let mut b = [0u8; 32]; OsRng.fill_bytes(&mut b); b }
fn hex32(h: &str) -> Result<[u8; 32], String> { let b = hex::decode(h).map_err(|e| format!("hex: {}", e))?; b[..32].try_into().map_err(|_| "len".to_string()) }
fn gen_ed25519() -> Result<(String,String),String> { let r=rand32(); let sk=SigningKey::from_bytes(&r); Ok((hex::encode(sk.verifying_key().as_bytes()), hex::encode(r))) }
fn sign_ed25519(priv_hex: &str, msg: &str) -> Result<String,String> { Ok(hex::encode(SigningKey::from_bytes(&hex32(priv_hex)?).sign(msg.as_bytes()).to_bytes())) }
fn derive_pub(priv_hex: &str) -> Result<String,String> { Ok(hex::encode(SigningKey::from_bytes(&hex32(priv_hex)?).verifying_key().as_bytes())) }

fn gen_x25519() -> Result<(String,String),String> { let r=rand32(); let s=Scalar::from_bytes_mod_order(r); let p=MontgomeryPoint::mul_base(&s); Ok((hex::encode(p.to_bytes()), hex::encode(r))) }

fn x25519_ecdh(priv_raw: &[u8; 32], pub_raw: &[u8; 32]) -> [u8; 32] {
    let scalar = Scalar::from_bytes_mod_order(*priv_raw);
    let point = MontgomeryPoint(*pub_raw);
    (scalar * point).to_bytes()
}

fn aes_enc(plain: &[u8], mat: &str) -> Result<Vec<u8>, String> {
    let key: [u8;32]=Sha256::digest(mat.as_bytes()).into();
    let c=Aes256Gcm::new_from_slice(&key).map_err(|_|"AES".to_string())?;
    let mut iv=[0u8;12]; OsRng.fill_bytes(&mut iv);
    let ct: Vec<u8>=c.encrypt(Nonce::from_slice(&iv), plain).map_err(|e| format!("encrypt: {}",e))?;
    let mut o=vec![]; o.extend_from_slice(&iv); o.extend_from_slice(&ct); Ok(o)
}

fn aes_dec(ciphertext: &[u8], mat: &str) -> Result<Vec<u8>, String> {
    let key: [u8;32]=Sha256::digest(mat.as_bytes()).into();
    let c=Aes256Gcm::new_from_slice(&key).map_err(|_|"AES".to_string())?;
    if ciphertext.len() < 12 { return Err("too short".into()); }
    c.decrypt(Nonce::from_slice(&ciphertext[..12]), &ciphertext[12..]).map_err(|e| format!("decrypt: {}",e))
}

fn dyn_port(site_id: &str, secret: &str) -> u16 {
    let slot=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64/60;
    let d=Sha256::digest(secret.as_bytes());
    let mut mac=hmac::Hmac::<Sha256>::new_from_slice(&d).unwrap();
    hmac::Mac::update(&mut mac, format!("{}:{}",site_id,slot).as_bytes());
    let r=hmac::Mac::finalize(mac).into_bytes();
    (u32::from_be_bytes([r[0],r[1],r[2],r[3]])%40000+20000) as u16
}

// ── Device Fingerprint ────────────────────────────────────────────
pub fn device_fingerprint() -> Result<String,String> {
    let mut parts: Vec<String>=Vec::new();
    #[cfg(target_os="linux")]{for p in &["/etc/machine-id","/var/lib/dbus/machine-id"]{if let Ok(d)=std::fs::read_to_string(p){parts.push(d.trim().into());break;}}}
    #[cfg(target_os="windows")]{if let Ok(o)=std::process::Command::new("powershell").args(["-NoProfile","-Command","(Get-CimInstance -Class Win32_ComputerSystemProduct).UUID"]).output(){parts.push(String::from_utf8_lossy(&o.stdout).trim().into());}}
    if let Ok(h)=hostname::get(){parts.push(h.to_string_lossy().into());}
    if parts.is_empty(){return Err("no hardware ids".into());}
    Ok(hex::encode(Sha256::digest(parts.join("|").as_bytes())))
}

// ── Enrollment ────────────────────────────────────────────────────

pub fn generate_site_keys(_site_id: &str) -> Result<String,String> {
    let (pub_h, priv_h) = gen_ed25519()?;
    Ok(serde_json::json!({ "public_key": pub_h, "private_key": priv_h }).to_string())
}

pub fn save_site_key(site_id: &str, priv_hex: &str) -> Result<(),String> {
    let cred = format!("kp_{}_priv", site_id);
    keyring_set(&cred, priv_hex)
}

pub fn get_x25519_identity() -> Result<(String,String),String> {
    let p = keyring_get("x25519_identity_pub").map_err(|e| format!("keyring read failed: {}. Is gnome-keyring running?", e))?;
    let s = keyring_get("x25519_identity_priv").map_err(|e| format!("keyring read failed: {}", e))?;
    Ok((p, s))
}

pub fn init_x25519_identity() -> Result<(String,String),String> {
    match get_x25519_identity() {
        Ok(k) => Ok(k),
        Err(_) => {
            let (p, s) = gen_x25519()?;
            keyring_set("x25519_identity_pub", &p).map_err(|e| format!("keyring write failed: {}", e))?;
            keyring_set("x25519_identity_priv", &s).map_err(|e| format!("keyring write failed: {}", e))?;
            Ok((p, s))
        }
    }
}

pub fn enroll_user_start(site_id: &str, _name: &str) -> Result<String,String> {
    let (xpub, _xpriv) = init_x25519_identity()?;
    Ok(serde_json::json!({
        "site_id": site_id,
        "x25519_public_key": xpub,
        "mode": "user",
        "instruction": "Send this X25519 public key to the admin."
    }).to_string())
}

pub fn enroll_user_import(_site_id: &str, _name: &str, _url: &str, encrypted_blob: &str) -> Result<String,String> {
    let xpriv = keyring_get("x25519_identity_priv")?;
    let xpriv_raw = hex32(&xpriv)?;
    let data = hex::decode(encrypted_blob).map_err(|e| format!("hex: {}",e))?;
    if data.len() < 44 { return Err("blob too short".into()); }
    let eph_pub_raw: [u8;32] = data[..32].try_into().unwrap();
    let shared = x25519_ecdh(&xpriv_raw, &eph_pub_raw);
    let site_priv = aes_dec(&data[32..], &hex::encode(shared))?;
    let site_priv_hex = String::from_utf8(site_priv).map_err(|e| format!("utf8: {}",e))?;
    let pub_h = derive_pub(&site_priv_hex)?;
    Ok(serde_json::json!({
        "private_key": site_priv_hex,
        "public_key": pub_h,
        "mode": "user",
        "imported": true
    }).to_string())
}

pub fn admin_encrypt(site_id: &str, user_x25519_pub: &str) -> Result<String,String> {
    let site_priv = keyring_get(&format!("kp_{}_priv", site_id))?;
    let user_pub_raw = hex32(user_x25519_pub)?;
    let eph_raw = rand32();
    let eph_scalar = Scalar::from_bytes_mod_order(eph_raw);
    let eph_pub = MontgomeryPoint::mul_base(&eph_scalar);
    let shared = x25519_ecdh(&eph_raw, &user_pub_raw);
    let key: [u8;32] = Sha256::digest(shared).into();
    let c = Aes256Gcm::new_from_slice(&key).map_err(|_|"AES".to_string())?;
    let mut iv = [0u8;12]; OsRng.fill_bytes(&mut iv);
    let ct: Vec<u8> = c.encrypt(Nonce::from_slice(&iv), site_priv.as_bytes()).map_err(|e| format!("encrypt: {}",e))?;
    let mut out = vec![]; out.extend_from_slice(eph_pub.to_bytes().as_slice()); out.extend_from_slice(&iv); out.extend_from_slice(&ct);
    Ok(hex::encode(out))
}

// ── Auth ──────────────────────────────────────────────────────────

pub fn spa_knock(host: &str, udp_port: u16, site_id: &str, credential: &str, user: &str, target: &str) -> Result<String,String> {
    let mut nonce=[0u8;16]; OsRng.fill_bytes(&mut nonce); let nonce_h=hex::encode(nonce);
    let now=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    let priv_h=keyring_get(&format!("{}_priv",credential)).unwrap_or_else(|_| credential.to_string());
    let msg=format!("{}{}{}{}",now,nonce_h,user,target);
    let sig=sign_ed25519(&priv_h,&msg).unwrap_or_else(|_|{let mut m=hmac::Hmac::<Sha256>::new_from_slice(credential.as_bytes()).unwrap();hmac::Mac::update(&mut m,msg.as_bytes());hex::encode(hmac::Mac::finalize(m).into_bytes())});
    let pkt=AuthPacket{version:1,site_id:site_id.into(),timestamp:now,nonce:nonce_h,user:user.into(),target:target.into(),signature:sig};
    let pt=serde_json::to_vec(&pkt).map_err(|e|format!("json: {}",e))?;
    let enc=aes_enc(&pt,credential)?;
    let port=if udp_port>0{udp_port}else{dyn_port(site_id,credential)};
    let sck=UdpSocket::bind("0.0.0.0:0").map_err(|e|format!("bind: {}",e))?;
    sck.set_write_timeout(Some(Duration::from_secs(5))).ok();
    sck.send_to(&enc,&format!("{}:{}",host,port)).map_err(|e|format!("send: {}",e))?;
    Ok(format!("SPA sent to {}:{}",host,port))
}

// ── Dispatch ──────────────────────────────────────────────────────
pub fn dispatch_with_db(req: &BrowserRequest, db: &crate::db::Database) -> BrowserResponse {
    match req.action.as_str() {
        "auth" => {
            match db.list_connections() {
                Ok(conns) => {
                    let conn = conns.into_iter().find(|c|
                        c.auth_method == "knockpass" && c.spa_site_id.as_deref() == Some(&req.site_id)
                    );
                    match conn {
                        Some(c) => {
                            let credential = c.spa_credential.unwrap_or_else(|| format!("kp_{}_priv", req.site_id));
                            match spa_knock(&c.host, 0, &req.site_id, &credential, c.username.as_deref().unwrap_or(""), &c.host) {
                                Ok(m) => BrowserResponse { success: true, message: m },
                                Err(e) => BrowserResponse { success: false, message: e },
                            }
                        }
                        None => BrowserResponse { success: false, message: format!("site {} not found in connections", req.site_id) },
                    }
                }
                Err(e) => BrowserResponse { success: false, message: format!("db: {}", e) },
            }
        }
        "list" => match db.list_connections() {
            Ok(conns) => {
                let sites: Vec<serde_json::Value> = conns.into_iter()
                    .filter(|c| c.auth_method == "knockpass" && c.conn_type == "web")
                    .map(|c| serde_json::json!({
                        "site_id": c.spa_site_id.unwrap_or_default(),
                        "name": c.name,
                        "url": c.launch_uri.unwrap_or_else(|| c.host),
                    }))
                    .collect();
                BrowserResponse{success:true,message:serde_json::to_string(&sites).unwrap_or_default()}
            }
            Err(e) => BrowserResponse{success:false,message:format!("db: {}",e)},
        },
        _ => BrowserResponse{success:false,message:format!("Unknown: {}",req.action)},
    }
}
