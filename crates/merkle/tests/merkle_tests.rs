//! Comprehensive tests for the QSB Merkle Tree engine.

use cryptography::core::traits::HashFunction;
use cryptography::providers::Sha256Provider;

use merkle::builder::TreeBuilder;
use merkle::proof::{MerkleProof, ProofStep, ProofVerifier};
use merkle::root::MerkleRoot;
use merkle::MerkleTree;

fn hasher() -> Sha256Provider {
    Sha256Provider
}

/// Builds a tree from `n` distinct leaves ("leaf-0" .. "leaf-(n-1)").
fn tree_of(n: usize) -> MerkleTree<Sha256Provider> {
    let leaves: Vec<Vec<u8>> = (0..n).map(|i| format!("leaf-{i}").into_bytes()).collect();
    MerkleTree::build(leaves, hasher()).unwrap()
}

#[test]
fn single_leaf_root_equals_hash_of_leaf() {
    let data = b"only-leaf";
    let tree = MerkleTree::build(vec![data.as_ref()], hasher()).unwrap();
    assert_eq!(tree.leaf_count(), 1);
    assert_eq!(tree.height(), 0);
    let expected = hasher().hash(data).into_inner();
    assert_eq!(tree.root_hash_bytes(), expected);
    assert_eq!(tree.root().as_bytes(), expected.as_slice());
}

#[test]
fn single_leaf_proof_verifies() {
    let tree = MerkleTree::build(vec![b"only-leaf".as_ref()], hasher()).unwrap();
    let proof = tree.proof(0).unwrap();
    assert!(proof.steps.is_empty());
    let verifier = ProofVerifier::new(hasher());
    assert!(verifier.verify(&proof));
    assert!(tree.verify_proof(0, hasher().hash(b"only-leaf").as_bytes(), &proof));
}

#[test]
fn two_leaves_root_is_hash_of_pair() {
    let a = hasher().hash(b"a").into_inner();
    let b = hasher().hash(b"b").into_inner();
    let tree = MerkleTree::build_from_hashes(vec![a.clone(), b.clone()], hasher()).unwrap();
    let mut expected = a.clone();
    expected.extend_from_slice(&b);
    let expected = hasher().hash(&expected).into_inner();
    assert_eq!(tree.root_hash_bytes(), expected);
    assert_eq!(tree.height(), 1);

    let proof = tree.proof(0).unwrap();
    assert_eq!(proof.steps.len(), 1);
    assert!(matches!(proof.steps[0], ProofStep::Right { .. }));
    let verifier = ProofVerifier::new(hasher());
    assert!(verifier.verify(&proof));
}

#[test]
fn three_leaves_uses_duplicate_last_leaf() {
    let tree = tree_of(3);
    assert_eq!(tree.leaf_count(), 3);
    // 3 leaves -> 2 internal -> 1 root => height 2.
    assert_eq!(tree.height(), 2);

    let root = tree.root_hash_bytes();
    for i in 0..3 {
        let proof = tree.proof(i).unwrap();
        let verifier = ProofVerifier::new(hasher());
        assert!(verifier.verify(&proof), "leaf {i} proof must verify");
        assert!(verifier.verify_against(&proof, &root));
    }
}

#[test]
fn odd_leaf_counts_from_1_to_33_all_verify() {
    for n in 1..=33 {
        let tree = tree_of(n);
        let root = tree.root_hash_bytes();
        let verifier = ProofVerifier::new(hasher());
        for i in 0..n {
            let proof = tree.proof(i).unwrap();
            assert_eq!(proof.leaf_index, i);
            assert!(verifier.verify_against(&proof, &root), "leaf {i}/{n}");
        }
    }
}

#[test]
fn large_tree_1000_leaves_all_proofs_verify() {
    let tree = tree_of(1000);
    assert_eq!(tree.leaf_count(), 1000);
    let root = tree.root_hash_bytes();
    let verifier = ProofVerifier::new(hasher());

    // Spot-check every 37th leaf plus the boundaries.
    let indices: Vec<usize> = (0..1000).step_by(37).chain([0, 999]).collect();
    for i in indices {
        let proof = tree.proof(i).unwrap();
        assert!(verifier.verify_against(&proof, &root), "leaf {i}");
    }
}

#[test]
fn build_is_deterministic() {
    let leaves: Vec<Vec<u8>> = (0..16).map(|i| format!("L{i}").into_bytes()).collect();
    let t1 = MerkleTree::build(leaves.clone(), hasher()).unwrap();
    let t2 = MerkleTree::build(leaves, hasher()).unwrap();
    assert_eq!(t1.root_hash_bytes(), t2.root_hash_bytes());
    assert_eq!(t1.to_json(), t2.to_json());
}

#[test]
fn contains_detects_leaves() {
    let tree = tree_of(10);
    let present = hasher().hash(b"leaf-4").into_inner();
    let absent = hasher().hash(b"leaf-99").into_inner();
    assert!(tree.contains(&present));
    assert!(!tree.contains(&absent));
    assert_eq!(tree.leaf(4), Some(present.as_slice()));
    assert_eq!(tree.leaf(10), None);
    assert_eq!(tree.leaves().len(), 10);
}

#[test]
fn empty_tree_is_rejected() {
    let err = MerkleTree::build::<Vec<u8>>(vec![], hasher());
    assert!(matches!(err, Err(merkle::CoreError::EmptyTree(_))));
    let builder_err = TreeBuilder::new(hasher()).build();
    assert!(matches!(builder_err, Err(merkle::CoreError::EmptyTree(_))));
}

#[test]
fn odd_tree_without_duplicate_is_rejected() {
    let leaves: Vec<Vec<u8>> = (0..3).map(|i| format!("x{i}").into_bytes()).collect();
    let tree = TreeBuilder::new(hasher())
        .with_leaves(leaves)
        .with_duplicate_last_leaf(false)
        .build();
    assert!(matches!(tree, Err(merkle::CoreError::InvalidTree(_))));
}

#[test]
fn proof_for_out_of_bounds_index_is_rejected() {
    let tree = tree_of(5);
    assert!(matches!(
        tree.proof(5),
        Err(merkle::CoreError::InvalidLeaf(_))
    ));
}

#[test]
fn invalid_proof_detected() {
    let tree = tree_of(8);
    let mut proof = tree.proof(3).unwrap();
    let tampered = proof.clone();

    // Corrupt a sibling hash.
    if let Some(ProofStep::Right { hash } | ProofStep::Left { hash }) = proof.steps.first_mut() {
        hash[0] ^= 0xFF;
    }
    let verifier = ProofVerifier::new(hasher());
    assert!(!verifier.verify(&proof));

    // Wrong expected root.
    let wrong_root = vec![0u8; 32];
    assert!(!verifier.verify_against(&tampered, &wrong_root));

    // Wrong leaf hash via the verifier.
    assert!(!verifier.verify_leaf(3, &[0u8; 32], &tampered, &tree.root_hash_bytes()));
}

#[test]
fn serialization_json_round_trip() {
    let tree = tree_of(17);
    let json = tree.to_json();
    let restored = MerkleTree::from_json(&json, hasher()).unwrap();
    assert_eq!(restored.root_hash_bytes(), tree.root_hash_bytes());
    assert_eq!(restored.leaf_count(), tree.leaf_count());
    assert_eq!(restored.height(), tree.height());

    // Proofs round-trip too.
    let proof = tree.proof(9).unwrap();
    let proof_json = proof.to_json().unwrap();
    let restored_proof = MerkleProof::from_json(&proof_json).unwrap();
    let verifier = ProofVerifier::new(hasher());
    assert!(verifier.verify(&restored_proof));
}

#[test]
fn serialization_binary_round_trip() {
    let tree = tree_of(24);
    let bytes = tree.to_binary().unwrap();
    let restored = MerkleTree::from_binary(&bytes, hasher()).unwrap();
    assert_eq!(restored.root_hash_bytes(), tree.root_hash_bytes());
    assert_eq!(restored.leaves(), tree.leaves());

    let proof = tree.proof(7).unwrap();
    let proof_bytes = proof.to_binary().unwrap();
    let restored_proof = MerkleProof::from_binary(&proof_bytes).unwrap();
    let verifier = ProofVerifier::new(hasher());
    assert!(verifier.verify(&restored_proof));
}

#[test]
fn corrupted_binary_tree_is_rejected() {
    let tree = tree_of(12);
    let bytes = tree.to_binary().unwrap();
    let root = tree.root_hash_bytes();

    // Truncation reliably breaks deserialization.
    let mut truncated = bytes.clone();
    truncated.pop();
    assert!(MerkleTree::from_binary(&truncated, hasher()).is_err());

    // Tampering the stored root hash is detected: the deserialized root no longer matches the
    // trusted root. Serialization formats carry no integrity check of their own; the hash
    // commitment is what provides tamper-evidence.
    let start = bytes
        .windows(root.len())
        .position(|w| w == root.as_slice())
        .expect("root hash must appear in the serialized tree");
    let mut corrupted = bytes;
    corrupted[start + 1] ^= 0xAA;
    let restored = MerkleTree::from_binary(&corrupted, hasher()).unwrap();
    assert_ne!(restored.root_hash_bytes(), root);

    // Malformed JSON is rejected.
    assert!(MerkleTree::from_json("{ not json", hasher()).is_err());
}

#[test]
fn corrupted_json_proof_is_rejected() {
    let result = MerkleProof::from_json("{ not a proof");
    assert!(matches!(result, Err(merkle::CoreError::InvalidProof(_))));
}

#[test]
fn merkle_root_hex_and_blockchain_conversion() {
    let tree = tree_of(6);
    let root: MerkleRoot = tree.root();
    let hex = root.to_hex();
    let decoded = MerkleRoot::from_hex(&hex).unwrap();
    assert_eq!(decoded, root);

    // Lossless conversion with blockchain_core::MerkleRoot.
    let bc_root: blockchain_core::MerkleRoot = root.clone().into();
    let back: MerkleRoot = bc_root.clone().into();
    assert_eq!(back, root);
    assert_eq!(bc_root.as_bytes(), root.as_bytes());
}

#[test]
fn blockchain_merkletreet_verify() {
    // Build a tree whose leaves are already hashes, matching the blockchain-core trait usage.
    let leaf_hashes: Vec<Vec<u8>> = (0..8)
        .map(|i| hasher().hash(format!("tx-{i}").as_bytes()).into_inner())
        .collect();
    let tree = MerkleTree::build_from_hashes(leaf_hashes.clone(), hasher()).unwrap();

    let root = blockchain_core::traits::MerkleTreeT::root(&tree);
    let verifier_root = tree.root_hash_bytes();

    for (i, leaf) in leaf_hashes.iter().enumerate().take(8) {
        // Build the sibling-only proof expected by the blockchain-core trait.
        let engine_proof = tree.proof(i).unwrap();
        let sibling_proof: Vec<blockchain_core::MerkleRoot> = engine_proof
            .steps
            .iter()
            .map(|s| blockchain_core::MerkleRoot::new(s.hash().to_vec()))
            .collect();
        let ok = blockchain_core::traits::MerkleTreeT::verify(&tree, i, leaf, &sibling_proof);
        assert!(ok, "blockchain verify failed for leaf {i}");
        assert_eq!(root.as_bytes(), verifier_root.as_slice());
    }

    // A tampered leaf must fail.
    let bad = blockchain_core::traits::MerkleTreeT::verify(&tree, 0, &[0u8; 32], &[]);
    assert!(!bad);
}

#[test]
fn duplicate_last_leaf_strategy_preserves_verification() {
    // 5 leaves -> last duplicated at first level.
    let tree = tree_of(5);
    let verifier = ProofVerifier::new(hasher());
    let root = tree.root_hash_bytes();
    for i in 0..5 {
        let proof = tree.proof(i).unwrap();
        assert!(verifier.verify_against(&proof, &root));
    }
}

#[test]
fn depth_and_height_aliases_agree() {
    let tree = tree_of(64);
    assert_eq!(tree.depth(), tree.height());
    assert_eq!(tree.height(), 6);
}
