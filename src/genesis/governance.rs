use alloy_genesis::GenesisAccount;
use alloy_primitives::{b256, Address, Bytes, B256, U256};
use std::collections::BTreeMap;

use super::accounts::dev_accounts;
use super::addresses::{
    CHAIN_CONFIG_ADDRESS, SIGNER_REGISTRY_ADDRESS, TIMELOCK_ADDRESS, TREASURY_ADDRESS,
};

/// Returns governance contract allocs for genesis.
///
/// Deploys ChainConfig, SignerRegistry, Treasury, and Timelock contracts with initial
/// storage slots pre-populated to match constructor arguments.
///
/// Storage layout reference (Solidity):
///   - slot 0: governance address
///   - subsequent slots: contract-specific state
pub(crate) fn governance_contract_alloc(
    governance: Address,
    signers: &[Address],
    gas_limit: u64,
    block_time: u64,
) -> BTreeMap<Address, GenesisAccount> {
    let mut contracts = BTreeMap::new();

    // --- ChainConfig ---
    // Storage layout:
    //   slot 0: governance
    //   slot 1: gasLimit
    //   slot 2: blockTime
    //   slot 3: maxContractSize
    //   slot 4: calldataGasPerByte
    //   slot 5: maxTxGas
    //   slot 6: eagerMining (bool)
    {
        let mut storage = BTreeMap::new();
        // slot 0: governance
        let mut gov_slot = [0u8; 32];
        gov_slot[12..32].copy_from_slice(governance.as_slice());
        storage.insert(B256::ZERO, B256::from(gov_slot));

        // slot 1: gasLimit
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000001"),
            B256::from(U256::from(gas_limit).to_be_bytes()),
        );
        // slot 2: blockTime
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000002"),
            B256::from(U256::from(block_time).to_be_bytes()),
        );
        // slot 3: maxContractSize = 24576
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000003"),
            B256::from(U256::from(24576u64).to_be_bytes()),
        );
        // slot 4: calldataGasPerByte = 16
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000004"),
            B256::from(U256::from(16u64).to_be_bytes()),
        );
        // slot 5: maxTxGas = gasLimit
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000005"),
            B256::from(U256::from(gas_limit).to_be_bytes()),
        );
        // slot 6: eagerMining = false (0)

        contracts.insert(
            CHAIN_CONFIG_ADDRESS,
            GenesisAccount {
                balance: U256::ZERO,
                nonce: Some(1),
                code: Some(Bytes::from_static(include_bytes!("../bytecodes/chain_config.bin"))),
                storage: Some(storage),
                private_key: None,
            },
        );
    }

    // --- SignerRegistry ---
    // Storage layout:
    //   slot 0: governance
    //   slot 1: signers.length (dynamic array)
    //   slot 2: isSigner mapping (mapping, individual slots)
    //   slot 3: signerThreshold
    //   keccak256(1): signers[0], signers[1], ... (dynamic array data)
    {
        use alloy_primitives::Keccak256;

        let mut storage = BTreeMap::new();
        // slot 0: governance
        let mut gov_slot = [0u8; 32];
        gov_slot[12..32].copy_from_slice(governance.as_slice());
        storage.insert(B256::ZERO, B256::from(gov_slot));

        // slot 1: signers.length
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000001"),
            B256::from(U256::from(signers.len()).to_be_bytes()),
        );

        // Dynamic array data: keccak256(slot_1) + index
        let mut hasher = Keccak256::new();
        hasher.update(B256::from(U256::from(1u64).to_be_bytes()).as_slice());
        let array_base = U256::from_be_bytes(hasher.finalize().0);

        for (i, signer) in signers.iter().enumerate() {
            let slot = array_base + U256::from(i);
            let mut addr_slot = [0u8; 32];
            addr_slot[12..32].copy_from_slice(signer.as_slice());
            storage.insert(B256::from(slot.to_be_bytes()), B256::from(addr_slot));
        }

        // slot 2: isSigner mapping — keccak256(address . slot_2)
        for signer in signers {
            let mut hasher = Keccak256::new();
            let mut key_padded = [0u8; 32];
            key_padded[12..32].copy_from_slice(signer.as_slice());
            hasher.update(&key_padded);
            hasher.update(B256::from(U256::from(2u64).to_be_bytes()).as_slice());
            let mapping_slot = hasher.finalize();
            storage.insert(mapping_slot, B256::from(U256::from(1u64).to_be_bytes()));
        }

        // slot 3: signerThreshold = (signers.len() / 2 + 1) for majority
        let threshold = signers.len() / 2 + 1;
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000003"),
            B256::from(U256::from(threshold).to_be_bytes()),
        );

        contracts.insert(
            SIGNER_REGISTRY_ADDRESS,
            GenesisAccount {
                balance: U256::ZERO,
                nonce: Some(1),
                code: Some(Bytes::from_static(include_bytes!("../bytecodes/signer_registry.bin"))),
                storage: Some(storage),
                private_key: None,
            },
        );
    }

    // --- Treasury ---
    // Storage layout:
    //   slot 0: governance
    //   slot 1: signerShare = 4000
    //   slot 2: devShare = 3000
    //   slot 3: communityShare = 2000
    //   slot 4: burnShare = 1000
    //   slot 5: devFund
    //   slot 6: communityFund
    //   slot 7: signerRegistry
    {
        let mut storage = BTreeMap::new();
        // slot 0: governance
        let mut gov_slot = [0u8; 32];
        gov_slot[12..32].copy_from_slice(governance.as_slice());
        storage.insert(B256::ZERO, B256::from(gov_slot));

        // slot 1: signerShare = 4000
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000001"),
            B256::from(U256::from(4000u64).to_be_bytes()),
        );
        // slot 2: devShare = 3000
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000002"),
            B256::from(U256::from(3000u64).to_be_bytes()),
        );
        // slot 3: communityShare = 2000
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000003"),
            B256::from(U256::from(2000u64).to_be_bytes()),
        );
        // slot 4: burnShare = 1000
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000004"),
            B256::from(U256::from(1000u64).to_be_bytes()),
        );
        // slot 5: devFund = dev_accounts()[5] (treasury account in production)
        let dev_fund = dev_accounts()[5];
        let mut dev_fund_slot = [0u8; 32];
        dev_fund_slot[12..32].copy_from_slice(dev_fund.as_slice());
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000005"),
            B256::from(dev_fund_slot),
        );
        // slot 6: communityFund = dev_accounts()[7] (community account)
        let community_fund = dev_accounts()[7];
        let mut community_fund_slot = [0u8; 32];
        community_fund_slot[12..32].copy_from_slice(community_fund.as_slice());
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000006"),
            B256::from(community_fund_slot),
        );
        // slot 7: signerRegistry
        let mut sr_slot = [0u8; 32];
        sr_slot[12..32].copy_from_slice(SIGNER_REGISTRY_ADDRESS.as_slice());
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000007"),
            B256::from(sr_slot),
        );

        contracts.insert(
            TREASURY_ADDRESS,
            GenesisAccount {
                balance: U256::ZERO,
                nonce: Some(1),
                code: Some(Bytes::from_static(include_bytes!("../bytecodes/treasury.bin"))),
                storage: Some(storage),
                private_key: None,
            },
        );
    }

    // --- Timelock ---
    // Delay-enforcing contract for sensitive governance operations.
    // Storage layout:
    //   slot 0: minDelay (uint256) = 86400 (24 hours)
    //   slot 1: proposer (address) = governance
    //   slot 2: executor (address) = governance
    //   slot 3: admin (address) = governance
    //   slot 4: paused (bool) = false
    //   slot 5: timestamps mapping (mapping, individual slots — empty at genesis)
    {
        let mut storage = BTreeMap::new();
        // slot 0: minDelay = 86400 seconds (24 hours)
        storage.insert(
            B256::ZERO,
            B256::from(U256::from(86400u64).to_be_bytes()),
        );
        // slot 1: proposer = governance
        let mut gov_slot = [0u8; 32];
        gov_slot[12..32].copy_from_slice(governance.as_slice());
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000001"),
            B256::from(gov_slot),
        );
        // slot 2: executor = governance
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000002"),
            B256::from(gov_slot),
        );
        // slot 3: admin = governance
        storage.insert(
            b256!("0000000000000000000000000000000000000000000000000000000000000003"),
            B256::from(gov_slot),
        );
        // slot 4: paused = false (0) — default, no need to set

        contracts.insert(
            TIMELOCK_ADDRESS,
            GenesisAccount {
                balance: U256::ZERO,
                nonce: Some(1),
                code: Some(Bytes::from_static(include_bytes!("../bytecodes/timelock.bin"))),
                storage: Some(storage),
                private_key: None,
            },
        );
    }

    contracts
}
