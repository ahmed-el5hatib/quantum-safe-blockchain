//! The Validation Engine: the reusable orchestrator for block and transaction validation.
//!
//! [`ValidationEngine`] is the single entry point of this crate. It owns two
//! [`ValidationPipeline`]s — one for blocks, one for transactions — assembled from the default rule
//! sets, and exposes ergonomic `validate_*` / `check_*` methods. The engine itself is stateless with
//! respect to any particular chain; all chain-specific facts (clock, previous hash/height, genesis
//! flag) arrive through the [`ValidationContext`] passed to each call.
//!
//! The engine is deliberately independent of networking, storage, consensus, and execution, and
//! depends only on the abstract [`HashFunction`](cryptography::core::traits::HashFunction) trait —
//! making it reusable by every future blockchain implementation built on QSB.

use blockchain_core::{Block, Transaction};

use crate::context::ValidationContext;
use crate::error::ValidationResult;
use crate::pipeline::ValidationPipeline;
use crate::report::ValidationReport;
use crate::rule::ValidationRule;
use crate::validator::Validator;

/// The reusable, deterministic validation engine for QSB.
///
/// Construct via [`ValidationEngine::with_defaults`] for the standard rule sets, or
/// [`ValidationEngine::with_pipelines`] to supply custom ones. Validate individual blocks and
/// transactions with [`validate_block`](Self::validate_block) /
/// [`validate_transaction`](Self::validate_transaction), or use the `check_*` variants which return
/// a `Result` suitable for `?`-propagation.
pub struct ValidationEngine {
    block_pipeline: ValidationPipeline<Block>,
    transaction_pipeline: ValidationPipeline<Transaction>,
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl ValidationEngine {
    /// Creates an engine with the default block and transaction rule sets.
    pub fn with_defaults() -> Self {
        Self {
            block_pipeline: ValidationPipeline::with_rules(crate::block::default_block_rules()),
            transaction_pipeline: ValidationPipeline::with_rules(
                crate::transaction::default_transaction_rules(),
            ),
        }
    }

    /// Creates an engine from explicitly provided pipelines.
    pub fn with_pipelines(
        block_pipeline: ValidationPipeline<Block>,
        transaction_pipeline: ValidationPipeline<Transaction>,
    ) -> Self {
        Self {
            block_pipeline,
            transaction_pipeline,
        }
    }

    /// Starts a fluent builder for assembling a custom engine.
    pub fn builder() -> EngineBuilder {
        EngineBuilder::default()
    }

    /// Validates a block, returning a full [`ValidationReport`].
    pub fn validate_block(&self, block: &Block, ctx: &ValidationContext) -> ValidationReport {
        let mut report = self.block_pipeline.run(block, ctx);
        report.kind = "block";
        report.target_id = Some(block.block_hash.to_string());
        report
    }

    /// Validates a transaction, returning a full [`ValidationReport`].
    pub fn validate_transaction(
        &self,
        tx: &Transaction,
        ctx: &ValidationContext,
    ) -> ValidationReport {
        let mut report = self.transaction_pipeline.run(tx, ctx);
        report.kind = "transaction";
        report.target_id = Some(format!(
            "tx(versions={},inputs={},outputs={})",
            tx.version,
            tx.inputs.len(),
            tx.outputs.len()
        ));
        report
    }

    /// Validates a block, returning `Ok(())` on success or the first error otherwise.
    pub fn check_block(&self, block: &Block, ctx: &ValidationContext) -> ValidationResult<()> {
        self.validate_block(block, ctx).into_result()
    }

    /// Validates a transaction, returning `Ok(())` on success or the first error otherwise.
    pub fn check_transaction(
        &self,
        tx: &Transaction,
        ctx: &ValidationContext,
    ) -> ValidationResult<()> {
        self.validate_transaction(tx, ctx).into_result()
    }

    /// Returns the block validation pipeline.
    pub fn block_pipeline(&self) -> &ValidationPipeline<Block> {
        &self.block_pipeline
    }

    /// Returns the transaction validation pipeline.
    pub fn transaction_pipeline(&self) -> &ValidationPipeline<Transaction> {
        &self.transaction_pipeline
    }
}

impl Validator<Block> for ValidationEngine {
    fn validate(&self, target: &Block, ctx: &ValidationContext) -> ValidationReport {
        self.validate_block(target, ctx)
    }
}

impl Validator<Transaction> for ValidationEngine {
    fn validate(&self, target: &Transaction, ctx: &ValidationContext) -> ValidationReport {
        self.validate_transaction(target, ctx)
    }
}

/// Fluent builder for a [`ValidationEngine`] with custom rule sets.
///
/// # Examples
///
/// ```
/// use validation::{ValidationEngine, ValidationPipeline};
/// use blockchain_core::Block;
///
/// let engine = ValidationEngine::builder()
///     .block_pipeline(ValidationPipeline::with_rules(validation::block::default_block_rules()))
///     .build();
/// assert_eq!(engine.block_pipeline().rules().len(), 8);
/// ```
#[derive(Default)]
pub struct EngineBuilder {
    block_rules: Option<Vec<Box<dyn ValidationRule<Block>>>>,
    transaction_rules: Option<Vec<Box<dyn ValidationRule<Transaction>>>>,
}

impl EngineBuilder {
    /// Supplies a custom block rule set.
    pub fn block_rules(mut self, rules: Vec<Box<dyn ValidationRule<Block>>>) -> Self {
        self.block_rules = Some(rules);
        self
    }

    /// Supplies a custom transaction rule set.
    pub fn transaction_rules(mut self, rules: Vec<Box<dyn ValidationRule<Transaction>>>) -> Self {
        self.transaction_rules = Some(rules);
        self
    }

    /// Supplies a custom block pipeline.
    pub fn block_pipeline(mut self, pipeline: ValidationPipeline<Block>) -> Self {
        self.block_rules = Some(pipeline.into_rules());
        self
    }

    /// Supplies a custom transaction pipeline.
    pub fn transaction_pipeline(mut self, pipeline: ValidationPipeline<Transaction>) -> Self {
        self.transaction_rules = Some(pipeline.into_rules());
        self
    }

    /// Finalizes the builder into a [`ValidationEngine`].
    pub fn build(self) -> ValidationEngine {
        let block_pipeline = match self.block_rules {
            Some(rules) => ValidationPipeline::with_rules(rules),
            None => ValidationPipeline::with_rules(crate::block::default_block_rules()),
        };
        let transaction_pipeline = match self.transaction_rules {
            Some(rules) => ValidationPipeline::with_rules(rules),
            None => ValidationPipeline::with_rules(crate::transaction::default_transaction_rules()),
        };
        ValidationEngine {
            block_pipeline,
            transaction_pipeline,
        }
    }
}
