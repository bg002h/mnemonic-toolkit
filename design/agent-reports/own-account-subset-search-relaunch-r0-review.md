> Reviewer: opus architect (RELAUNCH RE-R0) · 2026-06-22 · brainstorm/SPEC/plan `design/{BRAINSTORM,SPEC,IMPLEMENTATION_PLAN}_own_account_subset_search_2026-06-20.md` @ branch `feature/own-account-subset-search` HEAD `764d586b` · re-grounded onto final master **v0.69.1 `5e7b9dec`**. Scope: confirm the re-grounding FOLDS + the one P0 re-decision preserve the prior R0-GREEN funds-safety contract (brainstorm 2r / SPEC 2r / plan 1r, all 0C/0I, base `82e58674` v0.60.0) and are internally consistent. NOT a re-litigation of the settled model/math/engine/floors. All citations re-grepped LIVE against `origin/master` `5e7b9dec`; prior R0 reviews (spec-r0-r2, plan-r0-r1) read in full as the contract.

**Verdict: GREEN — 0 Critical, 0 Important.**

The re-grounding is faithful. Every refreshed citation lands on the live `5e7b9dec` line (including the corrected `perm_count_u128` FILE-fix). The L8/L9/H1 rewords match live source and REFINE rather than break the carried invariants. The P0 hard-prerequisite re-decision is sound — the over-supply path genuinely multiplies the un-scrubbed `Xpriv` residue, the substrate (`derive.rs`/`derive_slot.rs`) is verified stable at 0 commits, and a new fn + new newtype is genuinely additive. The design is internally consistent: no leftover "consumed-only / not authored / bare-field fallback" survives to contradict P0. Version 0.70.0 is the next-free MINOR; the GUI/manual lockstep surface and the fuzz-build resolution are confirmed live. No R0-GREEN funds-safety contract was weakened. One MINOR (non-blocking, P6-housekeeping). **Implementation may begin at P0.**

---

## 1. Citation-refresh spot-check — ALL CONFIRMED at `5e7b9dec`

Re-grepped against `git show origin/master:<path>`:

| Cite | Claimed | Live `5e7b9dec` | Status |
|---|---|---|---|
| `complete_multisig_template` | `restore.rs:1421` | `pub(crate) fn complete_multisig_template<E: Write>(` | ✓ |
| `--own-account-max` refuse | `restore.rs:1439` | `if ctx.own_account_max.is_some() {` | ✓ |
| pool supply gates | `restore.rs:1687`/`1696` | `if pool.len() < n {` / `if pool.len() > n {` | ✓ |
| `realized_s = perm_count_u128(n,n)` | `restore.rs:1722-1723` | `let realized_s =` / `perm_count_u128(n, n).ok_or_else(…)?` | ✓ |
| evaluator-filter | `restore.rs:1800` | `if sorted_shape && !assignment.iter().enumerate().all(\|(i,&v)\| i==v) {` | ✓ |
| **`perm_count_u128` DEF** | **`restore.rs:1943`** (NOT `permutation_search.rs:1882`) | `fn perm_count_u128(pool: usize, n: usize) -> Option<u128> {` — and grep for `perm_count_u128` in `permutation_search.rs` returns **NONE** | ✓ **mislabel FIXED** |
| `verify_multisig_template` | `verify_bundle.rs:808` | `fn verify_multisig_template<W: Write, E: Write>(` | ✓ |
| `own_account_max: None` | `verify_bundle.rs:865` | `own_account_max: None,` | ✓ |
| `complete_multisig_template` call (in verify) | `verify_bundle.rs:874` | `let outcome = complete_multisig_template(d, &ctx, stderr)?;` | ✓ |
| `--account` scalar (verify) | `verify_bundle.rs:64` | `pub account: u32,` | ✓ |
| `is_order_independent_shape` | `synthesize.rs:335` | `pub(crate) fn is_order_independent_shape(…)` | ✓ |
| test fns | `…multisig.rs:635`/`677`/`715` | `multi_account_own_resolves_both_slots` / `own_account_max_flag_refuses_…` / `pool_larger_than_slots_refuses_…` | ✓ |
| `#[test]` count | 27 | `grep -c` = **27** | ✓ |
| supporting | `restore.rs:1326` run-completion / `:86` conflicts_with / `:106` Vec account / `:133-134` own_account_max clap / `:1709` reject_duplicate_keys / `:1737` sorted_shape / `:1726-1727` id/addr | all live-confirmed | ✓ |

The `perm_count_u128` STRUCTURALLY-WRONG-FILE cite (the prior recon's one ❌) is corrected to `restore.rs:1943` in **all three docs** (SPEC §4.1 + §10.2-area, plan §0 + open-item 1, brainstorm §0 + §3). No still-wrong citation remains. The `git diff origin/master..HEAD -- crates/…/src crates/…/tests` is EMPTY (branch lead is design-docs-only; now 14 ahead/0 behind — still trivially rebasable, zero code conflicts).

## 2. L8 / L9 / H1 reword fidelity — CONFIRMED against live source

**L8 → C1 (SPEC §5, plan P2 R0-focus).** The fresh per-slot origin BUILD includes the network coin-type substitution. Live: `restore.rs:~1598-1608` patches `comps[1] = ChildNumber::from_hardened_idx(network.coin_type())` inside the all-own/no-`--cosigner`/no-`--origin` fallback closure, with a comment that explicitly says it MIRRORS the bundle emitter (coin 0' mainnet identity, coin 1' testnet/signet/regtest, else silent NO-MATCH). I traced the own-pool assembly: the own-candidate loop (`for &acct in own_accounts`, `restore.rs:~1634`) resolves `origin` via the `--origin` → cosigner-family → `own_origin_from_family(&fb, acct)` fallback chain (the same closure carrying the L8 patch), then derives and pushes `xpub_to_65(&acct_key.account_xpub)`. The `--own-account-max` over-supply path widens `own_accounts` and flows through this IDENTICAL loop ⇒ **all K_own own candidates derive through the same fresh-origin closure on every network.** This REFINES C1 ("built fresh" now subsumes the coin-type patch) and does NOT break it — the carried origin is still never loaded. ✓

**L9 3-gate prologue → P2 (SPEC §5a, plan P2).** Live `complete_multisig_template` prologue: I-1 own-account-max refuse (`:1439`) → **gate (a) hardened-use-site** `if md_codec::to_miniscript::has_hardened_use_site(d) { …ModeViolation }` (`:1465`) → **gate (b) #26 taproot-override** `if taproot_override_card(d) && !restorable_taproot_override_card(d) {` (`:1472`). Both gates exist at the cited lines. The compose-with claim is correct: the subset-search domain is a NAMED, non-hardened, non-taproot-override multisig template — for it both ModeViolation gates pass through transparently (the in-source comment at `:1461-1464` even states "Named multisig templates are non-hardened today and never reach the taproot-override leg"). P2 repurposes the I-1 gate; the two L9 gates are untouched; the subset pool-build inserts AFTER all three; the §5a premise-violation table (which fires during pool assembly, after the prologue) does not collide. ✓

**H1 verify-rationale (SPEC §4.6).** `verify_multisig_template` (`:808`) re-runs the shared `complete_multisig_template` (calls it at `:874`) — THAT is the verify==restore-parity basis (structural reconstruction), and that fn is byte-identical base→`5e7b9dec` (the recon's SHA/diff-empty claim; src diff is empty). H1 is genuinely ELSEWHERE: the cycle-1 H1 widening lives in `emit_multisig_checks` on the `md1_xpub_match` checks[] path (`verify_bundle.rs:2759-2825`, with the `use_site_path_overrides ==` compare at `:2802-2803` and the H1 comment at `:2759`/`:4019`) — a different, seed-derived card-multiset compare path, NOT `verify_multisig_template`. The reworded rationale ("re-runs the completion engine," NOT "shares the H1 comparator") is correct. ✓

## 3. P0 HARD-PREREQUISITE re-decision — the heart of this re-R0 — SOUND

**(a) Reasoning is sound — the bare-field fallback IS a bar-violation.** Live `derive_slot::derive_bip32_from_entropy_at_path` returns `Result<DerivedAccount, …>` (`:49`/`:71`), and `DerivedAccount` (`derive.rs:23`) carries `pub account_xpriv: Xpriv` (`:27`) — a bare, un-scrubbed field that drops without erase (the entropy is `Zeroizing` but the derived account `Xpriv`, equal spending authority, is not). The current own-pool loop calls this and discards everything but `account_xpub` — so over-supplying `K_own` candidates holds `K_own` un-scrubbed `Xpriv`s. The "multiplies the residue K_own-fold" claim is **literally true at live source.** Using the bare-field path for the over-supply loop would ship that multiplied residue ⇒ a genuine violation of the now-first-class hygiene bar. The fallback is correctly REJECTED.

**(b) "additive new fn + newtype, no conflict now substrate is stable" is CORRECT.** `derive.rs` and `derive_slot.rs` are both verified at **0 commits** since base `82e58674` (the churn that drove the architect's original full-decouple is genuinely gone). `ScrubbedXpriv` and `derive_account_xpub_only` do NOT exist yet (grep returns NONE in both files). The `account_xpriv` field has ~7 use sites across 4 files (convert.rs/restore.rs/derive.rs/derive_slot.rs, 8 grep hits = field def + uses) — the broader lift touches a PUBLIC field + `into_parts` + those consumers. A NEW fn + a NEW move-only newtype is genuinely additive: it neither changes the `DerivedAccount.account_xpriv` field nor any of the ~7 sites, so it cannot conflict with the deferred 7-site lift. ✓

**(c) P0 is a clean PREREQUISITE.** P1/P2 depend on it (P2 consumes `derive_account_xpub_only`); it is a small, architect-validated (2 passes, recorded in the filed FOLLOWUP), bounded addition. The `ScrubbedXpriv` contract is carried verbatim into SPEC §4.5 + plan P0: move-only by E0184 (`Copy`+`Drop` mutually exclusive), `Drop`→`private_key.non_secure_erase()` + volatile `chain_code` zero-write, no-escape-hatch API (`&self` accessors + `xpub(&self,&secp)->Xpub`; NO `into_inner`/`Deref<Xpriv>`/`Clone`/`Copy`/pub-field), compile-time `assert_not_impl_any!(ScrubbedXpriv: Copy, Clone)`, the byte-identical-derived-output regression test, and the tracked `rust-bitcoin-xpriv-zeroize-upstream` best-effort caveat. ✓

**(d) The broader 7-site lift is correctly OUT of scope.** SPEC §4.5 / plan P0 / brainstorm §3 all explicitly keep `DerivedAccount.account_xpriv → ScrubbedXpriv` across the ~7 sites as the filed FOLLOWUP. ✓

**(e) The filed FOLLOWUP slug needs narrowing — MINOR (P6 housekeeping, NOT a blocker).** `FOLLOWUPS.md:4515` `derive-slot-account-xpriv-scrub-confinement` Origin line still reads "subset-search CONSUMES `derive_account_xpub_only` as a 1-line dependency; it does NOT author this" — which the relaunch re-decision now contradicts (P0 may author the minimal helper). SPEC §8 P6 captures the narrow ("FOLLOWUP flip (minimal helper delivered → narrow the slug to the 7-site lift)"); however the PLAN §1-P6 housekeeping line (plan:56) only flips `template-multisig-own-account-range-subset-search` → RESOLVED and does NOT name narrowing the `derive-slot-account-xpriv-scrub-confinement` slug. See MINOR m-1. Not gate-blocking — the slug is still `open` and accurate today (the helper is not yet authored); it must be narrowed in the P0 (or P6) shipping commit if P0 takes the author-branch.

## 4. SemVer + version + lockstep + fuzz — CONFIRMED

- **0.70.0 is next-free MINOR.** `git tag` shows v0.61.0/0.62.x/0.63.0/0.64.0/0.65.x/0.66.0/0.67.0/0.68.0/0.69.0/0.69.1 all burned; NO 0.70.x tag. CHANGELOG tops at `[0.69.1] — 2026-06-22`. Master code = 0.69.1 (Cargo.toml:3, install.sh:32 self-pin `mnemonic-toolkit-v0.69.1`, Cargo.lock:727, fuzz/Cargo.lock:575). New clap surface (`--search-cosigner-subset` on restore; both names on verify-bundle) ⇒ MINOR. md/mk NO-BUMP. ✓
- **GUI schema-mirror surface accurate.** GUI `src/schema/mnemonic.rs` restore block ALREADY lists `--own-account-max` (`:713`, "reserved/refused this cycle" comment) ⇒ only `--search-cosigner-subset` is a NEW restore name; `--search-cosigner-subset` is ABSENT from the schema (correct). verify-bundle lists NEITHER flag ⇒ BOTH are new names there. Matches SPEC §9 exactly. Manual `41-mnemonic.md:938` `--own-account-max` "NOT SUPPORTED YET … refuses" row confirms the pending refuse→search edit; `:240` corroborates. ✓
- **Fuzz E0433 RESOLVED on master.** `taproot_override_classify.rs` exists with both predicates (`:32`/`:56`); `unrestorable_advisory.rs:116-117` now references `crate::taproot_override_classify::…`; `cmd::restore` re-exports at `:2675`. FOLLOWUPS.md:58 marks it ✓ RESOLVED (2026-06-20, NO-BUMP, pure relocation). P6 fuzz gate is genuinely unblocked. ✓

## 5. No regression to the GREEN contract — CONFIRMED

All R0-GREEN funds-safety contracts are unchanged in substance (the reword did not alter them):
- **Own-anchored bijection / enumerated≡counted floor** — SPEC §3 FLOOR + §4.1/§4.3 generators + §7 brute-force-reference tests carried verbatim.
- **`realized_s = S_own` (= `C(K_own,j)·N!`), NOT `n!`, NOT `P(pool,N)`** — SPEC §3 unchanged; the backward-compat collapse to `N!` at `K_own=j` (byte-identical to live `perm_count_u128(n,n)` at `:1722-1723`) preserved.
- **Distinct-keys floor** — live `reject_duplicate_keys` over whole-pool `key65` blobs BEFORE search (`restore.rs:1708-1709`), LOAD-BEARING for the subset collision axis. Unchanged.
- **Prefix-strength sized to `realized_s`** — `ceil((log2(S)+32)/8)` ladder; worked 10-byte margin unchanged.
- **`early_exit` IFF over-supply AND Address + exact-path byte-invariance** — SPEC §4.4 contract carried verbatim; the engine `search` (`permutation_search.rs:530-548`, 2nd-match short-circuit at `:623-624`) is SHA256-identical to base and full-scans both modes with 2nd-match ambiguity — the exact foundation the gate is built on. The reword did not touch the engine. Prefix-id NEVER gets early-exit.
- **§5a premise-violation fail-safes + `conflicts_with` mutex** — SPEC §5a table preserved (under-supply→NO-MATCH; over-supply-cosigners→refuse-up-front; own-as-cosigner→distinct-keys; multi-seed→out-of-scope); clap `conflicts_with="account"` with the I-4 "`--own-account-max 5` ALONE passes" regression guard intact.

## 6. Internal-consistency — CLEAN

Grep for `does NOT author` / `not authored` / `fully decoupled` / `bare-field fallback` / `soft consume-if-landed` across all three docs returns ONLY occurrences inside the NEW P0 re-decision text — every instance is the re-decision REJECTING the old stance ("the bare-field fallback is NOT acceptable / is rejected"; "supersedes the earlier 'fully decoupled, not authored'"), never a surviving contradiction. Brainstorm §3 (`:59`) + header (`:5`), SPEC §4.5 (`:48-51`) + §8 (`:84`) + header (`:6`), and plan P0 (`:20-24`) + §3-open-item-6 (`:70`) + header (`:7`) ALL agree: P0 HARD PREREQUISITE, consume-or-author-minimal, 7-site lift stays the FOLLOWUP. No drift.

---

## MINOR (non-blocking — fold opportunistically; NOT gate-blocking)

- **m-1 — plan P6 housekeeping does not name narrowing `derive-slot-account-xpriv-scrub-confinement`.** SPEC §8 P6 says "narrow the slug to the 7-site lift" (correct), but the plan's P6 housekeeping line (plan:56) flips only `template-multisig-own-account-range-subset-search` and omits the narrow of `derive-slot-account-xpriv-scrub-confinement`. The FOLLOWUP Origin line at `FOLLOWUPS.md:4515` ("does NOT author this") must be narrowed-to-the-7-site-lift in whichever commit P0 authors the minimal helper. One sentence at plan P0/P6. (The slug is `open` and accurate today, so no contract impact now.)

---

## Closing verdict

**GREEN — 0 Critical, 0 Important.** The relaunch folds re-ground the R0-GREEN design onto `5e7b9dec` faithfully: all spot-checked citations are live (the lone prior structural error — `perm_count_u128`'s file — is fixed to `restore.rs:1943`); L8 refines C1 (own candidates verified to flow through the same coin-type-patched fresh-origin closure on every network) without loading the carried origin; the L9 3-gate prologue and its compose-with claim match live source; the H1 rationale is correct (verify re-runs the engine; H1 is in `emit_multisig_checks`, elsewhere). The P0 hard-prerequisite re-decision is sound on every axis — the over-supply path provably multiplies the un-scrubbed `Xpriv` residue (live `DerivedAccount.account_xpriv: Xpriv`), the substrate is verified stable (0 commits), a new fn + newtype is genuinely additive, the `ScrubbedXpriv` contract is carried verbatim, and the 7-site lift stays deferred. Version 0.70.0 is next-free; the GUI/manual lockstep surface and the fuzz resolution are confirmed live. No GREEN funds-safety contract was weakened, and the design is internally consistent (no orphaned decouple language). One MINOR (narrow the `derive-slot-account-xpriv-scrub-confinement` slug at ship — P6 housekeeping). **The hard gate is satisfied: implementation may begin at P0.**

No rubber-stamp: this verdict rests on re-grepping every cited line at `5e7b9dec`, tracing the own-pool derivation loop through the L8 closure, reading the live L9 gates + the `DerivedAccount.account_xpriv` field + the `emit_multisig_checks` H1 path, confirming `derive.rs`/`derive_slot.rs` 0-commit stability and `ScrubbedXpriv` non-existence, verifying the GUI schema + manual rows + the live `taproot_override_classify.rs` fuzz fix, and a cross-doc grep for orphaned decouple language.
