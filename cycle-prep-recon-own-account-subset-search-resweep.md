# Pre-brainstorm recon — own-account subset-search RE-SWEEP

**Date:** 2026-06-21 · **Mode:** READ-ONLY recon (no code edits, no rebase).
**Paused cycle design base SHA:** `82e58674` (v0.60.0). **Current `origin/master`:** `ddabf5e3` (v0.67.0).
**Branch:** `feature/own-account-subset-search` — 9 commits ahead of `origin/master` (ALL `design/` docs; `crates/mnemonic-toolkit/src/` UNTOUCHED on our lead), 66 behind in symmetric-difference terms (the "62 commits" figure; the true `82e58674..origin/master` count is **66**, not 62).

---

## 1. Sync state

| Item | Value |
|---|---|
| `origin/master` short SHA | `ddabf5e3` |
| `origin/master` toolkit version | **0.67.0** (NOT 0.66.0 — the cycle-14/L22 secret-hygiene tail ALREADY shipped @`ddabf5e3`) |
| Our branch ahead | 9 commits, **all `design/`** (brainstorm/spec/plan/followup); `src/` clean |
| Our branch behind | 66 commits |
| Design-base → master commit count | **66** (`git rev-list --count 82e58674..origin/master`) |

### The 66 commits — cycles that touched OUR four surface files

| File | # commits | Cycles / findings (subject) |
|---|---|---|
| `permutation_search.rs` | **0** | **ENGINE 100% CLEAN — zero change.** P1 lands on identical ground. |
| `synthesize.rs` | 1 | `53787cbb` cycle-2 **H8** — thread `run_language` into template ms1 emit (in `synthesize_template_descriptor` + tests; NOT near `is_order_independent_shape`) |
| `cmd/restore.rs` | 2 | `8c145f19` cycle-13b **L8+L9** (multisig-template coin-type origin fix + early hardened/taproot refusals); `db0bf583` cycle-14 **L22** (stdin-secret locals → `Zeroizing`) |
| `cmd/verify_bundle.rs` | 4 | `4ed3bbe9` cycle-1 **H1** (md1_xpub_match → structural compare); `c4b46624` cycle-1 **H12** (descriptor-mode taproot 3' origin); `40c4db74` cycle-11b **L24** (descriptor-mode OOB → typed); `7daf8c69` cycle-14 **L22** (`SlotInput.value: String → SecretString`) |

The 66 commits are the constellation **bug-hunt program** (cycles 1–14, 57 of 58 findings fixed; the lone open is L16 WON'T-FIX). Version chain: v0.60.0 → 0.61.0 (cycle-1) → 0.62.0/.1 → 0.63.0 → 0.64.0 → 0.65.0/.1/.2 → 0.66.0 (cycle-13) → **0.67.0 (cycle-14, current)**.

---

## 2. Per-citation drift table (re-grepped vs `origin/master` `ddabf5e3`)

Legend: ✅ ACCURATE · ⚠️ DRIFTED-by-N (symbol intact, new line) · ❌ STRUCTURALLY-WRONG.

### `cmd/restore.rs` (all line shifts driven by the L8/L9 +57-line insertion + L22 Zeroizing wraps)

| Symbol / citation | Old (`82e58674`) | New (`ddabf5e3`) | Status |
|---|---|---|---|
| `fn complete_multisig_template` | :1416 | **:1421** | ⚠️ +5 |
| `--own-account-max` refuse gate (`ctx.own_account_max.is_some()`) | :1434 | **:1439** | ⚠️ +5 |
| pool.len supply gates | :1626 / :1635 | **:1687** (`< n`) / **:1696** (`> n`) | ⚠️ +61 |
| `realized_s = perm_count_u128(n,n)` | :1661 | **:1722** | ⚠️ +61 |
| id/addr mode select (`id_search`/`addr_search`) | :1665-1666 | **:1726-1727** | ⚠️ +61 |
| `reject_duplicate_keys` whole-pool | :1648 | **:1709** | ⚠️ +61 |
| `key65` field on CandidateKey | :1647 | **:1107** (struct def) / used :1708 | ⚠️ moved (def vs use) |
| `sorted_shape` binding | :1676 | **:1737** | ⚠️ +61 |
| evaluator-filter (`sorted_shape && assignment != identity → false`) | :1739 | **:1800** (`if sorted_shape && !assignment.iter()…all(i==v)`) | ⚠️ +61 — **mechanism byte-identical** |
| `validate_prefix_strength` call | :1700 | **:1761** | ⚠️ +61 |
| `perm_count_u128` helper | :1882 | **:1943** | ⚠️ +61 |
| `--account` `Vec<u32>` `default_value="0"` | :106 | **:106** (`pub account: Vec<u32>`) | ✅ |
| `conflicts_with` precedent | :86 | **:86** (`conflicts_with = "passphrase"`) | ✅ |
| `run_multisig_template_completion` | :1321 | **:1326** | ⚠️ +5 |
| `#[arg(long="own-account-max")]` clap def | (n/a) | **:133-134** (`Option<u32>`) | ✅ exists |

### `cmd/verify_bundle.rs`

| Symbol / citation | Old | New | Status |
|---|---|---|---|
| `fn verify_multisig_template` | :808 | **:808** | ✅ **byte-unchanged** |
| `complete_multisig_template` call (in verify_multisig_template) | :874 | **:874** | ✅ |
| `own_account_max: None` hardcoded | :865 | **:865** | ✅ |
| `--account` SCALAR `u32` `default_value="0"` | :64 | **:63-64** | ✅ |

**The `verify_multisig_template` template-completion path is 100% byte-identical** (808/865/874 all exact). The verify_bundle churn (+402 lines) is entirely in OTHER functions: `descriptor_mode_verify_run` (H12/L24/L22 at 1350–1638), `resolve_env_sentinels` (L22 at 1880), `emit_multisig_checks`/`emit_verify_checks` (H1 at 2715+).

### `permutation_search.rs` — ALL ✅ EXACT (zero file change)

| Symbol | Old | New | Status |
|---|---|---|---|
| `fn search` | :551 | :551 | ✅ |
| `fn unrank_permutation` | :494 | :494 | ✅ |
| `fn factorial` | :481 | :481 | ✅ |
| `fn total_candidates` | :509 | :509 | ✅ |
| `fn required_prefix_bytes` | :322 | :322 | ✅ |
| `fn validate_prefix_strength` | :342 | :342 | ✅ |
| pinned test `prefix_ladder_own_account_max_subset_space` | :740 | :740 | ✅ |
| full-scan/2nd-match doc | :530-548 | :528-548 | ✅ (intact) |

> Note: SPEC/plan cite a `perm_count_u128` at `permutation_search.rs:1882` in §0/§4.1 — that symbol lives in **restore.rs:1943** (a private fn), NOT in permutation_search.rs. `permutation_search.rs` exports `factorial`/`total_candidates`. (Pre-existing mislabel in the plan §0, not sweep-induced; mechanical.)

### `synthesize.rs`

| Symbol | Old | New | Status |
|---|---|---|---|
| `fn is_order_independent_shape` | :335 | :335 | ✅ **byte-identical (335-360 diff EMPTY)** |

### Tests to flip (`tests/cli_restore_md1_template_multisig.rs`) — FILE UNCHANGED

| Test | Old | New | Status |
|---|---|---|---|
| `own_account_max_flag_refuses_with_actionable_message` (REWRITE) | :677 | **:677** | ✅ |
| `pool_larger_than_slots_refuses_with_actionable_message` (UPDATE msg) | :715 | **:715** | ✅ |
| `multi_account_own_resolves_both_slots` (stays byte-GREEN) | :635 | **:635** | ✅ |
| `#[test]` count | 27 | **27** | ✅ |

**Drift verdict:** all restore.rs interior citations DRIFTED by a uniform +61 (below the L8/L9 insertion) or +5 (the clap-args block, above it). ZERO STRUCTURALLY-WRONG citations — every symbol still exists, all line shifts are mechanical re-greps. The math/test/engine surface (permutation_search, synthesize, the test file, verify_multisig_template) is byte-stable.

---

## 3. Design-impact assessment (the load-bearing part)

### 3.1 `complete_multisig_template` — TWO new early gates inserted into the exact region our design extends

The shared core gained a **+57-line block at the SAME gate position our P2 design edits** (right after the I-1 own-account-max refuse, before the `--cosigner` parse / pool assembly). Order at master:

```
restore.rs:1439  I-1 gate:  if ctx.own_account_max.is_some() → REFUSE   ← P2 REMOVES this
restore.rs:1465  L9 gate:   if has_hardened_use_site(d) → ModeViolation  ← NEW
restore.rs:1485  L9/#26:    if taproot_override_card(d) && !restorable_taproot_override_card(d) → ModeViolation  ← NEW
restore.rs:~1500 --cosigner parse → pool build → reject_duplicate_keys → pool.len gates → realized_s → search
```

- **Signature / `MultisigCompletionCtx` / mode-precedence:** UNCHANGED. `own_accounts: &[u32]`, `entropy`, `own_account_max: Option<u32>`, `expect_wallet_id`/`search_address` all intact. `id_search`/`addr_search` precedence (now :1726-1727) byte-identical.
- **The C1 carried-origin-never-loaded invariant + per-slot origin BUILD:** STILL HOLDS — but the BUILD gained an **L8 coin-type substitution** (restore.rs:1586-1608): in the all-own / no-`--cosigner` / no-`--origin` fallback, `canonical_origin(tree)` (mainnet coin 0') is patched to `network.coin_type()` at `comps[1]`. This is the ONLY origin source in the pure-own path our subset-search runs in. **Design-relevant:** the over-supplied own pool derives every own candidate through this same fresh-origin closure; the L8 patch is in the path, identity on mainnet, coin-1 on testnet/signet/regtest. Our design's "per-slot origins built fresh; carried origin never loaded (C1)" claim is preserved AND must now note the L8 coin-type substitution as part of "built fresh."
- **NEW gates our design must COMPOSE with (not conflict):** the L9 hardened-use-site refuse and the #26 taproot-override refuse now fire BEFORE pool assembly. For named-multisig templates (the subset-search domain) these are defense-in-depth (non-hardened, non-taproot-override today), so they pass through transparently — but our P2/P3 pool-build inserts AFTER them, and the design's "remove the I-1 refuse at :1434" edit now operates on a 3-gate prologue (I-1 → L9-hardened → L9-taproot), not a 1-gate one. **Mechanical for the common case; the brainstorm should note the new prologue so the P2 edit-site description is correct and the premise-violation table (§5a) doesn't collide with the new refusals.**
- **No new floor on the SEARCH/realized_s side:** the pool.len gates (:1687/:1696), `reject_duplicate_keys` (:1709), `realized_s` (:1722), `validate_prefix_strength` (:1761) are byte-stable in logic — only line-shifted. The funds-safety floor sequence our SPEC §5 builds on is intact.

### 3.2 `verify_multisig_template` + the H1 fix — our design's claim CONFIRMED

- **H1 fix did NOT touch `verify_multisig_template`.** The fix (`expected.tree == desc.tree && use_site_path == && tlv.use_site_path_overrides ==`) lives entirely in `emit_multisig_checks` (restore.rs/verify_bundle.rs:2756-2865), the KEYED `md1_xpub_match` `checks[]` path (the seed-derived `run_full` / supplied-vs-expected card-compare path). It widens a `passed` predicate; wire-shape unchanged.
- **`verify_multisig_template` (the TEMPLATE-completion core our P4 wires into) is byte-identical** — still calls `complete_multisig_template` at :874 with `own_account_max: None` hardcoded at :865. **Our SPEC's claim that "verify_multisig_template was already on the correct structural side" is CONFIRMED — but for a subtly different reason than the design implied:** verify_multisig_template is correct because it RE-RUNS the shared completion engine (structural reconstruction), NOT because it shares the H1 comparator. H1 fixed a DIFFERENT verify path (`md1_xpub_match`, the card-multiset compare). **Our subset-search verify story COMPOSES cleanly** — P4 just exposes `--own-account-max`/`--search-cosigner-subset` and replaces the hardcoded `None` at :865; it inherits the same completion engine restore uses, so verify==restore parity is structural and unaffected by H1. No conflict.

### 3.3 Secret-memory-hygiene is now a FIRST-CLASS bar (cycle-14 / L22) — a NEW contract our design inherits

- The sweep (per `feedback_secret_hygiene_first_class_bar.md`, user "100% correct and more!", 2026-06-21) elevated secret-memory-hygiene to a non-deferrable bar: Zeroize-on-drop + redacting Debug + off-argv + (mlock ≠ scrub) for ALL owned secrets.
- L22 already wrapped the seed-resolution path our subset-search uses: `TemplateSeed.passphrase: String → Zeroizing<String>`; `resolve_template_completion_seed`'s `from_value`/`passphrase` → `Zeroizing`; `SlotInput.value: String → SecretString`. **KEY LESSON the design must honor:** raw `Zeroizing<String>` derives a NON-redacting Debug that LEAKS into `{:?}`/panic — use the `SecretString` newtype for any secret String, NOT raw Zeroizing.
- **The over-supplied own pool's secret footprint:** our subset-search derives K_own own candidates from `ctx.entropy` (already `Zeroizing<Vec<u8>>`) via `derive_bip32_from_entropy_at_path`, pushing only the PUBLIC `account_xpub`→`key65` into the pool. So the HELD pool is public xpubs (not secret) — but an over-supplied pool means MORE BIP-32 derivations, each materializing a transient `Xpriv`/derived secret. **The design question:** does deriving K_own own keys (vs the current `--account`-list count) leave more transient private material un-zeroized in `derive_bip32_from_entropy_at_path`, and does the new bar require the subset-search derivation loop to wrap/scrub those transients? This is a NEW contract the cycle now inherits that the old (pre-bar) design did not weigh.

### 3.4 `is_order_independent_shape` + sorted-collapse — UNCHANGED (I-1 fold intact)

- `is_order_independent_shape` at synthesize.rs:335 byte-identical. The evaluator-filter (`sorted_shape && !assignment…all(i==v)`) moved 1739→**1800** but is mechanism-identical. **Our SPEC §3 / R0-r1 I-1 enumeration-side mechanism (drop `perm_rank` for sorted shapes) rests on an unchanged foundation.** Mechanical re-cite only.

### 3.5 `permutation_search.rs` — 0 functional change CONFIRMED → P1 lands clean

- `git diff --stat` is EMPTY. `search`/`unrank_permutation`/`factorial`/`total_candidates`/`required_prefix_bytes`/`validate_prefix_strength`/the pinned test — all byte-identical at their cited lines. The full-scan-with-2nd-match-ambiguity behavior our `early_exit` contract (SPEC §4.4) gates is unchanged. **P1 (the combinatorics engine work) builds on identical ground; the §7-P1 byte-invariance anchor remains valid.**

### 3.6 H12 / H13 / H8 / L24 — NOT in our completion path

- **H12** (descriptor-mode BIP-48 taproot 3' origin): in `descriptor_mode_verify_run` (verify_bundle:1397) + `bundle::compute_default_origin_path` (gained a `default_script_type` arg). This is the **descriptor-mode** verify path, NOT template-mode. Our design builds origins fresh from supplied keys in the template path; we don't call `compute_default_origin_path`. **No overlap.** (Caveat: if any future shared helper is touched — `bip48_script_type_for_root_tag` is new in `crate::template` — confirm our fresh-origin BUILD doesn't route through it; it does NOT today.)
- **H13** (hardened-multipath reject at toolkit lex): in the descriptor lex path, not our completion enumeration. No overlap.
- **H8** (synthesize_template_descriptor run_language threading): in template DESCRIPTOR synthesis (emit side), not the restore COMPLETION (decode/search) side. No overlap.
- **L24** (descriptor-mode slot-coverage OOB gate): in `descriptor_mode_verify_run`, not template-mode. No overlap.

### 3.7 Version landscape — **0.61.0 is BURNED; retarget is NOT a free renumber**

- The old plan targeted toolkit **MINOR 0.60.0 → 0.61.0**. **CRITICAL: 0.61.0 ALREADY SHIPPED** (cycle-1 CRITICAL funds-safety H12/H1/H13, `f9467cc5`, CHANGELOG line 119). So have 0.62–0.67.
- **New likely target: MINOR `0.67.0 → 0.68.0`** (new clap surface = `--search-cosigner-subset` on restore + `--own-account-max`/`--search-cosigner-subset` on verify-bundle ⇒ MINOR). md-codec/mk-codec NO-BUMP unchanged. GUI MINOR paired unchanged.
- **Coordination note:** if a cycle-14-tail follow-up (`phrase-overlay-secretstring`, `stdin-reader-transient-buf-zeroizing`) ships first it claims 0.68.0 — confirm the next-free MINOR at execution time. The 7 version-sites (both READMEs + `install.sh:32` self-pin + `fuzz/Cargo.lock` + `Cargo.lock` + `CHANGELOG.md` + Cargo.toml) and the pinned-test supersession all carry over mechanically.
- Also note: FOLLOWUP `fuzz-build-broken-…` (E0433, fuzz target set fails to compile) is OPEN on master — independent of our cycle but our P6 "run the fuzz gate before tag" step will hit it unless that lands first.

---

## 4. Re-brainstorm agenda

### DESIGN-CHANGING (must discuss with the user)

1. **Secret-footprint vs the new first-class hygiene bar (§3.3).** Does the over-supplied own pool (K_own derivations vs the `--account`-list count) materialize more transient private key material in `derive_bip32_from_entropy_at_path`, and does the now-mandatory Zeroize-on-drop bar require the subset-search derivation loop to scrub those transients / use `SecretString`-style wrappers? The pre-bar design never weighed this; it is now a gate. **(The single biggest new design surface.)**
2. **Version retarget 0.61.0 → 0.68.0 (§3.7).** 0.61.0 is consumed (cycle-1). Confirm 0.68.0 (or next-free MINOR after any cycle-14-tail follow-up), and that the only change is the renumber — verify no semver re-classification from the new surface.

### DESIGN-CONFIRMING (verify the design's claims survived the sweep — mostly favorable, brief discussion)

3. **The new 2-gate L9 prologue inside `complete_multisig_template` (§3.1).** Our P2 "remove the I-1 refuse" edit now operates on a 3-gate prologue (I-1 → hardened → taproot-override). Confirm the §5a premise-violation table doesn't collide with the new ModeViolation refusals and that the pool-build insertion point is re-described against the new layout. Likely mechanical for named-multisig templates, but the edit-site description must be re-grounded.
4. **L8 coin-type substitution in the fresh-origin BUILD (§3.1).** The C1 "origins built fresh" invariant now includes a network-coin-type patch in the all-own path our subset-search runs in. Confirm the over-supplied own derivations all flow through it correctly (testnet/signet/regtest), and update the C1 statement to mention L8.
5. **H1 fix vs our verify story (§3.2).** Our claim "verify_multisig_template is on the correct structural side" is CONFIRMED — but the reason is re-run-the-completion-engine, NOT shared-comparator-with-H1. Re-word the SPEC's verify rationale so it cites the right basis (and notes H1 touched a different verify path).

### MECHANICAL (re-grep / rebase only — no user discussion)

6. **Citation re-grep:** all restore.rs interior cites +61 (below L9 insert) or +5 (clap block); `perm_count_u128` is in restore.rs:1943 not permutation_search.rs (pre-existing mislabel). permutation_search / synthesize / verify_multisig_template / the test file: re-cite at the SAME lines (byte-stable). The plan §0 + SPEC body line numbers refresh against `ddabf5e3`.
7. **Rebase `feature/own-account-subset-search`** onto `ddabf5e3` (our 9-commit lead is design-docs-only; src clean — a trivial rebase, no code conflicts expected).
8. **P6 fuzz gate caveat:** the open `fuzz-build-broken-…` E0433 will fail the pre-tag fuzz build unless it lands first (not our cycle's bug; note it).

---

## 5. One-line bottom line

The engine, the sorted-collapse, `verify_multisig_template`, and the test file are **byte-stable** (P1 + the math + verify-parity land clean); restore.rs citations DRIFTED uniformly (+61/+5, zero structurally-wrong); the two genuinely NEW things the re-brainstorm must weigh are (a) the **first-class secret-hygiene bar** applied to the over-supplied own-pool derivation footprint, and (b) the **0.61.0→0.68.0 version retarget** (0.61.0 is burned). The H1 fix CONFIRMS — does not threaten — our verify story; the new L9/L8 gates inside the shared core are compose-with (mechanical re-description), not conflict-with.
