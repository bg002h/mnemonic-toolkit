# IMPLEMENTATION PLAN — own-account subset-search (+ opt-in bounded unowned)

**Date:** 2026-06-20 · **SPEC (R0-GREEN, 2 rounds):** `design/SPEC_own_account_subset_search_2026-06-20.md` + `design/agent-reports/own-account-subset-search-spec-r0-round{1,2}-review.md`. **Brainstorm (R0-GREEN, 2 rounds):** `…BRAINSTORM…`.
**Plan base SHA:** mnemonic-toolkit `5ab7df08` (branch `feature/own-account-subset-search`; src tree unchanged since v0.60.0 `82e58674` — all intervening commits are design docs, so SPEC citations are live).
**SemVer:** toolkit **MINOR `0.60.0 → 0.61.0`**. md-codec/mk-codec NO-BUMP. GUI MINOR paired (P6).
**Gate discipline:** per-phase TDD (RED-first) + per-phase opus R0 to **0C/0I before advancing** (CLAUDE.md). Funds-safety / silent-wrong-wallet + spurious-NO-MATCH class — the enumerated≡counted bijection, the distinct-keys floor, the prefix-strength=`realized_s` sizing, the early-exit gate's exact-path byte-invariance, and the address-equivalence differential vs an INDEPENDENT golden are the make-or-break gates. Single coupled toolkit PR; GUI schema-mirror + manual in lockstep (P6).

## 0. Grep-verified citations (plan base `5ab7df08`, full paths)
- `crates/mnemonic-toolkit/src/cmd/restore.rs`: `complete_multisig_template` (`:1416`, shared core), `--own-account-max` refuse gate (`:1434`), supply gates (`:1626`/`:1635`), `realized_s = perm_count_u128(n,n)` (`:1661`), id/addr mode select (`:1665-1666`), `reject_duplicate_keys` whole-pool (`:1648`, on `c.key65` `:1647`), `sorted_shape` binding (`:1676`) + the evaluator-filter (`:1739`, `assignment != identity → false`), `perm_count_u128` (`:1882`), `--account` `Vec<u32>` `default_value="0"` (`:106`), a `conflicts_with` precedent (`:86`), `run_multisig_template_completion` (`:1321`).
- `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`: `verify_multisig_template` calls `complete_multisig_template` (`:808`→`:874`); `own_account_max: None` hardcoded (`:865`); `--account` SCALAR `u32` `default_value="0"` (`:64`).
- `crates/mnemonic-toolkit/src/permutation_search.rs`: `search` (`:551`, full-scans both modes; short-circuits only at 2nd match `:623-624`, doc `:530-548`), `unrank_permutation` (`:494`), `factorial` (`:481`), `total_candidates` (`:509`), `required_prefix_bytes` (`:322`), `validate_prefix_strength` (`:342`), the pinned `prefix_ladder_own_account_max_subset_space` test (`:740`).
- `crates/mnemonic-toolkit/src/synthesize.rs`: `is_order_independent_shape` (`:335`).
- Tests to FLIP (SPEC §7-P2, R0-r1 I-2): `crates/mnemonic-toolkit/tests/cli_restore_md1_template_multisig.rs` — `own_account_max_flag_refuses_with_actionable_message` (`:677`, REWRITE), `pool_larger_than_slots_refuses_with_actionable_message` (`:715`, UPDATE message), `multi_account_own_resolves_both_slots` (`:635`, stays byte-GREEN). 27 `#[test]` in the file.

## 1. Phase map (each = TDD-RED → impl → per-phase R0 to 0C/0I)

### P1 — Engine: subset-select enumeration + cardinality (`permutation_search.rs`)
- **Impl:** (a) `unrank_kperm(rank, pool, n) -> Vec<usize>` (injective k-permutation, Lehmer-style, count `P(pool,n)`). (b) the OWN-ANCHORED composed-rank generator: `rank → (combo_rank = rank / N!, perm_rank = rank % N!)`; `combo_rank` → j-subset of `K_own` own indices via a CNS-unrank helper; the N selected keys = j chosen own ++ all M cosigners; `perm_rank → unrank_permutation(perm_rank, N)` orders them into N slots. (c) SORTED variant: enumeration-side — drop `perm_rank`, emit `C(K_own,j)` identity-ordered subsets (SPEC §3/§4.1). (d) the opt-in STRATIFIED generator (SPEC §4.3): partition `[0,S_opt)` by j-strata, compose `(own-combo, cosigner-combo, perm)`. (e) cardinality helpers: `c_choose(K_own,j) -> Option<u128>` (overflow-checked — `C(256,128)` is 252-bit; `None`→refuse, R0-r1 m-3); `S_own = c_choose(K_own,j)?.checked_mul(factorial(N)?)?`; `S_opt = Σ` (overflow-checked); `total_candidates_subset(mode, S)`. (f) the `early_exit` engine knob (SPEC §4.4): add `search` a per-call `early_exit: bool` (default false = today's full-scan-with-2nd-match-ambiguity, byte-unchanged) OR a `SearchMode::Address { early_exit }` variant — **decide the exact shape here** (open-item: lean `early_exit: bool` param to `search` — smallest surface; the exact path passes `false`, the over-supply address path passes `true`).
- **TDD-RED:** `unrank_kperm` bijection (exhaustive small-(pool,n): enumerated == brute-force injective-placement set, each once); own-anchored generator bijection over `S_own` (exhaustive small-(K_own,j,M): == independently-generated valid set; `count == C(K_own,j)·N!`; NO cosigner-dropping); sorted variant (`C(K_own,j)`, identity); opt-in stratified bijection over `S_opt` (m-2); `c_choose` overflow → None; `early_exit=false` reproduces today's full-scan outcome (the byte-invariance unit anchor).
- **R0 focus:** the bijection (off-by-one in CNS/k-perm unrank = funds-safety bug); overflow refuse; the `early_exit` knob default cannot change v0.60.0.

### P2 — restore own-only (`cmd/restore.rs`)
- **Impl:** remove the `--own-account-max` refuse (`:1434`); add `#[arg(long="own-account-max", conflicts_with="account")]` (R0-r1 I-4 — clap-native, `--own-account-max` alone passes). In `complete_multisig_template`: build the own pool from `--account` list OR `--own-account-max` range (`0..K_own−1`); the §5a premise gates (own-only REFUSES extra cosigners with the actionable message; under-supply → NO-MATCH; own-as-cosigner → distinct-keys floor; single-own-seed scope); `realized_s = S_own` (or `S_own_sorted`) via the P1 helpers (NOT `perm_count_u128(n,n)` for over-supply; exact pool still `N!`); drive `search` with the own-anchored generator; address-search over-supply passes `early_exit=true`, exact path `false`; the §6 hard ceiling (`K_own≤256`, `S_MAX=1e15`) refuses before calibration; `@N=`⊕subset-search mutual-exclusion refuse.
- **TDD-RED:** own at account 3 + `--own-account-max 5` completes to the INDEPENDENT golden; `--own-account-max 5` ALONE passes (I-4 guard) + `--account 0,1 --own-account-max 5` refuses (clap); §5a premise refusals each fail-safe; **backward-compat (R0-r1 I-2):** exact-pool tests byte-identical GREEN incl. `multi_account_own_resolves_both_slots:635` + a NEW v0.60.0-exact-path address-search before/after byte-regression guard (I-5); `own_account_max_flag_refuses_…:677` REWRITTEN (RED-first) to assert search-completes; `pool_larger_than_slots_…:715` message UPDATED; worked 10-byte prefix sizing; address-search early-exit finds a non-zero-own-account wallet.
- **R0 focus (LOAD-BEARING):** the per-slot origin BUILD unchanged (C1); `realized_s == enumerated`; the exact-path byte-invariance (address-search early-exit gated); the swapped/wrong-subset → refuse; the over-supply path inherits the v0.60.0 id-vs-addr mode precedence (`restore.rs:1665-1666`) — never both in one search (R0-r1 m-c).

### P3 — opt-in `--search-cosigner-subset` (`cmd/restore.rs`)
- **Impl:** the NEW `#[arg(long="search-cosigner-subset")]` boolean; when ON, relax the exact-cosigner gate, build the pool with over-supplied cosigners, `realized_s = S_opt` via the stratified generator + helpers; the §6 ceiling refuse; default OFF = own-only (P2).
- **TDD-RED:** over-supplied cosigner pool completes to the golden; the stratified-generator brute-force-reference bijection — incl. the SORTED opt-in variant (drop `·N!` per stratum → `Σ_j C(K_own,j)·C(M_sup,N−j)`, R0-r1 m-b) (m-2); the hard-ceiling refuse; `@N=`⊕`--search-cosigner-subset` refuse.
- **R0 focus:** `S_opt == enumerated` (stratified); the bound (ceiling + cap) cannot DoS.

### P4 — verify-bundle (`cmd/verify_bundle.rs`)
- **Impl:** add `--own-account-max` (NEW NAME) + `--search-cosigner-subset` (NEW NAME) to `VerifyBundleArgs` (R0-r1 I-3 — both gated; `--account` stays scalar `u32`); wire into the shared `complete_multisig_template` (it already gets the feature via the core — just expose + pass the flags, replacing the hardcoded `own_account_max: None` `:865`).
- **TDD-RED:** verify-bundle parity (verify == restore == golden) for an over-supplied own-account completion; the verify-bundle flag refusals mirror restore.
- **R0 focus:** parity; no verify-only regression.

### P5 — differential + property (`tests/`)
- **(default-CI)** randomized own-only subset-search completes to the INDEPENDENT rust-miniscript golden (over a range of own accounts); anti-vacuity (wrong subset ≠). Optionally an opt-in property row. (Bitcoind corpus row optional/`#[ignore]`.)
- **R0 focus:** oracle non-vacuity (independent golden, not md-codec reconstruction).

### P6 — locksteps + version + ship
- **GUI schema_mirror** (`mnemonic-gui/src/schema/mnemonic.rs`): restore +`--search-cosigner-subset`; verify-bundle +`--own-account-max` +`--search-cosigner-subset` (R0-r1 I-3). `cargo test --test schema_mirror` GREEN against a **v0.61.0** binary (NOT the stale `$PATH` `mnemonic` — use `MNEMONIC_BIN`, the [[GUI gotcha]]); GUI MINOR; do NOT cargo fmt the GUI.
- **Manual** (`docs/manual/src/40-cli-reference/41-mnemonic.md`): the `--own-account-max` row refuse→search (restore) + the new rows (verify-bundle ×2, `--search-cosigner-subset` ×2) + a "subset-search" subsection; `make -C docs/manual lint` GREEN (use v0.61.0 + sibling bins).
- **Version 0.61.0:** the 7 sites (`crates/mnemonic-toolkit/Cargo.toml`, BOTH READMEs, `scripts/install.sh:32` self-pin, `fuzz/Cargo.lock`, `Cargo.lock`, `CHANGELOG.md`) (R0-r1 m-a). fmt `cargo +1.95.0 fmt -p mnemonic-toolkit` then `git checkout -- …/mlock.rs` (g6).
- **Mandatory post-impl whole-diff adversarial exec review** over the full cycle BEFORE tag. Ship (commit → ff master → tag `mnemonic-toolkit-v0.61.0` → push). Update the pinned `prefix_ladder_own_account_max_subset_space` test (`permutation_search.rs:740`) to `S_own` (SPEC §3 supersession).
- **Housekeeping:** flip FOLLOWUP `template-multisig-own-account-range-subset-search` → RESOLVED in the shipping commit; cross-ref the umbrella `bundle-md1-template-only-option`.

## 2. Risks / per-phase R0 focus
- **P1 is the combinatorics R0** (the bijection / off-by-one) — exhaustive brute-force-reference tests are the gate.
- **P2 is the funds-safety R0** (the exact-path byte-invariance under the `early_exit` gate; `realized_s == enumerated`; the premise-violation fail-safe).
- The `early_exit` knob default MUST be the v0.60.0 full-scan behavior; the over-supply address path opts in.
- Per CLAUDE.md: run the FULL `cargo test -p` suite at each R0 (not targeted targets — [[the P4/P5 stale-lint lesson from #28]]).

## 3. Open execution-time items
1. Re-grep all citations vs the execution base SHA (decay; preserve full `crates/…/src/cmd/` paths).
2. The `early_exit` engine-API shape (`search` param vs `SearchMode` variant) — P1 decides (lean: `early_exit: bool` param).
3. The opt-in stratified-unrank concrete index/offset arithmetic — P1/P3 (brute-force-tested).
4. `--own-slots <j>` opt-in pin flag — default infer the `j` range; add only if the inferred range proves too loose (deferred; the inferred range is safe per SPEC §4.3).
5. Exact wording of the new refusal messages (premise gates, ceiling, mutex) — match the v0.60.0 actionable-message style.
