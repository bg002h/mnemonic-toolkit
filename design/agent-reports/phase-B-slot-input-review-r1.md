# Phase B — slot-input parsing review — r1

**Date:** 2026-05-05
**Commit under review:** `505533a`
**Reviewer:** feature-dev:code-reviewer (sonnet)
**Verdict:** APPROVE — 0C / 0I / 1L / 2N. Terminates the iterative-review loop per `feedback_iterative_review_every_phase` (no Critical or Important findings; convergence reached at r1).

## Critical / Important

None.

## Low

**L-1 — `is_legal_set` contains six unreachable match arms** (FIXED INLINE)

`crates/mnemonic-toolkit/src/slot_input.rs:303-310`

`is_legal_set` is always called with a sorted `&[SlotSubkey]` (caller pre-sorts at line 177). Given the derived `Ord` declaration order `Phrase(0) < Entropy(1) < Xpub(2) < Fingerprint(3) < Path(4) < Wif(5) < Xprv(6)`, the only reachable 2-element watch-only patterns are `[Xpub, Fingerprint]` and `[Xpub, Path]`; the only reachable 3-element pattern is `[Xpub, Fingerprint, Path]`. Six reverse / non-canonical permutations are dead code. Fixed inline (delete the unreachable arms; the sorted-input invariant comment added).

## Nits

**N-1 — Test count 17 vs plan-stated 18 for B.2.** The `[≥10]` threshold from the plan is comfortably satisfied (17 parser tests). No action required.

**N-2 — Module-wide `#![allow(dead_code)]` instead of per-item suppression.** Consistent with phase-staged module convention; will be removed in Phase C once callers wire `validate_slot_set` and `expand_legacy_to_slots`.

## Verified

- **B.1:** `SlotInput` / `SlotSubkey` shapes match SPIKE-2 lock verbatim (`slot_input.rs:12-60`). The plan's stronger `SlotValue` enum is correctly deferred to post-validation.
- **B.2:** Parser grammar correct for all probed branches: index range (`@0..@255`, `@256` rejected), missing `@`, missing index, missing `.`, missing `=`, empty subkey, unknown subkey, empty value (SPIKE-2 REJECT lock), `=` inside value (first-`=` split semantics), unicode subkey, leading whitespace.
- **B.3:** `--slot` clap wiring at `cmd/bundle.rs:115-116` correct (`ArgAction::Append`, `value_parser = parse_slot_input`); `ParseError(String)` threads through clap's "invalid value" wrapper. `args.slot` not read in `bundle::run()` at this commit (Phase C wires).
- **B.4:** `validate_slot_set` correct vs SPEC §6.6.b + row 8 contiguity. `{phrase, entropy}` correctly classified `invalid-set` (not `conflict`); `{fingerprint}` and `{path}` alone correctly `invalid-set`. Duplicate-subkey detection via sort+dedup correct.
- **B.5:** `expand_legacy_to_slots` correct vs SPEC §6.6.a rows 5-7. Row 6 fires before virtual phrase expansion. Row 5 derived-N computation handles empty-slots edge case.
- **B.6:** SPEC §6.6.a + §6.6.b cross-checked vs impl; subkey vocabulary, conflict-row error messages, validity-matrix shapes match. No SPEC drift.
- **No regressions.** Grep over 16 integration test files: zero `--slot` references. 199 lib + integration tests still pass.

## Verdict

**APPROVE** — 0C / 0I / 1L / 2N. Phase B complete; Phase C green-light. L-1 fixed inline; N-1 and N-2 noted, no action.
