use crate::CoreResult;

pub trait ConsensusEngine: Send + Sync + fmt::Debug {
    type Block: crate::BlockT;
    type Error: std::error::Error + Send + Sync + 'static;

    fn name(&self) -> &str;
    async fn validate_block(&self, block: &Self::Block) -> CoreResult<()>;
    async fn propose_block(&self, parent: &Self::Block) -> CoreResult<Self::Block>;
    async fn finalize_block(&self, block: &Self::Block) -> CoreResult<()>;
    async fn should_finalize(&self, block: &Self::Block) -> CoreResult<bool>;
}

pub trait ConsensusParams: Send + Sync + fmt::Debug + Clone {
    fn validate(&self) -> CoreResult<()>;
}

pub trait ConsensusState: Send + Sync + fmt::Debug {
    fn apply_block(&mut self, block: &dyn crate::BlockT) -> CoreResult<()>;
    fn revert_block(&mut self, block: &dyn crate::BlockT) -> CoreResult<()>;
}
