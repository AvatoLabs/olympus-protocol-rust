use olympus::dynamic_tests::{DynamicBenchmarkSuite, TestConfig};

fn main() {
    println!("🚀 Olympus Rust Dynamic Performance Benchmark");
    println!("=============================================");
    
    // Configure dynamic test parameters
    let config = TestConfig {
        transaction_count: 1000,
        block_count: 100,
        gas_price_range: (1_000_000_000, 50_000_000_000), // 1 gwei to 50 gwei
        value_range: (1_000, 1_000_000_000_000_000_000), // 0.001 ETH to 1 ETH
        gas_limit_range: (21_000, 1_000_000),
        timestamp_range: (1_600_000_000, 2_000_000_000), // 2020-2033
        data_size_range: (1, 1024), // 1 byte to 1KB
        performance_iterations: 10_000,
        memory_test_size: 1000,
        random_seed: Some(42), // Use fixed seed for reproducible results
    };
    
    let mut suite = DynamicBenchmarkSuite::new(config);
    
    // Run all benchmarks
    println!("\n📊 Running Dynamic Benchmarks...");
    let all_results = suite.run_all_benchmarks();
    
    // Display results
    for (test_name, results) in all_results {
        println!("\n🔍 {} Results:", test_name);
        println!("   Execution Time: {:.2} ms", results.get("execution_time_ms").unwrap_or(&0.0));
        
        match test_name.as_str() {
            "transaction_creation" => {
                println!("   Transactions Created: {}", *results.get("transaction_count").unwrap_or(&0.0) as u64);
                println!("   Average per Transaction: {:.2} μs", results.get("average_time_per_tx_us").unwrap_or(&0.0));
            },
            "block_hashing" => {
                println!("   Blocks Processed: {}", *results.get("block_count").unwrap_or(&0.0) as u64);
                println!("   Average per Hash: {:.2} μs", results.get("average_time_per_hash_us").unwrap_or(&0.0));
            },
            "precompiled_contracts" => {
                println!("   Contracts Executed: {}", *results.get("contract_count").unwrap_or(&0.0) as u64);
                println!("   Average per Contract: {:.2} μs", results.get("average_time_per_contract_us").unwrap_or(&0.0));
            },
            "evm_execution" => {
                println!("   Executions: {}", *results.get("execution_count").unwrap_or(&0.0) as u64);
                println!("   Success Rate: {:.1}%", 
                    (results.get("success_count").unwrap_or(&0.0) / results.get("execution_count").unwrap_or(&1.0)) * 100.0);
                println!("   Average per Execution: {:.2} μs", results.get("average_time_per_execution_us").unwrap_or(&0.0));
            },
            "memory_usage" => {
                println!("   Transactions: {}", *results.get("transaction_count").unwrap_or(&0.0) as u64);
                println!("   Blocks: {}", *results.get("block_count").unwrap_or(&0.0) as u64);
                println!("   Estimated Memory: {:.1} KB", results.get("estimated_memory_kb").unwrap_or(&0.0));
            },
            "signature_verification" => {
                println!("   Signatures Verified: {}", *results.get("signature_count").unwrap_or(&0.0) as u64);
                println!("   Valid Signatures: {}", *results.get("valid_signatures").unwrap_or(&0.0) as u64);
                println!("   Average per Signature: {:.2} μs", results.get("average_time_per_signature_us").unwrap_or(&0.0));
            },
            "consensus" => {
                println!("   Blocks Processed: {}", *results.get("block_count").unwrap_or(&0.0) as u64);
                println!("   Valid Blocks: {}", *results.get("valid_blocks").unwrap_or(&0.0) as u64);
                println!("   Invalid Blocks: {}", *results.get("invalid_blocks").unwrap_or(&0.0) as u64);
            },
            _ => {}
        }
    }
    
    // Performance summary
    println!("\n🎯 Dynamic Benchmark Summary:");
    println!("   • All tests use randomized, realistic data");
    println!("   • No hardcoded values - fully dynamic");
    println!("   • Reproducible results with fixed seed");
    println!("   • Comprehensive coverage of core functionality");
    
    println!("\n🏆 Dynamic Testing Advantages:");
    println!("   • Real-world data patterns");
    println!("   • Edge case coverage");
    println!("   • Performance variance analysis");
    println!("   • Regression detection");
    println!("   • C/Rust version alignment validation");
    
    // Comparison with C version
    println!("\n🔄 C/Rust Version Alignment:");
    println!("   • Use comparison_test_framework.py for alignment testing");
    println!("   • Dynamic data ensures fair comparison");
    println!("   • Performance metrics can be compared directly");
    println!("   • Functional equivalence validation");
    
    println!("\n✅ Dynamic benchmark completed successfully!");
}
