# v0.28.0 Phase 7 (G1) — P7B R0 self-review

**Phase:** P7B — REPLACE existing 6-line `writeln!(stderr, "notice: ...")` block at `bsms.rs:111-117` with the §10.4 DEPRECATION text (4-line stderr notice).
**Reviewer:** Executor self-review (Task-dispatch unavailable in autonomous session).
**Source SHA reviewed against:** branch `v0.28.0/g1-bsms-4line` rooted at `release/v0.28.0` `71592bc`.
**Verdict:** GREEN.

---

## Critical

NONE.

## Important

### I1. SPEC §10.4 deprecation text byte-equal to user-prompt spec (verbatim)

The user-prompt provides 4 prose lines:
```
notice: import-wallet: bsms: 6-line lenient shape is DEPRECATED in v0.28+ and
will be removed in a future minor version; convert your blob to the BIP-129-
canonical 4-line shape (BSMS_VERSION + DESCRIPTOR + path-restrictions +
FIRST_ADDRESS) for forward compatibility. See SPEC §10 for the canonical shape.
```

Implementation at `bsms.rs:163-180` (post-P7A insertion; the new arm is between 2-line and 6-line, so the 6-line arm shifted down a few lines from the pre-P7B `:111-117` plan-doc citation). Renders the 4 lines via 4 separate `writeln!` calls (each line is independently sized; multi-line single `writeln!` would be brittle and harder to grep for individual lines).

Verified byte-exact against the user-prompt spec by unit test `parse_6line_emits_deprecation_notice_shape` (substring grep on each of the 4 distinguishing fragments).

## Minor

### M1. legacy "not verified inline" + "--bsms-round1" wording FOLDED OUT

Pre-P7B the 6-line arm emitted a NOTICE pointing the user at v0.27.0's `--bsms-round1 <FILE>` flag for BIP-322 verification. P7B replaces this with the DEPRECATION text per the user-prompt §10.4. The `--bsms-round1` pointer is no longer surfaced from the 6-line ingest arm; users who need BIP-322 verification of Round-1 records continue to use the `--bsms-round1 <FILE>` flag independently (it remains documented in the help text + manual). The pre-existing integration cell `bsms_6_line_happy_path` (which asserted the legacy NOTICE) was migrated in-place to assert the new DEPRECATION shape.

### M2. integration cells use exit 0 + STDIN piping; unit tests use library-direct dispatch

`parse_6line_emits_deprecation_notice_shape` exercises the library API directly (`BsmsParser::parse(blob.as_bytes(), &mut Vec<u8>)`), avoiding the assert_cmd subprocess overhead. The integration cell `bsms_6line_still_accepted_with_deprecation_notice` exercises the same surface via the CLI subprocess. The two cells overlap intentionally — the unit test is the regression guard against the writeln! shape; the integration cell is the regression guard against the CLI-surface visibility.

## Stderr-emission shape verification

Test `parse_6line_emits_deprecation_notice_shape` asserts ALL 4 substrings:
1. `"6-line lenient shape is DEPRECATED in v0.28+ and"` — line 1
2. `"will be removed in a future minor version"` — line 2
3. `"canonical 4-line shape"` — line 3
4. `"SPEC §10 for the canonical shape"` — line 4

Each substring is unique to its line — partial-removal of any individual writeln! call would trip exactly one substring assertion. Regression-guard granularity is per-line.

## Reviewer-loop reconverge

R0 GREEN; no folds; no R1.
