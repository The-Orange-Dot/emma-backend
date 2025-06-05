use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
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
    
    let key_bytes = general_purpose::STANDARD.decode(env_key.trim())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    if key_bytes.len() != 32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Key must be 32 bytes, got {} bytes", key_bytes.len()),
        ));
    }
    Ok(key_bytes)
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
    if key.len() != 32 {
        return Err(format!("Invalid key length: expected 32 bytes, got {}", key.len()));
    }

    let bytes = match general_purpose::STANDARD.decode(ciphertext.trim()) {
        Ok(b) => b,
        Err(e) => {
            println!("BASE64 DECODE ERROR: {}", e);
            return Err(format!("Base64 decoding failed: {}", e));
        }
    };

    if bytes.len() < NONCE_LENGTH {
        return Err(format!("Ciphertext too short: {} bytes", bytes.len()));
    }

    let (nonce_bytes, ciphertext_bytes) = bytes.split_at(NONCE_LENGTH);
    
    let cipher = match Aes256Gcm::new_from_slice(key) {
        Ok(c) => c,
        Err(e) => {
            println!("CIPHER INIT ERROR: {:?}", e);
            return Err(format!("Cipher initialization failed: {}", e));
        }
    };

    match cipher.decrypt(nonce_bytes.into(), ciphertext_bytes) {
        Ok(plaintext) => String::from_utf8(plaintext)
            .map_err(|e| format!("UTF-8 conversion failed: {}", e)),
        Err(e) => {
            println!("DECRYPTION ERROR DETAIL: {:?}", e);
            Err(format!("Decryption failed: {}", e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
 #[test]   
  fn test_encrypt_decrypt_cycle() -> Result<(), String> {
      let key = match get_key() {
          Ok(k) => k,
          Err(_) => {
              println!("No ENCRYPTION_KEY found, using test key");
              let test_key: [u8; 32] = rand::random();
              test_key.to_vec()
          }
      };

      println!("Using key: {}", hex::encode(&key));

      let plaintext = "my_secret_password";
      println!("Original: {}", plaintext);
      
      let encrypted = encrypt_password(plaintext, &key)?;
      println!("Encrypted: {}", encrypted);

      let decrypted = decrypt_password(&encrypted, &key)?;
      println!("Decrypted: {}", decrypted);

      if plaintext == decrypted {
          println!("SUCCESS: Encryption/decryption cycle works!");
          Ok(())
      } else {
          Err("Decrypted text doesn't match original!".into())
      }
  }
}