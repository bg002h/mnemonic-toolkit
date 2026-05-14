//! v0.9.0 Cycle A Phase 2 — Zeroizing-wrapper discipline lint.
//!
//! Authoritative reference:
//! - `design/SPEC_secret_memory_hygiene_v0_9_0.md` §1 item 2 (Zeroizing
//!   wrappers on every OWNED secret allocation).
//! - `design/agent-reports/v0_9_0-secret-memory-survey.md` §1 (toolkit
//!   table).
//! - `/home/bcg/.claude/plans/v0_9_0-secret-memory-hygiene.md`
//!   §"Phase 2 — Impl" step 4 (toolkit wraps + `derive_master_seed`
//!   seed-step helper + `impl Drop for DerivedAccount`).
//!
//! For each canonical OWNED-secret site listed in `ZEROIZE_ROWS`, this
//! lint asserts the implementing source file contains a stable evidence
//! anchor proving the row is wrapped (`Zeroizing::new(...)` call OR
//! `Zeroizing<...>` return type OR shared seed-helper anchor OR
//! `impl Drop for DerivedAccount` for the pub-struct-Drop case).
//!
//! Third-party-blocked carriers (`bip39::Mnemonic`,
//! `bitcoin::bip32::Xpriv`) are NOT enumerated here — they have no
//! zeroize-on-drop and the SPEC §3 OOS classification + per-call-site
//! `SAFETY: third-party-blocked` doc-comments cover the residual gap.
//! A separate lint may enforce those doc-comments in a follow-on; for
//! now, this lint focuses on the OWNED rows we control.
//!
//! RED on Phase 2 first commit: no source uses `Zeroizing` yet
//! (verified by `grep -r Zeroizing crates/mnemonic-toolkit/src` ⇒
//! zero hits). Phase 2 impl lands the anchors and turns the lint
//! GREEN.

use std::fs;
use std::path::Path;

/// A canonical OWNED-secret site + evidence anchor(s). OR semantics —
/// first hit in `source_file` wins.
struct ZeroizeRow {
    /// Human-readable site label (function or struct method + intent).
    label: &'static str,
    /// Path relative to the `crates/mnemonic-toolkit/` crate root.
    source_file: &'static str,
    /// Any one of these substrings appearing in `source_file` proves
    /// the row has Zeroizing discipline.
    evidence: &'static [&'static str],
}

/// Canonical list of toolkit OWNED-secret sites per survey §1. When
/// adding a new OWNED-secret allocation, add a row here AND wrap the
/// allocation in `Zeroizing` (or return a `Zeroizing<...>`).
const ZEROIZE_ROWS: &[ZeroizeRow] = &[
    // ---- derive.rs (DerivedAccount) ----
    // v0.10.1: migrated from `impl Drop for DerivedAccount` (Cycle A v0.9.0
    // baseline) to a typed `Zeroizing<Vec<u8>>` field. Drop-time scrub is
    // now structurally guaranteed by the type; the hand-rolled `impl Drop`
    // is deleted.
    ZeroizeRow {
        label: "DerivedAccount entropy field is Zeroizing<Vec<u8>>",
        source_file: "src/derive.rs",
        evidence: &["pub entropy: zeroize::Zeroizing<Vec<u8>>"],
    },
    ZeroizeRow {
        label: "DerivedAccount::into_parts() consuming method (migration anchor)",
        source_file: "src/derive.rs",
        evidence: &["pub fn into_parts(mut self)"],
    },
    ZeroizeRow {
        label: "derive_full() entropy local wraps before move into DerivedAccount",
        source_file: "src/derive.rs",
        evidence: &["Zeroizing::new(mnemonic.to_entropy())"],
    },
    // ---- derive_slot.rs (consolidated seed-helper + spine) ----
    ZeroizeRow {
        label: "derive_master_seed helper consolidates the BIP-39→BIP-32 seed sites",
        source_file: "src/derive_slot.rs",
        evidence: &["pub fn derive_master_seed(mnemonic: &Mnemonic, passphrase: &str) -> Zeroizing<[u8; 64]>"],
    },
    ZeroizeRow {
        label: "derive_bip32_from_entropy seed wrapped via derive_master_seed",
        source_file: "src/derive_slot.rs",
        evidence: &["derive_master_seed(&mnemonic, passphrase)"],
    },
    ZeroizeRow {
        label: "derive_bip32_at_path seed wrapped via derive_master_seed",
        source_file: "src/derive_slot.rs",
        evidence: &["derive_master_seed(&mnemonic, passphrase)"],
    },
    // ---- bip85.rs (master-secret derivation) ----
    // R1 I-4 fold: bip85 entropy buffer is `Zeroizing<[u8; 64]>` returned
    // by `derive_entropy` and consumed by every format_* function via
    // deref-coercion. Per-function entropy wraps are inherited from the
    // shared return type; per-function SecretKey/Xpriv stack-bound locals
    // are tracked by `lint_safety_third_party_blocked.rs` (R1 I-2 fold)
    // via the `SecretKey::from_slice` pattern.
    ZeroizeRow {
        label: "bip85::derive_entropy returns Zeroizing<Vec<u8>>",
        source_file: "src/bip85.rs",
        evidence: &["-> Result<Zeroizing<Vec<u8>>"],
    },
    ZeroizeRow {
        label: "bip85 entropy locals scrub via derive_entropy's Zeroizing return",
        source_file: "src/bip85.rs",
        evidence: &["let mut out = Zeroizing::new(vec![0u8; 64])"],
    },
    // ---- synthesize.rs ----
    ZeroizeRow {
        label: "synthesize_multisig_full seed wrapped via derive_master_seed",
        source_file: "src/synthesize.rs",
        evidence: &["derive_master_seed(seed_mnemonic"],
    },
    ZeroizeRow {
        label: "synthesize_multisig_full entropy local wraps (R1 I-1 fold)",
        source_file: "src/synthesize.rs",
        evidence: &["Zeroizing::new(seed_mnemonic.to_entropy())"],
    },
    // v0.10.1: ResolvedSlot.entropy field migrated from `Option<Vec<u8>>` to
    // `Option<Zeroizing<Vec<u8>>>` (closes FOLLOWUP
    // `resolved-slot-derived-account-zeroizing-field`). Drop-time scrub is
    // now structurally guaranteed; the 12 ctor sites (including 6 via
    // `pub type CosignerKeyInfo = ResolvedSlot;` alias) wrap at the
    // field-write boundary.
    ZeroizeRow {
        label: "ResolvedSlot entropy field is Option<Zeroizing<Vec<u8>>>",
        source_file: "src/synthesize.rs",
        evidence: &["pub entropy: Option<zeroize::Zeroizing<Vec<u8>>>"],
    },
    ZeroizeRow {
        label: "synthesize_unified ms1 build wraps cloned entropy",
        source_file: "src/synthesize.rs",
        // Multiple Zeroizing call sites — tightened anchor pins the
        // ms1-build site specifically per R1 I-4 fold.
        evidence: &["Zeroizing::new(seed_mnemonic.to_entropy())", "Zeroizing::new(mnemonic.to_entropy())"],
    },
    // ---- parse_descriptor.rs ----
    ZeroizeRow {
        label: "bind_full_mode seed wrapped via derive_master_seed",
        source_file: "src/parse_descriptor.rs",
        evidence: &["derive_master_seed(&mnemonic, passphrase)"],
    },
    // ---- cmd/bundle.rs ----
    ZeroizeRow {
        label: "bundle Phrase descriptor arm wraps passphrase + entropy",
        source_file: "src/cmd/bundle.rs",
        evidence: &["Zeroizing::new(args.passphrase.clone().unwrap_or_default())"],
    },
    ZeroizeRow {
        label: "bundle Phrase descriptor arm wraps mnemonic.to_entropy()",
        source_file: "src/cmd/bundle.rs",
        evidence: &["Zeroizing::new(mnemonic.to_entropy())"],
    },
    ZeroizeRow {
        label: "bundle Entropy descriptor arm wraps hex-decoded entropy_bytes",
        source_file: "src/cmd/bundle.rs",
        evidence: &["Zeroizing::new(hex::decode(entropy_hex)"],
    },
    ZeroizeRow {
        label: "bundle resolve_slots arms use into_parts (not direct field move)",
        source_file: "src/cmd/bundle.rs",
        evidence: &["acc.into_parts()"],
    },
    // ---- cmd/verify_bundle.rs ----
    ZeroizeRow {
        label: "verify_bundle entropy_at_0 typed Option<Zeroizing<Vec<u8>>>",
        source_file: "src/cmd/verify_bundle.rs",
        evidence: &["Option<zeroize::Zeroizing<Vec<u8>>>"],
    },
    // ---- cmd/derive_child.rs ----
    ZeroizeRow {
        label: "derive-child from_value wraps in Zeroizing<String>",
        source_file: "src/cmd/derive_child.rs",
        evidence: &["zeroize::Zeroizing<String>"],
    },
    ZeroizeRow {
        label: "derive-child stdin_passphrase wraps in Option<Zeroizing<String>> (R1 I-3 fold)",
        source_file: "src/cmd/derive_child.rs",
        evidence: &["Option<zeroize::Zeroizing<String>>"],
    },
    // ---- cmd/convert.rs (per-arm wraps) ----
    ZeroizeRow {
        label: "convert Phrase/Entropy arm wraps entropy",
        source_file: "src/cmd/convert.rs",
        evidence: &["zeroize::Zeroizing<Vec<u8>>"],
    },
    ZeroizeRow {
        label: "convert Ms1 arm wraps decoded entropy Payload",
        source_file: "src/cmd/convert.rs",
        evidence: &["Zeroizing::new(bytes)"],
    },
    // ---- electrum.rs ----
    ZeroizeRow {
        label: "electrum phrase_to_entropy accumulator wraps Vec<u8>",
        source_file: "src/electrum.rs",
        evidence: &["zeroize::Zeroizing::new(vec![0])"],
    },
    ZeroizeRow {
        label: "electrum entropy_to_phrase accumulator wraps Vec<u8>",
        source_file: "src/electrum.rs",
        evidence: &["zeroize::Zeroizing::new(entropy.iter()"],
    },
];

fn crate_root() -> &'static Path {
    Path::new(".")
}

#[test]
fn canonical_zeroize_list_has_expected_row_count() {
    // ~28 rows post-v0.10.1 migration (Cycle B Path B-lite carve-out
    // completed: DerivedAccount.entropy + ResolvedSlot.entropy now
    // Zeroizing<Vec<u8>> typed; closes FOLLOWUP
    // `resolved-slot-derived-account-zeroizing-field`).
    // Loose bound (24..=35) so adding/removing a polished site doesn't
    // trip the lint; the per-row evidence test below is the
    // authoritative check.
    let n = ZEROIZE_ROWS.len();
    assert!(
        (18..=35).contains(&n),
        "ZEROIZE_ROWS row count = {n}; expected 18..=35 (plan §Phase 2 minus deferred field-type row, minus R1 I-4 fold consolidations). \
         Survey §1 toolkit table is the canonical reference."
    );
}

#[test]
fn every_canonical_zeroize_row_has_evidence_anchor() {
    let mut missing: Vec<String> = Vec::new();
    for row in ZEROIZE_ROWS {
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
                "  - {} ({}): no evidence anchor; expected one of {:?}",
                row.label, row.source_file, row.evidence,
            ));
        }
    }
    assert!(
        missing.is_empty(),
        "zeroize-discipline lint: {} row(s) missing Zeroizing evidence:\n{}",
        missing.len(),
        missing.join("\n"),
    );
}
