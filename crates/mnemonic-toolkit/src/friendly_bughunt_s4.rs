//! Bug-hunt triage RED row for tk bug #36 (slice S4). Tests only.
//!
//! `friendly` is binary-private (`lib.rs` mounts it under `cfg(fuzzing)` only),
//! so this row lives in a `#[cfg(test)]` module of the `mnemonic` bin target.

use super::friendly_ms_codec;

/// Bug 36: `ms_codec::Error::UnexpectedStringLength` carries the set the string
/// was checked against in `allowed`. `ms_codec::decode` reports the MNEM set
/// (`VALID_MNEM_STR_LENGTHS` = [51, 58, 64, 70, 77]) when a mnem-kind string has
/// an entr-only length. `friendly_ms_codec` ignores `allowed` and always prints
/// the entr set [50, 56, 62, 69, 75], so a mnem string of length 50 is reported
/// as "length 50 not in [50, ...]" -- self-contradictory.
#[test]
#[ignore = "deferred-minor: friendly-ms1-length-set-uses-allowed"]
fn bughunt_tk_36_friendly_length_set_follows_error_allowed() {
    let err = ms_codec::Error::UnexpectedStringLength {
        got: 50,
        allowed: ms_codec::consts::VALID_MNEM_STR_LENGTHS,
    };
    let msg = friendly_ms_codec(&err);
    assert!(
        msg.contains("[51, 58, 64, 70, 77]"),
        "friendly message must print the set the string was checked against \
         (the error's `allowed` field = the mnem set); got: {msg}"
    );
    assert!(
        !msg.contains("length 50 not in v0.1 set [50,"),
        "friendly message contradicts itself (length 50 'not in' a set that contains 50): {msg}"
    );
}
