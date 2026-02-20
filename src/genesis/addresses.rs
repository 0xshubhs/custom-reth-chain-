use alloy_primitives::{address, b256, Address, B256};

/// EIP-1967 Miner Proxy address - block rewards (coinbase) are sent here.
/// This proxy allows upgrading the reward distribution logic without changing consensus.
///
/// Storage layout (EIP-1967):
/// - Implementation slot: 0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc
/// - Admin slot: 0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103
pub const MINER_PROXY_ADDRESS: Address = address!("0000000000000000000000000000000000001967");

/// Admin slot for EIP-1967 proxy
pub(crate) const EIP1967_ADMIN_SLOT: B256 =
    b256!("b53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103");

/// ChainConfig contract address (deterministic, pre-assigned)
pub const CHAIN_CONFIG_ADDRESS: Address = address!("00000000000000000000000000000000C04F1600");

/// SignerRegistry contract address (deterministic, pre-assigned)
pub const SIGNER_REGISTRY_ADDRESS: Address = address!("000000000000000000000000000000005164EB00");

/// Treasury contract address (deterministic, pre-assigned)
pub const TREASURY_ADDRESS: Address = address!("0000000000000000000000000000000007EA5B00");

/// Timelock contract address (delay-enforcing governance)
pub const TIMELOCK_ADDRESS: Address = address!("00000000000000000000000000000000714E4C00");

/// Governance Safe address (the multisig that controls all governance contracts).
/// In dev mode this is just the first dev account; in production it would be a
/// deployed Gnosis Safe proxy.
pub const GOVERNANCE_SAFE_ADDRESS: Address = address!("000000000000000000000000000000006F5AFE00");

/// Gnosis Safe Singleton v1.3.0 canonical address
pub const SAFE_SINGLETON_ADDRESS: Address =
    address!("d9Db270c1B5E3Bd161E8c8503c55cEABeE709552");

/// Gnosis Safe Proxy Factory canonical address
pub const SAFE_PROXY_FACTORY_ADDRESS: Address =
    address!("a6B71E26C5e0845f74c812102Ca7114b6a896AB2");

/// Gnosis Safe Compatibility Fallback Handler canonical address
pub const SAFE_FALLBACK_HANDLER_ADDRESS: Address =
    address!("f48f2B2d2a534e402487b3ee7C18c33Aec0Fe5e4");

/// Gnosis Safe MultiSend canonical address
pub const SAFE_MULTISEND_ADDRESS: Address =
    address!("A238CBeb142c10Ef7Ad8442C6D1f9E89e07e7761");
