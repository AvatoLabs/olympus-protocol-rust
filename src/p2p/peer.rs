//! Peer management

use libp2p::{PeerId, Multiaddr};

/// Peer connection state
#[derive(Debug, Clone)]
pub enum PeerState {
    /// Peer is disconnected
    Disconnected,
    /// Peer is connecting
    Connecting,
    /// Peer is connected
    Connected,
    /// Peer connection failed
    Failed,
}

/// Peer information
#[derive(Debug, Clone)]
pub struct Peer {
    /// Peer ID
    pub peer_id: PeerId,
    /// Peer address
    pub address: Multiaddr,
    /// Connection state
    pub state: PeerState,
    /// Last seen timestamp
    pub last_seen: u64,
}
