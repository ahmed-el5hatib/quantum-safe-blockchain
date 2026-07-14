//! The [`Validator`] trait: the abstraction that higher-level components program against.
//!
//! A `Validator<T>` produces a [`ValidationReport`] for a target of type `T`. The engine's
//! pipelines implement this trait, so any pipeline (or a custom validator) can be substituted
//! without changing the caller.

use crate::context::ValidationContext;
use crate::report::ValidationReport;

/// A validator that evaluates a target of type `T` and returns a [`ValidationReport`].
///
/// Implemented by [`ValidationPipeline`](crate::ValidationPipeline); the
/// [`ValidationEngine`](crate::ValidationEngine) exposes its block and transaction pipelines through
/// this trait so consumers depend only on the abstraction.
pub trait Validator<T>: Send + Sync {
    /// Validates `target` under `ctx`, returning a full report (never panics, never returns early
    /// without a report).
    fn validate(&self, target: &T, ctx: &ValidationContext) -> ValidationReport;
}
