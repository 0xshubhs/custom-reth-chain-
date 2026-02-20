use alloy_primitives::{Address, B256, U256};
use super::StorageReader;

/// Wraps a Reth `StateProvider` reference to implement the `StorageReader` trait.
///
/// This is the production adapter that reads from the live MDBX database at runtime.
/// Used by `PoaPayloadBuilder` to read `ChainConfig` and `SignerRegistry` contracts
/// at each block, enabling live governance updates without node restart.
///
/// # Usage
/// ```ignore
/// let state = provider.latest()?;
/// let reader = StateProviderStorageReader(state.as_ref());
/// let gas_limit = read_gas_limit(&reader);
/// let signers   = read_signer_list(&reader);
/// ```
pub struct StateProviderStorageReader<'a>(pub &'a dyn reth_storage_api::StateProvider);

impl<'a> StorageReader for StateProviderStorageReader<'a> {
    fn read_storage(&self, address: Address, slot: U256) -> Option<B256> {
        // Convert U256 slot to B256 storage key (big-endian, Solidity layout)
        let key = B256::from(slot.to_be_bytes());
        self.0
            .storage(address, key)
            .ok()
            .flatten()
            .map(|v| B256::from(v.to_be_bytes()))
    }
}

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
