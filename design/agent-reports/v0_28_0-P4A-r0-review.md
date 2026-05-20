# v0.28.0 Phase P4A — architect R0 self-review (GREEN)

**Sub-phase:** P4A — `wallet_import/coldcard_multisig.rs` skeleton + sniff.

**Plan-doc anchors:** §S.4 + table-row P4A (plan-doc line 515).

**SPEC anchor:** SPEC_wallet_import_v0_28_0.md §11.4 + §6.2 + §12.

**Branch:** `v0.28.0/p4-coldcard-multisig` off `release/v0.28.0` @ `71592bc`.

**Source SHA at R0 author time:** `71592bc` (release/v0.28.0).

---

## Scope verification (P4A boundary)

P4A's per-plan-doc scope:
1. `wallet_import/coldcard_multisig.rs` skeleton (NEW file). ✓ Created with `ColdcardMultisigParser` struct, `ColdcardMultisigSourceMetadata` struct (with `xfp_was_blob_supplied` + `xfp_header_disagreed` telemetry fields per SPEC §11.4), `PolicyKOfN` struct, `ColdcardMsFormat` enum, and `WalletFormatParser` impl with real `sniff()` + skeleton `parse()` returning `Err(ImportWalletParse(... lands in P4B))`.
2. Sniff: text-shape (NOT JSON) requiring `Name:` + `Policy:` + `Format:` line-prefixes in first ~20 lines. ✓
3. `SniffOutcome::ColdcardMultisig` variant insertion — already pre-stubbed at P0B.1; P4A flips the `let coldcard_multisig = false` placeholder at `sniff.rs:78` to `ColdcardMultisigParser::sniff(blob)`. ✓
4. `ColdcardMultisigSourceMetadata` struct fields per SPEC §11.4: `name`, `policy`, `script_format`, `xfp_was_blob_supplied`, `xfp_header_disagreed`, `dropped_fields`. ✓
5. `ImportProvenance::ColdcardMultisig(...)` variant inserted alphabetically in `wallet_import/mod.rs:63-77` (after `Bsms`, before any future `Coldcard` variant which lands at P3B). ✓ Exhaustive `match` blocks in `bsms_audit()` + `source_metadata()` accessors extended with `ColdcardMultisig => None` arms.
6. Sniff unit tests (positive: shared-derivation shape, per-cosigner shape, CRLF, XFP-header, leading-comments; negative: missing Name/Policy/Format, BSMS blob, Core JSON blob, empty blob, non-UTF-8, random text). 14 sniff cells + 2 `line_key` helper cells = 16 unit tests. ✓

Out-of-scope (deferred to P4B/P4C): real `parse()` body, descriptor synthesis, xfp policy 5-row truth table, `canonicalize_coldcard_multisig` real body, dispatch arm flip at `cmd/import_wallet.rs`, fixture files, integration tests.

## SPEC fidelity

- SPEC §11.4 sniff signature: text format (NOT JSON), requires `Name:` + `Policy:` + `Format:` headers. ✓ Implementation matches; XFP header tolerated as optional (per "Some firmware variants prefix a `XFP: <hex>` header line; sniff tolerates both").
- SPEC §11.4 provenance struct field-set: `name`, `policy`, `script_format`, `xfp_was_blob_supplied`, `xfp_header_disagreed`, `dropped_fields`. ✓
- SPEC §11.4 `PolicyKOfN { k: u8, n: u8 }`. ✓
- SPEC §11.4 `ColdcardMsFormat { P2wsh, P2shP2wsh, P2sh }`. ✓ (Note: alphabetical-within-enum not enforced for `ColdcardMsFormat` because variant order is documentary; `P2wsh` first matches SPEC §11.4's enumeration order.)
- SPEC §6.2 `SniffOutcome::ColdcardMultisig` enum slot pre-existed from P0B.1; P4A only flips the bool wiring (no enum touched).
- SPEC §12 module layout entry `crates/mnemonic-toolkit/src/wallet_import/coldcard_multisig.rs (Phase P4)`. ✓

## Plan-doc fidelity

- §S.4 emit-reference cite: `wallet_export/coldcard.rs:254 emit_coldcard_multisig_text`. Grep-verified at SHA `71592bc`: ✓ (`pub(crate) fn emit_coldcard_multisig_text(inputs: &EmitInputs) -> Result<String, ToolkitError>` at line 254).
- §S.4 firmware-variance note: "Some firmware variants add a leading `XFP: <hex>\n` line" — sniff `sniff_true_with_xfp_header` cell exercises this. ✓
- §S.4 integration-assertion list (P4B/P4C scope): NOT touched at P4A. ✓ (deferred per scope-boundary discipline).
- Plan-doc Wave-1 instance D: branch `v0.28.0/p4-coldcard-multisig`, files `wallet_import/coldcard_multisig.rs`, tests, fixtures. ✓ Branch name matches.

## Cross-instance dependency

- Instance E (Jade, P5) DEPENDS on D's P4B publishing `coldcard_multisig::parse_text` + `ColdcardMultisigSourceMetadata` (plan-doc R3-C1 fold). At P4A, `ColdcardMultisigSourceMetadata` is published as `pub(crate)` — Jade can already `use super::coldcard_multisig::ColdcardMultisigSourceMetadata` once P4A merges. The `parse_text` helper itself is P4B scope, so E's P5B remains hard-blocked until P4B merges. The plan-doc Wave-1 cross-instance dependency note is preserved.

## Architectural decisions made within P4A scope

- **`#[allow(dead_code)]` on telemetry fields + `ColdcardMsFormat` variants:** P4A introduces typed metadata fields that the parser body (P4B) will populate, but no caller constructs them yet. CI runs `cargo clippy --all-targets -- -D warnings`; without the allow, the build breaks. Mirrors `wallet_import/json_envelope.rs:60`'s `#[allow(dead_code)]` precedent. Each allow is annotated with the P4B lift-removal note.
- **`#[allow(dead_code)]` on `ImportProvenance::ColdcardMultisig`:** P4C is the variant constructor; until then, dead-code warns. Same pattern.
- **`line_key()` helper marked `pub(super)`:** P4B's parser body needs the same helper for line classification. Exposing as `pub(super)` (not `pub(crate)`) confines visibility to the `wallet_import` module per existing convention (`bitcoin_core::extract_threshold` precedent at `wallet_import/bitcoin_core.rs:527`).
- **Sniff scans first 20 lines, not entire blob:** the header block is fixed ≤ ~5 lines + optional comments + XFP/Derivation. 20 is well above any plausible header-block size; bounding sniff scan keeps it O(1) for large blobs (a hostile attacker can't make sniff slow by appending kilobytes of garbage).

## Verification

- `cargo build` — clean. ✓
- `cargo clippy --lib --tests --all-targets -- -D warnings` — clean (no warnings). ✓
- `cargo test --bin mnemonic wallet_import::` — 106/106 unit tests pass (16 new P4A cells + 90 pre-existing). ✓
- `cargo test --test cli_import_wallet_p0c_dispatch` — 10/10 P0C dispatch tests pass (P4A sniff wiring does not disturb the dispatch surface; the `unimplemented!()` arms still fire for `--format coldcard-multisig` explicit invocations). ✓
- Full `cargo test` workspace run — all suites pass with no regressions; pre-existing G6/MNEMONIC_FORCE_TTY ignored suites unchanged.

## Findings

**Critical:** NONE.
**Important:** NONE.
**Minor:** NONE worth blocking on.

## Overall R0 verdict

**GREEN.** P4A scope satisfied; SPEC §11.4 sniff contract honored; alphabetical-discipline + clippy-clean preserved; no cross-instance regressions; cross-instance dependency hand-off to P5 (Jade) is set up correctly (E's P5A may now begin in parallel; E's P5B is blocked until P4B merges, per plan-doc Wave-1 dependency).

Recommendation: commit P4A, push branch, proceed to P4B in the same worktree.
