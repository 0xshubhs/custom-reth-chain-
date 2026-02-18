# Meowchain - Usage Guide

Custom POA blockchain on Reth. Chain ID **9323310**, all hardforks through Prague.

## Directory Structure

```
custom-reth-chain-/
├── src/                        # Rust source code
│   ├── main.rs                 # Entry point, CLI, block monitoring
│   ├── node.rs                 # PoaNode (custom node type)
│   ├── consensus.rs            # PoaConsensus (validation, signatures)
│   ├── chainspec.rs            # PoaChainSpec (hardforks, POA config)
│   ├── genesis.rs              # Genesis builder (dev + production)
│   ├── payload.rs              # PoaPayloadBuilder (block signing)
│   ├── onchain.rs              # StorageReader (on-chain contract reads)
│   ├── rpc.rs                  # meow_* RPC namespace
│   ├── signer.rs               # SignerManager + BlockSealer
│   └── bytecodes/              # Pre-compiled contract bytecodes (16 .bin/.hex)
├── genesis/                    # Genesis JSON files
│   ├── sample-genesis.json     # Dev genesis (chain ID 9323310, 37 alloc entries)
│   └── production-genesis.json # Production genesis (25 alloc entries)
├── genesis-contracts/          # Governance Solidity contracts
│   ├── ChainConfig.sol         # Dynamic chain parameters
│   ├── SignerRegistry.sol      # POA signer management
│   └── Treasury.sol            # Fee distribution
├── Docker/                     # Docker build artifacts
│   ├── Dockerfile              # Multi-stage build
│   └── docker-compose.yml      # Single-node compose
├── scoutup-go-explorer/        # Blockscout explorer integration
├── signatures/                 # Contract ABI signatures
│   ├── signatures-contracts.json
│   └── signatures-contracts.txt
├── md/                         # Documentation
│   ├── Remaining.md            # Status tracker + roadmap
│   ├── main.md                 # Strategy notes
│   └── USAGE.md                # This file
├── CLAUDE.md                   # AI context (architecture, pitfalls)
├── Justfile                    # Build automation
└── Cargo.toml
```

## Quick Start

```bash
# Build (fetches latest reth from main branch + compiles release)
just build

# Run in dev mode (auto-mines every 2s, 3 signers, 20 prefunded accounts)
just dev

# Run tests (187 passing)
just test
```

## CLI Flags

```bash
cargo run --release -- [FLAGS]

# Key flags:
--production            # Enable production mode (strict POA signature verification)
--no-dev                # Disable dev mode (requires --production for manual control)
--block-time <N>        # Block interval in seconds (default: 2 in dev, 12 in prod)
--gas-limit <N>         # Override block gas limit (default: 30M dev, 60M prod)
--eager-mining          # Mine immediately on tx arrival (not just on interval)
--signer-key <HEX>      # Private key for block signing (64 hex chars, no 0x prefix)
--datadir <PATH>        # Database directory (default: ./data)
--http-addr <ADDR>      # HTTP RPC bind address (default: 0.0.0.0)
--http-port <PORT>      # HTTP RPC port (default: 8545)
--ws-addr <ADDR>        # WebSocket RPC bind address (default: 0.0.0.0)
--ws-port <PORT>        # WebSocket RPC port (default: 8546)
```

## Running Modes

### Dev Mode (default)

Auto-mines blocks, relaxed consensus (no signature verification), 20 prefunded accounts, 3 default signers:

```bash
just dev

# With custom signer key
SIGNER_KEY=ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 just dev

# With 1-second blocks
cargo run --release -- --block-time 1
```

### Production Mode

Strict POA signature verification, persistent MDBX, 5 signers:

```bash
just run-production

# With explicit signer key
cargo run --release -- --production --block-time 12 \
  --signer-key <YOUR_64_CHAR_HEX_KEY>
```

### Custom Args

```bash
just run-custom -- --block-time 4 --gas-limit 100000000 --eager-mining
```

## Chain Configuration

| Parameter | Dev | Production |
|-----------|-----|------------|
| Chain ID | 9323310 | 9323310 |
| Block time | 2s | 12s |
| Gas limit | 30M | 60M |
| Signers | 3 (dev accounts 0-2) | 5 (dev accounts 0-4) |
| Prefunded accounts | 20 @ 10,000 ETH | 8 (tiered) |
| Coinbase | EIP-1967 Miner Proxy | EIP-1967 Miner Proxy |
| Consensus | Relaxed (no sig check) | Strict POA signatures |

## RPC Endpoints

After node starts:
- **HTTP**: `http://localhost:8545` (or your `--http-addr:--http-port`)
- **WebSocket**: `ws://localhost:8546`

### Standard eth_* Methods

```bash
# Block number
curl -s http://localhost:8545 -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Chain ID (returns 0x8e5eee = 9323310)
curl -s http://localhost:8545 -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}'

# Balance
curl -s http://localhost:8545 -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_getBalance","params":["0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266","latest"],"id":1}'
```

### meow_* Custom RPC

```bash
# On-chain ChainConfig parameters
curl -s http://localhost:8545 -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"meow_chainConfig","params":[],"id":1}'

# Active signers
curl -s http://localhost:8545 -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"meow_signers","params":[],"id":1}'

# Node info
curl -s http://localhost:8545 -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"meow_nodeInfo","params":[],"id":1}'
```

## Testing with Foundry

```bash
# Check block number
cast block-number --rpc-url http://localhost:8545

# Check balance
cast balance 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 --rpc-url http://localhost:8545

# Send transaction
cast send \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 \
  --rpc-url http://localhost:8545 \
  0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  --value 1ether
```

## Prefunded Dev Accounts

From mnemonic: `test test test test test test test test test test test junk`

| # | Address | Private Key |
|---|---------|-------------|
| 0 | `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266` | `ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80` |
| 1 | `0x70997970C51812dc3A010C7d01b50e0d17dc79C8` | `59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d` |
| 2 | `0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC` | `5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a` |
| 3-19 | See `src/genesis.rs:dev_accounts()` | Standard Foundry dev keys |

Signers in dev mode: accounts 0, 1, 2 (first 3). In production: accounts 0-4 (first 5).

## Genesis Files

| File | Purpose |
|------|---------|
| `genesis/sample-genesis.json` | Dev genesis — 37 alloc entries, chain ID 9323310, all contracts |
| `genesis/production-genesis.json` | Production genesis — 25 alloc entries, 5 signers |

Regenerate from code:
```bash
just genesis
# runs: cargo test test_regenerate_sample_genesis
```

## Pre-deployed Contracts

All deployed at genesis. No deployment tx needed.

| Contract | Address |
|----------|---------|
| EIP-1967 Miner Proxy (coinbase) | `0x0000000000000000000000000000000000001967` |
| ERC-4337 EntryPoint v0.7 | `0x0000000071727De22E5E9d8BAf0edAc6f37da032` |
| WETH9 | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` |
| Multicall3 | `0xcA11bde05977b3631167028862bE2a173976CA11` |
| CREATE2 Deployer | `0x4e59b44847b379578588920cA78FbF26c0B4956C` |
| SimpleAccountFactory | `0x9406Cc6185a346906296840746125a0E44976454` |
| ChainConfig (governance) | `0x00000000000000000000000000000000C04F1600` |
| SignerRegistry (governance) | `0x000000000000000000000000000000005164EB00` |
| Treasury (governance) | `0x0000000000000000000000000000000007EA5B00` |
| Governance Safe (reserved) | `0x000000000000000000000000000000006F5AFE00` |
| Safe Singleton v1.3.0 | `0xd9Db270c1B5E3Bd161E8c8503c55cEABeE709552` |
| Safe Proxy Factory | `0xa6B71E26C5e0845f74c812102Ca7114b6a896AB2` |
| Safe Fallback Handler | `0xf48f2B2d2a534e402487b3ee7C18c33Aec0Fe5e4` |
| Safe MultiSend | `0xA238CBeb142c10Ef7Ad8442C6D1f9E89e07e7761` |

## Hardhat / Foundry Config

### Hardhat (`hardhat.config.ts`)

```typescript
networks: {
  meowchain: {
    url: "http://localhost:8545",
    chainId: 9323310,
    accounts: ["0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"],
  }
}
```

### Foundry (`foundry.toml`)

```toml
[profile.meowchain]
eth_rpc_url = "http://localhost:8545"
chain_id = 9323310
```

```bash
forge script ... --rpc-url http://localhost:8545 --chain-id 9323310 --broadcast
```

## MetaMask Setup

1. Open MetaMask → Settings → Networks → Add Network
2. Fill in:
   - **Network Name**: Meowchain
   - **RPC URL**: `http://localhost:8545`
   - **Chain ID**: `9323310`
   - **Currency Symbol**: `ETH`

## Docker

```bash
# Build image (Dockerfile is in Docker/ subdir)
docker build -f Docker/Dockerfile -t meowchain .

# Or use compose
docker compose -f Docker/docker-compose.yml up
```

## Data Directory

```
data/
├── db/                    # MDBX database
│   ├── mdbx.dat
│   └── mdbx.lck
├── static_files/          # Headers, txns, receipts
└── jwt.hex               # JWT secret
```

Clean restart:
```bash
rm -rf data/
```

## Troubleshooting

| Problem | Fix |
|---------|-----|
| Port 8545 in use | Kill other node: `pkill -f meowchain` or change `--http-port` |
| Database errors | `rm -rf data/` and restart |
| Blocks not mining | Ensure dev mode is on (no `--production`/`--no-dev` without intent) |
| Signer not producing blocks | Check `--signer-key` matches one of the registered signers |
| RPC not responding | Node may still be initializing; wait ~5s and retry |

*Last updated: 2026-02-18 | Chain ID 9323310 | reth 1.11.0 | 187 tests passing*
