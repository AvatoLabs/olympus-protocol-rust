#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Olympus C/Rust Version Alignment Validation System
Comprehensive validation of functional and performance alignment between C and Rust implementations
"""

import json
import subprocess
import time
import os
import sys
import statistics
from typing import Dict, List, Any, Optional, Tuple
from dataclasses import dataclass, asdict
from pathlib import Path
import logging
import hashlib

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)

@dataclass
class AlignmentThresholds:
    """Alignment validation thresholds"""
    # Performance thresholds (percentage difference)
    max_performance_diff: float = 50.0  # Maximum acceptable performance difference
    warning_performance_diff: float = 20.0  # Warning threshold
    
    # Functional alignment thresholds
    min_functional_match_rate: float = 95.0  # Minimum functional match rate
    min_test_coverage: float = 90.0  # Minimum test coverage
    
    # Memory usage thresholds
    max_memory_diff: float = 30.0  # Maximum memory usage difference
    
    # Hash consistency thresholds
    min_hash_match_rate: float = 100.0  # Hash results must match exactly

@dataclass
class ValidationResult:
    """Validation result for a specific test"""
    test_name: str
    functional_alignment: bool
    performance_alignment: bool
    memory_alignment: bool
    hash_consistency: bool
    overall_score: float
    details: Dict[str, Any]
    recommendations: List[str]

@dataclass
class AlignmentReport:
    """Complete alignment validation report"""
    timestamp: str
    c_version_info: Dict[str, Any]
    rust_version_info: Dict[str, Any]
    validation_results: List[ValidationResult]
    overall_alignment_score: float
    critical_issues: List[str]
    recommendations: List[str]
    compliance_status: str

class VersionInfoExtractor:
    """Extract version information from executables"""
    
    @staticmethod
    def extract_c_version_info(executable_path: str) -> Dict[str, Any]:
        """Extract C version information"""
        try:
            # Try to get version info from executable
            result = subprocess.run([executable_path, "--version"], 
                                   capture_output=True, text=True, timeout=10)
            if result.returncode == 0:
                return {
                    "version": result.stdout.strip(),
                    "build_type": "Release",
                    "compiler": "GCC/Clang"
                }
        except:
            pass
        
        # Fallback to file info
        try:
            stat = os.stat(executable_path)
            return {
                "version": "Unknown",
                "build_type": "Release",
                "compiler": "GCC/Clang",
                "build_time": stat.st_mtime,
                "file_size": stat.st_size
            }
        except:
            return {"version": "Unknown", "build_type": "Unknown", "compiler": "Unknown"}
    
    @staticmethod
    def extract_rust_version_info(executable_path: str) -> Dict[str, Any]:
        """Extract Rust version information"""
        try:
            # Try to get version info from executable
            result = subprocess.run([executable_path, "--version"], 
                                   capture_output=True, text=True, timeout=10)
            if result.returncode == 0:
                return {
                    "version": result.stdout.strip(),
                    "build_type": "Release",
                    "compiler": "Rust"
                }
        except:
            pass
        
        # Fallback to file info
        try:
            stat = os.stat(executable_path)
            return {
                "version": "Unknown",
                "build_type": "Release",
                "compiler": "Rust",
                "build_time": stat.st_mtime,
                "file_size": stat.st_size
            }
        except:
            return {"version": "Unknown", "build_type": "Unknown", "compiler": "Unknown"}

class FunctionalAlignmentValidator:
    """Validate functional alignment between C and Rust versions"""
    
    def __init__(self, thresholds: AlignmentThresholds):
        self.thresholds = thresholds
    
    def validate_hash_consistency(self, c_results: Dict[str, Any], 
                                rust_results: Dict[str, Any]) -> Tuple[bool, Dict[str, Any]]:
        """Validate hash consistency between versions"""
        details = {}
        
        # Check if both versions have hash results
        c_hashes = c_results.get("sample_hashes", [])
        rust_hashes = rust_results.get("sample_hashes", [])
        
        if not c_hashes or not rust_hashes:
            return False, {"error": "Missing hash results"}
        
        # Compare hashes
        matches = 0
        total = min(len(c_hashes), len(rust_hashes))
        
        for i in range(total):
            if c_hashes[i] == rust_hashes[i]:
                matches += 1
        
        match_rate = (matches / total) * 100 if total > 0 else 0
        
        details = {
            "matches": matches,
            "total": total,
            "match_rate": match_rate,
            "c_hashes": c_hashes[:3],  # Sample
            "rust_hashes": rust_hashes[:3]  # Sample
        }
        
        is_aligned = match_rate >= self.thresholds.min_hash_match_rate
        return is_aligned, details
    
    def validate_transaction_results(self, c_results: Dict[str, Any], 
                                   rust_results: Dict[str, Any]) -> Tuple[bool, Dict[str, Any]]:
        """Validate transaction processing results"""
        details = {}
        
        # Check transaction counts
        c_count = c_results.get("transaction_count", 0)
        rust_count = rust_results.get("transaction_count", 0)
        
        if c_count != rust_count:
            return False, {"error": f"Transaction count mismatch: C={c_count}, Rust={rust_count}"}
        
        # Check other functional metrics
        c_success = c_results.get("success_count", 0)
        rust_success = rust_results.get("success_count", 0)
        
        success_rate_diff = abs(c_success - rust_success) / max(c_success, rust_success, 1) * 100
        
        details = {
            "transaction_count_match": c_count == rust_count,
            "success_count_c": c_success,
            "success_count_rust": rust_success,
            "success_rate_diff": success_rate_diff
        }
        
        is_aligned = success_rate_diff < 5.0  # Allow 5% difference
        return is_aligned, details
    
    def validate_block_results(self, c_results: Dict[str, Any], 
                             rust_results: Dict[str, Any]) -> Tuple[bool, Dict[str, Any]]:
        """Validate block processing results"""
        details = {}
        
        # Check block counts
        c_count = c_results.get("block_count", 0)
        rust_count = rust_results.get("block_count", 0)
        
        if c_count != rust_count:
            return False, {"error": f"Block count mismatch: C={c_count}, Rust={rust_count}"}
        
        # Check valid blocks
        c_valid = c_results.get("valid_blocks", 0)
        rust_valid = rust_results.get("valid_blocks", 0)
        
        valid_rate_diff = abs(c_valid - rust_valid) / max(c_valid, rust_valid, 1) * 100
        
        details = {
            "block_count_match": c_count == rust_count,
            "valid_blocks_c": c_valid,
            "valid_blocks_rust": rust_valid,
            "valid_rate_diff": valid_rate_diff
        }
        
        is_aligned = valid_rate_diff < 5.0  # Allow 5% difference
        return is_aligned, details

class PerformanceAlignmentValidator:
    """Validate performance alignment between C and Rust versions"""
    
    def __init__(self, thresholds: AlignmentThresholds):
        self.thresholds = thresholds
    
    def validate_execution_time(self, c_results: Dict[str, Any], 
                              rust_results: Dict[str, Any]) -> Tuple[bool, Dict[str, Any]]:
        """Validate execution time alignment"""
        c_time = c_results.get("execution_time_ms", 0)
        rust_time = rust_results.get("execution_time_ms", 0)
        
        if c_time == 0 or rust_time == 0:
            return False, {"error": "Invalid execution times"}
        
        # Calculate performance difference
        if c_time < rust_time:
            diff_percent = ((rust_time - c_time) / c_time) * 100
            faster_version = "C"
        else:
            diff_percent = ((c_time - rust_time) / rust_time) * 100
            faster_version = "Rust"
        
        details = {
            "c_time_ms": c_time,
            "rust_time_ms": rust_time,
            "performance_diff_percent": diff_percent,
            "faster_version": faster_version
        }
        
        is_aligned = diff_percent <= self.thresholds.max_performance_diff
        return is_aligned, details
    
    def validate_memory_usage(self, c_results: Dict[str, Any], 
                            rust_results: Dict[str, Any]) -> Tuple[bool, Dict[str, Any]]:
        """Validate memory usage alignment"""
        c_memory = c_results.get("estimated_memory_kb", 0)
        rust_memory = rust_results.get("estimated_memory_kb", 0)
        
        if c_memory == 0 or rust_memory == 0:
            return False, {"error": "Invalid memory usage data"}
        
        # Calculate memory difference
        memory_diff = abs(c_memory - rust_memory) / max(c_memory, rust_memory) * 100
        
        details = {
            "c_memory_kb": c_memory,
            "rust_memory_kb": rust_memory,
            "memory_diff_percent": memory_diff
        }
        
        is_aligned = memory_diff <= self.thresholds.max_memory_diff
        return is_aligned, details

class AlignmentValidator:
    """Main alignment validation system"""
    
    def __init__(self, thresholds: AlignmentThresholds = None):
        self.thresholds = thresholds or AlignmentThresholds()
        self.functional_validator = FunctionalAlignmentValidator(self.thresholds)
        self.performance_validator = PerformanceAlignmentValidator(self.thresholds)
    
    def validate_test_alignment(self, test_name: str, c_results: Dict[str, Any], 
                              rust_results: Dict[str, Any]) -> ValidationResult:
        """Validate alignment for a specific test"""
        recommendations = []
        details = {}
        
        # Functional alignment
        functional_aligned = True
        if "hash" in test_name.lower():
            functional_aligned, hash_details = self.functional_validator.validate_hash_consistency(
                c_results, rust_results)
            details.update(hash_details)
        elif "transaction" in test_name.lower():
            functional_aligned, tx_details = self.functional_validator.validate_transaction_results(
                c_results, rust_results)
            details.update(tx_details)
        elif "block" in test_name.lower():
            functional_aligned, block_details = self.functional_validator.validate_block_results(
                c_results, rust_results)
            details.update(block_details)
        
        # Performance alignment
        perf_aligned, perf_details = self.performance_validator.validate_execution_time(
            c_results, rust_results)
        details.update(perf_details)
        
        # Memory alignment
        memory_aligned, memory_details = self.performance_validator.validate_memory_usage(
            c_results, rust_results)
        details.update(memory_details)
        
        # Hash consistency
        hash_consistent = True
        if "hash" in test_name.lower():
            hash_consistent, _ = self.functional_validator.validate_hash_consistency(
                c_results, rust_results)
        
        # Calculate overall score
        score_components = [functional_aligned, perf_aligned, memory_aligned, hash_consistent]
        overall_score = (sum(score_components) / len(score_components)) * 100
        
        # Generate recommendations
        if not functional_aligned:
            recommendations.append("Fix functional misalignment - ensure both versions produce identical results")
        if not perf_aligned:
            recommendations.append(f"Optimize performance - current difference: {perf_details.get('performance_diff_percent', 0):.1f}%")
        if not memory_aligned:
            recommendations.append(f"Optimize memory usage - current difference: {memory_details.get('memory_diff_percent', 0):.1f}%")
        if not hash_consistent:
            recommendations.append("Fix hash consistency - ensure identical hash outputs")
        
        return ValidationResult(
            test_name=test_name,
            functional_alignment=functional_aligned,
            performance_alignment=perf_aligned,
            memory_alignment=memory_aligned,
            hash_consistency=hash_consistent,
            overall_score=overall_score,
            details=details,
            recommendations=recommendations
        )
    
    def generate_alignment_report(self, test_results: List[Dict[str, Any]], 
                                c_executable: str, rust_executable: str) -> AlignmentReport:
        """Generate comprehensive alignment report"""
        timestamp = time.strftime('%Y-%m-%d %H:%M:%S')
        
        # Extract version info
        c_version_info = VersionInfoExtractor.extract_c_version_info(c_executable)
        rust_version_info = VersionInfoExtractor.extract_rust_version_info(rust_executable)
        
        # Validate each test
        validation_results = []
        for result in test_results:
            validation_result = self.validate_test_alignment(
                result["test_name"], 
                result["c_version_result"], 
                result["rust_version_result"]
            )
            validation_results.append(validation_result)
        
        # Calculate overall alignment score
        overall_score = statistics.mean([vr.overall_score for vr in validation_results])
        
        # Identify critical issues
        critical_issues = []
        for vr in validation_results:
            if vr.overall_score < 80:
                critical_issues.append(f"{vr.test_name}: Score {vr.overall_score:.1f}%")
        
        # Generate recommendations
        all_recommendations = []
        for vr in validation_results:
            all_recommendations.extend(vr.recommendations)
        
        # Determine compliance status
        if overall_score >= 95:
            compliance_status = "EXCELLENT"
        elif overall_score >= 85:
            compliance_status = "GOOD"
        elif overall_score >= 70:
            compliance_status = "ACCEPTABLE"
        else:
            compliance_status = "NEEDS_IMPROVEMENT"
        
        return AlignmentReport(
            timestamp=timestamp,
            c_version_info=c_version_info,
            rust_version_info=rust_version_info,
            validation_results=validation_results,
            overall_alignment_score=overall_score,
            critical_issues=critical_issues,
            recommendations=list(set(all_recommendations)),  # Remove duplicates
            compliance_status=compliance_status
        )
    
    def save_report(self, report: AlignmentReport, filename: str = None) -> str:
        """Save alignment report to file"""
        if filename is None:
            filename = f"alignment_report_{int(time.time())}.json"
        
        # Convert to serializable format
        report_dict = asdict(report)
        
        with open(filename, 'w', encoding='utf-8') as f:
            json.dump(report_dict, f, indent=2, ensure_ascii=False)
        
        return filename
    
    def print_report(self, report: AlignmentReport):
        """Print alignment report to console"""
        print("=" * 80)
        print("OLYMPUS C/RUST VERSION ALIGNMENT VALIDATION REPORT")
        print("=" * 80)
        print(f"Timestamp: {report.timestamp}")
        print(f"Overall Alignment Score: {report.overall_alignment_score:.1f}%")
        print(f"Compliance Status: {report.compliance_status}")
        print()
        
        print("VERSION INFORMATION:")
        print(f"  C Version: {report.c_version_info.get('version', 'Unknown')}")
        print(f"  Rust Version: {report.rust_version_info.get('version', 'Unknown')}")
        print()
        
        print("VALIDATION RESULTS:")
        print("-" * 80)
        for vr in report.validation_results:
            print(f"Test: {vr.test_name}")
            print(f"  Overall Score: {vr.overall_score:.1f}%")
            print(f"  Functional: {'✅' if vr.functional_alignment else '❌'}")
            print(f"  Performance: {'✅' if vr.performance_alignment else '❌'}")
            print(f"  Memory: {'✅' if vr.memory_alignment else '❌'}")
            print(f"  Hash Consistency: {'✅' if vr.hash_consistency else '❌'}")
            if vr.recommendations:
                print(f"  Recommendations: {', '.join(vr.recommendations)}")
            print()
        
        if report.critical_issues:
            print("CRITICAL ISSUES:")
            for issue in report.critical_issues:
                print(f"  ❌ {issue}")
            print()
        
        if report.recommendations:
            print("RECOMMENDATIONS:")
            for i, rec in enumerate(report.recommendations, 1):
                print(f"  {i}. {rec}")
            print()
        
        print("=" * 80)

def main():
    """Main function for alignment validation"""
    # This would typically be called from the comparison test framework
    # For now, we'll create a sample validation
    thresholds = AlignmentThresholds()
    validator = AlignmentValidator(thresholds)
    
    # Sample test results (would come from actual test execution)
    sample_results = [
        {
            "test_name": "transaction_creation",
            "c_version_result": {
                "transaction_count": 1000,
                "execution_time_ms": 150.5,
                "estimated_memory_kb": 2048
            },
            "rust_version_result": {
                "transaction_count": 1000,
                "execution_time_ms": 145.2,
                "estimated_memory_kb": 2156
            }
        }
    ]
    
    # Generate report
    report = validator.generate_alignment_report(
        sample_results, 
        "./olympus-cpp/build/mcp", 
        "./olympus-rust/target/release/rust_version_test"
    )
    
    # Print and save report
    validator.print_report(report)
    filename = validator.save_report(report)
    print(f"\nReport saved to: {filename}")

if __name__ == "__main__":
    main()
