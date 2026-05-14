//! Phase 1 argv-leakage closure — parametric lint over the canonical
//! list of toolkit secret-bearing flag-rows.
//!
//! Authoritative reference: `design/SPEC_secret_memory_hygiene_v0_9_0.md`
//! §1 item 1 + survey §5 toolkit table (20 flag-rows enumerated).
//!
//! For each flag-row, this lint asserts that the implementing source
//! file contains a stable evidence anchor proving the row has a paired
//! `*-stdin` flag or `=-` carve-out wired. The lint is intentionally
//! source-grep, not behavior — its job is to ensure no flag-row is ever
//! added without a paired stdin route. New secret-bearing flag-rows
//! must be added to `CANONICAL_FLAG_ROWS` AND ship their stdin
//! alternative in the same PR.
//!
//! RED on Phase 1 first commit: 9 of 20 rows lack their evidence
//! anchor (the rows newly closed by this cycle — see
//! `cli_argv_leakage.rs` and plan §"Phase 1 — Impl"). Phase 1 impl lands
//! the anchors and turns the lint GREEN.
//!
//! Modeled on the schema-mirror lint precedent
//! (`docs/manual/tests/lint.sh`); the canonical list lives inline rather
//! than being derived from clap so the lint catches accidental flag
//! removals AND ensures the SPEC table remains the single source of
//! truth.

use std::fs;
use std::path::Path;

/// A toolkit secret-bearing flag-row + the evidence anchor proving its
/// stdin alternative is wired. The lint asserts `source_file` contains
/// at least one of the `evidence` strings.
struct FlagRow {
    /// Human-readable flag identifier, mirroring survey §5's table column.
    label: &'static str,
    /// Path to the implementing source file, relative to the
    /// `crates/mnemonic-toolkit/` crate root.
    source_file: &'static str,
    /// Any one of these substrings appearing in `source_file` proves
    /// the row's stdin alternative is wired. OR semantics — first hit
    /// wins.
    evidence: &'static [&'static str],
}

/// Canonical list of 20 toolkit secret-bearing flag-rows per survey §5
/// (toolkit subtable). When adding a new secret-bearing flag in
/// `cmd/*.rs`, add a row here AND wire its stdin alternative.
const CANONICAL_FLAG_ROWS: &[FlagRow] = &[
    // ---- bundle (5 rows) ----
    FlagRow {
        label: "bundle --passphrase",
        source_file: "src/cmd/bundle.rs",
        evidence: &["passphrase_stdin", "passphrase-stdin"],
    },
    FlagRow {
        label: "bundle --slot @N.phrase=",
        source_file: "src/cmd/bundle.rs",
        evidence: &["slot_stdin", "slot-stdin"],
    },
    FlagRow {
        label: "bundle --slot @N.entropy=",
        source_file: "src/cmd/bundle.rs",
        evidence: &["slot_stdin", "slot-stdin"],
    },
    FlagRow {
        label: "bundle --slot @N.wif=",
        source_file: "src/cmd/bundle.rs",
        evidence: &["slot_stdin", "slot-stdin"],
    },
    FlagRow {
        label: "bundle --slot @N.xprv=",
        source_file: "src/cmd/bundle.rs",
        evidence: &["slot_stdin", "slot-stdin"],
    },
    // ---- verify-bundle (2 rows) ----
    FlagRow {
        label: "verify-bundle --passphrase",
        source_file: "src/cmd/verify_bundle.rs",
        evidence: &["passphrase_stdin", "passphrase-stdin"],
    },
    FlagRow {
        label: "verify-bundle --slot @N.<secret>=",
        source_file: "src/cmd/verify_bundle.rs",
        evidence: &["slot_stdin", "slot-stdin"],
    },
    // ---- convert =- rows (8 from + 2 passphrase variants = 10 rows) ----
    FlagRow {
        label: "convert --from phrase=",
        source_file: "src/cmd/convert.rs",
        evidence: &["read_stdin_to_string", "value == \"-\""],
    },
    FlagRow {
        label: "convert --from entropy=",
        source_file: "src/cmd/convert.rs",
        evidence: &["read_stdin_to_string", "value == \"-\""],
    },
    FlagRow {
        label: "convert --from xprv=",
        source_file: "src/cmd/convert.rs",
        evidence: &["read_stdin_to_string", "value == \"-\""],
    },
    FlagRow {
        label: "convert --from wif=",
        source_file: "src/cmd/convert.rs",
        evidence: &["read_stdin_to_string", "value == \"-\""],
    },
    FlagRow {
        label: "convert --from ms1=",
        source_file: "src/cmd/convert.rs",
        evidence: &["read_stdin_to_string", "value == \"-\""],
    },
    FlagRow {
        label: "convert --from bip38=",
        source_file: "src/cmd/convert.rs",
        evidence: &["read_stdin_to_string", "value == \"-\""],
    },
    FlagRow {
        label: "convert --from minikey=",
        source_file: "src/cmd/convert.rs",
        evidence: &["read_stdin_to_string", "value == \"-\""],
    },
    FlagRow {
        label: "convert --from electrum-phrase=",
        source_file: "src/cmd/convert.rs",
        evidence: &["read_stdin_to_string", "value == \"-\""],
    },
    FlagRow {
        label: "convert --passphrase",
        source_file: "src/cmd/convert.rs",
        evidence: &["passphrase_stdin", "passphrase-stdin"],
    },
    FlagRow {
        label: "convert --bip38-passphrase",
        source_file: "src/cmd/convert.rs",
        evidence: &["bip38_passphrase_stdin", "bip38-passphrase-stdin"],
    },
    // ---- derive-child (3 rows) ----
    FlagRow {
        label: "derive-child --from xprv=",
        source_file: "src/cmd/derive_child.rs",
        evidence: &["read_stdin_to_string", "value == \"-\""],
    },
    FlagRow {
        label: "derive-child --from phrase=",
        source_file: "src/cmd/derive_child.rs",
        evidence: &["read_stdin_to_string", "value == \"-\""],
    },
    FlagRow {
        label: "derive-child --passphrase",
        source_file: "src/cmd/derive_child.rs",
        evidence: &["passphrase_stdin", "passphrase-stdin"],
    },
    // ---- final-word (1 row) — v0.11.0 ----
    FlagRow {
        label: "final-word --from phrase=",
        source_file: "src/cmd/final_word.rs",
        evidence: &["phrase=-", "secret_in_argv_warning"],
    },
    // ---- seed-xor (2 rows) — v0.12.0 ----
    FlagRow {
        label: "seed-xor split --from phrase=",
        source_file: "src/cmd/seed_xor.rs",
        evidence: &["--from phrase=-", "secret_in_argv_warning"],
    },
    FlagRow {
        label: "seed-xor combine --share phrase=",
        source_file: "src/cmd/seed_xor.rs",
        evidence: &["--share phrase=-", "secret_in_argv_warning"],
    },
    // ---- slip39 (5 rows) — v0.13.0 ----
    FlagRow {
        label: "slip39 split --from phrase=",
        source_file: "src/cmd/slip39.rs",
        evidence: &["--from phrase=-", "secret_in_argv_warning"],
    },
    FlagRow {
        label: "slip39 split --from entropy=",
        source_file: "src/cmd/slip39.rs",
        evidence: &["--from entropy=-", "secret_in_argv_warning"],
    },
    FlagRow {
        label: "slip39 split --passphrase",
        source_file: "src/cmd/slip39.rs",
        evidence: &["passphrase_stdin", "passphrase-stdin"],
    },
    FlagRow {
        label: "slip39 combine --share",
        source_file: "src/cmd/slip39.rs",
        evidence: &["--share -", "secret_in_argv_warning"],
    },
    FlagRow {
        label: "slip39 combine --passphrase",
        source_file: "src/cmd/slip39.rs",
        evidence: &["passphrase_stdin", "passphrase-stdin"],
    },
];

fn crate_root() -> &'static Path {
    // Tests run with CWD == crate dir (`crates/mnemonic-toolkit`) under
    // `cargo test`, so the source files resolve relative to the CWD.
    Path::new(".")
}

#[test]
fn canonical_list_has_twenty_eight_rows() {
    // v0.9.0 baseline = 20; v0.11.0 final-word +1; v0.12.0 seed-xor +2;
    // v0.13.0 slip39 +5 = 28. The slip39 contribution adds 5 rows
    // (one per (subcommand, flag) pair) — split: --from phrase=, --from
    // entropy=, --passphrase; combine: --share, --passphrase — per
    // PLAN_v0_13_0_p2.md §4.3 + Q1 fold (the SPEC §4 G6 enumeration
    // ships +5, NOT the +4 that initial drafts suggested; the paired
    // SPEC §4 G6 count `23 → 28` patch lands at P2.2 GREEN).
    assert_eq!(
        CANONICAL_FLAG_ROWS.len(),
        28,
        "survey §5 toolkit subtable enumerates 28 secret-bearing flag-rows \
         (20 v0.9.0 + 1 v0.11.0 final-word + 2 v0.12.0 seed-xor + 5 \
         v0.13.0 slip39); the canonical list must match exactly. \
         Adjust both in lockstep."
    );
}

#[test]
fn every_canonical_flag_row_has_stdin_evidence() {
    let mut missing: Vec<String> = Vec::new();
    for row in CANONICAL_FLAG_ROWS {
        let path = crate_root().join(row.source_file);
        let source = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "failed to read evidence source {} for row {:?}: {e}",
                path.display(),
                row.label
            )
        });
        let hit = row.evidence.iter().any(|needle| source.contains(needle));
        if !hit {
            missing.push(format!(
                "  - {} ({}): no evidence anchor found; expected one of {:?}",
                row.label, row.source_file, row.evidence,
            ));
        }
    }
    assert!(
        missing.is_empty(),
        "argv-leakage lint: {} flag-row(s) missing stdin-route evidence:\n{}",
        missing.len(),
        missing.join("\n"),
    );
}
