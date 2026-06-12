//! Fuzz target: `parse_descriptor` (the toolkit's untrusted descriptor-string
//! intake). Toolkit phase of the constellation stress-fuzz program.
//!
//! Oracle (never-panic charter): `parse_descriptor` is reached only via the
//! `cfg(fuzzing)`-gated mount in `crates/mnemonic-toolkit/src/lib.rs` (cargo-fuzz
//! passes `--cfg fuzzing`). It takes an arbitrary descriptor string plus empty
//! key/fingerprint binding slices and MUST return `Err`, not panic, on
//! malformed input. Any panic/abort is a libFuzzer failure — a real
//! never-panic-charter finding.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Descriptor strings are ASCII; the U+FFFD collapse from lossy conversion
    // just wastes a sliver of input space.
    let s = String::from_utf8_lossy(data);
    // Empty key/fingerprint slices are valid (concrete descriptors need no
    // bindings; `@N`-template inputs simply fail to resolve → Err, not panic).
    let _ = mnemonic_toolkit::parse_descriptor::parse_descriptor(&s, &[], &[]);
});
