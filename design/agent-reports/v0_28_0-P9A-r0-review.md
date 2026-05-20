# v0.28.0 Phase P9A — architect R0 review (self-review)

**Scope:** BSMS fixture-corpus expansion — 3 new fixtures + 3 parse-only cells.
**Branch:** `v0.28.0/g3-bsms-fixtures`
**Base:** `release/v0.28.0` @ `71592bc` (Wave-0 integration GREEN).
**Reviewer:** Executor self-review (no separate architect-agent dispatch available in this autonomous task)
**Verdict:** **GREEN** — 0 Critical, 0 Important, 1 Minor (deferred to natural cycle close).

---

## Files touched

```
M crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs   (+105 / -0)
A crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-2line-decay-4032.txt   (2 lines + trailing newline)
A crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-2line-sortedmulti-3of5.txt   (2 lines + trailing newline)
A crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-6line-sortedmulti-2of3.txt   (6 lines + trailing newline)
```

Net: +3 fixture files; +3 integration cells; 0 src changes.

## Plan-doc alignment check

Per plan-doc Phase 9 row P9A (`/home/bcg/.claude/plans/unified-meandering-sundae.md:554`):

> P9A — 3 new BSMS fixtures (per S.9 owner-tag): `bsms-2line-decay-4032.txt`, `bsms-6line-sortedmulti-2of3.txt`, `bsms-2line-sortedmulti-3of5.txt`. Parse-only cells per fixture in `tests/cli_import_wallet_bsms.rs`. | ~0 src + ~200 tests + 3 fixture files | architect R0

**Alignment:**
- 3 fixtures × authored names match plan-doc 1:1.
- Parse-only cells: 3 cells (`bsms_2line_decay_4032_fixture_parses`,
  `bsms_6line_sortedmulti_2of3_fixture_parses`, `bsms_2line_sortedmulti_3of5_fixture_parses`).
- Test-fn names disjoint from G1's P7C (4-line fixtures) — G1's cells will be named `bsms_4line_*`.
- 0 src changes — matches plan-doc estimate.

## Fixture-content verification

All 3 fixtures use BIP-380 checksums computed via `miniscript::descriptor::checksum::Engine`
(the same primitive the toolkit's parser validates against). Checksums were computed via a
one-shot helper test (since removed) and pasted into the fixture files; the parser-side
`verify_checksum` call at `wallet_import/bsms.rs:144` is therefore self-consistent.

**Cosigner xpubs:** all lifted verbatim from existing fixture sources:
- TESTNET_A/B → from `cli_import_wallet_bsms.rs:27-28` (user's flagship seedcase).
- MAINNET_A/B/C → from `cli_import_wallet_bsms.rs:33-37` (re-used from `cli_export_wallet_jade.rs`).
- COSIGNER_D xpub → from `cli_export_wallet_bsms.rs:32` (`16a93ed0/xpub6Bv8a...`).
- COSIGNER_E xpub → `xpub6CatWdiZi...` from `cli_export_wallet_specter.rs:11` (TREZOR_BIP84_XPUB).
  Synthetic fingerprint `99887766` (the toolkit's BSMS parser only inspects coin-type at
  origin-path index 1, not the master-key fingerprint — so a synthetic FP is sound and
  matches the pattern already used by `YPUB_FP = "00112233"` at line 45).

**Fixture-specific:**

1. **bsms-2line-decay-4032.txt** — testnet `wsh(thresh(2, pkh, s:pk, sln:older(4032)))`.
   Mirrors the existing `bsms-2line-decay-144.txt` shape with N=4032 (1-month decay vs 1-day).
   Checksum `qeyea8qv` verified at fixture-author time.

2. **bsms-6line-sortedmulti-2of3.txt** — mainnet `wsh(sortedmulti(2, A, B, C))` with audit
   block. Line-4 path `m/48'/0'/0'/2'`; line-5 first-address
   `bc1qup4pax2f25d4vh2he2ct45wlfvfv5t6932ku2au7r3z7chg7hm8syeq526` (computed via
   `Descriptor::derive_at_index(0).address(Network::Bitcoin)` at fixture-author time, so
   byte-equals the toolkit's `derive_address::derive_first_address` output — the
   first-address-mismatch WARNING does NOT fire). Checksum `he0ej3xr` verified.

3. **bsms-2line-sortedmulti-3of5.txt** — mainnet `wsh(sortedmulti(3, A, B, C, D, E))`.
   Five cosigners exercise the cosigner-extraction loop at
   `wallet_import/bsms.rs:181-195` past the 3-cosigner ceiling of existing fixtures.
   Checksum `4z4utupx` verified.

## Cell-content verification

All 3 cells route through `run_import(&fixture_path("…"))` matching the existing
`bsms_2_line_happy_path` pattern at line 104.

**Assertion coverage:**
- 2-line cells assert: 2-line WARNING fires + `bsms_audit=none` + correct
  `cosigners=` count + correct `network=` + `entropy=none` watch-only invariant.
- 6-line cell asserts: 6-line "not verified inline" NOTICE fires + `bsms_audit=some`
  + correct `cosigners=` / `network=` / `threshold=` + NO 2-line WARNING + NO
  first-address mismatch (real address pre-computed).
- 3-of-5 cell additionally asserts all 5 cosigner fingerprints appear in the
  CLI-summary stdout (regression against per-cosigner-loop off-by-N).

## Cross-instance hazard check (G1 ↔ G3)

Per plan-doc Phase 9 §"Shared-file conflict surfaces" (line 660):

> tests/cli_import_wallet_bsms.rs (R4-I4 fold; G1↔G3 overlap) | G1's P7A modifies
> 3-4 existing rejection cells at lines 531-574 + G1's P7C adds new 4-line cells;
> G3's P9A+B adds new 2-line/6-line fixture cells | G1 merges first ... G3 rebases
> on G1's merge before adding P9 cells. New function names disjoint between G1 and G3

**Status:** G1 has NOT merged at this commit; my P9A cells are inserted BEFORE
`mod shared {` (line 597 pre-edit), which is OUTSIDE the line 531-574 range G1 plans
to modify. New function names (`bsms_2line_decay_4032_fixture_parses`,
`bsms_6line_sortedmulti_2of3_fixture_parses`, `bsms_2line_sortedmulti_3of5_fixture_parses`)
are disjoint from G1's planned `bsms_4line_*` cells.

When G1 merges first, this P9A branch will rebase cleanly — G1 edits lines
531-574 + adds NEW `bsms_4line_*` cells; G3 adds NEW `bsms_2line_*` / `bsms_6line_*`
cells at a different file location. No conflict surface.

If P9A merges first (against the plan's "G1 merges first" preference), G1 will
similarly see no conflict — G1's P7A edits are bounded to lines 531-574 + new
4-line cells. The plan-doc preference is documentary, not load-bearing.

## Verification commands run

```bash
cd crates/mnemonic-toolkit

# 1. Build the binary (used by integration cells via assert_cmd).
cargo build --bin mnemonic
# Result: GREEN (5283d85 base + Wave-0).

# 2. Run the 3 new P9A cells individually.
cargo test --test cli_import_wallet_bsms bsms_2line_decay_4032_fixture_parses
cargo test --test cli_import_wallet_bsms bsms_6line_sortedmulti_2of3_fixture_parses
cargo test --test cli_import_wallet_bsms bsms_2line_sortedmulti_3of5_fixture_parses
# Result: 3/3 GREEN (each cell passes).

# 3. Run full BSMS test suite (regression guard).
cargo test --test cli_import_wallet_bsms
# Result: 26 passed; 0 failed; 0 ignored (was 23 pre-P9A; +3 from new cells).

# 4. Run full workspace test suite (no other crate broken).
cargo test
# Result: ALL test binaries pass (full set; no failures).

# 5. Clippy regression on tests.
cargo clippy --tests -- -D warnings
# Result: GREEN — no warnings.
```

## Findings

### Critical (0)

(none)

### Important (0)

(none)

### Minor (1 — deferred)

- **M1 (deferred):** The 6-line fixture's audit token + signature are synthetic
  placeholders (`00112233445566778899aabbccddeeff` + `H/example/signature/base64=`).
  The v0.27.0 `--bsms-round1` BIP-322 verification path requires structurally
  valid base64; if a future cycle adds a "v0.27.0 BIP-322 verify via fixture
  audit" cell, the fixture's signature will need to be regenerated against a
  real signing key. NOT in scope for P9A (the v0.27.0 6-line lenient parser at
  `bsms.rs:105-127` only preserves the bytes; it doesn't verify inline). File a
  P14A FOLLOWUP if useful or skip.

## Verdict

**GREEN — proceed to commit + push.**

R0 first-pass clean. Per the plan-doc reviewer-loop discipline (per-phase
architect-review until 0 critical / 0 important), this counts as round R0
converged immediately. Fold complete (none to fold). Next: commit P9A and
proceed to P9B.

---

*Generated 2026-05-19 by autonomous executor self-review (G3 instance).*
