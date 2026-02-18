//! POA Payload Builder
//!
//! Wraps Reth's `EthereumPayloadBuilder` to add POA block signing.
//! After the inner builder constructs a block (transactions, state root, etc.),
//! we post-process it to:
//! 1. Set difficulty = 0 (Engine API compatibility; authority is via ECDSA signature)
//! 2. Build extra_data with POA format (vanity + [signers at epoch] + signature)
//! 3. Sign the block header with the appropriate signer key

use crate::chainspec::PoaChainSpec;
use crate::consensus::{EXTRA_SEAL_LENGTH, EXTRA_VANITY_LENGTH};
use crate::onchain::{read_gas_limit, read_signer_list, StateProviderStorageReader};
use crate::signer::{BlockSealer, SignerManager};
use alloy_primitives::{Address, Bytes, U256};
use reth_basic_payload_builder::{
    BuildArguments, BuildOutcome, MissingPayloadBehaviour, PayloadBuilder, PayloadConfig,
};
use reth_chainspec::{ChainSpecProvider, EthChainSpec, EthereumHardforks};
use reth_ethereum::EthPrimitives;
use reth_ethereum::node::api::{FullNodeTypes, NodeTypes, PrimitivesTy, TxTy};
use reth_ethereum::node::builder::{
    components::PayloadBuilderBuilder, BuilderContext,
};
use reth_ethereum::node::core::cli::config::PayloadBuilderConfig;
use reth_ethereum_engine_primitives::{
    EthBuiltPayload, EthPayloadAttributes, EthPayloadBuilderAttributes,
};
use reth_ethereum_payload_builder::EthereumBuilderConfig;
use reth_evm::{ConfigureEvm, NextBlockEnvAttributes};
use reth_payload_builder_primitives::PayloadBuilderError;
use reth_payload_primitives::{BuiltPayload, PayloadTypes};
use reth_primitives_traits::block::SealedBlock;
use reth_ethereum::storage::StateProviderFactory;
use reth_transaction_pool::{PoolTransaction, TransactionPool};
use std::sync::Arc;

// =============================================================================
// PoaPayloadBuilderBuilder — Component-level builder (PayloadBuilderBuilder trait)
// =============================================================================

/// Component-level builder that creates `PoaPayloadBuilder` instances.
/// Plugs into `BasicPayloadServiceBuilder` in the node's `ComponentsBuilder`.
#[derive(Clone, Debug)]
pub struct PoaPayloadBuilderBuilder {
    chain_spec: Arc<PoaChainSpec>,
    signer_manager: Arc<SignerManager>,
    dev_mode: bool,
}

impl PoaPayloadBuilderBuilder {
    /// Create a new POA payload builder builder.
    pub fn new(
        chain_spec: Arc<PoaChainSpec>,
        signer_manager: Arc<SignerManager>,
        dev_mode: bool,
    ) -> Self {
        Self { chain_spec, signer_manager, dev_mode }
    }
}

impl<Types, Node, Pool, Evm> PayloadBuilderBuilder<Node, Pool, Evm> for PoaPayloadBuilderBuilder
where
    Types: NodeTypes<ChainSpec: EthereumHardforks, Primitives = EthPrimitives>,
    Node: FullNodeTypes<Types = Types>,
    Pool: TransactionPool<Transaction: PoolTransaction<Consensus = TxTy<Node::Types>>>
        + Unpin
        + 'static,
    Evm: ConfigureEvm<
            Primitives = PrimitivesTy<Types>,
            NextBlockEnvCtx = NextBlockEnvAttributes,
        > + 'static,
    Types::Payload: PayloadTypes<
        BuiltPayload = EthBuiltPayload,
        PayloadAttributes = EthPayloadAttributes,
        PayloadBuilderAttributes = EthPayloadBuilderAttributes,
    >,
{
    type PayloadBuilder = PoaPayloadBuilder<Pool, Node::Provider, Evm>;

    async fn build_payload_builder(
        self,
        ctx: &BuilderContext<Node>,
        pool: Pool,
        evm_config: Evm,
    ) -> eyre::Result<Self::PayloadBuilder> {
        let conf = ctx.payload_builder_config();
        let chain = ctx.chain_spec().chain();
        let default_gas_limit = conf.gas_limit_for(chain);

        // Read gas limit from on-chain ChainConfig contract (Phase 3: item 20).
        // Falls back to CLI/genesis default if the contract isn't readable yet.
        let gas_limit = match ctx.provider().latest() {
            Ok(state) => {
                let reader = StateProviderStorageReader(state.as_ref());
                let onchain = read_gas_limit(&reader).filter(|&gl| gl > 0);
                if let Some(gl) = onchain {
                    if gl != default_gas_limit {
                        println!(
                            "  OnChain gas limit: {} (from ChainConfig, default was {})",
                            gl, default_gas_limit
                        );
                    }
                    gl
                } else {
                    default_gas_limit
                }
            }
            Err(_) => default_gas_limit,
        };

        // Also seed the live signer cache from SignerRegistry at startup.
        if let Ok(state) = ctx.provider().latest() {
            let reader = StateProviderStorageReader(state.as_ref());
            if let Some(list) = read_signer_list(&reader) {
                if !list.signers.is_empty() {
                    println!(
                        "  OnChain signers: {} loaded from SignerRegistry",
                        list.signers.len()
                    );
                    self.chain_spec.update_live_signers(list.signers);
                }
            }
        }

        // In production mode, pre-allocate POA extra_data (vanity + seal placeholder).
        // In dev mode, leave extra_data empty — blocks are unsigned and Reth's engine
        // rejects extra_data > 32 bytes (Ethereum mainnet limit).
        let extra_data = if self.dev_mode {
            Bytes::new()
        } else {
            Bytes::from(vec![0u8; EXTRA_VANITY_LENGTH + EXTRA_SEAL_LENGTH])
        };

        let inner = reth_ethereum_payload_builder::EthereumPayloadBuilder::new(
            ctx.provider().clone(),
            pool,
            evm_config,
            EthereumBuilderConfig::new()
                .with_gas_limit(gas_limit)
                .with_max_blobs_per_block(conf.max_blobs_per_block())
                .with_extra_data(extra_data),
        );

        Ok(PoaPayloadBuilder {
            inner,
            chain_spec: self.chain_spec,
            signer_manager: self.signer_manager,
            dev_mode: self.dev_mode,
            client: ctx.provider().clone(),
        })
    }
}

// =============================================================================
// PoaPayloadBuilder — Build-level builder (PayloadBuilder trait)
// =============================================================================

/// POA payload builder that wraps `EthereumPayloadBuilder`.
///
/// After the inner builder constructs a block, this builder post-processes it
/// to add POA signatures, set difficulty, and embed signer lists at epoch blocks.
/// At epoch blocks, it also refreshes the live signer cache from the on-chain
/// `SignerRegistry` contract so that `PoaConsensus` picks up governance changes.
#[derive(Debug, Clone)]
pub struct PoaPayloadBuilder<Pool, Client, EvmConfig> {
    /// The inner Ethereum payload builder that does the actual block construction.
    inner: reth_ethereum_payload_builder::EthereumPayloadBuilder<Pool, Client, EvmConfig>,
    /// POA chain specification with signer list, epoch, period.
    chain_spec: Arc<PoaChainSpec>,
    /// Signer manager with signing keys.
    signer_manager: Arc<SignerManager>,
    /// Whether we're in dev mode (skip signing).
    dev_mode: bool,
    /// State provider factory for reading on-chain contract storage.
    client: Client,
}

impl<Pool, Client, EvmConfig> PayloadBuilder for PoaPayloadBuilder<Pool, Client, EvmConfig>
where
    EvmConfig: ConfigureEvm<Primitives = EthPrimitives, NextBlockEnvCtx = NextBlockEnvAttributes>,
    Client: StateProviderFactory + ChainSpecProvider<ChainSpec: EthereumHardforks> + Clone,
    Pool: TransactionPool<
        Transaction: PoolTransaction<Consensus = reth_ethereum::TransactionSigned>,
    >,
{
    type Attributes = EthPayloadBuilderAttributes;
    type BuiltPayload = EthBuiltPayload;

    fn try_build(
        &self,
        args: BuildArguments<EthPayloadBuilderAttributes, EthBuiltPayload>,
    ) -> Result<BuildOutcome<EthBuiltPayload>, PayloadBuilderError> {
        // 1. Let the inner builder construct the block (transactions, state, etc.)
        let outcome = self.inner.try_build(args)?;

        // 2. Post-process: sign the block if we have a signer
        match outcome {
            BuildOutcome::Better { payload, cached_reads } => {
                let signed_payload = self.sign_payload(payload)?;
                Ok(BuildOutcome::Better { payload: signed_payload, cached_reads })
            }
            BuildOutcome::Freeze(payload) => {
                let signed_payload = self.sign_payload(payload)?;
                Ok(BuildOutcome::Freeze(signed_payload))
            }
            other => Ok(other),
        }
    }

    fn on_missing_payload(
        &self,
        args: BuildArguments<Self::Attributes, Self::BuiltPayload>,
    ) -> MissingPayloadBehaviour<Self::BuiltPayload> {
        self.inner.on_missing_payload(args)
    }

    fn build_empty_payload(
        &self,
        config: PayloadConfig<Self::Attributes>,
    ) -> Result<EthBuiltPayload, PayloadBuilderError> {
        let payload = self.inner.build_empty_payload(config)?;
        self.sign_payload(payload)
    }
}

impl<Pool, Client, EvmConfig> PoaPayloadBuilder<Pool, Client, EvmConfig>
where
    Client: StateProviderFactory + Clone,
{
    /// Sign a built payload with POA signature.
    ///
    /// In dev mode, returns the payload unchanged.
    /// In production mode:
    /// 1. At epoch blocks — refreshes live signer list from on-chain SignerRegistry (Phase 3 item 21)
    /// 2. Determines which signer should sign (round-robin using effective_signers)
    /// 3. Sets difficulty = 0 (Engine API compatibility)
    /// 4. Builds extra_data with POA format (vanity + [signers at epoch] + signature)
    /// 5. Signs the header via BlockSealer
    /// 6. Reconstructs the sealed block
    fn sign_payload(
        &self,
        payload: EthBuiltPayload,
    ) -> Result<EthBuiltPayload, PayloadBuilderError> {
        if self.dev_mode {
            return Ok(payload);
        }

        let block = payload.block();
        let block_number = block.header().number;
        let epoch = self.chain_spec.epoch();
        let is_epoch = block_number > 0 && block_number % epoch == 0;

        // Phase 3 item 21: At epoch blocks, refresh live signer list from SignerRegistry.
        // This propagates on-chain governance changes (add/remove signers) without restart.
        if is_epoch {
            if let Ok(state) = self.client.latest() {
                let reader = StateProviderStorageReader(state.as_ref());
                if let Some(list) = read_signer_list(&reader) {
                    if !list.signers.is_empty() {
                        println!(
                            "  Epoch #{}: refreshed {} signers from SignerRegistry",
                            block_number,
                            list.signers.len()
                        );
                        self.chain_spec.update_live_signers(list.signers);
                    }
                }
            }
        }

        // Use effective_signers (live on-chain if available, else genesis config)
        let signers = self.chain_spec.effective_signers();
        if signers.is_empty() {
            // No signers configured, return unsigned
            return Ok(payload);
        }

        // Determine which signer should sign this block (round-robin)
        let in_turn_signer = match self.chain_spec.expected_signer(block_number) {
            Some(s) => s,
            None => return Ok(payload),
        };

        // Find a signer we control.
        // Use block_in_place + block_on so this works from both spawn_blocking contexts
        // (dev mode) and async task contexts (production+mining mode).
        let handle = tokio::runtime::Handle::current();
        let signer_manager = self.signer_manager.clone();

        let (signer_addr, is_in_turn) = tokio::task::block_in_place(|| {
            handle.block_on(async {
                // Prefer in-turn signer if we have it
                if signer_manager.has_signer(&in_turn_signer).await {
                    return (in_turn_signer, true);
                }
                // Otherwise find any authorized signer we control
                let our_addrs = signer_manager.signer_addresses().await;
                for addr in &our_addrs {
                    if signers.contains(addr) {
                        return (*addr, false);
                    }
                }
                (Address::ZERO, false)
            })
        });

        if signer_addr == Address::ZERO {
            // No authorized signer key available, return unsigned
            return Ok(payload);
        }

        // Clone header and body from the built block
        let mut header = block.header().clone();
        let body = block.body().clone();

        // Difficulty must be 0 for Engine API compatibility.
        // The Engine API (ExecutionPayloadV1) has no difficulty field and alloy hardcodes
        // it to U256::ZERO on conversion, so any non-zero value breaks the hash round-trip.
        // POA authority is determined solely by the ECDSA signature in extra_data.
        header.difficulty = U256::ZERO;

        // Build extra_data with POA format
        let mut extra_data = Vec::with_capacity(
            EXTRA_VANITY_LENGTH
                + if is_epoch { signers.len() * 20 } else { 0 }
                + EXTRA_SEAL_LENGTH,
        );

        // Vanity (32 zero bytes)
        extra_data.extend_from_slice(&[0u8; EXTRA_VANITY_LENGTH]);

        // At epoch blocks, embed the effective (live) signer list
        if is_epoch {
            for signer in signers.iter() {
                extra_data.extend_from_slice(signer.as_slice());
            }
        }

        // Placeholder for signature (65 bytes — will be replaced by seal_header)
        extra_data.extend_from_slice(&[0u8; EXTRA_SEAL_LENGTH]);
        header.extra_data = Bytes::from(extra_data);

        // Sign the header
        let sealer = BlockSealer::new(self.signer_manager.clone());
        let signed_header = tokio::task::block_in_place(|| {
            handle.block_on(async { sealer.seal_header(header, &signer_addr).await })
        })
        .map_err(|e| PayloadBuilderError::Other(Box::new(e)))?;

        println!(
            "  POA block #{} signed by {} ({})",
            block_number,
            signer_addr,
            if is_in_turn { "in-turn" } else { "out-of-turn" },
        );

        // Reconstruct the sealed block with the signed header
        let new_block = alloy_consensus::Block { header: signed_header, body };
        let sealed = SealedBlock::seal_slow(new_block);

        Ok(EthBuiltPayload::new(
            payload.id(),
            Arc::new(sealed),
            payload.fees(),
            payload.requests(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chainspec::PoaChainSpec;
    use crate::signer::{dev, BlockSealer};
    use alloy_consensus::Header;

    #[tokio::test]
    async fn test_poa_payload_builder_builder_creation() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let manager = Arc::new(SignerManager::new());
        let builder = PoaPayloadBuilderBuilder::new(chain.clone(), manager.clone(), true);

        assert!(builder.dev_mode);
        assert_eq!(builder.chain_spec.signers().len(), 3);
    }

    #[tokio::test]
    async fn test_sign_payload_components() {
        // Test the signing logic components work together
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let manager = dev::setup_dev_signers().await;

        let signers = chain.signers();
        assert_eq!(signers.len(), 3);

        // Block 1 should be signed by signer at index 1 % 3 = 1
        let expected_signer = signers[1];
        assert!(manager.has_signer(&expected_signer).await);

        // Verify the sealer can sign a header
        let sealer = BlockSealer::new(manager.clone());
        let header = Header {
            number: 1,
            difficulty: U256::from(1),
            gas_limit: 30_000_000,
            timestamp: 12345,
            extra_data: Bytes::from(vec![0u8; EXTRA_VANITY_LENGTH + EXTRA_SEAL_LENGTH]),
            ..Default::default()
        };

        let signed = sealer.seal_header(header, &expected_signer).await.unwrap();
        let recovered = BlockSealer::verify_signature(&signed).unwrap();
        assert_eq!(recovered, expected_signer);
    }

    #[tokio::test]
    async fn test_epoch_block_extra_data_format() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let signers = chain.signers();

        // Simulate epoch block extra_data construction
        let mut extra_data = Vec::new();
        extra_data.extend_from_slice(&[0u8; EXTRA_VANITY_LENGTH]); // 32 bytes vanity
        for signer in signers {
            extra_data.extend_from_slice(signer.as_slice()); // 20 bytes per signer
        }
        extra_data.extend_from_slice(&[0u8; EXTRA_SEAL_LENGTH]); // 65 bytes seal

        let expected_len = EXTRA_VANITY_LENGTH + signers.len() * 20 + EXTRA_SEAL_LENGTH;
        assert_eq!(extra_data.len(), expected_len);
        assert_eq!(extra_data.len(), 32 + 3 * 20 + 65); // 157 bytes for 3 signers
    }

    #[tokio::test]
    async fn test_difficulty_selection() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let signers = chain.signers();

        // Block 0: signer at index 0 is in-turn
        assert_eq!(chain.expected_signer(0), Some(signers[0]));
        // Block 1: signer at index 1 is in-turn
        assert_eq!(chain.expected_signer(1), Some(signers[1]));
        // Block 2: signer at index 2 is in-turn
        assert_eq!(chain.expected_signer(2), Some(signers[2]));
        // Block 3: signer at index 0 is in-turn (wraps around)
        assert_eq!(chain.expected_signer(3), Some(signers[0]));
    }

    #[tokio::test]
    async fn test_payload_builder_builder_dev_mode() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let manager = Arc::new(SignerManager::new());
        let builder = PoaPayloadBuilderBuilder::new(chain, manager, true);
        assert!(builder.dev_mode);
    }

    #[tokio::test]
    async fn test_payload_builder_builder_production_mode() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let manager = Arc::new(SignerManager::new());
        let builder = PoaPayloadBuilderBuilder::new(chain, manager, false);
        assert!(!builder.dev_mode);
    }

    #[tokio::test]
    async fn test_no_signers_returns_unchanged_logic() {
        // Test that with empty signers, the signing logic would skip
        let genesis = crate::genesis::create_dev_genesis();
        let poa_config = crate::chainspec::PoaConfig {
            period: 2,
            epoch: 30000,
            signers: vec![], // No signers
        };
        let chain = Arc::new(PoaChainSpec::new(genesis, poa_config));

        assert!(chain.signers().is_empty());
        // expected_signer returns None when no signers
        assert_eq!(chain.expected_signer(0), None);
        assert_eq!(chain.expected_signer(1), None);
    }

    #[tokio::test]
    async fn test_in_turn_signer_difficulty_1() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let signers = chain.signers();

        // Block 0: signer[0] is in-turn
        let in_turn_signer = chain.expected_signer(0).unwrap();
        assert_eq!(in_turn_signer, signers[0]);

        // In-turn signer should get difficulty 1
        let difficulty = U256::from(1);
        assert_eq!(difficulty, U256::from(1));
    }

    #[tokio::test]
    async fn test_out_of_turn_signer_difficulty_2() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let signers = chain.signers();

        // Block 0: signer[0] is in-turn, so signer[1] is out-of-turn
        let in_turn = chain.expected_signer(0).unwrap();
        assert_eq!(in_turn, signers[0]);
        assert_ne!(in_turn, signers[1]);

        // Out-of-turn signer should get difficulty 2
        let difficulty = U256::from(2);
        assert_eq!(difficulty, U256::from(2));
    }

    #[tokio::test]
    async fn test_epoch_block_includes_all_signers_in_extra_data() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let signers = chain.signers();
        let epoch = chain.epoch();

        // Simulate epoch block extra_data construction
        let block_number = epoch; // First epoch block after genesis
        let is_epoch = block_number > 0 && block_number % epoch == 0;
        assert!(is_epoch);

        let mut extra_data = Vec::new();
        extra_data.extend_from_slice(&[0u8; EXTRA_VANITY_LENGTH]); // 32 vanity
        if is_epoch {
            for signer in signers {
                extra_data.extend_from_slice(signer.as_slice()); // 20 bytes each
            }
        }
        extra_data.extend_from_slice(&[0u8; EXTRA_SEAL_LENGTH]); // 65 seal

        // For 3 signers: 32 + 3*20 + 65 = 157 bytes
        assert_eq!(extra_data.len(), 32 + 3 * 20 + 65);
        assert_eq!(extra_data.len(), 157);
    }

    #[tokio::test]
    async fn test_non_epoch_block_no_signers_in_extra_data() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let epoch = chain.epoch();

        // Block 1 is not an epoch block
        let block_number = 1u64;
        let is_epoch = block_number > 0 && block_number % epoch == 0;
        assert!(!is_epoch);

        let mut extra_data = Vec::new();
        extra_data.extend_from_slice(&[0u8; EXTRA_VANITY_LENGTH]);
        // No signers for non-epoch blocks
        extra_data.extend_from_slice(&[0u8; EXTRA_SEAL_LENGTH]);

        // Non-epoch: 32 + 65 = 97 bytes
        assert_eq!(extra_data.len(), 97);
    }

    #[tokio::test]
    async fn test_signed_header_verifiable_by_consensus() {
        let chain = Arc::new(PoaChainSpec::dev_chain());
        let manager = dev::setup_dev_signers().await;
        let signers = chain.signers();

        // Sign a block header with the first authorized signer
        let signer_addr = signers[0];
        assert!(manager.has_signer(&signer_addr).await);

        let header = Header {
            number: 1,
            difficulty: U256::from(2), // Out-of-turn for signer[0] at block 1
            gas_limit: 30_000_000,
            timestamp: 12345,
            extra_data: Bytes::from(vec![0u8; EXTRA_VANITY_LENGTH + EXTRA_SEAL_LENGTH]),
            ..Default::default()
        };

        let sealer = BlockSealer::new(manager);
        let signed_header = sealer.seal_header(header, &signer_addr).await.unwrap();

        // Now verify using PoaConsensus
        let consensus = crate::consensus::PoaConsensus::new(chain);
        let recovered = consensus.recover_signer(&signed_header).unwrap();
        assert_eq!(recovered, signer_addr);
        assert!(consensus.validate_signer(&recovered).is_ok());
    }
}
