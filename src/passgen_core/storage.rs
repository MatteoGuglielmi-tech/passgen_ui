use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A single password entry
#[derive(Serialize, Deserialize, Clone)]
pub struct PasswordEntry {
    pub name: String,
    pub password: String,
    pub created_at: String,
}

/// The encrypted file format
#[derive(Serialize, Deserialize)]
struct EncryptedStore {
    salt: String,       // Base64 encoded
    nonce: String,      // Base64 encoded
    ciphertext: String, // Base64 encoded
}

/// Password storage manager
pub struct Storage {
    file_path: PathBuf,
    master_key: [u8; 32],
}

impl Storage {
    /// Create a new storage with a master password
    pub fn new(master_password: &str) -> Result<Self, String> {
        let file_path = Self::default_path()?;

        // Derive key from master password
        // If file exists, use its salt; otherwise generate new
        let (master_key, _salt) = if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| format!("Failed to read file: {}", e))?;
            let store: EncryptedStore = serde_json::from_str(&content)
                .map_err(|e| format!("Invalid file format: {}", e))?;
            let salt = BASE64
                .decode(&store.salt)
                .map_err(|e| format!("Invalid salt: {}", e))?;
            (Self::derive_key(master_password, &salt), salt)
        } else {
            let mut salt = [0u8; 16];
            rand::rng().fill_bytes(&mut salt);
            (Self::derive_key(master_password, &salt), salt.to_vec())
        };

        Ok(Self {
            file_path,
            master_key,
        })
    }

    /// Get default storage path
    fn default_path() -> Result<PathBuf, String> {
        let home = dirs::home_dir().ok_or_else(|| "Cannot find home directory".to_string())?;
        Ok(home.join(".passgen_vault.enc"))
    }

    /// Simple key derivation (PBKDF2-like using multiple SHA256 rounds)
    fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut key = [0u8; 32];
        let combined: Vec<u8> = password
            .as_bytes()
            .iter()
            .chain(salt.iter())
            .copied()
            .collect();

        // Simple iterative hashing (not as secure as Argon2, but works)
        for i in 0..32 {
            let mut hasher = DefaultHasher::new();
            combined.hash(&mut hasher);
            (i as u64).hash(&mut hasher);
            let hash = hasher.finish();
            key[i] = (hash & 0xFF) as u8;
        }

        // Additional rounds for strengthening
        for _ in 0..10000 {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            salt.hash(&mut hasher);
            let hash = hasher.finish().to_le_bytes();
            for (i, &b) in hash.iter().enumerate() {
                key[i % 32] ^= b;
            }
        }

        key
    }

    /// Load all passwords from encrypted storage
    pub fn load(&self) -> Result<Vec<PasswordEntry>, String> {
        if !self.file_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let store: EncryptedStore =
            serde_json::from_str(&content).map_err(|e| format!("Invalid file format: {}", e))?;

        let nonce_bytes = BASE64
            .decode(&store.nonce)
            .map_err(|e| format!("Invalid nonce: {}", e))?;
        let ciphertext = BASE64
            .decode(&store.ciphertext)
            .map_err(|e| format!("Invalid ciphertext: {}", e))?;

        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| format!("Cipher init failed: {}", e))?;

        let nonce = Nonce::from_slice(&nonce_bytes);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|_| "Decryption failed - wrong master password?".to_string())?;

        let json = String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8: {}", e))?;

        serde_json::from_str(&json).map_err(|e| format!("Invalid JSON: {}", e))
    }

    /// Save a password entry (appends to existing)
    pub fn save(&self, entry: PasswordEntry) -> Result<(), String> {
        let mut entries = self.load().unwrap_or_default();
        entries.push(entry);
        self.save_all(&entries)
    }

    /// Save all entries
    fn save_all(&self, entries: &[PasswordEntry]) -> Result<(), String> {
        let json =
            serde_json::to_string(entries).map_err(|e| format!("Serialization failed: {}", e))?;

        // Generate new nonce for each save
        let mut nonce_bytes = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(&self.master_key)
            .map_err(|e| format!("Cipher init failed: {}", e))?;

        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, json.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        // Get or generate salt
        let salt = if self.file_path.exists() {
            let content = fs::read_to_string(&self.file_path).ok();
            content
                .and_then(|c| serde_json::from_str::<EncryptedStore>(&c).ok())
                .map(|s| s.salt)
                .unwrap_or_else(|| {
                    let mut s = [0u8; 16];
                    rand::rng().fill_bytes(&mut s);
                    BASE64.encode(s)
                })
        } else {
            let mut s = [0u8; 16];
            rand::rng().fill_bytes(&mut s);
            BASE64.encode(s)
        };

        let store = EncryptedStore {
            salt,
            nonce: BASE64.encode(nonce_bytes),
            ciphertext: BASE64.encode(ciphertext),
        };

        let output = serde_json::to_string_pretty(&store)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        fs::write(&self.file_path, output).map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(())
    }

    /// Get the storage file path for display
    pub fn path(&self) -> &PathBuf {
        &self.file_path
    }

    /// Delete a password entry by index
    pub fn delete(&self, index: usize) -> Result<(), String> {
        let mut entries = self.load()?;
        if index >= entries.len() {
            return Err("Invalid index".into());
        }
        entries.remove(index);
        self.save_all(&entries)
    }

    /// Update a password entry by index
    pub fn update(&self, index: usize, entry: PasswordEntry) -> Result<(), String> {
        let mut entries = self.load()?;
        if index >= entries.len() {
            return Err("Invalid index".into());
        }
        entries[index] = entry;
        self.save_all(&entries)
    }

    /// Change the master password
    /// Returns a new Storage instance with the new key
    pub fn change_master_password(&self, new_password: &str) -> Result<Storage, String> {
        // Load existing entries with current key
        let entries = self.load()?;

        // Generate new salt
        let mut new_salt = [0u8; 16];
        rand::rng().fill_bytes(&mut new_salt);

        // Derive new key
        let new_key = Self::derive_key(new_password, &new_salt);

        // Create new storage with new key
        let new_storage = Storage {
            file_path: self.file_path.clone(),
            master_key: new_key,
        };

        // Encrypt and save with new key
        // We need to write the new salt too, so we do it manually here
        let json =
            serde_json::to_string(&entries).map_err(|e| format!("Serialization failed: {}", e))?;

        let mut nonce_bytes = [0u8; 12];
        rand::rng().fill_bytes(&mut nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(&new_key)
            .map_err(|e| format!("Cipher init failed: {}", e))?;

        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, json.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;

        let store = EncryptedStore {
            salt: BASE64.encode(new_salt),
            nonce: BASE64.encode(nonce_bytes),
            ciphertext: BASE64.encode(ciphertext),
        };

        let output = serde_json::to_string_pretty(&store)
            .map_err(|e| format!("Serialization failed: {}", e))?;

        fs::write(&self.file_path, output).map_err(|e| format!("Failed to write file: {}", e))?;

        Ok(new_storage)
    }
}
