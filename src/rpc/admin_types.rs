use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node version string for meowchain
pub const NODE_VERSION: &str = "meowchain/v0.1.0";

/// Response for `admin_nodeInfo`
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminNodeInfo {
    /// The node's enode URL
    pub enode: String,
    /// Node ID (public key)
    pub id: String,
    /// Node name / client version
    pub name: String,
    /// IP address the node is listening on
    pub ip: String,
    /// TCP port for P2P
    pub ports: AdminPorts,
    /// Listening address
    pub listen_addr: String,
    /// Protocol information
    pub protocols: AdminProtocols,
}

/// Port information
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminPorts {
    pub discovery: u16,
    pub listener: u16,
}

/// Protocol information
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminProtocols {
    pub eth: AdminEthProtocol,
}

/// Eth protocol info
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminEthProtocol {
    /// Network ID
    pub network: u64,
    /// Difficulty (always 1 for POA)
    pub difficulty: u64,
    /// Genesis block hash
    pub genesis: String,
    /// Chain configuration
    pub config: AdminChainConfig,
    /// Current head block hash
    pub head: String,
}

/// Chain config in admin response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminChainConfig {
    pub chain_id: u64,
    /// POA specific
    pub clique: AdminCliqueConfig,
}

/// Clique config in admin response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminCliqueConfig {
    pub period: u64,
    pub epoch: u64,
}

/// Response for `admin_peers`
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminPeerInfo {
    /// Peer's enode URL
    pub enode: String,
    /// Peer ID
    pub id: String,
    /// Peer name/client
    pub name: String,
    /// Network addresses
    pub network: AdminPeerNetwork,
    /// Protocol versions
    pub protocols: HashMap<String, String>,
}

/// Peer network info
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminPeerNetwork {
    pub local_address: String,
    pub remote_address: String,
}

/// Response for health check
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthStatus {
    /// Whether the node is healthy
    pub healthy: bool,
    /// Whether the node is syncing
    pub syncing: bool,
    /// Current block number
    pub block_number: u64,
    /// Number of connected peers
    pub peer_count: usize,
    /// Number of authorized signers
    pub signer_count: usize,
    /// Whether this node is an active signer
    pub is_signer: bool,
    /// Node uptime in seconds
    pub uptime_seconds: u64,
    /// Node version
    pub version: String,
}

/// Request type for admin_addPeer
#[derive(Debug, Clone, Deserialize)]
pub struct AddPeerRequest {
    pub enode: String,
}
