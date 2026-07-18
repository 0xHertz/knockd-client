
#[tauri::command]
pub fn spa_encrypt(plaintext: String) -> Result<String, String> {
    crate::crypto_store::encrypt_value(&plaintext)
}

#[tauri::command]
pub fn spa_decrypt(encrypted: String) -> Result<String, String> {
    crate::crypto_store::decrypt_value(&encrypted)
}
