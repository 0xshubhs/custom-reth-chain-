// Meowchain Hardhat Configuration
//
// A template for deploying and testing smart contracts on Meowchain.
//
// Setup:
//   npm init -y
//   npm install --save-dev hardhat @nomicfoundation/hardhat-toolbox
//   cp configs/hardhat.config.js hardhat.config.js
//
// Usage:
//   npx hardhat compile
//   npx hardhat test --network meowchain
//   npx hardhat run scripts/deploy.js --network meowchain
//   npx hardhat verify --network meowchain <contract-address>

require("@nomicfoundation/hardhat-toolbox");

// Dev accounts from the "test test..." mnemonic (first 10).
// These are pre-funded with 10,000 ETH each in dev mode.
// NEVER use these keys on mainnet or any chain with real value.
const DEV_ACCOUNTS = [
  "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80", // Signer 1
  "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d", // Signer 2
  "0x5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a", // Signer 3
  "0x7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6",
  "0x47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a",
  "0x8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba",
  "0x92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e",
  "0x4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356",
  "0xdbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97",
  "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6",
];

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: {
    version: "0.8.28",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
      },
      evmVersion: "cancun",
      // Meowchain supports contracts larger than EIP-170's 24KB limit
      // when the node is run with --max-contract-size
    },
  },

  networks: {
    // Local dev node (just dev)
    meowchain: {
      url: "http://localhost:8545",
      chainId: 9323310,
      accounts: DEV_ACCOUNTS,
      // Dev mode: 300M gas limit
      gas: 300_000_000,
      // EIP-1559 base fee starts at 0.875 gwei
      gasPrice: 875_000_000,
      // Long timeout for large contract deployments
      timeout: 60_000,
    },

    // Production mode (just run-production)
    meowchain_production: {
      url: process.env.MEOWCHAIN_RPC_URL || "http://localhost:8545",
      chainId: 9323310,
      // In production, load keys from environment variables
      accounts: process.env.DEPLOYER_PRIVATE_KEY
        ? [process.env.DEPLOYER_PRIVATE_KEY]
        : [],
      // Production mode: 1B gas limit
      gas: 1_000_000_000,
      timeout: 120_000,
    },

    // Multi-node RPC endpoint (docker-compose-multinode.yml)
    meowchain_multinode: {
      url: "http://localhost:8548",
      chainId: 9323310,
      accounts: DEV_ACCOUNTS,
      gas: 1_000_000_000,
      timeout: 60_000,
    },
  },

  // Blockscout explorer verification
  etherscan: {
    apiKey: {
      meowchain: process.env.BLOCKSCOUT_API_KEY || "no-key-needed",
      meowchain_production: process.env.BLOCKSCOUT_API_KEY || "no-key-needed",
    },
    customChains: [
      {
        network: "meowchain",
        chainId: 9323310,
        urls: {
          apiURL: "http://localhost:4000/api",
          browserURL: "http://localhost:4000",
        },
      },
      {
        network: "meowchain_production",
        chainId: 9323310,
        urls: {
          apiURL:
            process.env.BLOCKSCOUT_API_URL || "http://localhost:4000/api",
          browserURL:
            process.env.BLOCKSCOUT_URL || "http://localhost:4000",
        },
      },
    ],
  },

  // Pre-deployed system contracts on Meowchain
  // These are available at genesis -- do not redeploy.
  // See configs/networks.json for the full contract registry.
  //
  // Governance:
  //   ChainConfig:     0x00000000000000000000000000000000C04F1600
  //   SignerRegistry:  0x000000000000000000000000000000005164EB00
  //   Treasury:        0x0000000000000000000000000000000007EA5B00
  //   Timelock:        0x00000000000000000000000000000000714E4C00
  //
  // Infrastructure:
  //   EntryPoint v0.7: 0x0000000071727De22E5E9d8BAf0edAc6f37da032
  //   Multicall3:      0xcA11bde05977b3631167028862bE2a173976CA11
  //   WETH9:           0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2
  //   CREATE2:         0x4e59b44847b379578588920cA78FbF26c0B4956C

  paths: {
    sources: "./contracts",
    tests: "./test",
    cache: "./cache",
    artifacts: "./artifacts",
  },

  mocha: {
    timeout: 60_000,
  },
};
