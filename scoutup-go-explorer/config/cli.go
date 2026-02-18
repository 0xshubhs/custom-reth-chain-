package config

import "github.com/urfave/cli/v2"

const (
	Anvil                = "anvil"
	Supersim             = "supersim"
	SupersimAdminRpc     = "supersim.admin.rpc"
	StartingFrontendPort = "frontend.starting.port"
	StartingBackendPort  = "backend.starting.port"
	StartingPostgresPort = "postgres.starting.port"
)

func BaseCLIFlags() []cli.Flag {
	return []cli.Flag{
		&cli.BoolFlag{
			Name:  Anvil,
			Value: false,
			Usage: "Runs single blockscout instance for default anvil config",
		},
		&cli.BoolFlag{
			Name:  Supersim,
			Value: false,
			Usage: "Fetches supersim config and runs blockscout instances for each chain",
		},
		&cli.StringFlag{
			Name:  SupersimAdminRpc,
			Value: "http://localhost:8420",
			Usage: "Admin RPC URL for supersim",
		},
		&cli.Uint64Flag{
			Name:  StartingFrontendPort,
			Value: 3000,
			Usage: "Starting port to increment for frontend containers",
		},
		&cli.Uint64Flag{
			Name:  StartingBackendPort,
			Value: 4000,
			Usage: "Starting port to increment for backend containers",
		},
		&cli.Uint64Flag{
			Name:  StartingPostgresPort,
			Value: 7432,
			Usage: "Starting port to increment for postgres containers",
		},
	}
}
