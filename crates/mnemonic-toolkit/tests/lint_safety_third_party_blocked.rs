//! v0.9.0 Cycle A Phase 2 — `SAFETY: third-party-blocked` doc-comment
//! discipline lint.
//!
//! Authoritative reference:
//! - `design/SPEC_secret_memory_hygiene_v0_9_0.md` §3 OOS rows for
//!   `bip39::Mnemonic` (CRATE-OWNED, no Drop+Zeroize) and
//!   `bitcoin::bip32::Xpriv` (Copy + no Drop + no Zeroize).
//! - `/home/bcg/.claude/plans/v0_9_0-secret-memory-hygiene.md`
//!   §"Phase 2 — Impl" step 4 (Document residual gap in a
//!   `SAFETY: third-party-blocked` doc-comment block at each call
//!   site).
//!
//! For each production call site that constructs a third-party-blocked
//! secret-bearing type, this lint asserts a `SAFETY: third-party-blocked`
//! doc-comment appears within ±15 source lines of the call.
//!
//! The lint scans the source text for the call patterns directly
//! (rather than hardcoded line numbers) so it tolerates upstream edits
//! that shift line offsets. Test-only call sites (within
//! `#[cfg(test)]` modules) are excluded by skipping any call that
//! follows a `#[cfg(test)]` attribute earlier in the file.
//!
//! RED on Phase 2 first commit: no source has `SAFETY:
//! third-party-blocked` comments yet. Phase 2 impl lands the
//! doc-comments and turns the lint GREEN alongside the OWNED-row
//! zeroize-discipline lint.

use std::fs;
use std::path::Path;

/// Files that contain production Mnemonic / Xpriv construction calls.
/// (Lint will scan each file's lines for the call patterns and check
/// for the SAFETY comment within ±WINDOW lines of each match.)
const SCAN_FILES: &[&str] = &[
    "src/bip85.rs",
    "src/derive.rs",
    "src/derive_slot.rs",
    "src/synthesize.rs",
    "src/parse_descriptor.rs",
    "src/cmd/derive_child.rs",
    "src/cmd/convert.rs",
    "src/cmd/bundle.rs",
];

/// Substrings whose appearance on a non-test source line constitutes
/// a third-party-blocked secret-bearing construction site.
const CALL_PATTERNS: &[&str] = &[
    "Mnemonic::parse_in",
    "Mnemonic::from_entropy_in",
    "Xpriv::new_master",
    ".derive_priv(",
    // SPEC v0.9.0 R1 fold I-2 — `secp256k1::SecretKey` is third-party-blocked
    // (non_secure_erase only, no Drop+Zeroize). Per FOLLOWUP
    // `rust-secp256k1-secretkey-zeroize-upstream`.
    "SecretKey::from_slice",
];

const SAFETY_NEEDLE: &str = "SAFETY: third-party-blocked";
const WINDOW: usize = 15;

fn crate_root() -> &'static Path {
    Path::new(".")
}

fn is_in_test_module(lines: &[&str], line_idx: usize) -> bool {
    // Walk backward from the call line; if we see `#[cfg(test)]` before
    // we see a top-level `fn`/`pub fn`/end-of-file boundary, the call
    // is in a test module.
    for i in (0..=line_idx).rev() {
        let l = lines[i].trim_start();
        if l.starts_with("#[cfg(test)]") {
            return true;
        }
        // A top-level fn at column 0 (no leading whitespace) signals
        // we've walked past the test-mod boundary into a production
        // function definition. Combined with the file structure
        // convention (`#[cfg(test)] mod tests { ... }` at file end),
        // this is sufficient.
        if !lines[i].is_empty()
            && !lines[i].starts_with(|c: char| c.is_whitespace())
            && (l.starts_with("pub fn ") || l.starts_with("fn ") || l.starts_with("pub(crate) fn "))
        {
            return false;
        }
    }
    false
}

#[test]
fn every_third_party_call_site_has_safety_comment_within_window() {
    let mut missing: Vec<String> = Vec::new();
    let mut sites_checked = 0;
    for path_rel in SCAN_FILES {
        let path = crate_root().join(path_rel);
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read source {}: {e}", path.display()));
        let lines: Vec<&str> = source.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let pattern = CALL_PATTERNS.iter().find(|p| line.contains(**p));
            let Some(pattern) = pattern else { continue };
            if is_in_test_module(&lines, i) {
                continue;
            }
            sites_checked += 1;
            let lo = i.saturating_sub(WINDOW);
            let hi = (i + WINDOW + 1).min(lines.len());
            let window = &lines[lo..hi];
            let hit = window.iter().any(|l| l.contains(SAFETY_NEEDLE));
            if !hit {
                missing.push(format!(
                    "  - {}:{} ({}): no `{}` in ±{} line window",
                    path_rel,
                    i + 1,
                    pattern,
                    SAFETY_NEEDLE,
                    WINDOW
                ));
            }
        }
    }
    // Sanity: lint should be exercising ≥ 20 production sites.
    assert!(
        sites_checked >= 20,
        "SAFETY lint checked only {sites_checked} production sites; expected ≥ 20. \
         The is_in_test_module heuristic may be over-rejecting."
    );
    assert!(
        missing.is_empty(),
        "third-party-blocked SAFETY-comment lint: {} site(s) missing (out of {sites_checked} checked):\n{}",
        missing.len(),
        missing.join("\n"),
    );
}
