//! Approve (election proof) data structure and operations

use crate::core::types::*;
use crate::{Address, H256, Result, OlympusError};
use rlp::{Rlp, RlpStream, Encodable, Decodable};
use serde::{Deserialize, Serialize};

/// Approve (election proof) structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approve {
    /// Sender address (election participant)
    pub from: Address,
    /// Proof data (81 bytes)
    pub proof: Vec<u8>,
    /// Approve signature
    pub signature: Signature,
}

impl Approve {
    /// Create a new approve
    pub fn new(from: Address, proof: Vec<u8>, signature: Signature) -> Self {
        Self {
            from,
            proof,
            signature,
        }
    }

    /// Calculate approve hash
    pub fn hash(&self) -> ApproveHash {
        let rlp = self.rlp_bytes();
        crate::common::keccak256(&rlp)
    }

    /// Get RLP encoded bytes
    pub fn rlp_bytes(&self) -> Vec<u8> {
        let mut stream = RlpStream::new();
        self.rlp_append(&mut stream);
        stream.out().to_vec()
    }

    /// Validate approve structure
    pub fn validate(&self) -> Result<()> {
        // Check that from address is not zero
        if self.from == Address::zero() {
            return Err(OlympusError::InvalidTransaction("From address cannot be zero".to_string()));
        }

        // Check proof length (should be 81 bytes)
        if self.proof.len() != 81 {
            return Err(OlympusError::InvalidTransaction("Proof must be 81 bytes".to_string()));
        }

        // Validate signature
        self.validate_signature()?;

        Ok(())
    }

    /// Validate approve signature
    fn validate_signature(&self) -> Result<()> {
        // This is a simplified validation
        // In a full implementation, you would verify the ECDSA signature
        if self.signature.r == H256::zero() && self.signature.s == H256::zero() {
            return Err(OlympusError::InvalidTransaction("Invalid signature".to_string()));
        }
        Ok(())
    }

    /// Get sender address
    pub fn from(&self) -> Address {
        self.from
    }

    /// Get proof data
    pub fn proof(&self) -> &[u8] {
        &self.proof
    }

    /// Get signature
    pub fn signature(&self) -> &Signature {
        &self.signature
    }
}

impl Encodable for Approve {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(5);
        s.append(&self.from);
        s.append(&self.proof);
        s.append(&self.signature.v);
        s.append(&self.signature.r);
        s.append(&self.signature.s);
    }
}

impl Decodable for Approve {
    fn decode(rlp: &Rlp) -> std::result::Result<Self, rlp::DecoderError> {
        if rlp.item_count()? != 5 {
            return Err(rlp::DecoderError::RlpIncorrectListLen);
        }

        Ok(Approve {
            from: rlp.val_at(0)?,
            proof: rlp.val_at(1)?,
            signature: Signature {
                v: rlp.val_at(2)?,
                r: rlp.val_at(3)?,
                s: rlp.val_at(4)?,
            },
        })
    }
}

/// Approve receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveReceipt {
    /// Sender address
    pub from: Address,
    /// Receipt output
    pub output: Vec<u8>,
    /// Execution status
    pub status: bool,
}

impl ApproveReceipt {
    /// Create a new approve receipt
    pub fn new(from: Address, output: Vec<u8>, status: bool) -> Self {
        Self {
            from,
            output,
            status,
        }
    }

    /// Get sender address
    pub fn from(&self) -> Address {
        self.from
    }

    /// Get output data
    pub fn output(&self) -> &[u8] {
        &self.output
    }

    /// Get execution status
    pub fn status(&self) -> bool {
        self.status
    }
}

/// Collection of approves
pub type Approves = Vec<Approve>;
