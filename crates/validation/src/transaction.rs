//! Transaction validation rules.
//!
//! Each rule implements [`ValidationRule<Transaction>`](crate::rule::ValidationRule) and checks one
//! concern from the milestone specification. Validation is **structural and semantic only** — it
//! never consults a UTXO set, a signature provider, or any chain state. "Inputs exist" and
//! "signature placeholder" are therefore *structural* checks (a reference/script is present and
//! well-formed), not full spend/verification checks, which belong to later milestones.
//!
//! The transaction type validated here is `blockchain_core::Transaction`, the on-chain transaction
//! carried inside a `Block`.

use std::collections::HashSet;

use blockchain_core::Transaction;

use crate::context::ValidationContext;
use crate::error::{ValidationError, ValidationResult};
use crate::hashes::compute_transaction_hash;
use crate::report::RuleId;
use crate::report::ValidationWarning;
use crate::rule::ValidationRule;

/// Validates the overall shape of the transaction (lock time, non-empty payload).
pub struct TransactionStructureRule;

impl ValidationRule<Transaction> for TransactionStructureRule {
    fn id(&self) -> RuleId {
        RuleId::new("transaction.structure")
    }
    fn description(&self) -> &'static str {
        "transaction shape is well-formed (lock time within bounds, non-empty payload)"
    }
    fn validate(
        &self,
        tx: &Transaction,
        ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        let cfg = ctx.config();
        if tx.lock_time > cfg.max_lock_time {
            return Err(ValidationError::InvalidTransaction(format!(
                "lock_time {} exceeds maximum {}",
                tx.lock_time, cfg.max_lock_time
            )));
        }
        if tx.inputs.is_empty() && tx.outputs.is_empty() {
            return Err(ValidationError::InvalidTransaction(
                "transaction has neither inputs nor outputs".into(),
            ));
        }
        Ok(())
    }
}

/// Validates the transaction version is a supported value.
pub struct TransactionVersionRule;

impl ValidationRule<Transaction> for TransactionVersionRule {
    fn id(&self) -> RuleId {
        RuleId::new("transaction.version")
    }
    fn description(&self) -> &'static str {
        "transaction version is within the supported range"
    }
    fn validate(
        &self,
        tx: &Transaction,
        ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        let cfg = ctx.config();
        let v = tx.version;
        if v < cfg.min_transaction_version {
            return Err(ValidationError::InvalidTransaction(format!(
                "transaction version {v} is below minimum {}",
                cfg.min_transaction_version
            )));
        }
        if v > cfg.max_transaction_version {
            return Err(ValidationError::InvalidTransaction(format!(
                "transaction version {v} exceeds maximum {}",
                cfg.max_transaction_version
            )));
        }
        Ok(())
    }
}

/// Structural check that every input references a previous output (placeholder only).
pub struct InputsExistRule;

impl ValidationRule<Transaction> for InputsExistRule {
    fn id(&self) -> RuleId {
        RuleId::new("transaction.inputs_exist")
    }
    fn description(&self) -> &'static str {
        "every input references a previous output (structural)"
    }
    fn validate(
        &self,
        tx: &Transaction,
        _ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        for (i, input) in tx.inputs.iter().enumerate() {
            if input.previous_output.transaction_hash.as_bytes().is_empty() {
                return Err(ValidationError::InvalidTransaction(format!(
                    "input {i} references an empty previous-output hash (structural)"
                )));
            }
        }
        Ok(())
    }
}

/// Structural check that the transaction carries at least one output.
pub struct OutputsExistRule;

impl ValidationRule<Transaction> for OutputsExistRule {
    fn id(&self) -> RuleId {
        RuleId::new("transaction.outputs_exist")
    }
    fn description(&self) -> &'static str {
        "transaction carries at least one output"
    }
    fn validate(
        &self,
        tx: &Transaction,
        _ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        if tx.outputs.is_empty() {
            return Err(ValidationError::InvalidTransaction(
                "transaction has no outputs".into(),
            ));
        }
        Ok(())
    }
}

/// Detects double spends inside the transaction (duplicate inputs).
pub struct DuplicateInputsRule;

impl ValidationRule<Transaction> for DuplicateInputsRule {
    fn id(&self) -> RuleId {
        RuleId::new("transaction.duplicate_inputs")
    }
    fn description(&self) -> &'static str {
        "no two inputs spend the same previous output"
    }
    fn validate(
        &self,
        tx: &Transaction,
        _ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        let mut seen: HashSet<(Vec<u8>, u32)> = HashSet::new();
        for (i, input) in tx.inputs.iter().enumerate() {
            let key = (
                input.previous_output.transaction_hash.as_bytes().to_vec(),
                input.previous_output.output_index,
            );
            if !seen.insert(key) {
                return Err(ValidationError::DuplicateInput(format!(
                    "input {i} duplicates an earlier spend of the same outpoint"
                )));
            }
        }
        Ok(())
    }
}

/// Detects duplicate outputs (identical value + script pairs).
pub struct DuplicateOutputsRule;

impl ValidationRule<Transaction> for DuplicateOutputsRule {
    fn id(&self) -> RuleId {
        RuleId::new("transaction.duplicate_outputs")
    }
    fn description(&self) -> &'static str {
        "no two outputs are identical (value + script)"
    }
    fn validate(
        &self,
        tx: &Transaction,
        _ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        let mut seen: HashSet<(u64, Vec<u8>)> = HashSet::new();
        for (i, out) in tx.outputs.iter().enumerate() {
            let key = (out.value, out.script_pubkey.clone());
            if !seen.insert(key) {
                return Err(ValidationError::DuplicateOutput(format!(
                    "output {i} duplicates an earlier output (value + script)"
                )));
            }
        }
        Ok(())
    }
}

/// Validates output amounts: positive and within the total-value ceiling.
pub struct AmountRule;

impl ValidationRule<Transaction> for AmountRule {
    fn id(&self) -> RuleId {
        RuleId::new("transaction.amount")
    }
    fn description(&self) -> &'static str {
        "output amounts are positive and the total does not overflow"
    }
    fn validate(
        &self,
        tx: &Transaction,
        ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        let mut total: u128 = 0;
        for (i, out) in tx.outputs.iter().enumerate() {
            if out.value == 0 {
                return Err(ValidationError::InvalidAmount(format!(
                    "output {i} has zero value"
                )));
            }
            total = total.checked_add(u128::from(out.value)).ok_or_else(|| {
                ValidationError::InvalidAmount("output value overflows total".into())
            })?;
        }
        if total > ctx.config().max_output_value {
            return Err(ValidationError::InvalidAmount(format!(
                "total output value {total} exceeds maximum {}",
                ctx.config().max_output_value
            )));
        }
        Ok(())
    }
}

/// Verifies the recomputed transaction hash has the expected digest length.
pub struct HashIntegrityRule;

impl ValidationRule<Transaction> for HashIntegrityRule {
    fn id(&self) -> RuleId {
        RuleId::new("transaction.hash_integrity")
    }
    fn description(&self) -> &'static str {
        "canonical transaction hash is non-empty and correctly sized"
    }
    fn validate(
        &self,
        tx: &Transaction,
        ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        let computed = compute_transaction_hash(tx, ctx.hasher().as_ref())?;
        if computed.is_empty() {
            return Err(ValidationError::InvalidHash(
                "computed transaction hash is empty".into(),
            ));
        }
        let hash_len = ctx.digest_size();
        if computed.len() != hash_len {
            return Err(ValidationError::InvalidHash(format!(
                "computed transaction hash length {} != expected {hash_len}",
                computed.len()
            )));
        }
        Ok(())
    }
}

/// Structural validation of the signature placeholder on each input.
pub struct SignaturePlaceholderRule;

impl ValidationRule<Transaction> for SignaturePlaceholderRule {
    fn id(&self) -> RuleId {
        RuleId::new("transaction.signature_placeholder")
    }
    fn description(&self) -> &'static str {
        "signature placeholders are present and within size bounds"
    }
    fn validate(
        &self,
        tx: &Transaction,
        ctx: &ValidationContext,
        _warnings: &mut Vec<ValidationWarning>,
    ) -> ValidationResult<()> {
        let cfg = ctx.config();
        for (i, input) in tx.inputs.iter().enumerate() {
            let len = input.signature.len();
            if len == 0 {
                return Err(ValidationError::InvalidTransaction(format!(
                    "input {i} has an empty signature placeholder"
                )));
            }
            if len > cfg.max_signature_len {
                return Err(ValidationError::InvalidTransaction(format!(
                    "input {i} signature length {len} exceeds maximum {}",
                    cfg.max_signature_len
                )));
            }
        }
        Ok(())
    }
}

/// The default, recommended rule set applied to transactions by the validation engine.
pub fn default_transaction_rules() -> Vec<Box<dyn ValidationRule<Transaction>>> {
    vec![
        Box::new(TransactionStructureRule),
        Box::new(TransactionVersionRule),
        Box::new(InputsExistRule),
        Box::new(OutputsExistRule),
        Box::new(DuplicateInputsRule),
        Box::new(DuplicateOutputsRule),
        Box::new(AmountRule),
        Box::new(HashIntegrityRule),
        Box::new(SignaturePlaceholderRule),
    ]
}
