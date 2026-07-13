//! # QSB Transaction Domain Model
//!
//! An immutable, extensible, and **agnostic** transaction model for the Quantum Safe Blockchain
//! (QSB) ecosystem. This crate is Milestone 4.3: it implements only the *domain objects* that
//! future components (validation, mempool, consensus, storage, wallet) will consume. It contains
//! **no validation logic, no state, no networking, and no signing**.
//!
//! ## Design Goals
//!
//! - **Blockchain agnostic** — no consensus- or chain-specific assumptions.
//! - **Cryptography agnostic** — depends only on the abstract [`HashFunction`](cryptography::core::traits::HashFunction)
//!   and [`SignatureAlgorithm`](cryptography::core::traits::SignatureAlgorithm) traits. It never
//!   names Ed25519 (or any algorithm) directly; signatures and keys are stored as opaque bytes
//!   plus an algorithm label.
//! - **Consensus agnostic** — the same model serves UTXO, account-based, and smart-contract chains.
//! - **Storage / network agnostic** — full Serde, JSON, and binary (`bincode`) support.
//! - **Extensible** — [`TransactionType`], [`OutputType`], and metadata `extensions` reserve room
//!   for future families (multi-sig, time-lock, identity, quantum migration) without redesign.
//!
//! ## Transaction Anatomy
//!
//! ```text
//! Transaction
//! ├── id            (= hash)
//! ├── version
//! ├── timestamp
//! ├── type          (Transfer | Coinbase | Custom(u32))
//! ├── inputs[]      (referenced_tx, output_index, sequence, unlocking/script/witness)
//! ├── outputs[]     (recipient, amount, type, locking/contract data)
//! ├── metadata      (fee, lock_time, memo, extensions)
//! ├── hash          (canonical hash, excluding signatures)
//! ├── signatures    (opaque key→signature entries)
//! └── public_keys[] (opaque key references)
//! ```
//!
//! ## Input / Output Model
//!
//! An [`TransactionInput`] points at a specific prior output (`referenced_tx` + index). UTXO chains
//! spend these directly; account-based chains can treat the set of inputs as a senders' authorization
//! list. [`TransactionOutput`] carries an opaque `recipient` (address bytes), an `amount`, an
//! `output_type`, and `locking_data` / `contract_data` placeholders for spending conditions and
//! smart-contract payloads.
//!
//! ## Builder Pattern
//!
//! Construction is performed exclusively through [`TransactionBuilder`], which validates structure
//! (duplicate inputs/outputs, zero amounts, missing inputs/outputs, invalid versions) *before*
//! producing an immutable [`Transaction`].
//!
//! ## Signature Abstraction
//!
//! The transaction stores only [`SignatureContainer`] (raw signature bytes) and
//! [`PublicKeyReference`] (raw key bytes + algorithm name). Verification is delegated to a future
//! validation engine via the `SignatureAlgorithm` trait, so ML-DSA / Falcon / SPHINCS+ integrate
//! by supplying a different provider — no change to this crate.
//!
//! ## Hashing Strategy
//!
//! The hash is computed (through the `HashFunction` trait) over the **canonical encoding** of the
//! hash-relevant fields — version, timestamp, type, inputs, outputs, metadata — **excluding**
//! signatures and public keys. This makes the hash stable regardless of which signatures are later
//! attached ("sign after hash"), and lets any future hash provider (SHA-3, BLAKE3) be swapped in
//! without changing the model. Use [`Transaction::verify_hash`] to re-check integrity.
//!
//! ## Future Compatibility
//!
//! - **UTXO**: spend via `inputs` referencing prior `outputs`; amounts/locking data unchanged.
//! - **Account model**: interpret `inputs` as authorizations and `recipient` as an account id.
//! - **Smart contracts**: use `output_type::Contract` + `contract_data`; `extensions` for params.
//! - **Multi-sig / time-lock / identity / quantum migration**: `Custom` type codes and `extensions`
//!   keys, with signatures carried opaquely for whichever algorithm is active.

pub mod builder;
pub mod error;
pub mod traits;
pub mod types;
pub mod validator;

pub use error::{TransactionError, TransactionResult};
pub use traits::TransactionT;

pub use builder::TransactionBuilder;
pub use types::{
    OutputType, PublicKeyReference, SignatureContainer, SignatureEntry, Transaction,
    TransactionHash, TransactionId, TransactionInput, TransactionMetadata, TransactionOutput,
    TransactionType, TransactionVersion,
};

pub use types::MIN_TRANSACTION_VERSION;

impl TransactionT for Transaction {
    fn id(&self) -> &TransactionId {
        self.id()
    }

    fn hash(&self) -> &TransactionHash {
        self.hash()
    }

    fn version(&self) -> TransactionVersion {
        self.version()
    }

    fn transaction_type(&self) -> TransactionType {
        self.transaction_type()
    }

    fn inputs(&self) -> &[TransactionInput] {
        self.inputs()
    }

    fn outputs(&self) -> &[TransactionOutput] {
        self.outputs()
    }
}
