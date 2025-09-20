//! Key management

use crate::{Address, Result, OlympusError};
use secp256k1::{Secp256k1, SecretKey, PublicKey};

/// Key manager
pub struct KeyManager {
    /// Secp256k1 context
    secp: Secp256k1<secp256k1::All>,
    /// Private keys
    keys: std::collections::HashMap<Address, SecretKey>,
}

impl KeyManager {
    /// Create new key manager
    pub fn new() -> Self {
        Self {
            secp: Secp256k1::new(),
            keys: std::collections::HashMap::new(),
        }
    }

    /// Generate new key pair
    pub fn generate_key(&mut self) -> Result<Address> {
        let secret_key = SecretKey::new(&mut secp256k1::rand::thread_rng());
        let public_key = PublicKey::from_secret_key(&self.secp, &secret_key);
        let address = Address::from_slice(&public_key.serialize_uncompressed()[1..21]);
        
        self.keys.insert(address, secret_key);
        Ok(address)
    }

    /// Import private key
    pub fn import_key(&mut self, private_key: &[u8]) -> Result<Address> {
        let secret_key = SecretKey::from_slice(private_key)
            .map_err(|e| OlympusError::Serialization(e.to_string()))?;
        
        let public_key = PublicKey::from_secret_key(&self.secp, &secret_key);
        let address = Address::from_slice(&public_key.serialize_uncompressed()[1..21]);
        
        self.keys.insert(address, secret_key);
        Ok(address)
    }

    /// Get private key for address
    pub fn get_private_key(&self, address: &Address) -> Option<&SecretKey> {
        self.keys.get(address)
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new()
    }
}
