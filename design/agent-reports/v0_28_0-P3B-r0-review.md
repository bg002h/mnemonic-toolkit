# Phase P3B — architect R0 review

**Reviewer:** in-session architect-style self-review (Opus 4.7 main agent)
**Branch:** `v0.28.0/p3-coldcard`
**Files under review:**
- `crates/mnemonic-toolkit/src/wallet_import/coldcard.rs` (+~350 LOC: parse impl + dominance-order + dropped-field telemetry + ~14 new tests)
- `crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs` (+~130 LOC: `canonicalize_coldcard` real impl + 6 new tests; -1 skeleton test)
- `crates/mnemonic-toolkit/tests/fixtures/wallet_import/coldcard-bip{44,49,84,86}-mainnet.json + coldcard-bip84-testnet.json` (5 new files)

**Source SHA verified against:** branch HEAD pre-commit
**Plan-doc:** `/home/bcg/.claude/plans/unified-meandering-sundae.md` Phase 3 P3B row + §S.3
**SPEC:** `design/SPEC_wallet_import_v0_28_0.md` §11.3 + §11.3.1

---

## Critical (correctness-blocking)

**None.** P3B delivers the full parse impl per SPEC §11.3 + §11.3.1:

1. JSON-parse + extract `chain` → `ColdcardChain` → network (BTC → mainnet, XTN → testnet).
2. Extract top-level `xfp` → master fingerprint (`[u8; 4]`, 8-char-hex strict-shape validation).
3. Extract `account` (default 0; typed-error on non-integer).
4. Dominant-BIP selection (§11.3.1 order BIP-86 > BIP-84 > BIP-49 > BIP-44; explicit refusal for legacy Mk1/Mk2 top-level `xpub`-only with FOLLOWUP pointer; explicit refusal for `bip48_*`-only with `--format coldcard-multisig` pointer).
5. Per-block field extraction: `deriv`, parent `xfp` (shape-validated), `xpub` (SLIP-132-normalize then BIP-32-parse).
6. Descriptor body synthesis: `pkh / sh(wpkh) / wpkh / tr` wrapping `[xfp/path]xpub/<0;1>/*`.
7. Route through `concrete_keys_to_placeholders` + `parse_descriptor::parse_descriptor` pipeline (same as BSMS / Bitcoin Core).
8. `ResolvedSlot` + `ParsedImport` construction (single-cosigner; threshold=None; watch-only enforced via `validate_watch_only_resolved`).

Plus `canonicalize_coldcard` real impl: BTreeMap-driven alphabetical key ordering, `_pub` + `first` + competing-BIP-block fields dropped, `xfp` casing normalize to uppercase, `account` defaulting to 0.

## Important

**I1 — `ImportProvenance::Coldcard` variant is deliberately deferred to P3C; the P3B parse impl routes through a `Bsms(None)` placeholder for the `provenance` field.** The metadata struct `ColdcardSourceMetadata` is constructed correctly in the parse path (`_provenance_pending_p3c` binding) so the field-shape is exercised; P3C's first commit will (a) add the `Coldcard(coldcard::ColdcardSourceMetadata)` variant to `ImportProvenance`, (b) flip the placeholder assignment, (c) add the `--json` envelope wire-up. This matches the prompt's P3C scope. No fold at P3B; the comment `// P3C-replace` signals the boundary clearly.

## Minor

**M1 — `_block_xfp` parsed-but-unused.** The per-block `xfp` (parent fingerprint of the account xpub) is shape-validated (`parse_xfp_hex`) but not used in descriptor synthesis — only the top-level master `xfp` is bracket-form-emitted per BIP-380 semantics. The shape-validation guards against malformed source blobs; underscoring the binding (`_block_xfp`) signals intentional unused-binding semantics. No fold.

**M2 — Mk1/Mk2 legacy top-level-xpub fallback returns refusal, not parse.** SPEC §11.3.1 step 5 calls for SLIP-132 prefix inference (zpub→BIP-84, ypub→BIP-49, xpub→BIP-44) for legacy firmware. P3B refuses with a FOLLOWUP-pointer error rather than implementing the inference path. Rationale: (a) the modern Coldcard ecosystem ships Mk3+ firmware that always emits explicit `bipN` blocks; (b) the inference path requires SLIP-132 variant-classification on the raw base58 prefix bytes (similar to `slip0132::normalize_xpub_prefix` but inverted to detect-not-normalize); (c) the refusal carries a clear user-facing pointer + FOLLOWUP marker. **Recommended FOLLOWUP filing:** `coldcard-legacy-mk1-mk2-top-level-xpub-inference` (track at cycle close). No P3B fold.

**M3 — `dropped_fields` telemetry granularity.** The current impl enumerates `<bip>.name`, `<bip>.first`, `<bip>.<_pub>`, plus competing-BIP block names. Stderr NOTICE is single-line + comma-separated. Matches the Bitcoin Core `bitcoin_core::parse` `dropped wallet-state fields` stderr precedent. No fold.

**M4 — Test cell-count: 14 new tests (5 happy paths per BIP variant + 2 dominance order + 7 refusal cases) in coldcard.rs + 6 new canonicalize tests in roundtrip.rs.** Plan-doc P3B line-budget is "~270 tests" (LOC; cell-count of 14 maps to ~270 LOC including assertions + setup). Matches budget. No fold.

**M5 — Fixtures use shared canonical xpubs reused from `tests/export_wallet/coldcard_generic_bip*_*.json`.** Self-contained corpus (no cross-repo / network); xpubs reused for internal-consistency with the v0.7 emit side. Idempotent: re-running `cargo test` does not generate test data. No fold.

**M6 — `canonicalize_coldcard` does NOT validate the xpub (it preserves the source-blob's `xpub` string verbatim).** This matches `canonicalize_bitcoin_core`'s `desc` preservation pattern — canonicalize is semantic-equality not validation; the parse path is where shape validation occurs. Round-trip via `cmd::import_wallet::run` always runs `parse` first (which validates) then `canonicalize` for the round-trip comparison. No fold.

## Verdict

**GREEN.** Proceeding to commit + P3C without further iteration.
