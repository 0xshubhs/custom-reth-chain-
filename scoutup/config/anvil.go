package config

func PrepareDefaultAnvilConfig() *NetworkConfig {
	return &NetworkConfig{
		Chains: []*ChainConfig{
			{
				Name:       "Meowchain",
				RPCUrl:     "http://host.docker.internal:8545",
				FirstBlock: 0,
				ChainID:    9323310,
			},
		},
	}
}
