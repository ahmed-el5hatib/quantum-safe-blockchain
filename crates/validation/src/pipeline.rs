//! Validation pipeline.
//!
//! A [`ValidationPipeline`] is an ordered, configurable collection of [`ValidationRule`]s for a
//! single target type `T`. Running the pipeline executes every *enabled* rule against the target,
//! measures each rule and the aggregate, and produces a [`ValidationReport`].
//!
//! Rules are enabled by default unless their `enabled_by_default` returns `false`, or a context
//! supplies an explicit override via [`ValidationConfig`](crate::ValidationConfig). This satisfies
//! the milestone requirement that rules be enable/disable-able through configuration.

use std::time::Instant;

use crate::context::ValidationContext;
use crate::report::{RuleResult, ValidationReport, ValidationStatus, ValidationWarning};
use crate::rule::ValidationRule;

/// An ordered collection of validation rules for a target type `T`, executed as a unit.
pub struct ValidationPipeline<T> {
    rules: Vec<Box<dyn ValidationRule<T>>>,
}

impl<T> Default for ValidationPipeline<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ValidationPipeline<T> {
    /// Creates an empty pipeline.
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Creates a pipeline pre-populated with `rules`.
    pub fn with_rules(rules: Vec<Box<dyn ValidationRule<T>>>) -> Self {
        Self { rules }
    }

    /// Appends a rule, consuming and returning `self` for fluent construction.
    pub fn register(mut self, rule: Box<dyn ValidationRule<T>>) -> Self {
        self.rules.push(rule);
        self
    }

    /// Returns the registered rules.
    pub fn rules(&self) -> &[Box<dyn ValidationRule<T>>] {
        &self.rules
    }

    /// Consumes the pipeline and returns its rules.
    pub fn into_rules(self) -> Vec<Box<dyn ValidationRule<T>>> {
        self.rules
    }

    /// Returns the ids of the rules that *would* execute under `ctx` (enabled rules).
    pub fn enabled_rule_ids(&self, ctx: &ValidationContext) -> Vec<crate::report::RuleId> {
        self.rules
            .iter()
            .filter(|r| self.is_enabled(r.as_ref(), ctx))
            .map(|r| r.id())
            .collect()
    }

    /// Determines whether a rule should execute under `ctx`, honoring explicit overrides.
    fn is_enabled(&self, rule: &dyn ValidationRule<T>, ctx: &ValidationContext) -> bool {
        match ctx.config().rule_enabled_override(rule.id().as_str()) {
            Some(enabled) => enabled,
            None => rule.enabled_by_default(),
        }
    }

    /// Executes every enabled rule against `target` and returns a [`ValidationReport`].
    ///
    /// Disabled rules are skipped and do not appear in the report. If the context's `fail_fast`
    /// flag is set, execution stops after the first failing rule (which is still recorded).
    pub fn run(&self, target: &T, ctx: &ValidationContext) -> ValidationReport {
        let mut report = ValidationReport::new(std::any::type_name::<T>(), None);
        let start = Instant::now();
        let fail_fast = ctx.config().fail_fast;

        for rule in &self.rules {
            if !self.is_enabled(rule.as_ref(), ctx) {
                continue;
            }

            let mut warnings: Vec<ValidationWarning> = Vec::new();
            let rule_start = Instant::now();
            let outcome = rule.validate(target, ctx, &mut warnings);
            let duration = rule_start.elapsed();

            let id = rule.id();
            let description = rule.description();

            match outcome {
                Ok(()) => report.record(RuleResult::passed(id, description, duration)),
                Err(err) => {
                    report.record(RuleResult::failed(id, description, duration, err));
                    if fail_fast {
                        break;
                    }
                }
            }

            for warning in warnings {
                report.warn(warning);
            }
        }

        report.execution_time = start.elapsed();
        if report.status == ValidationStatus::Passed && report.rule_results.is_empty() {
            // No rules executed (all disabled) — treat as passed but surface awareness via warning.
            report.warn(ValidationWarning::unattributed(
                "no validation rules were enabled for this target",
            ));
        }
        report
    }
}
