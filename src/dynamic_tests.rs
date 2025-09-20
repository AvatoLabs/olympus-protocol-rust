use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use crate::{Address, H256, U256};
use crate::core::transaction::Transaction;
use crate::core::block::Block;
use crate::evm::{Executive, create_precompiled_registry};
use crate::core::types::Signature;

#[derive(Debug, Clone)]
pub struct TestConfig {
    pub transaction_count: usize,
    pub block_count: usize,
    pub gas_price_range: (u64, u64),
    pub value_range: (u64, u64),
    pub gas_limit_range: (u64, u64),
    pub timestamp_range: (u64, u64),
    pub data_size_range: (usize, usize),
    pub performance_iterations: usize,
    pub memory_test_size: usize,
    pub random_seed: Option<u64>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            transaction_count: 1000,
            block_count: 100,
            gas_price_range: (1_000_000_000, 50_000_000_000), // 1 gwei to 50 gwei
            value_range: (1_000, 1_000_000_000_000_000_000), // 0.001 ETH to 1 ETH
            gas_limit_range: (21_000, 1_000_000),
            timestamp_range: (1_600_000_000, 2_000_000_000), // 2020-2033
            data_size_range: (1, 1024), // 1 byte to 1KB
            performance_iterations: 10_000,
            memory_test_size: 1000,
            random_seed: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DynamicTestData {
    pub transactions: Vec<Transaction>,
    pub blocks: Vec<Block>,
    pub addresses: Vec<Address>,
    pub timestamps: Vec<u64>,
    pub gas_prices: Vec<U256>,
    pub values: Vec<U256>,
    pub gas_limits: Vec<U256>,
    pub data_payloads: Vec<Vec<u8>>,
}

pub struct DynamicTestGenerator {
    config: TestConfig,
    rng: StdRng,
}

impl DynamicTestGenerator {
    pub fn new(config: TestConfig) -> Self {
        let seed = config.random_seed.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });
        
        Self {
            config,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    pub fn generate_addresses(&mut self, count: usize) -> Vec<Address> {
        let mut addresses = Vec::new();
        for _ in 0..count {
            let mut addr_bytes = [0u8; 20];
            self.rng.fill(&mut addr_bytes);
            addresses.push(Address::from(addr_bytes));
        }
        addresses
    }

    pub fn generate_timestamps(&mut self, count: usize) -> Vec<u64> {
        let mut timestamps = Vec::new();
        for _ in 0..count {
            let timestamp = self.rng.gen_range(self.config.timestamp_range.0..=self.config.timestamp_range.1);
            timestamps.push(timestamp);
        }
        timestamps
    }

    pub fn generate_gas_prices(&mut self, count: usize) -> Vec<U256> {
        let mut gas_prices = Vec::new();
        for _ in 0..count {
            let gas_price = self.rng.gen_range(self.config.gas_price_range.0..=self.config.gas_price_range.1);
            gas_prices.push(U256::from(gas_price));
        }
        gas_prices
    }

    pub fn generate_values(&mut self, count: usize) -> Vec<U256> {
        let mut values = Vec::new();
        for _ in 0..count {
            let value = self.rng.gen_range(self.config.value_range.0..=self.config.value_range.1);
            values.push(U256::from(value));
        }
        values
    }

    pub fn generate_gas_limits(&mut self, count: usize) -> Vec<U256> {
        let mut gas_limits = Vec::new();
        for _ in 0..count {
            let gas_limit = self.rng.gen_range(self.config.gas_limit_range.0..=self.config.gas_limit_range.1);
            gas_limits.push(U256::from(gas_limit));
        }
        gas_limits
    }

    pub fn generate_data_payloads(&mut self, count: usize) -> Vec<Vec<u8>> {
        let mut payloads = Vec::new();
        for _ in 0..count {
            let size = self.rng.gen_range(self.config.data_size_range.0..=self.config.data_size_range.1);
            let mut payload = vec![0u8; size];
            for i in 0..size {
                payload[i] = self.rng.gen();
            }
            payloads.push(payload);
        }
        payloads
    }

    pub fn generate_test_data(&mut self) -> DynamicTestData {
        // Generate base data
        let addresses = self.generate_addresses(self.config.transaction_count + self.config.block_count);
        let timestamps = self.generate_timestamps(self.config.block_count);
        let gas_prices = self.generate_gas_prices(self.config.transaction_count);
        let values = self.generate_values(self.config.transaction_count);
        let gas_limits = self.generate_gas_limits(self.config.transaction_count);
        let data_payloads = self.generate_data_payloads(self.config.transaction_count);

        // Generate transactions
        let mut transactions = Vec::new();
        for i in 0..self.config.transaction_count {
            let tx = Transaction::new(
                values[i],
                gas_prices[i],
                gas_limits[i],
                addresses[i],
                data_payloads[i].clone(),
                U256::from(i as u64),
            );
            transactions.push(tx);
        }

        // Generate blocks
        let mut blocks = Vec::new();
        for i in 0..self.config.block_count {
            let block = Block::new(
                addresses[self.config.transaction_count + i],
                H256::random(),
                vec![H256::random()],
                vec![H256::random()],
                vec![H256::random()],
                H256::random(),
                H256::random(),
                H256::random(),
                timestamps[i],
                U256::from(21000),
                Signature { 
                    v: 27, 
                    r: H256::random(), 
                    s: H256::random() 
                },
            );
            blocks.push(block);
        }

        DynamicTestData {
            transactions,
            blocks,
            addresses,
            timestamps,
            gas_prices,
            values,
            gas_limits,
            data_payloads,
        }
    }
}

pub struct DynamicBenchmarkSuite {
    config: TestConfig,
    generator: DynamicTestGenerator,
}

impl DynamicBenchmarkSuite {
    pub fn new(config: TestConfig) -> Self {
        let generator = DynamicTestGenerator::new(config.clone());
        Self { config, generator }
    }

    pub fn run_transaction_creation_benchmark(&mut self) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        let start = std::time::Instant::now();
        
        let test_data = self.generator.generate_test_data();
        let transactions = test_data.transactions;
        
        let duration = start.elapsed();
        
        results.insert("transaction_count".to_string(), transactions.len() as f64);
        results.insert("execution_time_ms".to_string(), duration.as_millis() as f64);
        results.insert("average_time_per_tx_us".to_string(), 
                      duration.as_micros() as f64 / transactions.len() as f64);
        
        results
    }

    pub fn run_block_hashing_benchmark(&mut self) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        let start = std::time::Instant::now();
        
        let test_data = self.generator.generate_test_data();
        let blocks = test_data.blocks;
        
        // Calculate hashes
        let mut hashes = Vec::new();
        for block in &blocks {
            let hash = block.hash();
            hashes.push(hash);
        }
        
        let duration = start.elapsed();
        
        results.insert("block_count".to_string(), blocks.len() as f64);
        results.insert("execution_time_ms".to_string(), duration.as_millis() as f64);
        results.insert("average_time_per_hash_us".to_string(), 
                      duration.as_micros() as f64 / blocks.len() as f64);
        
        results
    }

    pub fn run_precompiled_contracts_benchmark(&mut self) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        let start = std::time::Instant::now();
        
        let registry = create_precompiled_registry();
        let sha256_addr = Address::from([0x02; 20]);
        let sha256_contract = registry.get(&sha256_addr).unwrap();
        
        let test_data = self.generator.generate_test_data();
        let mut success_count = 0;
        
        for payload in test_data.data_payloads.iter().take(100) {
            if sha256_contract.execute(payload).is_ok() {
                success_count += 1;
            }
        }
        
        let duration = start.elapsed();
        
        results.insert("contract_count".to_string(), success_count as f64);
        results.insert("execution_time_ms".to_string(), duration.as_millis() as f64);
        results.insert("average_time_per_contract_us".to_string(), 
                      duration.as_micros() as f64 / success_count as f64);
        
        results
    }

    pub fn run_evm_execution_benchmark(&mut self) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        let start = std::time::Instant::now();
        
        let mut executive = Executive::new();
        let test_data = self.generator.generate_test_data();
        let mut success_count = 0;
        
        for tx in test_data.transactions.iter().take(100) {
            if executive.initialize(tx, U256::from(1), U256::from(test_data.timestamps[0])).is_ok() {
                if executive.execute(tx).is_ok() {
                    success_count += 1;
                }
            }
        }
        
        let duration = start.elapsed();
        
        results.insert("execution_count".to_string(), success_count as f64);
        results.insert("execution_time_ms".to_string(), duration.as_millis() as f64);
        results.insert("average_time_per_execution_us".to_string(), 
                      duration.as_micros() as f64 / success_count as f64);
        results.insert("success_count".to_string(), success_count as f64);
        
        results
    }

    pub fn run_memory_usage_benchmark(&mut self) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        let start = std::time::Instant::now();
        
        let test_data = self.generator.generate_test_data();
        let transactions = test_data.transactions;
        let blocks = test_data.blocks;
        
        let duration = start.elapsed();
        
        // Estimate memory usage
        let tx_memory = transactions.len() * std::mem::size_of::<Transaction>();
        let block_memory = blocks.len() * std::mem::size_of::<Block>();
        let total_memory = tx_memory + block_memory;
        
        results.insert("transaction_count".to_string(), transactions.len() as f64);
        results.insert("block_count".to_string(), blocks.len() as f64);
        results.insert("execution_time_ms".to_string(), duration.as_millis() as f64);
        results.insert("estimated_memory_kb".to_string(), (total_memory / 1024) as f64);
        results.insert("tx_memory_kb".to_string(), (tx_memory / 1024) as f64);
        results.insert("block_memory_kb".to_string(), (block_memory / 1024) as f64);
        
        results
    }

    pub fn run_signature_verification_benchmark(&mut self) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        let start = std::time::Instant::now();
        
        let test_data = self.generator.generate_test_data();
        let mut valid_count = 0;
        
        for tx in test_data.transactions.iter().take(100) {
            let mut signed_tx = tx.clone();
            let secret = [0x01; 32]; // Simplified for testing
            if signed_tx.sign_with_secret(&secret).is_ok() && signed_tx.has_signature() {
                valid_count += 1;
            }
        }
        
        let duration = start.elapsed();
        
        results.insert("signature_count".to_string(), valid_count as f64);
        results.insert("execution_time_ms".to_string(), duration.as_millis() as f64);
        results.insert("average_time_per_signature_us".to_string(), 
                      duration.as_micros() as f64 / valid_count as f64);
        results.insert("valid_signatures".to_string(), valid_count as f64);
        
        results
    }

    pub fn run_consensus_benchmark(&mut self) -> HashMap<String, f64> {
        let mut results = HashMap::new();
        let start = std::time::Instant::now();
        
        let test_data = self.generator.generate_test_data();
        let blocks = test_data.blocks;
        
        // Simulate consensus validation
        let valid_blocks = blocks.iter().filter(|block| {
            // Simple consensus rule validation - check if block has valid data
            !block.parents.is_empty() || !block.links.is_empty()
        }).count();
        
        let duration = start.elapsed();
        
        results.insert("block_count".to_string(), blocks.len() as f64);
        results.insert("execution_time_ms".to_string(), duration.as_millis() as f64);
        results.insert("valid_blocks".to_string(), valid_blocks as f64);
        results.insert("invalid_blocks".to_string(), (blocks.len() - valid_blocks) as f64);
        
        results
    }

    pub fn run_all_benchmarks(&mut self) -> HashMap<String, HashMap<String, f64>> {
        let mut all_results = HashMap::new();
        
        all_results.insert("transaction_creation".to_string(), self.run_transaction_creation_benchmark());
        all_results.insert("block_hashing".to_string(), self.run_block_hashing_benchmark());
        all_results.insert("precompiled_contracts".to_string(), self.run_precompiled_contracts_benchmark());
        all_results.insert("evm_execution".to_string(), self.run_evm_execution_benchmark());
        all_results.insert("memory_usage".to_string(), self.run_memory_usage_benchmark());
        all_results.insert("signature_verification".to_string(), self.run_signature_verification_benchmark());
        all_results.insert("consensus".to_string(), self.run_consensus_benchmark());
        
        all_results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_test_generator() {
        let config = TestConfig {
            transaction_count: 10,
            block_count: 5,
            random_seed: Some(42),
            ..Default::default()
        };
        
        let mut generator = DynamicTestGenerator::new(config);
        let test_data = generator.generate_test_data();
        
        assert_eq!(test_data.transactions.len(), 10);
        assert_eq!(test_data.blocks.len(), 5);
        assert_eq!(test_data.addresses.len(), 15);
    }

    #[test]
    fn test_benchmark_suite() {
        let config = TestConfig {
            transaction_count: 100,
            block_count: 10,
            random_seed: Some(42),
            ..Default::default()
        };
        
        let mut suite = DynamicBenchmarkSuite::new(config);
        let results = suite.run_all_benchmarks();
        
        assert!(results.contains_key("transaction_creation"));
        assert!(results.contains_key("block_hashing"));
        assert!(results.contains_key("precompiled_contracts"));
        assert!(results.contains_key("evm_execution"));
        assert!(results.contains_key("memory_usage"));
        assert!(results.contains_key("signature_verification"));
        assert!(results.contains_key("consensus"));
    }
}
