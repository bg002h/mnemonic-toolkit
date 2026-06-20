# P3a — RESTORE completion (CANONICAL multisig templates, #28 phase 2) — per-phase R0 EXECUTION review, ROUND 2 (opus architect, verbatim)

> Reviewer: opus architect (adversarial, source-verified, tests+clippy run). HEAD `2249e433566dceb22b962ed1d58762d300c0d798`, branch `feature/bundle-md1-template-multisig`. Verified against md-codec `0.37.0`, mk-codec `0.4.0`, ms-codec `0.4.4`, rust-miniscript `13.0.0` (git rev `95fdd1c5773bd918c574d2225787973f63e16a66`) per `Cargo.lock`. Fold under review: `cd58d742` (I-1 gate) + `92ddbfbd` (M-1 pins); persisted reviews `2249e433` carry the round-1 RED + P1/P2 reports.

**Verdict: GREEN — 0 Critical, 0 Important.**

I-1 is genuinely CLOSED, the fold introduces no drift or regression, the M-1 reproduction pins are non-vacuous, and the FOLLOWUP is filed accurately. The gate may advance to **P3b**.

---

## I-1 status — CLOSED (structurally, not just assert-backstopped)

The round-1 defect was: `--own-account-max K` / any `pool.len() > n` over-supply feeds an over-sized pool into a P1 engine that enumerates only `n!` permutations of the FIRST `n` pool entries, so pool indices `≥ n` are never evaluated → a legitimate wallet silently NO-MATCHes; and `realized_s = P(pool,n)` no longer reflects the `n!` space actually scanned. The fold closes every leg:

- **(i) every path to `ps::search` now guarantees `pool.len() == n`.** The sole engine entrypoint is `ps::search(n, evaluator, mode)` inside `run_capped_search` (`restore.rs:1718`), reachable only from the two `run_capped_search` calls at `:1502` (Id) / `:1548` (Address). Both sit AFTER the unconditional gates: under-supply `if pool.len() < n { return Err(ModeViolation) }` (`:1419-1427`) and over-supply `if pool.len() > n { return Err(ModeViolation) }` (`:1428-1437`). The over-supply check is a plain runtime `return`, NOT a `debug_assert`, so a release build refuses — it cannot mis-scan. `grep` confirms `ps::search` has exactly ONE call site (`:1718`) and `own_account_max` exactly TWO references: the arg decl (`:134`) and the I-1 gate (`:1179`) — the old `(0..k).collect()` range expansion is fully removed; `own_accounts` is now `args.account.clone()` only (`:1351`).
- **(ii) the refuse gates fire at INPUT, before any search, with actionable messages.** `--own-account-max` is refused at `:1179-1185` (`bad(...)` → `BadInput`), the FIRST substantive check in `run_multisig_template_completion`, before `--from` resolution, seed derivation, or pool assembly. Message names the flag and points at `--account <N[,N,…]>`. The `pool.len() > n` gate (`:1428-1437`) fires before `reject_duplicate_keys` (`:1441`) and before `realized_s` (`:1451`), message says "too many keys … must EXACTLY equal … Remove the extra key(s), or specify your exact own account(s) with --account".
- **(iii) realized_s fidelity restored.** `realized_s = perm_count_u128(n, n)` (`:1450-1451`) = `n!` = the space the engine actually enumerates, guarded by `debug_assert_eq!(pool.len(), n, …)` (`:1449`) as a backstop. Both `run_capped_search` calls pass this `realized_s` (`:1502`, `:1548`), which sizes `validate_prefix_strength` and the adaptive cap. SPEC §6.2/§7-floor-5 sizing now matches the scan.
- **(iv) backstop, not primary guard.** Confirmed above — the `debug_assert_eq!` is redundant to the `:1428` runtime refuse.

**Exit-code fail-safety (verified against `error.rs`).** The refusals are exit-distinguishable from a search miss and can never be a silent wrong wallet: `--own-account-max` gate → `BadInput` → exit **1** (`error.rs:215`, `:966`); over/under-supply → `ModeViolation` → exit **2** (`error.rs:558`); search NO-MATCH → `RestoreMismatch` → exit **4** (`error.rs:565`). Refuse direction is "loud actionable input error," never "complete a wrong wallet."

**Non-vacuity of the two refuse tests (`tests:649-720`).** `own_account_max_flag_refuses_with_actionable_message` drives the real binary (assert_cmd) on a genuine 2-of-2 with `--own-account-max 3`, asserts `.failure()`, and requires stderr to contain BOTH `own-account-max` AND `--account` AND NOT `no match`. Pre-fold this exact invocation built pool=4 into n=2 and produced an exit-4 `no match` (round-1 reviewer's own CLI repro) → the test FAILS pre-fold, PASSES post-fold: a real RED→GREEN exercising the refuse path, not a tautology. `pool_larger_than_slots_refuses_with_actionable_message` over-supplies a 3rd (outsider) cosigner card (`SEED_OUTSIDER`, a distinct valid BIP-39 phrase, `:33`) so pool(3) > n(2); same RED-before/GREEN-after structure.

## Fold-drift / regression check — clean

- **C1 carried-origin-never-loaded:** untouched. `synthesize.rs` is not in `cd58d742^..92ddbfbd` (diff stat: only `restore.rs`, the test file, `FOLLOWUPS.md`). `carried_origin_never_loaded_into_completion` passes.
- **own-origin default-family fail-safe:** the precedence chain (`--origin` → cosigner-family-with-account-substituted `:1373-1382` → BIP-48 `canonical_fallback`) is unchanged by the fold; only the `own_accounts` source narrowed from "list-or-range" to "list" (`:1351`). The wrong-own-origin → NO-MATCH-only property is preserved.
- **distinct-keys + strong-prefix + every-slot floors:** `reject_duplicate_keys` still runs pre-search (`:1441`); the every-slot floor TIGHTENED from `>= n` (range branch) to exact `== n` (`:1419/:1428`) — strictly safer.
- **three completion modes + single-sig non-regression:** all green in the suite (explicit-mode warn, id-search, address-search, `singlesig_template_completion_unchanged`).
- **Test/clippy/g6 evidence (run by reviewer):**
  - `cargo test -p mnemonic-toolkit --test cli_restore_md1_template_multisig` → **18 passed; 0 failed** (round-1's 14 + the 4 new: 2 I-1 refusals + 2 M-1 BIP-87).
  - `cargo test -p mnemonic-toolkit --lib` → **158 passed; 0 failed; 3 ignored** (unchanged from round 1 — fold added no lib tests).
  - `cargo test -p mnemonic-toolkit --test prop_backup_restore_roundtrip` → **13 passed; 0 failed** (no #25 full-policy regression).
  - `cargo clippy -p mnemonic-toolkit --tests --bins --lib -- -D warnings` → exit **0**, clean.
  - `git diff --name-only 8967294d..HEAD` does NOT contain `mlock.rs` — g6 fmt-exemption intact.

## M-1 reproduction pins — correct and non-vacuous

`default_family_bip87_id_search_completes_to_golden` / `default_family_bip87_address_search_completes_to_golden` (`tests:825-893`) emit a 2-of-2 `wsh-sortedmulti` at the DEFAULT BIP-87 family (`m/87'/0'/account'`, `bundle_bip87_arg_vec:765-792`) and complete via id/address search to `golden_addresses_bip87` (`:798-820`), which builds `wsh(sortedmulti(2,…))` DIRECTLY from the cosigner xpubs via rust-miniscript `Descriptor::from_str` → `into_single_descriptors` → `derive_at_index` → `.address()` — an INDEPENDENT external golden, NOT an md-codec reconstruction. **Anti-vacuity confirmed at source:** `md-codec::canonical_origin::canonical_origin` returns BIP-48 (`m/48'/0'/0'/2'`) for `wsh(sortedmulti)` unconditionally — it is structural, family-blind (`descriptor-mnemonic/crates/md-codec/src/canonical_origin.rs:57-61`, pinned by `wsh_sortedmulti_returns_bip48_type_2:195-199`). So if the own origin defaulted to `canonical_fallback` (BIP-48) instead of reading the cosigner's actual BIP-87 family off its mk1 and substituting the own account, the own key would derive at the WRONG path → id/address NO-MATCH → test FAILURE. These pins therefore genuinely exercise the cosigner-family own-origin path that every prior explicit-BIP-48 vector left untested — closing round-1 M-1.

## New findings

**Minor (non-blocking, NOT in the reviewed diff — does not gate P3a).**

- **m-1 (informational) — `cargo clippy --all-targets` is RED, but ONLY from untracked scratch files outside the fold.** The round-2 prompt's `--all-targets` invocation fails with 2 `clippy::format_collect` errors in `crates/mnemonic-toolkit/examples/idsearch_bench.rs:403` (and a sibling `addrsearch_bench.rs`). Both files are **untracked** (`git status` → `?? crates/mnemonic-toolkit/examples/`), absent from `origin/master`, and absent from the reviewed range `8967294d..HEAD` — ad-hoc developer/benchmark scratch (almost certainly the round-1 engine-enumeration probes). They are NOT part of P3a. Clippy over the tracked surface (`--tests --bins --lib -- -D warnings`) is exit **0**. This does not affect the verdict, but the scratch `examples/` dir should be `.gitignore`d or deleted before any commit/CI run so `--all-targets` doesn't trip the gate on unrelated code. (No code-under-review change required.)

The round-1 M-2/M-3/M-4 minors remain as previously dispositioned (M-2/M-4 within design intent; M-3 is the KEYED full-policy path, explicitly P3b — and the new FOLLOWUP plus the P3b routing comment at `:299-300` confirm the deferral is tracked). None block P3a.

## FOLLOWUP

`design/FOLLOWUPS.md:36-44` carries `template-multisig-own-account-range-subset-search` (`open`, tier `feature`/`next-cycle`), cross-citing the P3a R0 review. The "Fix (deferred)" body (`:41`) accurately describes the deferred work: a subset-aware engine enumerating `P(pool,n)` injective k-permutations (own-subset selection across an account range + which-slots-are-mine), the **unknown own-slot-count** UX, re-routing `--own-account-max` through it, and sizing `realized_s = P(pool,n)`. The "What"/"P3a disposition" (`:40`) correctly records the gentle-gate resolution + the `pool.len()==n` / `realized_s=n!` invariant + names both refuse tests. Accurate and complete.

## Closing verdict

**GREEN — 0 Critical, 0 Important.** I-1 is structurally closed (the engine cannot be fed an over-supplied pool; over-supply and `--own-account-max` refuse loudly at input with exit-1/2 actionable messages, never a silent exit-4 NO-MATCH or wrong wallet); `realized_s` fidelity is restored to `n!`; the fold weakened no round-1-sound invariant (C1 untouched, floors tightened, single-sig non-regressed); the M-1 BIP-87 pins are non-vacuous against the family-blind `canonical_origin`; and the FOLLOWUP captures the deferred subset-search precisely. Tests 18/158/13 green, tracked-surface clippy clean, mlock.rs untouched. The only loose end (m-1) is untracked scratch outside the diff. **P3a may close; advance to P3b.**
