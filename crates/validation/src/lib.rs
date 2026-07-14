//! # QSB Validation Engine (Milestone 4.4)
//!
//! A reusable, deterministic, trait-based validation framework for the Quantum Safe Blockchain (QSB).
//! It validates **blocks** and **transactions** independently of networking, storage, consensus, and
//! execution, so every future blockchain implementation built on QSB can reuse it unchanged.
//!
//! ## Design Principles
//!
//! - **Trait-based architecture.** Validation is built from [`ValidationRule`]s composed into a
//!   [`ValidationPipeline`], orchestrated by a [`ValidationEngine`].
//! - **Stateless validators.** Rules read only their target and the [`ValidationContext`]; they hold
//!   no mutable state and **never panic**.
//! - **`Result<T, ValidationError>` everywhere.** Every rule and validator returns a
//!   [`ValidationResult`].
//! - **No unsafe, no panics.** All fallible operations surface as errors.
//! - **Crypto-agile.** The engine depends only on the abstract
//!   [`HashFunction`](cryptography::core::traits::HashFunction) trait. The concrete algorithm
//!   (SHA-256, SHA-3, BLAKE3, or a future post-quantum hash) is injected through the context — the
//!   engine never names a specific algorithm.
//! - **Configurable rules.** Each rule can be independently enabled/disabled through the
//!   [`ValidationConfig`] carried by the context.
//!
//! ## Architecture
//!
//! ```text
//!                 ┌─────────────────────────────────────────────┐
//!                 │               ValidationEngine               │
//!                 │  - validate_block / check_block              │
//!                 │  - validate_transaction / check_transaction  │
//!                 └───────────────┬───────────────┬──────────────┘
//!                                 │               │
//!                    ┌────────────▼───────┐  ┌─────▼──────────────────┐
//!                    │ block pipeline     │  │ transaction pipeline    │
//!                    │ (ValidationPipeline│  │ (ValidationPipeline<…>) │
//!                    └─────────┬──────────┘  └──────────┬─────────────┘
//!                              │                        │
//!            ┌─────────────────┼───────────┐          ┌─┼──────────────┐
//!            ▼                 ▼           ▼          ▼ │              ▼
//!      HeaderFormatRule  MerkleRootRule  …    Structure  Amount  Signature …
//!            │                 │                      │
//!            └────────┬────────┴──────────────────────┘
//!                     ▼
//!            ValidationContext (hasher + config + chain facts)
//!                     │
//!                     ▼
//!            HashFunction trait (injected provider)
//! ```
//!
//! ## Validation Lifecycle
//!
//! 1. A caller constructs a [`ValidationEngine`] (typically [`ValidationEngine::with_defaults`]).
//! 2. For each target, the caller builds a [`ValidationContext`] carrying the injected hasher, the
//!    protocol [`ValidationConfig`], and chain facts (clock, previous hash/height, genesis flag).
//! 3. The engine runs the appropriate pipeline. Every **enabled** rule executes against the target;
//!    each rule's timing and outcome are recorded.
//! 4. The engine returns a [`ValidationReport`] summarizing success/failure, every executed rule, the
//!    aggregate execution time, error details, and any warnings.
//!
//! ## Extensibility
//!
//! Add a new check by implementing [`ValidationRule<T>`] for a new type and registering it in a
//! pipeline (or via [`EngineBuilder`]). Because rules are stateless and independently testable, the
//! engine gains coverage without touching existing rules or the orchestrator.
//!
//! ## Performance Considerations
//!
//! - Rules are stateless and run sequentially; per-rule and aggregate timings are recorded for
//!   profiling (see the `cargo bench -p validation` suite).
//! - The Merkle root is recomputed once per block via the generic `merkle` engine; switching to a
//!   faster hasher is a one-line context change.
//! - `fail_fast` (in [`ValidationConfig`]) short-circuits on the first failure for latency-sensitive
//!   paths (e.g. mempool admission), at the cost of a less complete report.

pub mod block;
pub mod context;
pub mod engine;
pub mod error;
pub mod hashes;
pub mod pipeline;
pub mod report;
pub mod rule;
pub mod transaction;
pub mod validator;

mod util;

pub use block::default_block_rules;
pub use context::{ContextBuilder, ValidationConfig, ValidationContext};
pub use engine::{EngineBuilder, ValidationEngine};
pub use error::{ValidationError, ValidationResult};
pub use hashes::{
    block_hash, block_header_bytes, compute_block_hash, compute_merkle_root,
    compute_transaction_hash, transaction_bytes,
};
pub use pipeline::ValidationPipeline;
pub use report::{RuleId, RuleResult, ValidationReport, ValidationStatus, ValidationWarning};
pub use rule::ValidationRule;
pub use transaction::default_transaction_rules;
pub use validator::Validator;
