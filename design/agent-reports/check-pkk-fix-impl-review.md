# Implementation Review — toolkit Check-PkK non-tap canonical fix (self-review)

Reviewer: orchestrator (Fable 5), 2026-06-12. Verified against the GREEN R2 spec.

## Verdict: GREEN (0C/0I)

A WIRE change (v0.55.0 MINOR). TDD-implemented + orchestrator-verified:

- **TDD RED→GREEN:** the 4 inverted AST tests + 4 leg-1 goldens fail against the pre-fix walker (golden_wsh_pk emitted `9ad78e4f` vs golden `58d18033`), pass after the fix. The user mandate ("if we change wire format, make sure our tests cover it") is met by 4 coverage legs.
- **The fix (parse_descriptor.rs):** the `if tap_context` gate dropped → `Check(PkK|PkH)→bare` collapses unconditionally (matching md-cli + SPEC §5.1); Check-over-non-key preserved. The dead `tap_context` param removed — `grep tap_context crates/.../src/parse_descriptor.rs` = 0 code uses (comments only). Verified.
- **Leg 1 (the always-on wire-capture):** new `tests/cli_check_pkk_canonical_golden.rs` — 4 goldens via `bundle --descriptor --json → md_codec::chunk::reassemble → compute_wallet_policy_id/compute_wallet_descriptor_template_id → hex` (in-crate, NO external binary). I ran it: **6 cells pass** (4 goldens + 2 round-trips). The 4 policy/template ids match the frozen table EXACTLY, and are byte-identical to md-cli 0.6.2's conformant output (cross-confirmed). Cycle-E already proved bitcoind derives these descriptors correctly, so the new wire is funds-correct.
- **Leg 2 (round-trip):** `bundle --descriptor wsh(pk(…)) → restore --md1` reconstructs `wsh(pk(…))` + valid bc1q addresses (targets wsh(pk)/wsh(pkh) — the prop test only does combinator-nested pk). Proves the toolkit reads its own new wire form.
- **Leg 3 (differential):** the 4 entries flipped Diverge→Match; the anti-vacuity guards restructured (kept `n_match>=1`+`saw_match`, dropped the hard Diverge requirement, added `n_both_error==0 && n_tool_error==0` — the real both-tools-error false-Match risk). I ran the differential with MNEMONIC_BIN+MD_BIN: **passes** (8/8 Match). Self-comments citing the renamed test updated.
- **Leg 4 (AST):** the 4 inverted (`walk_wsh_pk_root`/`walk_sh_ms_pk_root`→PkK, `walk_check_kept_in_non_tap_context`→renamed `walk_check_collapsed_in_non_tap` asserting bare PkK, `walk_pk_h_via_wsh_andor`→PkH directly).
- **Round-trip safety (the wire-change correctness anchor):** md-codec's to_miniscript accepts BOTH bare PkK and Check(PkK) (idempotence arm) — so old cards (Check(PkK)) AND new cards (bare PkK) both restore identically. No card-READING regression. CHANGELOG states this.
- **Suite:** full `cargo test -p mnemonic-toolkit` green (1719 passed, 0 failed; prop 9/9); `cargo clippy --all-targets -D warnings` exit 0; `cargo metadata --locked` in sync.
- **Release:** v0.54.4→0.55.0 across 6 lockstep sites + CHANGELOG `[0.55.0]` (MINOR wire-content change, framed like v0.48.0 NUMS, the 8 affected shapes + the no-card-reading-regression note).
- **FOLLOWUP resolved both repos:** toolkit primary + descriptor-mnemonic companion + the cross-linked v2-design-questions item 12.
- **No stray churn:** git status = only the intended files (parse_descriptor.rs, 2 differential/golden test files, 6 release sites, 2 FOLLOWUPS.md). No cargo fmt run.

Cleared to commit + tag mnemonic-toolkit-v0.55.0. This conforms the toolkit's walker to the constellation's canonical wire form — the Cycle-D divergence is closed, and the cross-tool differential (now all-Match) is its permanent regression gate.
