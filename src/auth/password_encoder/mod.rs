use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, generic_array::GenericArray},
    Aes256Gcm
};
use base64::{Engine as _, engine::general_purpose};
use std::io;

const ENCRYPTION_KEY: &str = "ENCRYPTION_KEY";
const NONCE_LENGTH: usize = 12; 

pub fn get_key() -> io::Result<Vec<u8>> {
    let env_key = std::env::var(ENCRYPTION_KEY)
        .map_err(|_| io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} environment variable must be set", ENCRYPTION_KEY),
        ))?;

    println!("Raw ENCRYPTION_KEY: {:?}", env_key); // Add this line
    
    general_purpose::STANDARD.decode(env_key.trim())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn encrypt_password(plaintext: &str, key: &[u8]) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;
    
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    let mut combined = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
    combined.extend_from_slice(&nonce);
    combined.extend_from_slice(&ciphertext);
    
    Ok(general_purpose::STANDARD.encode(&combined))
}

pub fn decrypt_password(ciphertext: &str, key: &[u8]) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;
    
    let bytes = general_purpose::STANDARD.decode(ciphertext)
        .map_err(|e| format!("Base64 decoding failed: {}", e))?;
    
    if bytes.len() < NONCE_LENGTH {
        return Err("Ciphertext too short".into());
    }
    
    let (nonce_bytes, ciphertext_bytes) = bytes.split_at(NONCE_LENGTH);
    let nonce = GenericArray::from_slice(nonce_bytes);
    
    cipher.decrypt(nonce, ciphertext_bytes)
        .map_err(|e| format!("Decryption failed: {}", e))
        .and_then(|bytes| String::from_utf8(bytes)
            .map_err(|e| format!("UTF-8 conversion failed: {}", e)))
}