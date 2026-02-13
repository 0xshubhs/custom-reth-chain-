# Meowchain - Custom POA Blockchain on Reth

## Project Overview

Custom Proof of Authority (POA) blockchain built on [Reth](https://github.com/paradigmxyz/reth) (Rust Ethereum client). The node is Ethereum mainnet-compatible for smart contract execution, hardforks, and JSON-RPC APIs, but replaces beacon consensus with a POA signer-based model.

**Reth:** Tracks `main` branch (latest). Use `just build` to fetch latest + build.

## Architecture

```
meowchain (PoaNode)
  ├── Consensus: PoaConsensus (validates headers, signatures, timing, gas limits)
  ├── Block Production: Reth interval mining (dev mode) - NOT yet POA-signed
  ├── Block Rewards: EIP-1967 Miner Proxy at 0x...1967 (coinbase)
  ├── EVM: Identical to Ethereum mainnet (all opcodes, precompiles)
  ├── Hardforks: Frontier through Prague (all active at genesis)
  ├── RPC: HTTP (8545) + WebSocket (8546) on 0.0.0.0
  └── Storage: MDBX persistent database (production NodeBuilder)
```

## Source Files

| File | Purpose | Status |
|------|---------|--------|
| `src/main.rs` | Entry point, CLI parsing, production node launch, block monitoring | Working |
| `src/node.rs` | `PoaNode` type - injects `PoaConsensus` with dev_mode flag | Working |
| `src/consensus.rs` | `PoaConsensus` - signature verification, header validation, post-execution checks | Working |
| `src/chainspec.rs` | `PoaChainSpec` - hardforks, POA config, signer list | Complete |
| `src/genesis.rs` | Genesis creation, system contracts, ERC-4337 pre-deploys, EIP-1967 miner proxy | Complete |
| `src/signer.rs` | `SignerManager` + `BlockSealer` - key management & signing | Working (not in pipeline) |
| `src/bytecodes/` | Pre-compiled contract bytecodes (.bin/.hex) | Complete |

## Key Types & Import Paths

- `PoaNode` → `src/node.rs` - custom `Node` impl, replaces `EthereumNode`
- `PoaConsensus` → `src/consensus.rs` - implements `HeaderValidator`, `Consensus`, `FullConsensus`
- `PoaConsensusBuilder` → `src/node.rs` - `ConsensusBuilder` trait impl
- `PoaChainSpec` → `src/chainspec.rs` - wraps `ChainSpec` + `PoaConfig`
- `SignerManager` → `src/signer.rs` - runtime key management (RwLock<HashMap>)
- `BlockSealer` → `src/signer.rs` - seal/verify block headers
- `MINER_PROXY_ADDRESS` → `src/genesis.rs` - EIP-1967 proxy at `0x...1967`

### Reth Import Conventions
```rust
reth_ethereum::node::builder::*        // = reth_node_builder
reth_ethereum::node::*                 // = reth_node_ethereum (EthereumNode, builders)
reth_ethereum::EthPrimitives           // from reth_ethereum_primitives
reth_ethereum::provider::EthStorage    // from reth_provider
reth_ethereum::rpc::eth::primitives::Block  // RPC block type
reth_ethereum::tasks::TaskExecutor     // = Runtime (alias). Create with TaskExecutor::with_existing_handle()
reth_payload_primitives::PayloadTypes  // NOT re-exported by reth_ethereum
alloy_consensus::BlockHeader           // Use this for header method access (gas_used, gas_limit, extra_data)
```

## What's Done

### P0-Alpha (All Fixed)
- [x] **A1** - `NodeConfig::default()` with proper args
- [x] **A2** - Production `NodeBuilder` with persistent MDBX (`init_db` + `with_database`)
- [x] **A3** - `PoaNode` replaces `EthereumNode`
- [x] **A5** - `PoaConsensus` wired into pipeline via `PoaConsensusBuilder`

### P0 (Mostly Fixed)
- [x] **#2** - External RPC server: HTTP + WS on 0.0.0.0
- [x] **#3** - Consensus enforces POA signatures in production mode (`recover_signer` + `validate_signer`)
- [x] **#4** - Post-execution validates gas_used, receipt root, and logs bloom
- [x] **#5** - Chain ID 9323310 everywhere including sample-genesis.json
- [x] **#6** - CLI parsing with clap
- [~] **#1** - Signer loaded at runtime, but blocks still unsigned (needs `PoaPayloadBuilder`)
- [~] **#7** - Keys loadable from env/CLI, but dev keys still hardcoded

### NOT Fixed
- [ ] **A4** - No custom `PoaPayloadBuilder` - blocks produced unsigned
- [ ] **A6** - `BlockSealer` exists but not wired into payload pipeline

## What's NOT Done (Major Gap)

**Block Signing in Pipeline** - The single biggest gap. Blocks are produced by Reth's default `EthereumPayloadBuilder` without POA signatures. Need a custom `PoaPayloadBuilder` that: signs blocks, sets difficulty 1/2, embeds signer list at epoch blocks.

## Chain Configuration

| Parameter | Dev Mode | Production |
|-----------|----------|------------|
| Chain ID | 9323310 | 9323310 |
| Block Time | 2s | 12s |
| Gas Limit | 30M | 60M |
| Signers | 3 (first 3 dev accounts) | 5 (first 5 dev accounts) |
| Epoch | 30,000 blocks | 30,000 blocks |
| Prefunded | 20 accounts @ 10K ETH | 8 accounts (tiered) |
| Coinbase | EIP-1967 Miner Proxy | EIP-1967 Miner Proxy |

## Genesis Pre-deployed Contracts

| Contract | Address | Source |
|----------|---------|--------|
| EIP-1967 Miner Proxy | `0x0000000000000000000000000000000000001967` | Block rewards (coinbase) |
| EIP-4788 Beacon Root | `0x000F3df6D732807Ef1319fB7B8bB8522d0Beac02` | System (Cancun) |
| EIP-2935 History Storage | `0x0000F90827F1C53a10cb7A02335B175320002935` | System (Prague) |
| EIP-7002 Withdrawal Requests | `0x00000961Ef480Eb55e80D19ad83579A64c007002` | System (Prague) |
| EIP-7251 Consolidation | `0x0000BBdDc7CE488642fb579F8B00f3a590007251` | System (Prague) |
| ERC-4337 EntryPoint v0.7 | `0x0000000071727De22E5E9d8BAf0edAc6f37da032` | Infra |
| WETH9 | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` | Infra |
| Multicall3 | `0xcA11bde05977b3631167028862bE2a173976CA11` | Infra |
| CREATE2 Deployer | `0x4e59b44847b379578588920cA78FbF26c0B4956C` | Infra |
| SimpleAccountFactory | `0x9406Cc6185a346906296840746125a0E44976454` | Infra |

## Building & Running

```bash
# Build (fetches latest reth + all crates, then builds release)
just build

# Quick build without updating deps
just build-fast

# Dev mode (default)
just dev

# Run with custom args
just run-custom --chain-id 9323310 --block-time 12 --datadir /data/meowchain

# With signer key from environment
SIGNER_KEY=ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 just dev

# Production mode
just run-production

# Run tests
just test

# Docker
just docker
```

## Development Notes

- 70 unit tests: `just test` (or `cargo test`)
- Consensus traits use `#[auto_impl::auto_impl(&, Arc)]` - `Arc<PoaConsensus>` auto-implements traits
- `launch_with_debug_capabilities()` requires `DebugNode` impl (in node.rs)
- Dev mode: auto-mines blocks, relaxed consensus (no signature checks)
- Production mode: strict consensus with POA signature verification
- The `clique` field in genesis config JSON is informational only - not parsed by Reth
- `just build` runs `cargo update` first to fetch latest reth from main branch

## Common Pitfalls

- `alloy_consensus::BlockHeader` vs `reth_primitives_traits::BlockHeader` - use alloy version for method access
- `NodeConfig::test()` enables dev mode by default; `NodeConfig::default()` does NOT
- `launch()` vs `launch_with_debug_capabilities()` - debug version needed for dev mining
- `TaskManager` is now internal in latest reth - use `TaskExecutor::with_existing_handle(Handle::current())`
- `HeaderValidator<Header>` uses concrete type - `Consensus<B>` needs `where PoaConsensus: HeaderValidator<B::Header>`
