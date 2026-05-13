# v0.9.0 Phase 1 — xprv-cell TDD design opinion (R1)

**Reviewer:** Opus 4.7 via `feature-dev:code-reviewer` agent, 2026-05-13.
**Branch:** `v0_9_0-phase-1-argv-leakage`.
**Pre-review HEAD:** `9a99766` (Phase 1 RED state — 3 tests, 19 RED cells).

## Question reviewed

Plan §"Phase 1 RED first" calls for 9 behavior cells in
`cli_argv_leakage.rs` (one per closed flag-row). Shipped 8.
Reason: `bundle --slot @0.xprv=-` behavior is identical pre- and
post-impl because `detect_bundle_mode` rejects on the
`SlotSubkey::Xprv` enum tag at `bundle.rs:470`, independent of slot
value. xprv `=-` covered structurally instead:
`lint_argv_secret_flags.rs` row 5 (asserts `slot_stdin` evidence
anchor) + `cli_secret_in_argv_warning.rs` cell 4 (asserts inline
xprv emits advisory).

Three options put to the reviewer:
- **A** Ship-as-is (8 cells + structural).
- **B** Add stdin-EOF discriminator cell (forces impl ordering).
- **C** Add `(read from stdin)` stderr annotation to discriminate.

## Verdict

**Decision: Option A — what was shipped is correct. Confidence 90.**

0 Critical, 0 Important. 2 supporting notes + 2 rejection findings.

## C/I/N findings (paraphrased; full Opus rationale follows)

- **N-1** (conf 90): Option A is structurally sound. The xprv-tag
  rejection fires on `subkeys.contains(&SlotSubkey::Xprv)`,
  independent of slot value. A behavior cell asserting
  `.failure().stderr(contains("deferred to v0.5+"))` passes
  pre-impl with `=-` taken literally — that's not RED, that's
  vacuous green. Inventing a discriminator for a runtime-deferred
  flag-row would be testing infrastructure, not user-visible
  behavior.

- **N-2** (conf 85): Structural coverage is adequate. The lint
  row 5 + advisory cell 4 jointly pin the impl contract:
  (a) parser-extension covers all four subkey tags uniformly
  (subkey-agnostic consume-step is the cheapest correct design),
  and (b) the advisory fires at run-entry before dispatch (Phase 1
  plan step 5 already mandates this). When v0.5+ lands ms-codec
  XPRV-tag support, the now-dormant route activates without
  re-TDD churn.

- **I-1 reject Option B** (conf 80): The stdin-EOF discriminator
  forces an impl ordering constraint (consume stdin at `run()`
  entry, before `detect_bundle_mode`) that the plan does NOT
  specify. Plan step 3 says "consumes from stdin at clap-parse-
  success time" — ambiguous on whether that precedes mode
  dispatch. If the impl reasonably wires consumption at slot-
  binding time (inside `detect_bundle_mode` flow), the cell's
  pre-impl reject would survive post-impl too, making RED↔GREEN
  non-durable.

- **I-2 reject Option C** (conf 95): Adding a `(read from stdin)`
  annotation purely to make a test discriminate is test-driven-by-
  the-test, not test-driven-by-design. Leaks test scaffolding
  into user-facing stderr; future stderr-stability reviewers will
  rightly question why xprv rejection has a provenance annotation
  that no other rejection has.

## Non-blocking suggestion (folded — see commit-on-fold below)

Strengthen the `cli_argv_leakage.rs` header comment by adding one
sentence:

> "When v0.5+ adds ms-codec XPRV-tag support and the
> `bundle.rs:470` reject branch is removed, add cell 9 here
> mirroring cells 1-3."

Converts the structural-only coverage gap into a tracked re-entry
point.

## Disposition

**0C / 0I / 2N — MERGE.** Option A confirmed. Suggestion folded
in-cycle (header re-entry-point sentence added).

## Sources

- `crates/mnemonic-toolkit/src/cmd/bundle.rs` (lines 470-477)
- `crates/mnemonic-toolkit/tests/cli_argv_leakage.rs`
- `crates/mnemonic-toolkit/tests/cli_secret_in_argv_warning.rs`
  (cell 4)
- `crates/mnemonic-toolkit/tests/lint_argv_secret_flags.rs`
  (row 5)
- `/home/bcg/.claude/plans/v0_9_0-secret-memory-hygiene.md` (Phase
  1, lines 65-147)
- `design/agent-reports/v0_9_0-secret-memory-survey.md` (toolkit
  §5 row 236)
