//! Block validation rules.
//!
//! Each rule implements [`ValidationRule<Block>`](crate::rule::ValidationRule) and checks one
//! concern from the milestone specification: header format, hash consistency, Merkle root
//! correctness, timestamp validity, height consistency, previous-hash presence, genesis constraints,
//! and metadata consistency. Rules are stateless and independently testable; the
//! [`default_block_rules`] set is what the [`ValidationEngine`](crate::ValidationEngine) runs by
//! default.

use blockchain_core::Block;

use crate::context::ValidationContext;
use crate::error::{ValidationError, ValidationResult};
use crate::hashes::{compute_block_hash, compute_merkle_root};
use crate::report::RuleId;
use crate::report::ValidationWarning;
use crate::rule::ValidationRule;
use crate::util::{is_all_zero, to_hex};

/// Validates the structural format of the block header.
pub struct HeaderFormatRule;

impl ValidationRule<Block> for HeaderFormatRule {
    fn id(&self) -> RuleId {
        RuleId::new("block.header_format")
    }
    fn description(&self) -> &'static str {
        "block header fields are within protocol ranges and well-formed"
    }
    fn validate(
        &self,
        block: &Block,
        ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        let cfg = ctx.config();
        let header = &block.header;

        let version = header.version.value();
        if version < cfg.min_block_version || version > cfg.max_block_version {
            return Err(ValidationError::InvalidHeader(format!(
                "version {version} out of range [{}, {}]",
                cfg.min_block_version, cfg.max_block_version
            )));
        }

        if header.difficulty.value() == 0 {
            return Err(ValidationError::InvalidHeader(
                "difficulty must be greater than zero".into(),
            ));
        }

        let root = header.merkle_root.as_bytes();
        let hash_len = ctx.digest_size();
        if root.len() != hash_len && root != cfg.empty_merkle_root {
            return Err(ValidationError::InvalidHeader(format!(
                "merkle root length {} does not match hash length {hash_len} (or the empty root)",
                root.len()
            )));
        }

        Ok(())
    }
}

/// Verifies that `block.block_hash` equals the hash of the header.
pub struct BlockHashConsistencyRule;

impl ValidationRule<Block> for BlockHashConsistencyRule {
    fn id(&self) -> RuleId {
        RuleId::new("block.hash_consistency")
    }
    fn description(&self) -> &'static str {
        "block hash is consistent with the header"
    }
    fn validate(
        &self,
        block: &Block,
        ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        let expected = compute_block_hash(&block.header, ctx.hasher().as_ref())?;
        let actual = block.block_hash.inner().as_bytes();
        if expected != actual {
            return Err(ValidationError::InvalidHash(format!(
                "block hash mismatch: expected {}, got {}",
                to_hex(&expected),
                to_hex(actual)
            )));
        }
        Ok(())
    }
}

/// Verifies that the header's Merkle root matches the transactions in the block.
pub struct MerkleRootRule;

impl ValidationRule<Block> for MerkleRootRule {
    fn id(&self) -> RuleId {
        RuleId::new("block.merkle_root")
    }
    fn description(&self) -> &'static str {
        "merkle root commits exactly to the block's transactions"
    }
    fn validate(
        &self,
        block: &Block,
        ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        let expected = compute_merkle_root(
            &block.transactions,
            ctx.hasher(),
            &ctx.config().empty_merkle_root,
        )?;
        let actual = block.header.merkle_root.as_bytes();
        if expected != actual {
            return Err(ValidationError::InvalidMerkleRoot(format!(
                "merkle root mismatch: expected {}, got {}",
                to_hex(&expected),
                to_hex(actual)
            )));
        }
        Ok(())
    }
}

/// Validates the block timestamp against the configured window.
pub struct TimestampValidityRule;

impl ValidationRule<Block> for TimestampValidityRule {
    fn id(&self) -> RuleId {
        RuleId::new("block.timestamp")
    }
    fn description(&self) -> &'static str {
        "block timestamp is within the acceptable window"
    }
    fn validate(
        &self,
        block: &Block,
        ctx: &ValidationContext,
        warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        let cfg = ctx.config();
        let ts = block.header.timestamp.value();

        if ts < cfg.min_timestamp {
            return Err(ValidationError::InvalidTimestamp(format!(
                "timestamp {ts} is before the minimum {}",
                cfg.min_timestamp
            )));
        }

        if let Some(clock) = ctx.clock() {
            if ts > clock + cfg.max_future_timestamp {
                return Err(ValidationError::InvalidTimestamp(format!(
                    "timestamp {ts} is too far in the future (clock {clock}, max drift {})",
                    cfg.max_future_timestamp
                )));
            }
            if ts > clock {
                warnings.push(ValidationWarning::new(
                    self.id(),
                    format!("timestamp {ts} is {}s in the future", ts - clock),
                ));
            }
        }

        Ok(())
    }
}

/// Validates block height consistency with the chain tip (or genesis).
pub struct HeightConsistencyRule;

impl ValidationRule<Block> for HeightConsistencyRule {
    fn id(&self) -> RuleId {
        RuleId::new("block.height")
    }
    fn description(&self) -> &'static str {
        "block height is consistent with the chain tip or genesis"
    }
    fn validate(
        &self,
        block: &Block,
        ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        let cfg = ctx.config();
        let height = block.header.height.value();

        if ctx.is_genesis() {
            if height != cfg.genesis_height {
                return Err(ValidationError::InvalidHeight(format!(
                    "genesis height {height} != expected {}",
                    cfg.genesis_height
                )));
            }
            return Ok(());
        }

        if height == cfg.genesis_height {
            return Err(ValidationError::InvalidHeight(format!(
                "non-genesis block cannot occupy the genesis height {}",
                cfg.genesis_height
            )));
        }

        if let Some(prev) = ctx.prev_height() {
            if height != prev + 1 {
                return Err(ValidationError::InvalidHeight(format!(
                    "height {height} is not the expected {} (prev + 1)",
                    prev + 1
                )));
            }
        }

        Ok(())
    }
}

/// Validates the presence and correctness of the previous-block hash.
pub struct PreviousHashRule;

impl ValidationRule<Block> for PreviousHashRule {
    fn id(&self) -> RuleId {
        RuleId::new("block.previous_hash")
    }
    fn description(&self) -> &'static str {
        "previous block hash is present and matches the chain tip"
    }
    fn validate(
        &self,
        block: &Block,
        ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        let ph = block.header.previous_hash.inner().as_bytes();

        if ctx.is_genesis() {
            if !is_all_zero(ph) {
                return Err(ValidationError::MissingPreviousHash(
                    "genesis previous hash must be the zero hash".into(),
                ));
            }
            return Ok(());
        }

        if is_all_zero(ph) {
            return Err(ValidationError::MissingPreviousHash(
                "non-genesis block must set a non-zero previous hash".into(),
            ));
        }

        if let Some(prev) = ctx.prev_block_hash() {
            if ph != prev {
                return Err(ValidationError::InvalidHash(format!(
                    "previous hash {} does not match chain tip {}",
                    to_hex(ph),
                    to_hex(prev)
                )));
            }
        }

        Ok(())
    }
}

/// Enforces genesis-specific constraints when the context marks the block as genesis.
pub struct GenesisConstraintsRule;

impl ValidationRule<Block> for GenesisConstraintsRule {
    fn id(&self) -> RuleId {
        RuleId::new("block.genesis_constraints")
    }
    fn description(&self) -> &'static str {
        "genesis block satisfies all genesis-specific constraints"
    }
    fn validate(
        &self,
        block: &Block,
        ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        if !(ctx.is_genesis() && ctx.config().enforce_genesis_constraints) {
            return Ok(());
        }

        let cfg = ctx.config();
        let header = &block.header;

        if header.height.value() != cfg.genesis_height {
            return Err(ValidationError::GenesisConstraint(format!(
                "genesis height must be {}, got {}",
                cfg.genesis_height,
                header.height.value()
            )));
        }
        if !is_all_zero(header.previous_hash.inner().as_bytes()) {
            return Err(ValidationError::GenesisConstraint(
                "genesis previous hash must be the zero hash".into(),
            ));
        }
        if !block.transactions.is_empty() {
            return Err(ValidationError::GenesisConstraint(
                "genesis block must carry zero transactions".into(),
            ));
        }
        if header.merkle_root.as_bytes() != cfg.empty_merkle_root {
            return Err(ValidationError::GenesisConstraint(
                "genesis merkle root must equal the empty root".into(),
            ));
        }
        if header.version.value() != cfg.min_block_version {
            return Err(ValidationError::GenesisConstraint(format!(
                "genesis version must be {}, got {}",
                cfg.min_block_version,
                header.version.value()
            )));
        }

        Ok(())
    }
}

/// Validates block metadata consistency (transaction count).
pub struct BlockMetadataRule;

impl ValidationRule<Block> for BlockMetadataRule {
    fn id(&self) -> RuleId {
        RuleId::new("block.metadata")
    }
    fn description(&self) -> &'static str {
        "block metadata is consistent with the block contents"
    }
    fn validate(
        &self,
        block: &Block,
        ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        let cfg = ctx.config();
        let actual = block.transactions.len();

        if block.metadata.transaction_count != actual {
            return Err(ValidationError::InvalidBlock(format!(
                "metadata transaction_count {} != actual transaction count {}",
                block.metadata.transaction_count, actual
            )));
        }

        if actual > cfg.max_transaction_count {
            return Err(ValidationError::InvalidBlock(format!(
                "transaction count {actual} exceeds maximum {}",
                cfg.max_transaction_count
            )));
        }

        Ok(())
    }
}

/// The default, recommended rule set applied to blocks by the validation engine.
pub fn default_block_rules() -> Vec<Box<dyn ValidationRule<Block>>> {
    vec![
        Box::new(HeaderFormatRule),
        Box::new(BlockHashConsistencyRule),
        Box::new(MerkleRootRule),
        Box::new(TimestampValidityRule),
        Box::new(HeightConsistencyRule),
        Box::new(PreviousHashRule),
        Box::new(GenesisConstraintsRule),
        Box::new(BlockMetadataRule),
    ]
}
