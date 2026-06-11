# SPEC — zeroize-lint source→declared completeness (promote 14 audited files to canonical ROWS + add the scan)

**Cycle:** toolkit NO-BUMP (test only) · **Source SHA:** `438de94` · **Audit:** the 19-file zeroize audit (this session) · **Recon:** `cycle-prep-recon-ci-test-hygiene-cluster.md` §Part B.
**Resolves:** `hand-frozen-lint-canons-no-completeness` (the deferred Cycle-B Part B).

The audit classified the 19 src files that use `Zeroizing`/`SecretString` but are NOT canonical `ZEROIZE_ROWS`: **14 CANONICAL** (own real secrets → promote to rows), **4 allowlist** (3 crypto-internal + 1 pass-through), **1 primitive** (`secret_string.rs`, the newtype def). So this is a genuine COVERAGE GAIN (16 new rows), not a whitewash allowlist. No binary/wire/CLI change → no schema_mirror/GUI/sibling/version/CHANGELOG (NO-BUMP test-only).

## PART A — add the 16 canonical ZEROIZE_ROWS (the audited owned-secret sites)
Add rows to `tests/lint_zeroize_discipline.rs` ZEROIZE_ROWS for the 14 CANONICAL files (16 rows; some files own 2 distinct secrets). **Every `evidence` substring MUST be grep-verified byte-exact against current source at write time** (the lint does `source.contains(needle)`); the audit quoted them but they decay — re-confirm each.

| File (`src/`) | Owned secret(s) → row |
|---|---|
| `cmd/addresses.rs` | master-secret entropy `Zeroizing<Vec<u8>>` (`:199`) |
| `cmd/electrum_decrypt.rs` | decrypt-password `Zeroizing<String>` (`:100-116`) |
| `cmd/import_wallet.rs` | (1) wallet blob `Zeroizing<Vec<u8>>` (read_blob); (2) decrypt-password + decrypted BSMS records `Zeroizing<String>` |
| `cmd/ms_shares.rs` | (1) `parse_secret_to_entropy → Zeroizing<Vec<u8>>`; (2) combine recovered entropy + rendered output `Zeroizing` |
| `cmd/restore.rs` | seed entropy `Zeroizing<Vec<u8>>` (`run` block + `resolve_seed_entropy`) |
| `cmd/seedqr.rs` | digits + decoded/encoded phrase `Zeroizing<String>` |
| `cmd/verify_bundle.rs` | (1) Phrase arm passphrase+entropy; (2) Entropy arm hex `entropy_bytes` |
| `cmd/xpub_search/account_of_descriptor.rs` | BIP-39 passphrase `Zeroizing<String>` |
| `cmd/xpub_search/passphrase_of_xpub.rs` | mandatory passphrase `Zeroizing<String>` (`:292`) |
| `cmd/xpub_search/path_of_xpub.rs` | passphrase `Zeroizing<String>` |
| `cmd/xpub_search/seed_intake.rs` | `Source::Phrase/Ms1(Zeroizing<String>)` + decoded entropy `Zeroizing<Vec<u8>>` |
| `seed_xor.rs` (library ≠ `cmd/seed_xor.rs`) | shares + recovered master `Zeroizing<Vec<u8>>` |
| `slot_ms1.rs` | `Ms1SlotResolution.entropy: Zeroizing<Vec<u8>>` |
| `wallet_import/overlay.rs` | cosigner entropy `Zeroizing<Vec<u8>>` |

(Proposed labels + exact evidence substrings: see the audit output appended in the agent-report. R0 + impl: grep-verify each evidence; the `every_canonical_zeroize_row_has_evidence_anchor` test will catch a stale anchor.) Bump the ZEROIZE_ROWS count-range upper bound at `lint_zeroize_discipline.rs:262` (`18..=42` → `18..=60`; 36 + 16 = 52 — confirm the exact len via `cargo test`).

## PART B — the source→declared completeness scan
Add a test `every_secret_bearing_src_file_is_declared_or_allowlisted`: enumerate every `src/**/*.rs` containing an owned-secret allocation pattern (`Zeroizing::new(`, `SecretString::new(`, `: Zeroizing<`, `: SecretString`), and assert each file is EITHER a `ZEROIZE_ROWS.source_file` OR in an explicit allowlist:
```rust
// Files that USE Zeroizing/SecretString but are NOT canonical owned-secret
// rows (audited 2026-06-11). Each line: why it's exempt.
const NON_ROW_SECRET_FILES: &[&str] = &[
    "src/bsms_crypto.rs",      // CRYPTO-INTERNAL: PBKDF2 AES key + AES-CTR plaintext buffer (consumer owns the plaintext)
    "src/electrum_crypto.rs",  // CRYPTO-INTERNAL: ECIES/CBC primitive (AES key, scalar, ECDH shared secret, key block)
    "src/slip39/feistel.rs",   // CRYPTO-INTERNAL: SLIP-0039 Feistel L/R halves + round key (consumer slip39/mod.rs owns output)
    "src/nostr.rs",            // PASS-THROUGH: decode_nostr_key hands the decoded INPUT upstream; cmd/nostr.rs owns the derived secret
    "src/secret_string.rs",    // PRIMITIVE: the SecretString newtype DEFINITION, not an allocation site
];
```
After Part A, every secret-pattern src file is in ROWS-source ∪ this allowlist (the audit's full partition: 14 rows-files + 5 here = 19 + the 16 pre-existing rows-files). A NEW secret-bearing file then FAILS the lint → forces a row or an explicit allowlist decision. (R0: the allowlist is now SMALL + each entry audited — the I3 concern from the Part-A-split round is addressed: verify_bundle/ms_shares are PROMOTED to rows, not allowlisted.)

**Tests:**
- `every_secret_bearing_src_file_is_declared_or_allowlisted` passes today (full partition). **RED-proof (R0-r1 I1 — exercise the REAL glob loop, not just the predicate):** during development, temporarily remove one allowlist entry (e.g. `secret_string.rs`) → the scan must RED (proving the glob actually enumerates that file + the assertion fires) → restore. Do NOT rely on a predicate-only synthetic-path test.
- **Persistent glob-cardinality FLOOR (R0-r1 I1):** assert the glob finds at least N secret-bearing files (e.g. `>= 35`) — a permanent automated guard so a future broken glob/path-prefix change (which would make the scan vacuously pass by enumerating nothing) is caught in CI. Mirrors the existing `ZEROIZE_ROWS.len()` count-range guard for the declared direction.
- Assert the allowlist (`NON_ROW_SECRET_FILES`) is non-empty + each entry's file actually still contains a secret pattern (catches a stale allowlist entry).

## Bonus FOLLOWUP (from the audit)
File `addresses-restore-passphrase-not-zeroizing`: in `cmd/addresses.rs` + `cmd/restore.rs` the BIP-39 passphrase is held as a PLAIN `String` (only the entropy is `Zeroizing`). A real (small) secret-hygiene gap — wrap the passphrase in `Zeroizing<String>` in a future cycle.

## Ritual
NO version bump / CHANGELOG (NO-BUMP test-only). FOLLOWUPS resolve `hand-frozen-lint-canons-no-completeness` + file `addresses-restore-passphrase-not-zeroizing`. Stage paths explicitly. Mandatory R0 gate to 0C/0I; persist reviews to `design/agent-reports/`.

## Non-goals
Site-level (vs file-level) completeness; the addresses/restore passphrase wrap (bonus FOLLOWUP); zeroizing the crypto-internal intermediates further (they already wrap).
