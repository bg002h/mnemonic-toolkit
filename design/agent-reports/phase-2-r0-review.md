# Phase 2 R0 review — wallet-import v0.26.0

**Date:** 2026-05-18
**Reviewer:** opus architect
**Commit under review:** `7689cbc` (`phase 2: BSMS Round-2 parser + watch-only invariant + 16 cells`)
**Worktree:** `.claude/worktrees/wallet-import-export-multiformat-brainstorm`

**Verdict:** YELLOW — 0 Critical, 2 Important, 4 Minor. Fold both Importants before Phase 3 dispatch.

The Phase 2 implementation is broadly sound. The pipeline correctly inverts the export-side adapter, the placeholder ordering correction is verified against `lex_placeholders` source, the up-front `verify_checksum` call closes the gap left by `substitute_synthetic`, and the watch-only invariant + 6-line audit-fields surface match SPEC §8.1-§8.2 byte-faithfully.

## Critical

(none)

## Important

### I1 — First-address-mismatch WARNING (SPEC §4.1 + §2.4 row 3) deferred unilaterally

**Sites:** `wallet_import/bsms.rs:198-205` (deferral comment + no-op); `SPEC §4.1:149` (mandated for v0.26.0); `SPEC §2.4 row 3:71` (normative template); plan §2.6:295 (cell mandate); `tests/cli_import_wallet_bsms.rs:165-189` (cell asserts unrelated signature-not-verified WARNING).

SPEC + plan-doc both mandate the first-address-mismatch WARNING for v0.26.0 (informational, not hard-error). Phase 2 code deferred via comment-only FOLLOWUP citation; corresponding test cell asserts a different WARNING.

**Fold:** Defer with SPEC amend (preferred — first-address verification is informational-only in v0.26.0; ship without it). Amend SPEC §4.1 + §2.4 row 3; file `bsms-first-address-verify` in FOLLOWUPS.md; rename cell to `bsms_first_address_field_preserved_unverified` asserting audit-field preservation only.

### I2 — FOLLOWUP citations in code reference entries not present in `design/FOLLOWUPS.md`

**Sites:** `bsms.rs:14-15` cites `wallet-import-signet-regtest-disambiguation`; `bsms.rs:26-27` cites `wallet-import-bsms-checksum-delegation-note`; `bsms.rs:204` cites `bsms-first-address-verify`; `mod.rs:65` cites `bsms-verify-signatures` (planned in BRAINSTORM §6 but not yet filed).

Per `[[feedback-per-phase-agents-forget-followup-status-flip]]`: canonical split-state hazard inverted (code claims FOLLOWUP exists; FOLLOWUPS.md doesn't carry it).

**Fold:** File 4 open FOLLOWUP entries in `design/FOLLOWUPS.md`.

## Minor

### m1 — `bsms_multi_non_sorted_2_of_3` bifurcated success/error logic is near-vacuous

Cell accepts EITHER success (declaration-order pin) OR failure (any non-empty stderr). Error-branch vacuously satisfied. Comment claim "bare `multi(...)` is forbidden inside `wsh()`" is factually inaccurate. Stronger fold: use `wsh(multi(...))` form. Non-blocking.

### m2 — Coin-type mismatch template diverges from SPEC §2.4 generic "parse error:" wording

SPEC §2.4 generic uses `parse error: <detail>`; SPEC §4.2 step 8 specific does NOT include `parse error:`. Implementation matches §4.2 step 8. Pick canonical form in Phase 7 spec amend.

### m3 — `key_regex` hardened-marker char-class is `'`-only, not `'|h`

BIP-129 BSMS canonically uses `'`. Phase 3 Core fixtures + Phase 4 round-trip may surface `h`-hardened markers (BIP-380 allows both). Tighten in Phase 3 regex from inception OR document `'`-only acceptance.

### m4 — `build_slot_fields` re-runs `extract_origin_components` N times (O(N²))

Trivial inefficiency for small N. Non-blocking.

## Per-finding verdict

- **Implementer's finding 1 (placeholder ordering `@N[fp/path]`): ACCEPT.** Verified against `parse_descriptor.rs:69-71`.
- **Implementer's finding 2 (checksum explicit verify_checksum): ACCEPT WITH AMEND.** Phase 7 SPEC amend to fix §4.4 wording; file `wallet-import-bsms-checksum-delegation-note` FOLLOWUP per I2.
- **Implementer's finding 3 (schema test `_twelve_` → `_thirteen_`): ACCEPT.**
- **Implementer's finding 4 (thin CLI scaffold): ACCEPT.** No scope creep.
- **Implementer's finding 5 (3 extra cells): ACCEPT WITH AMEND.** mixed_coin_types_rejected stderr assertion permissive (m2 above) but acceptable.

## Notable strengths

1. Placeholder ordering correction is source-grounded.
2. `verify_checksum` up-front closes a real correctness gap.
3. Watch-only invariant defense-in-depth at `bsms.rs:194`.
4. 6-line audit-fields preservation matches SPEC §8.1.
5. `FutureFormat` routing for `BSMS 2.0` → exit 3.
6. Trait shape `parse(blob, &mut dyn Write)` forward-compatible with Phase 5 dispatch.
7. SLIP-132 tolerance via `slip0132::normalize_xpub_prefix`.
8. WARNING templates byte-exact against SPEC §2.4 rows 1 + 2.

## Recommendation

**Proceed to Phase 3 after folding I1 + I2.**
