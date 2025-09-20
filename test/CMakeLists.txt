# CMakeLists.txt for C version test compilation
cmake_minimum_required(VERSION 3.10)
project(OlympusCVersionTest)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Find required packages
find_package(PkgConfig REQUIRED)
pkg_check_modules(JSONCPP REQUIRED jsoncpp)

# Include directories
include_directories(${JSONCPP_INCLUDE_DIRS})
include_directories(${CMAKE_SOURCE_DIR}/mcp)
include_directories(${CMAKE_SOURCE_DIR}/libdevcore)
include_directories(${CMAKE_SOURCE_DIR}/libdevcrypto)
include_directories(${CMAKE_SOURCE_DIR}/libevm)
include_directories(${CMAKE_SOURCE_DIR}/libinterpreter)

# Source files for C version test
set(C_VERSION_TEST_SOURCES
    test/c_version_test.cpp
    mcp/core/transaction.cpp
    mcp/core/block.cpp
    mcp/evm/executive.cpp
    mcp/common/address.cpp
    mcp/common/hash.cpp
    mcp/crypto/sha3.cpp
    libdevcore/Address.cpp
    libdevcore/Common.cpp
    libdevcore/RLP.cpp
    libdevcrypto/AES.cpp
    libdevcrypto/SHA3.cpp
    libevm/EVMC.cpp
    libinterpreter/VM.cpp
)

# Create executable
add_executable(c_version_test ${C_VERSION_TEST_SOURCES})

# Link libraries
target_link_libraries(c_version_test ${JSONCPP_LIBRARIES})

# Compiler flags
target_compile_options(c_version_test PRIVATE ${JSONCPP_CFLAGS_OTHER})

# Installation
install(TARGETS c_version_test DESTINATION bin)

# Test configuration
enable_testing()
add_test(NAME c_version_test COMMAND c_version_test)

# Print configuration info
message(STATUS "C Version Test Configuration:")
message(STATUS "  JSONCPP_INCLUDE_DIRS: ${JSONCPP_INCLUDE_DIRS}")
message(STATUS "  JSONCPP_LIBRARIES: ${JSONCPP_LIBRARIES}")
message(STATUS "  CMAKE_CXX_COMPILER: ${CMAKE_CXX_COMPILER}")
message(STATUS "  CMAKE_BUILD_TYPE: ${CMAKE_BUILD_TYPE}")
