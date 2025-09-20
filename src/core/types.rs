//! Core types and constants

use ethereum_types::{Address, H256, U256};
use serde::{Deserialize, Serialize};

/// Olympus chain ID
pub const CHAIN_ID: u64 = 970;

/// Block hash type
pub type BlockHash = H256;

/// Transaction hash type  
pub type TransactionHash = H256;

/// Approve hash type
pub type ApproveHash = H256;

/// Epoch number
pub type Epoch = u64;

/// Main Chain Index
pub type Mci = u64;

/// Stable index
pub type StableIndex = u64;

/// Gas limit for blocks
pub const DEFAULT_GAS_LIMIT: u64 = 50_000_000;

/// Gas price in wei
pub const DEFAULT_GAS_PRICE: u64 = 10_000_000;

/// Witness parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessParam {
    pub min_witnesses: u64,
    pub max_witnesses: u64,
    pub epoch_period: u64,
}

impl Default for WitnessParam {
    fn default() -> Self {
        Self {
            min_witnesses: 7,
            max_witnesses: 14,
            epoch_period: 10000,
        }
    }
}

/// Block status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockStatus {
    Success = 0,
    DoubleSpending = 1,
    Invalid = 2,
    ContractExecutionFailed = 3,
}

/// Block state content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStateContent {
    pub level: u64,
    pub witnessed_level: u64,
    pub best_parent: BlockHash,
}

/// Stable block state content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StableBlockStateContent {
    pub status: BlockStatus,
    pub stable_index: StableIndex,
    pub stable_timestamp: u64,
    pub mci: Mci,
    pub mc_timestamp: u64,
    pub is_on_mc: bool,
    pub is_free: bool,
}

/// Block state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockState {
    pub content: BlockStateContent,
    pub is_stable: bool,
    pub stable_content: Option<StableBlockStateContent>,
}

/// Block summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSummary {
    pub summaries: H256,
    pub previous_summary: H256,
    pub parent_summaries: Vec<H256>,
    pub skiplist_summaries: Vec<H256>,
    pub status: BlockStatus,
}

/// Advance info for consensus
#[derive(Debug, Clone, Default)]
pub struct AdvanceInfo {
    pub last_mci: Mci,
    pub last_stable_mci: Mci,
    pub min_retrievable_mci: Mci,
    pub last_stable_index: StableIndex,
}

/// Minimum witnessed level result
#[derive(Debug, Clone)]
pub struct MinWlResult {
    pub min_wl: u64,
    pub block_hash: BlockHash,
}

/// Block signature structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub v: u8,
    pub r: H256,
    pub s: H256,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub trace_address: Vec<u32>,
    pub subtraces: u32,
    pub trace_type: TraceType,
    pub action: TraceAction,
    pub result: Option<TraceResult>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceType {
    Call = 0,
    Create = 1,
    Suicide = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceAction {
    Call {
        call_type: String,
        from: Address,
        to: Address,
        gas: U256,
        data: Vec<u8>,
        amount: U256,
    },
    Create {
        from: Address,
        gas: U256,
        init: Vec<u8>,
        amount: U256,
    },
    Suicide {
        contract_account: Address,
        refund_account: Address,
        balance: U256,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceResult {
    Call {
        gas_used: U256,
        output: Vec<u8>,
    },
    Create {
        gas_used: U256,
        contract_account: Address,
        code: Vec<u8>,
    },
    Suicide,
}
