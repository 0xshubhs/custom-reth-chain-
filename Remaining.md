### Meowchain Custom POA Chain - Status Tracker

> **Last audited: 2026-02-13**

## Table of Contents

1. [What's Done](#1-whats-done)
2. [Critical Gaps (Production Blockers)](#2-critical-gaps-production-blockers)
   - 2.5 [Multi-Node POA Operation](#25-multi-node-poa-operation-how-others-run-the-chain)
3. [Remaining Infrastructure](#3-remaining-infrastructure)
4. [Chain Recovery & Resumption](#4-chain-recovery--resumption)
5. [Upgrade Mechanism](#5-upgrade-mechanism-hardfork-support)
6. [All Finalized EIPs by Hardfork](#6-all-finalized-eips-by-hardfork)
7. [ERC Standards Support](#7-erc-standards-support)
8. [ERC-8004: AI Agent Support](#8-erc-8004-trustless-ai-agents)
9. [Upcoming Ethereum Upgrades](#9-upcoming-ethereum-upgrades)
10. [Production Infrastructure Checklist](#10-production-infrastructure-checklist)
11. [Codebase Issues Found During Audit](#11-codebase-issues-found-during-audit)

---

## 1. What's Done

### Core Modules (src/)

| Module | File | Lines | Status |
|--------|------|-------|--------|
| Entry point | `main.rs` | ~304 | Working - CLI parsing, interval mining, dev mode, block monitoring |
| Node type | `node.rs` | ~175 | Working - PoaNode with PoaConsensusBuilder, DebugNode impl |
| Chain spec | `chainspec.rs` | ~292 | Complete - all hardforks, POA config, trait impls |
| Consensus | `consensus.rs` | ~385 | Partial - structural validation works, NO signature verification |
| Genesis | `genesis.rs` | ~575 | Complete - dev/production configs, system contracts + ERC-4337 pre-deploys |
| Signer | `signer.rs` | ~298 | Working module - loaded at runtime, NOT in block production pipeline |
| Bytecodes | `src/bytecodes/` | 10 files | Complete - .bin + .hex for all pre-deployed contracts |

### Hardforks Enabled (All at Block 0 / Timestamp 0)

| Hardfork | Status | Key Features |
|----------|--------|--------------|
| Frontier through London | Active | Full EVM, EIP-1559, CREATE2, REVERT, etc. |
| Paris (Merge) | Active | TTD=0, PREVRANDAO |
| Shanghai | Active | PUSH0, withdrawals ops |
| Cancun | Active | EIP-4844 blobs, TSTORE/TLOAD, MCOPY |
| Prague | Active | BLS precompile, EIP-7702, blob increase |

### System Contracts Deployed in Genesis

| EIP | Address | Purpose |
|-----|---------|---------|
| EIP-4788 | `0x000F3df6D732807Ef1319fB7B8bB8522d0Beac02` | Beacon block root |
| EIP-2935 | `0x0000F90827F1C53a10cb7A02335B175320002935` | History storage |
| EIP-7002 | `0x00000961Ef480Eb55e80D19ad83579A64c007002` | Withdrawal requests |
| EIP-7251 | `0x0000BBdDc7CE488642fb579F8B00f3a590007251` | Consolidation requests |

### ERC-4337 & Infrastructure Contracts in Genesis (NEW)

| Contract | Address | Purpose |
|----------|---------|---------|
| EntryPoint v0.7 | `0x0000000071727De22E5E9d8BAf0edAc6f37da032` | ERC-4337 core |
| WETH9 | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` | Wrapped native token |
| Multicall3 | `0xcA11bde05977b3631167028862bE2a173976CA11` | Batch RPC calls |
| CREATE2 Deployer | `0x4e59b44847b379578588920cA78FbF26c0B4956C` | Deterministic deploys |
| SimpleAccountFactory | `0x9406Cc6185a346906296840746125a0E44976454` | ERC-4337 wallet factory |

### Infrastructure Done

- [x] Docker build (`Dockerfile`)
- [x] Docker Compose (single node)
- [x] Blockscout explorer integration (Scoutup Go app in `scoutup/`)
- [x] MDBX persistent storage (`data/db/`)
- [x] Static files for headers/txns/receipts
- [x] Dev mode with configurable block time (default 2s)
- [x] 20 prefunded accounts (10,000 ETH each in dev, tiered in production)
- [x] 3 default POA signers (round-robin logic in chainspec)
- [x] EIP-1559 base fee (0.875 gwei initial)
- [x] EIP-4844 blob support enabled
- [x] Basic unit tests in each module
- [x] CLI argument parsing (clap) - chain-id, block-time, datadir, http/ws config, signer-key
- [x] External HTTP RPC on 0.0.0.0:8545
- [x] External WebSocket RPC on 0.0.0.0:8546
- [x] Runtime signer key loading from CLI `--signer-key` or `SIGNER_KEY` env var
- [x] Chain ID unified to 9323310 across dev and production genesis configs
- [x] PoaNode type replacing EthereumNode (injects PoaConsensus into Reth pipeline)
- [x] PoaConsensusBuilder wired into ComponentsBuilder
- [x] Production genesis config (5 signers, 60M gas, tiered treasury/ops/community allocation)
- [x] Genesis extra_data with POA format (vanity + signers + seal)
- [x] Block monitoring task that logs signer turn info
- [x] ERC-4337 EntryPoint, WETH9, Multicall3, CREATE2 Deployer pre-deployed at genesis

---

## 2. Critical Gaps (Production Blockers)

### P0 - Must Fix Before Any Deployment

| # | Issue | Status | Details | File |
|---|-------|--------|---------|------|
| 1 | **Block signing not integrated** | PARTIALLY FIXED | `SignerManager` is instantiated, loaded from CLI/env, and a block monitoring task is spawned. Blocks are produced unsigned by Reth's default payload builder. Block rewards go to EIP-1967 miner proxy at `0x...1967`. | `main.rs`, `signer.rs`, `genesis.rs` |
| 2 | **No external RPC server** | FIXED | HTTP RPC on `0.0.0.0:8545` and WS on `0.0.0.0:8546` configured via `RpcServerArgs`. | `main.rs:189-199` |
| 3 | **No consensus enforcement on sync** | FIXED | `PoaConsensus` validates headers with POA signature recovery in production mode. Dev mode skips signature checks. `recover_signer()` called in `validate_header()`. | `consensus.rs:249-287` |
| 4 | **Post-execution validation stubbed** | FIXED | Validates `gas_used`, receipt root, and logs bloom against pre-computed values. | `consensus.rs:393-429` |
| 5 | **Chain ID mismatch** | FIXED | All configs use 9323310. `sample-genesis.json` regenerated from code with correct chain ID, all contracts. | `genesis.rs`, `sample-genesis.json` |
| 6 | **No CLI argument parsing** | FIXED | Full `clap` CLI with all flags. | `main.rs:62-105` |
| 7 | **Hardcoded dev keys in binary** | PARTIALLY FIXED | Production loads from `--signer-key` / `SIGNER_KEY`. Dev keys still hardcoded for dev mode. | `main.rs:156-175`, `signer.rs:205-216` |

### P0-ALPHA - Fundamental Architecture Problems

> **Progress update (2026-02-13):** The node uses production `NodeBuilder` with persistent MDBX database. PoaConsensus validates signatures in production mode. EIP-1967 miner proxy collects block rewards. 70 tests pass.

| # | Issue | Status | What the code does now | What still needs to happen |
|---|-------|--------|------------------------|---------------------------|
| A1 | **`NodeConfig::test()` used** | FIXED | `NodeConfig::default()` with `.with_dev()`, `.with_rpc()`, `.with_chain()`, `.with_datadir_args()` | Done |
| A2 | **`testing_node_with_datadir()` used** | FIXED | Production `NodeBuilder::new(config).with_database(init_db()).with_launch_context(executor)` with persistent MDBX | Done |
| A3 | **`EthereumNode::default()` used** | FIXED | `.node(PoaNode::new(chain_spec).with_dev_mode(is_dev_mode))` injects `PoaConsensus` | Done |
| A4 | **No custom PayloadBuilder** | NOT FIXED | Still uses `BasicPayloadServiceBuilder::default()` wrapping `EthereumPayloadBuilder`. No signing, no difficulty field, no epoch signer list in produced blocks | Must implement `PoaPayloadBuilder` |
| A5 | **Consensus module is dead code** | FIXED | `PoaConsensus` LIVE in pipeline with signature verification | Done |
| A6 | **Signer module is dead code** | PARTIALLY FIXED | `SignerManager` loaded with keys, used in monitoring. `BlockSealer` exists but not in payload pipeline | Need `PoaPayloadBuilder` integration |

**Current architecture (2026-02-13):**

```
What we have now:
  main.rs -> NodeConfig::default() + CLI args (clap)
    -> Production NodeBuilder with persistent MDBX database
    -> PoaNode (custom node type, dev_mode flag)
      -> Components:
        consensus:       PoaConsensus (LIVE - signature verification, timing, gas, receipt root)
        payload_builder: EthereumPayloadBuilder (DEFAULT - no signing)
        network:         EthereumNetworkBuilder (DEFAULT)
        pool:            EthereumPoolBuilder (DEFAULT)
      -> Block rewards: go to EIP-1967 miner proxy (0x...1967)
      -> Block production: Reth dev mining (unsigned blocks)
      -> SignerManager: loaded with keys, ready for integration

What still needs to happen:
  1. Implement PoaPayloadBuilder (signs blocks, sets difficulty, epoch signers)
  2. Wire BlockSealer into the payload building pipeline
```

### P1 - Required for Production

| # | Issue | Details |
|---|-------|---------|
| 8 | No admin/debug/txpool RPC namespaces | Can't manage node, trace transactions, or inspect mempool |
| 9 | No signer voting mechanism | Can't add/remove signers dynamically via governance |
| 10 | No monitoring/metrics (Prometheus) | Port 9001 exposed but no metrics server running |
| 11 | No CI/CD pipeline | No automated testing, linting, or deployment |
| 12 | No integration tests | Only unit tests; no end-to-end block production/validation tests |
| 13 | No bootnodes configured | P2P discovery works but has no seed nodes |
| 14 | Reth deps pinned to `main` branch | Bleeding edge, risk of breaking changes. Should pin to release tags |

---

## 2.5 Multi-Node POA Operation (How Others Run the Chain)

> **No beacon chain needed.** POA is self-contained. Signers ARE the consensus. No validators, no staking, no attestations. Each signer node takes turns producing blocks in round-robin order.

### Current State: Single-Node Only

The chain currently runs as a **single isolated dev node**. There is zero support for:
- A second node joining the network
- Sharing genesis so another node starts from the same state
- Peer discovery between nodes
- Distributing the signer role across machines

### Network Topology for POA

```
What a real POA network looks like:

                    ┌─────────────────────┐
                    │   Bootnode(s)        │
                    │   (discovery only,   │
                    │    no signing)        │
                    └─────────┬───────────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
     ┌────────▼──────┐ ┌─────▼───────┐ ┌─────▼───────┐
     │ Signer Node 1 │ │ Signer Node 2│ │ Signer Node 3│
     │ (Account 0)   │ │ (Account 1)  │ │ (Account 2)  │
     │ Produces block │ │ Produces block│ │ Produces block│
     │ every 3rd turn │ │ every 3rd turn│ │ every 3rd turn│
     │ Has private key│ │ Has private key│ │ Has private key│
     └───────┬────────┘ └──────┬───────┘ └──────┬───────┘
             │                 │                 │
     ┌───────▼─────────────────▼─────────────────▼───────┐
     │              Full Nodes (RPC nodes)                │
     │  - No signing keys                                 │
     │  - Validate and store all blocks                   │
     │  - Serve RPC to users (MetaMask, dApps)           │
     │  - Anyone can run one                              │
     └───────────────────────────────────────────────────┘
```

### Node Types in POA

| Node Type | Has Private Key | Produces Blocks | Validates Blocks | Serves RPC | Who Runs It |
|-----------|----------------|-----------------|------------------|------------|-------------|
| **Signer Node** | Yes | Yes (when in-turn) | Yes | Optional | Authorized signers only |
| **Full Node** | No | No | Yes | Yes | Anyone |
| **Archive Node** | No | No | Yes (all history) | Yes | Infrastructure providers |
| **Bootnode** | No | No | No | No | Chain operators |

### How a New Operator Joins the Network

**Step 1: Get the genesis file**
```bash
# The genesis.json must be IDENTICAL across all nodes
# It defines: chain ID, initial state, signer list, system contracts
# Distribute via: git repo, IPFS, or direct download
curl -O https://meowchain.example.com/genesis.json
```

**Step 2: Initialize the node from genesis**
```bash
# This creates the database with the exact same initial state
meowchain init --genesis genesis.json --datadir /data/meowchain
```

**Step 3: Connect to the network**
```bash
# Bootnodes are the entry point to find other peers
meowchain run \
  --datadir /data/meowchain \
  --bootnodes "enode://<pubkey>@<ip>:30303,enode://<pubkey2>@<ip2>:30303" \
  --http --http.addr 0.0.0.0 --http.port 8545 \
  --ws --ws.addr 0.0.0.0 --ws.port 8546 \
  --port 30303
```

**Step 4: Sync state from peers**
```
Node connects to peers -> requests headers -> validates POA signatures
-> downloads block bodies -> replays transactions -> builds local state
-> reaches chain tip -> now a full node
```

**Step 5 (Signer only): Import signing key**
```bash
# Only if this node is an authorized signer
meowchain account import --keyfile signer-key.json --datadir /data/meowchain

# Then run with signing enabled
meowchain run \
  --datadir /data/meowchain \
  --signer 0xYourSignerAddress \
  --unlock 0xYourSignerAddress \
  --bootnodes "enode://..." \
  --mine  # Enable block production
```

### What's Missing for Multi-Node

| Component | Status | What's Needed |
|-----------|--------|---------------|
| **`meowchain init` command** | Not implemented | CLI subcommand to initialize DB from genesis.json |
| **`meowchain run` command** | Partially done | CLI exists with `--datadir`, `--http-*`, `--ws-*`, `--signer-key` flags. Missing: `--bootnodes`, `--port`, `--mine`, `--unlock` |
| **`meowchain account` command** | Not implemented | Import/export/list signing keys |
| **Genesis file distribution** | Partially done | `genesis.rs` can generate canonical JSON via `genesis_to_json()` and `write_genesis_file()`. `sample-genesis.json` is STALE (wrong chain ID, missing contracts) - needs regeneration |
| **Bootnode infrastructure** | Not implemented | At least 2-3 bootnodes with static IPs/DNS |
| **Enode URL generation** | Not implemented | Each node needs a public enode URL for peering |
| **State sync protocol** | Not implemented | Full sync from genesis + fast sync from snapshots |
| **Signer key isolation** | DONE | `--signer-key` CLI flag and `SIGNER_KEY` env var. In production mode, runs as non-signer if no key provided. Dev keys only loaded in dev mode. |
| **Block production scheduling** | Partially done | Round-robin logic exists in `chainspec.rs:expected_signer()`. Monitoring task detects in-turn/out-of-turn. But NOT enforced in block building. |
| **Fork choice rule** | Not implemented | Heaviest chain wins (sum of difficulties). In-turn blocks (diff=1) preferred over out-of-turn (diff=2) |
| **Signer voting** | Not implemented | `clique_propose(address, true/false)` to add/remove signers |
| **Epoch checkpoints** | Partially done | `is_epoch_block()` and `extract_signers_from_epoch_block()` exist in `consensus.rs`. Genesis extra_data includes signers. But NOT embedded during block production at epoch boundaries. |

### State Management When Multiple Nodes Run

```
The key insight: EVERY full node has the COMPLETE state.

Block 0 (Genesis):
  All nodes start from identical genesis.json
  State: same prefunded accounts, same system contracts

Block 1..N (Normal operation):
  Signer produces block -> broadcasts to all peers
  Each peer: validates signature -> executes transactions -> updates state
  Result: all nodes have identical state at every block height

Block N (New node joins late):
  Option A - Full Sync:
    Download all blocks 0..N from peers
    Replay every transaction sequentially
    End up with identical state at block N
    Slow but trustless (verifies every POA signature)

  Option B - Snap Sync (needs implementation):
    Download state snapshot at recent block M
    Verify snapshot against known block hash
    Download and replay blocks M..N
    Much faster, still verifiable

Block N+K (Node was offline, comes back):
    Node knows it was at block N
    Requests blocks N+1..N+K from peers
    Validates and replays each block
    Catches up to current chain tip
    RESUMES EXACTLY where it left off
```

### Decentralization in POA Context

POA is **intentionally semi-centralized** - that's the tradeoff:

| Aspect | POA (Meowchain) | PoS (Ethereum Mainnet) | Why POA is different |
|--------|-----------------|----------------------|---------------------|
| Who produces blocks | Fixed set of known signers | Any validator who stakes 32 ETH | Trust is in identity, not economics |
| How to join as producer | Must be voted in by existing signers | Deposit 32 ETH | Permission-based, not permissionless |
| Finality | Immediate (N/2+1 signers confirm) | ~13 min (2 epochs) | Fewer participants = faster |
| Censorship resistance | Lower (signers can collude) | Higher (thousands of validators) | Tradeoff for speed |
| Running a full node | Anyone can | Anyone can | Same - read access is permissionless |
| Sybil resistance | Identity-based (known entities) | Economic (staking cost) | No capital requirement |
| Block time | Configurable (2s, 12s, etc.) | Fixed 12s | More flexible |
| Throughput | Higher (fewer validators to coordinate) | Lower (global consensus) | POA can push gas limits higher |

### Scaling Approaches for POA

Since there's no beacon chain overhead, POA can scale differently:

| Approach | Description | Complexity |
|----------|-------------|------------|
| **Increase gas limit** | POA signers can agree to raise gas limit (e.g., 60M, 100M, 300M). No global consensus needed, just signer agreement | Low |
| **Decrease block time** | 2s -> 1s -> 500ms blocks. Feasible with few signers on good hardware | Low |
| **Parallel EVM execution** | Reth already has foundations for this. Execute non-conflicting txs in parallel | Medium |
| **State pruning** | Aggressive pruning since signers are trusted. Keep only recent state + proofs | Medium |
| **Read replicas** | Run many non-signer full nodes behind a load balancer for RPC traffic | Low |
| **Horizontal RPC scaling** | Multiple RPC nodes + Redis cache + load balancer | Medium |
| **L2 on top of POA** | Deploy an OP Stack / Arbitrum rollup on top of Meowchain as L1 | High |

---

## 3. Remaining Infrastructure

### Networking & P2P

- [ ] Custom P2P handshake with POA chain verification
- [ ] Bootnode configuration and discovery
- [ ] Peer filtering (reject non-POA peers)
- [ ] Network partition recovery
- [ ] Peer reputation / banning malicious peers

### RPC Server

- [x] HTTP JSON-RPC on port 8545 (configurable via `--http-addr` / `--http-port`)
- [x] WebSocket JSON-RPC on port 8546 (configurable via `--ws-addr` / `--ws-port`)
- [x] `eth_*` namespace (provided by Reth's default EthereumEthApiBuilder)
- [x] `web3_*` namespace (provided by Reth)
- [x] `net_*` namespace (provided by Reth)
- [ ] `admin_*` namespace (addPeer, removePeer, nodeInfo)
- [ ] `debug_*` namespace (traceTransaction, traceBlock)
- [ ] `txpool_*` namespace (content, status, inspect)
- [ ] `clique_*` namespace (getSigners, propose, discard) - POA specific (NEEDS CUSTOM IMPL)
- [ ] CORS configuration
- [ ] Rate limiting
- [ ] API key authentication

### State Management

- [ ] Configurable pruning (archive vs. pruned node)
- [ ] State snapshot export/import
- [ ] State sync from peers (fast sync)
- [ ] State trie verification
- [ ] Dead state garbage collection

### Monitoring & Observability

- [ ] Prometheus metrics endpoint (:9001)
- [ ] Grafana dashboard templates
- [ ] Block production rate monitoring
- [ ] Signer health checks
- [ ] Peer count monitoring
- [ ] Mempool size tracking
- [ ] Chain head monitoring
- [ ] Alerting (PagerDuty, Slack, etc.)
- [ ] Structured logging (JSON format)

### Security

- [ ] Encrypted keystore (EIP-2335 style)
- [ ] Key rotation mechanism
- [ ] RPC authentication (JWT for Engine API exists, need for public RPC)
- [ ] DDoS protection
- [ ] Firewall rules documentation
- [ ] Security audit
- [ ] Signer multi-sig support

### Developer Tooling

- [ ] Hardhat/Foundry network config template
- [ ] Contract verification on Blockscout
- [ ] Faucet for testnet tokens
- [ ] Gas estimation service
- [ ] Block explorer API (REST + GraphQL)
- [ ] SDK / client library

---

## 4. Chain Recovery & Resumption

### Current State: Partial Support

Reth's MDBX database persists across restarts. The chain **will resume from the last block** on normal restart. However, several recovery scenarios are NOT handled:

### What Works

| Scenario | Status | How |
|----------|--------|-----|
| Normal restart | Works | MDBX persists state in `data/db/`. Node reads last known head on startup |
| Data directory intact | Works | `data/static_files/` has headers, txns, receipts |

### What's Missing

| Scenario | Status | What's Needed |
|----------|--------|---------------|
| **Corrupted database** | Not handled | Need `reth db repair` or reimport from genesis + replay |
| **State export/import** | Not implemented | Need `reth dump-genesis` equivalent for current state |
| **Snapshot sync** | Not implemented | Need snapshot creation at epoch blocks and distribution |
| **Block replay from backup** | Not implemented | Need block export/import tooling |
| **Disaster recovery** | No plan | Need documented recovery procedures |
| **Multi-node failover** | Not implemented | Need signer failover if primary goes down |
| **Fork resolution** | Not implemented | POA should have canonical fork choice based on signer authority |

### Required Implementation

```
Recovery Tooling Needed:
1. `meowchain export-state --block <number> --output state.json`
2. `meowchain import-state --input state.json`
3. `meowchain export-blocks --from <start> --to <end> --output blocks.rlp`
4. `meowchain import-blocks --input blocks.rlp`
5. `meowchain db repair`
6. `meowchain db verify`
7. Epoch-based automatic snapshots
8. Signer failover with health monitoring
```

---

## 5. Upgrade Mechanism (Hardfork Support)

### Current State: Manual Recompilation Required

All hardforks are activated at genesis (block 0 / timestamp 0). There is **no mechanism** to schedule future hardforks at specific block heights or timestamps.

### What's Needed

| Feature | Status | Description |
|---------|--------|-------------|
| Timestamp-based hardfork scheduling | Not implemented | Schedule future activations like `fusaka_time: 1735689600` |
| Block-based hardfork scheduling | Not implemented | Schedule at specific block numbers |
| On-chain governance for upgrades | Not implemented | Signer voting for hardfork activation |
| Rolling upgrade support | Not implemented | Upgrade nodes one-by-one without downtime |
| Feature flags | Not implemented | Enable/disable features via config |
| Client version signaling | Not implemented | Nodes advertise supported hardforks |
| Emergency hardfork | Not implemented | Fast-track activation for critical patches |

### How Ethereum Mainnet Handles Upgrades

```
1. EIP proposed -> reviewed -> accepted for hardfork
2. Client teams implement in devnets
3. Tested on Holesky/Sepolia testnets
4. Activation time announced (timestamp for post-Merge)
5. All nodes must update before activation time
6. Hardfork activates at exact timestamp across network
7. Nodes running old software fork off and become invalid
```

### Recommended Implementation for Meowchain

```rust
// In chainspec.rs - add configurable future hardforks
pub struct HardforkSchedule {
    pub fusaka_time: Option<u64>,      // Timestamp-based activation
    pub glamsterdam_time: Option<u64>,
    pub custom_forks: BTreeMap<String, u64>,
}

// In genesis.json or chain config:
{
    "config": {
        "pragueTime": 0,
        "fusakaTime": 1735689600,  // Future activation
        "glamsterdamTime": null     // Not yet scheduled
    }
}
```

---

## 6. All Finalized EIPs by Hardfork

### Frontier (Block 0 - July 30, 2015)
> Genesis launch. Base EVM with ~60 opcodes, 5 ETH block reward, Ethash PoW.

### Homestead (Block 1,150,000 - March 14, 2016)

| EIP | Name | Description |
|-----|------|-------------|
| EIP-2 | Homestead Changes | Contract creation cost, tx signature rules, difficulty adjustment |
| EIP-7 | DELEGATECALL | Opcode 0xf4 for delegating execution while preserving caller context |
| EIP-8 | devp2p Forward Compatibility | Networking layer future-proofing |

### Tangerine Whistle (Block 2,463,000 - October 18, 2016)

| EIP | Name | Description |
|-----|------|-------------|
| EIP-150 | Gas cost changes for IO-heavy operations | Repriced opcodes to prevent DoS attacks |

### Spurious Dragon (Block 2,675,000 - November 22, 2016)

| EIP | Name | Description |
|-----|------|-------------|
| EIP-155 | Simple replay attack protection | Chain ID in transaction signatures |
| EIP-160 | EXP cost increase | Balanced computational cost |
| EIP-161 | State trie clearing | Remove empty accounts from DoS attacks |
| EIP-170 | Contract code size limit | Max 24,576 bytes bytecode |

### Byzantium (Block 4,370,000 - October 16, 2017)

| EIP | Name | Description |
|-----|------|-------------|
| EIP-100 | Difficulty adjustment including uncles | Prevents difficulty manipulation |
| EIP-140 | REVERT instruction | Stop execution, revert state, return data without consuming all gas |
| EIP-196 | alt_bn128 addition and scalar multiplication | Precompile for ZK-SNARK verification |
| EIP-197 | alt_bn128 pairing check | Precompile for ZK-SNARK pairing |
| EIP-198 | Big integer modular exponentiation | RSA and crypto precompile |
| EIP-211 | RETURNDATASIZE and RETURNDATACOPY | Variable-length return values |
| EIP-214 | STATICCALL | Non-state-changing calls |
| EIP-649 | Difficulty bomb delay + reward reduction | Block reward: 5 ETH -> 3 ETH |
| EIP-658 | Transaction status code in receipts | 0=failure, 1=success |

### Constantinople (Block 7,280,000 - February 28, 2019)

| EIP | Name | Description |
|-----|------|-------------|
| EIP-145 | Bitwise shifting (SHL, SHR, SAR) | Native shift opcodes, 3 gas each |
| EIP-1014 | CREATE2 | Deterministic contract addresses |
| EIP-1052 | EXTCODEHASH | Efficient contract code hash |
| EIP-1234 | Difficulty bomb delay + reward reduction | Block reward: 3 ETH -> 2 ETH |

### Istanbul (Block 9,069,000 - December 8, 2019)

| EIP | Name | Description |
|-----|------|-------------|
| EIP-152 | BLAKE2b precompile | Zcash interoperability |
| EIP-1108 | Reduce alt_bn128 gas costs | Cheaper ZK-SNARK verification |
| EIP-1344 | ChainID opcode | On-chain chain ID access |
| EIP-1884 | Repricing trie-dependent opcodes | SLOAD 200->800 gas |
| EIP-2028 | Calldata gas reduction | 68->16 gas per non-zero byte |
| EIP-2200 | SSTORE gas rebalancing | Net metering with reentrancy guard |

### Berlin (Block 12,244,000 - April 15, 2021)

| EIP | Name | Description |
|-----|------|-------------|
| EIP-2565 | ModExp gas cost reduction | Cheaper modular exponentiation |
| EIP-2718 | Typed Transaction Envelope | Foundation for future tx types |
| EIP-2929 | Gas cost increase for cold state access | DoS prevention via warm/cold access |
| EIP-2930 | Access Lists (Type 1 tx) | Declare accessed addresses/keys upfront |

### London (Block 12,965,000 - August 5, 2021)

| EIP | Name | Description |
|-----|------|-------------|
| EIP-1559 | Fee market change | Base fee (burned) + priority fee. Type 2 tx |
| EIP-3198 | BASEFEE opcode | On-chain base fee access |
| EIP-3529 | Reduce gas refunds | Kill gas tokens, reduce SELFDESTRUCT refund |
| EIP-3541 | Reject 0xEF prefix contracts | Reserve for future EOF |
| EIP-3554 | Difficulty bomb delay | Push to December 2021 |

### Paris / The Merge (Block 15,537,394 - September 15, 2022)

| EIP | Name | Description |
|-----|------|-------------|
| EIP-3675 | Upgrade to Proof-of-Stake | Replace PoW with PoS. Remove mining, uncles |
| EIP-4399 | DIFFICULTY -> PREVRANDAO | On-chain randomness from beacon chain |

### Shanghai (April 12, 2023)

| EIP | Name | Description |
|-----|------|-------------|
| EIP-3651 | Warm COINBASE | Reduce gas for MEV builder interactions |
| EIP-3855 | PUSH0 | Push zero onto stack (saves gas) |
| EIP-3860 | Limit and meter initcode | Max 49,152 bytes, gas per chunk |
| EIP-4895 | Beacon chain withdrawals | Validators can withdraw staked ETH |
| EIP-6049 | Deprecate SELFDESTRUCT | Formal deprecation notice |

### Cancun / Dencun (March 13, 2024)

| EIP | Name | Description |
|-----|------|-------------|
| EIP-4844 | Proto-Danksharding (Blob tx) | Type 3 tx with temporary blob data for L2 rollups |
| EIP-1153 | Transient storage (TSTORE/TLOAD) | Auto-cleared per-transaction storage |
| EIP-4788 | Beacon block root in EVM | System contract exposing consensus state |
| EIP-5656 | MCOPY | Efficient memory-to-memory copy |
| EIP-6780 | Restrict SELFDESTRUCT | Only works in same-tx contract creation |
| EIP-7516 | BLOBBASEFEE opcode | On-chain blob fee access |

### Prague / Pectra (May 7, 2025)

| EIP | Name | Description |
|-----|------|-------------|
| EIP-2537 | BLS12-381 precompile | Native BLS curve operations |
| EIP-2935 | Historical block hashes from state | ~8191 blocks accessible via system contract |
| EIP-6110 | Validator deposits on chain | Faster deposit processing (~13 min) |
| EIP-7002 | EL triggerable withdrawals | Exit validators from smart contracts |
| EIP-7251 | Increase MAX_EFFECTIVE_BALANCE | 32 ETH -> 2,048 ETH per validator |
| EIP-7549 | Committee index outside Attestation | 60x attestation aggregation improvement |
| EIP-7623 | Increase calldata cost | Push rollups toward blob usage |
| EIP-7685 | General purpose EL requests | Standard EL<->CL communication |
| EIP-7691 | Blob throughput increase | Target 6 blobs/block (was 3), max 9 (was 6) |
| EIP-7702 | Set EOA account code | EOAs delegate to smart contract code. Type 0x04 tx. Batch/sponsor/session keys |
| EIP-7840 | Blob schedule in EL config | Configurable blob params |

### Fusaka (December 3, 2025) -- NOT YET IN MEOWCHAIN

| EIP | Name | Description | Priority |
|-----|------|-------------|----------|
| EIP-7594 | PeerDAS | Data availability sampling for blobs | HIGH |
| EIP-7642 | History Expiry | Safe pruning of old chain data | MEDIUM |
| EIP-7823 | MODEXP Bounds | Cost limits for modexp precompile | LOW |
| EIP-7825 | Transaction Gas Limit Cap | Hard cap ~16.8M gas per tx | MEDIUM |
| EIP-7883 | MODEXP Gas Cost Increase | Adjusted gas pricing | LOW |
| EIP-7892 | Blob Parameter Only Hardforks | Adjust blobs without full upgrade | MEDIUM |
| EIP-7917 | Deterministic Proposer Lookahead | Predictable proposer sets | LOW |
| EIP-7918 | Blob Base Fee Floor | Reserve price for blob fees | LOW |
| EIP-7934 | RLP Block Size Limit | Cap at 10 MiB per block | MEDIUM |
| EIP-7935 | Default Gas Limit 60M | Double throughput | HIGH |
| EIP-7939 | CLZ Opcode | Count leading zeros for 256-bit | LOW |
| EIP-7951 | secp256r1 Precompile | Native WebAuthn/passkey support | HIGH |

---

## 7. ERC Standards Support

> ERCs are smart contract standards. They work automatically on any EVM-compatible chain - **no special chain-level support needed** for most of them. The EVM executes them as regular bytecode.

### Tier 1: Core Token Standards (Automatic - EVM handles these)

| ERC | Name | Status on Meowchain | Notes |
|-----|------|---------------------|-------|
| ERC-20 | Fungible Tokens | Supported (EVM native) | USDC, USDT, WETH, DAI pattern |
| ERC-721 | NFTs | Supported (EVM native) | Unique tokens, `ownerOf`, `safeTransferFrom` |
| ERC-1155 | Multi-Token | Supported (EVM native) | Batch operations, gaming assets |
| ERC-165 | Interface Detection | Supported (EVM native) | `supportsInterface()` |

### Tier 2: Account Abstraction & Modern Wallets

| ERC | Name | Status on Meowchain | Notes |
|-----|------|---------------------|-------|
| ERC-4337 | Account Abstraction (Alt Mempool) | EntryPoint v0.7 PRE-DEPLOYED in genesis | `0x0000000071727De22E5E9d8BAf0edAc6f37da032`. Still needs Bundler service. |
| EIP-7702 | EOA Account Code | Supported (Prague active) | Type 0x04 tx enabled at genesis |
| ERC-7579 | Modular Smart Accounts | Needs contract deployment | Plugin architecture for smart wallets |
| ERC-1271 | Contract Signature Validation | Supported (EVM native) | `isValidSignature()` |

### Tier 3: DeFi Standards

| ERC | Name | Status on Meowchain | Notes |
|-----|------|---------------------|-------|
| ERC-2612 | Permit (Gasless Approvals) | Supported (EVM native) | Requires EIP-712 typed data |
| ERC-4626 | Tokenized Vaults | Supported (EVM native) | Standard vault interface for DeFi |
| ERC-2981 | NFT Royalties | Supported (EVM native) | `royaltyInfo()` |
| ERC-6551 | Token Bound Accounts | Supported (EVM native) | NFTs own wallets |
| ERC-777 | Enhanced Tokens | Supported (EVM native) | Hooks on send/receive (reentrancy risk) |

### Tier 4: Infrastructure ERCs

| ERC | Name | Status on Meowchain | Notes |
|-----|------|---------------------|-------|
| EIP-712 | Typed Structured Data Signing | Supported (EVM native) | Used by permit, 4337, 8004 |
| EIP-155 | Replay Protection | Supported | Chain ID in tx signatures |
| ERC-1820 | Interface Registry | Needs deployment | Universal registry contract |
| ERC-173 | Contract Ownership | Supported (EVM native) | `owner()`, `transferOwnership()` |
| ERC-2771 | Meta Transactions | Supported (EVM native) | Trusted forwarder pattern |

### Tier 5: Emerging Standards (2025-2026)

| ERC | Name | Status on Meowchain | Action Required |
|-----|------|---------------------|-----------------|
| **ERC-8004** | Trustless AI Agents | Needs deployment | **See Section 8 below** |
| ERC-6900 | Modular Smart Accounts | Needs deployment | Alternative to ERC-7579 |

### What Meowchain Needs to Deploy for Full ERC Ecosystem

```
Priority 1 (Essential):
  - [x] ERC-4337 EntryPoint contract (v0.7) -- PRE-DEPLOYED IN GENESIS at 0x0000000071727De22E5E9d8BAf0edAc6f37da032
  - [ ] ERC-4337 Bundler service (off-chain component, not a contract)
  - [ ] ERC-4337 Paymaster contracts (for gasless tx)
  - [x] WETH (Wrapped ETH) contract -- PRE-DEPLOYED IN GENESIS at 0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
  - [x] Multicall3 contract (batch reads) -- PRE-DEPLOYED IN GENESIS at 0xcA11bde05977b3631167028862bE2a173976CA11
  - [x] CREATE2 Deployer (deterministic addresses) -- PRE-DEPLOYED IN GENESIS at 0x4e59b44847b379578588920cA78FbF26c0B4956C
  - [x] SimpleAccountFactory (ERC-4337 wallet factory) -- PRE-DEPLOYED IN GENESIS at 0x9406Cc6185a346906296840746125a0E44976454
  - [ ] ERC-1820 Registry

Priority 2 (Ecosystem Growth):
  - [ ] ERC-8004 registries (Identity, Reputation, Validation)
  - [ ] Uniswap V3/V4 or equivalent DEX
  - [ ] Chainlink oracle contracts (or equivalent)
  - [ ] ENS-equivalent naming system

Priority 3 (Developer Experience):
  - [ ] Hardhat/Foundry verification support
  - [ ] Sourcify integration
  - [ ] Standard proxy patterns (ERC-1967 transparent, UUPS)
```

---

## 8. ERC-8004: Trustless AI Agents

> **Status:** Draft | **Live on Ethereum Mainnet:** January 29, 2026
> **Purpose:** On-chain infrastructure for autonomous AI agents to discover, interact, and trust each other without pre-existing trust relationships.

### What It Does

ERC-8004 extends Google's Agent-to-Agent (A2A) protocol with an **on-chain trust layer**. Three registries:

### 8.1 Identity Registry (Built on ERC-721)

```
Each AI agent gets:
- Globally unique ID: {namespace}:{chainId}:{registryAddress}
- NFT-based identity (transferable, browseable)
- agentURI -> registration JSON containing:
  - Name, description
  - Service endpoints (A2A, MCP, ENS, DID, email, web)
  - Supported trust models
  - x402 payment support indicator
  - Multi-chain entries
```

### 8.2 Reputation Registry

```
- giveFeedback() callable by any address
- Fixed-point ratings (int128) with configurable decimals
- Tag-based filtering (tag1, tag2)
- Off-chain detail URIs with KECCAK-256 integrity hashing
- Response/dispute mechanism
- Immutable on-chain (revocation only flags, doesn't delete)
```

### 8.3 Validation Registry

```
- Generic hooks for independent verification of agent work
- Supported verification methods:
  - Stake-secured re-execution validators
  - Zero-knowledge ML (zkML) proofs
  - TEE (Trusted Execution Environment) oracles
  - Custom validator contracts
- Flow: validationRequest() -> validationResponse()
- Responses on 0-100 scale with evidence URIs
```

### Dependencies for ERC-8004 on Meowchain

```
Required:
  - [ ] EIP-155 (chain ID) -- DONE
  - [ ] EIP-712 (typed data signing) -- DONE (EVM native)
  - [ ] ERC-721 (NFT) -- DONE (EVM native)
  - [ ] ERC-1271 (contract signatures) -- DONE (EVM native)

Deploy:
  - [ ] Identity Registry contract
  - [ ] Reputation Registry contract
  - [ ] Validation Registry contract
  - [ ] Agent Wallet management integration
  - [ ] A2A protocol endpoint on chain RPC
```

### Ecosystem Building on ERC-8004

| Project | What It Does |
|---------|-------------|
| Unibase | Persistent memory storage tied to agent identities |
| x402 Protocol | Agent-to-agent payments |
| ETHPanda | Community tooling for trustless agents |

---

## 9. Upcoming Ethereum Upgrades

### Fusaka (December 3, 2025) -- MEOWCHAIN NEEDS THIS

**Headline features:**
- **PeerDAS (EIP-7594):** Nodes sample blob data instead of downloading all. Massive DA scaling
- **secp256r1 precompile (EIP-7951):** Native WebAuthn/passkey support
- **60M gas limit (EIP-7935):** Double throughput
- **Transaction gas cap (EIP-7825):** Prevents single-tx DoS

**Action for Meowchain:**
```
- [ ] Update Reth dependency to include Fusaka support
- [ ] Add fusakaTime to chain config
- [ ] Deploy any new Fusaka system contracts
- [ ] Test all 12 Fusaka EIPs
- [ ] Update chainspec.rs hardfork list
```

### Glamsterdam (Targeted: May/June 2026) -- PLAN AHEAD

**Confirmed:**
- **EIP-7732: Enshrined Proposer-Builder Separation (ePBS)** -- Protocol-level PBS, eliminates MEV-Boost relay dependency
- **EIP-7928: Block-level Access Lists** -- Gas efficiency optimization
- Parallel EVM execution under discussion

**Action for Meowchain:**
```
- [ ] Monitor Glamsterdam EIP finalization
- [ ] Plan ePBS integration (or skip if POA makes it irrelevant)
- [ ] Implement upgrade scheduling mechanism before this ships
```

### Hegota (Targeted: Late 2026) -- LONG-TERM

**Leading candidates:**
- **Verkle Trees:** Replace Merkle Patricia Tries. 10x smaller proofs, enables stateless clients
- **State/History Expiry:** Archive old data, prevent state bloat
- **EVM Optimizations:** Faster/cheaper execution
- Targeting 180M gas limit

### Ethereum Roadmap Pillars (2027+)

| Pillar | Focus | Key Tech |
|--------|-------|----------|
| The Surge | 100,000+ TPS | Full Danksharding, ZK-EVM |
| The Scourge | MEV mitigation | Encrypted mempools, inclusion lists |
| The Verge | Statelessness | Verkle trees, stateless clients |
| The Purge | State cleanup | State expiry, EVM simplification |
| The Splurge | Everything else | Account abstraction, VDFs |

---

## 10. Production Infrastructure Checklist

### Block Explorer

| Solution | Status | Notes |
|----------|--------|-------|
| Blockscout (via Scoutup) | Partially done | Go wrapper exists, needs full integration |
| Contract verification | Not done | Need Sourcify or Blockscout verification API |
| Token tracking | Not done | ERC-20/721/1155 indexing |
| Internal tx tracing | Not done | Requires debug_traceTransaction RPC |

### Bridges

| Feature | Status | Options |
|---------|--------|---------|
| Bridge to Ethereum mainnet | Not done | Chainlink CCIP, LayerZero, Hyperlane, custom |
| Bridge to other L2s | Not done | Across, Wormhole |
| Canonical bridge contract | Not done | Lock-and-mint or burn-and-mint |
| Bridge UI | Not done | Frontend for bridging |

### Oracles

| Feature | Status | Options |
|---------|--------|---------|
| Price feeds | Not done | Chainlink, Pyth, Chronicle, Redstone |
| VRF (verifiable randomness) | Not done | Chainlink VRF |
| Automation/Keepers | Not done | Chainlink Automation |
| Data feeds for AI agents | Not done | Custom oracle for ERC-8004 |

### MEV Protection

| Feature | Status | Relevance |
|---------|--------|-----------|
| MEV-Boost | Not needed | POA signers control ordering |
| Fair ordering | Partially done | Round-robin signers provide basic fairness |
| Encrypted mempool | Not done | Prevent frontrunning by signers |
| PBS (Proposer-Builder Separation) | Not needed for POA | May matter if transitioning to PoS |

### Data Availability (if operating as L2)

| Solution | Status | Notes |
|----------|--------|-------|
| Ethereum blobs (EIP-4844) | Supported at EVM level | Need sequencer to post blobs |
| Celestia | Not integrated | Alternative DA |
| EigenDA | Not integrated | Restaking-secured DA |

### Wallet & Key Infrastructure

| Feature | Status | Notes |
|---------|--------|-------|
| MetaMask support | UNBLOCKED | External RPC on 0.0.0.0:8545 is live. MetaMask can connect via `Add Network` with chain ID 9323310. Needs testing. |
| WalletConnect | Not done | Needs chain registry listing |
| Hardware wallet signing | Not done | Ledger/Trezor for signers |
| Faucet | Not done | Testnet token distribution |

### Developer Experience

| Feature | Status | Notes |
|---------|--------|-------|
| Hardhat config template | Not done | Network config + verification |
| Foundry config template | Not done | `foundry.toml` with chain RPC |
| Subgraph support (The Graph) | Not done | Event indexing |
| SDK / client library | Not done | TypeScript/Python wrappers |
| Documentation site | Not done | API docs, tutorials |

---

## Priority Execution Order

```
Phase 0 - Fix the Foundation:
  0a. Replace NodeConfig::test() with NodeConfig::default()               -- DONE
  0b. Replace testing_node_with_datadir() with proper node builder        -- DONE (production NodeBuilder + init_db + MDBX)
  0c. Create custom PoaNode type that injects PoaConsensus into pipeline  -- DONE (node.rs)
  0d. Build PoaPayloadBuilder that signs blocks with signer keys          -- NOT DONE (biggest remaining gap)
  0e. Wire signer.rs into block production (round-robin turn detection)   -- PARTIALLY DONE (monitoring only, not signing)
  0f. Set difficulty field (1=in-turn, 2=out-of-turn) in produced blocks  -- NOT DONE (requires PoaPayloadBuilder)
  0g. Embed signer list in extra_data at epoch blocks                     -- NOT DONE (helpers exist, not wired)
  0h. Verify POA signatures on blocks received from peers                 -- DONE (validate_header calls recover_signer in production)
  0i. EIP-1967 miner proxy for anonymous block reward collection          -- DONE (coinbase = 0x...1967)
  -> STATUS: ~70% complete. Node is real, consensus validates signatures, rewards to proxy.

Phase 1 - Make It Connectable:
  1. Add CLI argument parsing (clap)                                      -- DONE
  2. Implement `meowchain init --genesis genesis.json` subcommand         -- NOT DONE
  3. Add external HTTP/WS RPC server                                      -- DONE
  4. Resolve chain ID inconsistencies (pick ONE: 9323310)                 -- DONE (all fixed including sample-genesis.json)
  5. Fix pre-existing test failures                                       -- DONE (70 tests passing)
  6. Generate canonical genesis.json for distribution                     -- DONE (auto-regenerated from code)
  -> STATUS: ~85% complete.

Phase 2 - Make It Multi-Node:
  7. Set up 2-3 bootnodes with static enode URLs                          -- NOT DONE
  8. Test 3-signer network (3 machines, each with one key)                -- NOT DONE
  9. Implement state sync (full sync from genesis for new joiners)        -- NOT DONE
  10. Implement fork choice rule (heaviest chain / most in-turn blocks)   -- NOT DONE
  11. Key management: load signer key from file at runtime                -- DONE (--signer-key / SIGNER_KEY)
  12. Integration test: multi-node block production + validation          -- NOT DONE
  -> STATUS: ~15% complete.

Phase 3 - Make It Production:
  13. Implement signer voting (clique_propose RPC)                        -- NOT DONE
  14. Add admin/debug/txpool/clique RPC namespaces                        -- NOT DONE
  15. Add Prometheus metrics + Grafana dashboards                         -- NOT DONE
  16. Implement chain recovery tooling (export/import blocks, db repair)  -- NOT DONE
  17. Implement post-execution validation (state root, receipt root)      -- DONE (gas_used, receipt root, logs bloom)
  18. Set up CI/CD pipeline                                               -- NOT DONE
  19. Encrypted keystore (EIP-2335 style)                                 -- NOT DONE
  20. Security audit                                                      -- NOT DONE
  -> STATUS: ~10% complete.

Phase 4 - Make It Ecosystem:
  21. Deploy core contracts (WETH, Multicall3, CREATE2, EntryPoint)       -- DONE (pre-deployed in genesis!)
  22. Full Blockscout integration with contract verification              -- NOT DONE (Scoutup wrapper exists)
  23. Bridge to Ethereum mainnet                                          -- NOT DONE
  24. Deploy ERC-8004 registries (AI Agent support)                       -- NOT DONE
  25. Oracle integration (Chainlink/Pyth)                                 -- NOT DONE
  26. Faucet + developer docs + SDK                                      -- NOT DONE
  27. Add Fusaka hardfork support                                         -- NOT DONE
  28. Wallet integrations (MetaMask, WalletConnect)                       -- UNBLOCKED (RPC exists, needs testing)
  -> STATUS: ~10% complete.
```

---

## 11. Codebase Issues Found During Audit

> Issues discovered during the 2026-02-12 code review that need attention.

### Critical Issues

| # | Issue | File | Details |
|---|-------|------|---------|
| C1 | **`testing_node_with_datadir()` still used** | ~~`main.rs:219`~~ | **FIXED** - Now uses production `NodeBuilder::new(config).with_database(init_db()).with_launch_context(executor)` with persistent MDBX database. |
| C2 | **Block monitoring logs but doesn't sign** | `main.rs:244-283` | The spawned task detects which signer should sign each block and logs it, but `BlockSealer.seal_header()` is never called. Requires `PoaPayloadBuilder` integration. |
| C3 | **`validate_header()` doesn't verify signatures** | ~~`consensus.rs`~~ | **FIXED** - Production mode calls `recover_signer()` and `validate_signer()` to verify block signatures. Dev mode skips (unsigned blocks). |
| C4 | **`validate_block_pre_execution()` silently allows invalid extra_data** | ~~`consensus.rs`~~ | **FIXED** - Production mode rejects blocks with extra_data shorter than vanity+seal. Dev mode allows (unsigned blocks from Reth dev mining). |

### Non-Critical Issues

| # | Issue | File | Details |
|---|-------|------|---------|
| N1 | **`sample-genesis.json` is stale** | ~~`sample-genesis.json`~~ | **FIXED** - Regenerated from code with chain ID 9323310, all 30 alloc entries (20 dev + 4 system + 5 infra + 1 miner proxy). |
| N2 | **Dockerfile CMD format mismatch** | ~~`Dockerfile`~~ | **FIXED** - CMD uses correct `--http-addr`, `--http-port`, `--ws-addr`, `--ws-port` format. |
| N3 | **Dockerfile copies wrong binary name** | ~~`Dockerfile`~~ | **FIXED** - Copies `target/release/example-custom-poa-node` and renames to `meowchain`. |
| N5 | **Production config uses dev account keys** | `genesis.rs:130` | Still uses `dev_accounts()[0..5]` as signers. Real production MUST use unique keys. |
| N7 | **Double block stream subscription** | ~~`main.rs`~~ | **FIXED** - Single `canonical_state_stream()` subscription. |

### Suggestions for Next Steps

1. **Highest priority:** Implement `PoaPayloadBuilder` - this is the single biggest gap. Without it, blocks are unsigned and the chain is functionally identical to a vanilla Ethereum dev node.

2. **Second priority:** Wire `BlockSealer` into the payload building pipeline for block signing during production.

3. **Third priority:** Encrypted keystore support (EIP-2335) for production signer key management.

---

*Last updated: 2026-02-13 | Meowchain Custom POA on Reth*
*Tracks: All finalized EIPs through Fusaka + planned Glamsterdam/Hegota*
