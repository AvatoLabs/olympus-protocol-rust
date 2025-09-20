//! Transaction data structure and operations

use crate::core::types::*;
use crate::{Address, H256, U256, Result, OlympusError};
use rlp::{Rlp, RlpStream, Encodable, Decodable};
use serde::{Deserialize, Serialize};

/// Transaction skeleton for building transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSkeleton {
    pub from: Address,
    pub to: Address,
    pub value: U256,
    pub data: Vec<u8>,
    pub nonce: U256,
    pub gas: U256,
    pub gas_price: U256,
}

/// Transaction signature inclusion options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncludeSignature {
    WithoutSignature = 0,
    WithSignature = 1,
}

/// Transaction validation level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckTransaction {
    None,
    Cheap,
    Everything,
}

/// Olympus transaction structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction nonce
    pub nonce: U256,
    /// Value to transfer in wei
    pub value: U256,
    /// Receiving address (zero for contract creation)
    pub receive_address: Address,
    /// Gas price in wei
    pub gas_price: U256,
    /// Gas limit
    pub gas: U256,
    /// Transaction data or contract creation code
    pub data: Vec<u8>,
    /// Transaction signature
    pub signature: Option<Signature>,
    /// Chain ID for replay protection
    pub chain_id: Option<u64>,
}

impl Transaction {
    /// Create a new unsigned message call transaction
    pub fn new(
        value: U256,
        gas_price: U256,
        gas: U256,
        dest: Address,
        data: Vec<u8>,
        nonce: U256,
    ) -> Self {
        Self {
            nonce,
            value,
            receive_address: dest,
            gas_price,
            gas,
            data,
            signature: None,
            chain_id: Some(crate::core::types::CHAIN_ID),
        }
    }

    /// Create a new unsigned contract creation transaction
    pub fn new_contract_creation(
        value: U256,
        gas_price: U256,
        gas: U256,
        data: Vec<u8>,
        nonce: U256,
    ) -> Self {
        Self {
            nonce,
            value,
            receive_address: Address::zero(),
            gas_price,
            gas,
            data,
            signature: None,
            chain_id: Some(crate::core::types::CHAIN_ID),
        }
    }

    /// Create transaction from skeleton
    pub fn from_skeleton(skeleton: TransactionSkeleton, secret: Option<&[u8]>) -> Result<Self> {
        let mut tx = Self {
            nonce: skeleton.nonce,
            value: skeleton.value,
            receive_address: skeleton.to,
            gas_price: skeleton.gas_price,
            gas: skeleton.gas,
            data: skeleton.data,
            signature: None,
            chain_id: Some(crate::core::types::CHAIN_ID),
        };

        if let Some(secret_bytes) = secret {
            tx.sign_with_secret(secret_bytes)?;
        }

        Ok(tx)
    }

    /// Get transaction sender address
    pub fn sender(&self) -> Result<Address> {
        match &self.signature {
            Some(sig) => {
                self.recover_sender_from_signature(sig)
            }
            None => Err(OlympusError::InvalidTransaction("Transaction is unsigned".to_string())),
        }
    }

    /// Recover sender address from signature
    fn recover_sender_from_signature(&self, sig: &Signature) -> Result<Address> {
        use secp256k1::{Secp256k1, Message};
        use secp256k1::ecdsa::{RecoverableSignature, RecoveryId};
        
        let secp = Secp256k1::new();
        
        // Create message hash
        let message_hash = self.hash();
        let message = Message::from_digest_slice(&message_hash.as_bytes())
            .map_err(|_| OlympusError::InvalidTransaction("Invalid message hash".to_string()))?;
        
        // Calculate recovery ID from v value
        let chain_id = self.chain_id.unwrap_or(1);
        let recovery_id_value = sig.v as i32 - 27 - (chain_id * 2 + 35) as i32;
        let recovery_id = RecoveryId::from_i32(recovery_id_value)
            .map_err(|_| OlympusError::InvalidTransaction("Invalid recovery ID".to_string()))?;
        
        // Reconstruct signature
        let mut signature_bytes = [0u8; 64];
        signature_bytes[0..32].copy_from_slice(sig.r.as_bytes());
        signature_bytes[32..64].copy_from_slice(sig.s.as_bytes());
        
        let recoverable_sig = RecoverableSignature::from_compact(&signature_bytes, recovery_id)
            .map_err(|_| OlympusError::InvalidTransaction("Invalid signature".to_string()))?;
        
        // Recover public key
        let public_key = secp.recover_ecdsa(&message, &recoverable_sig)
            .map_err(|_| OlympusError::InvalidTransaction("Signature recovery failed".to_string()))?;
        
        // Convert public key to address (last 20 bytes of keccak256 hash)
        let public_key_bytes = public_key.serialize_uncompressed();
        let hash = crate::common::keccak256(&public_key_bytes[1..]); // Skip the 0x04 prefix
        let address_bytes = &hash[12..]; // Take last 20 bytes
        
        Ok(Address::from_slice(address_bytes))
    }

    /// Get transaction sender address without throwing
    pub fn safe_sender(&self) -> Address {
        self.sender().unwrap_or(Address::zero())
    }

    /// Check if transaction has signature
    pub fn has_signature(&self) -> bool {
        self.signature.is_some()
    }

    /// Check if transaction has zero signature
    pub fn has_zero_signature(&self) -> bool {
        if let Some(sig) = &self.signature {
            sig.r == H256::zero() && sig.s == H256::zero()
        } else {
            false
        }
    }

    /// Get chain ID
    pub fn chain_id(&self) -> Option<u64> {
        self.chain_id
    }

    /// Set signature
    pub fn set_signature(&mut self, r: H256, s: H256, v: u8) {
        self.signature = Some(Signature { v, r, s });
    }

    /// Get transaction sender address (alias for safe_sender)
    pub fn from(&self) -> Address {
        self.safe_sender()
    }

    /// Force sender to a particular value (for gas estimation)
    pub fn force_sender(&mut self, _sender: Address) {
        // This would be used for gas estimation where we don't have a real signature
        // Implementation would depend on how gas estimation works
    }

    /// Check if transaction is contract creation
    pub fn is_creation(&self) -> bool {
        self.receive_address == Address::zero()
    }

    /// Get transaction hash
    pub fn hash(&self) -> TransactionHash {
        let rlp = self.rlp_bytes(IncludeSignature::WithSignature);
        crate::common::keccak256(&rlp)
    }

    /// Get RLP encoded bytes
    pub fn rlp_bytes(&self, include_sig: IncludeSignature) -> Vec<u8> {
        let mut stream = RlpStream::new();
        self.rlp_append_with_signature(&mut stream, include_sig);
        stream.out().to_vec()
    }

    /// Sign transaction with private key
    pub fn sign_with_secret(&mut self, secret: &[u8]) -> Result<()> {
        use secp256k1::{SecretKey, Secp256k1, Message};
        
        if secret.len() != 32 {
            return Err(OlympusError::InvalidTransaction("Private key must be 32 bytes".to_string()));
        }
        
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(secret)
            .map_err(|_| OlympusError::InvalidTransaction("Invalid private key".to_string()))?;
        
        // Create message hash for signing
        let message_hash = self.hash();
        let message = Message::from_digest_slice(&message_hash.as_bytes())
            .map_err(|_| OlympusError::InvalidTransaction("Invalid message hash".to_string()))?;
        
        // Sign the message
        let signature = secp.sign_ecdsa_recoverable(&message, &secret_key);
        let (recovery_id, signature_bytes) = signature.serialize_compact();
        
        // Extract r and s from signature
        let mut r_bytes = [0u8; 32];
        let mut s_bytes = [0u8; 32];
        r_bytes.copy_from_slice(&signature_bytes[0..32]);
        s_bytes.copy_from_slice(&signature_bytes[32..64]);
        
        // Calculate v value with chain ID
        let chain_id = self.chain_id.unwrap_or(1);
        let v = recovery_id.to_i32() as u8 + 27 + (chain_id * 2 + 35) as u8;
        
        self.signature = Some(Signature {
            v,
            r: H256::from_slice(&r_bytes),
            s: H256::from_slice(&s_bytes),
        });
        
        Ok(())
    }

    /// Validate transaction
    pub fn validate(&self, check_level: CheckTransaction) -> Result<()> {
        match check_level {
            CheckTransaction::None => Ok(()),
            CheckTransaction::Cheap => {
                // Basic validation
                if self.gas == U256::zero() {
                    return Err(OlympusError::InvalidTransaction("Gas cannot be zero".to_string()));
                }
                if self.gas_price == U256::zero() {
                    return Err(OlympusError::InvalidTransaction("Gas price cannot be zero".to_string()));
                }
                Ok(())
            }
            CheckTransaction::Everything => {
                // Full validation including signature verification
                self.validate(CheckTransaction::Cheap)?;
                
                if self.signature.is_none() {
                    return Err(OlympusError::InvalidTransaction("Transaction must be signed".to_string()));
                }
                
                // Additional validation would go here
                Ok(())
            }
        }
    }

    /// Calculate base gas required
    pub fn base_gas_required(&self) -> u64 {
        let mut gas = 21000; // Base transaction cost
        
        if self.is_creation() {
            gas += 32000; // Contract creation cost
        }
        
        // Add cost for data (68 gas per non-zero byte, 4 gas per zero byte)
        for byte in &self.data {
            if *byte == 0 {
                gas += 4;
            } else {
                gas += 68;
            }
        }
        
        gas
    }

    /// Get transaction value
    pub fn value(&self) -> U256 {
        self.value
    }

    /// Get gas price
    pub fn gas_price(&self) -> U256 {
        self.gas_price
    }

    /// Get gas limit
    pub fn gas(&self) -> U256 {
        self.gas
    }

    /// Get receiving address
    pub fn receive_address(&self) -> Address {
        self.receive_address
    }

    /// Get transaction data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get transaction nonce
    pub fn nonce(&self) -> U256 {
        self.nonce
    }
}

impl Encodable for Transaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        self.rlp_append_with_signature(s, IncludeSignature::WithSignature);
    }
}

impl Transaction {
    fn rlp_append_with_signature(&self, s: &mut RlpStream, include_sig: IncludeSignature) {
        match include_sig {
            IncludeSignature::WithSignature => {
                s.begin_list(9);
                s.append(&self.nonce);
                s.append(&self.gas_price);
                s.append(&self.gas);
                s.append(&self.receive_address);
                s.append(&self.value);
                s.append(&self.data);
                s.append(&self.chain_id.unwrap_or(0));
                s.append(&0u8); // r
                s.append(&0u8); // s
                s.append(&0u8); // v
            }
            IncludeSignature::WithoutSignature => {
                s.begin_list(6);
                s.append(&self.nonce);
                s.append(&self.gas_price);
                s.append(&self.gas);
                s.append(&self.receive_address);
                s.append(&self.value);
                s.append(&self.data);
            }
        }
    }
}

impl Decodable for Transaction {
    fn decode(rlp: &Rlp) -> std::result::Result<Self, rlp::DecoderError> {
        let item_count = rlp.item_count()?;
        
        if item_count == 6 {
            // Unsigned transaction
            Ok(Transaction {
                nonce: rlp.val_at(0)?,
                gas_price: rlp.val_at(1)?,
                gas: rlp.val_at(2)?,
                receive_address: rlp.val_at(3)?,
                value: rlp.val_at(4)?,
                data: rlp.val_at(5)?,
                signature: None,
                chain_id: None,
            })
        } else if item_count == 9 {
            // Signed transaction
            Ok(Transaction {
                nonce: rlp.val_at(0)?,
                gas_price: rlp.val_at(1)?,
                gas: rlp.val_at(2)?,
                receive_address: rlp.val_at(3)?,
                value: rlp.val_at(4)?,
                data: rlp.val_at(5)?,
                chain_id: Some(rlp.val_at(6)?),
                signature: Some(Signature {
                    v: rlp.val_at(9)?,
                    r: rlp.val_at(7)?,
                    s: rlp.val_at(8)?,
                }),
            })
        } else {
            Err(rlp::DecoderError::RlpIncorrectListLen)
        }
    }
}

/// Localized transaction with block metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedTransaction {
    /// Base transaction
    pub transaction: Transaction,
    /// Block hash containing this transaction
    pub block_hash: BlockHash,
    /// Transaction index in block
    pub transaction_index: u32,
    /// Block number
    pub block_number: u64,
}

impl LocalizedTransaction {
    /// Create a new localized transaction
    pub fn new(
        transaction: Transaction,
        block_hash: BlockHash,
        transaction_index: u32,
        block_number: u64,
    ) -> Self {
        Self {
            transaction,
            block_hash,
            transaction_index,
            block_number,
        }
    }

    /// Get block hash
    pub fn block_hash(&self) -> BlockHash {
        self.block_hash
    }

    /// Get transaction index
    pub fn transaction_index(&self) -> u32 {
        self.transaction_index
    }

    /// Get block number
    pub fn block_number(&self) -> u64 {
        self.block_number
    }
}

/// Collection of transactions
pub type Transactions = Vec<Transaction>;
