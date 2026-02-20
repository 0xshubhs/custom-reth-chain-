use alloy_rpc_types_engine::{ExecutionData, ExecutionPayload, PayloadError};
use reth_ethereum::EthPrimitives;
use reth_ethereum::node::api::{EngineApiValidator, FullNodeComponents, PayloadValidator};
use reth_ethereum::node::api::{EngineTypes, AddOnsContext};
use reth_ethereum::node::builder::node::NodeTypes;
use reth_ethereum::node::builder::rpc::PayloadValidatorBuilder;
use reth_ethereum::node::EthereumEngineValidator;
use reth_ethereum_engine_primitives::EthPayloadAttributes;
use reth_payload_primitives::{
    EngineApiMessageVersion, EngineObjectValidationError, NewPayloadError, PayloadOrAttributes,
    PayloadTypes,
};
use reth_primitives_traits::SealedBlock;
use std::sync::Arc;

/// Strip `extra_data` from an [`ExecutionPayload`], returning `(stripped, original_extra_data)`.
///
/// POA blocks carry 97 bytes in extra_data (vanity + seal). Alloy's conversion
/// rejects any extra_data > 32 bytes, so we must strip it before conversion and
/// restore it after.
pub fn strip_extra_data(payload: ExecutionPayload) -> (ExecutionPayload, alloy_primitives::Bytes) {
    match payload {
        ExecutionPayload::V1(mut v1) => {
            let extra = std::mem::take(&mut v1.extra_data);
            (ExecutionPayload::V1(v1), extra)
        }
        ExecutionPayload::V2(mut v2) => {
            let extra = std::mem::take(&mut v2.payload_inner.extra_data);
            (ExecutionPayload::V2(v2), extra)
        }
        ExecutionPayload::V3(mut v3) => {
            let extra = std::mem::take(&mut v3.payload_inner.payload_inner.extra_data);
            (ExecutionPayload::V3(v3), extra)
        }
    }
}

/// Custom engine validator that allows POA blocks with extra_data > 32 bytes.
///
/// Wraps [`EthereumEngineValidator`] and overrides only [`PayloadValidator::convert_payload_to_block`]
/// to strip/restore POA extra_data around alloy's strict 32-byte check.
#[derive(Debug, Clone)]
pub struct PoaEngineValidator<ChainSpec = reth_chainspec::ChainSpec> {
    inner: EthereumEngineValidator<ChainSpec>,
}

impl<ChainSpec> PoaEngineValidator<ChainSpec> {
    /// Creates a new validator with the given chain spec.
    pub const fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self { inner: EthereumEngineValidator::new(chain_spec) }
    }
}

impl<ChainSpec, Types> PayloadValidator<Types> for PoaEngineValidator<ChainSpec>
where
    ChainSpec: reth_chainspec::EthChainSpec + reth_ethereum_forks::EthereumHardforks + 'static,
    Types: PayloadTypes<ExecutionData = ExecutionData>,
{
    type Block = reth_ethereum::Block;

    fn convert_payload_to_block(
        &self,
        payload: ExecutionData,
    ) -> Result<SealedBlock<Self::Block>, NewPayloadError> {
        let ExecutionData { payload, sidecar } = payload;
        let expected_hash = payload.block_hash();

        // Strip extra_data to bypass alloy's 32-byte MAXIMUM_EXTRA_DATA_SIZE check.
        // POA blocks use 97 bytes (65-byte vanity + 32-byte ECDSA seal).
        let (stripped, orig_extra) = strip_extra_data(payload);

        // Convert to block — succeeds now because extra_data is empty.
        let mut block: reth_ethereum::Block = stripped
            .try_into_block_with_sidecar(&sidecar)
            .map_err(|e| NewPayloadError::Other(e.into()))?;

        // Restore the original extra_data.
        block.header.extra_data = orig_extra;

        // Reseal: recompute the block hash with the restored extra_data.
        let sealed = SealedBlock::seal_slow(block);

        // Verify the hash matches what the engine sent us.
        if expected_hash != sealed.hash() {
            return Err(PayloadError::BlockHash {
                execution: sealed.hash(),
                consensus: expected_hash,
            }
            .into());
        }

        Ok(sealed)
    }
}

impl<ChainSpec, Types> EngineApiValidator<Types> for PoaEngineValidator<ChainSpec>
where
    ChainSpec: reth_chainspec::EthChainSpec + reth_ethereum_forks::EthereumHardforks + 'static,
    Types: PayloadTypes<PayloadAttributes = EthPayloadAttributes, ExecutionData = ExecutionData>,
{
    fn validate_version_specific_fields(
        &self,
        version: EngineApiMessageVersion,
        payload_or_attrs: PayloadOrAttributes<'_, Types::ExecutionData, EthPayloadAttributes>,
    ) -> Result<(), EngineObjectValidationError> {
        <EthereumEngineValidator<ChainSpec> as EngineApiValidator<Types>>::validate_version_specific_fields(
            &self.inner,
            version,
            payload_or_attrs,
        )
    }

    fn ensure_well_formed_attributes(
        &self,
        version: EngineApiMessageVersion,
        attributes: &EthPayloadAttributes,
    ) -> Result<(), EngineObjectValidationError> {
        <EthereumEngineValidator<ChainSpec> as EngineApiValidator<Types>>::ensure_well_formed_attributes(
            &self.inner,
            version,
            attributes,
        )
    }
}

/// Builder for [`PoaEngineValidator`].
#[derive(Debug, Default, Clone)]
pub struct PoaEngineValidatorBuilder;

impl<Node, Types> PayloadValidatorBuilder<Node> for PoaEngineValidatorBuilder
where
    Types: NodeTypes<
        ChainSpec: reth_chainspec::EthChainSpec
            + reth_ethereum_forks::EthereumHardforks
            + Clone
            + 'static,
        Payload: EngineTypes<ExecutionData = ExecutionData>
            + PayloadTypes<PayloadAttributes = EthPayloadAttributes>,
        Primitives = EthPrimitives,
    >,
    Node: FullNodeComponents<Types = Types>,
{
    type Validator = PoaEngineValidator<Types::ChainSpec>;

    async fn build(self, ctx: &AddOnsContext<'_, Node>) -> eyre::Result<Self::Validator> {
        Ok(PoaEngineValidator::new(ctx.config.chain.clone()))
    }
}
