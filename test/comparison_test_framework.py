#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Olympus C and Rust Version Comparison Test Framework
Used to ensure functional alignment and performance consistency between two versions
"""

import json
import subprocess
import time
import random
import hashlib
import os
import sys
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass, asdict
from pathlib import Path
import logging

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

@dataclass
class TestConfig:
    """Test configuration class"""
    # Test parameters
    transaction_count: int = 1000
    block_count: int = 100
    gas_price_range: Tuple[int, int] = (1000000000, 50000000000)  # 1 gwei to 50 gwei
    value_range: Tuple[int, int] = (1000, 1000000000000000000)  # 0.001 ETH to 1 ETH
    gas_limit_range: Tuple[int, int] = (21000, 1000000)
    
    # Address generation
    address_prefix: str = "0x"
    address_length: int = 40
    
    # Timestamp range
    timestamp_range: Tuple[int, int] = (1600000000, 2000000000)  # 2020-2033
    
    # Data size range
    data_size_range: Tuple[int, int] = (1, 1024)  # 1 byte to 1KB
    
    # Performance test parameters
    performance_iterations: int = 10000
    memory_test_size: int = 1000
    
    # Random seed (for reproducible tests)
    random_seed: Optional[int] = None

@dataclass
class TestData:
    """Test data class"""
    transactions: List[Dict[str, Any]]
    blocks: List[Dict[str, Any]]
    addresses: List[str]
    timestamps: List[int]
    gas_prices: List[int]
    values: List[int]
    gas_limits: List[int]
    data_payloads: List[bytes]

@dataclass
class TestResult:
    """Test result class"""
    test_name: str
    c_version_result: Dict[str, Any]
    rust_version_result: Dict[str, Any]
    performance_diff: float  # Performance difference percentage
    functional_match: bool  # Whether functionality matches
    error_message: Optional[str] = None

class TestDataGenerator:
    """Test data generator"""
    
    def __init__(self, config: TestConfig):
        self.config = config
        if config.random_seed:
            random.seed(config.random_seed)
    
    def generate_addresses(self, count: int) -> List[str]:
        """Generate random addresses"""
        addresses = []
        for _ in range(count):
            # Generate 40-character hex string
            hex_str = ''.join(random.choices('0123456789abcdef', k=self.config.address_length))
            addresses.append(f"{self.config.address_prefix}{hex_str}")
        return addresses
    
    def generate_timestamps(self, count: int) -> List[int]:
        """Generate random timestamps"""
        return [random.randint(*self.config.timestamp_range) for _ in range(count)]
    
    def generate_gas_prices(self, count: int) -> List[int]:
        """Generate random gas prices"""
        return [random.randint(*self.config.gas_price_range) for _ in range(count)]
    
    def generate_values(self, count: int) -> List[int]:
        """Generate random transaction values"""
        return [random.randint(*self.config.value_range) for _ in range(count)]
    
    def generate_gas_limits(self, count: int) -> List[int]:
        """Generate random gas limits"""
        return [random.randint(*self.config.gas_limit_range) for _ in range(count)]
    
    def generate_data_payloads(self, count: int) -> List[bytes]:
        """Generate random data payloads"""
        payloads = []
        for _ in range(count):
            size = random.randint(*self.config.data_size_range)
            payloads.append(os.urandom(size))
        return payloads
    
    def generate_test_data(self) -> TestData:
        """Generate complete test data"""
        logger.info("Generating test data...")
        
        # Generate basic data
        addresses = self.generate_addresses(self.config.transaction_count + self.config.block_count)
        timestamps = self.generate_timestamps(self.config.block_count)
        gas_prices = self.generate_gas_prices(self.config.transaction_count)
        values = self.generate_values(self.config.transaction_count)
        gas_limits = self.generate_gas_limits(self.config.transaction_count)
        data_payloads = self.generate_data_payloads(self.config.transaction_count)
        
        # Generate transaction data
        transactions = []
        for i in range(self.config.transaction_count):
            transaction = {
                "nonce": i,
                "value": values[i],
                "gas_price": gas_prices[i],
                "gas_limit": gas_limits[i],
                "to_address": addresses[i],
                "data": data_payloads[i].hex(),
                "timestamp": timestamps[i % len(timestamps)]
            }
            transactions.append(transaction)
        
        # Generate block data
        blocks = []
        for i in range(self.config.block_count):
            block = {
                "from_address": addresses[self.config.transaction_count + i],
                "previous_hash": hashlib.sha256(f"block_{i}".encode()).hexdigest(),
                "timestamp": timestamps[i],
                "transaction_count": random.randint(1, 100),
                "gas_used": random.randint(1000000, 10000000)
            }
            blocks.append(block)
        
        return TestData(
            transactions=transactions,
            blocks=blocks,
            addresses=addresses,
            timestamps=timestamps,
            gas_prices=gas_prices,
            values=values,
            gas_limits=gas_limits,
            data_payloads=data_payloads
        )

class VersionTester:
    """Version tester base class"""
    
    def __init__(self, executable_path: str):
        self.executable_path = executable_path
    
    def run_test(self, test_data: TestData, test_type: str) -> Dict[str, Any]:
        """Run test and return results"""
        raise NotImplementedError

class CVersionTester(VersionTester):
    """C version tester"""
    
    def run_test(self, test_data: TestData, test_type: str) -> Dict[str, Any]:
        """Run C version test"""
        logger.info(f"Running C version {test_type} test...")
        
        # Write test data to temporary file
        test_file = f"/tmp/c_test_data_{test_type}_{int(time.time())}.json"
        with open(test_file, 'w') as f:
            json.dump(asdict(test_data), f, indent=2)
        
        try:
            # Run C version test program
            cmd = [self.executable_path, test_file, test_type]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
            
            if result.returncode != 0:
                logger.error(f"C version test failed: {result.stderr}")
                return {"error": result.stderr}
            
            # Parse results
            return json.loads(result.stdout)
        
        except subprocess.TimeoutExpired:
            logger.error("C version test timeout")
            return {"error": "Test timeout"}
        except Exception as e:
            logger.error(f"C version test exception: {e}")
            return {"error": str(e)}
        finally:
            # Clean up temporary file
            if os.path.exists(test_file):
                os.remove(test_file)

class RustVersionTester(VersionTester):
    """Rust version tester"""
    
    def run_test(self, test_data: TestData, test_type: str) -> Dict[str, Any]:
        """Run Rust version test"""
        logger.info(f"Running Rust version {test_type} test...")
        
        # Write test data to temporary file
        test_file = f"/tmp/rust_test_data_{test_type}_{int(time.time())}.json"
        with open(test_file, 'w') as f:
            json.dump(asdict(test_data), f, indent=2)
        
        try:
            # Run Rust version test program
            cmd = [self.executable_path, test_file, test_type]
            result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
            
            if result.returncode != 0:
                logger.error(f"Rust version test failed: {result.stderr}")
                return {"error": result.stderr}
            
            # Parse results
            return json.loads(result.stdout)
        
        except subprocess.TimeoutExpired:
            logger.error("Rust version test timeout")
            return {"error": "Test timeout"}
        except Exception as e:
            logger.error(f"Rust version test exception: {e}")
            return {"error": str(e)}
        finally:
            # Clean up temporary file
            if os.path.exists(test_file):
                os.remove(test_file)

class ComparisonTestFramework:
    """Comparison test framework main class"""
    
    def __init__(self, c_executable: str, rust_executable: str, config: TestConfig):
        self.c_tester = CVersionTester(c_executable)
        self.rust_tester = RustVersionTester(rust_executable)
        self.config = config
        self.data_generator = TestDataGenerator(config)
    
    def run_comparison_test(self, test_name: str, test_type: str) -> TestResult:
        """Run comparison test"""
        logger.info(f"Starting {test_name} comparison test...")
        
        # Generate test data
        test_data = self.data_generator.generate_test_data()
        
        # Run C version test
        c_result = self.c_tester.run_test(test_data, test_type)
        
        # Run Rust version test
        rust_result = self.rust_tester.run_test(test_data, test_type)
        
        # Compare results
        functional_match = self._compare_functional_results(c_result, rust_result)
        performance_diff = self._compare_performance_results(c_result, rust_result)
        
        return TestResult(
            test_name=test_name,
            c_version_result=c_result,
            rust_version_result=rust_result,
            performance_diff=performance_diff,
            functional_match=functional_match
        )
    
    def _compare_functional_results(self, c_result: Dict[str, Any], rust_result: Dict[str, Any]) -> bool:
        """Compare functional results"""
        # Check for errors
        if "error" in c_result or "error" in rust_result:
            return False
        
        # Compare key fields
        key_fields = ["transaction_count", "block_count", "hash_results", "signature_results"]
        
        for field in key_fields:
            if field in c_result and field in rust_result:
                if c_result[field] != rust_result[field]:
                    logger.warning(f"Functional results mismatch - {field}: C={c_result[field]}, Rust={rust_result[field]}")
                    return False
        
        return True
    
    def _compare_performance_results(self, c_result: Dict[str, Any], rust_result: Dict[str, Any]) -> float:
        """Compare performance results"""
        if "error" in c_result or "error" in rust_result:
            return float('inf')
        
        c_time = c_result.get("execution_time_ms", 0)
        rust_time = rust_result.get("execution_time_ms", 0)
        
        if c_time == 0:
            return float('inf')
        
        return ((rust_time - c_time) / c_time) * 100
    
    def run_all_tests(self) -> List[TestResult]:
        """Run all tests"""
        test_suite = [
            ("Transaction Creation Performance Test", "transaction_creation"),
            ("Block Hashing Test", "block_hashing"),
            ("Precompiled Contract Test", "precompiled_contracts"),
            ("EVM Execution Test", "evm_execution"),
            ("Memory Usage Test", "memory_usage"),
            ("Signature Verification Test", "signature_verification"),
            ("Consensus Algorithm Test", "consensus"),
        ]
        
        results = []
        for test_name, test_type in test_suite:
            try:
                result = self.run_comparison_test(test_name, test_type)
                results.append(result)
            except Exception as e:
                logger.error(f"Test {test_name} failed: {e}")
                results.append(TestResult(
                    test_name=test_name,
                    c_version_result={},
                    rust_version_result={},
                    performance_diff=float('inf'),
                    functional_match=False,
                    error_message=str(e)
                ))
        
        return results
    
    def generate_report(self, results: List[TestResult]) -> str:
        """Generate test report"""
        report = []
        report.append("=" * 80)
        report.append("Olympus C and Rust Version Comparison Test Report")
        report.append("=" * 80)
        report.append(f"Test time: {time.strftime('%Y-%m-%d %H:%M:%S')}")
        report.append(f"Test configuration: {asdict(self.config)}")
        report.append("")
        
        # Statistics
        total_tests = len(results)
        passed_tests = sum(1 for r in results if r.functional_match)
        failed_tests = total_tests - passed_tests
        
        report.append("Test Statistics:")
        report.append(f"  Total tests: {total_tests}")
        report.append(f"  Passed tests: {passed_tests}")
        report.append(f"  Failed tests: {failed_tests}")
        report.append(f"  Pass rate: {(passed_tests/total_tests)*100:.1f}%")
        report.append("")
        
        # Detailed results
        report.append("Detailed Test Results:")
        report.append("-" * 80)
        
        for result in results:
            report.append(f"Test name: {result.test_name}")
            report.append(f"  Functional match: {'✅ Pass' if result.functional_match else '❌ Fail'}")
            
            if result.performance_diff != float('inf'):
                if abs(result.performance_diff) < 10:
                    report.append(f"  Performance difference: ✅ {result.performance_diff:+.1f}% (acceptable)")
                elif abs(result.performance_diff) < 50:
                    report.append(f"  Performance difference: ⚠️  {result.performance_diff:+.1f}% (needs attention)")
                else:
                    report.append(f"  Performance difference: ❌ {result.performance_diff:+.1f}% (needs optimization)")
            else:
                report.append("  Performance difference: ❌ Cannot compare")
            
            if result.error_message:
                report.append(f"  Error message: {result.error_message}")
            
            report.append("")
        
        # Recommendations
        report.append("Recommendations:")
        if failed_tests > 0:
            report.append("  - Fix functionally mismatched tests")
        if any(abs(r.performance_diff) > 50 for r in results if r.performance_diff != float('inf')):
            report.append("  - Optimize modules with large performance differences")
        if passed_tests == total_tests:
            report.append("  - All tests passed, versions are well aligned")
        
        return "\n".join(report)

def main():
    """Main function"""
    # Configuration
    config = TestConfig(
        transaction_count=1000,
        block_count=100,
        performance_iterations=10000,
        random_seed=42  # Use fixed seed to ensure reproducibility
    )
    
    # Executable file paths (need to adjust based on actual compilation results)
    c_executable = "./olympus-cpp/build/mcp"
    rust_executable = "./olympus-rust/target/release/rust_version_test"
    
    # Check if executable files exist
    if not os.path.exists(c_executable):
        logger.error(f"C version test program does not exist: {c_executable}")
        sys.exit(1)
    
    if not os.path.exists(rust_executable):
        logger.error(f"Rust version test program does not exist: {rust_executable}")
        sys.exit(1)
    
    # Create test framework
    framework = ComparisonTestFramework(c_executable, rust_executable, config)
    
    # Run all tests
    logger.info("Starting comparison tests...")
    results = framework.run_all_tests()
    
    # Generate report
    report = framework.generate_report(results)
    
    # Output report
    print(report)
    
    # Save report to file
    report_file = f"comparison_test_report_{int(time.time())}.txt"
    with open(report_file, 'w', encoding='utf-8') as f:
        f.write(report)
    
    logger.info(f"Test report saved to: {report_file}")
    
    # Return exit code
    failed_tests = sum(1 for r in results if not r.functional_match)
    sys.exit(0 if failed_tests == 0 else 1)

if __name__ == "__main__":
    main()
