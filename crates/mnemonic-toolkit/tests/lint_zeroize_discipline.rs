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
    ZeroizeRow {
        label: "DerivedAccount impl Drop scrubs entropy on drop",
        source_file: "src/derive.rs",
        evidence: &["impl Drop for DerivedAccount", "Zeroize"],
    },
    ZeroizeRow {
        label: "DerivedAccount::into_parts() consuming method (migration anchor)",
        source_file: "src/derive.rs",
        evidence: &["fn into_parts", "into_parts(self)"],
    },
    ZeroizeRow {
        label: "derive_full() entropy local wraps before move into DerivedAccount",
        source_file: "src/derive.rs",
        evidence: &["Zeroizing", "Zeroize"],
    },
    // ---- derive_slot.rs (consolidated seed-helper + spine) ----
    ZeroizeRow {
        label: "derive_master_seed helper consolidates the 5 BIP-39→BIP-32 seed sites",
        source_file: "src/derive_slot.rs",
        evidence: &["fn derive_master_seed", "Zeroizing<[u8; 64]>"],
    },
    ZeroizeRow {
        label: "derive_bip32_from_entropy seed wrapped via derive_master_seed",
        source_file: "src/derive_slot.rs",
        evidence: &["derive_master_seed", "Zeroizing"],
    },
    ZeroizeRow {
        label: "derive_bip32_at_path seed wrapped via derive_master_seed",
        source_file: "src/derive_slot.rs",
        evidence: &["derive_master_seed", "Zeroizing"],
    },
    // ---- bip85.rs (master-secret derivation) ----
    ZeroizeRow {
        label: "bip85::derive_entropy returns Zeroizing<[u8; 64]>",
        source_file: "src/bip85.rs",
        evidence: &["Zeroizing<[u8; 64]>", "-> Zeroizing"],
    },
    ZeroizeRow {
        label: "bip85::format_bip39_phrase wraps entropy local",
        source_file: "src/bip85.rs",
        evidence: &["Zeroizing"],
    },
    ZeroizeRow {
        label: "bip85::format_hd_seed_wif wraps entropy",
        source_file: "src/bip85.rs",
        evidence: &["Zeroizing"],
    },
    ZeroizeRow {
        label: "bip85::format_xprv_child wraps entropy + privkey scalar",
        source_file: "src/bip85.rs",
        evidence: &["Zeroizing"],
    },
    ZeroizeRow {
        label: "bip85::format_hex_bytes/base64/base85/dice_rolls wrap entropy",
        source_file: "src/bip85.rs",
        evidence: &["Zeroizing"],
    },
    // ---- synthesize.rs ----
    ZeroizeRow {
        label: "synthesize_multisig_full seed wrapped via derive_master_seed",
        source_file: "src/synthesize.rs",
        evidence: &["derive_master_seed", "Zeroizing"],
    },
    ZeroizeRow {
        label: "synthesize_multisig_full entropy local wraps",
        source_file: "src/synthesize.rs",
        evidence: &["Zeroizing"],
    },
    // ResolvedSlot.entropy field-type change deferred to FOLLOWUPS
    // `resolved-slot-entropy-zeroizing-field` (19-site cascade; not
    // representative of the per-row wrap discipline this lint is
    // enforcing — local wraps at producer + consumer sites cover the
    // value's transit, only the field-resident copy is unwrapped).
    ZeroizeRow {
        label: "synthesize_unified ms1 build wraps cloned entropy",
        source_file: "src/synthesize.rs",
        evidence: &["Zeroizing"],
    },
    // ---- parse_descriptor.rs ----
    ZeroizeRow {
        label: "bind_full_mode seed wrapped via derive_master_seed",
        source_file: "src/parse_descriptor.rs",
        evidence: &["derive_master_seed", "Zeroizing"],
    },
    // ---- cmd/bundle.rs ----
    ZeroizeRow {
        label: "bundle args.passphrase clone wraps in Zeroizing",
        source_file: "src/cmd/bundle.rs",
        evidence: &["Zeroizing"],
    },
    ZeroizeRow {
        label: "bundle resolve_slots Phrase arm uses into_parts (not direct field move)",
        source_file: "src/cmd/bundle.rs",
        evidence: &["into_parts"],
    },
    // ---- cmd/verify_bundle.rs ----
    ZeroizeRow {
        label: "verify_bundle entropy_at_0 clone wraps in Zeroizing",
        source_file: "src/cmd/verify_bundle.rs",
        evidence: &["Zeroizing"],
    },
    // ---- cmd/derive_child.rs ----
    ZeroizeRow {
        label: "derive-child from_value wraps in Zeroizing<String>",
        source_file: "src/cmd/derive_child.rs",
        evidence: &["Zeroizing"],
    },
    // ---- cmd/convert.rs (per-arm wraps) ----
    ZeroizeRow {
        label: "convert compute_outputs Phrase/Entropy arm wraps entropy + leaf privkey",
        source_file: "src/cmd/convert.rs",
        evidence: &["Zeroizing"],
    },
    ZeroizeRow {
        label: "convert Wif arm wraps PrivateKey.inner",
        source_file: "src/cmd/convert.rs",
        evidence: &["Zeroizing"],
    },
    ZeroizeRow {
        label: "convert Bip38 decrypt arm wraps raw 32-B privkey",
        source_file: "src/cmd/convert.rs",
        evidence: &["Zeroizing"],
    },
    ZeroizeRow {
        label: "convert Ms1 arm wraps entropy",
        source_file: "src/cmd/convert.rs",
        evidence: &["Zeroizing"],
    },
    ZeroizeRow {
        label: "convert MiniKey arm wraps raw 32-B privkey",
        source_file: "src/cmd/convert.rs",
        evidence: &["Zeroizing"],
    },
    ZeroizeRow {
        label: "convert ElectrumPhrase arm wraps entropy",
        source_file: "src/cmd/convert.rs",
        evidence: &["Zeroizing"],
    },
    // ---- electrum.rs ----
    ZeroizeRow {
        label: "electrum phrase_to_entropy accumulator wraps Vec<u8>",
        source_file: "src/electrum.rs",
        evidence: &["Zeroizing"],
    },
    ZeroizeRow {
        label: "electrum entropy_to_phrase accumulator wraps Vec<u8>",
        source_file: "src/electrum.rs",
        evidence: &["Zeroizing"],
    },
];

fn crate_root() -> &'static Path {
    Path::new(".")
}

#[test]
fn canonical_zeroize_list_has_expected_row_count() {
    // ~27 rows after deferring the ResolvedSlot.entropy field-type
    // change to FOLLOWUPS `resolved-slot-entropy-zeroizing-field`.
    // Loose bound (24..=35) so adding/removing a polished site doesn't
    // trip the lint; the per-row evidence test below is the
    // authoritative check.
    let n = ZEROIZE_ROWS.len();
    assert!(
        (24..=35).contains(&n),
        "ZEROIZE_ROWS row count = {n}; expected ~27 (plan §Phase 2 minus deferred field-type row). \
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
