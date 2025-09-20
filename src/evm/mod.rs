//! EVM integration using Rust EVM crates

pub mod executive;
pub mod precompiled;
pub mod state;
pub mod persistent_state;
pub mod transaction_executor;
pub mod environment;

// Re-export specific types to avoid conflicts
pub use executive::{Executive, EvmExecutionResult as ExecutiveEvmExecutionResult};
pub use precompiled::{PrecompiledContract, create_precompiled_registry};
pub use state::{State, MemoryState};
pub use persistent_state::{PersistentState, StateManager};
pub use transaction_executor::{TransactionExecutor, TransactionExecutionContext, TransactionLogEntry};
pub use environment::{EvmEnv, GasManager, EnvironmentLogEntry};
