//! EVM state management

use crate::{Address, H256, U256};
use std::collections::HashMap;

/// EVM state interface
pub trait State {
    /// Get account balance
    fn get_balance(&self, address: Address) -> U256;
    
    /// Set account balance
    fn set_balance(&mut self, address: Address, balance: U256);
    
    /// Get account nonce
    fn get_nonce(&self, address: Address) -> u64;
    
    /// Set account nonce
    fn set_nonce(&mut self, address: Address, nonce: u64);
    
    /// Get storage value
    fn get_storage(&self, address: Address, key: H256) -> Option<H256>;
    
    /// Set storage value
    fn set_storage(&mut self, address: Address, key: H256, value: H256);
    
    /// Check if account exists
    fn exists(&self, address: Address) -> bool;
    
    /// Create account
    fn create_account(&mut self, address: Address);
    
    /// Delete account
    fn delete_account(&mut self, address: Address);
    
    /// Commit state changes
    fn commit(&mut self);
    
    /// Revert state changes
    fn revert(&mut self);
}

/// In-memory state implementation
pub struct MemoryState {
    balances: HashMap<Address, U256>,
    nonces: HashMap<Address, u64>,
    storage: HashMap<(Address, H256), H256>,
}

impl MemoryState {
    /// Create new memory state
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            nonces: HashMap::new(),
            storage: HashMap::new(),
        }
    }
}

impl Default for MemoryState {
    fn default() -> Self {
        Self::new()
    }
}

impl State for MemoryState {
    fn get_balance(&self, address: Address) -> U256 {
        self.balances.get(&address).cloned().unwrap_or_default()
    }
    
    fn set_balance(&mut self, address: Address, balance: U256) {
        self.balances.insert(address, balance);
    }
    
    fn get_nonce(&self, address: Address) -> u64 {
        self.nonces.get(&address).cloned().unwrap_or(0)
    }
    
    fn set_nonce(&mut self, address: Address, nonce: u64) {
        self.nonces.insert(address, nonce);
    }
    
    fn get_storage(&self, address: Address, key: H256) -> Option<H256> {
        self.storage.get(&(address, key)).cloned()
    }
    
    fn set_storage(&mut self, address: Address, key: H256, value: H256) {
        self.storage.insert((address, key), value);
    }
    
    fn exists(&self, address: Address) -> bool {
        self.balances.contains_key(&address) || self.nonces.contains_key(&address)
    }
    
    fn create_account(&mut self, address: Address) {
        self.balances.insert(address, U256::zero());
        self.nonces.insert(address, 0);
    }
    
    fn delete_account(&mut self, address: Address) {
        self.balances.remove(&address);
        self.nonces.remove(&address);
        // Remove all storage entries for this address
        self.storage.retain(|(addr, _), _| *addr != address);
    }
    
    fn commit(&mut self) {
        // For memory state, commit is a no-op
        // In a persistent state implementation, this would flush to storage
    }
    
    fn revert(&mut self) {
        // For memory state, revert is a no-op
        // In a persistent state implementation, this would restore from checkpoint
    }
}
