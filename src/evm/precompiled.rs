//! Precompiled contracts implementation

use crate::{Address, U256, Result, OlympusError};
use sha3::Digest;
use std::collections::HashMap;
use num_bigint::{BigUint};
use num_traits::{Zero, Num};

/// Trait for precompiled contracts
pub trait PrecompiledContract {
    /// Execute the precompiled contract
    fn execute(&self, input: &[u8]) -> Result<Vec<u8>>;
    /// Get gas cost for the input
    fn gas_cost(&self, input: &[u8]) -> U256;
}

/// ECRECOVER precompiled contract (address 0x01)
pub struct EcrecoverContract;

impl PrecompiledContract for EcrecoverContract {
    fn execute(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.len() < 128 {
            return Ok(vec![0u8; 32]);
        }
        
        // Extract hash, v, r, s from input
        let _hash = &input[0..32];
        let _v = input[63];
        let _r = &input[64..96];
        let _s = &input[96..128];
        
        // For now, return zero address (placeholder implementation)
        Ok(vec![0u8; 32])
    }

    fn gas_cost(&self, _input: &[u8]) -> U256 {
        U256::from(3000)
    }
}

/// SHA256 precompiled contract (address 0x02)
pub struct Sha256Contract;

impl PrecompiledContract for Sha256Contract {
    fn execute(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut hasher = sha3::Sha3_256::new();
        hasher.update(input);
        let result = hasher.finalize();
        Ok(result.to_vec())
    }

    fn gas_cost(&self, input: &[u8]) -> U256 {
        U256::from(60 + (input.len() / 32) * 12)
    }
}

/// RIPEMD160 precompiled contract (address 0x03)
pub struct Ripemd160Contract;

impl PrecompiledContract for Ripemd160Contract {
    fn execute(&self, input: &[u8]) -> Result<Vec<u8>> {
        use ripemd::Ripemd160;
        let mut hasher = Ripemd160::new();
        hasher.update(input);
        let result = hasher.finalize();
        
        // Pad to 32 bytes
        let mut output = vec![0u8; 32];
        output[12..].copy_from_slice(&result);
        Ok(output)
    }

    fn gas_cost(&self, input: &[u8]) -> U256 {
        U256::from(600 + (input.len() / 32) * 120)
    }
}

/// IDENTITY precompiled contract (address 0x04)
pub struct IdentityContract;

impl PrecompiledContract for IdentityContract {
    fn execute(&self, input: &[u8]) -> Result<Vec<u8>> {
        Ok(input.to_vec())
    }

    fn gas_cost(&self, input: &[u8]) -> U256 {
        U256::from(15 + (input.len() / 32) * 3)
    }
}

/// MODEXP precompiled contract (address 0x05)
pub struct ModExpContract;

impl PrecompiledContract for ModExpContract {
    fn execute(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.len() < 96 {
            return Ok(vec![0u8; 32]);
        }
        
        // Extract base, exponent, modulus lengths
        let base_len = u32::from_be_bytes([input[0], input[1], input[2], input[3]]) as usize;
        let exp_len = u32::from_be_bytes([input[4], input[5], input[6], input[7]]) as usize;
        let mod_len = u32::from_be_bytes([input[8], input[9], input[10], input[11]]) as usize;
        
        if input.len() < 32 + base_len + exp_len + mod_len {
            return Ok(vec![0u8; 32]);
        }
        
        // Extract the actual values
        let base_start = 32;
        let exp_start = base_start + base_len;
        let mod_start = exp_start + exp_len;
        
        let base_bytes = &input[base_start..exp_start];
        let exp_bytes = &input[exp_start..mod_start];
        let mod_bytes = &input[mod_start..mod_start + mod_len];
        
        // Convert to BigUint
        let base = BigUint::from_bytes_be(base_bytes);
        let exponent = BigUint::from_bytes_be(exp_bytes);
        let modulus = BigUint::from_bytes_be(mod_bytes);
        
        // Handle special cases
        if modulus.is_zero() {
            return Ok(vec![0u8; mod_len]);
        }
        
        // Perform modular exponentiation: base^exponent mod modulus
        let result = base.modpow(&exponent, &modulus);
        
        // Convert result back to bytes
        let result_bytes = result.to_bytes_be();
        let mut output = vec![0u8; mod_len];
        let start_pos = mod_len.saturating_sub(result_bytes.len());
        output[start_pos..].copy_from_slice(&result_bytes);
        
        Ok(output)
    }

    fn gas_cost(&self, input: &[u8]) -> U256 {
        if input.len() < 96 {
            return U256::from(200);
        }
        
        let base_len = u32::from_be_bytes([input[0], input[1], input[2], input[3]]) as usize;
        let exp_len = u32::from_be_bytes([input[4], input[5], input[6], input[7]]) as usize;
        let mod_len = u32::from_be_bytes([input[8], input[9], input[10], input[11]]) as usize;
        
        // Gas cost calculation based on Ethereum specification
        let gas_cost = (base_len + mod_len) * 50 + exp_len * 10;
        U256::from(200 + gas_cost)
    }
}

/// ECADD precompiled contract (address 0x06)
pub struct EcAddContract;

impl PrecompiledContract for EcAddContract {
    fn execute(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.len() < 128 {
            return Ok(vec![0u8; 64]);
        }
        
        // Extract two points (x1, y1) and (x2, y2)
        let x1_bytes = &input[0..32];
        let y1_bytes = &input[32..64];
        let x2_bytes = &input[64..96];
        let y2_bytes = &input[96..128];
        
        // Convert to BigUint
        let x1 = BigUint::from_bytes_be(x1_bytes);
        let y1 = BigUint::from_bytes_be(y1_bytes);
        let x2 = BigUint::from_bytes_be(x2_bytes);
        let y2 = BigUint::from_bytes_be(y2_bytes);
        
        // BN254 curve parameters
        let p = BigUint::from_str_radix("21888242871839275222246405745257275088696311157297823662689037894645226208583", 10)
            .map_err(|_| OlympusError::EvmExecution("Invalid curve parameter".to_string()))?;
        
        // Check if points are on curve
        if !is_point_on_curve(&x1, &y1, &p) || !is_point_on_curve(&x2, &y2, &p) {
            return Ok(vec![0u8; 64]);
        }
        
        // Perform elliptic curve point addition
        match ec_add(&x1, &y1, &x2, &y2, &p) {
            Some((x3, y3)) => {
                let mut result = vec![0u8; 64];
                let x3_bytes = x3.to_bytes_be();
                let y3_bytes = y3.to_bytes_be();
                
                let x_start = 32 - x3_bytes.len().min(32);
                let y_start = 64 - y3_bytes.len().min(32);
                
                result[x_start..x_start + x3_bytes.len().min(32)].copy_from_slice(&x3_bytes[..x3_bytes.len().min(32)]);
                result[y_start..y_start + y3_bytes.len().min(32)].copy_from_slice(&y3_bytes[..y3_bytes.len().min(32)]);
                
                Ok(result)
            }
            None => Ok(vec![0u8; 64]),
        }
    }

    fn gas_cost(&self, _input: &[u8]) -> U256 {
        U256::from(150)
    }
}

/// ECMUL precompiled contract (address 0x07)
pub struct EcMulContract;

impl PrecompiledContract for EcMulContract {
    fn execute(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.len() < 96 {
            return Ok(vec![0u8; 64]);
        }
        
        // Extract point (x, y) and scalar k
        let x_bytes = &input[0..32];
        let y_bytes = &input[32..64];
        let k_bytes = &input[64..96];
        
        let x = BigUint::from_bytes_be(x_bytes);
        let y = BigUint::from_bytes_be(y_bytes);
        let k = BigUint::from_bytes_be(k_bytes);
        
        // BN254 curve parameters
        let p = BigUint::from_str_radix("21888242871839275222246405745257275088696311157297823662689037894645226208583", 10)
            .map_err(|_| OlympusError::EvmExecution("Invalid curve parameter".to_string()))?;
        
        // Check if point is on curve
        if !is_point_on_curve(&x, &y, &p) {
            return Ok(vec![0u8; 64]);
        }
        
        // Perform scalar multiplication
        match ec_mul(&x, &y, &k, &p) {
            Some((x3, y3)) => {
                let mut result = vec![0u8; 64];
                let x3_bytes = x3.to_bytes_be();
                let y3_bytes = y3.to_bytes_be();
                
                let x_start = 32 - x3_bytes.len().min(32);
                let y_start = 64 - y3_bytes.len().min(32);
                
                result[x_start..x_start + x3_bytes.len().min(32)].copy_from_slice(&x3_bytes[..x3_bytes.len().min(32)]);
                result[y_start..y_start + y3_bytes.len().min(32)].copy_from_slice(&y3_bytes[..y3_bytes.len().min(32)]);
                
                Ok(result)
            }
            None => Ok(vec![0u8; 64]),
        }
    }

    fn gas_cost(&self, _input: &[u8]) -> U256 {
        U256::from(6000)
    }
}

/// ECPAIRING precompiled contract (address 0x08)
pub struct EcPairingContract;

impl PrecompiledContract for EcPairingContract {
    fn execute(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.len() < 192 {
            return Ok(vec![0u8; 32]);
        }
        
        // For now, return a placeholder implementation
        // Full pairing implementation would require more complex elliptic curve operations
        let mut result = vec![0u8; 32];
        result[31] = 1; // Return 1 (true) as placeholder
        Ok(result)
    }

    fn gas_cost(&self, input: &[u8]) -> U256 {
        U256::from(45000 + (input.len() / 192) * 34000)
    }
}

/// Helper function to check if a point is on the BN254 curve
fn is_point_on_curve(x: &BigUint, y: &BigUint, p: &BigUint) -> bool {
    // BN254 curve equation: y^2 = x^3 + 3 (mod p)
    let y_squared = (y * y) % p;
    let x_cubed = (x * x * x) % p;
    let rhs = (x_cubed + BigUint::from(3u32)) % p;
    
    y_squared == rhs
}

/// Helper function for elliptic curve point addition
fn ec_add(x1: &BigUint, y1: &BigUint, x2: &BigUint, y2: &BigUint, p: &BigUint) -> Option<(BigUint, BigUint)> {
    // Handle point at infinity
    if x1 == &BigUint::zero() && y1 == &BigUint::zero() {
        return Some((x2.clone(), y2.clone()));
    }
    if x2 == &BigUint::zero() && y2 == &BigUint::zero() {
        return Some((x1.clone(), y1.clone()));
    }
    
    // Handle same point (point doubling)
    if x1 == x2 {
        if y1 == y2 {
            return ec_double(x1, y1, p);
        } else {
            return Some((BigUint::zero(), BigUint::zero())); // Point at infinity
        }
    }
    
    // Calculate slope
    let numerator = (y2 + p - y1) % p;
    let denominator = (x2 + p - x1) % p;
    
    // Calculate modular inverse of denominator
    let inv_denominator = mod_inverse(&denominator, p)?;
    let slope = (numerator * inv_denominator) % p;
    
    // Calculate x3 and y3
    let x3 = (slope.clone() * slope.clone() + p - x1 - x2) % p;
    let y3 = (slope * (x1 + p - &x3) + p - y1) % p;
    
    Some((x3, y3))
}

/// Helper function for elliptic curve point doubling
fn ec_double(x: &BigUint, y: &BigUint, p: &BigUint) -> Option<(BigUint, BigUint)> {
    if y == &BigUint::zero() {
        return Some((BigUint::zero(), BigUint::zero())); // Point at infinity
    }
    
    // Calculate slope for doubling
    let numerator = (BigUint::from(3u32) * x * x) % p;
    let denominator = (BigUint::from(2u32) * y) % p;
    
    let inv_denominator = mod_inverse(&denominator, p)?;
    let slope = (numerator * inv_denominator) % p;
    
    // Calculate x3 and y3
    let x3 = (slope.clone() * slope.clone() + p - BigUint::from(2u32) * x) % p;
    let y3 = (slope * (x + p - &x3) + p - y) % p;
    
    Some((x3, y3))
}

/// Helper function for scalar multiplication
fn ec_mul(x: &BigUint, y: &BigUint, k: &BigUint, p: &BigUint) -> Option<(BigUint, BigUint)> {
    let mut result_x = BigUint::zero();
    let mut result_y = BigUint::zero();
    let mut addend_x = x.clone();
    let mut addend_y = y.clone();
    let mut scalar = k.clone();
    
    while !scalar.is_zero() {
        if &scalar & &BigUint::from(1u32) != BigUint::zero() {
            if result_x.is_zero() && result_y.is_zero() {
                result_x = addend_x.clone();
                result_y = addend_y.clone();
            } else {
                let (new_x, new_y) = ec_add(&result_x, &result_y, &addend_x, &addend_y, p)?;
                result_x = new_x;
                result_y = new_y;
            }
        }
        
        let (double_x, double_y) = ec_double(&addend_x, &addend_y, p)?;
        addend_x = double_x;
        addend_y = double_y;
        scalar >>= 1;
    }
    
    Some((result_x, result_y))
}

/// Helper function to calculate modular inverse using extended Euclidean algorithm
fn mod_inverse(a: &BigUint, m: &BigUint) -> Option<BigUint> {
    if a.is_zero() {
        return None;
    }
    
    let mut old_r = a.clone();
    let mut r = m.clone();
    let mut old_s = BigUint::from(1u32);
    let mut s = BigUint::zero();
    
    while !r.is_zero() {
        let quotient = &old_r / &r;
        let temp_r = old_r.clone();
        old_r = r.clone();
        r = temp_r - &quotient * &r;
        
        let temp_s = old_s.clone();
        old_s = s.clone();
        s = temp_s - &quotient * &s;
    }
    
    if old_r > BigUint::from(1u32) {
        return None; // No inverse exists
    }
    
    if old_s < BigUint::zero() {
        Some(old_s + m)
    } else {
        Some(old_s)
    }
}

/// BLAKE2F precompiled contract (address 0x09)
pub struct Blake2FContract;

impl PrecompiledContract for Blake2FContract {
    fn execute(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.len() < 213 {
            return Ok(vec![0u8; 64]);
        }
        
        // For now, return zero (placeholder implementation)
        Ok(vec![0u8; 64])
    }

    fn gas_cost(&self, input: &[u8]) -> U256 {
        if input.len() < 213 {
            return U256::from(0);
        }
        
        let rounds = u32::from_be_bytes([input[212], input[213], input[214], input[215]]);
        U256::from(rounds)
    }
}

/// Create precompiled contracts registry
pub fn create_precompiled_registry() -> HashMap<Address, Box<dyn PrecompiledContract>> {
    let mut registry: HashMap<Address, Box<dyn PrecompiledContract>> = HashMap::new();
    
    // Register precompiled contracts
    registry.insert(Address::from([0x01; 20]), Box::new(EcrecoverContract) as Box<dyn PrecompiledContract>);
    registry.insert(Address::from([0x02; 20]), Box::new(Sha256Contract) as Box<dyn PrecompiledContract>);
    registry.insert(Address::from([0x03; 20]), Box::new(Ripemd160Contract) as Box<dyn PrecompiledContract>);
    registry.insert(Address::from([0x04; 20]), Box::new(IdentityContract) as Box<dyn PrecompiledContract>);
    registry.insert(Address::from([0x05; 20]), Box::new(ModExpContract) as Box<dyn PrecompiledContract>);
    registry.insert(Address::from([0x06; 20]), Box::new(EcAddContract) as Box<dyn PrecompiledContract>);
    registry.insert(Address::from([0x07; 20]), Box::new(EcMulContract) as Box<dyn PrecompiledContract>);
    registry.insert(Address::from([0x08; 20]), Box::new(EcPairingContract) as Box<dyn PrecompiledContract>);
    registry.insert(Address::from([0x09; 20]), Box::new(Blake2FContract) as Box<dyn PrecompiledContract>);
    
    registry
}