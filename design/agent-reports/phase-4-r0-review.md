# Phase 4 R0 review — wallet-import v0.26.0

**Date:** 2026-05-18
**Reviewer:** opus architect
**Commit under review:** `120e6b4` (`phase 4: round-trip discipline — canonicalize_bsms + canonicalize_bitcoin_core + similar dep + 50 cells`)
**Worktree:** `.claude/worktrees/wallet-import-export-multiformat-brainstorm`

**Verdict:** GREEN — 0 Critical, 2 Important, 4 Minor. Phase 4 helpers are structurally correct and the `similar`-based unified-diff matches the SPEC §7.4 sample byte-shape (`--- input\n+++ output\n` verified against `mitsuhiko/similar` source). Idempotency is exercised; both Core envelope shapes (object + bare-array) are covered. The two Important items are scope/fixture-corpus gaps that materially affect Phase 5/6 coverage planning, not correctness bugs in the helpers themselves.

## Critical

None.

## Important

### I1 — Fixture corpus shipped at ~50% of plan budget; gaps land squarely on Phase 5/6 coverage paths

**Site:** `crates/mnemonic-toolkit/tests/fixtures/wallet_import/` (13 fixtures total: 8 BSMS + 5 Core including the pre-existing `bsms_2line_decaying_multisig_32768.txt` + `core-multi-bip84.json`).

Plan `IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md:410-425` enumerates 15 BSMS fixtures; commit ships 8. Plan `:427` enumerates 12-15 Core fixtures (matching SPEC §10.2); commit ships 5. Missing fixtures with material downstream impact:

- **BSMS:** decay-4032 / decay-32768 (only 144 + the pre-existing 32768); sortedmulti-3of5; 6-line sortedmulti-2of3; mainnet+ypub; mainnet+zpub; taproot `tr(NUMS,...)`.
- **Core:** BIP-44 P2PKH; BIP-86 P2TR; wsh-sortedmulti 3-of-5; multipath `<0;1>/*` Core fixture (the existing `core-multi-bip84.json` uses split `/0/*` + `/1/*` form, NOT the multipath shape SPEC §10.2 explicitly requires); `active: false` mix is present in `core-multi-bip84.json` but not explicitly named as such in test cells.

Per SPEC §10.1 + the round-trip arithmetic of "12-15 inputs × 2 directions = 24-30 round-trip cells per format" (`SPEC §10.1:452`), the cell count is way under plan budget too: 50 total cells (31 unit + 19 integration) vs the planned ~30 cells in `cli_import_wallet_roundtrip.rs` alone. The BSMS bundle round-trip side (§4.5) is justifiably blocked on the missing emitter, but the §4.7 semantic-blob direction is NOT blocked and was implemented at well under target.

**Risk:** Phase 5's sniff cells + Phase 6's GUI integration depend on the fixture corpus being broad enough to surface tighten-heuristic conflicts (SPEC §6.1 second clause). Specifically, the SPEC §10.2 mainnet+ypub / mainnet+zpub fixtures exercise the SLIP-132 → `xpub` neutralization path in `slip0132.rs::normalize_xpub_prefix`; without them the canonicalize-side BIP-380 re-checksum path on non-`xpub` prefixes is untested.

**Fix:** Either (a) ship the missing fixtures inline before Phase 5 R0, or (b) explicitly file a FOLLOWUP `wallet-import-fixture-corpus-expansion` AND amend the SPEC §10.1/§10.2 to lock the v0.26.0-shipped subset, then accept the reduced corpus as final for this cycle.

**SPEC reference:** SPEC §10.1, §10.2 (corpus). Plan §4.3, §4.4 (fixture list).

### I2 — `canonicalize_bsms` and `canonicalize_bitcoin_core` are `pub(crate)` + `#[allow(dead_code)]` but no Phase 5 wiring contract pins their consumers

**Site:** `crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs:39,117,211,229` — all four exported items carry `#[allow(dead_code)]` with comment "Phase 5 wires this into `cmd/import_wallet.rs::run`."

The risk is well-known per `[[feedback-build-rs-stub-fallback-security-audit]]`: `#[allow(dead_code)]` silences the compiler-side gate that would otherwise catch a Phase 5 reviewer forgetting to wire these helpers in. Three of four helpers (`canonicalize_bsms`, `canonicalize_bitcoin_core`, `unified_diff`) are tested via 19 in-module unit cells + 18 integration cells using the CLI surface, so they're well-covered against regression. BUT the integration tests at `tests/cli_import_wallet_roundtrip.rs:50-65` invoke the binary's `import-wallet` subcommand whose `--json` envelope path (per SPEC §7.4) MUST consume these helpers — if Phase 5 wires `--json` without invoking `canonicalize_*` for the `semantic_match` field, the entire round-trip discipline is silently bypassed and no test catches it.

**Fix:** Add a Phase 5 R0 gate item: "Verify `cmd/import_wallet.rs::run` invokes `canonicalize_bsms`/`canonicalize_bitcoin_core` for the `--json` `roundtrip.semantic_match` field; verify `unified_diff` is invoked for the `roundtrip.diff` field." Track as a per-phase R0 check, NOT a FOLLOWUP. The `#[allow(dead_code)]` should be removed in the same commit that wires the helpers; Phase 5 R0 should grep that the attributes are gone.

**SPEC reference:** SPEC §7.4 (the `roundtrip` envelope field carries the semantic_match contract).

## Minor

### M1 — Comment "Audit lines 0/1/3/4/5" is wrong; line 0 is the header, not an audit line

**Site:** `crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs:80-81`.

The comment reads "Audit lines 0/1/3/4/5 (i.e., token + path + first-address + signature) are dropped per step 4." Line 0 is the `BSMS 1.0` header, which is NOT dropped — it's re-emitted at step 5. The actual dropped indices are 1/3/4/5. Cosmetic.

**Fix:** `Audit lines 1/3/4/5 (i.e., token + path + first-address + signature) are dropped per step 4.`

### M2 — `canonicalize_bsms` comment claims "Empty lines in the middle of the blob are a parse error caught earlier" but no earlier check exists

**Site:** `crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs:56-58`.

The comment "Empty lines in the middle of the blob are a parse error caught earlier" is misleading — there's no earlier check; empty middle lines just produce a non-2/non-6 length count and fall through to the "expected 2 or 6 lines" error at `:85-89`. The behavior is fine, just the comment is inaccurate.

**Fix:** Rephrase to "Empty lines in the middle of the blob will cause the 2/6 line-count match below to fail."

### M3 — `bsms-2line-multi-2of2.txt` shipped but plan §4.3 line 418 listed `bsms-2line-multi-2of3.txt`

**Site:** `crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-2line-multi-2of2.txt`. Plan `:418` calls for `bsms-2line-multi-2of3.txt`.

The shipped fixture is 2-of-2, not 2-of-3 (plan-doc divergence). Test cell `fixture_bsms_2line_multi_2of2_parses_clean` at `tests/cli_import_wallet_roundtrip.rs:99-108` uses `sh(multi(2,...))` 2-of-2; semantically distinct from the planned 2-of-3 `multi` case. The "declaration-order preserved (NOT lexsorted)" assertion (plan §2.12 / §4.3 spirit) is NOT verified by any cell — the only `multi` fixture has 2 cosigners where order-preservation vs lex-sort produces the same output.

**Fix:** Add a 2-of-3 or 2-of-N `multi(...)` fixture where the declaration order differs from lex order.

### M4 — SPEC §4.1 6-line shape diverges from BIP-129's actual Round-2 plaintext (4-line) format

**Site:** SPEC `design/SPEC_wallet_import_v0_26_0.md:140-147`; `bsms.rs:104-125`; `roundtrip.rs:82-90`.

Per BIP-129, the Round-2 wallet descriptor record plaintext is 4 lines (version + descriptor + derivation_path + first_address); the MAC/signature is OUTSIDE the plaintext (it's the encryption envelope). The toolkit's SPEC §4.1 invents a 6-line shape (version + token + descriptor + path + first_address + signature). The implementation matches the SPEC; what's at issue is the SPEC's deviation from BIP-129.

This is not a v0.26.0 blocker (we're explicitly building a lenient wallet importer, not a strict BIP-129 verifier), but it WILL surface in any future signature-verification work (FOLLOWUP `bsms-verify-signatures`).

**Fix:** Add a note to SPEC §4.1 lines 140-147 that the 6-line shape is a toolkit-specific lenient input shape consolidating the BIP-129 plaintext + envelope MAC fields into a single flat blob; cite this is for FOLLOWUP `bsms-verify-signatures` planning.

## Out of scope / observations

- **`similar = "2"` license + version pin.** Apache-2.0 OR MIT, resolved to 2.7.0. No concerns.
- **Cargo.lock additions** clean.
- **`recanonicalize_descriptor`'s `rsplit_once('#')` is sound** because BIP-380 reserves `#` exclusively as the checksum separator.
- **`similar`'s `header("input", "output")`** emits `--- input\n+++ output\n` byte-exact per `mitsuhiko/similar/src/udiff.rs:349-350` — matches SPEC §7.4 sample.
- **`canonicalize_bsms` idempotency** explicitly exercised at `roundtrip.rs:469-478`.
- **Both Core envelope shapes covered** at `roundtrip.rs:372-388`.
- **Audit-line drop discipline (4 prefixes).** The BSMS canonicalize uses POSITIONAL line-index dropping (lines 1/3/4/5 in the 6-line case), not prefix-token matching. This is correct for the v0.26.0 lenient parser — the audit fields are positional in the SPEC §4.1 shape.
- **Phase 4 deferrals.** `--json` envelope `roundtrip` cell deferred to Phase 5 is appropriate; BSMS bundle round-trip deferred per missing emitter is correctly tracked via the FOLLOWUP `wallet-export-bsms-emitter` to be filed at cycle close.

## Cell-coverage assessment

| Plan §4.X | Coverage in commit `120e6b4` | Verdict |
|---|---|---|
| §4.1 canonicalize_bsms + canonicalize_bitcoin_core impl | `roundtrip.rs:40-204` | OK |
| §4.2 unified_diff via `similar` | `roundtrip.rs:212-217` | OK |
| §4.3 BSMS fixtures (12-15) | 8 shipped | UNDER (I1) |
| §4.4 Core fixtures (12-15) | 5 shipped | UNDER (I1) |
| §4.5 Bundle round-trip BSMS | structurally blocked (no emitter) | DEFERRED; FOLLOWUP at cycle close |
| §4.6 Bundle round-trip Core | 5 cells in `tests/cli_import_wallet_roundtrip.rs:337-418` | OK |
| §4.7 Semantic blob round-trip BSMS | covered via canonicalize idempotency cells in `roundtrip.rs:469-573` (10 cells) | OK at reduced fixture count |
| §4.8 Semantic blob round-trip Core | covered via canonicalize cells in `roundtrip.rs:576-655` (7 cells) | OK at reduced fixture count |
| §4.9 `--json` envelope `roundtrip` cell | deferred to Phase 5 | DEFERRED; tracked in module doc-comment lines 33-39 |

**Cell count:** 31 in-module unit + 19 CLI-integration = 50 total. Plan §4.5-§4.9 target ~24-30 cells.

## Verdict reasoning

- **No correctness bugs in the helpers themselves.** BIP-380 checksum recompute via `MsDescriptor::from_str` → `to_string()` → `ChecksumEngine` is structurally correct. JSON envelope shape handling covers both object + bare-array forms. Alphabetic key sort via `BTreeMap` is the standard sound pattern. CRLF/trailing-whitespace normalization matches SPEC §7.3.1 steps 1-2.
- **No SPEC violations.** The implementation aligns with SPEC §7.3.1 / §7.3.2 / §7.4 byte-for-byte.
- **Two Important items** (I1 fixture gap + I2 Phase-5-wiring contract) are forward-looking risks, not correctness bugs in the code under review.
- **Four Minor items** are cleanups + a SPEC-level acknowledgment.

Recommendation: fold I1 (ship the missing fixtures) + I2 (Phase 5 R0 brief addition) + M1/M2/M4 inline; M3 folded as part of I1's expansion. Then GREEN → proceed to Phase 5.
