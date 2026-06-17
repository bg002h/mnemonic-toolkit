# Plan-R0 (Phase-0 test-hardening) round 2 — architect confirmation (verbatim)

> **Verdict: GREEN (0C/0I).** Both round-1 folds correctly applied; nothing regressed. Implementation unblocked.

- **I-1 (B1 negative cell)** — fold correct. `cli_restore_taproot.rs` cites match source: `build_at_in_both_descriptor` `:347`, `at_in_both_tr_refuses_structurally` `:447` (n=3 the genuine RED), `_2of2_` `:472`, `_sortedmulti_a_` `:489`, 2-of-2-coincidental nuance `:440-442`. Unconditional SKIP is right — @-in-both/non-NUMS refusal already comprehensively covered (incl. SortedMultiA arm), built md_codec-direct. B1 = positive tr leg only.
- **I-2 (B3 already-satisfied)** — fold correct. All 3 themes exist in `crates/ms-codec/tests/bch_all_lengths.rs` (sibling `mnemonic-secret` origin/master): `corrects_1_to_4_errors_every_length`, `five_to_eight_errors_never_return_original_every_length`, `raw_wrong_length_fails_closed_every_length`, faithfully asserting the claimed properties. No MUST-DO residual dropped; the `decode.rs:280-288` deterministic re-verify test is correctly deferred (theme-2 sweep already covers the property). Benign line-cite drift (plan slightly higher than master) — names/semantics resolve unambiguously.
- **Unchanged items still SOUND:** B1 positive leg, B2 (`arm_dup_if` mirror of `arm_non_zero`, walker `:668`), B4 (4-6 bitcoind corpus rows, no SortedMultiA) — untouched, stand as round-1 GREEN. No regression.

Implementation unblocked.
