//! Implementations of core traits for concrete types.

use crate::error::CoreResult;
use crate::traits::BlockT;
use crate::types::{Block, BlockHash, BlockHeader, BlockMetadata, Transaction};
use cryptography::core::traits::HashFunction;

impl BlockT for Block {
    fn header(&self) -> &BlockHeader {
        &self.header
    }

    fn transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    fn metadata(&self) -> &BlockMetadata {
        &self.metadata
    }

    fn block_hash(&self) -> &BlockHash {
        &self.block_hash
    }

    fn hash(&self) -> crate::HashDigest {
        use cryptography::providers::Sha256Provider;
        let hasher = Sha256Provider;
        self.hash_with(&hasher)
    }

    fn validate(&self) -> CoreResult<()> {
        if self.transactions.len() != self.metadata.transaction_count {
            return Err(crate::CoreError::InvalidBlock(
                "transaction count mismatch".to_string(),
            ));
        }

        let computed_size = bincode::serialize(self).map(|b| b.len()).unwrap_or(0);
        if computed_size != self.metadata.block_size && self.metadata.block_size > 0 {
            return Err(crate::CoreError::InvalidBlock(
                "block size mismatch".to_string(),
            ));
        }

        Ok(())
    }

    fn serialize(&self) -> CoreResult<Vec<u8>> {
        bincode::serialize(self).map_err(|e| crate::CoreError::Unknown(e.to_string()))
    }

    fn deserialize(data: &[u8]) -> CoreResult<Self>
    where
        Self: Sized,
    {
        bincode::deserialize(data).map_err(|e| crate::CoreError::Unknown(e.to_string()))
    }
}

impl Block {
    pub fn hash_with(&self, hasher: &dyn HashFunction) -> crate::HashDigest {
        let serialized =
            bincode::serialize(&self.header).expect("block header serialization is infallible");
        let digest = hasher.hash(&serialized);
        crate::HashDigest::new(digest.into_inner())
    }
}
