use alloy_primitives::Address;
use thiserror::Error;

/// Errors that can occur during signing operations
#[derive(Debug, Error)]
pub enum SignerError {
    /// No signing key available for the specified address
    #[error("No signer available for address {0}")]
    NoSignerForAddress(Address),

    /// Signing operation failed
    #[error("Signing failed: {0}")]
    SigningFailed(String),

    /// Invalid private key format
    #[error("Invalid private key")]
    InvalidPrivateKey,
}
