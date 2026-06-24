//! v0.9.0 Cycle A Phase 2 — ms-codec Zeroizing-wrapper discipline lint.
//!
//! Companion to the mnemonic-toolkit `lint_zeroize_discipline.rs` lint
//! (toolkit branch `v0_9_0-phase-2-zeroize`). Authoritative reference:
//! `mnemonic-toolkit/design/SPEC_secret_memory_hygiene_v0_9_0.md` §1
//! item 2 + survey §1 ms-codec table (4 OWNED rows).
//!
//! For each enumerated OWNED-secret site in ms-codec's encoder /
//! decoder spines, this lint asserts the implementing source file
//! contains a stable `Zeroizing` evidence anchor — proving the row's
//! `Vec<u8>` allocation is wrapped (internal-only, so the public
//! `Payload::Entr(Vec<u8>)` shape is preserved per SPEC §3 OOS-2 and
//! the v0.1.3 patch-tag semver compatibility plan).
//!
//! Public `Payload::Entr(Vec<u8>)` shape is intentionally unwrapped:
//! widening the public type to `Payload::Entr(Zeroizing<Vec<u8>>)` is a
//! breaking change deferred indefinitely. Callers are responsible for
//! wrapping the returned `Vec<u8>` at their use site (mnemonic-toolkit
//! does this; the contract is documented in `payload.rs` doc-comment).
//!
//! RED on Phase 2 first commit: no source uses `Zeroizing` yet.

use std::fs;
use std::path::Path;

struct ZeroizeRow {
    label: &'static str,
    source_file: &'static str,
    evidence: &'static [&'static str],
}

/// Canonical 4-row list: 3 v0.1 survey §1 ms-codec rows + 1 v0.2 K-of-N
/// shares.rs coverage row.
/// Per-row evidence anchors tightened post R1 I-4 fold so each row enforces
/// its specific call-site discipline (not just any Zeroizing reference in
/// the file).
///
/// cycle-15 Lane M (slug #2): the old `decode()` "scrubbed before public emit"
/// row was THEATER — it anchored on a `let scrubbed: Zeroizing<Vec<u8>>` that
/// only scrubbed an already-moved-from buffer while a deref-clone allocated a
/// fresh un-scrubbed copy as the live `Payload`. That row is dropped (4 rows
/// now); the honest invariant — the clone is GONE — is enforced by the
/// dedicated negative-anchor test `decode_has_no_clone_into_bare_vec`.
const ZEROIZE_ROWS: &[ZeroizeRow] = &[
    ZeroizeRow {
        label: "envelope::discriminate() wraps OWNED payload Vec",
        source_file: "src/envelope.rs",
        evidence: &["payload_with_prefix: Zeroizing<Vec<u8>>"],
    },
    ZeroizeRow {
        label: "envelope::package() wraps OWNED data Vec",
        source_file: "src/envelope.rs",
        evidence: &["let data: Zeroizing<Vec<u8>>"],
    },
    ZeroizeRow {
        label: "payload.rs documents caller-wrap contract",
        source_file: "src/payload.rs",
        evidence: &["Caller-wrap contract", "must wrap"],
    },
    // v0.2 K-of-N (SPEC_ms_v0_2_kofn §2): shares.rs wraps the OWNED secret
    // material it handles — the CSPRNG defining-share payload (`encode_shares`)
    // and the recovered secret-at-S bytes (`combine_shares`). Coverage row
    // (these are already Zeroizing-wrapped; this anchors them against regression).
    ZeroizeRow {
        label: "shares::{encode_shares,combine_shares} wrap OWNED secret Vecs",
        source_file: "src/shares.rs",
        evidence: &[
            "let mut filler: Zeroizing<Vec<u8>>",
            "let data: Zeroizing<Vec<u8>> = Zeroizing::new(secret.parts().data())",
        ],
    },
    // Cycle-B: the vendored `codex32::Codex32String` scrubs its inner secret
    // String on drop (`zeroize::ZeroizeOnDrop`) + redacting Debug. This is what
    // makes the share-spine `Codex32String`/`Vec<Codex32String>` bindings in
    // shares.rs (`secret_s`, `defining`, `parsed`, the recovered `secret`)
    // auto-drop-scrub WITHOUT a `Zeroizing` wrapper (they own their scrub). The
    // anchor enforces both the derive AND the redacting Debug stay present.
    ZeroizeRow {
        label: "codex32::Codex32String scrubs its inner String on drop (vendored, Cycle-B)",
        source_file: "src/codex32/mod.rs",
        evidence: &[
            "zeroize::ZeroizeOnDrop",
            "impl fmt::Debug for Codex32String",
        ],
    },
];

fn crate_root() -> &'static Path {
    Path::new(".")
}

#[test]
fn canonical_list_has_expected_row_count() {
    let n = ZEROIZE_ROWS.len();
    assert_eq!(
        n, 5,
        "ZEROIZE_ROWS row count = {n}; expected 5 (3 v0.1 survey §1 rows + 1 v0.2 K-of-N shares.rs row + 1 vendored-codex32 Codex32String drop-scrub row, Cycle-B; the theater decode row was dropped in cycle-15 Lane M slug #2)."
    );
}

/// cycle-15 Lane M (slug #2) — NEGATIVE anchor. The old decode-path scrub was
/// theater: a deref-clone copied a fresh un-scrubbed `Vec` out of a throwaway
/// `Zeroizing` envelope and made THAT the live `Payload`. The fix moves the
/// bytes straight into `Payload`, so both the clone and the throwaway envelope
/// must be GONE.
#[test]
fn decode_has_no_clone_into_bare_vec() {
    let src = fs::read_to_string(crate_root().join("src/decode.rs")).expect("read src/decode.rs");
    assert!(
        !src.contains("(*scrubbed).clone()"),
        "decode.rs still contains the theater `(*scrubbed).clone()` — the slug-#2 \
         move-into-Payload fix is missing or regressed."
    );
    assert!(
        !src.contains("let scrubbed: Zeroizing<Vec<u8>>"),
        "decode.rs still binds the throwaway `scrubbed` Zeroizing envelope — the \
         slug-#2 fix removed it; the bytes move straight into `Payload`."
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
        "ms-codec zeroize-discipline lint: {} row(s) missing Zeroizing evidence:\n{}",
        missing.len(),
        missing.join("\n"),
    );
}
