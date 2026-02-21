//! Custom EVM configuration for Meowchain (Phase 2).
//!
//! Provides [`PoaEvmFactory`] — a thin wrapper around Reth's [`EthEvmFactory`]
//! that applies POA-specific EVM overrides before creating each EVM instance:
//!
//! - **Max contract code size** (`limit_contract_code_size`): Ethereum mainnet
//!   caps deployed bytecode at 24,576 bytes (EIP-170).  POA chains can lift this
//!   to 128 KB–512 KB for richer on-chain logic.
//!
//! This module also exposes [`PoaExecutorBuilder`], the [`ExecutorBuilder`]
//! that wires [`PoaEvmFactory`] into the node's component builder.
//!
//! # Architecture
//! ```text
//!   PoaNode → PoaExecutorBuilder.build_evm()
//!              → EthEvmConfig::new_with_evm_factory(chain_spec, PoaEvmFactory)
//!                 → PoaEvmFactory::create_evm(db, env)
//!                    → set env.cfg_env.limit_contract_code_size
//!                    → EthEvmFactory::create_evm(db, env)
//! ```

use alloy_evm::{
    eth::{EthEvm, EthEvmContext, EthEvmFactory},
    precompiles::PrecompilesMap,
    revm::{
        context::BlockEnv,
        context_interface::result::{EVMError, HaltReason},
        inspector::NoOpInspector,
        primitives::hardfork::SpecId,
        Inspector,
    },
    Database, EvmEnv, EvmFactory,
};

use reth_chainspec::EthereumHardforks;
use reth_ethereum::node::EthEvmConfig;
use reth_ethereum::node::builder::{components::ExecutorBuilder, BuilderContext};
use reth_ethereum::node::api::{FullNodeTypes, NodeTypes};
use reth_ethereum::EthPrimitives;
use reth_ethereum_forks::Hardforks;
use alloy_evm::eth::spec::EthExecutorSpec;
use alloy_evm::revm::context::TxEnv;

/// POA-customised EVM factory.
///
/// Wraps [`EthEvmFactory`] and injects POA-specific `CfgEnv` overrides
/// (currently: `limit_contract_code_size`) before delegating EVM creation.
#[derive(Debug, Clone, Default)]
pub struct PoaEvmFactory {
    inner: EthEvmFactory,
    /// Optional override for the maximum deployed contract code size.
    ///
    /// `None` → use the Ethereum default (24,576 bytes, EIP-170).
    /// `Some(n)` → contracts larger than `n` bytes are rejected at deployment.
    pub max_contract_size: Option<usize>,
}

impl PoaEvmFactory {
    /// Create a factory with a custom contract size limit.
    pub fn new(max_contract_size: Option<usize>) -> Self {
        Self {
            inner: EthEvmFactory::default(),
            max_contract_size,
        }
    }

    /// Apply POA-specific overrides to an [`EvmEnv`] before EVM creation.
    fn patch_env(&self, mut env: EvmEnv) -> EvmEnv {
        if let Some(limit) = self.max_contract_size {
            env.cfg_env.limit_contract_code_size = Some(limit);
            // Also lift the initcode size limit (EIP-3860) proportionally.
            // EIP-3860 sets initcode_limit = 2 × code_limit.
            env.cfg_env.limit_contract_initcode_size = Some(limit * 2);
        }
        env
    }
}

impl EvmFactory for PoaEvmFactory {
    type Evm<DB: Database, I: Inspector<Self::Context<DB>>> = EthEvm<DB, I, PrecompilesMap>;
    type Context<DB: Database> = EthEvmContext<DB>;
    type Tx = TxEnv;
    type Error<DBError: core::error::Error + Send + Sync + 'static> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Spec = SpecId;
    type BlockEnv = BlockEnv;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(&self, db: DB, input: EvmEnv) -> Self::Evm<DB, NoOpInspector> {
        self.inner.create_evm(db, self.patch_env(input))
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>>>(
        &self,
        db: DB,
        input: EvmEnv,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        self.inner
            .create_evm_with_inspector(db, self.patch_env(input), inspector)
    }
}

/// Custom executor builder that uses [`PoaEvmFactory`] for EVM creation.
///
/// Plugged into [`PoaNode::components_builder`] instead of
/// [`EthereumExecutorBuilder`] when a non-default `max_contract_size` is set.
#[derive(Debug, Clone)]
pub struct PoaExecutorBuilder {
    /// Override for maximum deployed contract size.  `None` = Ethereum default.
    pub max_contract_size: Option<usize>,
}

impl PoaExecutorBuilder {
    /// Create a builder that overrides the deployed contract size limit.
    pub fn new(max_contract_size: Option<usize>) -> Self {
        Self { max_contract_size }
    }
}

impl<Types, Node> ExecutorBuilder<Node> for PoaExecutorBuilder
where
    Types: NodeTypes<
        ChainSpec: Hardforks + EthExecutorSpec + EthereumHardforks,
        Primitives = EthPrimitives,
    >,
    Node: FullNodeTypes<Types = Types>,
{
    type EVM = EthEvmConfig<Types::ChainSpec, PoaEvmFactory>;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        Ok(EthEvmConfig::new_with_evm_factory(
            ctx.chain_spec(),
            PoaEvmFactory::new(self.max_contract_size),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_evm::EvmEnv;

    fn make_env() -> EvmEnv {
        EvmEnv::default()
    }

    #[test]
    fn test_poa_evm_factory_no_override_keeps_default() {
        let factory = PoaEvmFactory::new(None);
        let env = make_env();
        let patched = factory.patch_env(env);
        // No override → limit stays None (revm uses EIP-170 default)
        assert!(patched.cfg_env.limit_contract_code_size.is_none());
    }

    #[test]
    fn test_poa_evm_factory_applies_code_size_limit() {
        let factory = PoaEvmFactory::new(Some(524_288)); // 512 KB
        let env = make_env();
        let patched = factory.patch_env(env);
        assert_eq!(patched.cfg_env.limit_contract_code_size, Some(524_288));
    }

    #[test]
    fn test_poa_evm_factory_sets_initcode_limit_double() {
        let factory = PoaEvmFactory::new(Some(131_072)); // 128 KB
        let env = make_env();
        let patched = factory.patch_env(env);
        // initcode limit = 2x code limit (EIP-3860 ratio)
        assert_eq!(
            patched.cfg_env.limit_contract_initcode_size,
            Some(131_072 * 2)
        );
    }

    #[test]
    fn test_poa_evm_factory_ethereum_default_is_24kb() {
        // Verify the Ethereum standard constant for reference
        use alloy_evm::revm::primitives::eip170::MAX_CODE_SIZE;
        assert_eq!(MAX_CODE_SIZE, 24_576);
    }

    #[test]
    fn test_poa_executor_builder_creation() {
        let builder = PoaExecutorBuilder::new(Some(524_288));
        assert_eq!(builder.max_contract_size, Some(524_288));
    }

    #[test]
    fn test_poa_executor_builder_no_override() {
        let builder = PoaExecutorBuilder::new(None);
        assert!(builder.max_contract_size.is_none());
    }

    #[test]
    fn test_poa_evm_factory_default_no_override() {
        let factory = PoaEvmFactory::default();
        assert!(factory.max_contract_size.is_none());
    }

    #[test]
    fn test_patch_env_does_not_change_other_fields() {
        let factory = PoaEvmFactory::new(Some(65_536));
        let env = EvmEnv::default();
        let chain_id_before = env.cfg_env.chain_id;
        let patched = factory.patch_env(env);
        // chain_id unchanged — we only touch contract size fields
        assert_eq!(patched.cfg_env.chain_id, chain_id_before);
        assert_eq!(patched.cfg_env.limit_contract_code_size, Some(65_536));
    }
}
