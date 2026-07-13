//! Comprehensive tests for the QSB Transaction Domain Model.

use cryptography::core::traits::{PublicKey, Signature, SignatureAlgorithm};
use cryptography::providers::{Ed25519Provider, Sha256Provider};

use transaction::builder::TransactionBuilder;
use transaction::types::{OutputType, TransactionType};
use transaction::{
    PublicKeyReference, SignatureContainer, Transaction, TransactionHash, TransactionId,
    TransactionInput, TransactionMetadata, TransactionOutput,
};

fn hasher() -> Sha256Provider {
    Sha256Provider
}

fn prev_id() -> TransactionId {
    // A non-null prior-transaction id (all 0x11 bytes) so inputs are not treated as coinbase.
    TransactionId::from_hex(&"11".repeat(32)).unwrap()
}

fn transfer_tx(n: usize) -> Transaction {
    let mut builder = TransactionBuilder::new(hasher())
        .with_timestamp(1_700_000_000)
        .with_type(TransactionType::Transfer);
    for i in 0..n {
        builder = builder
            .add_input(TransactionInput::new(prev_id(), i as u32))
            .unwrap();
        builder = builder
            .add_output(
                TransactionOutput::new(vec![0xAB; 32], 1_000u64 + i as u64, OutputType::Standard)
                    .unwrap(),
            )
            .unwrap();
    }
    builder.finalize().unwrap()
}

#[test]
fn transaction_creation_basic() {
    let tx = transfer_tx(1);
    assert_eq!(tx.inputs().len(), 1);
    assert_eq!(tx.outputs().len(), 1);
    assert_eq!(tx.transaction_type(), TransactionType::Transfer);
    assert_eq!(tx.version().value(), 1);
}

#[test]
fn builder_chaining_and_metadata() {
    let meta = TransactionMetadata::new()
        .with_fee(50)
        .with_lock_time(1_700_000_001)
        .with_memo("hello")
        .with_extension("identity", vec![9, 9, 9]);
    let tx = TransactionBuilder::new(hasher())
        .with_timestamp(123)
        .with_type(TransactionType::Custom(42))
        .add_input(TransactionInput::new(prev_id(), 0))
        .unwrap()
        .add_output(TransactionOutput::new(vec![1, 2, 3], 500, OutputType::Contract).unwrap())
        .unwrap()
        .attach_metadata(meta.clone())
        .finalize()
        .unwrap();

    assert_eq!(tx.timestamp(), 123);
    assert_eq!(tx.transaction_type(), TransactionType::Custom(42));
    assert_eq!(tx.metadata().fee, 50);
    assert_eq!(tx.metadata().lock_time, 1_700_000_001);
    assert_eq!(tx.metadata().memo.as_deref(), Some("hello"));
    assert_eq!(tx.metadata().extension("identity"), Some(&[9u8, 9, 9][..]));
}

#[test]
fn hash_determinism() {
    let a = transfer_tx(5);
    let b = transfer_tx(5);
    assert_eq!(a.hash(), b.hash());
    assert_eq!(a.id(), b.id());
    assert_eq!(*a.id(), TransactionId::from_hash(a.hash()));
}

#[test]
fn hash_consistency_with_same_inputs_order() {
    // Order matters: swapping inputs changes the canonical encoding and thus the hash.
    let mut b1 = TransactionBuilder::new(hasher());
    b1 = b1.add_input(TransactionInput::new(prev_id(), 0)).unwrap();
    b1 = b1.add_input(TransactionInput::new(prev_id(), 1)).unwrap();
    b1 = b1
        .add_output(TransactionOutput::new(vec![1; 32], 10, OutputType::Standard).unwrap())
        .unwrap();

    let mut b2 = TransactionBuilder::new(hasher());
    b2 = b2.add_input(TransactionInput::new(prev_id(), 1)).unwrap();
    b2 = b2.add_input(TransactionInput::new(prev_id(), 0)).unwrap();
    b2 = b2
        .add_output(TransactionOutput::new(vec![1; 32], 10, OutputType::Standard).unwrap())
        .unwrap();

    let t1 = b1.finalize().unwrap();
    let t2 = b2.finalize().unwrap();
    assert_ne!(t1.hash(), t2.hash());
}

#[test]
fn hash_independent_of_signatures() {
    let unsigned = transfer_tx(1);
    let mut builder = TransactionBuilder::new(hasher()).with_timestamp(1_700_000_000);
    builder = builder
        .add_input(TransactionInput::new(prev_id(), 0))
        .unwrap();
    builder = builder
        .add_output(TransactionOutput::new(vec![0xAB; 32], 1000, OutputType::Standard).unwrap())
        .unwrap();
    builder.add_signature(
        PublicKeyReference::new(vec![0xCC; 32], "Ed25519"),
        vec![0xDD; 64],
    );
    let signed = builder.finalize().unwrap();
    // Hash excludes signatures, so it equals the unsigned equivalent.
    assert_eq!(unsigned.hash(), signed.hash());
    assert_eq!(signed.signatures().len(), 1);
}

#[test]
fn verify_hash_recomputes_correctly() {
    let tx = transfer_tx(3);
    assert!(tx.verify_hash(&hasher()));
    assert_eq!(tx.compute_hash(&hasher()), *tx.hash());
}

#[test]
fn serialization_json_round_trip() {
    let tx = transfer_tx(4);
    let json = tx.to_json().unwrap();
    let restored = Transaction::from_json(&json).unwrap();
    assert_eq!(restored, tx);
    assert_eq!(restored.hash(), tx.hash());
    assert!(restored.verify_hash(&hasher()));
}

#[test]
fn serialization_binary_round_trip() {
    let tx = transfer_tx(4);
    let bytes = tx.to_binary().unwrap();
    let restored = Transaction::from_binary(&bytes).unwrap();
    assert_eq!(restored, tx);
    assert_eq!(restored.hash(), tx.hash());
}

#[test]
fn equality_and_clone() {
    let tx = transfer_tx(2);
    let cloned = tx.clone();
    assert_eq!(tx, cloned);
    assert_eq!(tx.hash(), cloned.hash());
}

#[test]
fn large_transaction() {
    let tx = transfer_tx(1_000);
    assert_eq!(tx.inputs().len(), 1_000);
    assert_eq!(tx.outputs().len(), 1_000);
    assert!(tx.verify_hash(&hasher()));
    let bytes = tx.to_binary().unwrap();
    let restored = Transaction::from_binary(&bytes).unwrap();
    assert_eq!(restored, tx);
}

#[test]
fn empty_transaction_rejected() {
    let err = TransactionBuilder::new(hasher()).finalize();
    assert!(matches!(
        err,
        Err(transaction::TransactionError::MissingInputs(_))
            | Err(transaction::TransactionError::MissingOutputs(_))
    ));
}

#[test]
fn missing_outputs_rejected() {
    let err = TransactionBuilder::new(hasher())
        .add_input(TransactionInput::new(prev_id(), 0))
        .unwrap()
        .finalize();
    assert!(matches!(
        err,
        Err(transaction::TransactionError::MissingOutputs(_))
    ));
}

#[test]
fn duplicate_inputs_rejected() {
    let mut b = TransactionBuilder::new(hasher());
    b = b.add_input(TransactionInput::new(prev_id(), 0)).unwrap();
    let err = b.add_input(TransactionInput::new(prev_id(), 0));
    assert!(matches!(
        err,
        Err(transaction::TransactionError::DuplicateInput(_))
    ));
}

#[test]
fn duplicate_outputs_rejected() {
    let out = TransactionOutput::new(vec![0xAB; 32], 100, OutputType::Standard).unwrap();
    let mut b = TransactionBuilder::new(hasher());
    b = b.add_input(TransactionInput::new(prev_id(), 0)).unwrap();
    b = b.add_output(out.clone()).unwrap();
    let err = b.add_output(out);
    assert!(matches!(
        err,
        Err(transaction::TransactionError::DuplicateOutput(_))
    ));
}

#[test]
fn invalid_amount_rejected() {
    let err = TransactionOutput::new(vec![0xAB; 32], 0, OutputType::Standard);
    assert!(matches!(
        err,
        Err(transaction::TransactionError::InvalidAmount(_))
    ));
}

#[test]
fn invalid_version_rejected() {
    let err = TransactionBuilder::new(hasher())
        .with_version(transaction::TransactionVersion::new(0))
        .add_input(TransactionInput::new(prev_id(), 0))
        .unwrap()
        .add_output(TransactionOutput::new(vec![0xAB; 32], 100, OutputType::Standard).unwrap())
        .unwrap()
        .finalize();
    assert!(matches!(
        err,
        Err(transaction::TransactionError::InvalidVersion(_))
    ));
}

#[test]
fn coinbase_requires_no_inputs() {
    let coinbase = TransactionBuilder::new(hasher())
        .with_type(TransactionType::Coinbase)
        .add_output(TransactionOutput::new(vec![0xAB; 32], 50, OutputType::Standard).unwrap())
        .unwrap()
        .finalize();
    assert!(coinbase.is_ok());
    assert_eq!(
        coinbase.unwrap().transaction_type(),
        TransactionType::Coinbase
    );

    let bad = TransactionBuilder::new(hasher())
        .with_type(TransactionType::Coinbase)
        .add_input(TransactionInput::coinbase())
        .unwrap()
        .add_output(TransactionOutput::new(vec![0xAB; 32], 50, OutputType::Standard).unwrap())
        .unwrap()
        .finalize();
    assert!(matches!(
        bad,
        Err(transaction::TransactionError::InvalidTransaction(_))
    ));
}

#[test]
fn signature_abstraction_is_crypto_agnostic() {
    // The transaction never names Ed25519: it only stores opaque bytes. We sign with Ed25519
    // *outside* the domain model and attach the raw signature.
    let provider = Ed25519Provider;
    let keypair = provider.generate_keypair().unwrap();

    let tx = TransactionBuilder::new(hasher())
        .with_timestamp(1)
        .add_input(TransactionInput::new(prev_id(), 0))
        .unwrap()
        .add_output(TransactionOutput::new(vec![0xAB; 32], 100, OutputType::Standard).unwrap())
        .unwrap()
        .finalize()
        .unwrap();

    let hash_bytes = tx.canonical_bytes();
    let signature = provider.sign(keypair.private_key(), &hash_bytes).unwrap();

    let mut signed_builder = TransactionBuilder::new(hasher()).with_timestamp(1);
    signed_builder = signed_builder
        .add_input(TransactionInput::new(prev_id(), 0))
        .unwrap();
    signed_builder = signed_builder
        .add_output(TransactionOutput::new(vec![0xAB; 32], 100, OutputType::Standard).unwrap())
        .unwrap();
    signed_builder.add_signature(
        PublicKeyReference::new(keypair.public_key().as_bytes().to_vec(), "Ed25519"),
        signature.as_bytes().to_vec(),
    );
    let signed = signed_builder.finalize().unwrap();
    assert_eq!(signed.hash(), tx.hash()); // hash unchanged by signature

    // Verify using the provider (simulating the future validation engine).
    let pk = cryptography::providers::Ed25519PublicKey::from_bytes(
        signed.signatures().entries()[0].key.key_bytes(),
    )
    .unwrap();
    let sig = cryptography::providers::Ed25519Signature::from_bytes(
        signed.signatures().entries()[0].signature.as_slice(),
    )
    .unwrap();
    provider.verify(&pk, &hash_bytes, &sig).unwrap();
}

#[test]
fn signature_container_lookup() {
    let key = PublicKeyReference::new(vec![0x01; 32], "Ed25519");
    let mut container = SignatureContainer::new();
    container.add(key.clone(), vec![0xAA; 64]);
    assert_eq!(container.len(), 1);
    assert_eq!(container.signature_for(&key), Some(&[0xAAu8; 64][..]));
    assert!(container
        .signature_for(&PublicKeyReference::new(vec![0x02; 32], "Ed25519"))
        .is_none());
}

#[test]
fn transaction_id_and_hash_hex() {
    let tx = transfer_tx(1);
    let hex = tx.hash().to_hex();
    assert_eq!(TransactionHash::from_hex(&hex).unwrap(), *tx.hash());
    let id_hex = tx.id().to_hex();
    assert_eq!(TransactionId::from_hex(&id_hex).unwrap(), *tx.id());
}

#[test]
fn merkle_commitment_integration() {
    // The transaction model integrates with the Merkle engine by committing transaction hashes.
    use merkle::MerkleTree;
    use merkle::ProofVerifier;
    let txs: Vec<Transaction> = (0..8).map(|_| transfer_tx(1)).collect();
    let hashes: Vec<Vec<u8>> = txs.iter().map(|t| t.hash().as_bytes().to_vec()).collect();
    let tree = MerkleTree::build_from_hashes(hashes, hasher()).unwrap();
    assert_eq!(tree.leaf_count(), 8);
    let proof = tree.proof(3).unwrap();
    let verifier = ProofVerifier::new(hasher());
    assert!(verifier.verify(&proof));
}
