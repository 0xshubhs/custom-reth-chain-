use alloy_primitives::Address;
use jsonrpsee::{core::RpcResult, proc_macros::rpc};

use super::types::{ChainConfigResponse, NodeInfoResponse};

/// The `meow_*` RPC namespace definition.
#[rpc(server, namespace = "meow")]
pub trait MeowApi {
    /// Returns the current chain configuration parameters.
    #[method(name = "chainConfig")]
    async fn chain_config(&self) -> RpcResult<ChainConfigResponse>;

    /// Returns the list of authorized POA signers.
    #[method(name = "signers")]
    async fn signers(&self) -> RpcResult<Vec<Address>>;

    /// Returns node information including local signer status.
    #[method(name = "nodeInfo")]
    async fn node_info(&self) -> RpcResult<NodeInfoResponse>;
}
