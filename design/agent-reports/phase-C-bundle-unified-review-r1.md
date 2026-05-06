# Phase C — bundle-unified pre-check ladder review — r1

**Date:** 2026-05-05
**Commit under review:** `ea592fd`
**Reviewer:** feature-dev:code-reviewer (sonnet)
**Verdict:** APPROVE WITH NITS — 0C / 0I / 1L / 3N. Terminates the iterative-review loop per `feedback_iterative_review_every_phase` (no Critical or Important findings; convergence reached at r1).

## Critical / Important

None.

## Low (FIXED INLINE)

**L-1 — Stale module-level comment in `slot_input.rs:6`** —
The dead_code allow comment said `// validate_slot_set + helpers wired in Phase C.` but Phase C deferred those wirings to Phase D. Updated to: `// validate_slot_set + expand_legacy_to_slots wired in Phase D; parse_slot_input wired via bundle.rs:115.`

## Nits (resolved inline)

- **N-1** — `detect_bundle_mode` missing N=3 hybrid test case. Phase D will add integration coverage naturally.
- **N-2** — `BundleMode` doc-comment cited `§3.3 (revised)` which has no anchor in the v0.4 SPEC delta file. Updated to `/// v0.4 bundle-mode classification (impl plan Phase C.3).`
- **N-3** — File-level `#![allow(dead_code)]` covers actively-used items (`REMOVED_SUBCOMMAND_ERR`, `detect_removed_subcommand`). No correctness impact; will be tightened in Phase D when the other items are wired and the file-level suppression is no longer needed.

## Verified

- **C.1 trap mechanism:** `detect_removed_subcommand` wired pre-clap in `main.rs:45-49`. Bundle-scoped exact-token match. Edge-case bypasses (`bundle multisig-full=value`, `bundle -- multisig-full`) accept-as-deferred per SPIKE r1 routing (FOLLOWUPS already filed at `v0.4-nice-to-have`).
- **C.1 byte-exact stderr:** `REMOVED_SUBCOMMAND_ERR` matches SPEC §6.6 row 1 verbatim. `writeln!` adds the trailing `\n`. Two CLI integration tests assert byte-exact via live binary.
- **C.3 BundleMode:** 5 variants per impl plan; descriptor presence orthogonal. `detect_bundle_mode` classifier exhaustive over validated inputs.
- **C.4 pre_check_threshold + pre_check_template_n:** SPEC rows 9, 9.5, 10, 11 byte-exact.
- **C.5 SPEC §6.6 row 1 cross-check:** verbatim agreement.
- **Scope decision:** legacy `bundle::run` deliberately untouched; Phase D wires. New helpers sitting dormant under `#![allow(dead_code)]` is the planned Phase C end-state, not a hazard.
- **No regressions:** 223 lib tests + integration suites pass; new tests add 24 + 2.

## Verdict

**APPROVE WITH NITS** — 0C / 0I / 1L / 3N. Phase C complete; Phase D green-light. L-1 + N-2 fixed inline; N-1 + N-3 deferred to Phase D natural cleanup.
