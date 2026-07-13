use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::{CoreError, CoreResult};

// =============================================================================
// Strongly Typed Wrappers
// =============================================================================

/// Block/chain protocol version.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version(pub u32);

impl Version {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Block height in the chain.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Height(pub u64);

impl Height {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for Height {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Mining difficulty target.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Difficulty(pub u32);

impl Difficulty {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Block nonce for proof-of-work.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Nonce(pub u64);

impl Nonce {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unix timestamp in seconds.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

impl Timestamp {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Previous block hash.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PreviousHash(pub HashDigest);

impl PreviousHash {
    pub fn new(hash: HashDigest) -> Self {
        Self(hash)
    }

    pub fn zero() -> Self {
        Self(HashDigest::new(vec![0u8; 32]))
    }

    pub fn inner(&self) -> &HashDigest {
        &self.0
    }
}

impl fmt::Display for PreviousHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Block hash.
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BlockHash(pub HashDigest);

impl BlockHash {
    pub fn new(hash: HashDigest) -> Self {
        Self(hash)
    }

    pub fn inner(&self) -> &HashDigest {
        &self.0
    }
}

impl fmt::Display for BlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// =============================================================================
// Core Types
// =============================================================================

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

// =============================================================================
// Block Header
// =============================================================================

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: Version,
    pub height: Height,
    pub timestamp: Timestamp,
    pub previous_hash: PreviousHash,
    pub merkle_root: MerkleRoot,
    pub difficulty: Difficulty,
    pub nonce: Nonce,
}

impl BlockHeader {
    pub fn new(
        version: Version,
        height: Height,
        timestamp: Timestamp,
        previous_hash: PreviousHash,
        merkle_root: MerkleRoot,
        difficulty: Difficulty,
        nonce: Nonce,
    ) -> Self {
        Self {
            version,
            height,
            timestamp,
            previous_hash,
            merkle_root,
            difficulty,
            nonce,
        }
    }
}

// =============================================================================
// Block Metadata
// =============================================================================

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub block_size: usize,
    pub transaction_count: usize,
}

impl BlockMetadata {
    pub fn new(block_size: usize, transaction_count: usize) -> Self {
        Self {
            block_size,
            transaction_count,
        }
    }
}

// =============================================================================
// Chain Metadata
// =============================================================================

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainMetadata {
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub chain_work: u128,
}

impl ChainMetadata {
    pub fn new(total_blocks: u64, total_transactions: u64, chain_work: u128) -> Self {
        Self {
            total_blocks,
            total_transactions,
            chain_work,
        }
    }
}

// =============================================================================
// Block
// =============================================================================

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub metadata: BlockMetadata,
    pub block_hash: BlockHash,
}

impl Block {
    pub fn new(
        header: BlockHeader,
        transactions: Vec<Transaction>,
        metadata: BlockMetadata,
        block_hash: BlockHash,
    ) -> Self {
        Self {
            header,
            transactions,
            metadata,
            block_hash,
        }
    }

    pub fn to_json(&self) -> CoreResult<String> {
        serde_json::to_string(self).map_err(|e| CoreError::Unknown(e.to_string()))
    }

    pub fn from_json(data: &str) -> CoreResult<Self> {
        serde_json::from_str(data).map_err(|e| CoreError::Unknown(e.to_string()))
    }

    pub fn to_binary(&self) -> CoreResult<Vec<u8>> {
        bincode::serialize(self).map_err(|e| CoreError::Unknown(e.to_string()))
    }

    pub fn from_binary(data: &[u8]) -> CoreResult<Self> {
        bincode::deserialize(data).map_err(|e| CoreError::Unknown(e.to_string()))
    }
}

// =============================================================================
// Genesis Config
// =============================================================================
// Genesis Block
// =============================================================================

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisBlock {
    pub block: Block,
    pub config: GenesisConfig,
}

impl GenesisBlock {
    pub fn new(block: Block, config: GenesisConfig) -> Self {
        Self { block, config }
    }
}

// =============================================================================
// Genesis Config
// =============================================================================

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
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

// =============================================================================
// Transactions
// =============================================================================

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionInput {
    pub previous_output: OutPoint,
    pub signature: Vec<u8>,
    pub sequence: u32,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutPoint {
    pub transaction_hash: HashDigest,
    pub output_index: u32,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionOutput {
    pub value: u64,
    pub script_pubkey: Vec<u8>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
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

// =============================================================================
// Chain State
// =============================================================================

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

// =============================================================================
// Network & Events
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
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
