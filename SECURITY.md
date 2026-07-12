# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.x     | Yes                |

## Reporting a Vulnerability

The QSB team takes security seriously. We appreciate your efforts to responsibly disclose any vulnerabilities.

### Reporting Process

1. **Do NOT** open a public GitHub issue for security vulnerabilities.
2. Email security@quantum-safe-blockchain.org with:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)
3. We will acknowledge receipt within 48 hours.
4. We will provide a detailed response within 7 days.
5. We will keep you informed of progress toward a fix.
6. We will credit you in the security advisory (unless you prefer anonymity).

### Security Considerations

QSB is designed with security as the highest priority:

- **Cryptography**: All cryptographic operations use constant-time implementations where possible.
- **Keys**: Private keys are never exposed outside secure enclaves and are zeroized when no longer needed.
- **Inputs**: Every external input is validated before processing.
- **Secrets**: No secrets are logged, and all secrets are stored encrypted at rest.
- **Unsafe Code**: All unsafe blocks are documented and reviewed. We aim for zero unsafe code.
- **Dependencies**: We audit dependencies with `cargo audit` and enable Dependabot.

### Best Practices for Developers

- Never commit secrets, keys, or passwords
- Use `zeroize` for sensitive data
- Prefer `subtle` or custom constant-time code for crypto
- Validate all inputs at system boundaries
- Use `#[must_use]` for results of security-critical functions
- Follow the principle of least privilege

### Disclosures

- Security vulnerabilities will be disclosed publicly after a fix is released.
- We follow coordinated disclosure principles.
- CVEs will be requested for significant vulnerabilities.

## Security Updates

Subscribe to GitHub Security Advisories for this repository to receive notifications about security updates.
