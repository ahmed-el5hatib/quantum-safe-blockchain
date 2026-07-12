use crate::CoreResult;

pub trait Client: Send + Sync + fmt::Debug {
    fn connect(&mut self, addr: std::net::SocketAddr) -> CoreResult<()>;
    fn disconnect(&mut self) -> CoreResult<()>;
    fn is_connected(&self) -> bool;

    fn send_transaction(&self, tx: &[u8]) -> CoreResult<crate::HashDigest>;
    fn get_block(&self, hash: &crate::HashDigest) -> CoreResult<Option<Vec<u8>>>;
    fn get_chain_state(&self) -> CoreResult<crate::ChainState>;
    fn list_peers(&self) -> CoreResult<Vec<crate::PeerId>>;
}
