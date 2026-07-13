use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("invalid block: {0}")]
    InvalidBlock(String),

    #[error("invalid transaction: {0}")]
    InvalidTransaction(String),

    #[error("invalid header: {0}")]
    InvalidHeader(String),

    #[error("chain validation failed: {0}")]
    ChainValidation(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("unknown error: {0}")]
    Unknown(String),

    #[error("invalid version: {0}")]
    InvalidVersion(String),

    #[error("invalid height: {0}")]
    InvalidHeight(String),

    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("invalid difficulty: {0}")]
    InvalidDifficulty(String),

    #[error("invalid nonce: {0}")]
    InvalidNonce(String),

    #[error("invalid previous hash: {0}")]
    InvalidPreviousHash(String),

    #[error("invalid merkle root: {0}")]
    InvalidMerkleRoot(String),

    #[error("genesis config error: {0}")]
    GenesisConfig(String),

    #[error("missing transaction inputs: {0}")]
    MissingInputs(String),

    #[error("missing transaction outputs: {0}")]
    MissingOutputs(String),

    #[error("invalid transaction amount: {0}")]
    InvalidAmount(String),

    #[error("duplicate transaction input: {0}")]
    DuplicateInput(String),

    #[error("duplicate transaction output: {0}")]
    DuplicateOutput(String),

    #[error("transaction serialization error: {0}")]
    TransactionSerialization(String),

    #[error("empty merkle tree: {0}")]
    EmptyTree(String),

    #[error("invalid merkle proof: {0}")]
    InvalidProof(String),

    #[error("invalid merkle leaf: {0}")]
    InvalidLeaf(String),

    #[error("invalid merkle tree: {0}")]
    InvalidTree(String),
}

pub type CoreResult<T> = Result<T, CoreError>;
