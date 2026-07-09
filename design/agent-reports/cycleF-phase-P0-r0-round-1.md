# P0 per-phase R0 review — ms1-repair-demote-to-candidate — round 1

**Verdict: GREEN (0 Critical / 0 Important)** — 4 Minor (comment/doc-hygiene + 1 pre-existing → FOLLOWUP).
**Reviewer:** Fable (fresh per-phase R0 over the code), per user directive. Worktree HEAD `c01e67aa` (base `ecce14a7`).
**Dispatched:** 2026-07-09 (Cycle F, per-phase P0 R0, FULL suite). Persisted verbatim per CLAUDE.md.

P0 cleared to advance to P1. Code matches R0-GREEN SPEC §0.3/0.4/2/3/4 + the plan P0 + TEST-FLIP INVENTORY + G1-G9.

## Independent runs
`cargo test -p mnemonic-toolkit` → **205 suites, 3664 passed, 0 failed, 18 ignored, exit 0**. `cargo clippy --all-targets -- -D warnings` → clean. `cargo fmt --check` → diff only in `mlock.rs` (g6-exempt). Plus a 40-probe binary battery against the real `mnemonic`.

## FUNDS property (verify-bundle compare) — SAFE
`ms1_ground_truth_compare` (`verify_bundle.rs:2016-2039`) + both sites (single-sig :2123, multisig :2590): the ONLY pass-path is `Some(true)` requiring `corrected.as_str() == expected_ms1` (literal byte equality vs the typed seed's own card). `Err`→None; `repairs.is_empty()`→None; corrected≠expected→`Some(false)`→failed `ms1_entropy_match` row. No route by which a corrected ms1 ≠ `expected.ms1[i]` is presented as recovered.
- §5.5 wrong-bundle single-sig (binary-run): seed E, ms1=wallet-A@pos17, clean mk1/md1 A → **exit 4**, `ms1_entropy_match:fail`, `result:mismatch`, full 9-row table, no "recovered", no 5, no exit-2 abort; `--json` same.
- §5.4 MATCH (binary): same-seed corruption → **exit 0**, "recovered…confirmed against expected seed", `result:ok`.
- Multisig: per-index compare vs `expected.ms1[i]`; cross-index swap fails closed. CLI test green.
- Empty ground truth impossible (watch-only skip gates @:2067/:2534-2535 before compare); even if broken, `corrected != ""` → `Some(false)` → fail-closed.
- No `?`-abort/short-circuit/typed error (Option-scoped `?`; mismatch = pushed row; exit 4 via pre-existing `any_fail`). Uncorrectable → None reproduces pre-Cycle-F decode-error rows byte-for-byte.

## Secret-hygiene (G5/§8.6) — NO LEAK
Binary-verified text + `--json`: corrected(A), supplied-corrupt, expected(E) all absent from stdout/stderr/json. `expected`/`actual`/`diff_byte_offset` = `None` (`..Default::default()`). Both advisory strings fixed-text, zero interpolation. Corrected string held in `Zeroizing` at the compare site. §8.6 tests genuinely catch a leak (unit scans detail/expected/actual of ALL checks for BOTH corrected+expected + asserts forensic fields `is_none()`; CLI scans stdout+stderr). (Pre-existing clean-decode mismatch row @:2105-2113 populates expected/actual — SPEC §5.7 forensic design, untouched.)

## Demotion + advisory (G1/G4) — CORRECT
Ms1 arm @:1161-1176 `Unverified` iff `!repairs.is_empty()` else `Blessed` (no false-Bless-with-touch; clean→exit 0; both binary-verified). I2 advisory @:1754-1771 after the empty-repairs early-return, `matches!(kind, Ms1)`-gated → fires convert/inspect/xpub (exit 1), NOT verify-bundle (direct `repair_card` bypasses helper — probed absent). Remaining `try_repair_and_short_circuit` in verify_bundle = Mk1(:2207,:2424)/Md1(:2445,:3106) — mk1 partial-set unchanged (unit + `cli_mk1_repair_reverify` green).

## Flip inventory — binary-verified
cell_9=4 ✓, cell_19/18b=**1** (`Codex32`⇒1, `invalid short checksum`) ✓, cell_24=no-envelope/1 ✓, cell_27=0-MATCH ✓, cell_30=0/VerifyBundleJson/D20-unreachable ✓. Extra flips beyond inventory (cell_10/10b/12b/14/b7_1/argv/stdin/positional/xpub-search-15/per-card-language/output-class/cell_26-md1-swap) — all correct intended-behavior, none masks a regression. §5.8 delivered as a dedicated CLI+unit test (cell_28 still green) — equivalent.

## §5.6/5.7/5.9
Indel keep-5 untouched (zero indel-engine/`IndelJson` diff; unique single-indel → exit 5 binary-verified; multi-hit→Ambiguous→4 unit-covered per `cli_indel.rs:8-14` rationale, ~2⁻⁶⁵ collision, no CLI fixture). Mixed-kind → exit 4 (binary). `RepairJson.verdict` at fixed position after `kind` (`[schema_version,kind,verdict,corrected_chunks,repairs]`) — P1 byte-match anchor; `IndelJson` untouched.

## Collateral — none
Non-ms1 unchanged; no codec/Cargo.toml/vendor change (NO-BUMP); `verdict_str` consistent with candidate_seen→exit-4.

## Minors (non-gating)
1. `cmd/repair.rs:12-15` module-header exit-code doc still says "5 — REPAIR_APPLIED" (now false for primary ms1; stale for mk1 since Cycle E). Add the exit-4 VERIFY-ME row.
2. Under-inclusive Cycle-E comments now ms1 shares the path: `cmd/repair.rs:118-122/160-163/241-243` (candidate_seen = mk1-only), stale "ms1 auto-fire short-circuit" comments at `xpub_search/path_of_xpub.rs:226` + `passphrase_of_xpub.rs:274`.
3. Plan asked for an `assert` on `expected.ms1[i]` non-empty at the compare sites; add `debug_assert!(!expected_ms1.is_empty())` @:2123/:2590 (invariant structurally enforced + fails closed → hygiene).
4. The `Zeroizing` holder is a defensive clone; the source `outcome.corrected_chunks` Vec + `repair_card` internals still drop un-zeroized (pre-existing engine-wide, pre-dates this cycle) → file FOLLOWUP `repair-engine-outcome-zeroization`.

**GREEN — advance to P1.** (Coordinator: folding Minors 1-3 into P0 now; Minor 4 → FOLLOWUP.)
