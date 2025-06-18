use bitcoin::p2p::address::AddrV2;
use bitcoin::p2p::ServiceFlags;
use bitcoin_peers_crawler::{Peer, PeerServices};
use serde::{Deserialize, Serialize};

/// Feature statistics for a specific connection type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTypeFeatures {
    /// Total number of nodes for this connection type.
    pub total_nodes: usize,
    /// Nodes supporting v2 transport (BIP-324).
    pub v2_transport: usize,
    /// Nodes supporting compact block filters (BIP-157/158).
    pub compact_filters: usize,
    /// Nodes supporting both v2 transport AND compact filters.
    pub v2_and_filters: usize,
}

impl Default for ConnectionTypeFeatures {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionTypeFeatures {
    pub fn new() -> Self {
        Self {
            total_nodes: 0,
            v2_transport: 0,
            compact_filters: 0,
            v2_and_filters: 0,
        }
    }

    /// Add a node with the given features to this connection type.
    pub fn add_node(&mut self, has_v2: bool, has_filters: bool) {
        self.total_nodes += 1;

        if has_v2 {
            self.v2_transport += 1;
        }

        if has_filters {
            self.compact_filters += 1;
        }

        if has_v2 && has_filters {
            self.v2_and_filters += 1;
        }
    }

    /// Calculate percentage of nodes with a feature for this connection type.
    pub fn percentage(&self, count: usize) -> f64 {
        if self.total_nodes == 0 {
            0.0
        } else {
            (count as f64 / self.total_nodes as f64) * 100.0
        }
    }
}

/// Statistics broken down by connection type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionTypeStats {
    /// IPv4 clearnet connections.
    pub ipv4: ConnectionTypeFeatures,
    /// IPv6 clearnet connections.
    pub ipv6: ConnectionTypeFeatures,
    /// Tor v2 onion addresses (deprecated).
    pub tor_v2: ConnectionTypeFeatures,
    /// Tor v3 onion addresses.
    pub tor_v3: ConnectionTypeFeatures,
    /// I2P addresses.
    pub i2p: ConnectionTypeFeatures,
    /// CJDNS mesh network addresses.
    pub cjdns: ConnectionTypeFeatures,
    /// Unknown/future address types.
    pub unknown: ConnectionTypeFeatures,
}

impl Default for ConnectionTypeStats {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionTypeStats {
    pub fn new() -> Self {
        Self {
            ipv4: ConnectionTypeFeatures::new(),
            ipv6: ConnectionTypeFeatures::new(),
            tor_v2: ConnectionTypeFeatures::new(),
            tor_v3: ConnectionTypeFeatures::new(),
            i2p: ConnectionTypeFeatures::new(),
            cjdns: ConnectionTypeFeatures::new(),
            unknown: ConnectionTypeFeatures::new(),
        }
    }

    /// Get total nodes across all connection types.
    pub fn total_nodes(&self) -> usize {
        self.ipv4.total_nodes
            + self.ipv6.total_nodes
            + self.tor_v2.total_nodes
            + self.tor_v3.total_nodes
            + self.i2p.total_nodes
            + self.cjdns.total_nodes
            + self.unknown.total_nodes
    }

    /// Add a node with features to the appropriate connection type.
    pub fn add_node(&mut self, addr: &AddrV2, has_v2: bool, has_filters: bool) {
        match addr {
            AddrV2::Ipv4(_) => self.ipv4.add_node(has_v2, has_filters),
            AddrV2::Ipv6(_) => self.ipv6.add_node(has_v2, has_filters),
            AddrV2::TorV2(_) => self.tor_v2.add_node(has_v2, has_filters),
            AddrV2::TorV3(_) => self.tor_v3.add_node(has_v2, has_filters),
            AddrV2::I2p(_) => self.i2p.add_node(has_v2, has_filters),
            AddrV2::Cjdns(_) => self.cjdns.add_node(has_v2, has_filters),
            AddrV2::Unknown(_, _) => self.unknown.add_node(has_v2, has_filters),
        }
    }

    /// Calculate percentage for a connection type based on total nodes.
    pub fn connection_percentage(&self, conn_type_total: usize) -> f64 {
        let total = self.total_nodes();
        if total == 0 {
            0.0
        } else {
            (conn_type_total as f64 / total as f64) * 100.0
        }
    }
}

/// Statistics about node features and capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureStats {
    /// Total number of nodes analyzed (sum across all connection types).
    pub total_nodes: usize,
    /// Nodes supporting v2 transport (BIP-324) (sum across all connection types).
    pub v2_transport: usize,
    /// Nodes supporting compact block filters (BIP-157/158) (sum across all connection types).
    pub compact_filters: usize,
    /// Nodes supporting both v2 transport AND compact filters (sum across all connection types).
    pub v2_and_filters: usize,
    /// Detailed breakdown by connection type.
    pub connection_types: ConnectionTypeStats,
}

impl Default for FeatureStats {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureStats {
    pub fn new() -> Self {
        Self {
            total_nodes: 0,
            v2_transport: 0,
            compact_filters: 0,
            v2_and_filters: 0,
            connection_types: ConnectionTypeStats::new(),
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

impl Default for NodeStats {
    fn default() -> Self {
        Self::new()
    }
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
        // Determine features from service flags
        let (has_v2, has_filters) = if let PeerServices::Known(flags) = peer.services {
            (
                flags.has(ServiceFlags::P2P_V2),
                flags.has(ServiceFlags::COMPACT_FILTERS),
            )
        } else {
            (false, false)
        };

        // Add to connection type-specific stats
        self.features
            .connection_types
            .add_node(&peer.address, has_v2, has_filters);

        // Update aggregate stats
        self.features.total_nodes += 1;
        if has_v2 {
            self.features.v2_transport += 1;
        }
        if has_filters {
            self.features.compact_filters += 1;
        }
        if has_v2 && has_filters {
            self.features.v2_and_filters += 1;
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
