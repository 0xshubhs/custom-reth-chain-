use alloy_primitives::keccak256;

/// Compute the Solidity function selector (first 4 bytes of keccak256(signature)).
pub fn function_selector(signature: &str) -> [u8; 4] {
    let hash = keccak256(signature.as_bytes());
    let mut selector = [0u8; 4];
    selector.copy_from_slice(&hash[..4]);
    selector
}

// ChainConfig getters
pub fn gas_limit() -> [u8; 4] {
    function_selector("gasLimit()")
}
pub fn block_time() -> [u8; 4] {
    function_selector("blockTime()")
}
pub fn max_contract_size() -> [u8; 4] {
    function_selector("maxContractSize()")
}
pub fn calldata_gas_per_byte() -> [u8; 4] {
    function_selector("calldataGasPerByte()")
}
pub fn max_tx_gas() -> [u8; 4] {
    function_selector("maxTxGas()")
}
pub fn eager_mining() -> [u8; 4] {
    function_selector("eagerMining()")
}
pub fn governance() -> [u8; 4] {
    function_selector("governance()")
}

// SignerRegistry getters
pub fn get_signers() -> [u8; 4] {
    function_selector("getSigners()")
}
pub fn signer_count() -> [u8; 4] {
    function_selector("signerCount()")
}
pub fn signer_threshold() -> [u8; 4] {
    function_selector("signerThreshold()")
}
pub fn is_signer() -> [u8; 4] {
    function_selector("isSigner(address)")
}
