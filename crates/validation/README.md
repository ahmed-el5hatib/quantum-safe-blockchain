# QSB Validation Engine

Reusable, deterministic, trait-based validation framework for the Quantum Safe Blockchain (QSB)
ecosystem — **Milestone 4.4**.

Validates **blocks** and **transactions** independently of networking, storage, consensus, and
execution, so every future QSB blockchain implementation reuses it unchanged.

## Highlights

- **Rule → Pipeline → Engine** architecture: atomic `ValidationRule`s composed into a
  `ValidationPipeline`, orchestrated by `ValidationEngine`.
- **Stateless & panic-free**: every validator returns `Result<T, ValidationError>`.
- **Crypto-agile**: depends only on the abstract `HashFunction` trait; the concrete algorithm is
  injected through `ValidationContext`.
- **Configurable rules**: enable/disable any rule via `ValidationConfig`.
- **Rich reporting**: `ValidationReport` with status, executed rules + timings, errors, and warnings.
- **Benchmarked**: `cargo bench -p validation`.

## Layout

| Module | Purpose |
|--------|---------|
| `error` | `ValidationError` variants and `ValidationResult`. |
| `context` | `ValidationContext` (hasher + chain facts) and `ValidationConfig`. |
| `report` | `ValidationReport`, `RuleResult`, `ValidationWarning`, `RuleId`. |
| `rule` | `ValidationRule<T>` trait. |
| `pipeline` | `ValidationPipeline<T>` (ordered, configurable rule execution). |
| `hashes` | Canonical commitments: `compute_block_hash`, `compute_transaction_hash`, `compute_merkle_root`. |
| `block` / `transaction` | Default rule sets for blocks and transactions. |
| `engine` | `ValidationEngine` + `EngineBuilder`. |

See [`docs/architecture/validation-engine.md`](../../docs/architecture/validation-engine.md) for the
full architecture, lifecycle, extensibility, and performance analysis.
