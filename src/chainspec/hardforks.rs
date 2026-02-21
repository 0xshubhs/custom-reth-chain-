use alloy_primitives::U256;
use reth_chainspec::{ChainHardforks, ForkCondition, Hardfork};
use reth_ethereum_forks::EthereumHardfork;

/// Creates hardforks configuration that matches Ethereum mainnet.
/// This ensures full smart contract compatibility.
pub fn mainnet_compatible_hardforks() -> ChainHardforks {
    // Enable all hardforks at genesis (block 0 / timestamp 0)
    // This gives you the latest Ethereum features immediately
    ChainHardforks::new(vec![
        // Block-based hardforks (all at block 0)
        (EthereumHardfork::Frontier.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Homestead.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Tangerine.boxed(), ForkCondition::Block(0)),
        (
            EthereumHardfork::SpuriousDragon.boxed(),
            ForkCondition::Block(0),
        ),
        (EthereumHardfork::Byzantium.boxed(), ForkCondition::Block(0)),
        (
            EthereumHardfork::Constantinople.boxed(),
            ForkCondition::Block(0),
        ),
        (
            EthereumHardfork::Petersburg.boxed(),
            ForkCondition::Block(0),
        ),
        (EthereumHardfork::Istanbul.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::Berlin.boxed(), ForkCondition::Block(0)),
        (EthereumHardfork::London.boxed(), ForkCondition::Block(0)),
        // The Merge - we use TTD of 0 since POA doesn't have proof of work
        (
            EthereumHardfork::Paris.boxed(),
            ForkCondition::TTD {
                activation_block_number: 0,
                fork_block: None,
                total_difficulty: U256::ZERO,
            },
        ),
        // Timestamp-based hardforks (all at timestamp 0)
        (
            EthereumHardfork::Shanghai.boxed(),
            ForkCondition::Timestamp(0),
        ),
        (
            EthereumHardfork::Cancun.boxed(),
            ForkCondition::Timestamp(0),
        ),
        (
            EthereumHardfork::Prague.boxed(),
            ForkCondition::Timestamp(0),
        ),
    ])
}
