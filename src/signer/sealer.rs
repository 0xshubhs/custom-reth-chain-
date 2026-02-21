use alloy_consensus::Header;
use alloy_primitives::{keccak256, Address, Signature, B256};
use std::sync::Arc;

use super::errors::SignerError;
use super::manager::SignerManager;

/// Block sealing utilities for POA
#[derive(Debug)]
pub struct BlockSealer {
    signer_manager: Arc<SignerManager>,
}

impl BlockSealer {
    /// Create a new block sealer
    pub fn new(signer_manager: Arc<SignerManager>) -> Self {
        Self { signer_manager }
    }

    /// Calculate the seal hash for a header (hash without signature)
    pub fn seal_hash(header: &Header) -> B256 {
        // Create a copy with signature stripped from extra data
        let mut header_for_hash = header.clone();

        const EXTRA_SEAL_LENGTH: usize = 65;
        let extra_data = &header.extra_data;
        if extra_data.len() >= EXTRA_SEAL_LENGTH {
            let without_seal = &extra_data[..extra_data.len() - EXTRA_SEAL_LENGTH];
            header_for_hash.extra_data = without_seal.to_vec().into();
        }

        keccak256(alloy_rlp::encode(&header_for_hash))
    }

    /// Seal a block header with a signature
    pub async fn seal_header(
        &self,
        mut header: Header,
        signer_address: &Address,
    ) -> Result<Header, SignerError> {
        // Calculate seal hash
        let seal_hash = Self::seal_hash(&header);

        // Sign the hash
        let signature = self
            .signer_manager
            .sign_hash(signer_address, seal_hash)
            .await?;

        // Encode signature as bytes (r, s, v)
        let sig_bytes = signature_to_bytes(&signature);

        // Update extra data with signature
        let mut extra_data = header.extra_data.to_vec();

        // Remove existing signature if present
        const EXTRA_SEAL_LENGTH: usize = 65;
        if extra_data.len() >= EXTRA_SEAL_LENGTH {
            extra_data.truncate(extra_data.len() - EXTRA_SEAL_LENGTH);
        }

        // Append new signature
        extra_data.extend_from_slice(&sig_bytes);
        header.extra_data = extra_data.into();

        Ok(header)
    }

    /// Verify a block's signature
    pub fn verify_signature(header: &Header) -> Result<Address, SignerError> {
        let seal_hash = Self::seal_hash(header);

        let extra_data = &header.extra_data;
        const EXTRA_SEAL_LENGTH: usize = 65;

        if extra_data.len() < EXTRA_SEAL_LENGTH {
            return Err(SignerError::SigningFailed("Extra data too short".into()));
        }

        let sig_bytes = &extra_data[extra_data.len() - EXTRA_SEAL_LENGTH..];
        let signature = bytes_to_signature(sig_bytes).map_err(SignerError::SigningFailed)?;

        signature
            .recover_address_from_prehash(&seal_hash)
            .map_err(|e| SignerError::SigningFailed(e.to_string()))
    }
}

/// Convert a signature to bytes (r || s || v)
pub fn signature_to_bytes(sig: &Signature) -> [u8; 65] {
    let mut bytes = [0u8; 65];
    bytes[..32].copy_from_slice(&sig.r().to_be_bytes::<32>());
    bytes[32..64].copy_from_slice(&sig.s().to_be_bytes::<32>());
    bytes[64] = sig.v() as u8;
    bytes
}

/// Convert bytes to a signature
pub fn bytes_to_signature(bytes: &[u8]) -> Result<Signature, String> {
    if bytes.len() != 65 {
        return Err(format!(
            "Invalid signature length: expected 65, got {}",
            bytes.len()
        ));
    }

    Signature::try_from(bytes).map_err(|e| format!("Invalid signature: {}", e))
}
