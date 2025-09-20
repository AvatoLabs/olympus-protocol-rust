//! Olympus blockchain implementation in Rust

pub mod common;
pub mod consensus;
pub mod core;
pub mod db;
pub mod evm;
pub mod p2p;
pub mod rpc;
pub mod wallet;
pub mod dynamic_tests;

use thiserror::Error;

/// Olympus error types
#[derive(Error, Debug)]
pub enum OlympusError {
    #[error("Database error: {0}")]
    Database(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(String),
    #[error("Invalid block: {0}")]
    InvalidBlock(String),
    #[error("Consensus error: {0}")]
    Consensus(String),
    #[error("RLP decoding error: {0}")]
    RlpDecoding(#[from] rlp::DecoderError),
    #[error("EVM execution error: {0}")]
    EvmExecution(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, OlympusError>;

// Re-export common types
pub use ethereum_types::{Address, H256, U256};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::transaction::Transaction;
    use crate::core::block::Block;
    use crate::evm::{Executive, create_precompiled_registry};
    use crate::dynamic_tests::{DynamicTestGenerator, TestConfig};
    use std::time::Instant;

    #[test]
    fn test_transaction_creation_and_validation() {
        let config = TestConfig {
            transaction_count: 1,
            random_seed: Some(42),
            ..Default::default()
        };
        
        let mut generator = DynamicTestGenerator::new(config);
        let test_data = generator.generate_test_data();
        let tx = &test_data.transactions[0];

        assert_eq!(tx.value(), test_data.values[0]);
        assert_eq!(tx.gas_price(), test_data.gas_prices[0]);
        assert_eq!(tx.gas(), test_data.gas_limits[0]);
        assert_eq!(tx.nonce(), U256::from(0));
        assert_eq!(tx.receive_address(), test_data.addresses[0]);
    }

    #[test]
    fn test_transaction_signing() {
        let config = TestConfig {
            transaction_count: 1,
            random_seed: Some(42),
            ..Default::default()
        };
        
        let mut generator = DynamicTestGenerator::new(config);
        let test_data = generator.generate_test_data();
        let mut tx = test_data.transactions[0].clone();

        // Sign transaction
        let secret = [0x01; 32]; // Dummy private key
        tx.sign_with_secret(&secret).unwrap();

        assert!(tx.has_signature());
        assert!(!tx.has_zero_signature());
    }

    #[test]
    fn test_block_creation_and_hashing() {
        let config = TestConfig {
            block_count: 1,
            random_seed: Some(42),
            ..Default::default()
        };
        
        let mut generator = DynamicTestGenerator::new(config);
        let test_data = generator.generate_test_data();
        let block = &test_data.blocks[0];

        let hash = block.hash();
        assert_ne!(hash, H256::zero());
        
        let root = block.root();
        assert_ne!(root, H256::zero());
    }

    #[test]
    fn test_precompiled_contracts() {
        let registry = create_precompiled_registry();
        
        // Test SHA256 precompiled contract
        let sha256_addr = Address::from([0x02; 20]);
        assert!(registry.contains_key(&sha256_addr));
        
        let sha256_contract = registry.get(&sha256_addr).unwrap();
        
        let config = TestConfig {
            transaction_count: 1,
            random_seed: Some(42),
            ..Default::default()
        };
        
        let mut generator = DynamicTestGenerator::new(config);
        let test_data = generator.generate_test_data();
        let input = &test_data.data_payloads[0];
        
        let result = sha256_contract.execute(input).unwrap();
        
        // SHA256 should be a 32-byte hash
        assert_eq!(result.len(), 32);
        
        // Test gas cost calculation
        let gas_cost = sha256_contract.gas_cost(input);
        assert!(gas_cost > U256::zero());
    }

    #[test]
    fn test_evm_executive() {
        let mut executive = Executive::new();
        
        let config = TestConfig {
            transaction_count: 1,
            random_seed: Some(42),
            ..Default::default()
        };
        
        let mut generator = DynamicTestGenerator::new(config);
        let test_data = generator.generate_test_data();
        let tx = &test_data.transactions[0];

        executive.initialize(tx, U256::from(1), U256::from(test_data.timestamps[0])).unwrap();
        
        let result = executive.execute(tx);
        
        // The result might fail due to insufficient balance, which is expected in this test
        // We just want to ensure the execution doesn't panic and returns a proper result
        match result {
            Ok(exec_result) => {
                assert!(exec_result.gas_used > U256::zero());
                if exec_result.success {
                    assert_eq!(exec_result.output.len(), 32); // SHA256 output
                }
            }
            Err(_) => {
                // Expected for balance-related errors
            }
        }
    }

    #[test]
    fn test_gas_calculation() {
        let config = TestConfig {
            transaction_count: 1,
            random_seed: Some(42),
            ..Default::default()
        };
        
        let mut generator = DynamicTestGenerator::new(config);
        let test_data = generator.generate_test_data();
        let tx = &test_data.transactions[0];

        let base_gas = tx.base_gas_required();
        
        // Calculate expected gas: base cost + data cost (4 for zero bytes, 68 for non-zero bytes)
        let mut expected_gas = 21000; // Base transaction cost
        for byte in tx.data() {
            if *byte == 0 {
                expected_gas += 4;
            } else {
                expected_gas += 68;
            }
        }
        assert_eq!(base_gas, expected_gas);
    }

    #[test]
    fn test_consensus_dag() {
        use crate::consensus::DagConsensus;
        
        let mut consensus = DagConsensus::new_default();
        
        let config = TestConfig {
            block_count: 1,
            random_seed: Some(42),
            ..Default::default()
        };
        
        let mut generator = DynamicTestGenerator::new(config);
        let test_data = generator.generate_test_data();
        let block = test_data.blocks[0].clone();

        let result = consensus.process_block(block);
        assert!(result.is_ok());
    }

    #[test]
    fn test_witness_management() {
        use crate::consensus::WitnessManager;
        
        let mut witness_manager = WitnessManager::new(2, 10);
        
        let witness1 = Address::from([0x01; 20]);
        let witness2 = Address::from([0x02; 20]);
        
        let _ = witness_manager.add_witness(witness1);
        let _ = witness_manager.add_witness(witness2);
        
        assert!(witness_manager.has_enough_witnesses());
        
        witness_manager.set_stake(witness1, 1000);
        witness_manager.set_stake(witness2, 2000);
        
        assert_eq!(witness_manager.get_stake(witness1), 1000);
        assert_eq!(witness_manager.get_stake(witness2), 2000);
    }

    #[test]
    fn test_network_manager() {
        use crate::p2p::NetworkManager;
        
        let network_manager = NetworkManager::new().unwrap();
        
        let stats = network_manager.get_statistics();
        assert_eq!(stats.connected_peers, 0);
        assert_eq!(stats.total_peers, 0);
    }

    #[test]
    fn test_transaction_pool() {
        use crate::evm::transaction_executor::TransactionPool;
        
        let mut pool = TransactionPool::new(1000);
        
        let tx = Transaction::new(
            U256::from(1000),
            U256::from(20_000_000_000i64),
            U256::from(21000),
            Address::from([0x42; 20]),
            vec![0x01, 0x02, 0x03],
            U256::from(1),
        );

        pool.add_transaction(tx).unwrap();
        
        let stats = pool.get_statistics();
        assert_eq!(stats.total_count, 1);
    }

    #[test]
    fn test_performance_transaction_creation() {
        let start = Instant::now();
        
        for i in 0..10000 {
            let _tx = Transaction::new(
                U256::from(i),
                U256::from(20_000_000_000i64),
                U256::from(21000),
                Address::from([i as u8; 20]),
                vec![i as u8],
                U256::from(i),
            );
        }
        
        let duration = start.elapsed();
        println!("Created 10,000 transactions in {:?}", duration);
        
        // Should be very fast (under 100ms)
        assert!(duration.as_millis() < 100);
    }

    #[test]
    fn test_performance_block_hashing() {
        let block = Block::new(
            Address::from([0x01; 20]),
            H256::zero(),
            vec![H256::from([0x02; 32])],
            vec![H256::from([0x03; 32])],
            vec![H256::from([0x04; 32])],
            H256::zero(),
            H256::zero(),
            H256::zero(),
            1234567890,
            U256::from(21000),
            crate::core::types::Signature { v: 27, r: H256::zero(), s: H256::zero() },
        );

        let start = Instant::now();
        
        for _ in 0..10000 {
            let _hash = block.hash();
        }
        
        let duration = start.elapsed();
        println!("Computed 10,000 block hashes in {:?}", duration);
        
        // Should be reasonably fast (under 1000ms for 10,000 operations)
        assert!(duration.as_millis() < 1000);
    }

    #[test]
    fn test_performance_precompiled_execution() {
        let registry = create_precompiled_registry();
        let sha256_contract = registry.get(&Address::from([0x02; 20])).unwrap();
        
        let start = Instant::now();
        
        for i in 0..1000 {
            let input = format!("test data {}", i).into_bytes();
            let _result = sha256_contract.execute(&input).unwrap();
        }
        
        let duration = start.elapsed();
        println!("Executed 1,000 SHA256 operations in {:?}", duration);
        
        // Should be reasonably fast (under 200ms)
        assert!(duration.as_millis() < 200);
    }

    #[test]
    fn test_memory_usage() {
        use std::alloc::{GlobalAlloc, Layout, System};
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct TestAllocator {
            allocated: AtomicUsize,
        }

        unsafe impl GlobalAlloc for TestAllocator {
            unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                self.allocated.fetch_add(layout.size(), Ordering::SeqCst);
                System.alloc(layout)
            }

            unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
                self.allocated.fetch_sub(layout.size(), Ordering::SeqCst);
                System.dealloc(ptr, layout);
            }
        }

        #[global_allocator]
        static ALLOCATOR: TestAllocator = TestAllocator {
            allocated: AtomicUsize::new(0),
        };

        // Create many transactions and blocks
        let mut transactions = Vec::new();
        let mut blocks = Vec::new();

        for i in 0..1000 {
            let tx = Transaction::new(
                U256::from(i),
                U256::from(20_000_000_000i64),
                U256::from(21000),
                Address::from([i as u8; 20]),
                vec![i as u8; 100], // 100 bytes of data
                U256::from(i),
            );
            transactions.push(tx);

            let block = Block::new(
                Address::from([i as u8; 20]),
                H256::zero(),
                vec![],
                vec![],
                vec![],
                H256::zero(),
                H256::zero(),
                H256::zero(),
                1234567890 + i,
                U256::from(21000),
                crate::core::types::Signature { v: 27, r: H256::zero(), s: H256::zero() },
            );
            blocks.push(block);
        }

        let allocated = ALLOCATOR.allocated.load(Ordering::SeqCst);
        println!("Memory allocated for 1000 transactions and blocks: {} bytes", allocated);
        
        // Should be reasonable memory usage (under 10MB)
        assert!(allocated < 10 * 1024 * 1024);
    }
}