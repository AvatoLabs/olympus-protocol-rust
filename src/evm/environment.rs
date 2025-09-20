//! Enhanced EVM execution environment

use crate::{Address, H256, U256, Result, OlympusError};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// EVM execution environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmEnv {
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
    /// Coinbase address (miner/validator)
    pub coinbase: Address,
    /// Difficulty (for PoW chains)
    pub difficulty: U256,
    /// Chain ID
    pub chain_id: u64,
}

/// Gas management for EVM execution
#[derive(Debug, Clone)]
pub struct GasManager {
    /// Gas limit for the transaction
    pub gas_limit: U256,
    /// Gas used so far
    pub gas_used: U256,
    /// Gas refunded
    pub gas_refunded: U256,
    /// Gas price
    pub gas_price: U256,
}

impl GasManager {
    /// Create new gas manager
    pub fn new(gas_limit: U256, gas_price: U256) -> Self {
        Self {
            gas_limit,
            gas_used: U256::zero(),
            gas_refunded: U256::zero(),
            gas_price,
        }
    }

    /// Consume gas
    pub fn consume_gas(&mut self, amount: U256) -> Result<()> {
        if self.gas_used + amount > self.gas_limit {
            return Err(OlympusError::EvmExecution("Out of gas".to_string()));
        }
        self.gas_used += amount;
        Ok(())
    }

    /// Refund gas
    pub fn refund_gas(&mut self, amount: U256) {
        self.gas_refunded += amount;
    }

    /// Get remaining gas
    pub fn remaining_gas(&self) -> U256 {
        if self.gas_used > self.gas_limit {
            U256::zero()
        } else {
            self.gas_limit - self.gas_used
        }
    }

    /// Get total gas cost
    pub fn total_cost(&self) -> U256 {
        self.gas_used * self.gas_price
    }
}

/// EVM execution context
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Current execution depth
    pub depth: u32,
    /// Maximum execution depth
    pub max_depth: u32,
    /// Environment information
    pub env: EvmEnv,
    /// Gas manager
    pub gas_manager: GasManager,
    /// Call stack
    pub call_stack: Vec<CallFrame>,
    /// Current call frame
    pub current_frame: Option<CallFrame>,
}

/// Call frame for EVM execution
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Caller address
    pub caller: Address,
    /// Callee address
    pub callee: Address,
    /// Value being transferred
    pub value: U256,
    /// Input data
    pub input_data: Vec<u8>,
    /// Gas limit for this call
    pub gas_limit: U256,
    /// Call depth
    pub depth: u32,
    /// Is this a contract creation call
    pub is_creation: bool,
}

impl ExecutionContext {
    /// Create new execution context
    pub fn new(env: EvmEnv, gas_limit: U256, gas_price: U256) -> Self {
        Self {
            depth: 0,
            max_depth: 1024, // Ethereum's max call depth
            env,
            gas_manager: GasManager::new(gas_limit, gas_price),
            call_stack: Vec::new(),
            current_frame: None,
        }
    }

    /// Push new call frame
    pub fn push_call_frame(&mut self, frame: CallFrame) -> Result<()> {
        if self.depth >= self.max_depth {
            return Err(OlympusError::EvmExecution("Maximum call depth exceeded".to_string()));
        }
        
        self.depth += 1;
        self.call_stack.push(frame.clone());
        self.current_frame = Some(frame);
        Ok(())
    }

    /// Pop call frame
    pub fn pop_call_frame(&mut self) -> Option<CallFrame> {
        if let Some(frame) = self.call_stack.pop() {
            self.depth -= 1;
            self.current_frame = self.call_stack.last().cloned();
            Some(frame)
        } else {
            None
        }
    }

    /// Get current call frame
    pub fn current_frame(&self) -> Option<&CallFrame> {
        self.current_frame.as_ref()
    }

    /// Update environment
    pub fn update_env(&mut self, env: EvmEnv) {
        self.env = env;
    }

    /// Get gas cost for operation
    pub fn get_gas_cost(&self, operation: &str) -> U256 {
        match operation {
            "ADD" | "SUB" | "MUL" | "DIV" | "MOD" | "ADDMOD" | "MULMOD" => U256::from(3),
            "LT" | "GT" | "SLT" | "SGT" | "EQ" => U256::from(3),
            "AND" | "OR" | "XOR" => U256::from(3),
            "NOT" | "BYTE" => U256::from(3),
            "SHA3" => U256::from(30),
            "SLOAD" => U256::from(200),
            "SSTORE" => U256::from(20000),
            "BALANCE" => U256::from(400),
            "BLOCKHASH" => U256::from(20),
            "COINBASE" | "TIMESTAMP" | "NUMBER" | "DIFFICULTY" | "GASLIMIT" => U256::from(2),
            "POP" => U256::from(2),
            "MLOAD" => U256::from(3),
            "MSTORE" => U256::from(3),
            "MSTORE8" => U256::from(3),
            "JUMP" => U256::from(8),
            "JUMPI" => U256::from(10),
            "PC" => U256::from(2),
            "MSIZE" => U256::from(2),
            "GAS" => U256::from(2),
            "JUMPDEST" => U256::from(1),
            "PUSH1" | "PUSH2" | "PUSH3" | "PUSH4" | "PUSH5" | "PUSH6" | "PUSH7" | "PUSH8" => U256::from(3),
            "PUSH9" | "PUSH10" | "PUSH11" | "PUSH12" | "PUSH13" | "PUSH14" | "PUSH15" | "PUSH16" => U256::from(3),
            "PUSH17" | "PUSH18" | "PUSH19" | "PUSH20" | "PUSH21" | "PUSH22" | "PUSH23" | "PUSH24" => U256::from(3),
            "PUSH25" | "PUSH26" | "PUSH27" | "PUSH28" | "PUSH29" | "PUSH30" | "PUSH31" | "PUSH32" => U256::from(3),
            "DUP1" | "DUP2" | "DUP3" | "DUP4" | "DUP5" | "DUP6" | "DUP7" | "DUP8" => U256::from(3),
            "DUP9" | "DUP10" | "DUP11" | "DUP12" | "DUP13" | "DUP14" | "DUP15" | "DUP16" => U256::from(3),
            "SWAP1" | "SWAP2" | "SWAP3" | "SWAP4" | "SWAP5" | "SWAP6" | "SWAP7" | "SWAP8" => U256::from(3),
            "SWAP9" | "SWAP10" | "SWAP11" | "SWAP12" | "SWAP13" | "SWAP14" | "SWAP15" | "SWAP16" => U256::from(3),
            "LOG0" => U256::from(375),
            "LOG1" => U256::from(750),
            "LOG2" => U256::from(1125),
            "LOG3" => U256::from(1500),
            "LOG4" => U256::from(1875),
            "CREATE" => U256::from(32000),
            "CALL" => U256::from(700),
            "CALLCODE" => U256::from(700),
            "RETURN" => U256::from(0),
            "DELEGATECALL" => U256::from(700),
            "CREATE2" => U256::from(32000),
            "STATICCALL" => U256::from(700),
            "REVERT" => U256::from(0),
            "SELFDESTRUCT" => U256::from(5000),
            _ => U256::from(1), // Default gas cost
        }
    }
}

impl Default for EvmEnv {
    fn default() -> Self {
        Self {
            block_number: U256::zero(),
            timestamp: U256::zero(),
            block_hash: H256::zero(),
            block_gas_limit: U256::from(30_000_000),
            base_fee: U256::from(1_000_000_000), // 1 gwei
            coinbase: Address::zero(),
            difficulty: U256::zero(),
            chain_id: 1,
        }
    }
}

/// Environment EVM execution result
#[derive(Debug, Clone)]
pub struct EnvironmentEvmExecutionResult {
    /// Success status
    pub success: bool,
    /// Gas used
    pub gas_used: U256,
    /// Gas refunded
    pub gas_refunded: U256,
    /// Output data
    pub output: Vec<u8>,
    /// Logs emitted
    pub logs: Vec<EnvironmentLogEntry>,
    /// Contract address (for contract creation)
    pub contract_address: Option<Address>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Execution trace
    pub trace: Vec<TraceEntry>,
}

/// Log entry for EVM execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentLogEntry {
    /// Address that emitted the log
    pub address: Address,
    /// Topics
    pub topics: Vec<H256>,
    /// Data
    pub data: Vec<u8>,
}

/// Trace entry for EVM execution
#[derive(Debug, Clone)]
pub struct TraceEntry {
    /// Program counter
    pub pc: usize,
    /// Opcode
    pub opcode: String,
    /// Gas cost
    pub gas_cost: U256,
    /// Stack state
    pub stack: Vec<U256>,
    /// Memory state
    pub memory: Vec<u8>,
    /// Storage changes
    pub storage: HashMap<H256, H256>,
}
