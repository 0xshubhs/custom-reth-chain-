/// ChainConfig contract storage layout.
///
/// Matches `contracts/ChainConfig.sol` and `genesis.rs:governance_contract_alloc`.
pub mod chain_config_slots {
    use alloy_primitives::U256;

    /// slot 0: governance (address)
    pub const GOVERNANCE: U256 = U256::from_limbs([0, 0, 0, 0]);
    /// slot 1: gasLimit (uint256)
    pub const GAS_LIMIT: U256 = U256::from_limbs([1, 0, 0, 0]);
    /// slot 2: blockTime (uint256)
    pub const BLOCK_TIME: U256 = U256::from_limbs([2, 0, 0, 0]);
    /// slot 3: maxContractSize (uint256)
    pub const MAX_CONTRACT_SIZE: U256 = U256::from_limbs([3, 0, 0, 0]);
    /// slot 4: calldataGasPerByte (uint256)
    pub const CALLDATA_GAS_PER_BYTE: U256 = U256::from_limbs([4, 0, 0, 0]);
    /// slot 5: maxTxGas (uint256)
    pub const MAX_TX_GAS: U256 = U256::from_limbs([5, 0, 0, 0]);
    /// slot 6: eagerMining (bool)
    pub const EAGER_MINING: U256 = U256::from_limbs([6, 0, 0, 0]);
}

/// SignerRegistry contract storage layout.
///
/// Matches `contracts/SignerRegistry.sol` and `genesis.rs:governance_contract_alloc`.
pub mod signer_registry_slots {
    use alloy_primitives::U256;

    /// slot 0: governance (address)
    pub const GOVERNANCE: U256 = U256::from_limbs([0, 0, 0, 0]);
    /// slot 1: signers.length (dynamic array length)
    pub const SIGNERS_LENGTH: U256 = U256::from_limbs([1, 0, 0, 0]);
    /// slot 2: isSigner mapping base (mapping(address => bool))
    pub const IS_SIGNER_MAPPING: U256 = U256::from_limbs([2, 0, 0, 0]);
    /// slot 3: signerThreshold (uint256)
    pub const SIGNER_THRESHOLD: U256 = U256::from_limbs([3, 0, 0, 0]);
}

/// Timelock contract storage layout.
///
/// Matches `genesis-contracts/Timelock.sol` and `genesis.rs:governance_contract_alloc`.
pub mod timelock_slots {
    use alloy_primitives::U256;

    /// slot 0: minDelay (uint256) — minimum delay in seconds before execution
    pub const MIN_DELAY: U256 = U256::from_limbs([0, 0, 0, 0]);
    /// slot 1: proposer (address)
    pub const PROPOSER: U256 = U256::from_limbs([1, 0, 0, 0]);
    /// slot 2: executor (address)
    pub const EXECUTOR: U256 = U256::from_limbs([2, 0, 0, 0]);
    /// slot 3: admin (address)
    pub const ADMIN: U256 = U256::from_limbs([3, 0, 0, 0]);
    /// slot 4: paused (bool)
    pub const PAUSED: U256 = U256::from_limbs([4, 0, 0, 0]);
}
