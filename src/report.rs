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
        ])?;

        // Write data.
        let stats = &self.stats;
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
        ])?;

        Ok(String::from_utf8(wtr.into_inner()?)?)
    }
}

impl fmt::Display for CensusReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stats = &self.stats;
        write!(
                f,
                "nodes: {} out of {} contacted | v2: {} ({:.1}%) | filters: {} ({:.1}%) | v2 & filters: {} ({:.1}%)",
                stats.total_nodes,
                self.total_contacted,
                stats.v2_transport,
                stats.percentage(stats.v2_transport),
                stats.compact_filters,
                stats.percentage(stats.compact_filters),
                stats.v2_and_filters,
                stats.percentage(stats.v2_and_filters)
            )
    }
}
