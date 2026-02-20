use alloy_primitives::Address;
use serde::{Deserialize, Serialize};

/// POA-specific configuration that extends the standard chain config
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoaConfig {
    /// Block period in seconds (time between blocks)
    pub period: u64,
    /// Number of blocks after which to checkpoint and reset the pending votes
    pub epoch: u64,
    /// List of authorized signer addresses
    pub signers: Vec<Address>,
}

impl Default for PoaConfig {
    fn default() -> Self {
        Self {
            period: 12, // 12 second block time like mainnet
            epoch: 30000,
            signers: vec![],
        }
    }
}
