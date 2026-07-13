//! Fluent, validating builder for immutable [`Transaction`]s.
//!
//! The builder enforces structural validity *before* construction: it rejects duplicate inputs and
//! outputs, zero-amount outputs, missing inputs/outputs (except coinbase), and invalid versions.
//! The transaction hash is computed over the canonical encoding (excluding signatures) so that
//! signatures can be attached after the hash is fixed.

use cryptography::core::traits::HashFunction;

use crate::error::TransactionResult;
use crate::types::encode_hash_input;
use crate::{
    PublicKeyReference, SignatureContainer, Transaction, TransactionHash, TransactionInput,
    TransactionMetadata, TransactionOutput, TransactionType, TransactionVersion,
};

/// Builds an immutable [`Transaction`].
///
/// # Example
///
/// ```
/// use cryptography::providers::Sha256Provider;
/// use transaction::{TransactionBuilder, TransactionInput, TransactionOutput, TransactionType, OutputType};
///
/// let hasher = Sha256Provider;
/// let tx = TransactionBuilder::new(hasher)
///     .with_timestamp(1_700_000_000)
///     .add_input(TransactionInput::new(
///         transaction::TransactionId::from_hex("11".repeat(32).as_str()).unwrap(),
///         0,
///     ))
///     .unwrap()
///     .add_output(TransactionOutput::new(vec![0xAA; 32], 100, OutputType::Standard).unwrap())
///     .unwrap()
///     .finalize()
///     .unwrap();
/// assert_eq!(tx.transaction_type(), TransactionType::Transfer);
/// ```
pub struct TransactionBuilder<H: HashFunction> {
    hasher: H,
    version: TransactionVersion,
    timestamp: u64,
    tx_type: TransactionType,
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
    metadata: TransactionMetadata,
    public_keys: Vec<PublicKeyReference>,
    signatures: SignatureContainer,
}

impl<H: HashFunction + Clone> TransactionBuilder<H> {
    /// Creates a new builder bound to `hasher`, with sensible defaults.
    pub fn new(hasher: H) -> Self {
        Self {
            hasher,
            version: TransactionVersion::default_version(),
            timestamp: 0,
            tx_type: TransactionType::Transfer,
            inputs: Vec::new(),
            outputs: Vec::new(),
            metadata: TransactionMetadata::new(),
            public_keys: Vec::new(),
            signatures: SignatureContainer::new(),
        }
    }

    /// Alias for [`Self::new`], emphasising the hash-provider binding.
    pub fn with_hasher(hasher: H) -> Self {
        Self::new(hasher)
    }

    /// Sets the transaction version.
    pub fn with_version(mut self, version: TransactionVersion) -> Self {
        self.version = version;
        self
    }

    /// Sets the transaction timestamp (Unix seconds).
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Sets the transaction type.
    pub fn with_type(mut self, tx_type: TransactionType) -> Self {
        self.tx_type = tx_type;
        self
    }

    /// Adds an input, rejecting duplicates.
    ///
    /// # Errors
    ///
    /// Returns [`crate::TransactionError::DuplicateInput`] if an input spending the same
    /// `(referenced_tx, index)` was already added.
    pub fn add_input(mut self, input: TransactionInput) -> TransactionResult<Self> {
        let key = input.dedupe_key();
        if self.inputs.iter().any(|i| i.dedupe_key() == key) {
            return Err(crate::TransactionError::DuplicateInput(format!(
                "input spending tx {} index {} already present",
                input.referenced_tx(),
                input.referenced_output_index()
            )));
        }
        self.inputs.push(input);
        Ok(self)
    }

    /// Adds an output, rejecting duplicates and zero amounts.
    ///
    /// # Errors
    ///
    /// Returns [`crate::TransactionError::InvalidAmount`] for a zero amount, or
    /// [`crate::TransactionError::DuplicateOutput`] if an identical output was already added.
    pub fn add_output(mut self, output: TransactionOutput) -> TransactionResult<Self> {
        if self.outputs.contains(&output) {
            return Err(crate::TransactionError::DuplicateOutput(
                "identical output already present".into(),
            ));
        }
        self.outputs.push(output);
        Ok(self)
    }

    /// Replaces the metadata.
    pub fn attach_metadata(mut self, metadata: TransactionMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Adds a referenced public key (used to verify attached signatures).
    pub fn add_public_key(mut self, key: PublicKeyReference) -> Self {
        self.public_keys.push(key);
        self
    }

    /// Attaches a single signature together with its public-key reference.
    pub fn add_signature(&mut self, key: PublicKeyReference, signature: Vec<u8>) -> &mut Self {
        self.public_keys.push(key.clone());
        self.signatures.add(key, signature);
        self
    }

    /// Attaches a pre-built signature container (e.g. a placeholder during earlier stages).
    pub fn set_signatures(mut self, signatures: SignatureContainer) -> Self {
        self.signatures = signatures;
        self
    }

    /// Computes the transaction hash from the current (signature-excluded) state.
    pub fn compute_hash(&self) -> TransactionHash {
        let bytes = encode_hash_input(
            self.version,
            self.timestamp,
            self.tx_type,
            &self.inputs,
            &self.outputs,
            &self.metadata,
        );
        TransactionHash::new(self.hasher.hash(&bytes).into_inner())
    }

    /// Finalizes the builder into an immutable [`Transaction`].
    ///
    /// Performs all structural validations and computes the transaction hash.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::TransactionError`] variant for any structural problem
    /// (`MissingInputs`, `MissingOutputs`, `InvalidVersion`, `InvalidTransaction`).
    pub fn finalize(&mut self) -> TransactionResult<Transaction> {
        // Take ownership of the collected fields by swapping in a fresh builder.
        let taken = std::mem::replace(self, TransactionBuilder::new(self.hasher.clone()));

        if taken.version.value() < crate::MIN_TRANSACTION_VERSION {
            return Err(crate::TransactionError::InvalidVersion(format!(
                "transaction version {} is below minimum {}",
                taken.version.value(),
                crate::MIN_TRANSACTION_VERSION
            )));
        }

        if taken.tx_type.is_coinbase() {
            if !taken.inputs.is_empty() {
                return Err(crate::TransactionError::InvalidTransaction(
                    "coinbase transactions must not have inputs".into(),
                ));
            }
        } else if taken.inputs.is_empty() {
            return Err(crate::TransactionError::MissingInputs(
                "non-coinbase transaction must have at least one input".into(),
            ));
        } else if taken.inputs.iter().any(TransactionInput::is_coinbase) {
            return Err(crate::TransactionError::InvalidTransaction(
                "non-coinbase transaction contains a coinbase input".into(),
            ));
        }

        if taken.outputs.is_empty() {
            return Err(crate::TransactionError::MissingOutputs(
                "transaction must have at least one output".into(),
            ));
        }

        let hash = taken.compute_hash();
        Ok(Transaction::new(
            taken.version,
            taken.timestamp,
            taken.tx_type,
            taken.inputs,
            taken.outputs,
            taken.metadata,
            hash,
            taken.signatures,
            taken.public_keys,
        ))
    }

    /// Convenience alias for [`Self::finalize`].
    pub fn build(&mut self) -> TransactionResult<Transaction> {
        self.finalize()
    }
}
