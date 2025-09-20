//! Block data structure and operations

use crate::core::types::*;
use crate::{Address, H256, U256, Result, OlympusError};
use rlp::{Rlp, RlpStream, Encodable, Decodable};
use serde::{Deserialize, Serialize};

/// Olympus block structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block creator address
    pub from: Address,
    /// Previous block hash from the same account
    pub previous: BlockHash,
    /// Parent blocks in the DAG
    pub parents: Vec<BlockHash>,
    /// Transaction hashes referenced by this block
    pub links: Vec<H256>,
    /// Approve hashes referenced by this block
    pub approves: Vec<H256>,
    /// Last summary hash
    pub last_summary: BlockHash,
    /// Last summary block hash
    pub last_summary_block: BlockHash,
    /// Last stable block hash
    pub last_stable_block: BlockHash,
    /// Execution timestamp
    pub exec_timestamp: u64,
    /// Gas used by transactions in this block
    pub gas_used: U256,
    /// Block signature
    pub signature: Signature,
}

impl Block {
    /// Create a new block
    pub fn new(
        from: Address,
        previous: BlockHash,
        parents: Vec<BlockHash>,
        links: Vec<H256>,
        approves: Vec<H256>,
        last_summary: BlockHash,
        last_summary_block: BlockHash,
        last_stable_block: BlockHash,
        exec_timestamp: u64,
        gas_used: U256,
        signature: Signature,
    ) -> Self {
        Self {
            from,
            previous,
            parents,
            links,
            approves,
            last_summary,
            last_summary_block,
            last_stable_block,
            exec_timestamp,
            gas_used,
            signature,
        }
    }

    /// Calculate block hash
    pub fn hash(&self) -> BlockHash {
        let rlp = self.rlp_bytes();
        crate::common::keccak256(&rlp)
    }

    /// Get RLP encoded bytes
    pub fn rlp_bytes(&self) -> Vec<u8> {
        let mut stream = RlpStream::new();
        self.rlp_append(&mut stream);
        stream.out().to_vec()
    }

    /// Calculate block root (Merkle root of transactions)
    pub fn root(&self) -> H256 {
        if self.links.is_empty() {
            return H256::zero();
        }
        
        // For now, return a simple hash of all links
        // In a full implementation, this would be a proper Merkle tree
        let mut data = Vec::new();
        for link in &self.links {
            data.extend_from_slice(link.as_bytes());
        }
        crate::common::keccak256(&data)
    }

    /// Validate block structure
    pub fn validate(&self) -> Result<()> {
        // Check that from address is not zero
        if self.from == Address::zero() {
            return Err(OlympusError::InvalidBlock("From address cannot be zero".to_string()));
        }

        // Check timestamp is reasonable (not too far in future/past)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if self.exec_timestamp > now + 300 { // 5 minutes tolerance
            return Err(OlympusError::InvalidBlock("Block timestamp too far in future".to_string()));
        }

        // Validate signature
        self.validate_signature()?;

        Ok(())
    }

    /// Validate block signature
    fn validate_signature(&self) -> Result<()> {
        // This is a simplified validation
        // In a full implementation, you would verify the ECDSA signature
        if self.signature.r == H256::zero() && self.signature.s == H256::zero() {
            return Err(OlympusError::InvalidBlock("Invalid signature".to_string()));
        }
        Ok(())
    }

    /// Initialize from genesis transaction
    pub fn init_from_genesis_transaction(
        from: Address,
        hashes: Vec<H256>,
        time: String,
    ) -> Result<Self> {
        let timestamp = time.parse::<u64>()
            .map_err(|_| OlympusError::InvalidBlock("Invalid timestamp format".to_string()))?;

        Ok(Self {
            from,
            previous: BlockHash::zero(),
            parents: vec![],
            links: hashes,
            approves: vec![],
            last_summary: BlockHash::zero(),
            last_summary_block: BlockHash::zero(),
            last_stable_block: BlockHash::zero(),
            exec_timestamp: timestamp,
            gas_used: U256::zero(),
            signature: Signature {
                v: 0,
                r: H256::zero(),
                s: H256::zero(),
            },
        })
    }
}

impl Encodable for Block {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(12);
        s.append(&self.from);
        s.append(&self.previous);
        s.append_list(&self.parents);
        s.append_list(&self.links);
        s.append_list(&self.approves);
        s.append(&self.last_summary);
        s.append(&self.last_summary_block);
        s.append(&self.last_stable_block);
        s.append(&self.exec_timestamp);
        s.append(&self.gas_used);
        s.append(&self.signature.v);
        s.append(&self.signature.r);
        s.append(&self.signature.s);
    }
}

impl Decodable for Block {
    fn decode(rlp: &Rlp) -> std::result::Result<Self, rlp::DecoderError> {
        if rlp.item_count()? != 12 {
            return Err(rlp::DecoderError::RlpIncorrectListLen);
        }

        Ok(Block {
            from: rlp.val_at(0)?,
            previous: rlp.val_at(1)?,
            parents: rlp.list_at(2)?,
            links: rlp.list_at(3)?,
            approves: rlp.list_at(4)?,
            last_summary: rlp.val_at(5)?,
            last_summary_block: rlp.val_at(6)?,
            last_stable_block: rlp.val_at(7)?,
            exec_timestamp: rlp.val_at(8)?,
            gas_used: rlp.val_at(9)?,
            signature: Signature {
                v: rlp.val_at(10)?,
                r: rlp.val_at(11)?,
                s: rlp.val_at(12)?,
            },
        })
    }
}

/// Localized block with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedBlock {
    /// Base block
    pub block: Block,
    /// Block number
    pub block_number: u64,
    /// Transactions in this block
    pub transactions: Vec<crate::core::transaction::Transaction>,
    /// State root
    pub state_root: H256,
    /// Receipts root
    pub receipts_root: H256,
    /// Parent block hash
    pub parent: H256,
    /// Gas used
    pub gas_used: U256,
    /// Minimum gas price
    pub min_gas_price: U256,
    /// Transactions root
    pub transactions_root: H256,
    /// Uncles hash
    pub sha3_uncles: H256,
}

impl LocalizedBlock {
    /// Create a new localized block
    pub fn new(
        block: Block,
        block_number: u64,
        transactions: Vec<crate::core::transaction::Transaction>,
        state_root: H256,
        receipts_root: H256,
        parent: H256,
    ) -> Self {
        let mut gas_used = U256::zero();
        let mut min_gas_price = U256::zero();
        
        for (i, tx) in transactions.iter().enumerate() {
            gas_used += tx.gas();
            if i == 0 {
                min_gas_price = tx.gas_price();
            } else {
                min_gas_price = min_gas_price.min(tx.gas_price());
            }
        }

        // Calculate transactions root
        let transactions_root = if transactions.is_empty() {
            H256::zero()
        } else {
            // Simplified - in full implementation would use proper Merkle tree
            let mut data = Vec::new();
            for tx in &transactions {
                data.extend_from_slice(&tx.hash().as_bytes());
            }
            crate::common::keccak256(&data)
        };

        Self {
            block,
            block_number,
            transactions,
            state_root,
            receipts_root,
            parent,
            gas_used,
            min_gas_price,
            transactions_root,
            sha3_uncles: H256::zero(), // Olympus doesn't use uncles
        }
    }

    /// Get block size in bytes
    pub fn size(&self) -> usize {
        self.block.rlp_bytes().len()
    }
}

impl Block {
    /// Get execution timestamp
    pub fn timestamp(&self) -> u64 {
        self.exec_timestamp
    }
    
    /// Get gas used by transactions in this block
    pub fn gas_used(&self) -> U256 {
        self.gas_used
    }
}
