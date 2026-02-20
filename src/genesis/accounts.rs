use alloy_primitives::{address, Address, U256};

/// Default balance for prefunded accounts (10,000 ETH in wei)
/// 10,000 ETH = 10,000 * 10^18 wei = 10,000,000,000,000,000,000,000 wei
pub fn default_prefund_balance() -> U256 {
    U256::from(10_000u64) * U256::from(10u64).pow(U256::from(18u64))
}

/// Standard dev mnemonic accounts (derived from "test test test test test test test test test test test junk")
pub fn dev_accounts() -> Vec<Address> {
    vec![
        address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266"),
        address!("70997970C51812dc3A010C7d01b50e0d17dc79C8"),
        address!("3C44CdDdB6a900fa2b585dd299e03d12FA4293BC"),
        address!("90F79bf6EB2c4f870365E785982E1f101E93b906"),
        address!("15d34AAf54267DB7D7c367839AAf71A00a2C6A65"),
        address!("9965507D1a55bcC2695C58ba16FB37d819B0A4dc"),
        address!("976EA74026E726554dB657fA54763abd0C3a0aa9"),
        address!("14dC79964da2C08b23698B3D3cc7Ca32193d9955"),
        address!("23618e81E3f5cdF7f54C3d65f7FBc0aBf5B21E8f"),
        address!("a0Ee7A142d267C1f36714E4a8F75612F20a79720"),
        address!("Bcd4042DE499D14e55001CcbB24a551F3b954096"),
        address!("71bE63f3384f5fb98995898A86B02Fb2426c5788"),
        address!("FABB0ac9d68B0B445fB7357272Ff202C5651694a"),
        address!("1CBd3b2770909D4e10f157cABC84C7264073C9Ec"),
        address!("dF3e18d64BC6A983f673Ab319CCaE4f1a57C7097"),
        address!("cd3B766CCDd6AE721141F452C550Ca635964ce71"),
        address!("2546BcD3c84621e976D8185a91A922aE77ECEc30"),
        address!("bDA5747bFD65F08deb54cb465eB87D40e51B197E"),
        address!("dD2FD4581271e230360230F9337D5c0430Bf44C0"),
        address!("8626f6940E2eb28930eFb4CeF49B2d1F2C9C1199"),
    ]
}

/// Default dev signers (first 3 accounts from dev mnemonic)
pub fn dev_signers() -> Vec<Address> {
    dev_accounts().into_iter().take(3).collect()
}
