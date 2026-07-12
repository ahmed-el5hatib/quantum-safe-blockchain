use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PeerId(pub [u8; 32]);

impl PeerId {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NodeId(pub [u8; 32]);

impl NodeId {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleRoot(pub Vec<u8>);

impl MerkleRoot {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for MerkleRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashDigest(pub Vec<u8>);

impl HashDigest {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for HashDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u32,
    pub previous_hash: HashDigest,
    pub merkle_root: MerkleRoot,
    pub timestamp: u64,
    pub bits: u32,
    pub nonce: u64,
    pub extra_data: Vec<u8>,
}

impl BlockHeader {
    pub fn new(
        version: u32,
        previous_hash: HashDigest,
        merkle_root: MerkleRoot,
        timestamp: u64,
        bits: u32,
    ) -> Self {
        Self {
            version,
            previous_hash,
            merkle_root,
            timestamp,
            bits,
            nonce: 0,
            extra_data: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block<Transaction> {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

impl<Transaction> Block<Transaction> {
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Self {
            header,
            transactions,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionInput {
    pub previous_output: OutPoint,
    pub signature: Vec<u8>,
    pub sequence: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutPoint {
    pub transaction_hash: HashDigest,
    pub output_index: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransactionOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub version: u32,
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
    pub lock_time: u64,
}

impl Transaction {
    pub fn new(
        version: u32,
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        lock_time: u64,
    ) -> Self {
        Self {
            version,
            inputs,
            outputs,
            lock_time,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub timestamp: u64,
    pub difficulty: u32,
    pub initial_supply: u64,
    pub genesis_message: String,
}

impl GenesisConfig {
    pub fn new(
        timestamp: u64,
        difficulty: u32,
        initial_supply: u64,
        genesis_message: String,
    ) -> Self {
        Self {
            timestamp,
            difficulty,
            initial_supply,
            genesis_message,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChainState {
    pub height: u64,
    pub best_block_hash: HashDigest,
    pub total_difficulty: u128,
    pub cumulative_difficulty: u128,
}

impl ChainState {
    pub fn new(height: u64, best_block_hash: HashDigest) -> Self {
        Self {
            height,
            best_block_hash,
            total_difficulty: 0,
            cumulative_difficulty: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NetworkMessage {
    Block(Vec<u8>),
    Transaction(Vec<u8>),
    PeersRequest,
    PeersResponse(Vec<PeerId>),
    SyncRequest { from_height: u64 },
    SyncResponse { blocks: Vec<Vec<u8>> },
    Ping,
    Pong,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    BlockMined {
        block_hash: HashDigest,
        height: u64,
    },
    TransactionReceived {
        tx_hash: HashDigest,
        from_peer: Option<PeerId>,
    },
    PeerConnected {
        peer_id: PeerId,
        addr: std::net::SocketAddr,
    },
    PeerDisconnected {
        peer_id: PeerId,
    },
    ChainReorg {
        old_height: u64,
        new_height: u64,
    },
    ConsensusFinalized {
        block_hash: HashDigest,
        height: u64,
    },
}
