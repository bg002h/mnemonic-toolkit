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
    // cycle-15t: the 7 `format_*` rendered child-secret returns
    // (phrase/WIF/xprv/hex/password/dice) are `Result<SecretString, _>` —
    // length-only redacting Debug; Display/Deref render verbatim so the
    // derive-child text path is byte-identical.
    ZeroizeRow {
        label: "bip85 format_* return rendered child secret as SecretString",
        source_file: "src/bip85.rs",
        evidence: &["-> Result<SecretString, ToolkitError>"],
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
    // cycle-15t: the single emitter local that aggregates every `format_*`
    // child secret is a SecretString (the 7 arms type-unify).
    ZeroizeRow {
        label: "derive-child output emitter local is SecretString",
        source_file: "src/cmd/derive_child.rs",
        evidence: &["let output: crate::secret_string::SecretString"],
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
    // cycle-15t: phrase_to_entropy returns the secret entropy BY MOVE
    // (Zeroizing<Vec<u8>>) instead of deref-cloning it out into a bare Vec.
    ZeroizeRow {
        label: "electrum phrase_to_entropy returns Zeroizing<Vec<u8>> by move (no clone-out)",
        source_file: "src/electrum.rs",
        evidence: &["-> Result<Zeroizing<Vec<u8>>, ElectrumError>"],
    },
    // cycle-15t: the normalize intermediates (norm_phrase/norm_pp +
    // HMAC-dispatch scratch) are Zeroizing<String> via the electrum-LOCAL
    // helper returns; the per-word scratch wraps at the consumption boundary
    // (M-4: wordlists::normalize_electrum itself stays `-> String`).
    ZeroizeRow {
        label: "electrum normalize_text_electrum + normalize_phrase_for_hmac return Zeroizing<String>",
        source_file: "src/electrum.rs",
        evidence: &["fn normalize_text_electrum(s: &str) -> Zeroizing<String>"],
    },
    ZeroizeRow {
        label: "electrum per-word normalize result wraps in Zeroizing at the call site",
        source_file: "src/electrum.rs",
        evidence: &["Zeroizing::new(normalize_electrum"],
    },
    // ---- cmd/final_word.rs (v0.11.0) ----
    ZeroizeRow {
        label: "final-word run() parsed partial wraps in Zeroizing<String>",
        source_file: "src/cmd/final_word.rs",
        evidence: &["zeroize::Zeroizing::new"],
    },
    // ---- cmd/seed_xor.rs (v0.12.0) ----
    ZeroizeRow {
        label: "seed-xor run() parsed master + shares wrap in Zeroizing<String> / Zeroizing<Vec<u8>>",
        source_file: "src/cmd/seed_xor.rs",
        evidence: &["zeroize::Zeroizing::new"],
    },
    // ---- slip39/mod.rs (v0.13.0 P1c-E.3 G6 hygiene) ----
    // The library cycle ships `slip39_combine` already returning
    // `Zeroizing<Vec<u8>>` (master) and `Share` already `ZeroizeOnDrop`
    // on its `value` field. P1c-E.3 closes the remaining gaps per plan
    // §3.6: intermediate secret buffers wrapped in `Zeroizing<Vec<u8>>`
    // and `mlock::pin_pages_for` called on the EMS in both split + combine.
    ZeroizeRow {
        label: "slip39 public surface (slip39_combine + recover_secret) returns Zeroizing<Vec<u8>>",
        source_file: "src/slip39/mod.rs",
        evidence: &["-> Result<Zeroizing<Vec<u8>>, Slip39Error>"],
    },
    ZeroizeRow {
        label: "split_secret wraps RNG-derived random_part `r` in Zeroizing",
        source_file: "src/slip39/mod.rs",
        evidence: &["Zeroizing::new(vec![0u8; random_len])"],
    },
    ZeroizeRow {
        label: "split_secret wraps digest_payload buffer in Zeroizing",
        source_file: "src/slip39/mod.rs",
        evidence: &["Zeroizing::new(Vec::with_capacity(n))"],
    },
    ZeroizeRow {
        label: "slip39_split + slip39_combine pin EMS pages via mlock::pin_pages_for",
        source_file: "src/slip39/mod.rs",
        // `slip39` is a library-exposed module in `lib.rs`; the crate-name
        // alias `mnemonic_toolkit::...` is unavailable from library code
        // and only works from `main.rs` / integration tests. Lib code
        // refers to sibling lib modules via `crate::mlock::...`.
        evidence: &["crate::mlock::pin_pages_for"],
    },
    // ---- cmd/slip39.rs (v0.13.0 P2.2) ----
    ZeroizeRow {
        label: "slip39 run() parsed --from + --share + --passphrase wrap in Zeroizing<String>",
        source_file: "src/cmd/slip39.rs",
        evidence: &["zeroize::Zeroizing::new"],
    },
    // ---- cmd/xpub_search/passphrase_search.rs (v0.46.0 candidate scan) ----
    ZeroizeRow {
        label: "passphrase-candidates-file scan wraps each candidate line in Zeroizing<String>",
        source_file: "src/cmd/xpub_search/passphrase_search.rs",
        evidence: &["Zeroizing::new(raw)"],
    },
    // ---- silent_payment.rs / nostr.rs (derived priv-key strings → SecretString) ----
    // v0.53.x (`silentpayment-nostr-priv-not-zeroizing`): the hex/WIF of derived
    // private keys is carried into `--json` / text output via SecretString
    // (Zeroizing<String> inner; serialize-transparent), so the heap copies
    // scrub on drop.
    ZeroizeRow {
        label: "silent-payment scan_priv hex wraps in SecretString",
        source_file: "src/cmd/silent_payment.rs",
        evidence: &["SecretString::new(hex::encode(b_scan.secret_bytes()))"],
    },
    ZeroizeRow {
        label: "silent-payment spend_priv hex wraps in SecretString",
        source_file: "src/cmd/silent_payment.rs",
        evidence: &["SecretString::new(hex::encode(b_spend.secret_bytes()))"],
    },
    ZeroizeRow {
        label: "nostr WIF wraps in SecretString from creation",
        source_file: "src/cmd/nostr.rs",
        evidence: &["SecretString::new(crate::nostr::wif_for"],
    },
    ZeroizeRow {
        label: "nostr electrum import string (embeds WIF) wraps in SecretString",
        source_file: "src/cmd/nostr.rs",
        evidence: &["SecretString::new(format!(\"{p}{wif}\"))"],
    },
    // ---- hand-frozen-lint-canons-no-completeness (2026-06-11 audit): the 14
    // previously-untracked CANONICAL owned-secret files, promoted to rows so
    // the new source→declared scan can require coverage. ----
    ZeroizeRow {
        label: "addresses seed arm wraps the master-secret entropy in Zeroizing<Vec<u8>>",
        source_file: "src/cmd/addresses.rs",
        evidence: &["zeroize::Zeroizing<Vec<u8>>", "Zeroizing::new"],
    },
    ZeroizeRow {
        label: "electrum-decrypt resolves the decrypt-password into Zeroizing<String>",
        source_file: "src/cmd/electrum_decrypt.rs",
        evidence: &["zeroize::Zeroizing<String>", "Zeroizing::new"],
    },
    ZeroizeRow {
        label: "import-wallet read_blob holds the (plaintext seed/xprv-bearing) wallet blob in Zeroizing<Vec<u8>>",
        source_file: "src/cmd/import_wallet.rs",
        evidence: &["Zeroizing<Vec<u8>>", "Zeroizing::new(fs::read"],
    },
    ZeroizeRow {
        label: "import-wallet decrypt-password + decrypted BSMS records wrap in Zeroizing<String>",
        source_file: "src/cmd/import_wallet.rs",
        evidence: &["Zeroizing<String>"],
    },
    ZeroizeRow {
        label: "ms-shares parse_secret_to_entropy returns the pre-split secret as Zeroizing<Vec<u8>>",
        source_file: "src/cmd/ms_shares.rs",
        evidence: &["zeroize::Zeroizing::new(m.to_entropy())"],
    },
    ZeroizeRow {
        label: "ms-shares combine recovers entropy + renders output in Zeroizing",
        source_file: "src/cmd/ms_shares.rs",
        evidence: &["zeroize::Zeroizing<String>"],
    },
    ZeroizeRow {
        label: "restore owns the seed entropy as Zeroizing<Vec<u8>> (run + resolve_seed_entropy)",
        source_file: "src/cmd/restore.rs",
        evidence: &["zeroize::Zeroizing<Vec<u8>>"],
    },
    ZeroizeRow {
        label: "seedqr owns the digits + phrase secrets in Zeroizing<String>",
        source_file: "src/cmd/seedqr.rs",
        evidence: &["zeroize::Zeroizing<String>", "Zeroizing::new"],
    },
    // cycle-15t: the LIB seedqr module wraps its internal scratch (raw-digit
    // `stripped`, per-word `words`/`normalized`/`digits`, decode_compact
    // hex-decoded `bytes`) in Zeroizing; M-2 keeps the four `pub fn` returns
    // bare `String` (no SemVer break).
    ZeroizeRow {
        label: "seedqr LIB module wraps internal scratch (digits/normalized/bytes) in Zeroizing",
        source_file: "src/seedqr.rs",
        evidence: &["Zeroizing::new"],
    },
    ZeroizeRow {
        label: "verify-bundle Phrase arm wraps passphrase + entropy in Zeroizing",
        source_file: "src/cmd/verify_bundle.rs",
        evidence: &["zeroize::Zeroizing::new(mnemonic.to_entropy())"],
    },
    ZeroizeRow {
        label: "verify-bundle Entropy arm wraps hex-decoded entropy in Zeroizing",
        source_file: "src/cmd/verify_bundle.rs",
        evidence: &["zeroize::Zeroizing::new(hex::decode"],
    },
    ZeroizeRow {
        label: "account-of-descriptor wraps the BIP-39 passphrase in Zeroizing<String>",
        source_file: "src/cmd/xpub_search/account_of_descriptor.rs",
        evidence: &["zeroize::Zeroizing<String>"],
    },
    ZeroizeRow {
        label: "passphrase-of-xpub wraps the mandatory BIP-39 passphrase in Zeroizing<String>",
        source_file: "src/cmd/xpub_search/passphrase_of_xpub.rs",
        evidence: &["zeroize::Zeroizing<String>"],
    },
    ZeroizeRow {
        label: "path-of-xpub wraps the BIP-39 passphrase in Zeroizing<String>",
        source_file: "src/cmd/xpub_search/path_of_xpub.rs",
        evidence: &["zeroize::Zeroizing<String>"],
    },
    ZeroizeRow {
        label: "xpub-search seed_intake owns the phrase/ms1 source + decoded entropy in Zeroizing",
        source_file: "src/cmd/xpub_search/seed_intake.rs",
        evidence: &["Phrase(Zeroizing<String>)", "Zeroizing<Vec<u8>>"],
    },
    ZeroizeRow {
        label: "seed_xor LIBRARY split/combine own shares + recovered master as Zeroizing<Vec<u8>>",
        source_file: "src/seed_xor.rs",
        evidence: &["zeroize::Zeroizing<Vec<u8>>"],
    },
    ZeroizeRow {
        label: "slot_ms1 Ms1SlotResolution.entropy field is Zeroizing<Vec<u8>>",
        source_file: "src/slot_ms1.rs",
        evidence: &["Zeroizing<Vec<u8>>", "Zeroizing::new"],
    },
    ZeroizeRow {
        label: "import-wallet overlay decodes cosigner ms1/phrase into Zeroizing<Vec<u8>> entropy",
        source_file: "src/wallet_import/overlay.rs",
        evidence: &["Zeroizing<Vec<u8>>", "Zeroizing::new"],
    },
    // ---- slot_input.rs (v0.67.0 — L22: stdin/@env: secret no longer lingers
    //      in a bare String; SlotInput.value is a SecretString) ----
    ZeroizeRow {
        label: "SlotInput value field is SecretString (Zeroizing<String> inner) — L22",
        source_file: "src/slot_input.rs",
        evidence: &["pub value: SecretString", "SecretString::new"],
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
        (18..=66).contains(&n),
        "ZEROIZE_ROWS row count = {n}; expected 18..=66. The upper bound carries headroom \
         above the current canonical-row count for near-term secret-site additions — widen \
         it deliberately when a cycle exceeds it. This count is a coarse drift tripwire; the \
         per-row evidence test below is authoritative. Survey §1 toolkit table is canonical."
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

// ---------------------------------------------------------------------------
// source→declared completeness scan (resolves
// `hand-frozen-lint-canons-no-completeness`).
//
// The `every_canonical_zeroize_row_has_evidence_anchor` test above proves
// the DECLARED direction: every row points at real wrapped source. The scan
// below proves the SOURCE direction: every src file that owns a secret is
// EITHER a canonical row OR an explicitly-audited exemption. Together they
// close the loop — a NEW secret-bearing file added later FAILS the scan
// until it gets a row or a deliberate allowlist decision.
// ---------------------------------------------------------------------------

/// Owned-secret allocation patterns. A src file matching ANY of these is
/// "secret-bearing" and MUST be a `ZEROIZE_ROWS.source_file` or in
/// `NON_ROW_SECRET_FILES`. Mirrors the live grep used to build the partition.
const SECRET_PATTERNS: &[&str] = &[
    "Zeroizing::new(",
    "SecretString::new(",
    ": Zeroizing<",
    ": SecretString",
];

/// Files that USE Zeroizing/SecretString but are NOT canonical owned-secret
/// rows (audited 2026-06-11; the 19-file zeroize audit). Each line: why it's
/// exempt. Kept SMALL and per-entry-audited — verify_bundle/ms_shares are
/// PROMOTED to rows, not allowlisted.
const NON_ROW_SECRET_FILES: &[&str] = &[
    "src/bsms_crypto.rs", // CRYPTO-INTERNAL: PBKDF2 AES key + AES-CTR plaintext buffer (consumer owns the plaintext)
    "src/electrum_crypto.rs", // CRYPTO-INTERNAL: ECIES/CBC primitive (AES key, scalar, ECDH shared secret, key block)
    "src/slip39/feistel.rs", // CRYPTO-INTERNAL: SLIP-0039 Feistel L/R halves + round key (consumer slip39/mod.rs owns output)
    "src/nostr.rs", // PASS-THROUGH: decode_nostr_key hands the decoded INPUT upstream; cmd/nostr.rs owns the derived secret
    "src/secret_string.rs", // PRIMITIVE: the SecretString newtype DEFINITION, not an allocation site
];

/// Files whose ONLY secret-pattern matches live inside a `#[cfg(test)]` region
/// (test fixtures), verified by `test_only_secret_files_confine_secret_patterns_to_cfg_test`.
/// Distinct from NON_ROW_SECRET_FILES (whole-file crypto-internal/primitive
/// exemptions) — these are exempt ONLY because the secret is test-scoped, so a
/// future PRODUCTION secret allocation above the cfg(test) marker is CAUGHT
/// (cycle-15 Group A, closing the bundle-unified whole-file-allowlist masking).
const TEST_ONLY_SECRET_FILES: &[&str] = &[
    "src/bundle_unified.rs", // the sole SecretString::new is the #[cfg(test)] s() SlotInput fixture (cycle-14 L22); SlotInput.value's canonical row is src/slot_input.rs
];

/// Persistent glob-cardinality floor. The partition is exactly 37
/// secret-bearing src files @ v0.67.0 (31 ROWS-source ∪ 6 allowlist; cycle-14
/// L22 added the slot_input.rs row + the bundle_unified.rs test-fixture
/// allowlist entry). Was 35 (30 ∪ 5) @ 438de94. The
/// floor fires only on the loss-of-coverage direction (count DROPS) — a
/// broken glob/path-prefix change that enumerates nothing would otherwise
/// make the scan vacuously pass. Deleting a secret-bearing file is a
/// conscious security-adjacent choice, so requiring a deliberate floor edit
/// is the correct friction. Mirrors the `ZEROIZE_ROWS.len()` count guard.
const SECRET_FILE_FLOOR: usize = 37;

/// Recursively collect every `*.rs` under `dir`, returning crate-root-relative
/// forward-slash paths (matching `ZEROIZE_ROWS.source_file` form).
fn collect_rs_files(dir: &Path, root: &Path, out: &mut Vec<String>) {
    let entries =
        fs::read_dir(dir).unwrap_or_else(|e| panic!("failed to read dir {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, root, out);
        } else if path.extension().and_then(|x| x.to_str()) == Some("rs") {
            let rel = path
                .strip_prefix(root)
                .expect("path under root")
                .to_string_lossy()
                .replace('\\', "/");
            out.push(rel);
        }
    }
}

fn file_is_secret_bearing(path: &Path) -> bool {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    SECRET_PATTERNS.iter().any(|p| source.contains(p))
}

/// Index of the first line containing `#[cfg(test)]`, if any.
fn first_cfg_test_line(src: &str) -> Option<usize> {
    src.lines().position(|l| l.contains("#[cfg(test)]"))
}

/// Line indices (0-based) of every line containing a SECRET_PATTERN that lies
/// BEFORE `boundary` (i.e. outside the test region). Substring-exact, same
/// SECRET_PATTERNS as the partition scan (NO comment-stripping — intended; a
/// SECRET_PATTERN substring in a doc/comment above #[cfg(test)] deliberately
/// trips, mirroring the partition scan's substring semantics).
fn production_secret_lines(src: &str, boundary: usize) -> Vec<usize> {
    src.lines()
        .enumerate()
        .take(boundary)
        .filter(|(_, l)| SECRET_PATTERNS.iter().any(|p| l.contains(p)))
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn confinement_helpers_flag_production_secret_above_cfg_test() {
    let synthetic = "fn prod() { let k = Zeroizing::new([0u8;32]); }\n#[cfg(test)]\nmod t { fn s() { SecretString::new(x); } }\n";
    let b = first_cfg_test_line(synthetic).expect("has #[cfg(test)]");
    assert_eq!(production_secret_lines(synthetic, b), vec![0]); // the prod Zeroizing::new line
                                                                // and a clean (test-confined-only) source yields no production lines:
    let clean = "fn prod() {}\n#[cfg(test)]\nmod t { fn s() { SecretString::new(x); } }\n";
    let cb = first_cfg_test_line(clean).unwrap();
    assert!(production_secret_lines(clean, cb).is_empty());
}

#[test]
fn every_secret_bearing_src_file_is_declared_or_allowlisted() {
    let root = crate_root();
    let mut all_rs = Vec::new();
    collect_rs_files(&root.join("src"), root, &mut all_rs);

    let declared: std::collections::HashSet<&str> =
        ZEROIZE_ROWS.iter().map(|r| r.source_file).collect();
    let allowlisted: std::collections::HashSet<&str> = NON_ROW_SECRET_FILES
        .iter()
        .chain(TEST_ONLY_SECRET_FILES.iter())
        .copied()
        .collect();

    let mut secret_files: Vec<String> = Vec::new();
    let mut undeclared: Vec<String> = Vec::new();
    for rel in &all_rs {
        if !file_is_secret_bearing(&root.join(rel)) {
            continue;
        }
        secret_files.push(rel.clone());
        if !declared.contains(rel.as_str()) && !allowlisted.contains(rel.as_str()) {
            undeclared.push(rel.clone());
        }
    }

    assert!(
        undeclared.is_empty(),
        "zeroize-completeness lint: {} secret-bearing src file(s) are neither a \
         ZEROIZE_ROWS.source_file nor in NON_ROW_SECRET_FILES — add a canonical row \
         (preferred) or an audited allowlist entry:\n  {}",
        undeclared.len(),
        undeclared.join("\n  "),
    );

    // Persistent glob-cardinality floor: a broken walk that enumerates
    // nothing would pass `undeclared.is_empty()` vacuously. This catches it.
    assert!(
        secret_files.len() >= SECRET_FILE_FLOOR,
        "zeroize-completeness lint: glob found only {} secret-bearing file(s), \
         expected >= {} (the partition floor @ 438de94). A drop means a \
         secret-bearing file was deleted (update the floor deliberately) OR the \
         glob/path-prefix broke (fix it).",
        secret_files.len(),
        SECRET_FILE_FLOOR,
    );
}

#[test]
fn non_row_secret_allowlist_is_non_empty_and_each_entry_still_bears_a_secret() {
    // Deliberate source-level tripwire: emptying the const flips this to a
    // hard FAIL, forcing a conscious decision rather than a silent dissolve of
    // the audited exemptions. `const_is_empty` allowed because the constness is
    // exactly what makes this a compile-aware guard.
    #[allow(clippy::const_is_empty)]
    {
        assert!(
            !NON_ROW_SECRET_FILES.is_empty(),
            "NON_ROW_SECRET_FILES must not be empty — the audited crypto-internal / \
             pass-through / primitive exemptions belong here"
        );
        assert!(
            !TEST_ONLY_SECRET_FILES.is_empty(),
            "TEST_ONLY_SECRET_FILES must not be empty — the cfg(test)-confined fixture \
             exemptions belong here (emptying it silently dissolves the confinement tier)"
        );
    }
    let root = crate_root();
    let mut stale: Vec<&str> = Vec::new();
    for entry in NON_ROW_SECRET_FILES
        .iter()
        .chain(TEST_ONLY_SECRET_FILES.iter())
    {
        let path = root.join(entry);
        assert!(
            path.exists(),
            "allowlist entry {entry} does not exist — remove the stale entry"
        );
        if !file_is_secret_bearing(&path) {
            stale.push(entry);
        }
    }
    assert!(
        stale.is_empty(),
        "allowlist entries no longer contain a secret pattern (remove \
         the stale allowlist entry):\n  {}",
        stale.join("\n  "),
    );
}

#[test]
fn test_only_secret_files_confine_secret_patterns_to_cfg_test() {
    let root = crate_root();
    for entry in TEST_ONLY_SECRET_FILES {
        let src =
            fs::read_to_string(root.join(entry)).unwrap_or_else(|e| panic!("read {entry}: {e}"));
        let boundary = first_cfg_test_line(&src).unwrap_or_else(|| {
            panic!("{entry} in TEST_ONLY_SECRET_FILES has no #[cfg(test)] marker")
        });
        let prod = production_secret_lines(&src, boundary);
        assert!(
            prod.is_empty(),
            "{entry}: secret pattern(s) at production line(s) {prod:?} (above #[cfg(test)] line {boundary}); \
             a TEST_ONLY exemption requires all secret patterns be test-scoped — move it to a canonical \
             ZEROIZE_ROWS row or NON_ROW_SECRET_FILES"
        );
    }
}
