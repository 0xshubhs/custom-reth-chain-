use crate::chainspec::PoaChainSpec;
use crate::consensus::PoaConsensus;
use crate::output;
use reth_ethereum::EthPrimitives;
use reth_ethereum::node::builder::{
    components::ConsensusBuilder,
    node::{FullNodeTypes, NodeTypes},
    BuilderContext,
};
use std::sync::Arc;

/// Custom consensus builder that provides `PoaConsensus` instead of `EthBeaconConsensus`.
///
/// This is the key integration point: when the node builder constructs components,
/// it calls this builder to create the consensus engine. By providing `PoaConsensus`,
/// all block validation flows through our POA rules.
#[derive(Debug, Clone)]
pub struct PoaConsensusBuilder {
    /// The POA chain specification with signer list, epoch, period, etc.
    chain_spec: Arc<PoaChainSpec>,
    /// Whether to create consensus in dev mode (relaxed validation)
    pub dev_mode: bool,
}

impl PoaConsensusBuilder {
    /// Create a new consensus builder with the given POA chain spec.
    pub fn new(chain_spec: Arc<PoaChainSpec>) -> Self {
        Self { chain_spec, dev_mode: false }
    }

    /// Set dev mode on the consensus builder
    pub fn with_dev_mode(mut self, dev_mode: bool) -> Self {
        self.dev_mode = dev_mode;
        self
    }
}

impl<N> ConsensusBuilder<N> for PoaConsensusBuilder
where
    N: FullNodeTypes<Types: NodeTypes<Primitives = EthPrimitives>>,
{
    type Consensus = Arc<PoaConsensus>;

    async fn build_consensus(self, _ctx: &BuilderContext<N>) -> eyre::Result<Self::Consensus> {
        let mode = if self.dev_mode { "dev (relaxed)" } else { "production (strict)" };
        output::print_consensus_init(
            self.chain_spec.signers().len(),
            self.chain_spec.epoch(),
            self.chain_spec.block_period(),
            mode,
        );
        Ok(Arc::new(
            PoaConsensus::new(self.chain_spec).with_dev_mode(self.dev_mode),
        ))
    }
}
