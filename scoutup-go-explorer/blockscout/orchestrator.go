package blockscout

import (
	"context"
	"fmt"
	"strings"

	"github.com/blockscout/scoutup/config"
	"github.com/ethereum/go-ethereum/log"
)

type Orchestrator struct {
	instances []*Instance

	log log.Logger
}

func NewOrchestrator(log log.Logger, closeApp context.CancelCauseFunc, configs []*config.BlockscoutConfig) (*Orchestrator, error) {
	globalWorkspace, err := createGlobalWorkspace()
	if err != nil {
		return nil, err
	}

	instances := []*Instance{}
	for _, config := range configs {
		instance, err := NewInstance(log, closeApp, config, globalWorkspace)
		if err != nil {
			return nil, err
		}
		instances = append(instances, instance)
	}
	return &Orchestrator{instances: instances, log: log}, nil
}

func (o *Orchestrator) Start(ctx context.Context) error {
	for _, instance := range o.instances {
		if err := instance.Start(ctx); err != nil {
			o.log.Error("Failed to start Blockscout instance", "err", err)
			return err
		}
	}

	o.log.Info(o.ConfigAsString())
	return nil
}

func (o *Orchestrator) Stop(ctx context.Context) error {
	for _, instance := range o.instances {
		instance.Stop(ctx)
	}
	return nil
}

func (o *Orchestrator) ConfigAsString() string {
	var b strings.Builder
	fmt.Fprintln(&b, "\nBlockscout Config:")
	fmt.Fprintln(&b, "------------------")
	for _, instance := range o.instances {
		fmt.Fprintln(&b, instance.ConfigAsString())
	}
	return b.String()
}

// no-op dead code in the cliapp lifecycle
func (b *Orchestrator) Stopped() bool {
	return false
}
