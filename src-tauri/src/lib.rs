mod commands;
mod crypto_store;
mod spa_cmds;
pub mod db;
mod knock;
pub mod knockpass;
mod launcher;
mod models;

use commands::AppState;
use db::Database;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_dir = dirs::data_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("knockd-client");

            let database = Database::new(&app_dir).expect("Failed to initialize database");
            app.manage(AppState { db: database });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_connections,
            commands::save_connection,
            commands::delete_connection,
            commands::knock_and_connect,
            commands::test_knock,
            commands::validate_ports_json,
            commands::detect_clients,
            commands::get_setting,
            commands::set_setting,
            spa_cmds::spa_encrypt,
            spa_cmds::spa_decrypt,
            commands::generate_site_keys,
            commands::store_encrypted_key,
            commands::get_x25519_identity,
            
            commands::enroll_user_import,
            commands::admin_encrypt_blob,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
