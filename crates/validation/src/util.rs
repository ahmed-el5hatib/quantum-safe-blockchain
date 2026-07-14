//! Small internal formatting helpers shared by validation rules.

/// Encodes bytes as a lowercase hex string for error messages.
pub(crate) fn to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Returns `true` if every byte is zero.
pub(crate) fn is_all_zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|b| *b == 0)
}
