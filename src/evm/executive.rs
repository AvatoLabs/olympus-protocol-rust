//! EVM Executive for transaction execution

use crate::core::transaction::Transaction;
use crate::{Address, H256, U256, Result, OlympusError};
use crate::evm::precompiled::{create_precompiled_registry, PrecompiledContract};
use crate::evm::environment::{ExecutionContext, EvmEnv, GasManager, EnvironmentLogEntry};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use revm::{
    handler::{MainBuilder, MainContext, ExecuteEvm},
    primitives::{U256 as RevmU256, Address as RevmAddress, TxKind, Bytes},
    context::{Context, TxEnv, BlockEnv, CfgEnv, result::{ExecResultAndState, ExecutionResult}},
    database::EmptyDB,
    state::EvmState,
};

/// EVM execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmExecutionResult {
    /// Gas used
    pub gas_used: U256,
    /// Gas refunded
    pub gas_refunded: U256,
    /// Output data
    pub output: Vec<u8>,
    /// Success status
    pub success: bool,
    /// Logs emitted
    pub logs: Vec<EnvironmentLogEntry>,
    /// Contract address (for contract creation)
    pub contract_address: Option<Address>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// EVM Executive for executing transactions
pub struct Executive {
    /// Execution context
    context: ExecutionContext,
    /// Precompiled contracts registry
    precompiled_registry: HashMap<Address, Box<dyn PrecompiledContract>>,
    /// REVM context
    revm_context: Context<BlockEnv, TxEnv, CfgEnv, EmptyDB>,
}

impl Executive {
    /// Create a new EVM executive
    pub fn new() -> Self {
        let env = EvmEnv::default();
        let context = ExecutionContext::new(env, U256::from(30_000_000), U256::from(1_000_000_000));
        
        // Initialize REVM context
        let revm_context = Context::mainnet();
        
        Self {
            context,
            precompiled_registry: create_precompiled_registry(),
            revm_context,
        }
    }

    /// Initialize executive with transaction and environment
    pub fn initialize(&mut self, transaction: &Transaction, block_number: U256, timestamp: U256) -> Result<()> {
        let env = EvmEnv {
            block_number,
            timestamp,
            block_hash: H256::zero(),
            block_gas_limit: U256::from(30_000_000),
            base_fee: U256::from(1_000_000_000),
            coinbase: Address::zero(),
            difficulty: U256::zero(),
            chain_id: 1,
        };
        
        self.context.update_env(env);
        self.context.gas_manager = GasManager::new(transaction.gas(), transaction.gas_price());
        Ok(())
    }

    /// Execute a transaction
    pub fn execute(&mut self, transaction: &Transaction) -> Result<EvmExecutionResult> {
        // Check if this is a precompiled contract call
        if self.precompiled_registry.contains_key(&transaction.receive_address) {
            return self.execute_precompiled_contract(transaction);
        }

        // Execute regular transaction using REVM
        self.execute_with_revm(transaction)
    }

    /// Execute precompiled contract
    fn execute_precompiled_contract(&mut self, transaction: &Transaction) -> Result<EvmExecutionResult> {
        let gas_cost = self.precompiled_registry.get(&transaction.receive_address)
            .unwrap()
            .gas_cost(&transaction.data);
        
        // Check if we have enough gas
        if self.context.gas_manager.remaining_gas() < gas_cost {
            return Ok(EvmExecutionResult {
                gas_used: self.context.gas_manager.gas_used,
                gas_refunded: self.context.gas_manager.gas_refunded,
                output: vec![],
                success: false,
                logs: vec![],
                contract_address: None,
                error: Some("Out of gas".to_string()),
            });
        }

        // Consume gas
        self.context.gas_manager.consume_gas(gas_cost)?;

        // Execute precompiled contract
        let output = self.precompiled_registry.get(&transaction.receive_address)
            .unwrap()
            .execute(&transaction.data)?;

        Ok(EvmExecutionResult {
            gas_used: self.context.gas_manager.gas_used,
            gas_refunded: self.context.gas_manager.gas_refunded,
            output,
            success: true,
            logs: vec![],
            contract_address: None,
            error: None,
        })
    }

    /// Execute transaction using REVM
    fn execute_with_revm(&mut self, transaction: &Transaction) -> Result<EvmExecutionResult> {
        // Convert transaction to REVM format
        let tx_env = self.convert_transaction_to_tx_env(transaction);
        
        // Update REVM context
        self.revm_context.tx = tx_env.clone();
        self.revm_context.block.number = RevmU256::from(self.context.env.block_number.as_u64());
        self.revm_context.block.timestamp = RevmU256::from(self.context.env.timestamp.as_u64());
        self.revm_context.block.beneficiary = RevmAddress::from_slice(&self.context.env.coinbase.as_bytes());
        self.revm_context.block.gas_limit = self.context.env.block_gas_limit.as_u64();
        self.revm_context.block.basefee = self.context.env.base_fee.as_u64();
        
        // Build EVM instance
        let mut evm = self.revm_context.clone().build_mainnet();
        
        // Execute transaction
        let result = evm.transact(tx_env).map_err(|e| {
            OlympusError::EvmExecution(format!("REVM execution failed: {:?}", e))
        })?;
        
        // Convert result
        self.convert_revm_result(result)
    }

    /// Convert transaction to REVM TxEnv
    fn convert_transaction_to_tx_env(&self, transaction: &Transaction) -> TxEnv {
        TxEnv {
            tx_type: 0, // Legacy transaction
            caller: RevmAddress::from_slice(&transaction.from().as_bytes()),
            gas_limit: transaction.gas().as_u64(),
            gas_price: transaction.gas_price().as_u64() as u128,
            gas_priority_fee: None,
            kind: if transaction.is_creation() {
                TxKind::Create
            } else {
                TxKind::Call(RevmAddress::from_slice(&transaction.receive_address.as_bytes()))
            },
            value: RevmU256::from(transaction.value().as_u64()),
            data: Bytes::from(transaction.data().to_vec()),
            nonce: transaction.nonce().as_u64(),
            chain_id: transaction.chain_id(),
            access_list: Default::default(),
            blob_hashes: vec![],
            max_fee_per_blob_gas: 0,
            authorization_list: vec![],
        }
    }

    /// Convert REVM result to our format
    fn convert_revm_result(&mut self, result: ExecResultAndState<ExecutionResult, EvmState>) -> Result<EvmExecutionResult> {
        let execution_result = result.result;
        
        Ok(EvmExecutionResult {
            gas_used: U256::from(execution_result.gas_used()),
            gas_refunded: U256::zero(),
            output: execution_result.output().unwrap_or(&Bytes::new()).to_vec(),
            success: execution_result.is_success(),
            logs: vec![], // TODO: Extract logs from execution result
            contract_address: execution_result.created_address().map(|addr| Address::from_slice(&addr.as_slice())),
            error: if execution_result.is_success() {
                None
            } else {
                Some(format!("Execution failed"))
            },
        })
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
        // Initialize with high gas limit for estimation
        self.context.gas_manager = GasManager::new(U256::from(100_000_000), transaction.gas_price());
        
        // Execute transaction
        let result = self.execute(transaction)?;
        
        if result.success {
            Ok(result.gas_used)
        } else {
            Err(OlympusError::EvmExecution(result.error.unwrap_or("Execution failed".to_string())))
        }
    }

    /// Call contract method (read-only)
    pub fn call(&mut self, _from: Address, to: Address, data: Vec<u8>) -> Result<Vec<u8>> {
        // Create a temporary transaction for the call
        let call_transaction = Transaction::new(
            U256::zero(), // No value transfer
            U256::from(1_000_000_000), // 1 gwei gas price
            U256::from(100_000), // Gas limit
            to,
            data,
            U256::zero(), // Nonce not important for calls
        );

        // Execute the call
        let result = self.execute(&call_transaction)?;
        
        if result.success {
            Ok(result.output)
        } else {
            Err(OlympusError::EvmExecution(result.error.unwrap_or("Call failed".to_string())))
        }
    }

    /// Get current gas usage
    pub fn gas_used(&self) -> U256 {
        self.context.gas_manager.gas_used
    }

    /// Get remaining gas
    pub fn remaining_gas(&self) -> U256 {
        self.context.gas_manager.remaining_gas()
    }

    /// Get execution context
    pub fn context(&self) -> &ExecutionContext {
        &self.context
    }

    /// Get mutable execution context
    pub fn context_mut(&mut self) -> &mut ExecutionContext {
        &mut self.context
    }
}