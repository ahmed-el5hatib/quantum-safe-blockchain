//! Genesis block generation and deterministic initialization.

use crate::error::CoreResult;
use crate::traits::{BlockT, GenesisT};
use crate::types::{
    Block, BlockHash, BlockHeader, BlockMetadata, Difficulty, GenesisBlock, GenesisConfig, Height,
    MerkleRoot, Nonce, PreviousHash, Timestamp, Version,
};
use cryptography::core::traits::HashFunction;

/// Deterministic genesis block generator.
///
/// Given an identical `GenesisConfig`, this generator always produces the same
/// `GenesisBlock`. This ensures that all nodes in the network agree on the
/// genesis state without requiring a trusted setup.
pub struct Genesis {
    config: GenesisConfig,
}

impl Genesis {
    pub fn new(config: GenesisConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &GenesisConfig {
        &self.config
    }

    pub fn generate(&self, hasher: &dyn HashFunction) -> CoreResult<GenesisBlock> {
        let version = Version::new(1);
        let height = Height::new(0);
        let timestamp = Timestamp::new(self.config.timestamp);
        let previous_hash = PreviousHash::zero();
        let difficulty = Difficulty::new(self.config.difficulty);
        let nonce = Nonce::new(0);

        let transactions = Vec::new();
        let merkle_root = MerkleRoot::new(vec![0u8; 32]);

        let header = BlockHeader::new(
            version,
            height,
            timestamp,
            previous_hash,
            merkle_root,
            difficulty,
            nonce,
        );

        let metadata = BlockMetadata::new(0, transactions.len());
        let block_hash = Self::compute_hash(&header, hasher);
        let block = Block::new(header, transactions, metadata, block_hash);

        Ok(GenesisBlock::new(block, self.config.clone()))
    }

    fn compute_hash(header: &BlockHeader, hasher: &dyn HashFunction) -> BlockHash {
        let serialized =
            bincode::serialize(header).expect("genesis header serialization is infallible");
        let digest = hasher.hash(&serialized);
        BlockHash::new(crate::HashDigest::new(digest.into_inner()))
    }
}

impl GenesisT for Genesis {
    fn config(&self) -> &crate::types::GenesisConfig {
        self.config()
    }

    fn genesis_block(&self) -> CoreResult<Box<dyn BlockT>> {
        use cryptography::providers::Sha256Provider;
        let hasher = Sha256Provider;
        let genesis = self.generate(&hasher)?;
        Ok(Box::new(genesis.block))
    }
}
