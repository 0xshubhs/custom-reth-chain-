/// Extra data structure for POA blocks
/// Format: [vanity (32 bytes)][signers list (N*20 bytes, only in epoch blocks)][signature (65 bytes)]
pub const EXTRA_VANITY_LENGTH: usize = 32;
/// Signature length in extra data (65 bytes: r=32, s=32, v=1)
pub const EXTRA_SEAL_LENGTH: usize = 65;
/// Ethereum address length (20 bytes)
pub const ADDRESS_LENGTH: usize = 20;
/// Default chain ID for Meowchain
pub const DEFAULT_CHAIN_ID: u64 = 9323310;
/// Default epoch length (blocks between signer list snapshots)
pub const DEFAULT_EPOCH: u64 = 30000;
