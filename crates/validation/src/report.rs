//! Validation reporting.
//!
//! A [`ValidationReport`] is the rich outcome of running a [`ValidationPipeline`](crate::ValidationPipeline).
//! Unlike a bare `Result`, it records *every* executed rule, its individual timing, the aggregate
//! execution time, and any accumulated warnings. This makes the report suitable for observability,
//! RPC diagnostics, and debugging without re-running validation.

use std::fmt;
use std::time::Duration;

use crate::error::{ValidationError, ValidationResult};

/// A stable, copyable identifier for a validation rule.
///
/// Rule ids are static strings (e.g. `"block.merkle_root"`). They double as the keys used to
/// enable/disable rules via [`ValidationConfig`](crate::ValidationConfig).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RuleId(&'static str);

impl RuleId {
    /// Creates a rule id from a static string.
    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }

    /// Returns the id as a string slice.
    pub fn as_str(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for RuleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// A non-fatal observation emitted by a rule during validation.
///
/// Warnings do not cause a validation to fail; they surface soft concerns (e.g. a timestamp that is
/// valid but suspiciously far in the future) so operators can investigate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationWarning {
    /// The rule that produced the warning, if attributable.
    pub rule_id: Option<RuleId>,
    /// The warning message.
    pub message: String,
}

impl ValidationWarning {
    /// Creates a warning attributable to a specific rule.
    pub fn new(rule_id: RuleId, message: impl Into<String>) -> Self {
        Self {
            rule_id: Some(rule_id),
            message: message.into(),
        }
    }

    /// Creates an unattributed warning.
    pub fn unattributed(message: impl Into<String>) -> Self {
        Self {
            rule_id: None,
            message: message.into(),
        }
    }
}

/// The overall outcome of a validation run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationStatus {
    /// All executed rules passed.
    Passed,
    /// At least one executed rule failed.
    Failed,
}

/// The per-rule outcome recorded in a [`ValidationReport`].
#[derive(Clone, Debug)]
pub struct RuleResult {
    /// The rule id.
    pub id: RuleId,
    /// A human-readable description of the rule.
    pub description: &'static str,
    /// Whether the rule passed.
    pub passed: bool,
    /// Time spent evaluating the rule.
    pub duration: Duration,
    /// The error, if the rule failed.
    pub error: Option<ValidationError>,
}

impl RuleResult {
    /// Constructs a passing rule result.
    pub fn passed(id: RuleId, description: &'static str, duration: Duration) -> Self {
        Self {
            id,
            description,
            passed: true,
            duration,
            error: None,
        }
    }

    /// Constructs a failing rule result.
    pub fn failed(
        id: RuleId,
        description: &'static str,
        duration: Duration,
        error: ValidationError,
    ) -> Self {
        Self {
            id,
            description,
            passed: false,
            duration,
            error: Some(error),
        }
    }
}

/// The complete, structured result of validating a single target (block or transaction).
///
/// The report captures success/failure, every executed rule (with timing), the aggregate execution
/// time, error details, and any warnings. It is the primary artifact produced by the
/// [`ValidationEngine`](crate::ValidationEngine).
#[derive(Clone, Debug)]
pub struct ValidationReport {
    /// The kind of target validated (`"block"` or `"transaction"`).
    pub kind: &'static str,
    /// An optional stable identifier for the target (e.g. block hash or transaction hash).
    pub target_id: Option<String>,
    /// Overall pass/fail status.
    pub status: ValidationStatus,
    /// Total wall-clock time spent validating the target.
    pub execution_time: Duration,
    /// One entry per executed rule, in execution order.
    pub rule_results: Vec<RuleResult>,
    /// Every error encountered across all executed rules.
    pub errors: Vec<ValidationError>,
    /// Every warning emitted by executed rules.
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationReport {
    /// Creates an empty report for a target of the given `kind`.
    pub(crate) fn new(kind: &'static str, target_id: Option<String>) -> Self {
        Self {
            kind,
            target_id,
            status: ValidationStatus::Passed,
            execution_time: Duration::ZERO,
            rule_results: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Returns `true` if validation passed (no rule failures).
    pub fn is_valid(&self) -> bool {
        matches!(self.status, ValidationStatus::Passed)
    }

    /// Number of rules executed.
    pub fn executed_rules(&self) -> usize {
        self.rule_results.len()
    }

    /// Number of rules that passed.
    pub fn passed_rules(&self) -> usize {
        self.rule_results.iter().filter(|r| r.passed).count()
    }

    /// Number of rules that failed.
    pub fn failed_rules(&self) -> usize {
        self.rule_results.iter().filter(|r| !r.passed).count()
    }

    /// Adds a rule result to the report, updating aggregate status and errors.
    pub(crate) fn record(&mut self, result: RuleResult) {
        if !result.passed {
            self.status = ValidationStatus::Failed;
            if let Some(err) = &result.error {
                self.errors.push(err.clone());
            }
        }
        self.rule_results.push(result);
    }

    /// Adds a warning to the report.
    pub(crate) fn warn(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    /// Converts the report into a [`ValidationResult`]: `Ok(())` when valid, otherwise the first
    /// error encountered.
    pub fn into_result(&self) -> ValidationResult<()> {
        if self.is_valid() {
            Ok(())
        } else {
            match self.errors.first() {
                Some(err) => Err(err.clone()),
                None => Err(crate::error::ValidationError::ValidationRuleFailed(
                    "validation reported failure without a captured error".into(),
                )),
            }
        }
    }
}
