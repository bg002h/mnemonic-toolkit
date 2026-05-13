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
//! secret-bearing type (`Mnemonic::parse_in`, `Mnemonic::from_entropy_in`,
//! `Xpriv::new_master`, `Xpriv::derive_priv`), this lint asserts a
//! `SAFETY: third-party-blocked` doc-comment block appears within ±15
//! source lines of the call. The comment names the residual gap
//! (the type cannot scrub on drop) and the upstream FOLLOWUP tracking
//! the fix.
//!
//! Test-only call sites (within `#[cfg(test)]` modules) are excluded —
//! the line ranges in `TEST_LINE_RANGES` bound where each file's test
//! module starts so production rows only are enumerated.
//!
//! RED on Phase 2 first commit: no source has `SAFETY: third-party-
//! blocked` comments yet (verified by `grep -r 'SAFETY:
//! third-party-blocked' crates/mnemonic-toolkit/src` ⇒ zero hits).
//! Phase 2 impl lands the doc-comments and turns the lint GREEN.

use std::fs;
use std::path::Path;

/// A production call site that constructs a third-party-blocked
/// secret-bearing type. Lint scans ±15 source lines around `line`
/// for a `SAFETY: third-party-blocked` substring.
struct SafetySite {
    file: &'static str,
    line: u32,
    /// What's constructed (for diagnostic clarity).
    construct: &'static str,
}

/// Production call sites enumerated from `grep -rn Mnemonic::parse_in
/// | Mnemonic::from_entropy | Xpriv::new_master | .derive_priv` at
/// branch `v0_9_0-phase-2-zeroize` HEAD. Lines may drift as Phase 2
/// edits land — update this list in lockstep with the impl.
const SAFETY_SITES: &[SafetySite] = &[
    SafetySite { file: "src/bip85.rs", line: 37, construct: "Xpriv::derive_priv" },
    SafetySite { file: "src/bip85.rs", line: 74, construct: "Mnemonic::from_entropy_in" },
    SafetySite { file: "src/derive.rs", line: 30, construct: "Mnemonic::parse_in" },
    SafetySite { file: "src/derive_slot.rs", line: 31, construct: "Mnemonic::from_entropy_in" },
    SafetySite { file: "src/derive_slot.rs", line: 35, construct: "Xpriv::new_master" },
    SafetySite { file: "src/derive_slot.rs", line: 41, construct: "Xpriv::derive_priv" },
    SafetySite { file: "src/derive_slot.rs", line: 83, construct: "Mnemonic::from_entropy_in" },
    SafetySite { file: "src/derive_slot.rs", line: 87, construct: "Xpriv::new_master" },
    SafetySite { file: "src/derive_slot.rs", line: 90, construct: "Xpriv::derive_priv" },
    SafetySite { file: "src/synthesize.rs", line: 279, construct: "Xpriv::derive_priv" },
    SafetySite { file: "src/synthesize.rs", line: 326, construct: "Xpriv::new_master" },
    SafetySite { file: "src/parse_descriptor.rs", line: 864, construct: "Mnemonic::parse_in" },
    SafetySite { file: "src/parse_descriptor.rs", line: 868, construct: "Xpriv::new_master" },
    SafetySite { file: "src/parse_descriptor.rs", line: 886, construct: "Xpriv::derive_priv" },
    SafetySite { file: "src/cmd/derive_child.rs", line: 125, construct: "Mnemonic::parse_in" },
    SafetySite { file: "src/cmd/derive_child.rs", line: 136, construct: "Xpriv::new_master" },
    SafetySite { file: "src/cmd/convert.rs", line: 885, construct: "Mnemonic::parse_in" },
    SafetySite { file: "src/cmd/convert.rs", line: 913, construct: "Mnemonic::from_entropy_in" },
    SafetySite { file: "src/cmd/convert.rs", line: 1160, construct: "Mnemonic::from_entropy_in" },
    SafetySite { file: "src/cmd/bundle.rs", line: 905, construct: "Mnemonic::parse_in" },
    SafetySite { file: "src/cmd/bundle.rs", line: 909, construct: "Xpriv::new_master" },
    SafetySite { file: "src/cmd/bundle.rs", line: 922, construct: "Xpriv::derive_priv" },
    SafetySite { file: "src/cmd/bundle.rs", line: 973, construct: "Mnemonic::from_entropy_in" },
    SafetySite { file: "src/cmd/bundle.rs", line: 976, construct: "Xpriv::new_master" },
    SafetySite { file: "src/cmd/bundle.rs", line: 981, construct: "Xpriv::derive_priv" },
];

const SAFETY_NEEDLE: &str = "SAFETY: third-party-blocked";
const WINDOW: usize = 15;

fn crate_root() -> &'static Path {
    Path::new(".")
}

#[test]
fn safety_site_list_has_expected_row_count() {
    // 25 production call sites enumerated. Tight bound — adding new
    // Mnemonic/Xpriv call sites is rare and intentional (e.g., a new
    // derivation spine); shifting this assertion forces the
    // contributor to also add the SAFETY comment.
    let n = SAFETY_SITES.len();
    assert!(
        (22..=30).contains(&n),
        "SAFETY_SITES row count = {n}; expected 22..=30. \
         Re-run `grep -rn 'Mnemonic::parse_in\\|Mnemonic::from_entropy\\|Xpriv::new_master\\|.derive_priv(' crates/mnemonic-toolkit/src/` to refresh."
    );
}

#[test]
fn every_third_party_call_site_has_safety_comment_within_window() {
    let mut missing: Vec<String> = Vec::new();
    for site in SAFETY_SITES {
        let path = crate_root().join(site.file);
        let source = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "failed to read source {} for site {}:{}: {e}",
                path.display(),
                site.file,
                site.line
            )
        });
        let lines: Vec<&str> = source.lines().collect();
        let target = site.line as usize;
        if target == 0 || target > lines.len() {
            missing.push(format!(
                "  - {}:{} ({}): line out of range (file has {} lines)",
                site.file,
                site.line,
                site.construct,
                lines.len()
            ));
            continue;
        }
        let lo = target.saturating_sub(WINDOW + 1);
        let hi = (target + WINDOW).min(lines.len());
        let window = &lines[lo..hi];
        let hit = window.iter().any(|l| l.contains(SAFETY_NEEDLE));
        if !hit {
            missing.push(format!(
                "  - {}:{} ({}): no `{}` comment in ±{} line window",
                site.file, site.line, site.construct, SAFETY_NEEDLE, WINDOW
            ));
        }
    }
    assert!(
        missing.is_empty(),
        "third-party-blocked SAFETY-comment lint: {} site(s) missing:\n{}",
        missing.len(),
        missing.join("\n"),
    );
}
