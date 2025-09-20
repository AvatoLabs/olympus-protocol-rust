//! DAG consensus implementation

use crate::{Address, H256, Result, OlympusError};
use crate::core::block::Block;
use crate::consensus::witness::WitnessManager;
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

/// DAG consensus engine
pub struct DagConsensus {
    /// Current epoch
    pub current_epoch: u64,
    /// Witnesses for current epoch
    pub witnesses: Vec<Address>,
    /// Witness manager
    pub witness_manager: WitnessManager,
    /// Block DAG
    pub dag: BlockDag,
    /// Confirmation threshold
    pub confirmation_threshold: u64,
    /// Epoch duration in blocks
    pub epoch_duration: u64,
}

/// Block DAG structure
#[derive(Debug, Clone)]
pub struct BlockDag {
    /// All blocks in the DAG
    pub blocks: HashMap<H256, Block>,
    /// Block references (parents)
    pub references: HashMap<H256, Vec<H256>>,
    /// Block approvals
    pub approvals: HashMap<H256, Vec<H256>>,
    /// Confirmed blocks
    pub confirmed: HashSet<H256>,
    /// Stable blocks
    pub stable: HashSet<H256>,
    /// Maximum number of blocks to keep in memory
    pub max_blocks: usize,
}

/// Consensus result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusResult {
    /// Whether consensus was reached
    pub consensus_reached: bool,
    /// Confirmed blocks
    pub confirmed_blocks: Vec<H256>,
    /// Stable blocks
    pub stable_blocks: Vec<H256>,
    /// Next epoch witnesses
    pub next_witnesses: Vec<Address>,
}

impl DagConsensus {
    /// Create new DAG consensus engine with configurable parameters
    pub fn new(min_witnesses: u64, max_witnesses: u64, confirmation_threshold: u64, epoch_duration: u64) -> Self {
        Self {
            current_epoch: 0,
            witnesses: Vec::new(),
            witness_manager: WitnessManager::new(min_witnesses, max_witnesses),
            dag: BlockDag::new_default(),
            confirmation_threshold,
            epoch_duration,
        }
    }
    
    /// Create new DAG consensus engine with default parameters
    pub fn new_default() -> Self {
        Self::new(3, 21, 2, 100)
    }

    /// Process new block
    pub fn process_block(&mut self, block: Block) -> Result<ConsensusResult> {
        let block_hash = block.hash();
        
        // Add block to DAG
        self.dag.add_block(block_hash, block)?;
        
        // Update references and approvals
        self.update_dag_structure(&block_hash)?;
        
        // Check for consensus
        let consensus_result = self.check_consensus()?;
        
        // Update epoch if necessary
        if self.should_update_epoch() {
            self.update_epoch()?;
        }
        
        Ok(consensus_result)
    }

    /// Update DAG structure with new block
    fn update_dag_structure(&mut self, block_hash: &H256) -> Result<()> {
        if let Some(block) = self.dag.blocks.get(block_hash) {
            // Add parent references
            for parent in &block.parents {
                self.dag.references.entry(*block_hash)
                    .or_insert_with(Vec::new)
                    .push(*parent);
            }
            
            // Add approvals
            for approval in &block.approves {
                self.dag.approvals.entry(*block_hash)
                    .or_insert_with(Vec::new)
                    .push(*approval);
            }
        }
        
        Ok(())
    }

    /// Check for consensus
    fn check_consensus(&mut self) -> Result<ConsensusResult> {
        let mut confirmed_blocks = Vec::new();
        let mut stable_blocks = Vec::new();
        
        // Find blocks that can be confirmed
        for (block_hash, _block) in &self.dag.blocks {
            if self.dag.confirmed.contains(block_hash) {
                continue;
            }
            
            // Check if block has enough confirmations
            if self.has_enough_confirmations(*block_hash) {
                self.dag.confirmed.insert(*block_hash);
                confirmed_blocks.push(*block_hash);
                
                // Check if block can be marked as stable
                if self.can_be_stable(*block_hash) {
                    self.dag.stable.insert(*block_hash);
                    stable_blocks.push(*block_hash);
                }
            }
        }
        
        // Determine next epoch witnesses based on stable blocks
        let next_witnesses = self.select_next_witnesses(&stable_blocks)?;
        
        Ok(ConsensusResult {
            consensus_reached: !confirmed_blocks.is_empty(),
            confirmed_blocks,
            stable_blocks,
            next_witnesses,
        })
    }

    /// Check if block has enough confirmations
    fn has_enough_confirmations(&self, block_hash: H256) -> bool {
        if let Some(approvals) = self.dag.approvals.get(&block_hash) {
            approvals.len() >= self.confirmation_threshold as usize
        } else {
            false
        }
    }

    /// Check if block can be marked as stable
    fn can_be_stable(&self, block_hash: H256) -> bool {
        // A block is stable if it's confirmed and all its dependencies are stable
        if !self.dag.confirmed.contains(&block_hash) {
            return false;
        }
        
        if let Some(references) = self.dag.references.get(&block_hash) {
            for reference in references {
                if !self.dag.stable.contains(reference) {
                    return false;
                }
            }
        }
        
        true
    }

    /// Select next epoch witnesses
    fn select_next_witnesses(&self, stable_blocks: &[H256]) -> Result<Vec<Address>> {
        // Simple witness selection based on block creators
        let mut witness_candidates = HashMap::new();
        
        for block_hash in stable_blocks {
            if let Some(block) = self.dag.blocks.get(block_hash) {
                let count = witness_candidates.entry(block.from).or_insert(0);
                *count += 1;
            }
        }
        
        // Select top witnesses by block count
        let mut candidates: Vec<_> = witness_candidates.into_iter().collect();
        candidates.sort_by(|a, b| b.1.cmp(&a.1));
        
        let mut witnesses = Vec::new();
        for (address, _) in candidates.into_iter().take(self.witness_manager.max_witnesses as usize) {
            witnesses.push(address);
        }
        
        // Ensure minimum witnesses
        while witnesses.len() < self.witness_manager.min_witnesses as usize {
            // Add default witnesses if not enough candidates
            witnesses.push(Address::zero());
        }
        
        Ok(witnesses)
    }

    /// Check if epoch should be updated
    fn should_update_epoch(&self) -> bool {
        self.dag.stable.len() >= self.epoch_duration as usize
    }

    /// Update epoch
    fn update_epoch(&mut self) -> Result<()> {
        self.current_epoch += 1;
        
        // Clear old blocks to prevent memory growth
        self.dag.clear_old_blocks();
        
        Ok(())
    }

    /// Get stable blocks
    pub fn get_stable_blocks(&self) -> Vec<H256> {
        self.dag.stable.iter().cloned().collect()
    }

    /// Get confirmed blocks
    pub fn get_confirmed_blocks(&self) -> Vec<H256> {
        self.dag.confirmed.iter().cloned().collect()
    }

    /// Check if block is stable
    pub fn is_stable(&self, block_hash: H256) -> bool {
        self.dag.stable.contains(&block_hash)
    }

    /// Check if block is confirmed
    pub fn is_confirmed(&self, block_hash: H256) -> bool {
        self.dag.confirmed.contains(&block_hash)
    }
}

impl BlockDag {
    /// Create new block DAG
    pub fn new(max_blocks: usize) -> Self {
        Self {
            blocks: HashMap::new(),
            references: HashMap::new(),
            approvals: HashMap::new(),
            confirmed: HashSet::new(),
            stable: HashSet::new(),
            max_blocks,
        }
    }
    
    /// Create new block DAG with default max blocks
    pub fn new_default() -> Self {
        Self::new(1000)
    }

    /// Add block to DAG
    pub fn add_block(&mut self, block_hash: H256, block: Block) -> Result<()> {
        if self.blocks.contains_key(&block_hash) {
            return Err(OlympusError::Consensus("Block already exists in DAG".to_string()));
        }
        
        self.blocks.insert(block_hash, block);
        Ok(())
    }

    /// Clear old blocks to prevent memory growth
    pub fn clear_old_blocks(&mut self) {
        // Keep only recent blocks up to max_blocks limit
        if self.blocks.len() > self.max_blocks {
            let mut to_remove = Vec::new();
            let mut count = 0;
            
            for (hash, _) in &self.blocks {
                if count < self.blocks.len() - self.max_blocks {
                    to_remove.push(*hash);
                    count += 1;
                } else {
                    break;
                }
            }
            
            for hash in to_remove {
                self.blocks.remove(&hash);
                self.references.remove(&hash);
                self.approvals.remove(&hash);
                self.confirmed.remove(&hash);
                self.stable.remove(&hash);
            }
        }
    }

    /// Get block by hash
    pub fn get_block(&self, block_hash: H256) -> Option<&Block> {
        self.blocks.get(&block_hash)
    }

    /// Get block references
    pub fn get_references(&self, block_hash: H256) -> Vec<H256> {
        self.references.get(&block_hash).cloned().unwrap_or_default()
    }

    /// Get block approvals
    pub fn get_approvals(&self, block_hash: H256) -> Vec<H256> {
        self.approvals.get(&block_hash).cloned().unwrap_or_default()
    }
}

impl Default for DagConsensus {
    fn default() -> Self {
        Self::new_default()
    }
}

impl Default for BlockDag {
    fn default() -> Self {
        Self::new_default()
    }
}
