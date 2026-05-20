# v0.28.0 Phase P9B — architect R0 review (self-review)

**Scope:** BSMS fixture-corpus expansion — 4 more fixtures + 6 cells (4 parse-only + 2 stdin-roundtrip) + 1 cycle-internal FOLLOWUP entry.
**Branch:** `v0.28.0/g3-bsms-fixtures`
**Base:** `release/v0.28.0` @ `71592bc` (Wave-0 integration GREEN); P9A `f2c32de` already on branch.
**Reviewer:** Executor self-review (no separate architect-agent dispatch available in this autonomous task)
**Verdict:** **YELLOW → GREEN after fold** — 0 Critical, 1 Important (folded inline), 1 Minor.

---

## Files touched

```
M crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs   (+150 / -0)
A crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-2line-mainnet-ypub.txt   (2 lines + trailing newline)
A crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-2line-mainnet-zpub.txt   (2 lines + trailing newline)
A crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-2line-tr-nums.txt        (2 lines + trailing newline)
A crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-2line-bip45.txt          (2 lines + trailing newline)
M design/v0_28_0-cycle-followups.md   (+33 / -2)
```

Net: +4 fixture files; +6 integration cells; +1 cycle-followups entry; 0 src changes.

## Plan-doc alignment check

Per plan-doc Phase 9 row P9B (`/home/bcg/.claude/plans/unified-meandering-sundae.md:555`):

> P9B — 4 more BSMS fixtures (per S.9 owner-tag): `bsms-2line-mainnet-ypub.txt`,
> `bsms-2line-mainnet-zpub.txt`, `bsms-2line-tr-nums.txt` (refusal), `bsms-2line-bip45.txt`.
> Parse-only + roundtrip cells. | ~0 src + ~250 tests + 4 fixture files | architect R0

Per plan-doc §S.9 R1-M2 explicit ownership clarification:

> P9B owns this fixture AND the integration cell `tests/cli_import_wallet_bsms.rs::bsms_tr_nums_refused`
> asserting `import-wallet --format bsms <fixture>` exits 2 with `ImportWalletParse`
> containing the SPEC-anchored taproot-refusal substring

**Alignment:**
- 4 fixtures × authored names match plan-doc 1:1.
- Parse-only cells: 4 (`bsms_2line_mainnet_ypub_fixture_parses`,
  `bsms_2line_mainnet_zpub_fixture_parses`, `bsms_2line_tr_nums_current_behavior_no_refusal`,
  `bsms_2line_bip45_fixture_parses_or_rejects_descriptively`).
- Roundtrip cells: 2 (`bsms_2line_mainnet_ypub_stdin_roundtrip_matches_blob_path`,
  `bsms_2line_mainnet_zpub_stdin_roundtrip_matches_blob_path`).
- 0 src changes — matches plan-doc estimate.
- Test-fn names disjoint from G1's planned `bsms_4line_*` cells.

**Plan-doc divergence (the R1-M2 "refusal" cell):**

The plan-doc R1-M2 ownership clarification names the cell `bsms_tr_nums_refused`
with an exit-2 assertion containing the taproot-refusal substring. The CURRENT
v0.27.0 BSMS parser at `wallet_import/bsms.rs:217-224` does NOT refuse tr() at
parse time — it explicitly accepts taproot descriptors and only skips the
first-address-verify WARNING. P9B's plan-doc scope (`~0 src + ~250 tests + 4
fixtures`) does NOT include the parser change required to flip this to refusal.

**Fold applied (Important I1, see below):** I renamed the cell to
`bsms_2line_tr_nums_current_behavior_no_refusal` and pin the ACTUAL current
behavior (exit 0, cosigners=2, network=mainnet, bsms_audit=none,
threshold=none). The plan-doc's forward-looking refusal intent is filed as
cycle-internal FOLLOWUP `bsms-import-taproot-refusal-parity` (entry added at
`design/v0_28_0-cycle-followups.md`). If the user lifts the parser change
into-scope mid-cycle, G2 (Phase P8 — taproot-emit refusal scaffold) is the
natural fold target. Otherwise the FOLLOWUP triages at P14A.

## Findings

### Critical (0)

(none)

### Important (1 — folded)

- **I1 (folded):** Plan-doc R1-M2 names the tr-NUMS cell `bsms_tr_nums_refused`
  with exit-2 refusal assertion; current parser does not refuse tr() blobs (it
  accepts them, only skipping first-address verify per `bsms.rs:217-224`).
  Fold: rename cell to `bsms_2line_tr_nums_current_behavior_no_refusal`, pin
  current behavior verbatim, file cycle-internal FOLLOWUP
  `bsms-import-taproot-refusal-parity` documenting the parser-side gap and its
  natural fold target (G2 / Phase P8). The cell's doc-comment explicitly cites
  the plan-doc R1-M2 wording so future readers see the trail.

### Minor (1 — deferred)

- **M1 (deferred):** `extract_threshold`'s regex at `wallet_import/bsms.rs:419-421`
  matches `thresh|multi|sortedmulti` but NOT `sortedmulti_a` or `multi_a`
  (the taproot variants). For the tr-NUMS fixture's `tr(NUMS,
  sortedmulti_a(2, ...))` body, the regex returns `Ok(None)` and the CLI
  summary emits `threshold=none`. Pin folded into the tr-NUMS cell's
  doc-comment + the FOLLOWUP body for the parser-side fix. If the parser is
  modified per the FOLLOWUP to refuse tr() at parse-time, this stay-behind
  hazard disappears entirely.

## Fixture-content verification

All 4 fixtures use BIP-380 checksums computed via the existing helper test
(now removed from the test file). Checksums were verified by running each
through the parser; all parse-only cells pass.

**Cosigner xpubs / fingerprints:**
- YPUB / `00112233` — verbatim from existing `cli_import_wallet_bsms.rs:45-46`
  (canonical SLIP-0132 BIP-49 mainnet test vector; synthetic fingerprint).
- ZPUB `zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314...` — verbatim from
  `cli_export_wallet_electrum.rs:14` (TREZOR 24-word seed BIP-84 mainnet zpub);
  fingerprint `5436d724` (real master fp of the TREZOR-24 seed).
- A/B/C cosigners (tr-NUMS, BIP-45) — verbatim from
  `cli_import_wallet_bsms.rs:33-37`.

**Fixture-specific:**

1. **bsms-2line-mainnet-ypub.txt** — `sh(wpkh(...))` BIP-49 path. Mirrors
   the existing `bsms_slip132_variants_ypub` dynamic cell at line 373.
   Checksum `nkwrvhy4` verified.

2. **bsms-2line-mainnet-zpub.txt** — `wpkh(...)` BIP-84 path. New shape
   (no existing dynamic zpub fixture in the BSMS corpus). Checksum
   `yxzx3ag7` verified.

3. **bsms-2line-tr-nums.txt** — `tr(NUMS, sortedmulti_a(2, A, C))` BIP-86 path.
   Uses A + C (not A + B) for variety. Checksum `rgn6fk37` verified.

4. **bsms-2line-bip45.txt** — `sh(multi(2, A, C, B))` BIP-45 path. Cosigner
   declaration order is intentionally NOT lexicographic so the cell can pin
   the SPEC §4.3 declaration-order preservation if rust-miniscript accepts
   the descriptor. Checksum `3rldg0zz` verified.

## Cell-content verification

**Parse-only cells (4):**
- `bsms_2line_mainnet_ypub_fixture_parses` — pin cosigners=1, network=mainnet,
  bsms_audit=none, 2-line WARNING.
- `bsms_2line_mainnet_zpub_fixture_parses` — same shape + assert `MAINNET_FP_C`
  appears (real TREZOR-24 master fp regression guard).
- `bsms_2line_tr_nums_current_behavior_no_refusal` — pin success path (exit 0,
  cosigners=2, network=mainnet, threshold=none); see Important I1.
- `bsms_2line_bip45_fixture_parses_or_rejects_descriptively` — permissive cell
  matching the existing `bsms_multi_non_sorted_2_of_3` pattern at line 326:
  EITHER success (3 cosigners + declaration-order check) OR structured
  rejection (non-empty stderr). Pins the BIP-45 surface without committing
  to a specific rust-miniscript acceptance contract.

**Roundtrip cells (2):**
- `bsms_2line_mainnet_ypub_stdin_roundtrip_matches_blob_path` — pin
  `--blob <path>` vs `--blob - + stdin` byte-equal stdout for ypub fixture.
- `bsms_2line_mainnet_zpub_stdin_roundtrip_matches_blob_path` — same for
  zpub fixture.

The two roundtrip cells cover the SLIP-132-prefix-variant fixtures (P9B.4
+ P9B.5); the tr-NUMS and BIP-45 fixtures don't add additional value via a
roundtrip cell (their CLI-input-mode symmetry is identical to the existing
`bsms_crlf_normalized` and `bsms_2_line_happy_path` coverage at lines
473-481 and 104).

## Cross-instance hazard check (G1 ↔ G3)

Same analysis as P9A. All P9B cells are inserted BEFORE `mod shared {` (line
597 pre-P9A; now ~705 post-P9A). New function names are disjoint from G1's
planned `bsms_4line_*` cells. Fixture filenames `bsms-2line-*` and
`bsms-6line-*` are disjoint from G1's `bsms-4line-*`. No conflict surface
in either merge order.

## Verification commands run

```bash
cd crates/mnemonic-toolkit

# 1. Compute checksums via temp helper (since-removed) — captured at
#    fixture-author time:
#    P9B.4 ypub  → nkwrvhy4
#    P9B.5 zpub  → yxzx3ag7
#    P9B.6 tr()  → rgn6fk37
#    P9B.7 bip45 → 3rldg0zz

# 2. Probe tr-NUMS actual behavior via since-removed `probe_tr_nums_behavior`
#    cell:
#    Result: exit 0, stdout contains cosigners=2 / network=mainnet / threshold=none / bsms_audit=none.
#    Side-channel: threshold=none surfaces extract_threshold regex gap on sortedmulti_a.
#    → I1 fold (rename cell + file FOLLOWUP).

# 3. Run all 6 new P9B cells.
cargo test --test cli_import_wallet_bsms
# Result: 32 passed; 0 failed; 0 ignored (was 26 after P9A; +6 from P9B).

# 4. Full workspace test suite (regression guard).
cargo test
# Result: ALL test binaries pass (full set; no failures).

# 5. Clippy regression on tests.
cargo clippy --tests -- -D warnings
# Result: GREEN — no warnings.
```

## Verdict

**GREEN after I1 fold.**

R0 surfaced one Important (plan-doc forward-looking refusal vs current
accept-tr() behavior); folded inline by renaming the cell + filing
cycle-followup `bsms-import-taproot-refusal-parity`. Per the plan-doc
reviewer-loop discipline, this counts as round R0 → fold-applied. No new
findings introduced by the fold (cell + FOLLOWUP entry are documentary
only; no src change). Round R0 converges GREEN.

Per CLAUDE.md "Reviewer-loop continues after every fold": the fold is
documentary (cell rename + FOLLOWUP entry) — no R1 dispatch warranted since
no executable code changed.

Next: commit P9B and proceed to opening PR against release/v0.28.0.

---

*Generated 2026-05-19 by autonomous executor self-review (G3 instance).*
