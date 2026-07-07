# PER-PHASE R0 (P0+P1) — round 1

**Verdict:** GREEN (0C / 0I)
**Reviewer:** opus architect, worktree `feature/core-receive-change-pair-merge` @ HEAD (`750dc195`), base `afaabee5`
**Dispatched:** 2026-07-06 (Cycle B, per-phase R0 over the P0+P1 code diff, funds-weighted). Persisted verbatim per CLAUDE.md.

**GREEN — cleared to advance to P2 (docs).**

## Independent verification (ran in the worktree)
- **Full suite `cargo test -p mnemonic-toolkit`: PASS, exit 0.** Zero `FAILED`/`failed`/`error[`/`panicked`. Independently confirmed (also confirmed by the coordinator: CARGO_EXIT=0, 201 "test result: ok" binaries).
- **`cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`: PASS, exit 0.**
- **fmt:** `cargo fmt -p mnemonic-toolkit --check` diffs ONLY in `src/mlock.rs` (permanently fmt-exempt g6, untouched by this diff); new code is fmt-clean.
- Diff scope confirmed: only `bitcoin_core.rs` (+445), `mod.rs` (ripple), `import_wallet.rs` (emit), and the two bitcoin-core test files.

## Critical — none
## Important — none

## Minor (non-gating)
- **[m1] §7 near-miss message precision** — `bitcoin_core.rs:538-560`. The differentiated-refusal message is the SPEC-locked wording (rev-5 §7), funds-safe (exit 2 either way). It NEVER fires on coherent Core `listdescriptors` output (every pair consumed by a same-group merge; `merged_candidate_idx` skip covers all candidates — traced). It can only fire on a hand-crafted blob mixing two independent wallets or an asymmetric lone-pair, where "look like a receive/change pair" slightly overclaims two unrelated lone descriptors; mitigated by the same message's "If these ARE one wallet, combine them by hand…". Not a code defect (matches locked SPEC). No action for GREEN.

## Scrutiny findings (7 funds-weighted asks)
1. **Guard matrix — SAFE / airtight.** Extraction via `MsDescriptor::from_str` (§4.1, never `lex_placeholders`). Cond-2: `MergeGroupKey` = `Vec<MergeKeySig>` with `xk.xkey.to_string()` + fp + origin, positional `PartialEq` → two different wallets get different groups → cannot merge. Script-path `tr`: `tap_tree().is_some() → None` (floor-reject; §8.15 confirms). Global replace (`:509-519`): trailing `/*` matches only use-sites (origins end `']`, never contain `*`; an adversarial `[fp/0/*]` fails `from_str`); multi-digit safe via cond-7 uniformity; nested `sh(wsh)` safe via global `iter_pk()`+`str::replace`; defensive `contains` fail-closed → no partial per-key rewrite possible. M9 checksum-before-consume verified. Vec remove-both-insert-one N-pair safe.
2. **§7 near-miss — SPEC-compliant, funds-safe, deterministic.** Never fires on real Core output; a false-negative merely defers to the generic floor reject (also exit 2). No funds risk.
3. **Deviation 1 (roundtrip flips) — correctly restored, NOT weakened.** All 6 cells assert the right new behavior (`bundles=1`, FP/xpub, `cosigners=2`+`threshold=2`, merged `<0;1>/*` fresh checksum, select→1 bundle, and `semantic_match==true` on export→import). §8.2/§8.3 flipped; lone-entry floor reject kept.
4. **Deviation 3 (`assert_descriptor_verify_bundle_ok` MkField) — correctness fix, NOT a weakening.** Handles `MkField::Single`+`Multi` untagged shapes; `result=="ok"` preserved; explicitly the secondary net, not a substitute for the address oracle.
5. **Original-anchored oracles (§8.4/§8.10/§8.15) — GENUINELY non-tautological.** Derive `expected_recv`/`expected_chg` from the ORIGINAL `/0/*` and `/1/*` strings; read merged desc from `--json bundle.descriptor` with `.expect("must NOT fall back to re-authored <0;1>")`; split via `into_single_descriptors()`; assert chain-0==orig `/0/*` AND chain-1==orig `/1/*`. §8.10 (3-key sortedmulti) is the anti-C1 guard: a partial rewrite fails `got_chg != expected_chg`.
6. **Collateral — NONE.** Only bitcoin-core test files changed. `CoreSourceMetadata` is bitcoin-core-only; new `apply_select_descriptor` predicates behavior-identical for `Some`; non-core hits `unwrap_or(false)` unchanged. Full suite green confirms no silent regression.
7. **Determinism / §8.13 / clippy-fmt.** Deterministic (first-seen Vec bucketing; HashSet/HashMap only for keyed contains/remove, never iterated to output; original-index assembly; no Date/random). §8.13 carries the regression-lock + select-semantics teeth against shape-based `None`. Clippy green; new code fmt-clean.

## Notes
- Faithfully realizes SPEC rev-5 + PLAN rev-2: miniscript extraction (§4.1), uniformity-verified global-replace (§4.3), explicit `internal` provenance (§5), M9 checksum validation, and the P1 "cheap insurance" test `in_scope_merge_candidate_shapes_parse_via_rust_miniscript`.
- `multi_keyword_of` (longest-prefix-first) is a sound belt-and-suspenders discriminant on `desc_type()`; cond-2 key identity is the funds backstop regardless.
- Nothing blocks P2. The mandatory post-impl whole-diff review still runs after P2, weighted to the guard matrix + §8.10 anti-C1 oracle.
