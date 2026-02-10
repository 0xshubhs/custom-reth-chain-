//! Custom POA Node Type
//!
//! Defines a `PoaNode` that replaces Ethereum's beacon consensus with `PoaConsensus`.
//! This is the core architectural change that makes the node actually use POA consensus
//! instead of being a vanilla Ethereum dev-mode node with unused POA code.

use crate::chainspec::PoaChainSpec;
use crate::consensus::PoaConsensus;
use std::sync::Arc;

// Node builder types
use reth_ethereum::node::builder::{
    components::{BasicPayloadServiceBuilder, ComponentsBuilder, ConsensusBuilder},
    node::{FullNodeTypes, NodeTypes},
    BuilderContext, DebugNode, Node, NodeAdapter,
};

// Node API types
use reth_ethereum::node::api::{FullNodeComponents, PayloadAttributesBuilder};

// Ethereum component builders (pool, network, executor, payload)
use reth_ethereum::node::{
    payload::EthereumPayloadBuilder, EthEngineTypes, EthereumAddOns, EthereumEngineValidatorBuilder,
    EthereumEthApiBuilder, EthereumExecutorBuilder, EthereumNetworkBuilder, EthereumPoolBuilder,
};

// Primitive and storage types
use reth_ethereum::{provider::EthStorage, EthPrimitives};

// Engine types for payload attributes
use reth_ethereum::engine::local::LocalPayloadAttributesBuilder;

// Payload types
use reth_payload_primitives::PayloadTypes;

// Chain spec
use reth_chainspec::ChainSpec;

/// Custom consensus builder that provides `PoaConsensus` instead of `EthBeaconConsensus`.
///
/// This is the key integration point: when the node builder constructs components,
/// it calls this builder to create the consensus engine. By providing `PoaConsensus`,
/// all block validation flows through our POA rules.
#[derive(Debug, Clone)]
pub struct PoaConsensusBuilder {
    /// The POA chain specification with signer list, epoch, period, etc.
    chain_spec: Arc<PoaChainSpec>,
}

impl PoaConsensusBuilder {
    /// Create a new consensus builder with the given POA chain spec.
    pub fn new(chain_spec: Arc<PoaChainSpec>) -> Self {
        Self { chain_spec }
    }
}

impl<N> ConsensusBuilder<N> for PoaConsensusBuilder
where
    N: FullNodeTypes<Types: NodeTypes<Primitives = EthPrimitives>>,
{
    type Consensus = Arc<PoaConsensus>;

    async fn build_consensus(self, _ctx: &BuilderContext<N>) -> eyre::Result<Self::Consensus> {
        println!(
            "POA Consensus initialized with {} signers, epoch: {}, period: {}s",
            self.chain_spec.signers().len(),
            self.chain_spec.epoch(),
            self.chain_spec.block_period()
        );
        Ok(Arc::new(PoaConsensus::new(self.chain_spec)))
    }
}

/// Custom POA Node type.
///
/// This replaces `EthereumNode` as the node type passed to the builder.
/// It uses the exact same primitives, storage, and engine types as Ethereum,
/// but provides `PoaConsensus` instead of `EthBeaconConsensus` for block validation.
///
/// The architecture is:
/// ```text
/// PoaNode
///   ├── Primitives: EthPrimitives (identical to mainnet)
///   ├── ChainSpec: ChainSpec (standard Reth chain spec)
///   ├── Storage: EthStorage (standard MDBX storage)
///   ├── Payload: EthEngineTypes (standard engine API)
///   └── Components:
///       ├── Pool: EthereumPoolBuilder (standard tx pool)
///       ├── Network: EthereumNetworkBuilder (standard P2P)
///       ├── Executor: EthereumExecutorBuilder (standard EVM)
///       ├── Payload: EthereumPayloadBuilder (standard block building)
///       └── Consensus: PoaConsensusBuilder ← OUR CUSTOM CONSENSUS
/// ```
#[derive(Debug, Clone)]
pub struct PoaNode {
    /// POA chain specification with signer config.
    /// Stored here so it can be passed to the consensus builder.
    chain_spec: Arc<PoaChainSpec>,
}

impl PoaNode {
    /// Create a new PoaNode with the given chain specification.
    pub fn new(chain_spec: Arc<PoaChainSpec>) -> Self {
        Self { chain_spec }
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
// The only difference from EthereumNode is the consensus builder.
impl<N> Node<N> for PoaNode
where
    N: FullNodeTypes<Types = Self>,
{
    type ComponentsBuilder = ComponentsBuilder<
        N,
        EthereumPoolBuilder,
        BasicPayloadServiceBuilder<EthereumPayloadBuilder>,
        EthereumNetworkBuilder,
        EthereumExecutorBuilder,
        PoaConsensusBuilder,
    >;

    type AddOns =
        EthereumAddOns<NodeAdapter<N>, EthereumEthApiBuilder, EthereumEngineValidatorBuilder>;

    fn components_builder(&self) -> Self::ComponentsBuilder {
        ComponentsBuilder::default()
            .node_types::<N>()
            .pool(EthereumPoolBuilder::default())
            .executor(EthereumExecutorBuilder::default())
            .payload(BasicPayloadServiceBuilder::default())
            .network(EthereumNetworkBuilder::default())
            .consensus(PoaConsensusBuilder::new(self.chain_spec.clone()))
    }

    fn add_ons(&self) -> Self::AddOns {
        EthereumAddOns::default()
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
}
