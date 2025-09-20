//! Sled database implementation

use crate::{Result, OlympusError};
use sled::{Db, Tree};

/// Sled database wrapper
pub struct SledDatabase {
    db: Db,
}

impl SledDatabase {
    /// Create new sled database
    pub fn new(path: &str) -> Result<Self> {
        let db = sled::open(path)
            .map_err(|e| OlympusError::Database(e.to_string()))?;
        
        Ok(Self { db })
    }

    /// Get tree by name
    pub fn tree(&self, name: &str) -> Result<Tree> {
        self.db.open_tree(name)
            .map_err(|e| OlympusError::Database(e.to_string()))
    }

    /// Insert key-value pair
    pub fn insert(&self, tree: &str, key: &[u8], value: &[u8]) -> Result<()> {
        let tree = self.tree(tree)?;
        tree.insert(key, value)
            .map_err(|e| OlympusError::Database(e.to_string()))?;
        Ok(())
    }

    /// Get value by key
    pub fn get(&self, tree: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let tree = self.tree(tree)?;
        let result = tree.get(key)
            .map_err(|e| OlympusError::Database(e.to_string()))?;
        
        Ok(result.map(|v| v.to_vec()))
    }
}
