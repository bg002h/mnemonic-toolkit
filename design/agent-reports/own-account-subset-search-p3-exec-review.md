> Reviewer: opus architect — P3 per-phase R0 EXECUTION review (own-account-subset-search, commit `d1fc4a35`)

**Verdict: GREEN — 0 Critical, 0 Important.**

P3 (opt-in `--search-cosigner-subset`) is funds-safe and may advance to P4. All three make-or-break gates hold: the opt-in path ranks over INDEPENDENT-golden-correct subsets, the bound genuinely refuses a DoS-sized pool BEFORE any calibration/derivation blow-up, and off-by-default is byte-invariant (the 3-way branch does not perturb the exact or own-only arms). Evidence below.

---

### Scope / churn
`git diff --name-only d1fc4a35^..d1fc4a35` = 4 files: `cmd/restore.rs` (+163/−33), `cmd/verify_bundle.rs` (+3), `derive_slot.rs` (−1, the stale `#[allow(dead_code)]` removal), `tests/cli_restore_md1_template_multisig.rs` (+285). NO Cargo.toml/Cargo.lock/mlock churn, no new deps, no cross-file fmt churn. Clean.

### 1. Headline opt-in golden — NON-VACUOUS (PASS)
`search_cosigner_subset_completes_with_extra_cosigner` (test ~L1038): a 2-of-3 `wsh-multi` `{A@0, B@0, C@0}`, operator supplies own A + real cosigners B,C + ONE EXTRA outsider card (`SEED_OUTSIDER`), `--search-cosigner-subset`, full `--expect-wallet-id`. Asserts the completed receive addresses == `golden_addresses(...)`. The golden is an INDEPENDENT rust-miniscript build — `Descriptor::<DescriptorPublicKey>::from_str("wsh(multi(2,...))")` + `into_single_descriptors()` + `derive_at_index()` (test L232–270), NOT an md-codec reconstruction — built from ONLY the 3 real cosigners. The outsider is NOT in the golden, so a passing assertion proves the search SELECTED `{B,C}` and DROPPED the outsider. Reproduced end-to-end: the test passes.
- **Anti-vacuity floor confirmed two ways.** (a) `search_cosigner_subset_anti_vacuity_missing_true_cosigner_no_match` (L1067): pool lacks a true cosigner (C replaced by two outsiders) → asserts exit code **4** AND stderr contains "NO MATCH" — a genuine search NO-MATCH (`RestoreMismatch`), not a parse/input refuse; the opt-in search REACHED the full space and found no reproducing subset. Passes. (b) Complementary direction — the pre-existing `own_only_over_supplied_cosigners_refuses` (L851): the SAME over-supply WITHOUT the flag REFUSES up front (BadInput, naming `--search-cosigner-subset`), proving the flag is load-bearing (over-supply is impossible on the default path). Passes.

### 2. `realized_s == s_opt` (enumerated ≡ counted) — I-1 floor (PASS)
The opt-in `realized_s` (restore.rs L1831) is `ps::s_opt(k_own, m_cosigners, n, sorted)` with `sorted = is_order_independent_shape(&d.tree)` (L1830). The engine's `Enumeration::OptIn { k_own, m_sup: m_cosigners, n, sorted: sorted_shape }` (L1955–1961) has cardinality `s_opt(k_own, m_sup, n, sorted)` (permutation_search.rs L871–877) with `sorted_shape = is_order_independent_shape(&d.tree)` (L1944) — the SAME function over the SAME `d.tree`, and `k_own`/`m_sup=m_cosigners`/`n=d.n` are the SAME variables. So `realized_s` (drives prefix-strength L1999 + the cap L2018/2085) EQUALS `enumeration.cardinality()` (drives the engine's `[0,S)` rank range, permutation_search.rs L1012–1014) byte-for-byte. NOT `s_own`, NOT `n!`. The OptIn `unrank` (`opt_in_unrank`, L746–793) is P1-proven bijective onto exactly `s_opt`: `opt_in_bijects_s_opt_nonsorted`/`_sorted` (L2088/L2116) assert via `assert_bijects` (L1849) that the generated set over `[0,s_opt)` EQUALS the brute-force reference (no dup, no miss) AND `generated.len() == s_opt` AND `reference.len() == s_opt`. Disjoint `j`-strata (own-slot-count) ⇒ no cross-strata double-count (s_opt.rs L600–617 sums disjoint strata; the unrank locates the stratum by cumulative size L759–789). No scanned-but-uncounted placement.

### 3. The BOUND is real (can't DoS) (PASS)
`search_cosigner_subset_hard_ceiling_refuses` (L1144): 2-of-7 `wsh-multi` (N=7), `--own-account-max 256` (passes the K≤256 gate at L1486 since 256 is not >256 → k_own=256), 2 cosigner cards. **Independently recomputed** s_opt(256, 2, 7, false) = 3,759,210,773,176,320 ≈ **3.76e15 > REALIZED_S_MAX (1e15)** — and the commit's claimed j=6 stratum `C(256,6)·C(2,1)·7! = 3,714,810,645,934,080` matches exactly. The refuse fires in the `realized_s` block (L1843–1848, `s > REALIZED_S_MAX → bad`) which runs BEFORE `reject_duplicate_keys` (L1930), BEFORE `calibrate_per_candidate`/`cap_decision` (run_capped_search L2260/2266), and BEFORE any `unrank`. Test asserts failure with "ceiling"/"exceeds"/"overflow" AND no "panic". Passes.
- **DoS-tightness:** the only work BEFORE the refuse is (i) the own-pool derivation, bounded by `--own-account-max ≤ 256` (gated L1486) or, for an `--account`-list opt-in pool, the re-asserted `k_own > 256 → refuse` (L1811–1815) — 256 BIP-32 derivations, fast; (ii) `s_opt` itself, O(min(k_own, n−1)) overflow-checked `c_choose` ops — trivial. `m_cosigners > 256 → refuse` (L1817–1822) caps the cosigner axis. A hostile pool cannot pin the CPU before the refuse. `m_sup>256`, `K_own>256`, `s_opt`-overflow (`None → bad`, L1832, cardinality-`Some`-first), and `s_opt==0` (ModeViolation "not enough keys", L1833–1841) all refuse with NO `.expect()`/panic on attacker input. The engine's internal `.expect()`s divide the already-validated cardinality (P1 M-1) and cannot fire on an unguarded overflow (cardinality `Some`-checked at search_enumerated L1012 before any unrank).

### 4. Off-by-default BYTE-UNCHANGED (PASS)
All new behavior is gated behind `opt_in = ctx.search_cosigner_subset` (L1684). The three touches to shared arms are pure boolean extensions that reduce to the prior expression when `opt_in=false`: the `@N=` mutex `own_account_max.is_some() || ctx.search_cosigner_subset` (L1605-ish) ≡ old when false; pool-build/early_exit `over_supply || opt_in` (L1738, L2080) ≡ `over_supply` when false; `apply_identity_filter = sorted_shape && !over_supply && !opt_in` (L1975) ≡ old when false. `verify_bundle.rs` sets `search_cosigner_subset: false` (L865) — verify-bundle opt-in path never taken (P4 scope). Full `mnemonic-toolkit` suite **3467 passed / 0 failed / 15 ignored** (= 3460 + 7, exactly the commit's claim). The full multisig-template file (exact v0.60.0 + P2 own-only + 7 new P3) = **42 passed / 0 failed** — the 3-way branch does not perturb the other 2 arms.

### 5. No owned `Xpriv` on the opt-in own pool (PASS)
The `over_supply || opt_in` branch (restore.rs L1738) routes the opt-in own-pool through `derive_accounts_xpub_only` (L1746) — returns `Vec<(Xpub, Fingerprint)>` (derive_slot.rs L272–293); every `Xpriv` (master + per-account) is confined to `ScrubbedXpriv` and scrubbed at scope exit (L313/336/351). The bare `derive_bip32_from_entropy_at_path` path (L1762–1777) runs ONLY on the EXACT arm (`else`). Correct routing.

### 6. Distinct-keys floor covers the COSIGNER-SUBSET collision axis + early_exit gating (PASS)
`reject_duplicate_keys(&pool_key_blobs)` (L1929–1930) runs on the WHOLE opt-in pool (own `0..k_own` + ALL `m_cosigners` incl. the outsider) AFTER pool assembly (L1792–1793) and BEFORE mode selection + search. Distinct cosigner subsets ⇒ distinct key SETS ⇒ distinct scriptPubKey, so address-search `early_exit=true` (L2080) is collision-free on the opt-in path. The outsider card is byte-distinct (own-A + outsider seed at a distinct family) so it does not collide. `early_exit` stays **false** for id-search (L2019) and is only set on address-search (L2080) — prefix-id never reaches `early_exit=true`, preserving the 2nd-match ambiguity certification (full-scan). Verified the address-search opt-in path end-to-end via `search_cosigner_subset_address_search_completes` (L1118, sorted `wsh-sortedmulti`).

### 7. Composition + mutex (PASS)
`search_cosigner_subset_composes_with_own_account_max` (L1162): own@2 over-supplied via `--own-account-max 4` AND an extra cosigner — both axes over-supplied; the combined `s_opt(k_own=4, m_sup=2, …)` resolves own@2 + the `{B}` cosigner subset to the golden. Passes. `@N=` ⊕ flag → BadInput (`search_cosigner_subset_at_n_conflict` L1265; the mutex at L1609 `any_assigned && (own_account_max.is_some() || ctx.search_cosigner_subset)`). Passes. The headline (flag alone, one extra cosigner) degrades sanely.

### 8. C1 preserved (PASS)
On the opt-in path origins are built FRESH via `own_origin_for` (L1714–1728): `--origin` override → cosigner-family-with-account-substituted → canonical BIP-48 fallback WITH the L8 mainnet-coin-type→`network.coin_type()` patch (L1651–1673). `build_candidate` builds descriptors from the permuted `(key, origin, fp)` triples (L1980–1992), NEVER the carried `path_decl`. The opt-in path uses the identical closure; L8 substitution applies identically.

### 9. Weak-prefix refuses over the larger opt-in space (PASS)
`search_cosigner_subset_weak_prefix_refuses` (L1213): a 4-byte `--expect-wallet-id` over the widened opt-in space (`--own-account-max 32` + extra cosigner) refuses via `validate_prefix_strength(prefix.len(), realized_s)` (L1999) — sized to `s_opt`, not `s_own`/`n!`. Passes.

### 10. No regression + clean (PASS)
`cargo test -p mnemonic-toolkit` = **3467 passed / 0 failed / 15 ignored** (ran). `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` = clean (ran after a forced recompile, not a cache hit). The `derive_slot.rs` `#[allow(dead_code)]` removal on `derive_accounts_xpub_only` is correct (it is now a production caller from restore.rs:1746) and clippy stays clean. NEW flag `--search-cosigner-subset` (restore) noted for the **P6 lockstep** (see Minor below).

---

### Critical
None.

### Important
None.

### Minor (non-blocking; do not gate P4)
- **M-a — P6 lockstep is a release gate, track it.** `--search-cosigner-subset` is in the toolkit's clap surface (confirmed in `restore --help` and auto-emitted by `gui-schema`) but NOT yet in `docs/manual/src/40-cli-reference/41-mnemonic.md` (P2's `--own-account-max` IS, count=2) nor the GUI hand-maintained `mnemonic-gui/src/schema/mnemonic.rs`. The commit explicitly DEFERS both to P6. This is correct for P3 (the manual lint is a CI `make` target, not `cargo test`, so the in-repo suite is GREEN), but per CLAUDE.md the manual-mirror + GUI schema_mirror invariants are RELEASE gates — P6 MUST land both before tag, else the manual lint (CI) fails and the GUI drift gate accumulates silently. Just confirm P6 closes it.
- **M-b — stale `#[allow(dead_code)]` on the private xpub-only helpers.** `derive_master_for_xpub_only` (derive_slot.rs L299) and `derive_one_xpub_only` (L323) retain `#[allow(dead_code)]`, but both are now reachable from production via `derive_accounts_xpub_only` (restore.rs:1746). The allows are now superfluous (Rust does not lint redundant allows, so clippy `-D warnings` stays clean — harmless). Optional tidy in a later phase; not a defect.
- **M-c — coverage gap (non-correctness): non-sorted (`multi`) address-search under opt-in is not directly tested.** The opt-in address-search test uses sorted `wsh-sortedmulti`; the non-sorted opt-in address-search early-exit is collision-free by the same structural argument (distinct subset OR distinct ordering ⇒ distinct order-dependent `multi` scriptPubKey), but a direct test would close the matrix. Optional add.

**GREEN (0C/0I). The gate advances to P4.**
