//! Transaction execution logic

use crate::core::transaction::Transaction;
use crate::{Address, H256, U256, Result, OlympusError};
use crate::evm::{Executive, State};
use crate::evm::executive::EvmExecutionResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Transaction execution context
#[derive(Debug, Clone)]
pub struct TransactionExecutionContext {
    /// Block number
    pub block_number: U256,
    /// Block timestamp
    pub timestamp: U256,
    /// Block hash
    pub block_hash: H256,
    /// Gas limit for the block
    pub block_gas_limit: U256,
    /// Base fee per gas
    pub base_fee: U256,
}

/// Transaction execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionExecutionResult {
    /// Transaction hash
    pub transaction_hash: H256,
    /// Gas used
    pub gas_used: U256,
    /// Gas price paid
    pub gas_price: U256,
    /// Success status
    pub success: bool,
    /// Output data
    pub output: Vec<u8>,
    /// Logs emitted
    pub logs: Vec<TransactionLogEntry>,
    /// Contract address (for contract creation)
    pub contract_address: Option<Address>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLogEntry {
    /// Address that emitted the log
    pub address: Address,
    /// Topics
    pub topics: Vec<H256>,
    /// Data
    pub data: Vec<u8>,
}

/// Transaction executor
pub struct TransactionExecutor {
    /// EVM executive
    executive: Executive,
    /// State manager
    state_manager: Box<dyn State>,
    /// Execution context
    context: TransactionExecutionContext,
    /// Transaction pool
    transaction_pool: HashMap<H256, Transaction>,
}

impl TransactionExecutor {
    /// Create new transaction executor
    pub fn new(state_manager: Box<dyn State>, context: TransactionExecutionContext) -> Self {
        Self {
            executive: Executive::new(),
            state_manager,
            context,
            transaction_pool: HashMap::new(),
        }
    }

    /// Execute a single transaction
    pub fn execute_transaction(&mut self, transaction: Transaction) -> Result<TransactionExecutionResult> {
        let transaction_hash = transaction.hash();
        
        // Validate transaction
        self.validate_transaction(&transaction)?;
        
        // Check nonce
        let sender_nonce = self.state_manager.get_nonce(transaction.from());
        if transaction.nonce() != U256::from(sender_nonce) {
            return Err(OlympusError::InvalidTransaction(
                format!("Invalid nonce: expected {}, got {}", sender_nonce, transaction.nonce())
            ));
        }
        
        // Check balance
        let sender_balance = self.state_manager.get_balance(transaction.from());
        let total_cost = transaction.value() + (transaction.gas() * transaction.gas_price());
        if sender_balance < total_cost {
            return Err(OlympusError::InvalidTransaction(
                format!("Insufficient balance: required {}, available {}", total_cost, sender_balance)
            ));
        }
        
        // Initialize EVM executive
        self.executive.initialize(&transaction, self.context.block_number, self.context.timestamp)?;
        
        // Execute transaction
        let evm_result = self.executive.execute(&transaction)?;
        
        // Update state if successful
        if evm_result.success {
            self.update_state_after_transaction(&transaction, &evm_result)?;
        }
        
        // Create execution result
        let result = TransactionExecutionResult {
            transaction_hash,
            gas_used: evm_result.gas_used,
            gas_price: transaction.gas_price(),
            success: evm_result.success,
            output: evm_result.output,
            logs: vec![], // TODO: Extract logs from EVM execution
            contract_address: if transaction.receive_address == Address::zero() {
                Some(self.calculate_contract_address(&transaction))
            } else {
                None
            },
            error: if evm_result.success { None } else { Some("Transaction execution failed".to_string()) },
        };
        
        Ok(result)
    }

    /// Execute multiple transactions in a block
    pub fn execute_block_transactions(&mut self, transactions: Vec<Transaction>) -> Result<Vec<TransactionExecutionResult>> {
        let mut results = Vec::new();
        
        for transaction in transactions {
            match self.execute_transaction(transaction) {
                Ok(result) => results.push(result),
                Err(e) => {
                    // Log error but continue with other transactions
                    eprintln!("Transaction execution failed: {}", e);
                    // Create failed result
                    results.push(TransactionExecutionResult {
                        transaction_hash: H256::zero(),
                        gas_used: U256::zero(),
                        gas_price: U256::zero(),
                        success: false,
                        output: vec![],
                        logs: vec![],
                        contract_address: None,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        
        Ok(results)
    }

    /// Validate transaction
    fn validate_transaction(&self, transaction: &Transaction) -> Result<()> {
        // Check gas limit
        if transaction.gas() > self.context.block_gas_limit {
            return Err(OlympusError::InvalidTransaction(
                format!("Gas limit exceeds block limit: {} > {}", transaction.gas(), self.context.block_gas_limit)
            ));
        }
        
        // Check gas price
        if transaction.gas_price() < self.context.base_fee {
            return Err(OlympusError::InvalidTransaction(
                format!("Gas price too low: {} < {}", transaction.gas_price(), self.context.base_fee)
            ));
        }
        
        // Check transaction size
        let tx_size = transaction.rlp_bytes(crate::core::transaction::IncludeSignature::WithoutSignature).len();
        if tx_size > 128 * 1024 { // 128KB limit
            return Err(OlympusError::InvalidTransaction(
                format!("Transaction too large: {} bytes", tx_size)
            ));
        }
        
        Ok(())
    }

    /// Update state after successful transaction
    fn update_state_after_transaction(&mut self, transaction: &Transaction, evm_result: &EvmExecutionResult) -> Result<()> {
        // Update sender nonce
        let sender_nonce = self.state_manager.get_nonce(transaction.from());
        self.state_manager.set_nonce(transaction.from(), sender_nonce + 1);
        
        // Deduct gas cost from sender
        let gas_cost = evm_result.gas_used * transaction.gas_price();
        let sender_balance = self.state_manager.get_balance(transaction.from());
        self.state_manager.set_balance(transaction.from(), sender_balance - gas_cost);
        
        // Add value to recipient (if not contract creation)
        if transaction.receive_address != Address::zero() {
            let recipient_balance = self.state_manager.get_balance(transaction.receive_address);
            self.state_manager.set_balance(transaction.receive_address, recipient_balance + transaction.value());
        }
        
        // Create account if it doesn't exist
        if !self.state_manager.exists(transaction.receive_address) && transaction.receive_address != Address::zero() {
            self.state_manager.create_account(transaction.receive_address);
        }
        
        Ok(())
    }

    /// Calculate contract address for contract creation
    fn calculate_contract_address(&self, transaction: &Transaction) -> Address {
        // Simple contract address calculation based on sender and nonce
        // In a full implementation, this would use CREATE2 or proper CREATE logic
        let mut data = Vec::new();
        data.extend_from_slice(transaction.from().as_bytes());
        data.extend_from_slice(&transaction.nonce().as_u64().to_be_bytes());
        crate::common::keccak256(&data).into()
    }

    /// Estimate gas for transaction
    pub fn estimate_gas(&mut self, transaction: &Transaction) -> Result<U256> {
        self.executive.initialize(transaction, self.context.block_number, self.context.timestamp)?;
        self.executive.estimate_gas(transaction)
    }

    /// Call contract method (read-only)
    pub fn call_contract(&mut self, from: Address, to: Address, data: Vec<u8>) -> Result<Vec<u8>> {
        self.executive.call(from, to, data)
    }

    /// Get transaction from pool
    pub fn get_transaction(&self, hash: H256) -> Option<&Transaction> {
        self.transaction_pool.get(&hash)
    }

    /// Add transaction to pool
    pub fn add_transaction_to_pool(&mut self, transaction: Transaction) {
        let hash = transaction.hash();
        self.transaction_pool.insert(hash, transaction);
    }

    /// Remove transaction from pool
    pub fn remove_transaction_from_pool(&mut self, hash: H256) {
        self.transaction_pool.remove(&hash);
    }

    /// Get transaction pool size
    pub fn pool_size(&self) -> usize {
        self.transaction_pool.len()
    }

    /// Update execution context
    pub fn update_context(&mut self, context: TransactionExecutionContext) {
        self.context = context;
    }

    /// Get current context
    pub fn get_context(&self) -> &TransactionExecutionContext {
        &self.context
    }
}

impl Default for TransactionExecutionContext {
    fn default() -> Self {
        Self {
            block_number: U256::zero(),
            timestamp: U256::zero(),
            block_hash: H256::zero(),
            block_gas_limit: U256::from(30_000_000), // 30M gas limit
            base_fee: U256::from(1_000_000_000), // 1 gwei base fee
        }
    }
}

/// Transaction pool manager
pub struct TransactionPool {
    /// Pending transactions
    pending: HashMap<H256, Transaction>,
    /// Queued transactions
    queued: HashMap<H256, Transaction>,
    /// Maximum pool size
    max_size: usize,
}

impl TransactionPool {
    /// Create new transaction pool
    pub fn new(max_size: usize) -> Self {
        Self {
            pending: HashMap::new(),
            queued: HashMap::new(),
            max_size,
        }
    }

    /// Add transaction to pool
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<()> {
        let hash = transaction.hash();
        
        if self.pending.len() + self.queued.len() >= self.max_size {
            return Err(OlympusError::InvalidTransaction("Transaction pool is full".to_string()));
        }
        
        // Add to pending if gas price is high enough, otherwise to queued
        if transaction.gas_price() > U256::from(1_000_000_000) { // 1 gwei threshold
            self.pending.insert(hash, transaction);
        } else {
            self.queued.insert(hash, transaction);
        }
        
        Ok(())
    }

    /// Get pending transactions
    pub fn get_pending_transactions(&self) -> Vec<&Transaction> {
        self.pending.values().collect()
    }

    /// Get queued transactions
    pub fn get_queued_transactions(&self) -> Vec<&Transaction> {
        self.queued.values().collect()
    }

    /// Remove transaction
    pub fn remove_transaction(&mut self, hash: H256) {
        self.pending.remove(&hash);
        self.queued.remove(&hash);
    }

    /// Promote queued transactions to pending
    pub fn promote_queued_transactions(&mut self, gas_price_threshold: U256) {
        let mut to_promote = Vec::new();
        
        for (hash, transaction) in &self.queued {
            if transaction.gas_price() >= gas_price_threshold {
                to_promote.push(*hash);
            }
        }
        
        for hash in to_promote {
            if let Some(transaction) = self.queued.remove(&hash) {
                self.pending.insert(hash, transaction);
            }
        }
    }

    /// Get pool statistics
    pub fn get_statistics(&self) -> PoolStatistics {
        PoolStatistics {
            pending_count: self.pending.len(),
            queued_count: self.queued.len(),
            total_count: self.pending.len() + self.queued.len(),
            max_size: self.max_size,
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStatistics {
    pub pending_count: usize,
    pub queued_count: usize,
    pub total_count: usize,
    pub max_size: usize,
}
