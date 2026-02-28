//! Clique RPC Response Types
//!
//! Types for the standard `clique_*` RPC namespace used by tools like
//! MetaMask and Blockscout to interact with Clique POA networks.

use alloy_primitives::{Address, B256};
use serde::Serialize;
use std::collections::HashMap;

/// Response for `clique_getSnapshot`
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliqueSnapshot {
    /// Block number this snapshot was taken at
    pub number: u64,
    /// Block hash this snapshot was taken at
    pub hash: B256,
    /// Set of authorized signers at this moment
    pub signers: Vec<Address>,
    /// Current list of votes (signer -> (address, authorize))
    pub votes: Vec<CliqueVote>,
    /// Current tally of votes
    pub tally: HashMap<Address, CliqueTally>,
}

/// A pending signer vote
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliqueVote {
    /// Signer who cast the vote
    pub signer: Address,
    /// Target address being voted on
    pub address: Address,
    /// Whether this is an authorize (true) or deauthorize (false) vote
    pub authorize: bool,
}

/// Tally of votes for an address
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliqueTally {
    /// Whether this is an authorize (true) or deauthorize (false) tally
    pub authorize: bool,
    /// Number of votes received
    pub votes: u64,
}

/// Response for `clique_status`
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliqueStatus {
    /// Number of signers currently in the set
    pub signer_count: usize,
    /// Number of blocks in the latest snapshot's sealing history
    pub num_blocks: u64,
    /// Map of signer address to number of blocks signed recently
    pub sealers_activity: HashMap<Address, SealerActivity>,
}

/// Activity info for a sealer
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SealerActivity {
    /// Number of blocks signed
    pub signed: u64,
    /// Total number of blocks they could have signed (in-turn blocks)
    pub total: u64,
}

/// Proposals for signer addition/removal
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliqueProposals {
    /// Map of proposed address -> authorize (true to add, false to remove)
    pub proposals: HashMap<Address, bool>,
}
