//! Persistent EVM state management using sled database

use crate::{Address, H256, U256, Result, OlympusError};
use crate::evm::state::State;
use sled::{Db, Tree};
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use bincode;

/// Persistent state implementation using sled database
pub struct PersistentState {
    /// Database instance
    db: Arc<Db>,
    /// Accounts tree
    accounts_tree: Tree,
    /// Storage tree
    storage_tree: Tree,
    /// Code tree
    code_tree: Tree,
}

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    /// Account balance
    pub balance: U256,
    /// Account nonce
    pub nonce: u64,
    /// Account code hash
    pub code_hash: H256,
    /// Account storage root
    pub storage_root: H256,
}

/// State checkpoint for rollback
#[derive(Debug, Clone)]
pub struct StateCheckpoint {
    /// Checkpoint ID
    pub id: u64,
    /// Block number at checkpoint
    pub block_number: u64,
    /// Modified accounts
    pub modified_accounts: Vec<Address>,
    /// Modified storage
    pub modified_storage: Vec<(Address, H256)>,
}

impl PersistentState {
    /// Create new persistent state
    pub fn new(db_path: &str) -> Result<Self> {
        let db = Arc::new(
            sled::open(db_path)
                .map_err(|e| OlympusError::Database(format!("Failed to open database: {}", e)))?
        );
        
        let accounts_tree = db.open_tree("accounts")
            .map_err(|e| OlympusError::Database(format!("Failed to open accounts tree: {}", e)))?;
        
        let storage_tree = db.open_tree("storage")
            .map_err(|e| OlympusError::Database(format!("Failed to open storage tree: {}", e)))?;
        
        let code_tree = db.open_tree("code")
            .map_err(|e| OlympusError::Database(format!("Failed to open code tree: {}", e)))?;

        Ok(Self {
            db,
            accounts_tree,
            storage_tree,
            code_tree,
        })
    }

    /// Create checkpoint
    pub fn create_checkpoint(&self, block_number: u64) -> Result<StateCheckpoint> {
        Ok(StateCheckpoint {
            id: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            block_number,
            modified_accounts: Vec::new(),
            modified_storage: Vec::new(),
        })
    }

    /// Serialize account info
    fn serialize_account(&self, account: &AccountInfo) -> Result<Vec<u8>> {
        bincode::serialize(account)
            .map_err(|e| OlympusError::Serialization(format!("Failed to serialize account: {}", e)))
    }

    /// Deserialize account info
    fn deserialize_account(&self, data: &[u8]) -> Result<AccountInfo> {
        bincode::deserialize(data)
            .map_err(|e| OlympusError::Serialization(format!("Failed to deserialize account: {}", e)))
    }

    /// Get account key
    fn account_key(&self, address: Address) -> Vec<u8> {
        address.as_bytes().to_vec()
    }

    /// Get storage key
    fn storage_key(&self, address: Address, key: H256) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(address.as_bytes());
        result.extend_from_slice(key.as_bytes());
        result
    }

    /// Get code key
    fn code_key(&self, address: Address) -> Vec<u8> {
        address.as_bytes().to_vec()
    }
}

impl State for PersistentState {
    fn get_balance(&self, address: Address) -> U256 {
        let key = self.account_key(address);
        if let Ok(Some(data)) = self.accounts_tree.get(&key) {
            if let Ok(account) = self.deserialize_account(&data) {
                return account.balance;
            }
        }
        U256::zero()
    }
    
    fn set_balance(&mut self, address: Address, balance: U256) {
        let key = self.account_key(address);
        let mut account = if let Ok(Some(data)) = self.accounts_tree.get(&key) {
            self.deserialize_account(&data).unwrap_or_default()
        } else {
            AccountInfo {
                balance: U256::zero(),
                nonce: 0,
                code_hash: H256::zero(),
                storage_root: H256::zero(),
            }
        };
        
        account.balance = balance;
        
        if let Ok(data) = self.serialize_account(&account) {
            let _ = self.accounts_tree.insert(&key, data);
        }
    }
    
    fn get_nonce(&self, address: Address) -> u64 {
        let key = self.account_key(address);
        if let Ok(Some(data)) = self.accounts_tree.get(&key) {
            if let Ok(account) = self.deserialize_account(&data) {
                return account.nonce;
            }
        }
        0
    }
    
    fn set_nonce(&mut self, address: Address, nonce: u64) {
        let key = self.account_key(address);
        let mut account = if let Ok(Some(data)) = self.accounts_tree.get(&key) {
            self.deserialize_account(&data).unwrap_or_default()
        } else {
            AccountInfo {
                balance: U256::zero(),
                nonce: 0,
                code_hash: H256::zero(),
                storage_root: H256::zero(),
            }
        };
        
        account.nonce = nonce;
        
        if let Ok(data) = self.serialize_account(&account) {
            let _ = self.accounts_tree.insert(&key, data);
        }
    }
    
    fn get_storage(&self, address: Address, key: H256) -> Option<H256> {
        let storage_key = self.storage_key(address, key);
        if let Ok(Some(data)) = self.storage_tree.get(&storage_key) {
            if data.len() == 32 {
                let mut hash_bytes = [0u8; 32];
                hash_bytes.copy_from_slice(&data);
                return Some(H256::from(hash_bytes));
            }
        }
        None
    }
    
    fn set_storage(&mut self, address: Address, key: H256, value: H256) {
        let storage_key = self.storage_key(address, key);
        let _ = self.storage_tree.insert(&storage_key, value.as_bytes());
    }
    
    fn exists(&self, address: Address) -> bool {
        let key = self.account_key(address);
        self.accounts_tree.contains_key(&key).unwrap_or(false)
    }
    
    fn create_account(&mut self, address: Address) {
        let key = self.account_key(address);
        let account = AccountInfo {
            balance: U256::zero(),
            nonce: 0,
            code_hash: H256::zero(),
            storage_root: H256::zero(),
        };
        
        if let Ok(data) = self.serialize_account(&account) {
            let _ = self.accounts_tree.insert(&key, data);
        }
    }
    
    fn delete_account(&mut self, address: Address) {
        let key = self.account_key(address);
        let _ = self.accounts_tree.remove(&key);
        
        // Remove all storage entries for this address
        let prefix = address.as_bytes();
        let _ = self.storage_tree.scan_prefix(prefix).for_each(|item| {
            if let Ok((key, _)) = item {
                let _ = self.storage_tree.remove(&key);
            }
        });
        
        // Remove code
        let code_key = self.code_key(address);
        let _ = self.code_tree.remove(&code_key);
    }
    
    fn commit(&mut self) {
        let _ = self.accounts_tree.flush();
        let _ = self.storage_tree.flush();
        let _ = self.code_tree.flush();
    }
    
    fn revert(&mut self) {
        // For persistent state, revert is more complex
        // In a full implementation, you would restore from checkpoint
        // For now, this is a no-op
    }
}

impl Default for AccountInfo {
    fn default() -> Self {
        Self {
            balance: U256::zero(),
            nonce: 0,
            code_hash: H256::zero(),
            storage_root: H256::zero(),
        }
    }
}

/// State manager for handling multiple state instances
pub struct StateManager {
    /// Current state
    current_state: Box<dyn State>,
    /// State checkpoints
    checkpoints: Vec<StateCheckpoint>,
}

impl StateManager {
    /// Create new state manager
    pub fn new(state: Box<dyn State>) -> Self {
        Self {
            current_state: state,
            checkpoints: Vec::new(),
        }
    }

    /// Get current state
    pub fn state(&self) -> &dyn State {
        self.current_state.as_ref()
    }

    /// Get mutable current state
    pub fn state_mut(&mut self) -> &mut dyn State {
        self.current_state.as_mut()
    }

    /// Create checkpoint
    pub fn create_checkpoint(&mut self, block_number: u64) -> Result<u64> {
        let checkpoint = StateCheckpoint {
            id: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
            block_number,
            modified_accounts: Vec::new(),
            modified_storage: Vec::new(),
        };
        
        let checkpoint_id = checkpoint.id;
        self.checkpoints.push(checkpoint);
        Ok(checkpoint_id)
    }

    /// Revert to checkpoint
    pub fn revert_to_checkpoint(&mut self, checkpoint_id: u64) -> Result<()> {
        if let Some(pos) = self.checkpoints.iter().position(|c| c.id == checkpoint_id) {
            self.checkpoints.truncate(pos);
            self.current_state.revert();
            Ok(())
        } else {
            Err(OlympusError::Database("Checkpoint not found".to_string()))
        }
    }

    /// Commit all changes
    pub fn commit(&mut self) {
        self.current_state.commit();
        self.checkpoints.clear();
    }
}
