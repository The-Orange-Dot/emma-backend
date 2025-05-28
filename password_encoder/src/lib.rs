use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng, generic_array::GenericArray},
    Aes256Gcm
};
use base64::{Engine as _, engine::general_purpose};
use std::fs;
use std::path::Path;
use std::io;

const DEV_KEY_PATH: &str = ".dev_encryption_key";
const NONCE_LENGTH: usize = 12; // 96 bits for AES-GCM

/// Gets or creates a development key
pub fn get_or_create_dev_key() -> io::Result<Vec<u8>> {
    let key_path = Path::new(DEV_KEY_PATH);
    
    if key_path.exists() {
        let key_base64 = fs::read_to_string(key_path)?;
        general_purpose::STANDARD.decode(key_base64.trim())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        let key = generate_key();
        let key_base64 = general_purpose::STANDARD.encode(&key);
        fs::write(key_path, key_base64)?;
        Ok(key)
    }
}

/// Generates a new encryption key
pub fn generate_key() -> Vec<u8> {
    Aes256Gcm::generate_key(&mut OsRng).to_vec()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_key();
        let original = "super_secret_password";
        
        let encrypted = encrypt_password(original, &key).unwrap();
        let decrypted = decrypt_password(&encrypted, &key).unwrap();
        
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_key_persistence() {
        // Clean up any existing test file
        let _ = std::fs::remove_file(DEV_KEY_PATH);
        
        let key1 = get_or_create_dev_key().unwrap();
        let key2 = get_or_create_dev_key().unwrap();
        
        assert_eq!(key1, key2);
    }
}