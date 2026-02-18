# Implementation Log — 2026-02-18

## Summary

This session implemented `PoaEngineValidator` — the final piece needed to allow production-mode
POA blocks (with 97-byte `extra_data`) to pass through Reth's Engine API pipeline without being
rejected by alloy's 32-byte `MAXIMUM_EXTRA_DATA_SIZE` check.

Also added `--mining` CLI flag for testing production signing with interval block production,
and fixed a critical `difficulty` hash-mismatch bug.

---

## 1. Problem: Engine API Rejects 97-Byte extra_data

### Root Cause

Alloy's `ExecutionPayloadV1::into_block_raw_with_transactions_root_opt()` enforces:
```
MAXIMUM_EXTRA_DATA_SIZE = 32 bytes
```

POA blocks use 97-byte `extra_data`:
```
[vanity: 32 bytes][ECDSA signature: 65 bytes] = 97 bytes total
(epoch blocks additionally carry N*20 bytes of signer addresses between vanity and seal)
```

When the Engine API receives a `engine_newPayloadV3` call with a POA block, it calls
`convert_payload_to_block()` → alloy rejects with "extra_data too long" before
`PoaConsensus` ever gets a chance to validate the signature.

### Solution: PoaEngineValidator (src/node.rs)

Strip `extra_data` before alloy conversion, restore it after, then reseal to get the correct hash:

```
engine_newPayloadV3 call
    │
    ▼
PoaEngineValidator::convert_payload_to_block()
    │
    ├── 1. extract expected_hash = payload.block_hash()
    ├── 2. strip_extra_data(payload) → (stripped_payload, orig_extra_97_bytes)
    ├── 3. stripped_payload.try_into_block_with_sidecar() → Block (extra_data = "")
    ├── 4. block.header.extra_data = orig_extra_97_bytes  (restore)
    ├── 5. SealedBlock::seal_slow(block)  (recompute hash with restored extra_data)
    └── 6. assert sealed.hash() == expected_hash  (or return PayloadError::BlockHash)
```

---

## 2. Implementation: PoaEngineValidator

### strip_extra_data (src/node.rs:66-81)

```rust
fn strip_extra_data(payload: ExecutionPayload) -> (ExecutionPayload, alloy_primitives::Bytes) {
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
```

### PoaEngineValidator struct (src/node.rs:88-97)

Wraps `EthereumEngineValidator` and overrides only `convert_payload_to_block`:

```rust
#[derive(Debug, Clone)]
pub struct PoaEngineValidator<ChainSpec = reth_chainspec::ChainSpec> {
    inner: EthereumEngineValidator<ChainSpec>,
}

impl<ChainSpec> PoaEngineValidator<ChainSpec> {
    pub const fn new(chain_spec: Arc<ChainSpec>) -> Self {
        Self { inner: EthereumEngineValidator::new(chain_spec) }
    }
}
```

### PayloadValidator impl (src/node.rs:99-139)

```rust
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
        let (stripped, orig_extra) = strip_extra_data(payload);
        let mut block: reth_ethereum::Block = stripped
            .try_into_block_with_sidecar(&sidecar)
            .map_err(|e| NewPayloadError::Other(e.into()))?;
        block.header.extra_data = orig_extra;
        let sealed = SealedBlock::seal_slow(block);
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
```

### EngineApiValidator impl (src/node.rs:141-169)

Delegates `validate_version_specific_fields` and `ensure_well_formed_attributes` to inner
via fully-qualified trait syntax (required by Rust to disambiguate):

```rust
impl<ChainSpec, Types> EngineApiValidator<Types> for PoaEngineValidator<ChainSpec>
where ...
{
    fn validate_version_specific_fields(&self, ...) -> ... {
        <EthereumEngineValidator<ChainSpec> as EngineApiValidator<Types>>::validate_version_specific_fields(
            &self.inner, version, payload_or_attrs,
        )
    }
    fn ensure_well_formed_attributes(&self, ...) -> ... {
        <EthereumEngineValidator<ChainSpec> as EngineApiValidator<Types>>::ensure_well_formed_attributes(
            &self.inner, version, attributes,
        )
    }
}
```

### PoaEngineValidatorBuilder (src/node.rs:172-193)

Implements `PayloadValidatorBuilder<Node>` for injection into the node's AddOns:

```rust
#[derive(Debug, Default, Clone)]
pub struct PoaEngineValidatorBuilder;

impl<Node, Types> PayloadValidatorBuilder<Node> for PoaEngineValidatorBuilder
where
    Types: NodeTypes<
        ChainSpec: EthChainSpec + EthereumHardforks + Clone + 'static,
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
```

### PoaNode::add_ons() updated (src/node.rs:325-360)

```rust
type AddOns = EthereumAddOns<
    NodeAdapter<N>,
    EthereumEthApiBuilder,
    PoaEngineValidatorBuilder,
    BasicEngineApiBuilder<PoaEngineValidatorBuilder>,
    BasicEngineValidatorBuilder<PoaEngineValidatorBuilder>,
    Identity,
>;

fn add_ons(&self) -> Self::AddOns {
    EthereumAddOns::new(RpcAddOns::new(
        EthereumEthApiBuilder::default(),
        PoaEngineValidatorBuilder,
        BasicEngineApiBuilder::<PoaEngineValidatorBuilder>::default(),
        BasicEngineValidatorBuilder::new(PoaEngineValidatorBuilder),
        Identity::default(),
    ))
}
```

---

## 3. Bug Fix: Difficulty Hash Mismatch

### Root Cause

The payload builder was setting `difficulty = 1` (in-turn) or `2` (out-of-turn).
The Engine API (`ExecutionPayloadV1`) has **no difficulty field**. Alloy hardcodes
`difficulty = U256::ZERO` when deserializing a payload back to a block.

In `PoaEngineValidator::convert_payload_to_block`, we strip extra_data, convert
(which sets difficulty = 0), restore extra_data, and reseal. The new hash includes
`difficulty = 0`, but the original block was sealed with `difficulty = 1/2`, so
`expected_hash != sealed.hash()` → hash mismatch error.

### Fix

**`src/payload.rs`**: Always set `difficulty = U256::ZERO`:
```rust
// Difficulty must be 0 for Engine API compatibility.
// ExecutionPayloadV1 has no difficulty field; alloy hardcodes U256::ZERO on conversion.
// POA authority is determined solely by the ECDSA signature in extra_data.
header.difficulty = U256::ZERO;
```

**`src/consensus.rs`**: Updated `validate_difficulty` to require 0:
```rust
pub fn validate_difficulty(&self, header: &Header, _signer: &Address) -> Result<(), PoaConsensusError> {
    if header.difficulty != U256::ZERO {
        return Err(PoaConsensusError::InvalidDifficulty);
    }
    Ok(())
}
```

Updated 3 tests:
- `test_validate_difficulty_zero` — passes with `U256::ZERO`
- `test_validate_difficulty_zero_any_signer` — any authorized signer with `U256::ZERO`
- `test_validate_difficulty_nonzero_rejected` — `U256::from(1)` is rejected

---

## 4. New CLI Flag: --mining

**`src/main.rs`**: Added `--mining` flag to force interval block production in production mode:

```rust
/// Force interval-based block production even in production mode.
/// Useful for testing: node uses production signing (97-byte extra_data, strict POA)
/// but still auto-mines blocks at --block-time interval.
#[arg(long)]
mining: bool,
```

```rust
let mining_enabled = is_dev_mode || cli.mining;
let dev_args = if !mining_enabled {
    DevArgs::default()
} else {
    DevArgs { dev: true, block_time: Some(Duration::from_secs(...)), ... }
};
```

Mode display:
```
Mode: production+mining   (--production --mining)
Mode: production           (--production alone)
Mode: dev                  (default)
```

---

## 5. Bug Fix: block_in_place for Async Context

### Problem

In production+mining mode, the payload builder's `sign_payload()` is called from inside
a Tokio async task (the mining loop). `Handle::block_on()` panics if called from within
an async context: "Cannot start a runtime from within a runtime".

### Fix (src/payload.rs)

```rust
// Correct: wrap block_on with block_in_place to park the async task
let (signer_addr, is_in_turn) = tokio::task::block_in_place(|| {
    handle.block_on(async {
        // ... async signer selection
    })
});

let signed_header = tokio::task::block_in_place(|| {
    handle.block_on(async { sealer.seal_header(header, &signer_addr).await })
})
.map_err(|e| PayloadBuilderError::Other(Box::new(e)))?;
```

---

## 6. Compile Errors Encountered and Fixed

| Error | Cause | Fix |
|-------|-------|-----|
| `reth_node_api::PayloadTypes` unresolved | `reth_node_api` not a direct dep | Use `PayloadTypes` from `reth_payload_primitives` |
| `reth_node_api::EngineTypes` unresolved | Same | Add `EngineTypes` to import from `reth_ethereum::node::api` |
| `reth_ethereum_primitives::Block` unresolved | Not a direct dep | Use `reth_ethereum::Block` |
| `BasicEngineApiBuilder::new()` not found | No `new()` method | Use `BasicEngineApiBuilder::<PVB>::default()` |
| Type ambiguity in `EngineApiValidator` delegation | Compiler can't resolve trait method | Use fully-qualified path: `<EthereumEngineValidator<ChainSpec> as EngineApiValidator<Types>>::method(...)` |
| `EthereumAddOns::default()` fails for custom PVB | `Default` only for `EthereumEngineValidatorBuilder` | Construct via `EthereumAddOns::new(RpcAddOns::new(...))` |
| `validate_version_specific_fields` unused import | Import not needed | Removed from import |

---

## 7. New Dependency

**`Cargo.toml`**: Added `alloy-rpc-types-engine = "1"` for `PayloadError::BlockHash` variant
used in `PoaEngineValidator::convert_payload_to_block`.

---

## 8. Test Results

```
test result: ok. 194 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

New tests added in `src/node.rs`:
- `test_poa_node_creation`
- `test_poa_node_with_dev_mode`
- `test_poa_node_with_signer_manager`
- `test_poa_node_full_builder_chain`
- `test_poa_consensus_builder_creation`
- `test_poa_consensus_builder_dev_mode`
- `test_strip_extra_data_v1`
- `test_poa_engine_validator_builder_is_default`

---

## 9. Production Mode Verification

Test command:
```bash
./target/release/example-custom-poa-node \
  --production --mining --block-time 2 \
  --signer-key ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --datadir /tmp/meow-prod-test \
  --http-port 8547 --ws-port 8548
```

Observed output (9 blocks in 20s, no errors):
```
POA Consensus initialized: 5 signers, epoch: 30000, period: 2s, mode: production (strict)
POA node started successfully!
  POA block #1 signed by 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (out-of-turn)
  Block #1 - 0 txs (out-of-turn, expected: 0x70997970...)
  POA block #5 signed by 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 (in-turn)
  Block #5 - 0 txs (in-turn: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266)
  ...
```

No hash mismatch errors. Blocks accepted by the engine tree with 97-byte extra_data.

---

## 10. Key Reth Architecture Learnings

### Engine API flow for custom extra_data

```
Block Producer (PoaPayloadBuilder)
  └── Signs block with 97-byte extra_data → submits via engine_newPayloadV3

Engine Tree receives newPayload call
  └── PoaEngineValidator::convert_payload_to_block()
       ├── strips extra_data (bypasses alloy 32-byte check)
       ├── converts payload to block
       ├── restores extra_data
       └── reseals → passes sealed block to PoaConsensus

PoaConsensus::validate_block_pre_execution()
  └── recovers signer from ECDSA sig in extra_data → validates against signer list
```

### Why difficulty must be 0

The Engine API post-merge design treats difficulty as not transmitted. `ExecutionPayloadV1`
has no difficulty field; alloy hardcodes `difficulty = U256::ZERO` when deserializing.
POA authority is conveyed entirely through the ECDSA signature in `extra_data`, not difficulty.
Any non-zero difficulty breaks the hash round-trip through the Engine API.

### PayloadValidatorBuilder vs EthereumAddOns::default()

`EthereumAddOns` is generic over the `PayloadValidatorBuilder` type:
```
EthereumAddOns<N, EthApiBuilder, PVB, EngineApiBuilder, EngineValidator, Events>
```
The `Default` impl only exists when `PVB = EthereumEngineValidatorBuilder`. For a custom PVB,
construct via `EthereumAddOns::new(RpcAddOns::new(...))`.

### block_in_place + block_on

When calling async code from within a Tokio multi-threaded async context (e.g., payload builder
triggered by the dev miner):
- `Handle::block_on()` → panics ("Cannot start a runtime from within a runtime")
- `tokio::task::block_in_place(|| handle.block_on(...))` → correct approach
