//! Cryptographic utilities

use crate::H256;
use sha3::{Digest, Keccak256};

/// Calculate Keccak256 hash
pub fn keccak256(data: &[u8]) -> H256 {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    H256::from_slice(&hasher.finalize())
}

/// Calculate Keccak256 hash of RLP encoded data
pub fn keccak256_rlp<T: rlp::Encodable>(data: &T) -> H256 {
    let mut stream = rlp::RlpStream::new();
    data.rlp_append(&mut stream);
    keccak256(&stream.out())
}
