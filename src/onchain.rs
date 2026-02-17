//! On-chain Governance Contract Readers
//!
//! Reads dynamic chain parameters from the ChainConfig and SignerRegistry
//! contracts deployed in genesis. This is THE critical wiring that makes
//! the chain governable without downtime.
//!
//! Architecture:
//!   Governance Safe → ChainConfig.setGasLimit(300_000_000)
//!   Governance Safe → SignerRegistry.addSigner(addr)
//!   ↓
//!   PoaPayloadBuilder reads ChainConfig.gasLimit at each block
//!   PoaConsensus reads SignerRegistry.getSigners at epoch blocks
//!   ↓
//!   No restart, no recompile, no downtime
//!
//! Storage layout must match genesis.rs pre-population and the Solidity contracts.

use alloy_primitives::{keccak256, Address, Keccak256, B256, U256};
use crate::genesis::{CHAIN_CONFIG_ADDRESS, SIGNER_REGISTRY_ADDRESS};

// =============================================================================
// Storage Slot Constants — must match Solidity storage layout
// =============================================================================

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

// =============================================================================
// ABI Function Selectors — for eth_call interface
// =============================================================================

/// Compute the Solidity function selector (first 4 bytes of keccak256(signature)).
pub fn function_selector(signature: &str) -> [u8; 4] {
    let hash = keccak256(signature.as_bytes());
    let mut selector = [0u8; 4];
    selector.copy_from_slice(&hash[..4]);
    selector
}

/// Pre-computed ABI function selectors for governance contract view functions.
pub mod selectors {
    use super::function_selector;

    // ChainConfig getters
    pub fn gas_limit() -> [u8; 4] { function_selector("gasLimit()") }
    pub fn block_time() -> [u8; 4] { function_selector("blockTime()") }
    pub fn max_contract_size() -> [u8; 4] { function_selector("maxContractSize()") }
    pub fn calldata_gas_per_byte() -> [u8; 4] { function_selector("calldataGasPerByte()") }
    pub fn max_tx_gas() -> [u8; 4] { function_selector("maxTxGas()") }
    pub fn eager_mining() -> [u8; 4] { function_selector("eagerMining()") }
    pub fn governance() -> [u8; 4] { function_selector("governance()") }

    // SignerRegistry getters
    pub fn get_signers() -> [u8; 4] { function_selector("getSigners()") }
    pub fn signer_count() -> [u8; 4] { function_selector("signerCount()") }
    pub fn signer_threshold() -> [u8; 4] { function_selector("signerThreshold()") }
    pub fn is_signer() -> [u8; 4] { function_selector("isSigner(address)") }
}

// =============================================================================
// Storage Reading Helpers
// =============================================================================

/// Compute the base slot for a Solidity dynamic array's data.
///
/// For `address[] public signers` at slot 1:
///   base = keccak256(abi.encode(1))
///   signers[0] lives at base + 0
///   signers[1] lives at base + 1
///   etc.
pub fn dynamic_array_base_slot(array_slot: U256) -> U256 {
    let mut hasher = Keccak256::new();
    hasher.update(B256::from(array_slot.to_be_bytes()).as_slice());
    U256::from_be_bytes(hasher.finalize().0)
}

/// Compute the storage slot for a Solidity `mapping(address => bool)` entry.
///
/// For `isSigner[addr]` at mapping slot 2:
///   slot = keccak256(abi.encode(addr, 2))
pub fn mapping_address_bool_slot(key: Address, mapping_slot: U256) -> B256 {
    let mut hasher = Keccak256::new();
    let mut key_padded = [0u8; 32];
    key_padded[12..32].copy_from_slice(key.as_slice());
    hasher.update(&key_padded);
    hasher.update(B256::from(mapping_slot.to_be_bytes()).as_slice());
    hasher.finalize()
}

/// Decode an address from a B256 storage value (left-padded with zeros).
pub fn decode_address(value: B256) -> Address {
    Address::from_slice(&value[12..32])
}

/// Decode a u64 from a B256 storage value.
pub fn decode_u64(value: B256) -> u64 {
    U256::from_be_bytes(value.0).as_limbs()[0]
}

/// Decode a bool from a B256 storage value.
pub fn decode_bool(value: B256) -> bool {
    value[31] != 0
}

/// Encode a u64 value into a B256 storage value.
pub fn encode_u64(value: u64) -> B256 {
    B256::from(U256::from(value).to_be_bytes())
}

/// Encode an address into a B256 storage value (left-padded).
pub fn encode_address(addr: Address) -> B256 {
    let mut bytes = [0u8; 32];
    bytes[12..32].copy_from_slice(addr.as_slice());
    B256::from(bytes)
}

// =============================================================================
// StorageReader trait — abstracts storage access for testing
// =============================================================================

/// Trait for reading contract storage slots.
///
/// In production: implemented by the state provider (MDBX database)
/// In tests: implemented by GenesisStorageReader (reads from genesis alloc)
pub trait StorageReader {
    /// Read a storage slot value from a contract address.
    /// Returns None if the contract or slot doesn't exist.
    fn read_storage(&self, address: Address, slot: U256) -> Option<B256>;
}

// =============================================================================
// DynamicChainConfig — runtime chain parameters
// =============================================================================

/// Dynamic chain configuration read from the on-chain ChainConfig contract.
///
/// Replaces hardcoded values from genesis/CLI with governance-controlled parameters.
/// Updated via: Governance Safe → ChainConfig.setGasLimit(300_000_000)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicChainConfig {
    /// Governance address (the Safe multisig)
    pub governance: Address,
    /// Block gas limit (default: 30_000_000)
    pub gas_limit: u64,
    /// Block interval in seconds (default: 2)
    pub block_time: u64,
    /// Max contract bytecode size (default: 24_576)
    pub max_contract_size: u64,
    /// Calldata gas cost per byte (default: 16)
    pub calldata_gas_per_byte: u64,
    /// Max gas per transaction (default: gasLimit)
    pub max_tx_gas: u64,
    /// Mine on tx arrival vs interval (default: false)
    pub eager_mining: bool,
}

/// Dynamic signer list read from the on-chain SignerRegistry contract.
///
/// Updated via: Governance Safe → SignerRegistry.addSigner(addr)
/// Changes take effect at the next epoch block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicSignerList {
    /// Governance address (the Safe multisig)
    pub governance: Address,
    /// Ordered list of authorized signers
    pub signers: Vec<Address>,
    /// Minimum signers for chain liveness
    pub threshold: u64,
}

// =============================================================================
// Reading functions
// =============================================================================

/// Read the full ChainConfig from on-chain storage.
///
/// This is called by PoaPayloadBuilder at each block to get the current gas limit
/// and other parameters. The Governance Safe can change these live via transactions.
pub fn read_chain_config(reader: &impl StorageReader) -> Option<DynamicChainConfig> {
    let addr = CHAIN_CONFIG_ADDRESS;

    let governance_val = reader.read_storage(addr, chain_config_slots::GOVERNANCE)?;
    let gas_limit_val = reader.read_storage(addr, chain_config_slots::GAS_LIMIT)?;
    let block_time_val = reader.read_storage(addr, chain_config_slots::BLOCK_TIME)?;
    let max_contract_size_val = reader.read_storage(addr, chain_config_slots::MAX_CONTRACT_SIZE)?;
    let calldata_gas_val = reader.read_storage(addr, chain_config_slots::CALLDATA_GAS_PER_BYTE)?;
    let max_tx_gas_val = reader.read_storage(addr, chain_config_slots::MAX_TX_GAS)?;
    let eager_mining_val = reader
        .read_storage(addr, chain_config_slots::EAGER_MINING)
        .unwrap_or(B256::ZERO);

    Some(DynamicChainConfig {
        governance: decode_address(governance_val),
        gas_limit: decode_u64(gas_limit_val),
        block_time: decode_u64(block_time_val),
        max_contract_size: decode_u64(max_contract_size_val),
        calldata_gas_per_byte: decode_u64(calldata_gas_val),
        max_tx_gas: decode_u64(max_tx_gas_val),
        eager_mining: decode_bool(eager_mining_val),
    })
}

/// Read just the gas limit from ChainConfig (hot path for payload builder).
pub fn read_gas_limit(reader: &impl StorageReader) -> Option<u64> {
    reader
        .read_storage(CHAIN_CONFIG_ADDRESS, chain_config_slots::GAS_LIMIT)
        .map(|v| decode_u64(v))
}

/// Read just the block time from ChainConfig.
pub fn read_block_time(reader: &impl StorageReader) -> Option<u64> {
    reader
        .read_storage(CHAIN_CONFIG_ADDRESS, chain_config_slots::BLOCK_TIME)
        .map(|v| decode_u64(v))
}

/// Read the full signer list from SignerRegistry storage.
///
/// This is called by PoaConsensus at epoch blocks to update the authorized
/// signer list. Changes propagate on-chain without node restart.
pub fn read_signer_list(reader: &impl StorageReader) -> Option<DynamicSignerList> {
    let addr = SIGNER_REGISTRY_ADDRESS;

    let governance_val = reader.read_storage(addr, signer_registry_slots::GOVERNANCE)?;
    let length_val = reader.read_storage(addr, signer_registry_slots::SIGNERS_LENGTH)?;
    let threshold_val = reader.read_storage(addr, signer_registry_slots::SIGNER_THRESHOLD)?;

    let signer_count = decode_u64(length_val) as usize;
    let base_slot = dynamic_array_base_slot(signer_registry_slots::SIGNERS_LENGTH);

    let mut signers = Vec::with_capacity(signer_count);
    for i in 0..signer_count {
        let slot = base_slot + U256::from(i);
        if let Some(val) = reader.read_storage(addr, slot) {
            signers.push(decode_address(val));
        }
    }

    Some(DynamicSignerList {
        governance: decode_address(governance_val),
        signers,
        threshold: decode_u64(threshold_val),
    })
}

/// Check if a specific address is a signer via the on-chain mapping.
pub fn is_signer_on_chain(reader: &impl StorageReader, address: Address) -> bool {
    let slot_hash = mapping_address_bool_slot(address, signer_registry_slots::IS_SIGNER_MAPPING);
    let slot = U256::from_be_bytes(slot_hash.0);
    reader
        .read_storage(SIGNER_REGISTRY_ADDRESS, slot)
        .map(|val| decode_bool(val))
        .unwrap_or(false)
}

// =============================================================================
// GenesisStorageReader — reads from genesis alloc for testing
// =============================================================================

/// A StorageReader that reads from the genesis configuration's alloc.
///
/// This lets us verify that the on-chain readers produce the correct values
/// when reading the pre-populated genesis storage, without needing a running node.
pub struct GenesisStorageReader {
    /// The genesis configuration to read from
    alloc: std::collections::BTreeMap<Address, alloy_genesis::GenesisAccount>,
}

impl GenesisStorageReader {
    /// Create a reader from a genesis configuration.
    pub fn from_genesis(genesis: &alloy_genesis::Genesis) -> Self {
        Self { alloc: genesis.alloc.clone() }
    }
}

impl StorageReader for GenesisStorageReader {
    fn read_storage(&self, address: Address, slot: U256) -> Option<B256> {
        let account = self.alloc.get(&address)?;
        let storage = account.storage.as_ref()?;
        let slot_key = B256::from(slot.to_be_bytes());
        storage.get(&slot_key).copied()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genesis::{
        create_dev_genesis, create_genesis, dev_accounts, dev_signers,
        GenesisConfig, GOVERNANCE_SAFE_ADDRESS,
    };
    use std::collections::BTreeMap;

    // =========================================================================
    // Helper: In-memory storage reader for unit tests
    // =========================================================================

    struct MockStorage {
        storage: BTreeMap<(Address, U256), B256>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self { storage: BTreeMap::new() }
        }

        fn set(&mut self, address: Address, slot: U256, value: B256) {
            self.storage.insert((address, slot), value);
        }
    }

    impl StorageReader for MockStorage {
        fn read_storage(&self, address: Address, slot: U256) -> Option<B256> {
            self.storage.get(&(address, slot)).copied()
        }
    }

    // =========================================================================
    // Storage slot constant tests
    // =========================================================================

    #[test]
    fn test_chain_config_slot_values() {
        assert_eq!(chain_config_slots::GOVERNANCE, U256::ZERO);
        assert_eq!(chain_config_slots::GAS_LIMIT, U256::from(1));
        assert_eq!(chain_config_slots::BLOCK_TIME, U256::from(2));
        assert_eq!(chain_config_slots::MAX_CONTRACT_SIZE, U256::from(3));
        assert_eq!(chain_config_slots::CALLDATA_GAS_PER_BYTE, U256::from(4));
        assert_eq!(chain_config_slots::MAX_TX_GAS, U256::from(5));
        assert_eq!(chain_config_slots::EAGER_MINING, U256::from(6));
    }

    #[test]
    fn test_signer_registry_slot_values() {
        assert_eq!(signer_registry_slots::GOVERNANCE, U256::ZERO);
        assert_eq!(signer_registry_slots::SIGNERS_LENGTH, U256::from(1));
        assert_eq!(signer_registry_slots::IS_SIGNER_MAPPING, U256::from(2));
        assert_eq!(signer_registry_slots::SIGNER_THRESHOLD, U256::from(3));
    }

    // =========================================================================
    // ABI function selector tests
    // =========================================================================

    #[test]
    fn test_function_selector_computation() {
        // Known Solidity selectors (verified against solc output)
        // gasLimit() → 0xf68d4018
        let gas_limit_sel = function_selector("gasLimit()");
        assert_eq!(gas_limit_sel.len(), 4);
        // The selector should be deterministic
        assert_eq!(gas_limit_sel, function_selector("gasLimit()"));
    }

    #[test]
    fn test_different_functions_different_selectors() {
        let gas_limit = selectors::gas_limit();
        let block_time = selectors::block_time();
        let max_contract_size = selectors::max_contract_size();
        let governance = selectors::governance();
        let get_signers = selectors::get_signers();
        let signer_count = selectors::signer_count();

        // All selectors should be unique
        let all = vec![gas_limit, block_time, max_contract_size, governance, get_signers, signer_count];
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert_ne!(all[i], all[j], "Selectors at index {} and {} should differ", i, j);
            }
        }
    }

    #[test]
    fn test_selector_length_always_4_bytes() {
        assert_eq!(selectors::gas_limit().len(), 4);
        assert_eq!(selectors::block_time().len(), 4);
        assert_eq!(selectors::max_contract_size().len(), 4);
        assert_eq!(selectors::calldata_gas_per_byte().len(), 4);
        assert_eq!(selectors::max_tx_gas().len(), 4);
        assert_eq!(selectors::eager_mining().len(), 4);
        assert_eq!(selectors::governance().len(), 4);
        assert_eq!(selectors::get_signers().len(), 4);
        assert_eq!(selectors::signer_count().len(), 4);
        assert_eq!(selectors::signer_threshold().len(), 4);
        assert_eq!(selectors::is_signer().len(), 4);
    }

    // =========================================================================
    // Encoding/Decoding helpers
    // =========================================================================

    #[test]
    fn test_encode_decode_u64_roundtrip() {
        let values = [0u64, 1, 30_000_000, 60_000_000, 100_000_000, 1_000_000_000, u64::MAX];
        for val in values {
            let encoded = encode_u64(val);
            let decoded = decode_u64(encoded);
            assert_eq!(decoded, val, "Roundtrip failed for {}", val);
        }
    }

    #[test]
    fn test_encode_decode_address_roundtrip() {
        let addresses = [
            Address::ZERO,
            GOVERNANCE_SAFE_ADDRESS,
            CHAIN_CONFIG_ADDRESS,
            SIGNER_REGISTRY_ADDRESS,
            dev_accounts()[0],
            dev_accounts()[19],
        ];
        for addr in addresses {
            let encoded = encode_address(addr);
            let decoded = decode_address(encoded);
            assert_eq!(decoded, addr, "Roundtrip failed for {}", addr);
        }
    }

    #[test]
    fn test_decode_bool_true_and_false() {
        assert!(!decode_bool(B256::ZERO));
        assert!(decode_bool(B256::from(U256::from(1).to_be_bytes())));
        // Non-zero last byte is true
        let mut b = [0u8; 32];
        b[31] = 0xFF;
        assert!(decode_bool(B256::from(b)));
    }

    #[test]
    fn test_encode_address_is_left_padded() {
        let addr = dev_accounts()[0];
        let encoded = encode_address(addr);
        // First 12 bytes should be zero
        assert_eq!(&encoded[..12], &[0u8; 12]);
        // Last 20 bytes should be the address
        assert_eq!(&encoded[12..32], addr.as_slice());
    }

    // =========================================================================
    // Dynamic array base slot computation
    // =========================================================================

    #[test]
    fn test_dynamic_array_base_slot_matches_genesis() {
        // genesis.rs computes the same thing for SignerRegistry signers array:
        //   let mut hasher = Keccak256::new();
        //   hasher.update(B256::from(U256::from(1u64).to_be_bytes()).as_slice());
        //   let array_base = U256::from_be_bytes(hasher.finalize().0);
        //
        // Our function should produce the same result
        let our_base = dynamic_array_base_slot(U256::from(1));

        let mut hasher = Keccak256::new();
        hasher.update(B256::from(U256::from(1u64).to_be_bytes()).as_slice());
        let genesis_base = U256::from_be_bytes(hasher.finalize().0);

        assert_eq!(our_base, genesis_base, "Array base slot must match genesis.rs computation");
    }

    #[test]
    fn test_dynamic_array_base_slot_deterministic() {
        let base1 = dynamic_array_base_slot(U256::from(1));
        let base2 = dynamic_array_base_slot(U256::from(1));
        assert_eq!(base1, base2);
    }

    #[test]
    fn test_dynamic_array_different_slots_different_bases() {
        let base1 = dynamic_array_base_slot(U256::from(1));
        let base2 = dynamic_array_base_slot(U256::from(2));
        assert_ne!(base1, base2);
    }

    // =========================================================================
    // Mapping slot computation
    // =========================================================================

    #[test]
    fn test_mapping_slot_matches_genesis() {
        // genesis.rs computes isSigner[addr] mapping slots the same way:
        //   let mut hasher = Keccak256::new();
        //   let mut key_padded = [0u8; 32];
        //   key_padded[12..32].copy_from_slice(signer.as_slice());
        //   hasher.update(&key_padded);
        //   hasher.update(B256::from(U256::from(2u64).to_be_bytes()).as_slice());
        //   let mapping_slot = hasher.finalize();
        //
        // Our function should produce the same result
        let signer = dev_signers()[0];
        let our_slot = mapping_address_bool_slot(signer, U256::from(2));

        let mut hasher = Keccak256::new();
        let mut key_padded = [0u8; 32];
        key_padded[12..32].copy_from_slice(signer.as_slice());
        hasher.update(&key_padded);
        hasher.update(B256::from(U256::from(2u64).to_be_bytes()).as_slice());
        let genesis_slot = hasher.finalize();

        assert_eq!(our_slot, genesis_slot, "Mapping slot must match genesis.rs computation");
    }

    #[test]
    fn test_mapping_slot_different_addresses_different_slots() {
        let slot1 = mapping_address_bool_slot(dev_accounts()[0], U256::from(2));
        let slot2 = mapping_address_bool_slot(dev_accounts()[1], U256::from(2));
        assert_ne!(slot1, slot2);
    }

    #[test]
    fn test_mapping_slot_different_base_different_slots() {
        let slot1 = mapping_address_bool_slot(dev_accounts()[0], U256::from(2));
        let slot2 = mapping_address_bool_slot(dev_accounts()[0], U256::from(3));
        assert_ne!(slot1, slot2);
    }

    // =========================================================================
    // MockStorage reader tests
    // =========================================================================

    #[test]
    fn test_mock_storage_read_write() {
        let mut mock = MockStorage::new();
        let value = encode_u64(42);
        mock.set(CHAIN_CONFIG_ADDRESS, U256::from(1), value);
        assert_eq!(mock.read_storage(CHAIN_CONFIG_ADDRESS, U256::from(1)), Some(value));
    }

    #[test]
    fn test_mock_storage_missing_returns_none() {
        let mock = MockStorage::new();
        assert_eq!(mock.read_storage(CHAIN_CONFIG_ADDRESS, U256::from(999)), None);
    }

    #[test]
    fn test_read_chain_config_from_mock() {
        let mut mock = MockStorage::new();
        let gov = GOVERNANCE_SAFE_ADDRESS;

        mock.set(CHAIN_CONFIG_ADDRESS, chain_config_slots::GOVERNANCE, encode_address(gov));
        mock.set(CHAIN_CONFIG_ADDRESS, chain_config_slots::GAS_LIMIT, encode_u64(300_000_000));
        mock.set(CHAIN_CONFIG_ADDRESS, chain_config_slots::BLOCK_TIME, encode_u64(1));
        mock.set(CHAIN_CONFIG_ADDRESS, chain_config_slots::MAX_CONTRACT_SIZE, encode_u64(524_288));
        mock.set(CHAIN_CONFIG_ADDRESS, chain_config_slots::CALLDATA_GAS_PER_BYTE, encode_u64(4));
        mock.set(CHAIN_CONFIG_ADDRESS, chain_config_slots::MAX_TX_GAS, encode_u64(300_000_000));

        let config = read_chain_config(&mock).unwrap();
        assert_eq!(config.governance, gov);
        assert_eq!(config.gas_limit, 300_000_000);
        assert_eq!(config.block_time, 1);
        assert_eq!(config.max_contract_size, 524_288);
        assert_eq!(config.calldata_gas_per_byte, 4);
        assert_eq!(config.max_tx_gas, 300_000_000);
        assert!(!config.eager_mining);
    }

    #[test]
    fn test_read_chain_config_missing_returns_none() {
        let mock = MockStorage::new();
        assert!(read_chain_config(&mock).is_none());
    }

    #[test]
    fn test_read_gas_limit_from_mock() {
        let mut mock = MockStorage::new();
        mock.set(CHAIN_CONFIG_ADDRESS, chain_config_slots::GAS_LIMIT, encode_u64(1_000_000_000));
        assert_eq!(read_gas_limit(&mock), Some(1_000_000_000));
    }

    #[test]
    fn test_read_block_time_from_mock() {
        let mut mock = MockStorage::new();
        mock.set(CHAIN_CONFIG_ADDRESS, chain_config_slots::BLOCK_TIME, encode_u64(12));
        assert_eq!(read_block_time(&mock), Some(12));
    }

    #[test]
    fn test_read_signer_list_from_mock() {
        let mut mock = MockStorage::new();
        let gov = GOVERNANCE_SAFE_ADDRESS;
        let signers = dev_signers();

        mock.set(SIGNER_REGISTRY_ADDRESS, signer_registry_slots::GOVERNANCE, encode_address(gov));
        mock.set(SIGNER_REGISTRY_ADDRESS, signer_registry_slots::SIGNERS_LENGTH, encode_u64(3));
        mock.set(SIGNER_REGISTRY_ADDRESS, signer_registry_slots::SIGNER_THRESHOLD, encode_u64(2));

        // Set dynamic array entries
        let base = dynamic_array_base_slot(signer_registry_slots::SIGNERS_LENGTH);
        for (i, signer) in signers.iter().enumerate() {
            mock.set(SIGNER_REGISTRY_ADDRESS, base + U256::from(i), encode_address(*signer));
        }

        let list = read_signer_list(&mock).unwrap();
        assert_eq!(list.governance, gov);
        assert_eq!(list.signers.len(), 3);
        assert_eq!(list.signers, signers);
        assert_eq!(list.threshold, 2);
    }

    #[test]
    fn test_read_signer_list_empty() {
        let mut mock = MockStorage::new();
        mock.set(SIGNER_REGISTRY_ADDRESS, signer_registry_slots::GOVERNANCE, encode_address(GOVERNANCE_SAFE_ADDRESS));
        mock.set(SIGNER_REGISTRY_ADDRESS, signer_registry_slots::SIGNERS_LENGTH, encode_u64(0));
        mock.set(SIGNER_REGISTRY_ADDRESS, signer_registry_slots::SIGNER_THRESHOLD, encode_u64(0));

        let list = read_signer_list(&mock).unwrap();
        assert!(list.signers.is_empty());
    }

    #[test]
    fn test_is_signer_on_chain_mock() {
        let mut mock = MockStorage::new();
        let signer = dev_signers()[0];

        // Set isSigner[signer] = true
        let slot_hash = mapping_address_bool_slot(signer, signer_registry_slots::IS_SIGNER_MAPPING);
        let slot = U256::from_be_bytes(slot_hash.0);
        mock.set(SIGNER_REGISTRY_ADDRESS, slot, encode_u64(1));

        assert!(is_signer_on_chain(&mock, signer));
        // Unknown address should return false
        assert!(!is_signer_on_chain(&mock, dev_accounts()[5]));
    }

    // =========================================================================
    // GenesisStorageReader tests — THE critical integration tests
    // These verify that our reader correctly reads the values that genesis.rs
    // pre-populates in the contract storage.
    // =========================================================================

    #[test]
    fn test_genesis_reader_reads_chain_config() {
        let genesis = create_dev_genesis();
        let reader = GenesisStorageReader::from_genesis(&genesis);

        let config = read_chain_config(&reader).unwrap();

        assert_eq!(config.governance, GOVERNANCE_SAFE_ADDRESS);
        assert_eq!(config.gas_limit, 30_000_000);
        assert_eq!(config.block_time, 2);
        assert_eq!(config.max_contract_size, 24_576);
        assert_eq!(config.calldata_gas_per_byte, 16);
        assert_eq!(config.max_tx_gas, 30_000_000);
        assert!(!config.eager_mining);
    }

    #[test]
    fn test_genesis_reader_reads_production_chain_config() {
        let genesis = create_genesis(GenesisConfig::production());
        let reader = GenesisStorageReader::from_genesis(&genesis);

        let config = read_chain_config(&reader).unwrap();

        assert_eq!(config.governance, GOVERNANCE_SAFE_ADDRESS);
        assert_eq!(config.gas_limit, 60_000_000);
        assert_eq!(config.block_time, 12);
        assert_eq!(config.max_contract_size, 24_576);
        assert_eq!(config.calldata_gas_per_byte, 16);
        assert_eq!(config.max_tx_gas, 60_000_000);
    }

    #[test]
    fn test_genesis_reader_reads_signer_list() {
        let genesis = create_dev_genesis();
        let reader = GenesisStorageReader::from_genesis(&genesis);

        let list = read_signer_list(&reader).unwrap();

        assert_eq!(list.governance, GOVERNANCE_SAFE_ADDRESS);
        assert_eq!(list.signers.len(), 3);
        assert_eq!(list.signers, dev_signers());
        assert_eq!(list.threshold, 2); // 3/2 + 1 = 2
    }

    #[test]
    fn test_genesis_reader_reads_production_signer_list() {
        let genesis = create_genesis(GenesisConfig::production());
        let reader = GenesisStorageReader::from_genesis(&genesis);

        let list = read_signer_list(&reader).unwrap();

        assert_eq!(list.governance, GOVERNANCE_SAFE_ADDRESS);
        assert_eq!(list.signers.len(), 5);
        assert_eq!(list.signers, dev_accounts().into_iter().take(5).collect::<Vec<_>>());
        assert_eq!(list.threshold, 3); // 5/2 + 1 = 3
    }

    #[test]
    fn test_genesis_reader_gas_limit_shortcut() {
        let genesis = create_dev_genesis();
        let reader = GenesisStorageReader::from_genesis(&genesis);

        assert_eq!(read_gas_limit(&reader), Some(30_000_000));
    }

    #[test]
    fn test_genesis_reader_block_time_shortcut() {
        let genesis = create_dev_genesis();
        let reader = GenesisStorageReader::from_genesis(&genesis);

        assert_eq!(read_block_time(&reader), Some(2));
    }

    #[test]
    fn test_genesis_reader_is_signer_check() {
        let genesis = create_dev_genesis();
        let reader = GenesisStorageReader::from_genesis(&genesis);

        // All 3 dev signers should be registered
        for signer in dev_signers() {
            assert!(
                is_signer_on_chain(&reader, signer),
                "Signer {} should be registered on-chain",
                signer
            );
        }

        // Non-signers should NOT be registered
        for account in dev_accounts().into_iter().skip(3) {
            assert!(
                !is_signer_on_chain(&reader, account),
                "Account {} should NOT be registered as signer",
                account
            );
        }
    }

    #[test]
    fn test_genesis_reader_production_is_signer_check() {
        let genesis = create_genesis(GenesisConfig::production());
        let reader = GenesisStorageReader::from_genesis(&genesis);

        // First 5 accounts should be signers in production
        for account in dev_accounts().into_iter().take(5) {
            assert!(
                is_signer_on_chain(&reader, account),
                "Account {} should be a production signer",
                account
            );
        }

        // Account 5+ should NOT be signers
        for account in dev_accounts().into_iter().skip(5) {
            assert!(
                !is_signer_on_chain(&reader, account),
                "Account {} should NOT be a production signer",
                account
            );
        }
    }

    // =========================================================================
    // Custom genesis configurations
    // =========================================================================

    #[test]
    fn test_genesis_reader_custom_gas_limit() {
        let mut config = GenesisConfig::dev();
        config.gas_limit = 100_000_000; // 100M gas
        let genesis = create_genesis(config);
        let reader = GenesisStorageReader::from_genesis(&genesis);

        let chain_config = read_chain_config(&reader).unwrap();
        assert_eq!(chain_config.gas_limit, 100_000_000);
        assert_eq!(chain_config.max_tx_gas, 100_000_000); // max_tx_gas = gas_limit in genesis
    }

    #[test]
    fn test_genesis_reader_custom_block_time() {
        let mut config = GenesisConfig::dev();
        config.block_period = 1; // 1-second blocks
        let genesis = create_genesis(config);
        let reader = GenesisStorageReader::from_genesis(&genesis);

        let chain_config = read_chain_config(&reader).unwrap();
        assert_eq!(chain_config.block_time, 1);
    }

    #[test]
    fn test_genesis_reader_custom_signers() {
        let custom_signers: Vec<Address> = (1..=5u64)
            .map(|i| {
                let mut bytes = [0u8; 20];
                bytes[19] = i as u8;
                Address::from(bytes)
            })
            .collect();

        let config = GenesisConfig::default().with_signers(custom_signers.clone());
        let genesis = create_genesis(config);
        let reader = GenesisStorageReader::from_genesis(&genesis);

        let list = read_signer_list(&reader).unwrap();
        assert_eq!(list.signers.len(), 5);
        assert_eq!(list.signers, custom_signers);
        assert_eq!(list.threshold, 3); // 5/2 + 1 = 3

        // Verify isSigner mapping
        for signer in &custom_signers {
            assert!(is_signer_on_chain(&reader, *signer));
        }
    }

    #[test]
    fn test_genesis_reader_single_signer() {
        let signer = dev_accounts()[0];
        let config = GenesisConfig::default().with_signers(vec![signer]);
        let genesis = create_genesis(config);
        let reader = GenesisStorageReader::from_genesis(&genesis);

        let list = read_signer_list(&reader).unwrap();
        assert_eq!(list.signers.len(), 1);
        assert_eq!(list.signers[0], signer);
        assert_eq!(list.threshold, 1); // 1/2 + 1 = 1
        assert!(is_signer_on_chain(&reader, signer));
    }

    // =========================================================================
    // Consistency tests — values from reader match genesis config
    // =========================================================================

    #[test]
    fn test_chain_config_reader_matches_genesis_config() {
        let configs = vec![
            GenesisConfig::dev(),
            GenesisConfig::production(),
            GenesisConfig::default()
                .with_chain_id(42)
                .with_block_period(5)
                .with_signers(dev_signers()),
        ];

        for config in configs {
            let gas_limit = config.gas_limit;
            let block_time = config.block_period;
            let genesis = create_genesis(config);
            let reader = GenesisStorageReader::from_genesis(&genesis);

            let chain_config = read_chain_config(&reader).unwrap();
            assert_eq!(
                chain_config.gas_limit, gas_limit,
                "Gas limit mismatch: expected {}, got {}",
                gas_limit, chain_config.gas_limit
            );
            assert_eq!(
                chain_config.block_time, block_time,
                "Block time mismatch: expected {}, got {}",
                block_time, chain_config.block_time
            );
        }
    }

    #[test]
    fn test_signer_list_reader_matches_genesis_config() {
        let configs = vec![
            (GenesisConfig::dev(), 3usize),
            (GenesisConfig::production(), 5usize),
        ];

        for (config, expected_count) in configs {
            let signers = config.signers.clone();
            let genesis = create_genesis(config);
            let reader = GenesisStorageReader::from_genesis(&genesis);

            let list = read_signer_list(&reader).unwrap();
            assert_eq!(list.signers.len(), expected_count);
            assert_eq!(list.signers, signers);
        }
    }

    // =========================================================================
    // Edge cases
    // =========================================================================

    #[test]
    fn test_decode_u64_zero() {
        assert_eq!(decode_u64(B256::ZERO), 0);
    }

    #[test]
    fn test_decode_address_zero() {
        assert_eq!(decode_address(B256::ZERO), Address::ZERO);
    }

    #[test]
    fn test_dynamic_array_slot_0() {
        // Even if array_slot is 0, it should compute without panicking
        let base = dynamic_array_base_slot(U256::ZERO);
        assert_ne!(base, U256::ZERO);
    }

    #[test]
    fn test_genesis_reader_nonexistent_address() {
        let genesis = create_dev_genesis();
        let reader = GenesisStorageReader::from_genesis(&genesis);

        let fake_addr: Address = "0x0000000000000000000000000000000000000099".parse().unwrap();
        assert_eq!(reader.read_storage(fake_addr, U256::ZERO), None);
    }

    #[test]
    fn test_genesis_reader_nonexistent_slot() {
        let genesis = create_dev_genesis();
        let reader = GenesisStorageReader::from_genesis(&genesis);

        // Slot 999 doesn't exist in ChainConfig
        assert_eq!(reader.read_storage(CHAIN_CONFIG_ADDRESS, U256::from(999)), None);
    }

    // =========================================================================
    // DynamicChainConfig struct tests
    // =========================================================================

    #[test]
    fn test_dynamic_chain_config_equality() {
        let a = DynamicChainConfig {
            governance: GOVERNANCE_SAFE_ADDRESS,
            gas_limit: 30_000_000,
            block_time: 2,
            max_contract_size: 24_576,
            calldata_gas_per_byte: 16,
            max_tx_gas: 30_000_000,
            eager_mining: false,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn test_dynamic_chain_config_debug() {
        let config = DynamicChainConfig {
            governance: Address::ZERO,
            gas_limit: 0,
            block_time: 0,
            max_contract_size: 0,
            calldata_gas_per_byte: 0,
            max_tx_gas: 0,
            eager_mining: false,
        };
        let debug = format!("{:?}", config);
        assert!(debug.contains("DynamicChainConfig"));
    }

    #[test]
    fn test_dynamic_signer_list_equality() {
        let a = DynamicSignerList {
            governance: GOVERNANCE_SAFE_ADDRESS,
            signers: dev_signers(),
            threshold: 2,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    // =========================================================================
    // Integration: simulate a governance parameter change
    // =========================================================================

    #[test]
    fn test_simulate_gas_limit_change() {
        // Start with dev genesis
        let genesis = create_dev_genesis();
        let reader = GenesisStorageReader::from_genesis(&genesis);

        // Original gas limit should be 30M
        assert_eq!(read_gas_limit(&reader), Some(30_000_000));

        // Simulate governance calling ChainConfig.setGasLimit(300_000_000)
        // by creating a mock with the updated value
        let mut mock = MockStorage::new();

        // Copy all existing ChainConfig storage
        let chain_config_account = genesis.alloc.get(&CHAIN_CONFIG_ADDRESS).unwrap();
        if let Some(storage) = &chain_config_account.storage {
            for (key, value) in storage {
                let slot = U256::from_be_bytes(key.0);
                mock.set(CHAIN_CONFIG_ADDRESS, slot, *value);
            }
        }

        // Override gas limit: governance set it to 300M
        mock.set(CHAIN_CONFIG_ADDRESS, chain_config_slots::GAS_LIMIT, encode_u64(300_000_000));

        // Now the reader should see the updated value
        let config = read_chain_config(&mock).unwrap();
        assert_eq!(config.gas_limit, 300_000_000);
        // Other values should be unchanged
        assert_eq!(config.block_time, 2);
        assert_eq!(config.governance, GOVERNANCE_SAFE_ADDRESS);
    }

    #[test]
    fn test_simulate_signer_addition() {
        // Start with dev genesis (3 signers)
        let genesis = create_dev_genesis();
        let reader = GenesisStorageReader::from_genesis(&genesis);

        let list = read_signer_list(&reader).unwrap();
        assert_eq!(list.signers.len(), 3);

        // Simulate governance calling SignerRegistry.addSigner(new_signer)
        let new_signer = dev_accounts()[3]; // 4th account
        let mut mock = MockStorage::new();

        // Copy existing SignerRegistry storage
        let sr_account = genesis.alloc.get(&SIGNER_REGISTRY_ADDRESS).unwrap();
        if let Some(storage) = &sr_account.storage {
            for (key, value) in storage {
                let slot = U256::from_be_bytes(key.0);
                mock.set(SIGNER_REGISTRY_ADDRESS, slot, *value);
            }
        }

        // Update: signers.length = 4
        mock.set(SIGNER_REGISTRY_ADDRESS, signer_registry_slots::SIGNERS_LENGTH, encode_u64(4));

        // Add new signer to array at index 3
        let base = dynamic_array_base_slot(signer_registry_slots::SIGNERS_LENGTH);
        mock.set(SIGNER_REGISTRY_ADDRESS, base + U256::from(3), encode_address(new_signer));

        // Set isSigner[new_signer] = true
        let is_signer_slot = mapping_address_bool_slot(new_signer, signer_registry_slots::IS_SIGNER_MAPPING);
        mock.set(
            SIGNER_REGISTRY_ADDRESS,
            U256::from_be_bytes(is_signer_slot.0),
            encode_u64(1),
        );

        // Verify
        let updated_list = read_signer_list(&mock).unwrap();
        assert_eq!(updated_list.signers.len(), 4);
        assert_eq!(updated_list.signers[3], new_signer);
        assert!(is_signer_on_chain(&mock, new_signer));
    }

    #[test]
    fn test_simulate_block_time_change_to_1_second() {
        let genesis = create_dev_genesis();
        let reader = GenesisStorageReader::from_genesis(&genesis);

        // Original block time is 2s
        assert_eq!(read_block_time(&reader), Some(2));

        // Simulate: Governance Safe → ChainConfig.setBlockTime(1)
        let mut mock = MockStorage::new();
        let chain_config_account = genesis.alloc.get(&CHAIN_CONFIG_ADDRESS).unwrap();
        if let Some(storage) = &chain_config_account.storage {
            for (key, value) in storage {
                let slot = U256::from_be_bytes(key.0);
                mock.set(CHAIN_CONFIG_ADDRESS, slot, *value);
            }
        }
        mock.set(CHAIN_CONFIG_ADDRESS, chain_config_slots::BLOCK_TIME, encode_u64(1));

        assert_eq!(read_block_time(&mock), Some(1));
        // Gas limit unchanged
        assert_eq!(read_gas_limit(&mock), Some(30_000_000));
    }

    // =========================================================================
    // Large signer set test
    // =========================================================================

    #[test]
    fn test_genesis_reader_21_signers() {
        let signers: Vec<Address> = (1..=21u64)
            .map(|i| {
                let mut bytes = [0u8; 20];
                bytes[18] = (i >> 8) as u8;
                bytes[19] = i as u8;
                Address::from(bytes)
            })
            .collect();

        let config = GenesisConfig::default().with_signers(signers.clone());
        let genesis = create_genesis(config);
        let reader = GenesisStorageReader::from_genesis(&genesis);

        let list = read_signer_list(&reader).unwrap();
        assert_eq!(list.signers.len(), 21);
        assert_eq!(list.signers, signers);
        assert_eq!(list.threshold, 11); // 21/2 + 1 = 11

        // All should be registered as signers
        for signer in &signers {
            assert!(is_signer_on_chain(&reader, *signer));
        }
    }
}
