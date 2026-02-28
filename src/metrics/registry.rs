//! Prometheus-compatible metrics registry and HTTP server.
//!
//! [`MetricsRegistry`] holds atomic counters for all chain-wide metrics that
//! can be scraped by a Prometheus server.  [`start_metrics_server`] launches a
//! lightweight TCP listener on `0.0.0.0:{port}/metrics` that serves the
//! Prometheus text exposition format.
//!
//! Thread safety: all fields are [`AtomicU64`] — no locks required on write.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// ── MetricsRegistry ──────────────────────────────────────────────────────────

/// Global metrics registry for Prometheus-style export.
///
/// All counters are [`AtomicU64`] for lock-free concurrent updates from the
/// block monitoring task, payload builder, and cache layer.
#[derive(Debug, Default)]
pub struct MetricsRegistry {
    /// Total blocks produced (counter).
    pub blocks_total: AtomicU64,
    /// Total transactions processed (counter).
    pub transactions_total: AtomicU64,
    /// Total gas used across all blocks (counter).
    pub gas_used_total: AtomicU64,
    /// Current block number (gauge).
    pub block_height: AtomicU64,
    /// Number of connected peers (gauge).
    pub peer_count: AtomicU64,
    /// Number of authorized signers (gauge).
    pub signer_count: AtomicU64,
    /// Number of pending transactions in mempool (gauge).
    pub pending_tx_count: AtomicU64,
    /// Last block build time in milliseconds (gauge).
    pub last_build_time_ms: AtomicU64,
    /// Last block sign time in milliseconds (gauge).
    pub last_sign_time_ms: AtomicU64,
    /// Number of in-turn blocks produced (counter).
    pub in_turn_blocks: AtomicU64,
    /// Number of out-of-turn blocks produced (counter).
    pub out_of_turn_blocks: AtomicU64,
    /// Chain ID (gauge — constant after init).
    pub chain_id: AtomicU64,
    /// Whether this node is currently a signer (gauge, 1=yes, 0=no).
    pub is_signer: AtomicU64,
    /// Node start timestamp in unix seconds (gauge — constant after init).
    pub start_time: AtomicU64,
    /// Total state diff accounts changed (counter).
    pub state_diff_accounts: AtomicU64,
    /// Total state diff storage slots changed (counter).
    pub state_diff_slots: AtomicU64,
    /// Cache hit count (counter).
    pub cache_hits: AtomicU64,
    /// Cache miss count (counter).
    pub cache_misses: AtomicU64,
}

impl MetricsRegistry {
    /// Create a new registry with the given chain ID and the current time as start time.
    pub fn new(chain_id: u64) -> Self {
        let registry = Self::default();
        registry.chain_id.store(chain_id, Ordering::Relaxed);
        registry.start_time.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            Ordering::Relaxed,
        );
        registry
    }

    /// Record a new block: increments counters and updates the block height gauge.
    pub fn record_block(&self, block_number: u64, tx_count: usize, gas_used: u64, in_turn: bool) {
        self.blocks_total.fetch_add(1, Ordering::Relaxed);
        self.transactions_total
            .fetch_add(tx_count as u64, Ordering::Relaxed);
        self.gas_used_total.fetch_add(gas_used, Ordering::Relaxed);
        self.block_height.store(block_number, Ordering::Relaxed);
        if in_turn {
            self.in_turn_blocks.fetch_add(1, Ordering::Relaxed);
        } else {
            self.out_of_turn_blocks.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Record state diff statistics for a block.
    pub fn record_state_diff(&self, accounts_changed: usize, slots_changed: usize) {
        self.state_diff_accounts
            .fetch_add(accounts_changed as u64, Ordering::Relaxed);
        self.state_diff_slots
            .fetch_add(slots_changed as u64, Ordering::Relaxed);
    }

    /// Record build and sign timing for the latest block.
    pub fn record_build_timing(&self, build_ms: u64, sign_ms: u64) {
        self.last_build_time_ms.store(build_ms, Ordering::Relaxed);
        self.last_sign_time_ms.store(sign_ms, Ordering::Relaxed);
    }

    /// Record a cache hit.
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss.
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Export all metrics in Prometheus text exposition format.
    ///
    /// Each metric includes `# HELP` and `# TYPE` annotations as required by
    /// the Prometheus specification.
    pub fn to_prometheus(&self) -> String {
        let mut output = String::with_capacity(4096);

        // Helper macro for Prometheus metric lines
        macro_rules! metric {
            ($name:expr, $help:expr, $type:expr, $value:expr) => {
                output.push_str(&format!(
                    "# HELP {} {}\n# TYPE {} {}\n{} {}\n",
                    $name, $help, $name, $type, $name, $value
                ));
            };
        }

        metric!(
            "meowchain_blocks_total",
            "Total blocks produced",
            "counter",
            self.blocks_total.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_transactions_total",
            "Total transactions processed",
            "counter",
            self.transactions_total.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_gas_used_total",
            "Total gas used",
            "counter",
            self.gas_used_total.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_block_height",
            "Current block height",
            "gauge",
            self.block_height.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_peer_count",
            "Connected peers",
            "gauge",
            self.peer_count.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_signer_count",
            "Authorized signers",
            "gauge",
            self.signer_count.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_pending_tx_count",
            "Pending transactions",
            "gauge",
            self.pending_tx_count.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_last_build_time_ms",
            "Last block build time",
            "gauge",
            self.last_build_time_ms.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_last_sign_time_ms",
            "Last block sign time",
            "gauge",
            self.last_sign_time_ms.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_in_turn_blocks_total",
            "In-turn blocks produced",
            "counter",
            self.in_turn_blocks.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_out_of_turn_blocks_total",
            "Out-of-turn blocks produced",
            "counter",
            self.out_of_turn_blocks.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_chain_id",
            "Chain ID",
            "gauge",
            self.chain_id.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_is_signer",
            "Whether this node is a signer",
            "gauge",
            self.is_signer.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_start_time_seconds",
            "Node start time (unix)",
            "gauge",
            self.start_time.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_state_diff_accounts_total",
            "State diff accounts changed",
            "counter",
            self.state_diff_accounts.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_state_diff_slots_total",
            "State diff storage slots changed",
            "counter",
            self.state_diff_slots.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_cache_hits_total",
            "Cache hits",
            "counter",
            self.cache_hits.load(Ordering::Relaxed)
        );
        metric!(
            "meowchain_cache_misses_total",
            "Cache misses",
            "counter",
            self.cache_misses.load(Ordering::Relaxed)
        );

        output
    }
}

// ── HTTP server ──────────────────────────────────────────────────────────────

/// Start a lightweight HTTP server for Prometheus metrics scraping.
///
/// Listens on `0.0.0.0:{port}` and serves metrics at any path (typically
/// scraped at `/metrics`).  Uses raw tokio TCP — no framework dependency.
///
/// The server runs in a background tokio task and will keep serving until the
/// runtime shuts down.
pub async fn start_metrics_server(port: u16, registry: Arc<MetricsRegistry>) -> eyre::Result<()> {
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpListener;

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    tokio::spawn(async move {
        loop {
            if let Ok((mut stream, _)) = listener.accept().await {
                let metrics_text = registry.to_prometheus();
                let response = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Type: text/plain; version=0.0.4; charset=utf-8\r\n\
                     Content-Length: {}\r\n\
                     \r\n\
                     {}",
                    metrics_text.len(),
                    metrics_text
                );
                let _ = stream.write_all(response.as_bytes()).await;
            }
        }
    });

    Ok(())
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_registry_new_sets_chain_id_and_start_time() {
        let registry = MetricsRegistry::new(9323310);
        assert_eq!(registry.chain_id.load(Ordering::Relaxed), 9323310);

        let start = registry.start_time.load(Ordering::Relaxed);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // start_time should be within 2 seconds of now
        assert!(now.abs_diff(start) < 2, "start_time should be close to now");
    }

    #[test]
    fn test_record_block_increments_counters() {
        let registry = MetricsRegistry::new(1);
        registry.record_block(1, 5, 105_000, true);

        assert_eq!(registry.blocks_total.load(Ordering::Relaxed), 1);
        assert_eq!(registry.transactions_total.load(Ordering::Relaxed), 5);
        assert_eq!(registry.gas_used_total.load(Ordering::Relaxed), 105_000);
        assert_eq!(registry.block_height.load(Ordering::Relaxed), 1);
        assert_eq!(registry.in_turn_blocks.load(Ordering::Relaxed), 1);
        assert_eq!(registry.out_of_turn_blocks.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_record_block_out_of_turn() {
        let registry = MetricsRegistry::new(1);
        registry.record_block(1, 2, 42_000, false);

        assert_eq!(registry.in_turn_blocks.load(Ordering::Relaxed), 0);
        assert_eq!(registry.out_of_turn_blocks.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_multiple_record_block_calls_accumulate() {
        let registry = MetricsRegistry::new(1);
        registry.record_block(1, 3, 63_000, true);
        registry.record_block(2, 7, 147_000, false);
        registry.record_block(3, 10, 210_000, true);

        assert_eq!(registry.blocks_total.load(Ordering::Relaxed), 3);
        assert_eq!(registry.transactions_total.load(Ordering::Relaxed), 20);
        assert_eq!(registry.gas_used_total.load(Ordering::Relaxed), 420_000);
        assert_eq!(registry.block_height.load(Ordering::Relaxed), 3);
        assert_eq!(registry.in_turn_blocks.load(Ordering::Relaxed), 2);
        assert_eq!(registry.out_of_turn_blocks.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_to_prometheus_output_format_valid() {
        let registry = MetricsRegistry::new(9323310);
        registry.record_block(42, 10, 210_000, true);

        let output = registry.to_prometheus();

        // Every metric must have HELP, TYPE, and value lines
        for line in output.lines() {
            let trimmed = line.trim();
            assert!(
                trimmed.starts_with("# HELP ")
                    || trimmed.starts_with("# TYPE ")
                    || trimmed.starts_with("meowchain_"),
                "Unexpected line in prometheus output: {}",
                trimmed
            );
        }

        // Check that output is non-empty
        assert!(!output.is_empty());
    }

    #[test]
    fn test_all_metric_names_present_in_prometheus_output() {
        let registry = MetricsRegistry::new(9323310);
        let output = registry.to_prometheus();

        let expected_metrics = [
            "meowchain_blocks_total",
            "meowchain_transactions_total",
            "meowchain_gas_used_total",
            "meowchain_block_height",
            "meowchain_peer_count",
            "meowchain_signer_count",
            "meowchain_pending_tx_count",
            "meowchain_last_build_time_ms",
            "meowchain_last_sign_time_ms",
            "meowchain_in_turn_blocks_total",
            "meowchain_out_of_turn_blocks_total",
            "meowchain_chain_id",
            "meowchain_is_signer",
            "meowchain_start_time_seconds",
            "meowchain_state_diff_accounts_total",
            "meowchain_state_diff_slots_total",
            "meowchain_cache_hits_total",
            "meowchain_cache_misses_total",
        ];

        for metric in expected_metrics {
            assert!(
                output.contains(metric),
                "Prometheus output missing metric: {}",
                metric
            );
        }
    }

    #[test]
    fn test_counter_and_gauge_types_correct() {
        let registry = MetricsRegistry::new(1);
        let output = registry.to_prometheus();

        // Counters (monotonically increasing)
        let counters = [
            "meowchain_blocks_total",
            "meowchain_transactions_total",
            "meowchain_gas_used_total",
            "meowchain_in_turn_blocks_total",
            "meowchain_out_of_turn_blocks_total",
            "meowchain_state_diff_accounts_total",
            "meowchain_state_diff_slots_total",
            "meowchain_cache_hits_total",
            "meowchain_cache_misses_total",
        ];
        for name in counters {
            let type_line = format!("# TYPE {} counter", name);
            assert!(
                output.contains(&type_line),
                "{} should be a counter, not found: {}",
                name,
                type_line
            );
        }

        // Gauges (can go up and down)
        let gauges = [
            "meowchain_block_height",
            "meowchain_peer_count",
            "meowchain_signer_count",
            "meowchain_pending_tx_count",
            "meowchain_last_build_time_ms",
            "meowchain_last_sign_time_ms",
            "meowchain_chain_id",
            "meowchain_is_signer",
            "meowchain_start_time_seconds",
        ];
        for name in gauges {
            let type_line = format!("# TYPE {} gauge", name);
            assert!(
                output.contains(&type_line),
                "{} should be a gauge, not found: {}",
                name,
                type_line
            );
        }
    }

    #[test]
    fn test_start_time_is_reasonable() {
        let registry = MetricsRegistry::new(1);
        let start = registry.start_time.load(Ordering::Relaxed);

        // Must be after 2024-01-01 (1704067200) and before 2030-01-01 (1893456000)
        assert!(
            start > 1_704_067_200,
            "start_time {} is before 2024",
            start
        );
        assert!(
            start < 1_893_456_000,
            "start_time {} is after 2030",
            start
        );
    }

    #[test]
    fn test_empty_registry_outputs_zeros() {
        let registry = MetricsRegistry::default();
        let output = registry.to_prometheus();

        // All counters and gauges should be 0
        assert!(output.contains("meowchain_blocks_total 0"));
        assert!(output.contains("meowchain_transactions_total 0"));
        assert!(output.contains("meowchain_gas_used_total 0"));
        assert!(output.contains("meowchain_block_height 0"));
        assert!(output.contains("meowchain_peer_count 0"));
        assert!(output.contains("meowchain_signer_count 0"));
        assert!(output.contains("meowchain_chain_id 0"));
        assert!(output.contains("meowchain_is_signer 0"));
        assert!(output.contains("meowchain_cache_hits_total 0"));
        assert!(output.contains("meowchain_cache_misses_total 0"));
    }

    #[test]
    fn test_is_signer_toggle() {
        let registry = MetricsRegistry::new(1);
        assert_eq!(registry.is_signer.load(Ordering::Relaxed), 0);

        registry.is_signer.store(1, Ordering::Relaxed);
        assert_eq!(registry.is_signer.load(Ordering::Relaxed), 1);

        registry.is_signer.store(0, Ordering::Relaxed);
        assert_eq!(registry.is_signer.load(Ordering::Relaxed), 0);

        // Verify it appears in prometheus output
        registry.is_signer.store(1, Ordering::Relaxed);
        let output = registry.to_prometheus();
        assert!(output.contains("meowchain_is_signer 1"));
    }

    #[test]
    fn test_concurrent_record_block_is_safe() {
        let registry = Arc::new(MetricsRegistry::new(1));
        let threads: Vec<_> = (0..10)
            .map(|i| {
                let reg = registry.clone();
                std::thread::spawn(move || {
                    for j in 0..100 {
                        reg.record_block(i * 100 + j, 1, 21_000, j % 2 == 0);
                    }
                })
            })
            .collect();

        for t in threads {
            t.join().expect("thread panicked");
        }

        // 10 threads x 100 iterations = 1000 blocks total
        assert_eq!(registry.blocks_total.load(Ordering::Relaxed), 1000);
        assert_eq!(registry.transactions_total.load(Ordering::Relaxed), 1000);
        assert_eq!(
            registry.gas_used_total.load(Ordering::Relaxed),
            1000 * 21_000
        );
        // 50 in-turn per thread × 10 threads = 500
        assert_eq!(registry.in_turn_blocks.load(Ordering::Relaxed), 500);
        assert_eq!(registry.out_of_turn_blocks.load(Ordering::Relaxed), 500);
    }

    #[test]
    fn test_record_state_diff() {
        let registry = MetricsRegistry::new(1);
        registry.record_state_diff(5, 20);
        registry.record_state_diff(3, 10);

        assert_eq!(
            registry.state_diff_accounts.load(Ordering::Relaxed),
            8
        );
        assert_eq!(registry.state_diff_slots.load(Ordering::Relaxed), 30);
    }

    #[test]
    fn test_record_build_timing() {
        let registry = MetricsRegistry::new(1);
        registry.record_build_timing(15, 3);

        assert_eq!(registry.last_build_time_ms.load(Ordering::Relaxed), 15);
        assert_eq!(registry.last_sign_time_ms.load(Ordering::Relaxed), 3);

        // Overwrite with new values
        registry.record_build_timing(22, 5);
        assert_eq!(registry.last_build_time_ms.load(Ordering::Relaxed), 22);
        assert_eq!(registry.last_sign_time_ms.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn test_cache_hit_miss_counters() {
        let registry = MetricsRegistry::new(1);
        registry.record_cache_hit();
        registry.record_cache_hit();
        registry.record_cache_hit();
        registry.record_cache_miss();

        assert_eq!(registry.cache_hits.load(Ordering::Relaxed), 3);
        assert_eq!(registry.cache_misses.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_prometheus_values_after_recording() {
        let registry = MetricsRegistry::new(9323310);
        registry.record_block(100, 25, 525_000, true);
        registry.record_state_diff(3, 7);
        registry.record_build_timing(12, 2);
        registry.record_cache_hit();
        registry.signer_count.store(5, Ordering::Relaxed);
        registry.peer_count.store(3, Ordering::Relaxed);

        let output = registry.to_prometheus();

        assert!(output.contains("meowchain_blocks_total 1"));
        assert!(output.contains("meowchain_transactions_total 25"));
        assert!(output.contains("meowchain_gas_used_total 525000"));
        assert!(output.contains("meowchain_block_height 100"));
        assert!(output.contains("meowchain_in_turn_blocks_total 1"));
        assert!(output.contains("meowchain_chain_id 9323310"));
        assert!(output.contains("meowchain_signer_count 5"));
        assert!(output.contains("meowchain_peer_count 3"));
        assert!(output.contains("meowchain_last_build_time_ms 12"));
        assert!(output.contains("meowchain_last_sign_time_ms 2"));
        assert!(output.contains("meowchain_state_diff_accounts_total 3"));
        assert!(output.contains("meowchain_state_diff_slots_total 7"));
        assert!(output.contains("meowchain_cache_hits_total 1"));
    }

    #[tokio::test]
    async fn test_metrics_server_responds() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpStream;

        let registry = Arc::new(MetricsRegistry::new(9323310));
        registry.record_block(1, 10, 210_000, true);

        // Use port 0 to let the OS assign a free port
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap();
        let addr = listener.local_addr().unwrap();

        let reg_clone = registry.clone();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let metrics_text = reg_clone.to_prometheus();
                let response = format!(
                    "HTTP/1.1 200 OK\r\n\
                     Content-Type: text/plain; version=0.0.4; charset=utf-8\r\n\
                     Content-Length: {}\r\n\
                     \r\n\
                     {}",
                    metrics_text.len(),
                    metrics_text
                );
                let _ = stream.write_all(response.as_bytes()).await;
            }
        });

        // Connect and read response
        let mut stream = TcpStream::connect(addr).await.unwrap();
        stream
            .write_all(b"GET /metrics HTTP/1.1\r\nHost: localhost\r\n\r\n")
            .await
            .unwrap();

        let mut buf = vec![0u8; 8192];
        let n = stream.read(&mut buf).await.unwrap();
        let response = String::from_utf8_lossy(&buf[..n]);

        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains("text/plain"));
        assert!(response.contains("meowchain_blocks_total 1"));
        assert!(response.contains("meowchain_chain_id 9323310"));
    }
}
