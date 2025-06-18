use crate::stats::{FeatureStats, NodeStats};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Json,
    Jsonl,
    Csv,
}

/// A census report containing statistics from a network crawl.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CensusReport {
    /// When the census was taken (Unix timestamp in seconds).
    pub timestamp: u64,
    /// Duration of the census in seconds.
    pub duration_seconds: u64,
    /// Total number of nodes contacted (listening + non-listening).
    #[serde(default)]
    pub total_contacted: usize,
    /// Feature statistics.
    pub stats: FeatureStats,
    /// Version of the census tool.
    pub census_version: String,
}

impl CensusReport {
    pub fn from_stats(node_stats: &NodeStats) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            timestamp,
            duration_seconds: node_stats.duration(),
            total_contacted: node_stats.total_contacted(),
            stats: node_stats.features.clone(),
            census_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    pub fn write(
        &self,
        format: OutputFormat,
        output: Option<PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = match format {
            OutputFormat::Json => serde_json::to_string_pretty(self)?,
            OutputFormat::Jsonl => format!("{}\n", serde_json::to_string(self)?),
            OutputFormat::Csv => self.format_csv()?,
        };

        if let Some(path) = output {
            std::fs::write(path, content)?;
        } else {
            print!("{content}");
        }

        Ok(())
    }

    fn format_csv(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_writer(vec![]);

        // Write headers.
        wtr.write_record([
            "timestamp",
            "duration_seconds",
            "total_nodes",
            "total_contacted",
            "v2_transport",
            "v2_transport_pct",
            "compact_filters",
            "compact_filters_pct",
            "v2_and_filters",
            "v2_and_filters_pct",
            // IPv4 stats
            "ipv4_total",
            "ipv4_total_pct",
            "ipv4_v2",
            "ipv4_v2_pct",
            "ipv4_filters",
            "ipv4_filters_pct",
            "ipv4_v2_and_filters",
            "ipv4_v2_and_filters_pct",
            // IPv6 stats
            "ipv6_total",
            "ipv6_total_pct",
            "ipv6_v2",
            "ipv6_v2_pct",
            "ipv6_filters",
            "ipv6_filters_pct",
            "ipv6_v2_and_filters",
            "ipv6_v2_and_filters_pct",
            // Tor v2 stats
            "tor_v2_total",
            "tor_v2_total_pct",
            "tor_v2_v2",
            "tor_v2_v2_pct",
            "tor_v2_filters",
            "tor_v2_filters_pct",
            "tor_v2_v2_and_filters",
            "tor_v2_v2_and_filters_pct",
            // Tor v3 stats
            "tor_v3_total",
            "tor_v3_total_pct",
            "tor_v3_v2",
            "tor_v3_v2_pct",
            "tor_v3_filters",
            "tor_v3_filters_pct",
            "tor_v3_v2_and_filters",
            "tor_v3_v2_and_filters_pct",
            // I2P stats
            "i2p_total",
            "i2p_total_pct",
            "i2p_v2",
            "i2p_v2_pct",
            "i2p_filters",
            "i2p_filters_pct",
            "i2p_v2_and_filters",
            "i2p_v2_and_filters_pct",
            // CJDNS stats
            "cjdns_total",
            "cjdns_total_pct",
            "cjdns_v2",
            "cjdns_v2_pct",
            "cjdns_filters",
            "cjdns_filters_pct",
            "cjdns_v2_and_filters",
            "cjdns_v2_and_filters_pct",
            // Unknown stats
            "unknown_total",
            "unknown_total_pct",
            "unknown_v2",
            "unknown_v2_pct",
            "unknown_filters",
            "unknown_filters_pct",
            "unknown_v2_and_filters",
            "unknown_v2_and_filters_pct",
        ])?;

        // Write data.
        let stats = &self.stats;
        let conn_types = &stats.connection_types;
        wtr.write_record(&[
            self.timestamp.to_string(),
            self.duration_seconds.to_string(),
            stats.total_nodes.to_string(),
            self.total_contacted.to_string(),
            stats.v2_transport.to_string(),
            format!("{:.2}", stats.percentage(stats.v2_transport)),
            stats.compact_filters.to_string(),
            format!("{:.2}", stats.percentage(stats.compact_filters)),
            stats.v2_and_filters.to_string(),
            format!("{:.2}", stats.percentage(stats.v2_and_filters)),
            // IPv4 stats
            conn_types.ipv4.total_nodes.to_string(),
            format!(
                "{:.2}",
                conn_types.connection_percentage(conn_types.ipv4.total_nodes)
            ),
            conn_types.ipv4.v2_transport.to_string(),
            format!(
                "{:.2}",
                conn_types.ipv4.percentage(conn_types.ipv4.v2_transport)
            ),
            conn_types.ipv4.compact_filters.to_string(),
            format!(
                "{:.2}",
                conn_types.ipv4.percentage(conn_types.ipv4.compact_filters)
            ),
            conn_types.ipv4.v2_and_filters.to_string(),
            format!(
                "{:.2}",
                conn_types.ipv4.percentage(conn_types.ipv4.v2_and_filters)
            ),
            // IPv6 stats
            conn_types.ipv6.total_nodes.to_string(),
            format!(
                "{:.2}",
                conn_types.connection_percentage(conn_types.ipv6.total_nodes)
            ),
            conn_types.ipv6.v2_transport.to_string(),
            format!(
                "{:.2}",
                conn_types.ipv6.percentage(conn_types.ipv6.v2_transport)
            ),
            conn_types.ipv6.compact_filters.to_string(),
            format!(
                "{:.2}",
                conn_types.ipv6.percentage(conn_types.ipv6.compact_filters)
            ),
            conn_types.ipv6.v2_and_filters.to_string(),
            format!(
                "{:.2}",
                conn_types.ipv6.percentage(conn_types.ipv6.v2_and_filters)
            ),
            // Tor v2 stats
            conn_types.tor_v2.total_nodes.to_string(),
            format!(
                "{:.2}",
                conn_types.connection_percentage(conn_types.tor_v2.total_nodes)
            ),
            conn_types.tor_v2.v2_transport.to_string(),
            format!(
                "{:.2}",
                conn_types.tor_v2.percentage(conn_types.tor_v2.v2_transport)
            ),
            conn_types.tor_v2.compact_filters.to_string(),
            format!(
                "{:.2}",
                conn_types
                    .tor_v2
                    .percentage(conn_types.tor_v2.compact_filters)
            ),
            conn_types.tor_v2.v2_and_filters.to_string(),
            format!(
                "{:.2}",
                conn_types
                    .tor_v2
                    .percentage(conn_types.tor_v2.v2_and_filters)
            ),
            // Tor v3 stats
            conn_types.tor_v3.total_nodes.to_string(),
            format!(
                "{:.2}",
                conn_types.connection_percentage(conn_types.tor_v3.total_nodes)
            ),
            conn_types.tor_v3.v2_transport.to_string(),
            format!(
                "{:.2}",
                conn_types.tor_v3.percentage(conn_types.tor_v3.v2_transport)
            ),
            conn_types.tor_v3.compact_filters.to_string(),
            format!(
                "{:.2}",
                conn_types
                    .tor_v3
                    .percentage(conn_types.tor_v3.compact_filters)
            ),
            conn_types.tor_v3.v2_and_filters.to_string(),
            format!(
                "{:.2}",
                conn_types
                    .tor_v3
                    .percentage(conn_types.tor_v3.v2_and_filters)
            ),
            // I2P stats
            conn_types.i2p.total_nodes.to_string(),
            format!(
                "{:.2}",
                conn_types.connection_percentage(conn_types.i2p.total_nodes)
            ),
            conn_types.i2p.v2_transport.to_string(),
            format!(
                "{:.2}",
                conn_types.i2p.percentage(conn_types.i2p.v2_transport)
            ),
            conn_types.i2p.compact_filters.to_string(),
            format!(
                "{:.2}",
                conn_types.i2p.percentage(conn_types.i2p.compact_filters)
            ),
            conn_types.i2p.v2_and_filters.to_string(),
            format!(
                "{:.2}",
                conn_types.i2p.percentage(conn_types.i2p.v2_and_filters)
            ),
            // CJDNS stats
            conn_types.cjdns.total_nodes.to_string(),
            format!(
                "{:.2}",
                conn_types.connection_percentage(conn_types.cjdns.total_nodes)
            ),
            conn_types.cjdns.v2_transport.to_string(),
            format!(
                "{:.2}",
                conn_types.cjdns.percentage(conn_types.cjdns.v2_transport)
            ),
            conn_types.cjdns.compact_filters.to_string(),
            format!(
                "{:.2}",
                conn_types
                    .cjdns
                    .percentage(conn_types.cjdns.compact_filters)
            ),
            conn_types.cjdns.v2_and_filters.to_string(),
            format!(
                "{:.2}",
                conn_types.cjdns.percentage(conn_types.cjdns.v2_and_filters)
            ),
            // Unknown stats
            conn_types.unknown.total_nodes.to_string(),
            format!(
                "{:.2}",
                conn_types.connection_percentage(conn_types.unknown.total_nodes)
            ),
            conn_types.unknown.v2_transport.to_string(),
            format!(
                "{:.2}",
                conn_types
                    .unknown
                    .percentage(conn_types.unknown.v2_transport)
            ),
            conn_types.unknown.compact_filters.to_string(),
            format!(
                "{:.2}",
                conn_types
                    .unknown
                    .percentage(conn_types.unknown.compact_filters)
            ),
            conn_types.unknown.v2_and_filters.to_string(),
            format!(
                "{:.2}",
                conn_types
                    .unknown
                    .percentage(conn_types.unknown.v2_and_filters)
            ),
        ])?;

        Ok(String::from_utf8(wtr.into_inner()?)?)
    }
}

impl fmt::Display for CensusReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stats = &self.stats;
        let conn_types = &stats.connection_types;
        write!(
            f,
            "nodes: {} out of {} contacted | v2: {} ({:.1}%) | filters: {} ({:.1}%) | v2 & filters: {} ({:.1}%) | ipv4: {} ({:.1}%) | ipv6: {} ({:.1}%) | tor: {} ({:.1}%) | i2p: {} ({:.1}%) | cjdns: {} ({:.1}%)",
            stats.total_nodes,
            self.total_contacted,
            stats.v2_transport,
            stats.percentage(stats.v2_transport),
            stats.compact_filters,
            stats.percentage(stats.compact_filters),
            stats.v2_and_filters,
            stats.percentage(stats.v2_and_filters),
            conn_types.ipv4.total_nodes,
            conn_types.connection_percentage(conn_types.ipv4.total_nodes),
            conn_types.ipv6.total_nodes,
            conn_types.connection_percentage(conn_types.ipv6.total_nodes),
            conn_types.tor_v2.total_nodes + conn_types.tor_v3.total_nodes,
            conn_types.connection_percentage(conn_types.tor_v2.total_nodes + conn_types.tor_v3.total_nodes),
            conn_types.i2p.total_nodes,
            conn_types.connection_percentage(conn_types.i2p.total_nodes),
            conn_types.cjdns.total_nodes,
            conn_types.connection_percentage(conn_types.cjdns.total_nodes)
        )
    }
}
