use clap::Parser;
use std::path::PathBuf;

/// CLI arguments for the POA node
#[derive(Parser, Debug)]
#[command(name = "meowchain", about = "Meowchain POA Node")]
pub struct Cli {
    /// Chain ID for the network
    #[arg(long, default_value = "9323310")]
    pub chain_id: u64,

    /// Block production interval in seconds
    #[arg(long, default_value = "2")]
    pub block_time: u64,

    /// Data directory for chain storage
    #[arg(long, default_value = "data")]
    pub datadir: PathBuf,

    /// HTTP RPC listen address
    #[arg(long, default_value = "0.0.0.0")]
    pub http_addr: String,

    /// HTTP RPC port
    #[arg(long, default_value = "8545")]
    pub http_port: u16,

    /// WebSocket RPC listen address
    #[arg(long, default_value = "0.0.0.0")]
    pub ws_addr: String,

    /// WebSocket RPC port
    #[arg(long, default_value = "8546")]
    pub ws_port: u16,

    /// Signer private key (hex, without 0x prefix).
    /// Can also be set via SIGNER_KEY environment variable.
    #[arg(long, env = "SIGNER_KEY")]
    pub signer_key: Option<String>,

    /// Use production genesis configuration (chain ID 9323310)
    #[arg(long)]
    pub production: bool,

    /// Disable dev mode (no auto-mining)
    #[arg(long)]
    pub no_dev: bool,

    /// Override block gas limit (e.g., 100000000 for 100M, 1000000000 for 1B)
    #[arg(long)]
    pub gas_limit: Option<u64>,

    /// Enable eager mining: build block immediately when transactions arrive
    /// instead of waiting for block-time interval
    #[arg(long)]
    pub eager_mining: bool,

    /// Force interval-based block production even in production mode.
    /// Useful for testing: node uses production signing (97-byte extra_data, strict POA)
    /// but still auto-mines blocks at --block-time interval.
    #[arg(long)]
    pub mining: bool,

    /// P2P listener port for peer-to-peer connections.
    #[arg(long, default_value = "30303")]
    pub port: u16,

    /// Comma-separated bootnode enode URLs for peer discovery.
    /// Example: enode://pubkey@ip:port,enode://pubkey2@ip2:port2
    #[arg(long, value_delimiter = ',')]
    pub bootnodes: Option<Vec<String>>,

    /// Disable P2P peer discovery (useful for single-node testing).
    #[arg(long)]
    pub disable_discovery: bool,
}
