use std::path::{Path, PathBuf};

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use rand::RngCore;

use crate::error::TokeniserError;
use crate::tokenizer::TokenMapData;

const MAGIC: &[u8; 4] = b"LTOK";
const VERSION: u8 = 0x01;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const HEADER_LEN: usize = 4 + 1 + SALT_LEN + NONCE_LEN; // 33 bytes

/// Derive a 256-bit key from passphrase + salt using Argon2id.
fn derive_key(passphrase: &[u8], salt: &[u8]) -> Result<[u8; 32], TokeniserError> {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(passphrase, salt, &mut key)
        .map_err(|e| TokeniserError::StoreError {
            message: format!("Key derivation failed: {}", e),
        })?;
    Ok(key)
}

/// Encrypted token store using AES-256-GCM with Argon2id key derivation.
///
/// File format:
/// ```text
/// [4 bytes]   Magic: "LTOK"
/// [1 byte]    Version: 0x01
/// [16 bytes]  Salt (for Argon2 key derivation)
/// [12 bytes]  Nonce (for AES-GCM)
/// [N bytes]   Ciphertext (AES-GCM encrypted JSON of TokenMapData, includes 16-byte auth tag)
/// ```
#[derive(Debug)]
pub struct Store {
    path: PathBuf,
    passphrase: String,
}

impl Store {
    /// Create a new Store targeting `store_dir/store.enc` with an explicit passphrase.
    pub fn with_passphrase(store_dir: &Path, passphrase: String) -> Result<Self, TokeniserError> {
        let store_path = store_dir.join("store.enc");
        if let Some(parent) = store_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| TokeniserError::StoreError {
                message: format!("Cannot create store directory: {}", e),
            })?;
        }

        Ok(Self {
            path: store_path,
            passphrase,
        })
    }

    /// Create a new Store targeting `store_dir/store.enc`.
    /// Requires `LOGTOK_KEY` environment variable to be set.
    pub fn new(store_dir: &Path) -> Result<Self, TokeniserError> {
        let passphrase =
            std::env::var("LOGTOK_KEY").map_err(|_| TokeniserError::StoreError {
                message: "LOGTOK_KEY environment variable is required for token store encryption. \
                          Set it with: export LOGTOK_KEY='your-passphrase'"
                    .to_string(),
            })?;
        Self::with_passphrase(store_dir, passphrase)
    }

    /// Load TokenMapData from the encrypted store file.
    /// Returns empty TokenMapData if the file does not exist (first run).
    pub fn load(&self) -> Result<TokenMapData, TokeniserError> {
        if !self.path.exists() {
            return Ok(TokenMapData::default());
        }

        let data = std::fs::read(&self.path).map_err(|e| TokeniserError::StoreError {
            message: format!("Cannot read store file: {}", e),
        })?;

        // Validate magic and version
        if data.len() < HEADER_LEN || &data[0..4] != MAGIC || data[4] != VERSION {
            return Err(TokeniserError::StoreError {
                message: "Invalid store file format".to_string(),
            });
        }

        let salt = &data[5..5 + SALT_LEN];
        let nonce_bytes = &data[5 + SALT_LEN..HEADER_LEN];
        let ciphertext = &data[HEADER_LEN..];

        let key = derive_key(self.passphrase.as_bytes(), salt)?;
        let cipher = Aes256Gcm::new(&key.into());
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|_| {
            TokeniserError::StoreError {
                message: "Decryption failed -- wrong LOGTOK_KEY or corrupted store file"
                    .to_string(),
            }
        })?;

        serde_json::from_slice(&plaintext).map_err(|e| TokeniserError::StoreError {
            message: format!("Store data deserialization failed: {}", e),
        })
    }

    /// Save TokenMapData to the encrypted store file.
    /// Preserves existing salt for cross-session key consistency.
    /// Generates a fresh random nonce on every save.
    /// Uses atomic write (temp file + rename) to prevent corruption.
    pub fn save(&self, data: &TokenMapData) -> Result<(), TokeniserError> {
        // Load existing salt if store exists, else generate new
        let salt = if self.path.exists() {
            let existing =
                std::fs::read(&self.path).map_err(|e| TokeniserError::StoreError {
                    message: format!("Cannot read existing store for salt: {}", e),
                })?;
            if existing.len() >= 5 + SALT_LEN {
                let mut s = [0u8; SALT_LEN];
                s.copy_from_slice(&existing[5..5 + SALT_LEN]);
                s
            } else {
                let mut s = [0u8; SALT_LEN];
                rand::rng().fill_bytes(&mut s);
                s
            }
        } else {
            let mut s = [0u8; SALT_LEN];
            rand::rng().fill_bytes(&mut s);
            s
        };

        let key = derive_key(self.passphrase.as_bytes(), &salt)?;
        let cipher = Aes256Gcm::new(&key.into());
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        let plaintext = serde_json::to_vec(data).map_err(|e| TokeniserError::StoreError {
            message: format!("Serialization failed: {}", e),
        })?;

        let ciphertext = cipher
            .encrypt(&nonce, plaintext.as_ref())
            .map_err(|_| TokeniserError::StoreError {
                message: "Encryption failed".to_string(),
            })?;

        // Build output: magic + version + salt + nonce + ciphertext
        let mut output = Vec::with_capacity(HEADER_LEN + ciphertext.len());
        output.extend_from_slice(MAGIC);
        output.push(VERSION);
        output.extend_from_slice(&salt);
        output.extend_from_slice(nonce.as_slice());
        output.extend_from_slice(&ciphertext);

        // Atomic write via temp file + rename
        let tmp_path = self.path.with_extension("enc.tmp");
        std::fs::write(&tmp_path, &output).map_err(|e| TokeniserError::StoreError {
            message: format!("Cannot write temp store file: {}", e),
        })?;
        std::fs::rename(&tmp_path, &self.path).map_err(|e| TokeniserError::StoreError {
            message: format!("Cannot finalize store file: {}", e),
        })?;

        Ok(())
    }

    /// Delete the store file.
    pub fn reset(&self) -> Result<(), TokeniserError> {
        if self.path.exists() {
            std::fs::remove_file(&self.path).map_err(|e| TokeniserError::StoreError {
                message: format!("Cannot delete store file: {}", e),
            })?;
        }
        Ok(())
    }
}
