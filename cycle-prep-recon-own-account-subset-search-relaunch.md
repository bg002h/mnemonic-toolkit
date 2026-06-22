# cycle-prep recon — 2026-06-22 — own-account-subset-search RELAUNCH

**Origin/master SHA at recon time:** `5e7b9dec` (`5e7b9decb5c16bd6a3e0bdb5802f0c51dad2fed0`)
**Origin/master toolkit version:** **0.69.1** (code is v0.69.1; the tip commit `5e7b9dec` is docs-only — files the FOLLOWUP `derive-slot-account-xpriv-scrub-confinement`)
**Local branch:** `feature/own-account-subset-search` — **11 ahead / 0 behind** origin/master; the 11-commit lead is **design-docs-only** (`git diff origin/master..HEAD -- crates/mnemonic-toolkit/src/` is EMPTY → trivial rebase, zero code conflicts).
**Design base SHA (paused cycle):** `82e58674` (v0.60.0). **Prior recon base:** `ddabf5e3` (v0.67.0).
**Full delta:** `82e58674..origin/master` = **81 commits**. Since the prior recon: `ddabf5e3..origin/master` = **15 commits**.
**Untracked:** the design docs + a large set of `cycle-prep-recon-*.md` / `cycle-b-*` scratch files (none in scope).

**Headline:** the entire +15-commit burndown since the prior recon (`ddabf5e3`→`5e7b9dec`) touched **ZERO of the 6 subset-search surface files** — all 15 are the cycle-15 secret-zeroize program (bip85-scratch / bsms HMAC_KEY / cycle-15 Lane-T derived-output) + design/followup docs. The 6 surface files + the test file are **byte-identical** (verified by SHA256) between `ddabf5e3` and `5e7b9dec`. **The prior recon's drift table therefore transfers VERBATIM to final master — re-confirmed citation-by-citation below.** The only material new facts beyond the prior recon are non-code: (1) the `fuzz-build-broken` E0433 FOLLOWUP is now **RESOLVED**; (2) the derive_slot hygiene gap is now a **filed, DECOUPLED FOLLOWUP** (consumed, not authored, by this cycle); (3) the version retarget is now **0.70.0** (0.68.0/0.69.0/0.69.1 also burned, not just 0.61.0→0.68.0).

---

## 1. Sync + delta

| Item | Value |
|---|---|
| `origin/master` short SHA | `5e7b9dec` |
| toolkit version on master | **0.69.1** |
| our branch | `feature/own-account-subset-search`, **11 ahead / 0 behind**, src clean |
| full delta `82e58674..master` | **81** commits |
| delta since prior recon `ddabf5e3..master` | **15** commits |

### Per-file commit counts since base `82e58674` (CONFIRMED, matches the task's stated 2/4/1/0/0/0)

| File | `82e58674..master` | `82e58674..ddabf5e3` | `ddabf5e3..master` |
|---|---|---|---|
| `cmd/restore.rs` | **2** | 2 | **0** |
| `cmd/verify_bundle.rs` | **4** | 4 | **0** |
| `synthesize.rs` | **1** | 1 | **0** |
| `permutation_search.rs` | **0** | 0 | **0** |
| `derive_slot.rs` | **0** | 0 | **0** |
| `derive.rs` | **0** | 0 | **0** |

All surface-file churn happened BEFORE the prior recon's base. The +15 since added 0 to every file. Byte-identity `ddabf5e3`→`master` confirmed by SHA256 for all 6 files + `tests/cli_restore_md1_template_multisig.rs`; `complete_multisig_template` (full fn 1421-1940) and `verify_multisig_template` core (808-880) and the `is_order_independent_shape` region (330-370) `diff`-empty.

### The +15 since the prior recon (none touch our surface)

```
5e7b9dec design(followup): file derive-slot-account-xpriv-scrub-confinement (decoupled hygiene slug)
1576b2d6 design(bip85-scratch): persist whole-diff review — GREEN
5293e4a6 release(bip85-scratch): toolkit v0.69.1
b010123b feat(bip85-scratch): wrap encode/dice internal scratch in Zeroizing
1cad7ec9 design(bip85-scratch): spec+plan R0-GREEN
d6e8757d design(cycle-15 groupA): persist whole-diff review — GREEN
360a6e88 release(cycle-15 groupA): toolkit v0.69.0
db6b8d81 feat(cycle-15 groupA): #4 cfg(test)-confinement lint tier
793e1e66 feat(cycle-15 groupA): #3 derive_hmac_key -> Zeroizing<[u8;32]>
a7188aca design(cycle-15 groupA): bsms HMAC_KEY + lint cfg(test)-confinement R0-GREEN
9b7c78a7 release(cycle-15): toolkit v0.68.0 — derived-output SecretString/Zeroizing + ms-codec 0.6 pin
b16a44c8 feat(cycle-15t): P1 — derived-output SecretString/Zeroizing
07d50b50 design(cycle-15): Lane M SHIPPED (ms-codec 0.6.0 + ms-cli 0.10.0)
391be04a design(cycle-15): secret-zeroize program — 3-lane spec/plan R0 trail
79100a66 design(sweep): secret-key-material hygiene sweep — file FOLLOWUP slugs + 5-repo sweep reports
```

---

## 2. Per-citation drift table — re-grepped vs CURRENT `origin/master` `5e7b9dec`

Legend: ✅ ACCURATE (live line) · ⚠️ DRIFTED-by-N from base `82e58674` (symbol intact, new line) · ❌ STRUCTURALLY-WRONG.
**All "new" lines below are the LIVE lines on `5e7b9dec` and are identical to the prior recon's `ddabf5e3` lines (no movement in the +15).**

### `cmd/restore.rs` — drift driven by the L8/L9 +57-line insertion (+61 below it) / Zeroizing-block (+5 above it), ALL from the `82e58674..ddabf5e3` cycle-13b commits

| Symbol / citation | base `82e58674` | LIVE `5e7b9dec` | Status |
|---|---|---|---|
| `fn complete_multisig_template` | :1416 | **:1421** | ⚠️ +5 |
| `--own-account-max` refuse gate (`ctx.own_account_max.is_some()`) | :1434 | **:1439** | ⚠️ +5 |
| pool.len supply gates | :1626 / :1635 | **:1687** (`< n`) / **:1696** (`> n`) | ⚠️ +61 |
| `realized_s` = `perm_count_u128(n, n)` | :1661 | **:1722** (`realized_s`) / **:1723** (call) | ⚠️ +61 |
| id/addr mode select (`id_search`/`addr_search`) | :1665-1666 | **:1726-1727** | ⚠️ +61 |
| `reject_duplicate_keys` whole-pool | :1648 | **:1709** | ⚠️ +61 |
| `key65` (struct field def / use) | :1647 | def **:1107**, pool use **:1708** | ⚠️ moved (def vs use) |
| `sorted_shape` binding | :1676 | **:1737** | ⚠️ +61 |
| evaluator-filter (`sorted_shape && assignment != identity → false`) | :1739 | **:1800** (`if sorted_shape && !assignment.iter().enumerate().all(\|(i,&v)\| i==v)`) | ⚠️ +61 — mechanism byte-identical |
| `validate_prefix_strength` call | :1700 | **:1761** | ⚠️ +61 |
| **`fn perm_count_u128` def** | (SPEC/plan say `permutation_search.rs:1882`) | **`restore.rs:1943`** | ❌ **STRUCTURALLY-WRONG file** (see note) |
| `--account` `Vec<u32>` `default_value="0"` | :106 | **:106** (`pub account: Vec<u32>`) | ✅ |
| `conflicts_with` precedent | :86 | **:86** (`conflicts_with = "passphrase"`) | ✅ |
| `#[arg(long="own-account-max")]` clap def | (exists) | **:133-134** (`pub own_account_max: Option<u32>`) | ✅ |
| `run_multisig_template_completion` | :1321 | **:1326** | ⚠️ +5 |

> **`perm_count_u128` STRUCTURAL note:** the SPEC §3 (`restore.rs:1661`) and plan §0/§4.1 cite this helper as living at **`permutation_search.rs:1882`**. It does **NOT** exist in `permutation_search.rs` (grep returns NONE there); it is a private fn at **`restore.rs:1943`**. `permutation_search.rs` exports `factorial`/`total_candidates`. This is a **pre-existing mislabel in the design** (NOT sweep-induced; flagged by the prior recon too). Mechanical fix, but it is a wrong-FILE error, so tagged STRUCTURALLY-WRONG per the gate's strict reading.

### `cmd/verify_bundle.rs` — ALL ✅ (byte-unchanged region)

| Symbol / citation | base | LIVE `5e7b9dec` | Status |
|---|---|---|---|
| `fn verify_multisig_template` | :808 | **:808** | ✅ byte-unchanged |
| `complete_multisig_template` call (inside verify) | :874 | **:874** | ✅ |
| `own_account_max: None` hardcoded | :865 | **:865** | ✅ |
| `--account` SCALAR `u32` `default_value="0"` | :64 | **:64** (`pub account: u32`) | ✅ |

### `permutation_search.rs` — ALL ✅ EXACT (file BYTE-IDENTICAL to base; SHA256 a0c1cef8…)

| Symbol | base | LIVE | Status |
|---|---|---|---|
| `fn search` | :551 | :551 | ✅ |
| `fn unrank_permutation` | :494 | :494 | ✅ |
| `fn factorial` | :481 | :481 | ✅ |
| `fn total_candidates` | :509 | :509 | ✅ |
| `fn required_prefix_bytes` | :322 | :322 | ✅ |
| `fn validate_prefix_strength` | :342 | :342 | ✅ |
| pinned test `prefix_ladder_own_account_max_subset_space` | :740 | :740 | ✅ |

### `synthesize.rs` — ✅

| Symbol | base | LIVE | Status |
|---|---|---|---|
| `fn is_order_independent_shape` | :335 | **:335** | ✅ byte-identical region |

### Tests to flip (`tests/cli_restore_md1_template_multisig.rs`) — FILE UNCHANGED

| Test | base | LIVE | Status |
|---|---|---|---|
| `own_account_max_flag_refuses_with_actionable_message` (REWRITE) | :677 | **:677** | ✅ |
| `pool_larger_than_slots_refuses_with_actionable_message` (UPDATE msg) | :715 | **:715** | ✅ |
| `multi_account_own_resolves_both_slots` (stays byte-GREEN) | :635 | **:635** | ✅ |
| `#[test]` count | 27 | **27** | ✅ |

**Drift verdict:** identical to the prior recon. restore.rs interior cites DRIFTED uniformly (+61 below the L9 insert / +5 in the clap block); `perm_count_u128` is the ONE structurally-wrong cite (wrong FILE — `restore.rs:1943`, not `permutation_search.rs:1882`); everything else (verify_bundle core, the whole engine, synthesize, the test file) is byte-stable at the SAME lines.

---

## 3. DESIGN-IMPACT — the full delta (with attention to NEW vs the prior recon)

Because the +15 burndown commits touched none of the 6 surface files, every code-level design-impact finding from the prior recon (resweep §3) **re-confirms unchanged at final master**. RE-CONFIRMATION below, then the NEW (non-code) deltas.

### 3.1 `complete_multisig_template` — RE-CONFIRMED (signature / ctx / floors / C1 all stable)

- **Signature UNCHANGED:** `pub(crate) fn complete_multisig_template<E: Write>(d, ctx: &MultisigCompletionCtx, stderr) -> Result<MultisigCompletionOutcome, ToolkitError>` (restore.rs:1421). **`MultisigCompletionCtx` fields UNCHANGED** (`own_accounts: Vec<u32>`, `own_account_max: Option<u32>`, `passphrase`/`derive_language` [the entropy/seed inputs], `explicit_own_origin`, `cosigner_specs`, `expect_wallet_id`, `search_address`, `search_addr_min/max`, `search_chain`, `accept_search_time`, `network`). No new field, no new param.
- **The L9 two-gate prologue (NEW gates the prior recon flagged) — RE-CONFIRMED present, byte-stable.** Inside the fn, BEFORE pool assembly (restore.rs ~1465 / ~1488):
  - `if md_codec::to_miniscript::has_hardened_use_site(d) → ModeViolation` (hardened use-site refuse).
  - `if taproot_override_card(d) && !restorable_taproot_override_card(d) → ModeViolation` (#26 taproot-override refuse; non-hardened `tr(NUMS,multi_a)` admitted).
  - Both are defense-in-depth for named-multisig templates (non-hardened, non-taproot-override today) → pass through transparently. **P2's "remove the I-1 refuse at :1434/now :1439" edit operates on a 3-gate prologue** (I-1 own-account-max → L9-hardened → L9-taproot). Composes-with, not conflicts-with. No NEW gate beyond these two appeared in the +15 (the fn body is `diff`-empty `ddabf5e3`→master).
- **L8 coin-type substitution (the fresh-origin BUILD) — RE-CONFIRMED present, byte-stable.** In the all-own / no-`--cosigner` / no-`--origin` fallback, `canonical_origin(d.tree)` (mainnet coin 0') is patched: `comps[1] = ChildNumber::from_hardened_idx(network.coin_type())` (restore.rs ~1586-1608) — identity on mainnet, coin-1 on testnet/signet/regtest. This is the ONLY origin source in the pure-own path the subset-search runs in. **The C1 "per-slot origins built fresh; carried origin never loaded" invariant STILL HOLDS** — and "built fresh" now subsumes the L8 coin-type patch. Over-supplied own candidates derive through this same closure. No change vs the prior recon.
- **Floor sequence UNCHANGED in logic (only line-shifted):** pool.len gates (:1687/:1696), `reject_duplicate_keys` whole-pool (:1709), `realized_s` (:1722-1723), `validate_prefix_strength` (:1761), the `sorted_shape` evaluator-filter (:1737/:1800). The funds-safety floor sequence the SPEC §5 builds on is intact.

### 3.2 `verify_multisig_template` + H1 — RE-CONFIRMED (verify story COMPOSES; H1 elsewhere)

- **`verify_multisig_template` is BYTE-IDENTICAL** base→`ddabf5e3`→master (808/865/874 all exact; the 800-880 region is `diff`-empty across the whole 81-commit span). It RE-RUNS the shared `complete_multisig_template` engine, so verify==restore parity is structural. P4 just exposes `--own-account-max`/`--search-cosigner-subset` and replaces the hardcoded `own_account_max: None` at :865 — inherits the feature.
- **H1 did NOT touch `verify_multisig_template`.** The cycle-1 H1 widening lives in `emit_multisig_checks` (verify_bundle.rs:2759-2821, the keyed `md1_xpub_match` `checks[]` path — `expected_md_decoded.tlv.use_site_path_overrides == desc.tlv.use_site_path_overrides`). It is the seed-derived card-multiset compare path, a DIFFERENT verify path. The SPEC's verify rationale ("verify_multisig_template was already on the correct structural side") is CONFIRMED, but the BASIS is "re-runs the completion engine," NOT "shares the H1 comparator." No change vs the prior recon (resweep §3.2).

### 3.3 `is_order_independent_shape` + sorted-collapse — RE-CONFIRMED UNCHANGED

`synthesize.rs:335` byte-identical (region `diff`-empty base→master). The sorted-shape evaluator-filter (`restore.rs:1800`) is mechanism-identical (only line-shifted). The SPEC §3 / R0-r1 I-1 enumeration-side mechanism (drop `perm_rank` for sorted shapes) rests on an unchanged foundation. Mechanical re-cite only.

### 3.4 `permutation_search.rs` — RE-CONFIRMED 0 FUNCTIONAL CHANGE (byte-identical to base)

The whole file is **byte-identical** base `82e58674` → master `5e7b9dec` (SHA256 `a0c1cef8…` on both ends). `search`/`unrank_permutation`/`factorial`/`total_candidates`/`required_prefix_bytes`/`validate_prefix_strength`/the pinned test — all at their cited lines. **P1 lands on identical ground.** The §7-P1 byte-invariance anchor + the §4.4 full-scan-with-2nd-match-ambiguity contract are valid as written. (Engine-clean confirmed.)

### 3.5 derive_slot hygiene gap — STILL UNCLAIMED in code; now a FILED, DECOUPLED FOLLOWUP (NEW since the prior recon)

- `derive_slot.rs` / `derive.rs`: **`account_xpriv` is STILL a bare `Xpriv` field** in `DerivedAccount` (`derive.rs:27` `pub account_xpriv: Xpriv`), un-scrubbed on drop; `account_xpub` is the public field consumed. **`derive_account_xpub_only` / `ScrubbedXpriv` do NOT exist yet** — the gap is unclaimed in code (both files have 0 commits since base).
- **NEW (this is the change vs the prior recon's §3.3 open design-question):** the gap is now formally tracked as FOLLOWUP **`derive-slot-account-xpriv-scrub-confinement`** (FOLLOWUPS.md:4514, filed @`5e7b9dec`, status `open`, tier `polish`/`next-cycle`). Its **Origin** line states it was **DECOUPLED OUT of this paused cycle on architect advice**: "subset-search CONSUMES `derive_account_xpub_only` as a 1-line dependency; it does NOT author this." The provisional fix (a move-only `ScrubbedXpriv(Xpriv)` RAII newtype + a `derive_account_xpub_only(...) -> (Xpub, Fingerprint)` entrypoint) is specified there. **Design impact:** the prior recon's "biggest new design surface" (the over-supply derivation secret footprint) is now RESOLVED as a clean external dependency — the subset-search design's hygiene thread must be rewritten as a CONSUMED dependency, NOT authored here.

### 3.6 Other burndown findings (H12 / H13 / H8 / L24 / cycle-15) — NOT in our completion path

- **H12 / H13 / H8 / L24** (descriptor-mode taproot origin / hardened-multipath lex reject / synthesize run_language / descriptor-mode slot OOB): all in the descriptor-mode verify path or the emit side, NOT the template-completion path. No overlap (re-confirmed; these were already in the `82e58674..ddabf5e3` window). The fresh-origin BUILD does NOT route through `compute_default_origin_path` / `bip48_script_type_for_root_tag`.
- **cycle-15 (+15 commits, the new burndown):** secret-zeroize program — bip85 scratch, BSMS HMAC_KEY, cycle-15 Lane-T derived-output SecretString/Zeroizing. **None touch the 6 surface files.** Its only relevance to this cycle is upstream context for §3.5 (the derive_slot gap is "a natural sibling of the cycle-15/Lane-T derived-output zeroize work").

### 3.7 NEW-DELTA verdict

**NO NEW DESIGN-CHANGING DELTA beyond L8 / L9 / H1.** The "more changed than planned" burndown (the +15 since the prior recon) was entirely secret-hygiene + docs and is byte-disjoint from the subset-search surface. The two genuinely-new facts are both FAVORABLE / non-code: (a) the derive_slot hygiene question is now a decoupled, consumed FOLLOWUP (removes the prior recon's open design surface), and (b) the fuzz-build E0433 is RESOLVED (removes the P6 fuzz-gate blocker). Neither is design-changing for the engine/floors/math.

---

## 4. Version + locksteps

### 4.1 Version retarget — **0.61.0 / 0.68.0 / 0.69.0 / 0.69.1 are ALL BURNED → target MINOR `0.70.0`**

- CHANGELOG confirms shipped: 0.67.0, **0.68.0** (cycle-15 Lane-T, the prior recon's tentative target — now consumed), **0.69.0** (Group A), **0.69.1** (bip85-scratch). The old plan's 0.60.0→0.61.0 is long burned (0.61.0 = cycle-1 @ `f9467cc5`).
- **Next-free MINOR = `0.70.0`** (matches the task). New clap surface (`--search-cosigner-subset` on restore + `--own-account-max`/`--search-cosigner-subset` on verify-bundle) ⇒ MINOR. md-codec/mk-codec NO-BUMP. GUI MINOR paired.
- **The 7 version-sites** carry over mechanically: `crates/mnemonic-toolkit/Cargo.toml:3` (=0.69.1), `scripts/install.sh:32` self-pin (=`mnemonic-toolkit-v0.69.1`), `Cargo.lock:727` (=0.69.1), `fuzz/Cargo.lock`, BOTH READMEs, `CHANGELOG.md`. The pinned `prefix_ladder_own_account_max_subset_space` test (`permutation_search.rs:740`) supersession also carries over.
- Confirm the next-free MINOR AT EXECUTION TIME (any cycle-15-tail follow-up that ships first claims 0.70.0).

### 4.2 GUI schema-mirror + manual lockstep — surface CONFIRMED accurate

- **GUI pins toolkit `v0.60.0`** (`mnemonic-gui/Cargo.toml:42` + `pinned-upstream.toml:22`, tag `mnemonic-toolkit-v0.60.0`) — the SAME base as the design. GUI HEAD `5ce9d53`. So the GUI's flag-name surface matches the design's assumptions (the GUI's pin-bump cadence is independent of this cycle; it is 10 MINORs behind master, which does not affect the lockstep delta this cycle adds).
- **restore:** GUI schema (`mnemonic-gui/src/schema/mnemonic.rs:713`) ALREADY lists `--own-account-max` (with a "reserved/refused this cycle" comment) ⇒ only **`--search-cosigner-subset` is a NEW NAME** to add for restore. (`--own-account-max` refuse→search is behavior-only; schema gates flag-NAMES ⇒ no restore schema delta for it — but its MANUAL row still needs the refuse→search edit.)
- **verify-bundle:** the GUI schema lists NEITHER `--own-account-max` NOR `--search-cosigner-subset`, and the live verify-bundle clap has neither (`own_account_max: None` hardcoded at verify_bundle.rs:865) ⇒ **BOTH are NEW NAMES** there ⇒ schema-mirror BOTH + add both manual rows. `--account` stays scalar `u32` (verify_bundle.rs:64). **SPEC §9 is accurate as written.**
- **Manual:** `docs/manual/src/40-cli-reference/41-mnemonic.md:938` still says `--own-account-max` "**NOT SUPPORTED YET** … passing this flag **refuses**" ⇒ the refuse→search edit + the new verify-bundle rows + the `--search-cosigner-subset` rows + a subset-search subsection are all still pending exactly as the plan P6 describes. (Single manual file `41-mnemonic.md`; the plan's path is correct.)

### 4.3 Fuzz-build status — **RESOLVED (was OPEN at the prior recon)**

- FOLLOWUP `fuzz-build-broken-unrestorable-advisory-references-bin-only-cmd` is now **✓ RESOLVED (2026-06-20, NO-BUMP)** (FOLLOWUPS.md:57). Fixed via option (a): the two predicates (`taproot_override_card` / `restorable_taproot_override_card`) moved VERBATIM to a new lib-leaf module `crates/mnemonic-toolkit/src/taproot_override_classify.rs` (mounted bin-private in `main.rs` + under `#[cfg(fuzzing)]` in `lib.rs`); `cmd::restore` re-exports them. `cargo +nightly fuzz build` GREEN. **Pure relocation, byte-identical predicate logic.**
- **Impact:** the prior recon's P6 caveat ("the open fuzz-build E0433 will fail the pre-tag fuzz build") is **CLEARED** — P6's pre-tag fuzz gate is unblocked. (Note: those two predicates now live in `taproot_override_classify.rs`, re-exported by `cmd::restore` — the L9 #26 taproot-override gate in `complete_multisig_template` resolves the bare names unchanged via the re-export; no impact on the gate's behavior or P2's edit-site.)

---

## 5. Re-brainstorm / SPEC / plan FOLD LIST (before re-R0)

### MECHANICAL (just edit + re-R0 — no human/architect judgment)

1. **(a) Citation refresh.** All `restore.rs` interior cites: **+61** below the L9 insert (pool.len gates 1626/1635→**1687/1696**; realized_s 1661→**1722/1723**; id/addr 1665-1666→**1726-1727**; reject_duplicate_keys 1648→**1709**; sorted_shape 1676→**1737**; evaluator-filter 1739→**1800**; validate_prefix_strength 1700→**1761**) or **+5** in the clap block (complete_multisig_template 1416→**1421**; refuse gate 1434→**1439**; run_multisig_template_completion 1321→**1326**). `key65` is def **:1107** / use **:1708**. `--account` :106, `conflicts_with` :86, `own_account_max` clap def :133-134 unchanged. verify_bundle (808/865/874/64), permutation_search (all), synthesize (335), the test file (635/677/715, 27 tests): re-cite at the SAME lines (byte-stable). Cite source SHA `5e7b9dec`.
2. **(e) `perm_count_u128` location fix.** SPEC §3 + plan §0/§4.1 say `permutation_search.rs:1882`; it is **`restore.rs:1943`** (a private fn; NOT in permutation_search.rs). Fix the file+line.
3. **(b) Version.** `0.60.0 → 0.61.0` → **`0.60.0 → 0.70.0`** everywhere (SPEC §0/§9, plan §0/P6, phase-map). Confirm next-free at execution time.
4. **(f) Fuzz-gate caveat UPDATE.** Plan P6 / the prior-recon caveat: the fuzz-build E0433 is **RESOLVED** — drop the "open blocker" note. (Optional: note the predicates now live in `taproot_override_classify.rs`, re-exported by `cmd::restore` — relevant only if a doc cites the predicate's module.)
5. **Rebase** `feature/own-account-subset-search` (11-commit, design-docs-only lead) onto `5e7b9dec` — trivial, no code conflicts.

### DESIGN-CHANGING (needs human/architect judgment at re-R0) — but smaller than the prior recon implied

6. **(c) Hygiene thread → CONSUMED dependency, NOT authored.** The SPEC's secret-footprint thread (old §3.3 of the resweep) must be **rewritten as a 1-line consumed dependency on `derive_account_xpub_only`** (FOLLOWUP `derive-slot-account-xpriv-scrub-confinement`, FOLLOWUPS.md:4514) — the cycle CONSUMES it, does NOT author it. This RESOLVES the prior recon's "single biggest new design surface" into an external dep. Architect should confirm: (i) does the subset-search over-supply loop call `derive_account_xpub_only` (so it never owns an `Xpriv`), and (ii) is the FOLLOWUP a hard pre-req of P2 or can P2 land against the current bare-field `derive_bip32_from_entropy_at_path` with the scrub deferred? (The FOLLOWUP is tier `polish`/`next-cycle` and status `open` — sequencing call.)
7. **(d) L8 / L9 / H1 design-doc folds** (re-grounding, not mechanism change):
   - **L8 into C1:** the SPEC §5 C1 invariant ("origins built fresh; carried origin never loaded") must NOTE the L8 coin-type substitution (restore.rs ~1586-1608) as part of "built fresh" — and confirm over-supplied own derivations flow through it on testnet/signet/regtest.
   - **L9 prologue into P2:** the plan P2 "remove the I-1 refuse" edit + the SPEC §5a premise-violation table must be re-described against the **3-gate prologue** (I-1 own-account-max → L9-hardened → L9-taproot-override) so the edit-site is correct and §5a doesn't collide with the two ModeViolation refusals (mechanical for named-multisig templates, but the description must be re-grounded).
   - **H1 verify-rationale reword:** the SPEC §3.2/verify rationale must cite the correct basis — `verify_multisig_template` is correct because it **re-runs the completion engine** (structural reconstruction), NOT because it shares the H1 comparator (H1 fixed `md1_xpub_match` in `emit_multisig_checks`, a DIFFERENT verify path).

### NEW delta beyond L8/L9/H1

8. **None that is design-changing.** The two new facts (derive_slot FOLLOWUP filed+decoupled; fuzz-build resolved) are folded above as 6 and 4. The engine, floors, math, sorted-collapse, `verify_multisig_template`, and the test file are byte-stable across the full 81-commit delta. **Explicit: no new design-changing delta beyond L8/L9/H1.**

---

## 6. Bottom line

The 81-commit / +15-since-prior-recon burndown that "changed more than planned" changed **nothing on the subset-search surface** — all 6 surface files (+ the test file) are byte-identical from base `82e58674` through `ddabf5e3` to final master `5e7b9dec`; the engine is byte-identical (SHA256-verified). The prior recon's drift table transfers verbatim: restore.rs interior cites DRIFTED uniformly (+61/+5), the lone STRUCTURALLY-WRONG cite is `perm_count_u128` (it is `restore.rs:1943`, not `permutation_search.rs:1882`), and verify_multisig_template / permutation_search / synthesize / the test file are byte-stable at the SAME lines. `complete_multisig_template`'s signature + `MultisigCompletionCtx` + floor sequence + the C1 carried-origin-never-loaded invariant are UNCHANGED; the L8 coin-type substitution and the L9 two-gate prologue are present and stable (compose-with, the design already weighed them). The two NEW facts are FAVORABLE and non-code: the derive_slot `account_xpriv` hygiene gap is now a filed, **decoupled, consumed** FOLLOWUP (not authored by this cycle), and the fuzz-build E0433 is **RESOLVED** (P6 unblocked). Retarget MINOR **0.70.0**. **No new design-changing delta beyond L8/L9/H1.**
