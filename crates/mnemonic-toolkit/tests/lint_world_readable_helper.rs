//! v0.13.0 P2.2 — `secret_advisory::warn_if_world_readable` shared
//! helper lint pin (R0 Q5 fold).
//!
//! At P2.2 GREEN, `emit_world_readable_advisory` is extracted from its
//! private home at `src/cmd/seed_xor.rs:425-445` into
//! `src/secret_advisory.rs::warn_if_world_readable`, and 3 call sites
//! are updated in lockstep:
//!   - `src/cmd/seed_xor.rs` (was inline; now calls helper)
//!   - `src/cmd/final_word.rs` (was inline; now calls helper)
//!   - `src/cmd/slip39.rs` (NEW; first-time call site)
//!
//! This lint pins:
//!   1. `secret_advisory.rs` exposes `warn_if_world_readable` (post-extraction).
//!   2. All 3 call sites reference it (`warn_if_world_readable`
//!      appears in each cmd/*.rs).
//!   3. `cmd/seed_xor.rs` no longer defines its own
//!      `emit_world_readable_advisory` (the extracted-out signal —
//!      partial migration would leave both helpers existing, which
//!      this lint catches).
//!
//! All assertions FAIL at RED — the extraction happens at GREEN.

use std::fs;
use std::path::Path;

fn read_or_panic(p: &str) -> String {
    fs::read_to_string(Path::new(p))
        .unwrap_or_else(|e| panic!("failed to read {p}: {e}"))
}

#[test]
fn secret_advisory_module_exports_warn_if_world_readable() {
    let src = read_or_panic("src/secret_advisory.rs");
    assert!(
        src.contains("pub fn warn_if_world_readable"),
        "secret_advisory.rs must export pub fn warn_if_world_readable post-extraction \
         (R0 Q5 fold); got source not containing 'pub fn warn_if_world_readable'"
    );
}

#[test]
fn cmd_seed_xor_calls_shared_warn_if_world_readable() {
    let src = read_or_panic("src/cmd/seed_xor.rs");
    assert!(
        src.contains("warn_if_world_readable("),
        "src/cmd/seed_xor.rs must call secret_advisory::warn_if_world_readable \
         (R0 Q5 fold); got source without that helper reference"
    );
}

#[test]
fn cmd_final_word_calls_shared_warn_if_world_readable() {
    let src = read_or_panic("src/cmd/final_word.rs");
    assert!(
        src.contains("warn_if_world_readable("),
        "src/cmd/final_word.rs must call secret_advisory::warn_if_world_readable \
         (R0 Q5 fold); got source without that helper reference"
    );
}

#[test]
fn cmd_slip39_calls_shared_warn_if_world_readable() {
    let src = read_or_panic("src/cmd/slip39.rs");
    assert!(
        src.contains("warn_if_world_readable("),
        "src/cmd/slip39.rs must call secret_advisory::warn_if_world_readable \
         (R0 Q5 fold; first-time call site for slip39); got source without it"
    );
}

#[test]
fn cmd_seed_xor_no_longer_defines_private_emit_world_readable() {
    // Partial-migration guard: after extraction, the private helper
    // definition in cmd/seed_xor.rs is deleted. Re-defining it would
    // mean two parallel helpers exist — the lint catches that.
    let src = read_or_panic("src/cmd/seed_xor.rs");
    assert!(
        !src.contains("fn emit_world_readable_advisory"),
        "src/cmd/seed_xor.rs must NOT define its own fn emit_world_readable_advisory \
         after the extraction to secret_advisory::warn_if_world_readable \
         (Q5 fold partial-migration guard); found the private definition still in place"
    );
}
