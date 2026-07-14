//! Validation rule abstraction.
//!
//! A [`ValidationRule`] is the atomic, independently-testable unit of the engine. Rules are
//! stateless: all input arrives through the target value and the [`ValidationContext`], and all
//! output is communicated through the returned [`ValidationResult`] plus an optional warnings sink.
//! This makes each rule trivially unit-testable in isolation.

use crate::context::ValidationContext;
use crate::error::ValidationResult;
use crate::report::{RuleId, ValidationWarning};

/// A single, stateless validation check over a target of type `T`.
///
/// Implementors must be `Send + Sync` and must **never panic** — every code path returns a
/// [`ValidationResult`]. Rules are composable: a [`ValidationPipeline`](crate::ValidationPipeline)
/// runs an ordered collection of them and aggregates their outcomes into a
/// [`ValidationReport`](crate::ValidationReport).
pub trait ValidationRule<T>: Send + Sync {
    /// The stable identifier of the rule (used for reporting and enable/disable configuration).
    fn id(&self) -> RuleId;

    /// A human-readable description of what the rule checks.
    fn description(&self) -> &'static str;

    /// Whether the rule runs by default. Individual contexts can override this via configuration.
    fn enabled_by_default(&self) -> bool {
        true
    }

    /// Evaluates the rule against `target`.
    ///
    /// Returns `Ok(())` when the rule passes. A failure returns `Err(ValidationError)` describing the
    /// specific violation. Non-fatal observations may be pushed onto `warnings`.
    fn validate(
        &self,
        target: &T,
        ctx: &ValidationContext,
        warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()>;
}
