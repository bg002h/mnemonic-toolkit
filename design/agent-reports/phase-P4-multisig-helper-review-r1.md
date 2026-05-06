# Phase P.4 multisig helper expansion — code-reviewer r1 (2026-05-06)

## Findings

### Critical
**C-1: Stale `run_multisig` doc comment (lines 401-417 of pre-fix verify_bundle.rs).** Comment described pre-P.4 ordering with `stub_linkage[i]` (no SPEC §5.7 equivalent) and the wrong interleaving (per-check-batched instead of per-cosigner-interleaved).

**Status:** addressed inline. Doc comment rewritten to match actual emission (interleaved-per-cosigner 6N + 3 shared md1).

### Important

**I-1: `md1_xpub_match` uses ordered Vec equality.** SPEC §5.7 line 103 "all N pubkeys match" arguably implies set-equality. Template-mode synthesis preserves cosigner-index order so ordered comparison is correct for P.4's path. Descriptor-mode P.5 may surface false-fail cases.

**Status:** captured as FOLLOWUP `verify-bundle-multisig-md1-xpub-match-set-equality` at v0.4.5-nice-to-have. Re-evaluate after P.5 implementation.

**I-2: `card_for_cosigner` silent missing-card vs wrong-key conflation.** When supplied mk1 card's xpub matches no entry in the descriptor's pubkeys-TLV, `mk1_decode[i]` emits "skipped: mk1[i] not supplied or decode failed" — conflates two distinct failure modes.

**Status:** captured as FOLLOWUP `verify-bundle-multisig-cosigner-mapping-diagnostic` at v0.4.5-nice-to-have. Diagnostic clarity, not correctness.

### Low / Nit

**N-1: Missing-but-expected ms1 emits passed=true.** Full-mode multisig with no --ms1 supplied reports `result: ok` if mk1+md1 match. SPEC §5.7 line 104 specifies "skipped: watch-only slot" semantics ONLY for `ms1[i] == ""`; the missing-but-expected case is unspecified.

**Status:** captured as FOLLOWUP `verify-bundle-multisig-missing-ms1-passes-true` at v0.4.5-nice-to-have. Awaiting SPEC clarification.

**N-2: Test fixture comment described pre-final state (same chunk_set_id).** Test was updated to use 2 distinct seeds → distinct csi → grouping works.

**Status:** addressed inline (test-fixture comment updated to reflect distinct-seeds reality).

**N-3: `run_multisig` doc-comment cosmetic.** Subsumed by C-1 fix.

## Outcome

P.4 APPROVED with C-1 + N-2 + N-3 addressed inline. I-1, I-2, N-1 deferred via FOLLOWUPS at v0.4.5-nice-to-have tier. Proceed to P.5.
