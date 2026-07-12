# Contributing to QSB

Thank you for your interest in contributing to Quantum Safe Blockchain (QSB)!

## Code of Conduct

This project is governed by our [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How to Contribute

### Reporting Bugs

- Use the GitHub issue tracker
- Describe the expected behavior and actual behavior
- Include reproduction steps and environment details
- Specify Rust version and platform

### Suggesting Features

- Open a GitHub issue with the `enhancement` label
- Describe the use case and motivation
- Provide API design suggestions if applicable

### Development Setup

```bash
# Install Rust stable (1.75+)
rustup install stable
rustup default stable

# Install additional tools
rustup component add rustfmt clippy
cargo install cargo-audit cargo-nextest

# Clone and build
git clone https://github.com/quantum-safe-blockchain/quantum-safe-blockchain.git
cd quantum-safe-blockchain
cargo build --workspace
cargo test --workspace
```

### Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy --workspace --all-targets -- -D warnings`
- Write documentation for every public item
- Add unit tests for every module
- Follow SOLID principles and layered architecture
- Never couple implementations to concrete types
- Always program to traits

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add ML-DSA signature support
fix: correct mempool eviction logic
docs: update architecture diagram
refactor: extract storage trait from implementation
test: add property tests for transaction validation
perf: optimize merkle tree hashing
ci: add audit job to workflow
```

### Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Ensure all tests pass (`cargo test --workspace`)
5. Ensure code is formatted and linted
6. Push to your fork
7. Open a Pull Request against `develop`

### Review Process

- All PRs require at least one approval
- CI must pass (fmt, clippy, test, audit, doc, build)
- Security-sensitive changes require security team review
- Documentation updates are required for API changes

## Project Architecture

We use:
- **Layered Architecture**: Presentation, Application, Domain, Infrastructure
- **Hexagonal Architecture**: Core business logic isolated from external concerns
- **Dependency Injection**: Traits and constructors for testability
- **Trait-based Interfaces**: Every implementation must be replaceable

### Design Principles

- SOLID
- DRY
- KISS
- YAGNI
- Composition over inheritance
- Dependency inversion
- Explicit interfaces
- Zero global mutable state

## Security

Security has the highest priority. Before contributing, review [SECURITY.md](SECURITY.md).

- Never expose private keys
- Never log secrets
- Zero hardcoded credentials
- Validate every input
- Prefer constant-time implementations
- Document every unsafe block

## Questions?

Join our discussions on GitHub Discussions or open an issue.
