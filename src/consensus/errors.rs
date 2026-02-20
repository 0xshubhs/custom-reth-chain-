use alloy_primitives::Address;
use reth_consensus::ConsensusError;
use thiserror::Error;

/// POA-specific consensus errors
#[derive(Debug, Error)]
#[allow(missing_docs)]
pub enum PoaConsensusError {
    /// Block signer is not in the authorized signers list
    #[error("Block signer {signer} is not authorized")]
    UnauthorizedSigner {
        /// The unauthorized signer address
        signer: Address,
    },

    /// Block signature is invalid or cannot be recovered
    #[error("Invalid block signature")]
    InvalidSignature,

    /// Extra data is too short to contain required POA information
    #[error("Extra data too short: expected at least {expected} bytes, got {got}")]
    ExtraDataTooShort {
        /// Expected minimum length
        expected: usize,
        /// Actual length
        got: usize,
    },

    /// Block timestamp is earlier than allowed
    #[error("Block timestamp {timestamp} is before parent timestamp {parent_timestamp}")]
    TimestampTooEarly {
        /// Block timestamp
        timestamp: u64,
        /// Parent block timestamp
        parent_timestamp: u64,
    },

    /// Block timestamp is too far in the future
    #[error("Block timestamp {timestamp} is too far in the future")]
    TimestampTooFarInFuture {
        /// Block timestamp
        timestamp: u64,
    },

    /// Block was signed by wrong signer (not in-turn)
    #[error("Wrong block signer: expected {expected}, got {got}")]
    WrongSigner {
        /// Expected signer
        expected: Address,
        /// Actual signer
        got: Address,
    },

    /// Difficulty field has invalid value for POA
    #[error("Difficulty must be 0 (Engine API compatibility; authority is via ECDSA signature)")]
    InvalidDifficulty,

    /// Signer list in epoch block is invalid
    #[error("Invalid signer list in epoch block")]
    InvalidSignerList,
}

impl From<PoaConsensusError> for ConsensusError {
    fn from(err: PoaConsensusError) -> Self {
        ConsensusError::Custom(std::sync::Arc::new(err))
    }
}
