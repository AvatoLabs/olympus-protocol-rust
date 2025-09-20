#!/bin/bash
# Build and run comparison tests for C and Rust versions

set -e

echo "🔧 Building Olympus C and Rust Version Comparison Tests"
echo "====================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "CMakeLists.txt" ]; then
    print_error "Please run this script from the Olympus root directory"
    exit 1
fi

# Build Olympus C++ implementation
print_status "Building Olympus C++ implementation..."
cd olympus-cpp

# Create build directory
mkdir -p build
cd build

# Configure and build
if cmake .. -DCMAKE_BUILD_TYPE=RelWithDebInfo; then
    if make -j$(nproc); then
        print_success "Olympus C++ implementation built successfully"
    else
        print_error "Failed to build Olympus C++ implementation"
        exit 1
    fi
else
    print_error "Failed to configure Olympus C++ implementation"
    exit 1
fi

cd ../..

# Build Rust version test
print_status "Building Rust version test..."
cd olympus-rust

# Add required dependencies to Cargo.toml if not present
if ! grep -q "serde_json" Cargo.toml; then
    print_status "Adding serde_json dependency..."
    echo 'serde_json = "1.0"' >> Cargo.toml
fi

if ! grep -q "hex" Cargo.toml; then
    print_status "Adding hex dependency..."
    echo 'hex = "0.4"' >> Cargo.toml
fi

if ! grep -q "rand" Cargo.toml; then
    print_status "Adding rand dependency..."
    echo 'rand = "0.8"' >> Cargo.toml
fi

# Build Rust version
if cargo build --release --bin rust_version_test; then
    print_success "Rust version test built successfully"
else
    print_error "Failed to build Rust version test"
    exit 1
fi

cd ..

# Install Python dependencies
print_status "Installing Python dependencies..."
if command -v pip3 &> /dev/null; then
    pip3 install -r test/requirements.txt 2>/dev/null || {
        print_warning "Failed to install from requirements.txt, installing manually..."
        pip3 install requests json
    }
else
    print_warning "pip3 not found, please install Python dependencies manually"
fi

# Run comparison tests
print_status "Running comparison tests..."
if python3 test/comparison_test_framework.py; then
    print_success "Comparison tests completed successfully"
else
    print_error "Comparison tests failed"
    exit 1
fi

# Run dynamic benchmark
print_status "Running dynamic benchmark..."
cd olympus-rust
if cargo run --release --example dynamic_benchmark; then
    print_success "Dynamic benchmark completed successfully"
else
    print_error "Dynamic benchmark failed"
    exit 1
fi

cd ..

print_success "All tests completed successfully!"
print_status "Check the generated report files for detailed results"

# Display summary
echo ""
echo "📊 Test Summary:"
echo "  ✅ Olympus C++ implementation: Built and ready"
echo "  ✅ Rust version test: Built and ready"
echo "  ✅ Comparison framework: Executed"
echo "  ✅ Dynamic benchmark: Completed"
echo ""
echo "🔍 Next Steps:"
echo "  1. Review comparison test reports"
echo "  2. Analyze performance differences"
echo "  3. Fix any functional misalignments"
echo "  4. Optimize performance bottlenecks"
echo ""
echo "📁 Generated Files:"
echo "  - olympus-cpp/build/mcp (Olympus C++ executable)"
echo "  - olympus-rust/target/release/rust_version_test (Rust test executable)"
echo "  - comparison_test_report_*.txt (Test reports)"
echo ""
echo "🎯 The dynamic test framework ensures:"
echo "  • No hardcoded test values"
echo "  • Realistic test data patterns"
echo "  • Fair C/Rust version comparison"
echo "  • Reproducible results"
echo "  • Comprehensive coverage"
