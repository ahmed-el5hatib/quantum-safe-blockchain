use crate::{NetworkMessage, PeerId};

pub trait Transport: Send + Sync + fmt::Debug {
    fn listen(&self) -> CoreResult<()>;
    fn dial(&self, peer_id: PeerId) -> CoreResult<()>;
    fn close(&self, peer_id: PeerId) -> CoreResult<()>;
}

pub trait PeerDiscovery: Send + Sync + fmt::Debug {
    fn discover(&self) -> CoreResult<Vec<PeerId>>;
    fn register_peer(&self, peer_id: PeerId) -> CoreResult<()>;
    fn unregister_peer(&self, peer_id: PeerId) -> CoreResult<()>;
}

pub trait MessageCodec: Send + Sync + fmt::Debug {
    fn encode(&self, message: &NetworkMessage) -> CoreResult<Vec<u8>>;
    fn decode(&self, data: &[u8]) -> CoreResult<NetworkMessage>;
}

pub trait PeerManager: Send + Sync + fmt::Debug {
    fn connected_peers(&self) -> CoreResult<Vec<PeerId>>;
    fn ban_peer(&self, peer_id: PeerId, duration: std::time::Duration) -> CoreResult<()>;
    fn unban_peer(&self, peer_id: PeerId) -> CoreResult<()>;
    fn peer_score(&self, peer_id: PeerId) -> CoreResult<i32>;
}

pub trait SyncManager: Send + Sync + fmt::Debug {
    fn start_sync(&self) -> CoreResult<()>;
    fn stop_sync(&self) -> CoreResult<()>;
    fn sync_status(&self) -> CoreResult<SyncStatus>;
}

#[derive(Clone, Debug)]
pub struct SyncStatus {
    pub syncing: bool,
    pub current_height: u64,
    pub highest_height: u64,
}
