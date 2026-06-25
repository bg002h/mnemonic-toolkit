//! Frozen stripe-padding rule (plan §4.1 / §4.6, fold M4).
//!
//! RAID striping (P5) requires every xpub payload in the array to be the same
//! byte length before the 8→11 regroup. The frozen rule is: **zero-pad each
//! payload on the right** to the array-wide maximum byte length. We freeze and
//! test that primitive here at P1 so the P5 RAID layer is self-contained.

/// Zero-pad `bytes` on the right (append zero bytes) so the result is exactly
/// `target_len` bytes long.
///
/// - If `bytes.len() == target_len`, the bytes are returned unchanged (no extra
///   allocation beyond the copy).
/// - If `bytes.len() < target_len`, `target_len - bytes.len()` zero bytes are
///   appended.
///
/// # Panics
/// Panics if `target_len < bytes.len()` — padding never truncates; a caller
/// asking to pad below the input length is a programming error (the array-wide
/// max is, by definition, ≥ every member length).
pub fn pad_payload_to(bytes: &[u8], target_len: usize) -> Vec<u8> {
    assert!(
        target_len >= bytes.len(),
        "pad_payload_to: target_len ({target_len}) < input length ({})",
        bytes.len()
    );
    let mut out = Vec::with_capacity(target_len);
    out.extend_from_slice(bytes);
    out.resize(target_len, 0);
    out
}
