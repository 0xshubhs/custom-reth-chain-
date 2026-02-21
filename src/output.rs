//! Colored console output for the Meowchain POA node.
//!
//! Replaces raw `println!` calls with structured, colored output.
//! Color scheme: blue+bold headers, cyan values, green success,
//! yellow warnings, dimmed secondary text.

use alloy_primitives::Address;
use colored::Colorize;
use std::fmt;
use std::path::Path;

// ── Banner & Identity ──────────────────────────────────────────────

/// Print the startup banner with chain identity.
pub fn print_banner(chain_id: u64, block_period: u64) {
    println!();
    println!("{}", "=== Meowchain POA Node ===".blue().bold());
    println!("  Chain ID:     {}", chain_id.to_string().cyan());
    println!(
        "  Block period: {} seconds",
        block_period.to_string().cyan()
    );
}

/// Print the operating mode.
pub fn print_mode(mode: &str) {
    println!("  Mode:         {}", mode.cyan());
}

// ── Signer Info ────────────────────────────────────────────────────

/// Print the authorized signer list.
pub fn print_signers(signers: &[Address]) {
    println!(
        "  Authorized signers ({}):",
        signers.len().to_string().cyan()
    );
    for (i, signer) in signers.iter().enumerate() {
        println!(
            "    {}. {}",
            (i + 1).to_string().dimmed(),
            format!("{signer}").cyan()
        );
    }
}

/// Print confirmation that a signer key was loaded.
pub fn print_signer_loaded(addr: &Address) {
    println!(
        "  {} Signer key loaded: {}",
        "OK".green().bold(),
        format!("{addr}").cyan()
    );
}

/// Print confirmation that dev signers were loaded.
pub fn print_dev_signers_loaded(count: usize) {
    println!(
        "  {} Dev signers loaded: {} keys",
        "OK".green().bold(),
        count.to_string().cyan()
    );
}

/// Print a warning when no signer key is provided.
pub fn print_no_signer_warning() {
    println!(
        "  {} No signer key provided. Node will validate but not produce blocks.",
        "WARNING:".yellow().bold()
    );
    println!(
        "  {}",
        "Set --signer-key or SIGNER_KEY environment variable.".dimmed()
    );
}

// ── Node Configuration ─────────────────────────────────────────────

/// Print the node configuration block.
#[allow(clippy::too_many_arguments)]
pub fn print_config(
    is_dev_mode: bool,
    mining_mode: &str,
    gas_limit: u64,
    http_addr: &str,
    http_port: u16,
    ws_addr: &str,
    ws_port: u16,
    p2p_port: u16,
    bootnode_count: Option<usize>,
    datadir: &Path,
) {
    println!();
    println!("{}", "Node configuration:".blue().bold());
    println!(
        "  {} {}",
        "Dev mode:   ".dimmed(),
        if is_dev_mode {
            "true".green()
        } else {
            "false".normal()
        }
    );
    println!("  {} {}", "Mining mode:".dimmed(), mining_mode.cyan());
    println!(
        "  {} {}",
        "Gas limit:  ".dimmed(),
        gas_limit.to_string().cyan()
    );
    println!(
        "  {} {}:{}",
        "HTTP RPC:   ".dimmed(),
        http_addr.cyan(),
        http_port.to_string().cyan()
    );
    println!(
        "  {} {}:{}",
        "WS RPC:     ".dimmed(),
        ws_addr.cyan(),
        ws_port.to_string().cyan()
    );
    println!(
        "  {} {}",
        "P2P port:   ".dimmed(),
        p2p_port.to_string().cyan()
    );
    if let Some(count) = bootnode_count {
        println!(
            "  {} {} configured",
            "Bootnodes:  ".dimmed(),
            count.to_string().cyan()
        );
    }
    println!("  {} {:?}", "Data dir:   ".dimmed(), datadir);
}

// ── RPC ────────────────────────────────────────────────────────────

/// Print that a custom RPC namespace was registered.
pub fn print_rpc_registered(namespace: &str) {
    println!(
        "  {} {} RPC namespace registered",
        "OK".green().bold(),
        namespace.cyan()
    );
}

// ── Node Lifecycle ─────────────────────────────────────────────────

/// Print that the node started successfully.
pub fn print_node_started(genesis_hash: impl fmt::Debug) {
    println!();
    println!("{}", "Node started successfully!".green().bold());
    println!("  Genesis hash: {:?}", genesis_hash);
}

/// Print prefunded accounts.
pub fn print_prefunded(accounts: &[Address]) {
    println!();
    println!("{}", "Prefunded accounts:".blue().bold());
    for (i, account) in accounts.iter().enumerate() {
        println!(
            "  {}. {}",
            (i + 1).to_string().dimmed(),
            format!("{account}").cyan()
        );
    }
}

/// Print chain data storage info and block period.
pub fn print_chain_data(datadir: &Path, block_period: u64) {
    println!();
    println!(
        "  Chain data stored in: {}",
        datadir.display().to_string().dimmed()
    );
    println!(
        "  Blocks produced every {} seconds (POA interval mining)",
        block_period.to_string().cyan()
    );
}

/// Print the final "running" message with RPC URLs.
pub fn print_running(http_addr: &str, http_port: u16, ws_addr: &str, ws_port: u16) {
    println!();
    println!(
        "{}",
        "POA node running. Press Ctrl+C to stop.".green().bold()
    );
    println!(
        "  HTTP RPC: {}",
        format!("http://{http_addr}:{http_port}").cyan()
    );
    println!("  WS RPC:   {}", format!("ws://{ws_addr}:{ws_port}").cyan());
}

// ── Consensus ──────────────────────────────────────────────────────

/// Print consensus initialization info.
pub fn print_consensus_init(signer_count: usize, epoch: u64, period: u64, mode: &str) {
    println!(
        "  {} POA Consensus: {} signers, epoch: {}, period: {}s, mode: {}",
        "OK".green().bold(),
        signer_count.to_string().cyan(),
        epoch.to_string().cyan(),
        period.to_string().cyan(),
        mode.cyan(),
    );
}

// ── On-Chain Reads ─────────────────────────────────────────────────

/// Print when on-chain gas limit differs from default.
pub fn print_onchain_gas_limit(onchain: u64, default: u64) {
    println!(
        "  {} OnChain gas limit: {} (default was {})",
        "OK".green().bold(),
        onchain.to_string().cyan(),
        default.to_string().dimmed(),
    );
}

/// Print when on-chain signers are loaded at startup.
pub fn print_onchain_signers(count: usize) {
    println!(
        "  {} OnChain signers: {} loaded from SignerRegistry",
        "OK".green().bold(),
        count.to_string().cyan(),
    );
}

// ── Payload / Block Production ─────────────────────────────────────

/// Print when signers are refreshed at an epoch block.
pub fn print_epoch_refresh(block_number: u64, signer_count: usize) {
    println!(
        "  {} Epoch #{}: refreshed {} signers from SignerRegistry",
        "OK".green().bold(),
        block_number.to_string().cyan(),
        signer_count.to_string().cyan(),
    );
}

/// Print when a block is signed by a POA signer.
///
/// `sign_ms` is the wall-clock time spent on ECDSA signing (Phase 5 timing).
pub fn print_block_signed(block_number: u64, signer: &Address, in_turn: bool, sign_ms: u64) {
    let turn = if in_turn {
        "in-turn".green()
    } else {
        "out-of-turn".yellow()
    };
    println!(
        "  {} POA block #{} signed by {} ({}, {}ms)",
        "OK".green().bold(),
        block_number.to_string().cyan(),
        format!("{signer}").cyan(),
        turn,
        sign_ms.to_string().dimmed(),
    );
}

// ── Block Monitor ──────────────────────────────────────────────────

/// Print block info when no signers are configured (dev/monitoring).
pub fn print_block_no_signers(block_num: u64, tx_count: usize) {
    println!(
        "  {} Block #{} - {} txs {}",
        "#".dimmed(),
        block_num.to_string().cyan(),
        tx_count.to_string().cyan(),
        "(no signers configured)".dimmed(),
    );
}

/// Print block info when in-turn signer produced it.
pub fn print_block_in_turn(block_num: u64, tx_count: usize, signer: &Address) {
    println!(
        "  {} Block #{} - {} txs (in-turn: {})",
        "#".green(),
        block_num.to_string().cyan(),
        tx_count.to_string().cyan(),
        format!("{signer}").green(),
    );
}

/// Print block info for an out-of-turn block.
pub fn print_block_out_of_turn(block_num: u64, tx_count: usize, expected: &Address) {
    println!(
        "  {} Block #{} - {} txs (out-of-turn, expected: {})",
        "#".yellow(),
        block_num.to_string().cyan(),
        tx_count.to_string().cyan(),
        format!("{expected}").dimmed(),
    );
}

/// Print block info for a block we're only observing.
pub fn print_block_observed(block_num: u64, tx_count: usize, expected: &Address) {
    println!(
        "  {} Block #{} - {} txs (expected signer: {})",
        "#".dimmed(),
        block_num.to_string().cyan(),
        tx_count.to_string().cyan(),
        format!("{expected}").dimmed(),
    );
}
