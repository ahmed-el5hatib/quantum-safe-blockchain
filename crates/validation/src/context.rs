//! Validation context and configuration.
//!
//! The [`ValidationContext`] is the single, immutable (but cheaply cloneable) bundle that every
//! validation rule receives. It carries the injected [`HashFunction`] provider, the protocol
//! configuration ([`ValidationConfig`]), and the *chain tip* facts that contextual validation needs
//! (current clock, previous block hash/height, and whether the target is the genesis block).
//!
//! By routing the hasher through the context rather than hard-coding an algorithm, the engine
//! remains crypto-agile: swapping SHA-256 for SHA-3, BLAKE3, or a post-quantum hash requires only a
//! different provider at construction time — no rule code changes.

use std::collections::HashMap;
use std::sync::Arc;

use cryptography::core::traits::HashFunction;

/// Protocol and policy limits used by the validation rules.
///
/// All limits are intentionally conservative and overridable. The default values are safe for a
/// research blockchain; production deployments tune them through the [`ValidationContext`] builder.
#[derive(Clone, Debug)]
pub struct ValidationConfig {
    /// Maximum accepted serialized block size in bytes (informational upper bound).
    pub max_block_size: usize,
    /// Maximum number of transactions a single block may carry.
    pub max_transaction_count: usize,
    /// Minimum accepted block version (inclusive).
    pub min_block_version: u32,
    /// Maximum accepted block version (inclusive).
    pub max_block_version: u32,
    /// Minimum accepted transaction version (inclusive).
    pub min_transaction_version: u32,
    /// Maximum accepted transaction version (inclusive).
    pub max_transaction_version: u32,
    /// Minimum accepted block timestamp (Unix seconds).
    pub min_timestamp: u64,
    /// How far into the future a block timestamp may be, relative to the context clock (seconds).
    pub max_future_timestamp: u64,
    /// Maximum allowed signature byte length in a transaction input (placeholder validation).
    pub max_signature_len: usize,
    /// Maximum allowed script/witness byte length in a transaction input (placeholder validation).
    pub max_script_len: usize,
    /// Maximum accepted lock time (Unix seconds / block height, consensus-defined).
    pub max_lock_time: u64,
    /// Maximum accepted total output value across a transaction (in smallest units).
    pub max_output_value: u128,
    /// The Merkle root committed by a block that carries zero transactions (e.g. genesis).
    ///
    /// Stored explicitly so the engine is independent of any single algorithm's digest size.
    pub empty_merkle_root: Vec<u8>,
    /// The height of the genesis block.
    pub genesis_height: u64,
    /// Whether genesis-specific constraints are enforced when the context marks a block as genesis.
    pub enforce_genesis_constraints: bool,
    /// When `true`, the pipeline stops after the first failing rule.
    pub fail_fast: bool,
    /// Per-rule enable/disable overrides, keyed by rule id. Absent keys fall back to the rule's own
    /// `enabled_by_default` value.
    rule_overrides: HashMap<String, bool>,
}

impl ValidationConfig {
    /// Creates a configuration tuned for a hasher that produces `digest_size`-byte digests.
    ///
    /// The `empty_merkle_root` is set to `digest_size` zero bytes, matching the genesis convention
    /// used by `blockchain-core`.
    pub fn default_with_digest_size(digest_size: usize) -> Self {
        Self {
            max_block_size: 8 * 1024 * 1024,
            max_transaction_count: 100_000,
            min_block_version: 1,
            max_block_version: 1,
            min_transaction_version: 1,
            max_transaction_version: 1,
            min_timestamp: 1_234_567_890,
            max_future_timestamp: 2 * 60 * 60,
            max_signature_len: 4096,
            max_script_len: 4096,
            max_lock_time: u64::MAX,
            max_output_value: u128::MAX,
            empty_merkle_root: vec![0u8; digest_size],
            genesis_height: 0,
            enforce_genesis_constraints: true,
            fail_fast: false,
            rule_overrides: HashMap::new(),
        }
    }

    /// Overrides the enabled flag for a rule by id. `None` clears an override.
    pub fn set_rule_enabled(&mut self, id: impl Into<String>, enabled: bool) {
        self.rule_overrides.insert(id.into(), enabled);
    }

    /// Returns the explicit override for a rule, if one was configured.
    pub fn rule_enabled_override(&self, id: &str) -> Option<bool> {
        self.rule_overrides.get(id).copied()
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        // 32-byte digest is the most common case (SHA-256); callers constructing a context for a
        // different hasher should use `default_with_digest_size`.
        Self::default_with_digest_size(32)
    }
}

/// The full evaluation context handed to every validation rule.
///
/// The context is cheap to clone (the hasher is reference-counted) so it can be derived per block
/// without allocation pressure. It is intentionally *immutable* from the rule's point of view: rules
/// read configuration and chain facts but never mutate the context.
#[derive(Clone)]
pub struct ValidationContext {
    hasher: Arc<dyn HashFunction>,
    config: ValidationConfig,
    clock: Option<u64>,
    prev_block_hash: Option<Vec<u8>>,
    prev_height: Option<u64>,
    is_genesis: bool,
}

impl ValidationContext {
    /// Creates a context with the supplied hasher and default configuration.
    ///
    /// The default empty Merkle root length is derived from the hasher's digest size.
    pub fn new(hasher: Arc<dyn HashFunction>) -> Self {
        let hash_len = hasher.digest_size();
        Self {
            hasher,
            config: ValidationConfig::default_with_digest_size(hash_len),
            clock: None,
            prev_block_hash: None,
            prev_height: None,
            is_genesis: false,
        }
    }

    /// Starts a builder pre-bound to `hasher`.
    pub fn builder(hasher: Arc<dyn HashFunction>) -> ContextBuilder {
        ContextBuilder::new(hasher)
    }

    /// Returns the reference-counted hash provider.
    pub fn hasher(&self) -> &Arc<dyn HashFunction> {
        &self.hasher
    }

    /// Returns the hash provider as a trait object (for passing to hashing helpers).
    pub fn hasher_ref(&self) -> &dyn HashFunction {
        self.hasher.as_ref()
    }

    /// Returns the digest size (in bytes) of the configured hash provider.
    pub fn digest_size(&self) -> usize {
        self.hasher.digest_size()
    }

    /// Returns the protocol configuration.
    pub fn config(&self) -> &ValidationConfig {
        &self.config
    }

    /// Returns the current consensus clock (Unix seconds), if set.
    pub fn clock(&self) -> Option<u64> {
        self.clock
    }

    /// Returns the previous block's hash, if the target follows an existing tip.
    pub fn prev_block_hash(&self) -> Option<&[u8]> {
        self.prev_block_hash.as_deref()
    }

    /// Returns the previous block's height, if the target follows an existing tip.
    pub fn prev_height(&self) -> Option<u64> {
        self.prev_height
    }

    /// Returns `true` if the target is the genesis block.
    pub fn is_genesis(&self) -> bool {
        self.is_genesis
    }

    /// Returns a copy of this context with the configuration replaced by `config`.
    ///
    /// Useful for layering rule overrides or custom limits onto a context already configured with
    /// chain facts (clock, previous hash/height, genesis flag).
    pub fn with_config(mut self, config: ValidationConfig) -> Self {
        self.config = config;
        self
    }
}

/// Fluent builder for [`ValidationContext`].
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use cryptography::providers::Sha256Provider;
/// use validation::ValidationContext;
///
/// let ctx = ValidationContext::builder(Arc::new(Sha256Provider))
///     .with_clock(1_700_000_000)
///     .with_prev_height(41)
///     .build();
/// assert_eq!(ctx.prev_height(), Some(41));
/// ```
pub struct ContextBuilder {
    inner: ValidationContext,
}

impl ContextBuilder {
    /// Creates a builder pre-bound to `hasher` with default configuration.
    pub fn new(hasher: Arc<dyn HashFunction>) -> Self {
        Self {
            inner: ValidationContext::new(hasher),
        }
    }

    /// Sets the consensus clock (Unix seconds).
    pub fn with_clock(mut self, clock: u64) -> Self {
        self.inner.clock = Some(clock);
        self
    }

    /// Sets the previous block's hash (the expected `previous_hash` of the next block).
    pub fn with_prev_block_hash(mut self, hash: Vec<u8>) -> Self {
        self.inner.prev_block_hash = Some(hash);
        self
    }

    /// Sets the previous block's height.
    pub fn with_prev_height(mut self, height: u64) -> Self {
        self.inner.prev_height = Some(height);
        self
    }

    /// Marks the target as the genesis block (enables genesis constraints).
    pub fn with_genesis(mut self, is_genesis: bool) -> Self {
        self.inner.is_genesis = is_genesis;
        self
    }

    /// Replaces the entire configuration.
    pub fn with_config(mut self, config: ValidationConfig) -> Self {
        self.inner.config = config;
        self
    }

    /// Overrides a single configuration limit.
    pub fn with_limit(mut self, key: impl FnOnce(&mut ValidationConfig)) -> Self {
        key(&mut self.inner.config);
        self
    }

    /// Enables or disables a rule by id for this context.
    pub fn with_rule_enabled(mut self, id: impl Into<String>, enabled: bool) -> Self {
        self.inner.config.set_rule_enabled(id, enabled);
        self
    }

    /// Finalizes the builder into an immutable [`ValidationContext`].
    pub fn build(self) -> ValidationContext {
        self.inner
    }
}
