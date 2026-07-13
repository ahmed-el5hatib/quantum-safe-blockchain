//! Immutable transaction domain types.
//!
//! This module defines the complete, crypto-agnostic transaction model: identifiers, versions,
//! extensible types, inputs, outputs, metadata, signatures, and the immutable [`Transaction`].
//!
//! The model is deliberately storage-, consensus-, and cryptography-agnostic. It depends only on
//! the abstract [`HashFunction`] trait for hashing and stores signatures/keys as opaque bytes, so
//! future post-quantum algorithms integrate without touching this code.

use std::collections::BTreeMap;
use std::fmt;

use cryptography::core::traits::HashFunction;
use serde::{Deserialize, Serialize};

use crate::error::{TransactionError, TransactionResult};

/// Minimum supported transaction version.
pub const MIN_TRANSACTION_VERSION: u32 = 1;
/// Default transaction version.
pub const DEFAULT_TRANSACTION_VERSION: u32 = 1;

// =============================================================================
// Transaction Version
// =============================================================================

/// Transaction protocol version.
///
/// Versions are forward compatible: a node understanding version `N` can still validate the
/// structural parts of version `N+1` transactions, while new semantics are carried in metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TransactionVersion(pub u32);

impl TransactionVersion {
    /// Creates a new version.
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    /// Returns the raw version number.
    pub fn value(&self) -> u32 {
        self.0
    }

    /// Returns the default version.
    pub fn default_version() -> Self {
        Self(DEFAULT_TRANSACTION_VERSION)
    }
}

impl Default for TransactionVersion {
    fn default() -> Self {
        Self(DEFAULT_TRANSACTION_VERSION)
    }
}

impl fmt::Display for TransactionVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// =============================================================================
// Transaction Type
// =============================================================================

/// The high-level category of a transaction.
///
/// The model is extensible: `Custom(u32)` reserves room for future transaction families
/// (identity, multi-signature, time-lock, quantum-migration, etc.) without redesigning the
/// struct. Once a category is standardized it can be promoted to a named variant while `Custom`
/// continues to cover everything else for wire compatibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TransactionType {
    /// A standard value transfer between addresses.
    #[default]
    Transfer,
    /// A coinbase / block-reward transaction (no inputs).
    Coinbase,
    /// A custom or future transaction category identified by a numeric code.
    Custom(u32),
}

impl TransactionType {
    /// Numeric code for the `Transfer` type.
    pub const TRANSFER: u32 = 0;
    /// Numeric code for the `Coinbase` type.
    pub const COINBASE: u32 = 1;

    /// Returns the stable numeric code for this type.
    pub fn code(&self) -> u32 {
        match self {
            TransactionType::Transfer => Self::TRANSFER,
            TransactionType::Coinbase => Self::COINBASE,
            TransactionType::Custom(c) => *c,
        }
    }

    /// Reconstructs a type from its numeric code.
    pub fn from_code(code: u32) -> Self {
        match code {
            Self::TRANSFER => TransactionType::Transfer,
            Self::COINBASE => TransactionType::Coinbase,
            other => TransactionType::Custom(other),
        }
    }

    /// Returns `true` if this is a coinbase transaction.
    pub fn is_coinbase(&self) -> bool {
        matches!(self, TransactionType::Coinbase)
    }
}

// =============================================================================
// Output Type
// =============================================================================

/// The category of a transaction output.
///
/// Like [`TransactionType`], this is open for extension. `Custom(u32)` covers future output
/// kinds (e.g. smart-contract escrow, identity records) without breaking existing code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum OutputType {
    /// A standard payment output.
    #[default]
    Standard,
    /// A smart-contract / script output.
    Contract,
    /// A data-only / OP_RETURN style output.
    Data,
    /// A custom or future output category.
    Custom(u32),
}

impl OutputType {
    /// Numeric code for the `Standard` type.
    pub const STANDARD: u32 = 0;
    /// Numeric code for the `Contract` type.
    pub const CONTRACT: u32 = 1;
    /// Numeric code for the `Data` type.
    pub const DATA: u32 = 2;

    /// Returns the stable numeric code for this output type.
    pub fn code(&self) -> u32 {
        match self {
            OutputType::Standard => Self::STANDARD,
            OutputType::Contract => Self::CONTRACT,
            OutputType::Data => Self::DATA,
            OutputType::Custom(c) => *c,
        }
    }

    /// Reconstructs an output type from its numeric code.
    pub fn from_code(code: u32) -> Self {
        match code {
            Self::STANDARD => OutputType::Standard,
            Self::CONTRACT => OutputType::Contract,
            Self::DATA => OutputType::Data,
            other => OutputType::Custom(other),
        }
    }
}

// =============================================================================
// Transaction Id / Hash
// =============================================================================

/// A unique transaction identifier.
///
/// By convention the id is equal to the transaction [`TransactionHash`]; it is provided as a
/// distinct newtype so APIs can express intent.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(pub Vec<u8>);

impl TransactionId {
    /// Creates an id from raw bytes.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Derives a transaction id from a hash (the standard derivation).
    pub fn from_hash(hash: &TransactionHash) -> Self {
        Self(hash.as_bytes().to_vec())
    }

    /// Returns the id bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns `true` if this is the null id (all zero bytes).
    pub fn is_null(&self) -> bool {
        self.0.iter().all(|b| *b == 0)
    }

    /// Returns the null id, used for coinbase inputs that reference no prior output.
    pub fn null() -> Self {
        Self(vec![0u8; 32])
    }

    /// Encodes the id as a lowercase hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    /// Decodes a hex string into an id.
    ///
    /// # Errors
    ///
    /// Returns [`TransactionError::TransactionSerialization`] if the string is not valid hex.
    pub fn from_hex(s: &str) -> TransactionResult<Self> {
        hex::decode(s).map(Self).map_err(|e| {
            TransactionError::TransactionSerialization(format!("invalid txid hex: {e}"))
        })
    }
}

impl fmt::Display for TransactionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// The cryptographic hash of a transaction.
///
/// The hash is computed (via the [`HashFunction`] trait) over the canonical encoding of the
/// transaction's hash-relevant fields, **excluding** signatures and public-key references. This is
/// the standard "sign-after-hash" model: the hash is stable regardless of which signatures are
/// later attached.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionHash(pub Vec<u8>);

impl TransactionHash {
    /// Creates a hash from raw bytes.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Returns the hash bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Returns `true` if the hash is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Encodes the hash as a lowercase hex string.
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    /// Decodes a hex string into a hash.
    ///
    /// # Errors
    ///
    /// Returns [`TransactionError::TransactionSerialization`] if the string is not valid hex.
    pub fn from_hex(s: &str) -> TransactionResult<Self> {
        hex::decode(s).map(Self).map_err(|e| {
            TransactionError::TransactionSerialization(format!("invalid hash hex: {e}"))
        })
    }
}

impl fmt::Display for TransactionHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

// =============================================================================
// Transaction Input
// =============================================================================

/// A reference to a previous output being spent, plus unlocking material.
///
/// # Input Model
///
/// An input points at a specific output of a previous transaction (`referenced_tx` +
/// `referenced_output_index`). `sequence` enables relative time-locks (future use). `unlocking_data`
/// is the placeholder for signatures/scripts that satisfy the referenced output's locking
/// conditions; `script` and `witness` are reserved for future script and witness programs.
///
/// Coinbase transactions have no prior output, so their single input uses the
/// [`TransactionId::null`] reference.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionInput {
    referenced_tx: TransactionId,
    referenced_output_index: u32,
    sequence: u32,
    unlocking_data: Vec<u8>,
    script: Vec<u8>,
    witness: Vec<u8>,
}

impl TransactionInput {
    /// Creates a new input referencing `referenced_output_index` of `referenced_tx`.
    pub fn new(referenced_tx: TransactionId, referenced_output_index: u32) -> Self {
        Self {
            referenced_tx,
            referenced_output_index,
            sequence: 0,
            unlocking_data: Vec::new(),
            script: Vec::new(),
            witness: Vec::new(),
        }
    }

    /// Creates a coinbase input (references no prior output).
    pub fn coinbase() -> Self {
        Self::new(TransactionId::null(), u32::MAX)
    }

    /// Sets the sequence number (returns `self` for chaining).
    pub fn with_sequence(mut self, sequence: u32) -> Self {
        self.sequence = sequence;
        self
    }

    /// Sets the unlocking data (returns `self` for chaining).
    pub fn with_unlocking_data(mut self, data: Vec<u8>) -> Self {
        self.unlocking_data = data;
        self
    }

    /// Sets the script placeholder (returns `self` for chaining).
    pub fn with_script(mut self, script: Vec<u8>) -> Self {
        self.script = script;
        self
    }

    /// Sets the witness placeholder (returns `self` for chaining).
    pub fn with_witness(mut self, witness: Vec<u8>) -> Self {
        self.witness = witness;
        self
    }

    /// Returns the referenced transaction id.
    pub fn referenced_tx(&self) -> &TransactionId {
        &self.referenced_tx
    }

    /// Returns the referenced output index.
    pub fn referenced_output_index(&self) -> u32 {
        self.referenced_output_index
    }

    /// Returns the sequence number.
    pub fn sequence(&self) -> u32 {
        self.sequence
    }

    /// Returns the unlocking data.
    pub fn unlocking_data(&self) -> &[u8] {
        &self.unlocking_data
    }

    /// Returns the script placeholder.
    pub fn script(&self) -> &[u8] {
        &self.script
    }

    /// Returns the witness placeholder.
    pub fn witness(&self) -> &[u8] {
        &self.witness
    }

    /// Returns `true` if this is a coinbase input (null reference).
    pub fn is_coinbase(&self) -> bool {
        self.referenced_tx.is_null()
    }

    /// Returns the (referenced_tx, index) pair used for duplicate detection.
    pub(crate) fn dedupe_key(&self) -> (Vec<u8>, u32) {
        (self.referenced_tx.0.clone(), self.referenced_output_index)
    }
}

// =============================================================================
// Transaction Output
// =============================================================================

/// A transaction output: a payment or programmatic destination.
///
/// # Output Model
///
/// `recipient` is opaque address bytes (account- or UTXO-agnostic). `amount` is the value.
/// `output_type` selects the output semantics. `locking_data` is the placeholder for the
/// conditions that must be satisfied to spend the output; `contract_data` reserves space for
/// smart-contract / script payloads.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionOutput {
    recipient: Vec<u8>,
    amount: u64,
    output_type: OutputType,
    locking_data: Vec<u8>,
    contract_data: Vec<u8>,
}

impl TransactionOutput {
    /// Creates a new output.
    ///
    /// # Errors
    ///
    /// Returns [`TransactionError::InvalidAmount`] if `amount` is zero.
    pub fn new(
        recipient: Vec<u8>,
        amount: u64,
        output_type: OutputType,
    ) -> TransactionResult<Self> {
        if amount == 0 {
            return Err(TransactionError::InvalidAmount(
                "transaction output amount must be greater than zero".into(),
            ));
        }
        Ok(Self {
            recipient,
            amount,
            output_type,
            locking_data: Vec::new(),
            contract_data: Vec::new(),
        })
    }

    /// Sets the locking data (returns `self` for chaining).
    pub fn with_locking_data(mut self, data: Vec<u8>) -> Self {
        self.locking_data = data;
        self
    }

    /// Sets the contract data (returns `self` for chaining).
    pub fn with_contract_data(mut self, data: Vec<u8>) -> Self {
        self.contract_data = data;
        self
    }

    /// Returns the recipient address bytes.
    pub fn recipient(&self) -> &[u8] {
        &self.recipient
    }

    /// Returns the output amount.
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Returns the output type.
    pub fn output_type(&self) -> OutputType {
        self.output_type
    }

    /// Returns the locking data.
    pub fn locking_data(&self) -> &[u8] {
        &self.locking_data
    }

    /// Returns the contract data.
    pub fn contract_data(&self) -> &[u8] {
        &self.contract_data
    }
}

// =============================================================================
// Public Key Reference
// =============================================================================

/// An opaque reference to a public key.
///
/// The transaction never instantiates a concrete signature type (e.g. Ed25519). It stores the raw
/// public-key bytes together with the algorithm name. Future post-quantum schemes (ML-DSA,
/// Falcon, SPHINCS+) plug in simply by carrying a different algorithm string and byte format,
/// requiring zero changes to this model.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKeyReference {
    key_bytes: Vec<u8>,
    algorithm: String,
}

impl PublicKeyReference {
    /// Creates a reference to a public key given its raw bytes and algorithm name.
    pub fn new(key_bytes: Vec<u8>, algorithm: impl Into<String>) -> Self {
        Self {
            key_bytes,
            algorithm: algorithm.into(),
        }
    }

    /// Returns the public key bytes.
    pub fn key_bytes(&self) -> &[u8] {
        &self.key_bytes
    }

    /// Returns the algorithm name (e.g. `"Ed25519"`, `"ML-DSA"`).
    pub fn algorithm(&self) -> &str {
        &self.algorithm
    }
}

// =============================================================================
// Signature Container
// =============================================================================

/// A single (key, signature) pair.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignatureEntry {
    /// The public key that produced the signature.
    pub key: PublicKeyReference,
    /// The raw signature bytes (algorithm-specific).
    pub signature: Vec<u8>,
}

/// A crypto-agnostic container of transaction signatures.
///
/// Signatures are stored as opaque bytes keyed by [`PublicKeyReference`]. The transaction model
/// performs no verification; it merely carries the material that a future validation engine will
/// check against the configured [`SignatureAlgorithm`](cryptography::core::traits::SignatureAlgorithm).
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SignatureContainer {
    entries: Vec<SignatureEntry>,
}

impl SignatureContainer {
    /// Creates an empty signature container (a placeholder, ready to be filled).
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a container from a list of entries.
    pub fn with_entries(entries: Vec<SignatureEntry>) -> Self {
        Self { entries }
    }

    /// Appends a signature for `key`.
    pub fn add(&mut self, key: PublicKeyReference, signature: Vec<u8>) -> &mut Self {
        self.entries.push(SignatureEntry { key, signature });
        self
    }

    /// Returns `true` if no signatures are present.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Returns the number of signatures.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns the signature entries.
    pub fn entries(&self) -> &[SignatureEntry] {
        &self.entries
    }

    /// Returns an iterator over the entries.
    pub fn iter(&self) -> std::slice::Iter<'_, SignatureEntry> {
        self.entries.iter()
    }

    /// Returns the signature bytes for `key`, if present.
    pub fn signature_for(&self, key: &PublicKeyReference) -> Option<&[u8]> {
        self.entries
            .iter()
            .find(|e| e.key == *key)
            .map(|e| e.signature.as_slice())
    }
}

// =============================================================================
// Transaction Metadata
// =============================================================================

/// Optional, extensible transaction metadata.
///
/// `extensions` is a lexicographically-ordered map (so hashing is deterministic) used to carry
/// forward-compatible data: time-locks, identity claims, smart-contract parameters, quantum
/// migration markers, and more.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TransactionMetadata {
    /// Transaction fee.
    pub fee: u64,
    /// Absolute lock time (Unix seconds / block height, consensus-defined). `0` means unlocked.
    pub lock_time: u64,
    /// Optional human-readable memo.
    pub memo: Option<String>,
    /// Forward-compatible extension fields.
    pub extensions: BTreeMap<String, Vec<u8>>,
}

impl TransactionMetadata {
    /// Creates empty metadata.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the fee (builder-style).
    pub fn with_fee(mut self, fee: u64) -> Self {
        self.fee = fee;
        self
    }

    /// Sets the lock time (builder-style).
    pub fn with_lock_time(mut self, lock_time: u64) -> Self {
        self.lock_time = lock_time;
        self
    }

    /// Sets the memo (builder-style).
    pub fn with_memo(mut self, memo: impl Into<String>) -> Self {
        self.memo = Some(memo.into());
        self
    }

    /// Adds a forward-compatible extension field.
    pub fn with_extension(mut self, key: impl Into<String>, value: Vec<u8>) -> Self {
        self.extensions.insert(key.into(), value);
        self
    }

    /// Returns the value of an extension field, if present.
    pub fn extension(&self, key: &str) -> Option<&[u8]> {
        self.extensions.get(key).map(Vec::as_slice)
    }
}

// =============================================================================
// Transaction
// =============================================================================

/// An immutable, fully-constructed transaction.
///
/// A `Transaction` is produced only by [`TransactionBuilder`](crate::TransactionBuilder) and is
/// immutable thereafter. It carries its own [`TransactionId`] (equal to its [`TransactionHash`]),
/// version, timestamp, type, inputs, outputs, metadata, hash, signatures, and the public-key
/// references needed to verify those signatures.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    id: TransactionId,
    version: TransactionVersion,
    timestamp: u64,
    tx_type: TransactionType,
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
    metadata: TransactionMetadata,
    hash: TransactionHash,
    signatures: SignatureContainer,
    public_keys: Vec<PublicKeyReference>,
}

impl Transaction {
    /// Constructs a transaction. Intended for use by [`TransactionBuilder`](crate::TransactionBuilder).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        version: TransactionVersion,
        timestamp: u64,
        tx_type: TransactionType,
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        metadata: TransactionMetadata,
        hash: TransactionHash,
        signatures: SignatureContainer,
        public_keys: Vec<PublicKeyReference>,
    ) -> Self {
        let id = TransactionId::from_hash(&hash);
        Self {
            id,
            version,
            timestamp,
            tx_type,
            inputs,
            outputs,
            metadata,
            hash,
            signatures,
            public_keys,
        }
    }

    /// Returns the transaction id.
    pub fn id(&self) -> &TransactionId {
        &self.id
    }

    /// Returns the transaction hash.
    pub fn hash(&self) -> &TransactionHash {
        &self.hash
    }

    /// Returns the version.
    pub fn version(&self) -> TransactionVersion {
        self.version
    }

    /// Returns the timestamp (Unix seconds).
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Returns the transaction type.
    pub fn transaction_type(&self) -> TransactionType {
        self.tx_type
    }

    /// Returns the inputs.
    pub fn inputs(&self) -> &[TransactionInput] {
        &self.inputs
    }

    /// Returns the outputs.
    pub fn outputs(&self) -> &[TransactionOutput] {
        &self.outputs
    }

    /// Returns the metadata.
    pub fn metadata(&self) -> &TransactionMetadata {
        &self.metadata
    }

    /// Returns the signature container.
    pub fn signatures(&self) -> &SignatureContainer {
        &self.signatures
    }

    /// Returns the referenced public keys.
    pub fn public_keys(&self) -> &[PublicKeyReference] {
        &self.public_keys
    }

    /// Returns the canonical byte encoding used for hashing (excludes signatures/keys).
    pub fn canonical_bytes(&self) -> Vec<u8> {
        encode_hash_input(
            self.version,
            self.timestamp,
            self.tx_type,
            &self.inputs,
            &self.outputs,
            &self.metadata,
        )
    }

    /// Recomputes the transaction hash with `hasher` and checks it against the stored hash.
    ///
    /// This lets a future validation engine confirm the transaction has not been tampered with,
    /// independent of any signature check.
    pub fn verify_hash<H: HashFunction>(&self, hasher: &H) -> bool {
        let bytes = self.canonical_bytes();
        let recomputed = TransactionHash::new(hasher.hash(&bytes).into_inner());
        recomputed == self.hash
    }

    /// Recomputes the transaction hash using `hasher` (independent of the stored hash).
    pub fn compute_hash<H: HashFunction>(&self, hasher: &H) -> TransactionHash {
        TransactionHash::new(hasher.hash(&self.canonical_bytes()).into_inner())
    }

    /// Serializes the transaction to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns [`TransactionError::TransactionSerialization`] on failure.
    pub fn to_json(&self) -> TransactionResult<String> {
        serde_json::to_string(self)
            .map_err(|e| TransactionError::TransactionSerialization(format!("json encode: {e}")))
    }

    /// Deserializes a transaction from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns [`TransactionError::TransactionSerialization`] on failure.
    pub fn from_json(data: &str) -> TransactionResult<Self> {
        serde_json::from_str(data)
            .map_err(|e| TransactionError::TransactionSerialization(format!("json decode: {e}")))
    }

    /// Serializes the transaction to a `bincode` binary form.
    ///
    /// # Errors
    ///
    /// Returns [`TransactionError::TransactionSerialization`] on failure.
    pub fn to_binary(&self) -> TransactionResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| TransactionError::TransactionSerialization(format!("binary encode: {e}")))
    }

    /// Deserializes a transaction from its `bincode` binary form.
    ///
    /// # Errors
    ///
    /// Returns [`TransactionError::TransactionSerialization`] on failure.
    pub fn from_binary(data: &[u8]) -> TransactionResult<Self> {
        bincode::deserialize(data)
            .map_err(|e| TransactionError::TransactionSerialization(format!("binary decode: {e}")))
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transaction(id={}, version={}, type={}, inputs={}, outputs={})",
            self.id,
            self.version,
            self.tx_type.code(),
            self.inputs.len(),
            self.outputs.len(),
        )
    }
}

// =============================================================================
// Canonical hashing
// =============================================================================

/// Appends `data` to `buf` as a length-prefixed (u32 little-endian) byte string.
fn put_bytes(buf: &mut Vec<u8>, data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u32).to_le_bytes());
    buf.extend_from_slice(data);
}

/// Produces the deterministic canonical byte encoding of a transaction's hash-relevant fields.
///
/// The encoding is order- and length-prefixed, and the `extensions` map is iterated in key order
/// (it is a `BTreeMap`), so the output is identical for equal transactions regardless of how the
/// transaction was constructed. This is the sole input to the transaction hash.
pub(crate) fn encode_hash_input(
    version: TransactionVersion,
    timestamp: u64,
    tx_type: TransactionType,
    inputs: &[TransactionInput],
    outputs: &[TransactionOutput],
    metadata: &TransactionMetadata,
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&version.value().to_le_bytes());
    buf.extend_from_slice(&timestamp.to_le_bytes());
    buf.extend_from_slice(&tx_type.code().to_le_bytes());

    buf.extend_from_slice(&(inputs.len() as u32).to_le_bytes());
    for input in inputs {
        put_bytes(&mut buf, input.referenced_tx.as_bytes());
        buf.extend_from_slice(&input.referenced_output_index.to_le_bytes());
        buf.extend_from_slice(&input.sequence.to_le_bytes());
        put_bytes(&mut buf, &input.unlocking_data);
        put_bytes(&mut buf, &input.script);
        put_bytes(&mut buf, &input.witness);
    }

    buf.extend_from_slice(&(outputs.len() as u32).to_le_bytes());
    for output in outputs {
        put_bytes(&mut buf, &output.recipient);
        buf.extend_from_slice(&output.amount.to_le_bytes());
        buf.extend_from_slice(&output.output_type.code().to_le_bytes());
        put_bytes(&mut buf, &output.locking_data);
        put_bytes(&mut buf, &output.contract_data);
    }

    buf.extend_from_slice(&metadata.fee.to_le_bytes());
    buf.extend_from_slice(&metadata.lock_time.to_le_bytes());
    match &metadata.memo {
        Some(memo) => {
            buf.push(1);
            put_bytes(&mut buf, memo.as_bytes());
        }
        None => buf.push(0),
    }
    buf.extend_from_slice(&(metadata.extensions.len() as u32).to_le_bytes());
    for (key, value) in &metadata.extensions {
        put_bytes(&mut buf, key.as_bytes());
        put_bytes(&mut buf, value);
    }

    buf
}
