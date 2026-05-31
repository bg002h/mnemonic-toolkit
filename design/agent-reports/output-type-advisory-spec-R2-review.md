# SPEC R2 review (after R1 fold) — output-type-stderr-advisory Phase 1
**Date:** 2026-05-31 · **Reviewer:** opus architect · **SHA:** `18cfdce` · **Verdict: GREEN (0C/0I).** SPEC R0 gate SATISFIED.

> Closes the SPEC reviewer-loop: R0 RED 1C/3I → R1 RED 0C/2I → R2 GREEN.

## R1 folds confirmed correct
- I-A: `worst_class_on_stdout(&[OutputClass]) -> Option<OutputClass>` (None=all-inert→no line); §4.3 "one line OR none"; inert = absence (no enum variant). No stale "always emits" claim survives. ✓
- I-B: verify-bundle/xpub-search inert-on-normal; inspect/convert emit §3-row class on normal + repaired-card class on the SEPARATE input-decode-failure short-circuit (`inspect.rs:135`/`convert.rs:994` inside `is_codec_decode_err`→`return Err(orig)`, mutually exclusive). All 3 branch claims match source. ✓
- m2: convert gate `is_argv_secret_bearing()` (`convert.rs:1099/117-123`); path/fingerprint = `is_side_input_only` = inert. ✓ m3: ms derive coexists with language note `derive.rs:246-248`. ✓

## Whole-spec coherence
§3 table ↔ §4.3 wiring ↔ §7 tests consistent for every multi-artifact/conditional command. Inert set consistent across §2.1/§3/§4.1/§4.3/§7/§8. Phase ordering safe (P0 helpers incl. Option-return before wiring; no forward-symbol). SemVer PATCH both, no GUI/list/lint lockstep, toolkit tag v0.38.2 + ms crates.io v0.5.1. Bundle keyless-md1-only edge impossible (mk1 always present → floor W). No invented issues.

**SPEC R2 GREEN.** Proceed to the implementation plan-doc (own R0 gate).
