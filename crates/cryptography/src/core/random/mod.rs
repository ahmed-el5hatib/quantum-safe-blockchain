//! Cryptographically secure random number generation.
//!
//! All randomness in QSB must come from OS-level entropy sources. This module provides
//! a [`RandomGenerator`] implementation backed by [`rand_core::OsRng`], which is the
//! standard secure randomness source on all supported platforms.

use rand_core::{OsRng, RngCore};

use crate::core::errors::CryptoResult;
use crate::core::traits::RandomGenerator;

/// A cryptographically secure random number generator backed by the operating system.
///
/// This is the default RNG for all QSB cryptographic operations. It uses `rand_core::OsRng`,
/// which reads from the OS entropy pool (`/dev/urandom` on Unix, `CryptGenRandom` on Windows,
/// etc.).
///
/// # Security
///
/// - Never use this for non-cryptographic purposes (e.g., Monte Carlo simulations).
/// - Never seed this with predictable values.
/// - Never use the same seed twice.
///
/// # Examples
///
/// ```
/// use cryptography::core::random::StdRngGenerator;
/// use cryptography::core::traits::RandomGenerator;
///
/// let rng = StdRngGenerator;
/// let mut bytes = [0u8; 32];
/// rng.fill(&mut bytes).unwrap();
/// assert_eq!(bytes.len(), 32);
/// ```
#[derive(Clone, Debug, Default)]
pub struct StdRngGenerator;

impl StdRngGenerator {
    /// Creates a new `StdRngGenerator`.
    pub fn new() -> Self {
        Self
    }
}

impl RandomGenerator for StdRngGenerator {
    fn fill(&self, dest: &mut [u8]) -> CryptoResult<()> {
        OsRng.fill_bytes(dest);
        Ok(())
    }

    fn try_fill(&self, dest: &mut [u8]) -> CryptoResult<()> {
        self.fill(dest)
    }

    fn algorithm_name(&self) -> &str {
        "OSRng"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::RandomGenerator;

    #[test]
    fn test_fill_produces_different_bytes() {
        let rng = StdRngGenerator;
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        rng.fill(&mut a).unwrap();
        rng.fill(&mut b).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn test_fill_correct_length() {
        let rng = StdRngGenerator;
        let mut bytes = [0u8; 64];
        rng.fill(&mut bytes).unwrap();
        assert_eq!(bytes.len(), 64);
    }

    #[test]
    fn test_try_fill_succeeds() {
        let rng = StdRngGenerator;
        let mut bytes = [0u8; 16];
        rng.try_fill(&mut bytes).unwrap();
        // OsRng should always succeed on supported platforms
        assert!(bytes.iter().any(|&b| b != 0));
    }
}
