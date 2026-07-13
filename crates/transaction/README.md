# transaction

Immutable, extensible, and **agnostic** transaction domain model for the Quantum Safe
Blockchain (QSB) ecosystem.

This crate is Milestone 4.3. It implements **only the immutable transaction domain
objects** that future components (validation, mempool, consensus, storage, wallet) will
consume. It contains **no validation logic, no state, no networking, and no signing**.

## Design Goals

- **Blockchain agnostic** — no consensus- or chain-specific assumptions.
- **Cryptography agnostic** — depends only on the abstract `HashFunction` and
  `SignatureAlgorithm` traits from the [`cryptography`](../cryptography) crate. It never
  names Ed25519 (or any algorithm) directly; signatures and keys are stored as opaque bytes
  plus an algorithm label.
- **Consensus agnostic** — the same model serves UTXO, account-based, and smart-contract
  chains.
- **Storage / network agnostic** — full Serde, JSON, and binary (`bincode`) support.
- **Extensible** — `TransactionType`, `OutputType`, and metadata `extensions` reserve room
  for future families (multi-sig, time-lock, identity, quantum migration) without redesign.

## Transaction Anatomy

```text
Transaction
├── id            (= hash)
├── version
├── timestamp
├── type          (Transfer | Coinbase | Custom(u32))
├── inputs[]      (referenced_tx, output_index, sequence, unlocking/script/witness)
├── outputs[]     (recipient, amount, type, locking/contract data)
├── metadata      (fee, lock_time, memo, extensions)
├── hash          (canonical hash, excluding signatures)
├── signatures    (opaque key→signature entries)
└── public_keys[] (opaque key references)
```

The immutable [`Transaction`] is constructed exclusively through
[`TransactionBuilder`]. It carries its own `TransactionId` (equal to its
`TransactionHash`), version, timestamp, type, inputs, outputs, metadata, hash,
signatures, and the public-key references needed to verify those signatures.

## Input / Output Model

A [`TransactionInput`] points at a specific prior output (`referenced_tx` + index). UTXO
chains spend these directly; account-based chains can treat the set of inputs as a senders'
authorization list. `sequence` enables relative time-locks (future use); `unlocking_data`,
`script`, and `witness` are placeholders for the spending material, script programs, and
witness data of future validation engines.

A [`TransactionOutput`] carries an opaque `recipient` (address bytes), an `amount`, an
`output_type`, and `locking_data` / `contract_data` placeholders for spending conditions
and smart-contract payloads.

## Builder Pattern

Construction is performed exclusively through [`TransactionBuilder`], which validates
structure (duplicate inputs/outputs, zero amounts, missing inputs/outputs, invalid
versions) **before** producing an immutable `Transaction`.

```rust,no_run
use cryptography::providers::Sha256Provider;
use transaction::{TransactionBuilder, TransactionInput, TransactionOutput, OutputType};

let tx = TransactionBuilder::new(Sha256Provider)
    .with_timestamp(1_700_000_000)
    .add_input(TransactionInput::new(
        transaction::TransactionId::from_hex(&"11".repeat(32)).unwrap(),
        0,
    ))
    .unwrap()
    .add_output(TransactionOutput::new(vec![0xAA; 32], 100, OutputType::Standard).unwrap())
    .unwrap()
    .finalize()
    .unwrap();

assert_eq!(tx.transaction_type(), transaction::TransactionType::Transfer);
```

## Signature Abstraction

The transaction stores only [`SignatureContainer`] (raw signature bytes) and
[`PublicKeyReference`] (raw key bytes + algorithm name). Verification is delegated to a
future validation engine via the `SignatureAlgorithm` trait, so ML-DSA / Falcon / SPHINCS+
integrate by supplying a different provider — no change to this crate. The transaction
itself never instantiates a concrete signature type.

## Hashing Strategy

The hash is computed (through the `HashFunction` trait) over the **canonical encoding** of
the hash-relevant fields — version, timestamp, type, inputs, outputs, metadata —
**excluding** signatures and public keys. This makes the hash stable regardless of which
signatures are later attached ("sign after hash"), and lets any future hash provider
(SHA-3, BLAKE3) be swapped in without changing the model. Use
[`Transaction::verify_hash`] to re-check integrity.

## Serialization

- **Serde** derive on every domain type.
- **JSON** via `Transaction::to_json` / `Transaction::from_json`.
- **Binary** via `Transaction::to_binary` / `Transaction::from_binary` (`bincode`).

## Future Compatibility

- **UTXO**: spend via `inputs` referencing prior `outputs`; amounts/locking data unchanged.
- **Account model**: interpret `inputs` as authorizations and `recipient` as an account id.
- **Smart contracts**: use `OutputType::Contract` + `contract_data`; `extensions` for params.
- **Multi-sig / time-lock / identity / quantum migration**: `TransactionType::Custom` codes
  and `extensions` keys, with signatures carried opaquely for whichever algorithm is active.

## Integration with the Ecosystem

- **Blockchain Core** ([`blockchain-core`](../blockchain-core)): `Transaction` reuses the
  unified `CoreError`/`CoreResult` types and is the unit committed into `Block` structures.
- **Merkle Tree** ([`merkle`](../merkle)): transaction hashes (`TransactionHash`) are the
  leaves committed into the block merkle root; the hash is stable so commitments never shift
  once signatures are added.
- **Cryptography Layer** ([`cryptography`](../cryptography)): hashing is performed only
  through the `HashFunction` trait, and signatures/keys are carried as opaque,
  algorithm-labelled bytes via the `SignatureAlgorithm` abstraction.
- **Future Validation Engine**: reads the `TransactionT` trait view, recomputes/verifies the
  hash via `Transaction::verify_hash`, and verifies `SignatureContainer` entries against the
  active `SignatureAlgorithm` provider — none of which require changes to this crate.

## Testing & Benchmarks

```text
cargo test   -p transaction   # 20+ unit/integration tests
cargo bench  -p transaction   # construction, hashing, builder, serialization, large txs
```

## Module Layout

```text
transaction/src
├── lib.rs        # crate documentation + re-exports
├── types.rs      # Transaction, TransactionId, TransactionVersion, TransactionType,
│                 #   TransactionInput, TransactionOutput, TransactionMetadata,
│                 #   TransactionHash, SignatureContainer, PublicKeyReference, OutputType
├── builder.rs    # TransactionBuilder (validating, immutable construction)
├── traits.rs     # TransactionT read-only abstraction
├── error.rs      # TransactionError / TransactionResult (re-export of CoreError)
├── validator.rs  # reserved for the future validation engine (out of scope)
└── tests/        # comprehensive integration tests
└── benches/      # criterion benchmarks
```
