use bitcoin::p2p::ServiceFlags;
use bitcoin_peers_crawler::{Peer, PeerServices};
use serde::{Deserialize, Serialize};

/// Statistics about node features and capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStats {
    /// Total number of nodes analyzed.
    pub total_nodes: usize,
    /// Nodes supporting v2 transport (BIP-324).
    pub v2_transport: usize,
    /// Nodes supporting compact block filters (BIP-157/158).
    pub compact_filters: usize,
    /// Nodes supporting both v2 transport AND compact filters.
    pub v2_and_filters: usize,
}

impl FeatureStats {
    pub fn new() -> Self {
        Self {
            total_nodes: 0,
            v2_transport: 0,
            compact_filters: 0,
            v2_and_filters: 0,
        }
    }

    /// Calculate percentage of nodes with a feature.
    pub fn percentage(&self, count: usize) -> f64 {
        if self.total_nodes == 0 {
            0.0
        } else {
            (count as f64 / self.total_nodes as f64) * 100.0
        }
    }
}

/// Container for all node statistics.
pub struct NodeStats {
    pub features: FeatureStats,
    duration_seconds: u64,
    /// Total number of nodes contacted (listening + non-listening).
    total_contacted: usize,
}

impl NodeStats {
    pub fn new() -> Self {
        Self {
            features: FeatureStats::new(),
            duration_seconds: 0,
            total_contacted: 0,
        }
    }

    pub fn add_node(&mut self, peer: Peer) {
        self.features.total_nodes += 1;

        // Check service flags if known.
        if let PeerServices::Known(flags) = peer.services {
            let has_v2 = flags.has(ServiceFlags::P2P_V2);
            let has_filters = flags.has(ServiceFlags::COMPACT_FILTERS);

            if has_v2 {
                self.features.v2_transport += 1;
            }

            if has_filters {
                self.features.compact_filters += 1;
            }

            // Check for both features
            if has_v2 && has_filters {
                self.features.v2_and_filters += 1;
            }
        }
    }

    /// Increment the total number of nodes contacted.
    pub fn increment_contacted(&mut self) {
        self.total_contacted += 1;
    }

    pub fn total_nodes(&self) -> usize {
        self.features.total_nodes
    }

    pub fn total_contacted(&self) -> usize {
        self.total_contacted
    }

    pub fn set_duration(&mut self, seconds: u64) {
        self.duration_seconds = seconds;
    }

    pub fn duration(&self) -> u64 {
        self.duration_seconds
    }
}
