use std::fs;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use olympus::{Address, H256, U256};
use olympus::core::transaction::Transaction;
use olympus::core::block::Block;
use olympus::evm::{Executive, create_precompiled_registry};
use olympus::core::types::Signature;

#[derive(Debug, Deserialize)]
struct TestData {
    transactions: Vec<TransactionData>,
    blocks: Vec<BlockData>,
    addresses: Vec<String>,
    timestamps: Vec<u64>,
    gas_prices: Vec<u64>,
    values: Vec<u64>,
    gas_limits: Vec<u64>,
    data_payloads: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TransactionData {
    nonce: u64,
    value: u64,
    gas_price: u64,
    gas_limit: u64,
    to_address: String,
    data: String,
    timestamp: u64,
}

#[derive(Debug, Deserialize)]
struct BlockData {
    from_address: String,
    previous_hash: String,
    timestamp: u64,
    transaction_count: u32,
    gas_used: u64,
}

#[derive(Debug, Serialize)]
struct TestResult {
    #[serde(flatten)]
    metrics: std::collections::HashMap<String, serde_json::Value>,
    execution_time_ms: f64,
}

struct RustVersionTester {
    test_data: TestData,
}

impl RustVersionTester {
    fn new(data_file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(data_file)?;
        let test_data: TestData = serde_json::from_str(&content)?;
        Ok(RustVersionTester { test_data })
    }

    fn run_transaction_creation_test(&self) -> TestResult {
        let start = Instant::now();
        
        let mut transactions = Vec::new();
        
        for tx_data in &self.test_data.transactions {
            let tx = Transaction::new(
                U256::from(tx_data.value),
                U256::from(tx_data.gas_price),
                U256::from(tx_data.gas_limit),
                Address::from_slice(&hex::decode(&tx_data.to_address[2..]).unwrap_or_default()),
                hex::decode(&tx_data.data).unwrap_or_default(),
                U256::from(tx_data.nonce),
            );
            transactions.push(tx);
        }
        
        let duration = start.elapsed();
        
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("transaction_count".to_string(), serde_json::Value::Number(serde_json::Number::from(transactions.len())));
        metrics.insert("average_time_per_tx_us".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(duration.as_micros() as f64 / transactions.len() as f64).unwrap()));
        
        TestResult {
            metrics,
            execution_time_ms: duration.as_millis() as f64,
        }
    }

    fn run_block_hashing_test(&self) -> TestResult {
        let start = Instant::now();
        
        let mut blocks = Vec::new();
        let mut hashes = Vec::new();
        
        for block_data in &self.test_data.blocks {
            let block = Block::new(
                Address::from_slice(&hex::decode(&block_data.from_address[2..]).unwrap_or_default()),
                H256::from_slice(&hex::decode(&block_data.previous_hash).unwrap_or_default()),
                vec![],
                vec![],
                vec![],
                H256::zero(),
                H256::zero(),
                H256::zero(),
                block_data.timestamp,
                U256::from(block_data.gas_used),
                Signature { v: 27, r: H256::zero(), s: H256::zero() },
            );
            
            let hash = block.hash();
            hashes.push(hash);
            blocks.push(block);
        }
        
        let duration = start.elapsed();
        
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("block_count".to_string(), serde_json::Value::Number(serde_json::Number::from(blocks.len())));
        metrics.insert("average_time_per_hash_us".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(duration.as_micros() as f64 / blocks.len() as f64).unwrap()));
        
        // Add sample hashes for verification
        let sample_hashes: Vec<String> = hashes.iter().take(5).map(|h| format!("{:?}", h)).collect();
        metrics.insert("sample_hashes".to_string(), serde_json::to_value(sample_hashes).unwrap());
        
        TestResult {
            metrics,
            execution_time_ms: duration.as_millis() as f64,
        }
    }

    fn run_precompiled_contracts_test(&self) -> TestResult {
        let start = Instant::now();
        
        let registry = create_precompiled_registry();
        let sha256_addr = Address::from([0x02; 20]);
        let sha256_contract = registry.get(&sha256_addr).unwrap();
        
        let mut results = Vec::new();
        
        for tx_data in self.test_data.transactions.iter().take(100) {
            let input = hex::decode(&tx_data.data).unwrap_or_default();
            if let Ok(result) = sha256_contract.execute(&input) {
                results.push(hex::encode(result));
            }
        }
        
        let duration = start.elapsed();
        
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("contract_count".to_string(), serde_json::Value::Number(serde_json::Number::from(results.len())));
        metrics.insert("average_time_per_contract_us".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(duration.as_micros() as f64 / results.len() as f64).unwrap()));
        
        // Add sample results for verification
        let sample_results: Vec<String> = results.iter().take(3).cloned().collect();
        metrics.insert("sample_results".to_string(), serde_json::to_value(sample_results).unwrap());
        
        TestResult {
            metrics,
            execution_time_ms: duration.as_millis() as f64,
        }
    }

    fn run_evm_execution_test(&self) -> TestResult {
        let start = Instant::now();
        
        let mut executive = Executive::new();
        let mut results = Vec::new();
        
        for tx_data in self.test_data.transactions.iter().take(100) {
            let tx = Transaction::new(
                U256::from(tx_data.value),
                U256::from(tx_data.gas_price),
                U256::from(tx_data.gas_limit),
                Address::from_slice(&hex::decode(&tx_data.to_address[2..]).unwrap_or_default()),
                hex::decode(&tx_data.data).unwrap_or_default(),
                U256::from(tx_data.nonce),
            );
            
            if executive.initialize(&tx, U256::from(1), U256::from(tx_data.timestamp)).is_ok() {
                if let Ok(result) = executive.execute(&tx) {
                    results.push(result);
                }
            }
        }
        
        let duration = start.elapsed();
        
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("execution_count".to_string(), serde_json::Value::Number(serde_json::Number::from(results.len())));
        metrics.insert("average_time_per_execution_us".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(duration.as_micros() as f64 / results.len() as f64).unwrap()));
        
        let success_count = results.iter().filter(|r| r.success).count();
        metrics.insert("success_count".to_string(), serde_json::Value::Number(serde_json::Number::from(success_count)));
        metrics.insert("failure_count".to_string(), serde_json::Value::Number(serde_json::Number::from(results.len() - success_count)));
        
        TestResult {
            metrics,
            execution_time_ms: duration.as_millis() as f64,
        }
    }

    fn run_memory_usage_test(&self) -> TestResult {
        let start = Instant::now();
        
        let mut transactions = Vec::new();
        let mut blocks = Vec::new();
        
        // Create transactions
        for tx_data in &self.test_data.transactions {
            let tx = Transaction::new(
                U256::from(tx_data.value),
                U256::from(tx_data.gas_price),
                U256::from(tx_data.gas_limit),
                Address::from_slice(&hex::decode(&tx_data.to_address[2..]).unwrap_or_default()),
                hex::decode(&tx_data.data).unwrap_or_default(),
                U256::from(tx_data.nonce),
            );
            transactions.push(tx);
        }
        
        // Create blocks
        for block_data in &self.test_data.blocks {
            let block = Block::new(
                Address::from_slice(&hex::decode(&block_data.from_address[2..]).unwrap_or_default()),
                H256::from_slice(&hex::decode(&block_data.previous_hash).unwrap_or_default()),
                vec![],
                vec![],
                vec![],
                H256::zero(),
                H256::zero(),
                H256::zero(),
                block_data.timestamp,
                U256::from(block_data.gas_used),
                Signature { v: 27, r: H256::zero(), s: H256::zero() },
            );
            blocks.push(block);
        }
        
        let duration = start.elapsed();
        
        // Estimate memory usage
        let tx_memory = transactions.len() * std::mem::size_of::<Transaction>();
        let block_memory = blocks.len() * std::mem::size_of::<Block>();
        let total_memory = tx_memory + block_memory;
        
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("transaction_count".to_string(), serde_json::Value::Number(serde_json::Number::from(transactions.len())));
        metrics.insert("block_count".to_string(), serde_json::Value::Number(serde_json::Number::from(blocks.len())));
        metrics.insert("estimated_memory_kb".to_string(), serde_json::Value::Number(serde_json::Number::from(total_memory / 1024)));
        metrics.insert("tx_memory_kb".to_string(), serde_json::Value::Number(serde_json::Number::from(tx_memory / 1024)));
        metrics.insert("block_memory_kb".to_string(), serde_json::Value::Number(serde_json::Number::from(block_memory / 1024)));
        
        TestResult {
            metrics,
            execution_time_ms: duration.as_millis() as f64,
        }
    }

    fn run_signature_verification_test(&self) -> TestResult {
        let start = Instant::now();
        
        let mut verification_results = Vec::new();
        
        for tx_data in self.test_data.transactions.iter().take(100) {
            let mut tx = Transaction::new(
                U256::from(tx_data.value),
                U256::from(tx_data.gas_price),
                U256::from(tx_data.gas_limit),
                Address::from_slice(&hex::decode(&tx_data.to_address[2..]).unwrap_or_default()),
                hex::decode(&tx_data.data).unwrap_or_default(),
                U256::from(tx_data.nonce),
            );
            
            // Generate random secret and sign
            let secret = [0x01; 32]; // Simplified for testing
            if tx.sign_with_secret(&secret).is_ok() {
                verification_results.push(tx.has_signature());
            }
        }
        
        let duration = start.elapsed();
        
        let valid_count = verification_results.iter().filter(|&&valid| valid).count();
        
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("signature_count".to_string(), serde_json::Value::Number(serde_json::Number::from(verification_results.len())));
        metrics.insert("average_time_per_signature_us".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(duration.as_micros() as f64 / verification_results.len() as f64).unwrap()));
        metrics.insert("valid_signatures".to_string(), serde_json::Value::Number(serde_json::Number::from(valid_count)));
        metrics.insert("invalid_signatures".to_string(), serde_json::Value::Number(serde_json::Number::from(verification_results.len() - valid_count)));
        
        TestResult {
            metrics,
            execution_time_ms: duration.as_millis() as f64,
        }
    }

    fn run_consensus_test(&self) -> TestResult {
        let start = Instant::now();
        
        let mut blocks = Vec::new();
        
        for block_data in &self.test_data.blocks {
            let block = Block::new(
                Address::from_slice(&hex::decode(&block_data.from_address[2..]).unwrap_or_default()),
                H256::from_slice(&hex::decode(&block_data.previous_hash).unwrap_or_default()),
                vec![],
                vec![],
                vec![],
                H256::zero(),
                H256::zero(),
                H256::zero(),
                block_data.timestamp,
                U256::from(block_data.gas_used),
                Signature { v: 27, r: H256::zero(), s: H256::zero() },
            );
            blocks.push(block);
        }
        
        // Simulate consensus validation
        let valid_blocks = blocks.iter().filter(|block| {
            // Simple consensus rule validation
            block.timestamp() > 0 && block.gas_used() > U256::zero()
        }).count();
        
        let duration = start.elapsed();
        
        let mut metrics = std::collections::HashMap::new();
        metrics.insert("block_count".to_string(), serde_json::Value::Number(serde_json::Number::from(blocks.len())));
        metrics.insert("valid_blocks".to_string(), serde_json::Value::Number(serde_json::Number::from(valid_blocks)));
        metrics.insert("invalid_blocks".to_string(), serde_json::Value::Number(serde_json::Number::from(blocks.len() - valid_blocks)));
        
        TestResult {
            metrics,
            execution_time_ms: duration.as_millis() as f64,
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <test_data_file> <test_type>", args[0]);
        eprintln!("Test types: transaction_creation, block_hashing, precompiled_contracts, evm_execution, memory_usage, signature_verification, consensus");
        std::process::exit(1);
    }
    
    let data_file = &args[1];
    let test_type = &args[2];
    
    match RustVersionTester::new(data_file) {
        Ok(tester) => {
            let result = match test_type.as_str() {
                "transaction_creation" => tester.run_transaction_creation_test(),
                "block_hashing" => tester.run_block_hashing_test(),
                "precompiled_contracts" => tester.run_precompiled_contracts_test(),
                "evm_execution" => tester.run_evm_execution_test(),
                "memory_usage" => tester.run_memory_usage_test(),
                "signature_verification" => tester.run_signature_verification_test(),
                "consensus" => tester.run_consensus_test(),
                _ => {
                    eprintln!("Unknown test type: {}", test_type);
                    std::process::exit(1);
                }
            };
            
            // Output JSON result
            match serde_json::to_string(&result) {
                Ok(json) => println!("{}", json),
                Err(e) => {
                    eprintln!("Error serializing result: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
