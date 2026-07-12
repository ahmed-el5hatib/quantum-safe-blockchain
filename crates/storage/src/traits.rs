use crate::CoreResult;

pub trait BlockStore: Send + Sync + fmt::Debug {
    fn get_block(&self, hash: &crate::HashDigest) -> CoreResult<Option<Vec<u8>>>;
    fn put_block(&self, hash: &crate::HashDigest, block: &[u8]) -> CoreResult<()>;
    fn delete_block(&self, hash: &crate::HashDigest) -> CoreResult<()>;
    fn contains_block(&self, hash: &crate::HashDigest) -> CoreResult<bool>;
    fn latest_block_hash(&self) -> CoreResult<Option<crate::HashDigest>>;
}

pub trait TransactionStore: Send + Sync + fmt::Debug {
    fn get_transaction(&self, hash: &crate::HashDigest) -> CoreResult<Option<Vec<u8>>>;
    fn put_transaction(&self, hash: &crate::HashDigest, tx: &[u8]) -> CoreResult<()>;
    fn delete_transaction(&self, hash: &crate::HashDigest) -> CoreResult<()>;
    fn contains_transaction(&self, hash: &crate::HashDigest) -> CoreResult<bool>;
}

pub trait StateStore: Send + Sync + fmt::Debug {
    fn get_state(&self, key: &[u8]) -> CoreResult<Option<Vec<u8>>>;
    fn put_state(&self, key: &[u8], value: &[u8]) -> CoreResult<()>;
    fn delete_state(&self, key: &[u8]) -> CoreResult<()>;
    fn contains_state(&self, key: &[u8]) -> CoreResult<bool>;
}

pub trait KeyValueStore: Send + Sync + fmt::Debug {
    fn get(&self, key: &[u8]) -> CoreResult<Option<Vec<u8>>>;
    fn put(&self, key: &[u8], value: &[u8]) -> CoreResult<()>;
    fn delete(&self, key: &[u8]) -> CoreResult<()>;
    fn contains(&self, key: &[u8]) -> CoreResult<bool>;
    fn iterator(
        &self,
        prefix: &[u8],
    ) -> CoreResult<Box<dyn Iterator<Item = CoreResult<(Vec<u8>, Vec<u8>)>> + Send + Sync>>;
}

pub trait SnapshotStore: Send + Sync + fmt::Debug {
    fn create_snapshot(&self) -> CoreResult<Vec<u8>>;
    fn restore_snapshot(&self, snapshot: &[u8]) -> CoreResult<()>;
}
