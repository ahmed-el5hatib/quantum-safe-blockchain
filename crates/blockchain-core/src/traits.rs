use std::fmt;

use crate::types::{
    BlockHash, BlockHeader, BlockMetadata, ChainState, HashDigest, MerkleRoot, Transaction,
    TransactionInput, TransactionOutput,
};
use crate::CoreResult;

pub trait BlockT: Send + Sync + fmt::Debug {
    fn header(&self) -> &BlockHeader;
    fn transactions(&self) -> &[Transaction];
    fn metadata(&self) -> &BlockMetadata;
    fn block_hash(&self) -> &BlockHash;
    fn hash(&self) -> HashDigest;
    fn validate(&self) -> CoreResult<()>;
    fn serialize(&self) -> CoreResult<Vec<u8>>;
    fn deserialize(data: &[u8]) -> CoreResult<Self>
    where
        Self: Sized;
}

pub trait TransactionT: Send + Sync + fmt::Debug {
    fn hash(&self) -> HashDigest;
    fn inputs(&self) -> &[TransactionInput];
    fn outputs(&self) -> &[TransactionOutput];
    fn validate(&self) -> CoreResult<()>;
    fn serialize(&self) -> CoreResult<Vec<u8>>;
    fn deserialize(data: &[u8]) -> CoreResult<Self>
    where
        Self: Sized;
}

pub trait MerkleTreeT: Send + Sync {
    fn root(&self) -> MerkleRoot;
    fn verify(&self, index: usize, leaf: &[u8], proof: &[MerkleRoot]) -> bool;
}

pub trait ChainValidatorT: Send + Sync {
    fn validate_block<B: BlockT>(&self, block: &B, state: &ChainState) -> CoreResult<()>;
    fn validate_chain<B: BlockT>(&self, blocks: &[B]) -> CoreResult<ChainState>;
}

pub trait StateT: Send + Sync {
    fn apply_block<B: BlockT>(&mut self, block: &B) -> CoreResult<()>;
    fn revert_block<B: BlockT>(&mut self, block: &B) -> CoreResult<()>;
    fn current_state(&self) -> &ChainState;
}

pub trait GenesisT: Send + Sync {
    fn config(&self) -> &crate::types::GenesisConfig;
    fn genesis_block(&self) -> CoreResult<Box<dyn BlockT>>;
}
