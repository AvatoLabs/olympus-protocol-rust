//! Keystore management

use crate::Address;
use serde::{Deserialize, Serialize};

/// Keystore entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoreEntry {
    /// Address
    pub address: Address,
    /// Encrypted private key
    pub encrypted_key: String,
    /// Encryption parameters
    pub crypto: CryptoParams,
}

/// Encryption parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoParams {
    /// Cipher algorithm
    pub cipher: String,
    /// Cipher parameters
    pub cipherparams: CipherParams,
    /// Key derivation function
    pub kdf: String,
    /// KDF parameters
    pub kdfparams: KdfParams,
    /// MAC
    pub mac: String,
}

/// Cipher parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipherParams {
    /// Initialization vector
    pub iv: String,
}

/// KDF parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    /// Salt
    pub salt: String,
    /// Number of iterations
    pub c: u32,
    /// Key length
    pub dklen: u32,
}

/// Keystore manager
pub struct KeystoreManager {
    /// Keystore entries
    entries: std::collections::HashMap<Address, KeystoreEntry>,
}

impl KeystoreManager {
    /// Create new keystore manager
    pub fn new() -> Self {
        Self {
            entries: std::collections::HashMap::new(),
        }
    }

    /// Add keystore entry
    pub fn add_entry(&mut self, entry: KeystoreEntry) {
        self.entries.insert(entry.address, entry);
    }

    /// Get keystore entry
    pub fn get_entry(&self, address: &Address) -> Option<&KeystoreEntry> {
        self.entries.get(address)
    }

    /// List all addresses
    pub fn list_addresses(&self) -> Vec<Address> {
        self.entries.keys().cloned().collect()
    }
}

impl Default for KeystoreManager {
    fn default() -> Self {
        Self::new()
    }
}
