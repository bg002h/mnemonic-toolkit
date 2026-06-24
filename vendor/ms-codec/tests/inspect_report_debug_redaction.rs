//! cycle-15 Lane M (slug #1 marquee) — `InspectReport`'s `Debug` MUST NOT echo
//! the secret `payload_bytes`. RULE Z-DEBUG: `Zeroizing<Vec<u8>>`'s derived
//! `Debug` is non-redacting, so the hand-rolled redacting `Debug` is mandatory.
//!
//! `InspectReport` is `#[non_exhaustive]`, so it cannot be struct-literal'd from
//! an external test — build it via the public `inspect()` on a known ms1 whose
//! entropy is a fixed sentinel.

use ms_codec::{encode, inspect, Payload, Tag};
use zeroize::Zeroizing;

/// Lowercase-hex encode a byte slice (ms-codec has no `hex` dev-dep).
fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// A fixed sentinel entropy whose hex (`deadbeef…`) and Vec-Debug element form
/// (`222, 173, 190, 239`) are both easy to grep for in a Debug string.
fn sentinel_entropy() -> Vec<u8> {
    // 16 bytes (valid entr-16 length): DE AD BE EF repeated.
    let mut v = Vec::with_capacity(16);
    for _ in 0..4 {
        v.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
    }
    v
}

#[test]
fn inspect_report_debug_does_not_leak_entropy() {
    let entropy = sentinel_entropy();
    let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
    let report = inspect(&s).unwrap();

    let dbg = format!("{report:?}");

    // 1. Raw hex form must not appear.
    assert!(
        !dbg.to_lowercase().contains("deadbeef"),
        "Debug leaked the entropy hex: {dbg}"
    );
    // 2. Vec element-wise Debug leak guard (the `Zeroizing([222, 173, 190, 239, …])` form).
    assert!(
        !dbg.contains("222, 173, 190, 239"),
        "Debug leaked the entropy as a Vec element dump: {dbg}"
    );
    // 3. A redaction placeholder must be present.
    assert!(
        dbg.to_lowercase().contains("redacted"),
        "Debug must render payload_bytes as a redaction placeholder: {dbg}"
    );
    // 4. Non-secret structural fields stay visible.
    assert!(dbg.contains("hrp"), "Debug must still surface `hrp`: {dbg}");
    assert!(
        dbg.contains("kind"),
        "Debug must still surface `kind`: {dbg}"
    );
    assert!(
        dbg.contains("prefix_byte"),
        "Debug must still surface `prefix_byte`: {dbg}"
    );
    assert!(
        dbg.contains("language"),
        "Debug must still surface `language`: {dbg}"
    );
}

#[test]
fn inspect_report_payload_bytes_is_zeroizing() {
    // Type-level: `payload_bytes` is `Zeroizing<Vec<u8>>` (scrub-on-drop).
    let entropy = sentinel_entropy();
    let s = encode(Tag::ENTR, &Payload::Entr(entropy)).unwrap();
    let report = inspect(&s).unwrap();
    // Compiles only if the field is `Zeroizing<Vec<u8>>`.
    fn _assert_zeroizing(_: &Zeroizing<Vec<u8>>) {}
    _assert_zeroizing(&report.payload_bytes);
}

#[test]
fn inspect_report_payload_bytes_deref_readers_still_compile() {
    // Design A keeps the ms-cli read-only consumers green via `Deref`.
    let entropy = sentinel_entropy();
    let s = encode(Tag::ENTR, &Payload::Entr(entropy.clone())).unwrap();
    let report = inspect(&s).unwrap();
    // `.len()` (auto-deref) and `hex::encode(&field)` (Deref coercion) compile.
    let _n = report.payload_bytes.len();
    let hexed = to_hex(&report.payload_bytes);
    assert_eq!(hexed, to_hex(&entropy), "deref read recovers the bytes");
}
