#[cfg(test)]
mod tests {
    use blockchain_core::traits::{BlockT, GenesisT};
    use blockchain_core::*;
    use cryptography::providers::Sha256Provider;

    #[test]
    fn test_version_new() {
        let v = Version::new(1);
        assert_eq!(v.value(), 1);
        assert_eq!(v, Version::new(1));
    }

    #[test]
    fn test_height_new() {
        let h = Height::new(100);
        assert_eq!(h.value(), 100);
        assert_eq!(h, Height::new(100));
    }

    #[test]
    fn test_difficulty_new() {
        let d = Difficulty::new(4);
        assert_eq!(d.value(), 4);
        assert_eq!(d, Difficulty::new(4));
    }

    #[test]
    fn test_nonce_new() {
        let n = Nonce::new(42);
        assert_eq!(n.value(), 42);
        assert_eq!(n, Nonce::new(42));
    }

    #[test]
    fn test_timestamp_new() {
        let t = Timestamp::new(1234567890);
        assert_eq!(t.value(), 1234567890);
        assert_eq!(t, Timestamp::new(1234567890));
    }

    #[test]
    fn test_previous_hash_zero() {
        let zero = PreviousHash::zero();
        assert_eq!(zero.inner().as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn test_block_hash_new() {
        let digest = HashDigest::new(vec![1u8; 32]);
        let block_hash = BlockHash::new(digest.clone());
        assert_eq!(block_hash.inner().as_bytes(), digest.as_bytes());
    }

    #[test]
    fn test_block_header_creation() {
        let header = BlockHeader::new(
            Version::new(1),
            Height::new(0),
            Timestamp::new(1000),
            PreviousHash::zero(),
            MerkleRoot::new(vec![0u8; 32]),
            Difficulty::new(1),
            Nonce::new(0),
        );
        assert_eq!(header.version, Version::new(1));
        assert_eq!(header.height, Height::new(0));
        assert_eq!(header.difficulty, Difficulty::new(1));
    }

    #[test]
    fn test_block_header_serialization() {
        let header = BlockHeader::new(
            Version::new(1),
            Height::new(0),
            Timestamp::new(1000),
            PreviousHash::zero(),
            MerkleRoot::new(vec![0u8; 32]),
            Difficulty::new(1),
            Nonce::new(0),
        );
        let json = serde_json::to_string(&header).unwrap();
        let restored: BlockHeader = serde_json::from_str(&json).unwrap();
        assert_eq!(header, restored);
    }

    #[test]
    fn test_block_creation() {
        let header = BlockHeader::new(
            Version::new(1),
            Height::new(0),
            Timestamp::new(1000),
            PreviousHash::zero(),
            MerkleRoot::new(vec![0u8; 32]),
            Difficulty::new(1),
            Nonce::new(0),
        );
        let transactions = vec![Transaction::new(1, vec![], vec![], 0)];
        let metadata = BlockMetadata::new(100, transactions.len());
        let block_hash = BlockHash::new(HashDigest::new(vec![0u8; 32]));

        let block = Block::new(header, transactions.clone(), metadata, block_hash);
        assert_eq!(block.transactions.len(), 1);
        assert_eq!(block.metadata.transaction_count, 1);
    }

    #[test]
    fn test_block_serialization_json() {
        let header = BlockHeader::new(
            Version::new(1),
            Height::new(0),
            Timestamp::new(1000),
            PreviousHash::zero(),
            MerkleRoot::new(vec![0u8; 32]),
            Difficulty::new(1),
            Nonce::new(0),
        );
        let transactions = vec![Transaction::new(1, vec![], vec![], 0)];
        let metadata = BlockMetadata::new(100, transactions.len());
        let block_hash = BlockHash::new(HashDigest::new(vec![0u8; 32]));

        let block = Block::new(header, transactions, metadata, block_hash);
        let json = block.to_json().unwrap();
        let restored = Block::from_json(&json).unwrap();
        assert_eq!(block, restored);
    }

    #[test]
    fn test_block_serialization_binary() {
        let header = BlockHeader::new(
            Version::new(1),
            Height::new(0),
            Timestamp::new(1000),
            PreviousHash::zero(),
            MerkleRoot::new(vec![0u8; 32]),
            Difficulty::new(1),
            Nonce::new(0),
        );
        let transactions = vec![Transaction::new(1, vec![], vec![], 0)];
        let metadata = BlockMetadata::new(100, transactions.len());
        let block_hash = BlockHash::new(HashDigest::new(vec![0u8; 32]));

        let block = Block::new(header, transactions, metadata, block_hash);
        let binary = block.to_binary().unwrap();
        let restored = Block::from_binary(&binary).unwrap();
        assert_eq!(block, restored);
    }

    #[test]
    fn test_block_hash_consistency() {
        let header = BlockHeader::new(
            Version::new(1),
            Height::new(0),
            Timestamp::new(1000),
            PreviousHash::zero(),
            MerkleRoot::new(vec![0u8; 32]),
            Difficulty::new(1),
            Nonce::new(0),
        );
        let transactions = vec![Transaction::new(1, vec![], vec![], 0)];
        let metadata = BlockMetadata::new(100, transactions.len());
        let block_hash = BlockHash::new(HashDigest::new(vec![0u8; 32]));

        let block = Block::new(header, transactions, metadata, block_hash);
        let hasher = Sha256Provider;
        let hash1 = block.hash_with(&hasher);
        let hash2 = block.hash_with(&hasher);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_block_hash_trait() {
        let header = BlockHeader::new(
            Version::new(1),
            Height::new(0),
            Timestamp::new(1000),
            PreviousHash::zero(),
            MerkleRoot::new(vec![0u8; 32]),
            Difficulty::new(1),
            Nonce::new(0),
        );
        let transactions = vec![Transaction::new(1, vec![], vec![], 0)];
        let metadata = BlockMetadata::new(100, transactions.len());
        let block_hash = BlockHash::new(HashDigest::new(vec![0u8; 32]));

        let block = Block::new(header, transactions, metadata, block_hash);
        let hash1 = block.hash();
        let hash2 = block.hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_block_clone() {
        let header = BlockHeader::new(
            Version::new(1),
            Height::new(0),
            Timestamp::new(1000),
            PreviousHash::zero(),
            MerkleRoot::new(vec![0u8; 32]),
            Difficulty::new(1),
            Nonce::new(0),
        );
        let transactions = vec![Transaction::new(1, vec![], vec![], 0)];
        let metadata = BlockMetadata::new(100, transactions.len());
        let block_hash = BlockHash::new(HashDigest::new(vec![0u8; 32]));

        let block = Block::new(header, transactions, metadata, block_hash);
        let cloned = block.clone();
        assert_eq!(block, cloned);
    }

    #[test]
    fn test_block_equality() {
        let header = BlockHeader::new(
            Version::new(1),
            Height::new(0),
            Timestamp::new(1000),
            PreviousHash::zero(),
            MerkleRoot::new(vec![0u8; 32]),
            Difficulty::new(1),
            Nonce::new(0),
        );
        let transactions = vec![Transaction::new(1, vec![], vec![], 0)];
        let metadata = BlockMetadata::new(100, transactions.len());
        let block_hash = BlockHash::new(HashDigest::new(vec![0u8; 32]));

        let block1 = Block::new(
            header.clone(),
            transactions.clone(),
            metadata.clone(),
            block_hash.clone(),
        );
        let block2 = Block::new(header, transactions, metadata, block_hash);
        assert_eq!(block1, block2);
    }

    #[test]
    fn test_genesis_determinism() {
        let config = GenesisConfig::new(1000000, 1, 0, "Genesis".to_string());
        let genesis1 = Genesis::new(config.clone());
        let genesis2 = Genesis::new(config);

        let hasher = Sha256Provider;
        let block1 = genesis1.generate(&hasher).unwrap();
        let block2 = genesis2.generate(&hasher).unwrap();

        assert_eq!(block1.block, block2.block);
        assert_eq!(block1.config, block2.config);
        assert_eq!(block1.block.block_hash, block2.block.block_hash);
    }

    #[test]
    fn test_genesis_block_hash() {
        let config = GenesisConfig::new(1000000, 1, 0, "Genesis".to_string());
        let genesis = Genesis::new(config);
        let hasher = Sha256Provider;
        let genesis_block = genesis.generate(&hasher).unwrap();

        assert_eq!(genesis_block.block.header.height, Height::new(0));
        assert_eq!(genesis_block.block.header.difficulty, Difficulty::new(1));
        assert_eq!(genesis_block.block.transactions.len(), 0);
    }

    #[test]
    fn test_genesis_trait() {
        let config = GenesisConfig::new(1000000, 1, 0, "Genesis".to_string());
        let genesis = Genesis::new(config);

        let block = genesis.genesis_block().unwrap();
        assert_eq!(block.header().height, Height::new(0));
    }

    #[test]
    fn test_block_metadata() {
        let metadata = BlockMetadata::new(256, 3);
        assert_eq!(metadata.block_size, 256);
        assert_eq!(metadata.transaction_count, 3);
    }

    #[test]
    fn test_chain_metadata() {
        let metadata = ChainMetadata::new(100, 500, 10000);
        assert_eq!(metadata.total_blocks, 100);
        assert_eq!(metadata.total_transactions, 500);
        assert_eq!(metadata.chain_work, 10000);
    }

    #[test]
    fn test_empty_transactions() {
        let header = BlockHeader::new(
            Version::new(1),
            Height::new(0),
            Timestamp::new(1000),
            PreviousHash::zero(),
            MerkleRoot::new(vec![0u8; 32]),
            Difficulty::new(1),
            Nonce::new(0),
        );
        let transactions: Vec<Transaction> = vec![];
        let metadata = BlockMetadata::new(0, 0);
        let block_hash = BlockHash::new(HashDigest::new(vec![0u8; 32]));

        let block = Block::new(header, transactions, metadata, block_hash);
        assert!(block.transactions.is_empty());
    }

    #[test]
    fn test_block_validate_success() {
        let header = BlockHeader::new(
            Version::new(1),
            Height::new(0),
            Timestamp::new(1000),
            PreviousHash::zero(),
            MerkleRoot::new(vec![0u8; 32]),
            Difficulty::new(1),
            Nonce::new(0),
        );
        let transactions = vec![Transaction::new(1, vec![], vec![], 0)];
        let block_hash = BlockHash::new(HashDigest::new(vec![0u8; 32]));
        let binary_size = bincode::serialize(&Block::new(
            header.clone(),
            transactions.clone(),
            BlockMetadata::new(0, 0),
            block_hash.clone(),
        ))
        .unwrap()
        .len();
        let metadata = BlockMetadata::new(binary_size, transactions.len());
        let block_hash = BlockHash::new(HashDigest::new(vec![0u8; 32]));

        let block = Block::new(header, transactions, metadata, block_hash);
        assert!(block.validate().is_ok());
    }

    #[test]
    fn test_block_validate_transaction_count_mismatch() {
        let header = BlockHeader::new(
            Version::new(1),
            Height::new(0),
            Timestamp::new(1000),
            PreviousHash::zero(),
            MerkleRoot::new(vec![0u8; 32]),
            Difficulty::new(1),
            Nonce::new(0),
        );
        let transactions = vec![Transaction::new(1, vec![], vec![], 0)];
        let metadata = BlockMetadata::new(100, 999);
        let block_hash = BlockHash::new(HashDigest::new(vec![0u8; 32]));

        let block = Block::new(header, transactions, metadata, block_hash);
        assert!(block.validate().is_err());
    }
}
