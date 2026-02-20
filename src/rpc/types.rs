use alloy_primitives::Address;
use serde::Serialize;

/// Response for `meow_chainConfig`
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainConfigResponse {
    pub chain_id: u64,
    pub gas_limit: u64,
    pub block_time: u64,
    pub epoch: u64,
    pub signer_count: usize,
    pub governance_safe: Address,
    pub chain_config_contract: Address,
    pub signer_registry_contract: Address,
    pub treasury_contract: Address,
}

/// Response for `meow_nodeInfo`
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfoResponse {
    pub chain_id: u64,
    pub dev_mode: bool,
    pub signer_count: usize,
    pub local_signer_count: usize,
    pub local_signers: Vec<Address>,
    pub authorized_signers: Vec<Address>,
}
