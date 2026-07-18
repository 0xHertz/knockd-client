#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{self, Read, Write};
use knockd_client_lib::{knockpass, db::Database};

fn run_native_host() {
    let app_dir = dirs::data_dir().unwrap_or_default().join("knockd-client");
    let database = match Database::new(&app_dir) {
        Ok(db) => db,
        Err(_) => return,
    };
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();
    while let Ok(req) = read_msg(&mut stdin) {
        let resp = knockpass::dispatch_with_db(&req, &database);
        let json = serde_json::to_vec(&resp).unwrap_or_default();
        let _ = write_msg(&mut stdout, &json);
    }
}

fn read_msg(r: &mut impl Read) -> Result<knockpass::BrowserRequest, String> {
    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf).map_err(|e| format!("read: {}", e))?;
    let len = u32::from_ne_bytes(len_buf);
    if len > 1024 * 1024 { return Err("msg too large".into()); }
    let mut buf = vec![0u8; len as usize];
    r.read_exact(&mut buf).map_err(|e| format!("read: {}", e))?;
    serde_json::from_slice(&buf).map_err(|e| format!("json: {}", e))
}

fn write_msg(w: &mut impl Write, data: &[u8]) -> io::Result<()> {
    w.write_all(&(data.len() as u32).to_ne_bytes())?;
    w.write_all(data)?;
    w.flush()
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--activate" {
        match knockpass::device_fingerprint() {
            Ok(fp) => println!("{}", fp),
            Err(e) => eprintln!("fingerprint error: {}", e),
        }
        return;
    }
    if args.iter().any(|a| a.starts_with("chrome-extension://")) {
        run_native_host();
        return;
    }
    knockd_client_lib::run()
}
