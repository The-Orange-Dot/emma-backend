use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, generic_array::GenericArray},
    Aes256Gcm
};
use base64::{Engine as _, engine::general_purpose};
use std::io;

const ENV_ENCRYPTION_KEY: &str = "ENCRYPTION_KEY";
const NONCE_LENGTH: usize = 12; // 96 bits for AES-GCM

/// Retrieves the encryption key from the environment. 
/// Fails if `ENCRYPTION_KEY` is not set.
pub fn get_key() -> io::Result<Vec<u8>> {
    let env_key = std::env::var(ENV_ENCRYPTION_KEY)
        .map_err(|_| io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} environment variable must be set", ENV_ENCRYPTION_KEY),
        ))?;

    general_purpose::STANDARD.decode(env_key.trim())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

/// Encrypts data with a randomly generated nonce
pub fn encrypt_password(plaintext: &str, key: &[u8]) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;
    
    // Generate random nonce for each encryption
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    
    cipher.encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))
        .map(|mut ciphertext| {
            // Prepend nonce to ciphertext
            ciphertext.splice(0..0, nonce.iter().cloned());
            general_purpose::STANDARD.encode(&ciphertext)
        })
}

/// Decrypts data with the nonce included in the ciphertext
pub fn decrypt_password(ciphertext: &str, key: &[u8]) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;
    
    let bytes = general_purpose::STANDARD.decode(ciphertext)
        .map_err(|e| format!("Base64 decoding failed: {}", e))?;
    
    // Split nonce and ciphertext
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