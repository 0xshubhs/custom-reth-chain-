//! Custom POA Node Type
//!
//! Defines a `PoaNode` that replaces Ethereum's beacon consensus with `PoaConsensus`.
//! This is the core architectural change that makes the node actually use POA consensus
//! instead of being a vanilla Ethereum dev-mode node with unused POA code.

pub mod builder;
pub mod engine;

pub use builder::PoaConsensusBuilder;
pub use engine::{PoaEngineValidator, PoaEngineValidatorBuilder, strip_extra_data};

use crate::chainspec::PoaChainSpec;
use crate::payload::PoaPayloadBuilderBuilder;
use crate::signer::SignerManager;
use std::sync::Arc;

// Node builder types
use reth_ethereum::node::builder::{
    components::{BasicPayloadServiceBuilder, ComponentsBuilder},
    node::{FullNodeTypes, NodeTypes},
    DebugNode, Node, NodeAdapter,
};

// Node API types
use reth_ethereum::node::api::{FullNodeComponents, PayloadAttributesBuilder};

// Ethereum component builders (pool, network, executor, payload)
use reth_ethereum::node::{
    EthEngineTypes, EthereumAddOns, EthereumEthApiBuilder,
    EthereumExecutorBuilder, EthereumNetworkBuilder, EthereumPoolBuilder,
};

// Primitive and storage types
use reth_ethereum::{provider::EthStorage, EthPrimitives};

// Engine types for payload attributes
use reth_ethereum::engine::local::LocalPayloadAttributesBuilder;

// Payload types
use reth_payload_primitives::PayloadTypes;

// Chain spec
use reth_chainspec::ChainSpec;

// RPC add-ons
use reth_ethereum::node::builder::rpc::{
    BasicEngineApiBuilder, BasicEngineValidatorBuilder, Identity, RpcAddOns,
};

/// Custom POA Node type.
///
/// This replaces `EthereumNode` as the node type passed to the builder.
/// It uses the exact same primitives, storage, and engine types as Ethereum,
/// but provides `PoaConsensus` instead of `EthBeaconConsensus` for block validation,
/// and `PoaEngineValidator` to accept POA blocks with 97-byte extra_data.
#[derive(Debug, Clone)]
pub struct PoaNode {
    /// POA chain specification with signer config.
    chain_spec: Arc<PoaChainSpec>,
    /// Signer manager with signing keys for block production.
    signer_manager: Arc<SignerManager>,
    /// Whether the node runs in dev mode (relaxed consensus validation)
    dev_mode: bool,
}

impl PoaNode {
    /// Create a new PoaNode with the given chain specification.
    pub fn new(chain_spec: Arc<PoaChainSpec>) -> Self {
        Self {
            chain_spec,
            signer_manager: Arc::new(SignerManager::new()),
            dev_mode: false,
        }
    }

    /// Set dev mode on the node
    pub fn with_dev_mode(mut self, dev_mode: bool) -> Self {
        self.dev_mode = dev_mode;
        self
    }

    /// Set the signer manager for block production
    pub fn with_signer_manager(mut self, signer_manager: Arc<SignerManager>) -> Self {
        self.signer_manager = signer_manager;
        self
    }
}

// PoaNode uses the same type configuration as EthereumNode
impl NodeTypes for PoaNode {
    type Primitives = EthPrimitives;
    type ChainSpec = ChainSpec;
    type Storage = EthStorage;
    type Payload = EthEngineTypes;
}

// The Node implementation provides the ComponentsBuilder that wires everything together.
// The only difference from EthereumNode is the consensus builder and the engine validator.
impl<N> Node<N> for PoaNode
where
    N: FullNodeTypes<Types = Self>,
{
    type ComponentsBuilder = ComponentsBuilder<
        N,
        EthereumPoolBuilder,
        BasicPayloadServiceBuilder<PoaPayloadBuilderBuilder>,
        EthereumNetworkBuilder,
        EthereumExecutorBuilder,
        PoaConsensusBuilder,
    >;

    type AddOns = EthereumAddOns<
        NodeAdapter<N>,
        EthereumEthApiBuilder,
        PoaEngineValidatorBuilder,
        BasicEngineApiBuilder<PoaEngineValidatorBuilder>,
        BasicEngineValidatorBuilder<PoaEngineValidatorBuilder>,
        Identity,
    >;

    fn components_builder(&self) -> Self::ComponentsBuilder {
        ComponentsBuilder::default()
            .node_types::<N>()
            .pool(EthereumPoolBuilder::default())
            .executor(EthereumExecutorBuilder::default())
            .payload(BasicPayloadServiceBuilder::new(
                PoaPayloadBuilderBuilder::new(
                    self.chain_spec.clone(),
                    self.signer_manager.clone(),
                    self.dev_mode,
                ),
            ))
            .network(EthereumNetworkBuilder::default())
            .consensus(
                PoaConsensusBuilder::new(self.chain_spec.clone()).with_dev_mode(self.dev_mode),
            )
    }

    fn add_ons(&self) -> Self::AddOns {
        EthereumAddOns::new(RpcAddOns::new(
            EthereumEthApiBuilder::default(),
            PoaEngineValidatorBuilder,
            BasicEngineApiBuilder::<PoaEngineValidatorBuilder>::default(),
            BasicEngineValidatorBuilder::new(PoaEngineValidatorBuilder),
            Identity::default(),
        ))
    }
}

// DebugNode enables launch_with_debug_capabilities(), which properly sets up dev mining.
impl<N: FullNodeComponents<Types = Self>> DebugNode<N> for PoaNode {
    type RpcBlock = reth_ethereum::rpc::eth::primitives::Block;

    fn rpc_to_primitive_block(rpc_block: Self::RpcBlock) -> reth_ethereum::Block {
        rpc_block.into_consensus().convert_transactions()
    }

    fn local_payload_attributes_builder(
        chain_spec: &Self::ChainSpec,
    ) -> impl PayloadAttributesBuilder<<Self::Payload as PayloadTypes>::PayloadAttributes> {
        LocalPayloadAttributesBuilder::new(Arc::new(chain_spec.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poa_node_creation() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let node = PoaNode::new(chain.clone());
        assert_eq!(node.chain_spec.signers().len(), 3);
    }

    #[test]
    fn test_poa_node_with_dev_mode() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let node = PoaNode::new(chain).with_dev_mode(true);
        assert!(node.dev_mode);
    }

    #[test]
    fn test_poa_node_with_signer_manager() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let manager = Arc::new(SignerManager::new());
        let node = PoaNode::new(chain).with_signer_manager(manager.clone());
        // Verify the manager is set (compare Arc pointers)
        assert!(Arc::ptr_eq(&node.signer_manager, &manager));
    }

    #[test]
    fn test_poa_node_full_builder_chain() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let manager = Arc::new(SignerManager::new());
        let node = PoaNode::new(chain)
            .with_dev_mode(true)
            .with_signer_manager(manager.clone());
        assert!(node.dev_mode);
        assert!(Arc::ptr_eq(&node.signer_manager, &manager));
        assert_eq!(node.chain_spec.signers().len(), 3);
    }

    #[test]
    fn test_poa_consensus_builder_creation() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let builder = PoaConsensusBuilder::new(chain);
        assert!(!builder.dev_mode);
    }

    #[test]
    fn test_poa_consensus_builder_dev_mode() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let builder = PoaConsensusBuilder::new(chain).with_dev_mode(true);
        assert!(builder.dev_mode);
    }

    #[test]
    fn test_strip_extra_data_v1() {
        use alloy_rpc_types_engine::{ExecutionPayload, ExecutionPayloadV1};
        use alloy_primitives::{Bytes, B256, U256, Address, Bloom};

        let v1 = ExecutionPayloadV1 {
            parent_hash: B256::ZERO,
            fee_recipient: Address::ZERO,
            state_root: B256::ZERO,
            receipts_root: B256::ZERO,
            logs_bloom: Bloom::default(),
            prev_randao: B256::ZERO,
            block_number: 1,
            gas_limit: 30_000_000,
            gas_used: 0,
            timestamp: 0,
            extra_data: Bytes::from(vec![0u8; 97]),
            base_fee_per_gas: U256::from(1000000000u64),
            block_hash: B256::ZERO,
            transactions: vec![],
        };

        let payload = ExecutionPayload::V1(v1);
        let (stripped, orig) = strip_extra_data(payload);
        assert_eq!(orig.len(), 97);
        match stripped {
            ExecutionPayload::V1(v) => assert_eq!(v.extra_data.len(), 0),
            _ => panic!("expected V1"),
        }
    }

    #[test]
    fn test_poa_engine_validator_builder_is_default() {
        let _builder = PoaEngineValidatorBuilder;
        let _default = PoaEngineValidatorBuilder::default();
    }
}
