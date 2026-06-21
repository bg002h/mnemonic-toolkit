> Reviewer: opus architect (plan-doc R0 round 1) · 2026-06-20 · IMPLEMENTATION_PLAN `design/IMPLEMENTATION_PLAN_own_account_subset_search_2026-06-20.md` @ HEAD `581a744d` (branch `feature/own-account-subset-search`), plan base `5ab7df08` · src tree source-verified against v0.60.0 `82e58674` (confirmed byte-identical: `git diff --stat 82e58674 5ab7df08 -- crates/.../src crates/.../tests` is EMPTY). R0-GREEN SPEC (`...spec-r0-round{1,2}-review.md`) + R0-GREEN brainstorm read in full as the contract.

**Verdict: GREEN — 0 Critical, 0 Important.**

The plan faithfully maps the R0-GREEN SPEC's funds-safety contract into 6 executable phases with the right dependency order (P1 engine → P2/P3 consumers → P4 verify-bundle → P5 differential → P6 ship). **Every §0 citation is structurally correct** — I re-grepped all 23 against the live src tree (unchanged since v0.60.0) and each lands on the exact symbol/line claimed. The plan's backward-compat split is HONEST: I read all three named tests at `:635`/`:677`/`:715` and the plan's REWRITE/UPDATE/keep-byte-green characterization matches the actual test bodies. Every SPEC §-contract item maps to a phase + RED-first TDD home (full matrix below) — nothing dropped on the floor. The early-exit knob is correctly placed in P1 (engine) with its over-supply-gated use in P2. The 7 version sites, the GUI schema_mirror scope (restore +1 / verify-bundle +2), the stale-`$PATH`-binary gotcha, the pinned `prefix_ladder_…→S_own` update, the FOLLOWUP flip, the whole-diff review, and the g6 fmt discipline are all present and assigned. The open execution-items (§3) are genuine impl-detail (the SPEC pins the INVARIANT for each). **This plan is cleared to BEGIN implementation (P1).**

Three Minors below are non-blocking polish (fold opportunistically at execution, NOT gate-blocking).

---

## Citation audit (§0) — ALL 23 CONFIRMED at the plan base (src unchanged since v0.60.0)

I re-grepped every §0 citation against the working tree (verified identical to the SPEC base `82e58674`). The implementer edits against these — all land correctly:

| §0 citation | Confirmed |
|---|---|
| `complete_multisig_template` `restore.rs:1416` | ✓ `pub(crate) fn complete_multisig_template<E: Write>` |
| `--own-account-max` refuse gate `:1434` | ✓ `if ctx.own_account_max.is_some() {` → `bad("…not supported yet…")` |
| supply gates `:1626`/`:1635` | ✓ `if pool.len() < n` / `if pool.len() > n` (both `ModeViolation`) |
| `realized_s = perm_count_u128(n,n)` `:1661` | ✓ `let realized_s = perm_count_u128(n, n).ok_or_else(…)?` |
| id/addr mode select `:1665-1666` | ✓ `let id_search = ctx.expect_wallet_id.is_some();` / `let addr_search = ctx.search_address.is_some();` |
| `reject_duplicate_keys` whole-pool `:1648` on `c.key65` `:1647` | ✓ `:1647` `pool.iter().map(\|c\| c.key65)`; `:1648` `ps::reject_duplicate_keys(&pool_key_blobs)` |
| `sorted_shape` binding `:1676` | ✓ `let sorted_shape = crate::synthesize::is_order_independent_shape(&d.tree);` |
| evaluator-filter `:1739` `assignment != identity → false` | ✓ `if sorted_shape && !assignment.iter().enumerate().all(\|(i,&v)\| i==v) { return false; }` — INSIDE the address-search evaluator closure |
| `perm_count_u128` `:1882` | ✓ `fn perm_count_u128(pool: usize, n: usize) -> Option<u128>` (`None` on `pool<n` + `checked_mul`) |
| `--account` `Vec<u32>` `default_value="0"` `:106` | ✓ `#[arg(long, value_delimiter = ',', default_value = "0")] pub account: Vec<u32>` |
| `conflicts_with` precedent `:86` | ✓ `#[arg(long, conflicts_with = "passphrase")] pub passphrase_stdin: bool` (live precedent for the idiom) |
| `run_multisig_template_completion` `:1321` | ✓ `fn run_multisig_template_completion<R: Read, W: Write, E: Write>` |
| `verify_multisig_template`→core `verify_bundle.rs:808`→`:874` | ✓ `:808` `fn verify_multisig_template<W: Write, E: Write>`; `:874` `complete_multisig_template(d, &ctx, stderr)?` |
| `own_account_max: None` `:865` | ✓ hardcoded `own_account_max: None,` in the ctx literal |
| scalar `--account` `:64` | ✓ `#[arg(long, default_value = "0")] pub account: u32` (scalar, NOT `Vec`) |
| `search` full-scan + 2nd-match short-circuit `permutation_search.rs:551`/`:623-624` | ✓ `:551` `pub fn search<E: CandidateEvaluator>`; `:623` `global_matches.fetch_add(1, …)`, `:624` `if prior + 1 >= 2 {` → stop. Doc `:530-548` ("does NOT stop at the first match") confirms BOTH modes full-scan. |
| `unrank_permutation` `:494` | ✓ `fn unrank_permutation(mut rank: u128, n: usize)` builds `elems = (0..n)` |
| `factorial` `:481`; `total_candidates` `:509` | ✓ both confirmed |
| `validate_prefix_strength` `:342`; `required_prefix_bytes` `:322` | ✓ both confirmed |
| pinned `prefix_ladder_own_account_max_subset_space` `:740` | ✓ body computes `S = P((11−4)+K, 11) = P(7+K, 11)` — confirming P6's "update to `S_own`" is real work |
| `is_order_independent_shape` `synthesize.rs:335` | ✓ `pub(crate) fn is_order_independent_shape(tree: &md_codec::tree::Node) -> bool` |
| flip/keep tests `cli_restore_md1_template_multisig.rs:677`/`:715`/`:635` | ✓ all three at the exact lines (bodies audited below); file has **27** `#[test]` fns (`grep -c '#\[test\]'` = 27) |

**No structurally-wrong citation.** The plan's bare-basename convention (`restore.rs:NNNN`) is unambiguous; §0 also gives the full `crates/mnemonic-toolkit/src/cmd/` paths. The "src unchanged since v0.60.0" premise (plan §0 / line 4) is verified true, so no citation decay between SPEC base and plan base.

---

## Phase-decomposition soundness

- **Deliverables + RED-first anchor + per-phase R0:** each phase (P1–P6) has a concrete deliverable, a TDD-RED list, and an explicit per-phase R0 focus. ✓
- **Dependency order is correct:** P1 (engine: `unrank_kperm` + own-anchored generator + cardinality + `early_exit` knob) is consumed by P2 (own-only) and P3 (opt-in); P4 (verify-bundle) wires the same core after P2/P3 exist; P5 differential after the feature is live; P6 ships last. No forward dependency. ✓
- **The `early_exit` knob is correctly in P1, not P2:** P1 impl (f) adds the engine-level `early_exit: bool` (or `SearchMode::Address { early_exit }`) with a P1 unit anchor (`early_exit=false` reproduces today's full-scan); P2 merely PASSES `true` (over-supply address) / `false` (exact) at the use-site. The knob (engine surface) and its use (caller policy) are split across the right phases. ✓ This matches the SPEC §4.4 contract (invariant in §4.4; API SHAPE is plan-level).
- **Nothing belongs in an earlier phase:** the `conflicts_with` clap attr is correctly P2 (it's a restore-arg-struct edit, consumed nowhere in P1). The verify-bundle flags are correctly P4 (a separate arg struct). No mis-placement.
- **No SPEC contract item unassigned** — see the matrix.

## SPEC-contract → phase coverage matrix (the key check — NO unassigned item)

| SPEC contract item | SPEC § (+ R0 finding) | Plan home (impl + TDD) |
|---|---|---|
| `unrank_kperm` injective k-perm, count `P(pool,n)` | §4.1 | P1 (a); TDD exhaustive small-(pool,n) bijection ✓ |
| Own-anchored composed-rank generator (combo×perm, NO cosigner-drop) | §4.1 | P1 (b); TDD `count==C(K_own,j)·N!`, no-cosigner-drop ✓ |
| Sorted enumeration-side (drop `perm_rank`, `C(K_own,j)` identity) — NOT `:1739` verbatim | §3/§4.1 (I-1) | P1 (c); TDD "sorted variant (`C(K_own,j)`, identity)" ✓ |
| Opt-in stratified generator over `S_opt` | §4.3 | P1 (d) + P3; TDD stratified brute-force bijection (m-2) ✓ |
| `c_choose` overflow-checked + `S_own`/`S_opt` `checked_mul` | §4.1 (m-3) | P1 (e); TDD `c_choose` overflow→None ✓ |
| `early_exit` knob, default = v0.60.0 full-scan BYTE-UNCHANGED | §4.4 (I-5) | P1 (f); TDD `early_exit=false` reproduces today's outcome ✓ |
| Remove `:1434` refuse + `conflicts_with="account"` | §2 (I-4) | P2; TDD `--own-account-max 5` ALONE passes + `--account+--own-account-max` refuses ✓ |
| §5a premise gates (4 rows, all fail-safe) | §5a | P2; TDD "each §5a premise refusal fail-safe" ✓ |
| `realized_s = S_own`/`S_own_sorted` (NOT `n!`, NOT `P(pool,N)`) | §3 | P2 impl + R0 focus `realized_s == enumerated` ✓ |
| Early-exit gate IFF over-supply AND Address; exact-path byte-invariance | §4.4 (I-5) | P2 (early_exit=true over-supply / false exact) + NEW exact-path address byte-guard (I-5) ✓ |
| §6 hard ceilings (`K_own≤256`, `S_MAX=1e15`) before calibration | §6 | P2 + P3 impl ✓ |
| `@N=`⊕subset-search mutex | §2 | P2 impl + P3 TDD refuse ✓ |
| Flipped/kept tests (`:677` rewrite, `:715` update, `:635` byte-green) | §7 (I-2) | §0 + P2 TDD, honest split ✓ |
| `--search-cosigner-subset` opt-in (default OFF = own-only) | §2/§4.3 | P3 impl + TDD ✓ |
| verify-bundle BOTH new flags + `--account` stays scalar | §2/§9 (I-3) | P4 impl + P6 schema_mirror ✓ |
| Differential vs INDEPENDENT golden (non-vacuous) | §7-P5 | P5 ✓ |
| GUI schema_mirror (restore +1 / verify-bundle +2) | §9 | P6 ✓ |
| Manual rows (restore refuse→search + vb ×2 + subset section) | §9 | P6 ✓ |
| 7 version sites (Cargo.toml, ×2 READMEs, install.sh, fuzz/Cargo.lock, Cargo.lock, CHANGELOG) | release ritual | P6 ✓ (all 7 verified to exist) |
| Pinned `prefix_ladder_…` → `S_own` | §3 supersession | P6 ✓ |
| FOLLOWUP flip in shipping commit | tracking discipline | P6 ✓ (FOLLOWUP confirmed `open` at `FOLLOWUPS.md:49`) |
| m-4 id+addr precedence (`:1665-1666`, never both) | §2 (m-4) | Inherited via P2 "drive `search` with id/addr mode select"; see Minor m-c |
| Distinct-keys floor LOAD-BEARING (own-as-cosigner refuse) | §5 | Reused unchanged (`:1648`, live whole-pool); P2 §5a "own-as-cosigner → distinct-keys floor" TDD ✓ |

**No SPEC funds-safety contract item is unassigned.** The distinct-keys floor is correctly treated as a live invariant (mechanically unchanged from v0.60.0 — it already operates whole-pool at `:1648`) that the subset path now leans on, with a §5a premise test, not as new code.

---

## Test-flip honesty (P2 / §0) — HONEST, source-verified

I read all three test bodies at HEAD:

- **`:635` `multi_account_own_resolves_both_slots`** — `--account 0,1`, no `--own-account-max`, **exact pool** (`pool.len()==n`), asserts golden addresses. The plan's "stays byte-GREEN" is correct: it routes the byte-identical exact path. ✓
- **`:677` `own_account_max_flag_refuses_with_actionable_message`** — currently asserts `--own-account-max 3` `.failure()` + stderr contains `own-account-max` AND `--account` AND NOT `no match`. The plan's "REWRITE (RED-first) to assert search-COMPLETES" is HONEST — this pinned test FLIPS by design (it asserts the exact refuse-behavior P2 removes). ✓
- **`:715` `pool_larger_than_slots_refuses_with_actionable_message`** — `--account 0` + cosigner-B + extra-outsider-C ⇒ pool 3 > n 2; asserts refusal with message matching `--account || "more keys" || "over-supply" || "exactly"` and NOT `no match`. Under §5a this becomes the own-only over-supplied-cosigners case (`M'>M` → "REFUSE up front: own-only needs exact cosigners; use `--search-cosigner-subset`"). The plan's "UPDATE message (refusal preserved, gate+message changed)" is HONEST — the refusal outcome survives but the new message wording may not satisfy the current OR-assertion, so the assertion needs updating. The plan correctly flags this as UPDATE not keep-green. ✓

The "27 `#[test]`" count is exact (`grep -c` = 27). The plan does NOT repeat the SPEC's superseded imprecise "25." No dishonest backward-compat claim.

---

## Locksteps / SemVer / ship (P6) — complete

- **7 version sites** — all listed (Cargo.toml, BOTH READMEs, install.sh self-pin, fuzz/Cargo.lock, Cargo.lock, CHANGELOG) and all verified present: `Cargo.toml:3` `version="0.60.0"`; both READMEs reference `0.60.0`; `scripts/install.sh:32` self-pins `mnemonic-toolkit-v0.60.0`; `fuzz/Cargo.lock:575`; `Cargo.lock:727`; `CHANGELOG.md` present. (The plan abbreviates "install.sh" — the file lives at `scripts/install.sh`; this is a known release-ritual label, not a §0 grep citation, so not a structural-citation finding. See Minor m-a.) ✓
- **GUI schema_mirror scope** — restore +`--search-cosigner-subset` (the only NEW name there; `--own-account-max` pre-exists on restore ⇒ no delta); verify-bundle +`--own-account-max` +`--search-cosigner-subset` (BOTH new names there). Matches the SPEC §9 I-3 correction exactly. ✓
- **Stale-`$PATH`-binary gotcha** — explicitly noted ("NOT the stale `$PATH` `mnemonic` — use `MNEMONIC_BIN`, the [[GUI gotcha]]"). ✓
- **Pinned `prefix_ladder_own_account_max_subset_space:740` → `S_own`** — assigned to P6 (SPEC §3 supersession; the live body computes `P(7+K,11)`, confirming real update work). ✓
- **FOLLOWUP flip in the shipping commit** — assigned (P6 housekeeping; FOLLOWUP `template-multisig-own-account-range-subset-search` confirmed `open` at `FOLLOWUPS.md:49`, so the flip is genuine). ✓
- **Whole-diff adversarial exec review before tag** — present (P6, "Mandatory post-impl whole-diff adversarial exec review … BEFORE tag"). Matches CLAUDE.md post-implementation mandatory review. ✓
- **`cargo fmt` g6** — `cargo +1.95.0 fmt -p mnemonic-toolkit` (NOT `--all`) then `git checkout -- …/mlock.rs`; "do NOT cargo fmt the GUI." Matches the g6 fmt-exemption memory. ✓
- **SemVer** — toolkit MINOR `0.60.0→0.61.0` (re-enables a behavior + 1 new flag, both subcommands), md/mk NO-BUMP, GUI MINOR paired. Correct per SPEC §9. ✓

---

## Open-items (§3) check — all genuine impl-detail, no hidden contract gap

1. **Re-grep citations vs execution base SHA** — standard decay hygiene; safe. ✓
2. **`early_exit` API shape** (`search` param vs `SearchMode` variant) — the INVARIANT is the SPEC §4.4 hard contract (IFF over-supply AND Address; exact + id/prefix byte-unchanged); only the SHAPE is deferred. Safe. ✓
3. **Opt-in stratified-unrank offset arithmetic** — the COUNT (`Σ_j C(K_own,j)·C(M_sup,N−j)·N!`), the composition (CNS-unrank own + CNS-unrank cosigner + `unrank_permutation`), and the brute-force-reference floor (m-2) are all in the SPEC; only the concrete rank→stratum offset layout is deferred, and it is brute-force-tested. Safe (the SPEC R0-r2 explicitly cleared this deferral). ✓
4. **`--own-slots <j>` pin flag** — deferred; the inferred `j_min..j_max` range is SPEC-proven safe (§4.3). Adding the pin only if the inferred range proves too loose. Safe. ✓
5. **Exact refusal-message wording** — UX detail; the §5a/§6/mutex INVARIANTS (what refuses, fail-safe) are pinned; only the prose is deferred. Safe. ✓

None hides a funds-safety contract decision.

---

## MINOR (non-blocking — fold opportunistically at execution; NOT gate-blocking)

- **m-a — `install.sh` path.** The plan lists "install.sh self-pin" among the 7 version sites; the file is actually at `scripts/install.sh` (line 32 pins `mnemonic-toolkit-v0.60.0`). Harmless abbreviation (a release-ritual label, not a §0 source citation), but at execution P6 should edit `scripts/install.sh:32`. No structural impact.
- **m-b — sorted-OPT-IN variant not explicitly enumerated in P1/P3.** P1 (c) covers the sorted OWN-ONLY collapse (`C(K_own,j)`, drop `perm_rank`); the SPEC §4.3 also specifies a sorted-OPT-IN collapse (drop `·N!` per stratum → `Σ_j C(K_own,j)·C(M_sup,N−j)`). The plan's opt-in deliverable (P1 (d) / P3) does not call this out explicitly. It is subsumed under the same enumeration-side mechanism (stratified generator with `perm_rank` dropped) and the §6 ceiling + brute-force bijection floor would catch any mis-sizing — so it is not a contract gap, but the execution author should keep the sorted-opt-in `realized_s` collapse in the P3 brute-force-reference test set (the SPEC §4.3 parenthetical). One sentence at P3.
- **m-c — m-4 (id+addr precedence) has no dedicated TDD line.** The plan inherits the v0.60.0 `id_search`/`addr_search` mutual-decision (`:1665-1666`) via P2's "drive `search`," and the SPEC §2 (m-4) pins "never BOTH in one search." This is correct (the over-supply path inherits the existing precedence), but P2 has no explicit "over-supply + `--expect-wallet-id` + `--search-address` → one mode wins" assertion. Optional: add a one-line P2 TDD pinning that the over-supply path inherits the v0.60.0 precedence (cheap regression guard; the live precedence already excludes running both).

---

## Closing verdict

**GREEN — 0 Critical, 0 Important.** All 23 §0 citations are structurally correct against the live src tree (verified unchanged since v0.60.0). The phase decomposition is sound and correctly ordered (engine → consumers → verify-bundle → differential → ship), with the `early_exit` knob in the right phase. Every SPEC funds-safety contract item — the own-anchored bijection, enumerated≡counted, the distinct-keys floor, `realized_s = S_own`/`S_own_sorted`/`S_opt` (never `n!`/`P(pool,N)`), the sorted enumeration-side mechanism (not `:1739` verbatim), the early-exit IFF-gate + exact-path byte-invariance guard, the §5a fail-safe gates, the `conflicts_with` mutex + alone-passes guard, the §6 ceilings + `c_choose` overflow, the prefix-strength sizing — has a phase + RED-first TDD home (no unassigned item). The test-flip backward-compat plan is honest and source-verified against the actual `:635`/`:677`/`:715` bodies. The 7 version sites, schema_mirror scope (restore +1 / vb +2), `$PATH`-binary gotcha, pinned-test→`S_own` update, FOLLOWUP flip, whole-diff review, and g6 fmt are all complete and assigned. The §3 open-items are genuine impl-detail (each INVARIANT pinned in the SPEC). The three Minors are non-blocking polish.

No rubber-stamp: this verdict rests on a re-grep of all 23 citations at the plan base, a read of all three flip/keep test bodies, a read of the live `search` full-scan/2nd-match logic, verification of all 7 version sites + the FOLLOWUP `open` status, and the full contract→phase matrix. **The plan is cleared to BEGIN implementation (P1).**
