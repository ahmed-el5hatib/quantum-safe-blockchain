//! Signature algorithm interfaces and implementations.
//!
//! This module re-exports the [`SignatureAlgorithm`], [`PublicKey`], [`PrivateKey`],
//! and [`Signature`] traits, and provides provider implementations.

pub use crate::core::traits::{PrivateKey, PublicKey, Signature, SignatureAlgorithm};
