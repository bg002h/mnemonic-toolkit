# IMPLEMENTATION PLAN — `bundle --md1-form=template` MULTISIG + general-policy (#28 phase 2)

**Date:** 2026-06-20 · **SPEC (R0-GREEN, 3 rounds):** `design/SPEC_bundle_md1_template_multisig_2026-06-20.md` + `design/agent-reports/template-multisig-spec-r0-round{1,2,3}-review.md`. Brainstorm (R0-GREEN, 4 rounds): `…BRAINSTORM…` + the C1 advisory.
**Source SHAs (grep-verify at execution — they decay):** mnemonic-toolkit `219468e4` (master), md-codec **0.37.0** (registry lib, `54dd765`) / **mk-codec `0.4.0`** (linked lib), mnemonic-gui (latest pin), mk-cli v0.10.0.
**SemVer:** toolkit **MINOR** `0.59.1 → 0.60.0` (additive: multisig template emit + completion + verify + new completion flags). **md-codec/mk-codec NO-BUMP.** GUI MINOR paired. No publish gate (toolkit ships via git tag).

## 0. Gate + discipline
Per-phase TDD (RED-first) + a per-phase opus R0 to **0C/0I before advancing** (CLAUDE.md). **Funds-safety / silent-wrong-wallet class** — the address-equivalence differential vs an INDEPENDENT golden + the C1 carried-origin-never-loaded invariant + the distinct-keys/strong-prefix floors are the make-or-break gates, not exit-0. Single coupled toolkit PR for the CLI; GUI schema-mirror + manual in lockstep (§Phase 6). Internal sub-phasing within the completion: **canonical multisig (multi/sortedmulti) first, general/thresh second** (SPEC §9) — both under the same SPEC.

## 1. Phase map (each = TDD + per-phase R0)
- **P1 — Permutation-search engine** (standalone module; no CLI surface). The reusable core: `MatchPredicate`, parallel search (`std::thread`, `min(20,ncpu)`), the realized-S sizing, the adaptive cap, ascending-address-outer order, the distinct-keys + strong-prefix primitives. Unit-tested in isolation (no md-codec completion yet).
- **P2 — EMIT (Slice 1)** — `bundle --md1-form=template` admits multisig/general; C1-conditional origin; binding; the loud warning.
- **P3 — RESTORE completion (Slice 2, R0-HEAVY)** — the net-new mk1 origin intake + per-slot origin BUILD + the three completion modes wired to the engine + the funds-safety floors. Canonical-first then general/thresh.
- **P4 — VERIFY-BUNDLE (Slice 3)** — `verify_multisig_template` + the intake + short-circuit.
- **P5 — differential + property + corpus tests** (the funds-safety gate, default-CI + opportunistic bitcoind).
- **P6 — GUI schema-mirror + manual lockstep + version + ship.**

---

## 2. P1 — Permutation-search engine (`src/permutation_search.rs`, new)
- **Impl:** a module taking `(keys_with_origins: Vec<(Key65, Origin)>, slots: N, predicate: MatchPredicate)` and returning `SearchOutcome::{Unique(assignment), None, Ambiguous}`. `MatchPredicate` enum: `WalletPolicyId(prefix)` | `Address(scriptpubkey, chain, range)`. Parallel across `min(20, std::thread::available_parallelism())` threads (no rayon in deps). Ascending-address-index-outer for address-search (§SPEC 6.3); plain `N!` permutation enumeration (Heap's or lexicographic next_permutation) for id-search; the `--own-account-max K` subset enumeration multiplies the own-key candidates. Adaptive cap: micro-calibrate per-candidate cost, estimate exhaustive `S × per-cand-cost`, progress bar/ETA, 1-hour ceiling, `--accept-search-time` override with forced acknowledgment. Early-terminate on match; Ctrl-C abort.
- **Funds-safety primitives:** `reject_duplicate_keys(&[Key65])` (pairwise 65-byte, floor 2); `required_prefix_bytes(S) = ceil((log2(S)+32)/8)` (floor 5 / I_new); ambiguity (≥2) + no-match → refuse.
- **TDD (RED):** the realized-S byte ladder (S=N!→8B at N=11, K=32→11B); duplicate-key rejection; ambiguity/no-match refusal; a synthetic `MatchPredicate` resolves the unique assignment over a small N; the cap calibration + override-acknowledgment. Reuse the throwaway benches (`examples/{idsearch,addrsearch}_bench.rs`) as the cost model (decide: commit as `#[ignore]` benches or keep local — SPEC §9).

## 3. P2 — EMIT (Slice 1; `synthesize.rs`)
- **Impl:** `synthesize_template_descriptor` — replace the three single-sig guards (A `n!=1` `:987-994`; B `cli_template_from_tree().is_none()` `:1005-1012`; C `canonical_origin().is_none()` `:1013-1021`) with a **template-admissible** gate (admit non-taproot multisig/general + shipped `tr(NUMS,multi_a)`; refuse `tr(sortedmulti_a)`/hardened). **C1-conditional origin (§SPEC 3.2):** per-`@N`, `canonical_origin(tree).is_some()` → `Shared(empty)`; `is_none()` → write the source per-`@N` origins (`Divergent` via `synthesize_unified:899-908`). Null `tlv.pubkeys`/`tlv.fingerprints`; preserve use-site + #25 overrides; generalize the single-slot card back-half (`:1047-1078`) to N cosigners. Binding stub already form-generic (`bundle.rs:1151-1159`). D7 `WalletPolicyId` print (full hex + `to_phrase` + 4-byte) + the **loud N!/asymmetric warning** (§SPEC 3.4) on stderr.
- **TDD (RED):** canonical multisig template byte-identical across two seeds + two accounts + `md decode` round-trips; **general-policy (degrade2) template `md decode` round-trips** (carried origins — FAILS with empty origins, the C1 regression pin); `tr(sortedmulti_a)`/hardened refused; the warning fires for order-dependent, softened for sortedmulti.

## 4. P3 — RESTORE completion (Slice 2, R0-HEAVY; `cmd/restore.rs`)
- **P3.1 routing carve-out** (`restore.rs:1655-1661`): keyless template md1 + `--from` → new completion; without `--from` → refuse (floor 1(i)).
- **P3.2 net-new mk1 origin intake (I-B):** the unassigned `--cosigner <mk1>` parse (today `:1955-1990` requires `@N=` + reads only `supplied65`); decode each `--cosigner` mk1 → `KeyCard` → `(key65, origin_fingerprint, origin_path)` (mk-codec 0.4.0 `key_card.rs:36/42/53`). Build the `--account` LIST parse (own multi-account) + `--own-account-max K`.
- **P3.3 per-slot origin BUILD (I-A, the funds-safety core):** for each candidate, BUILD a fresh `path_decl` (`Divergent`) from the permuted `(key,origin)` pairs — own from `--account`/`--origin` honoring ACTUAL purpose (NOT `compute_default_origin_path`'s BIP-48), cosigner from mk1 `origin_path`; use-sites from the template slots. The carried template `path_decl` is NEVER loaded (C1 invariant). Compute the id/address on the fresh descriptor.
- **P3.4 the three modes → the P1 engine:** id-search (`--expect-wallet-id`, strong-prefix), address-search (`--search-address` + range + chain), explicit (`--cosigner @N=` + warning, no search), sorted-shape carve-out (no search; address or BIP-67-normalized id). Mode precedence (§SPEC 2). Distinct-keys floor (P1) before the search.
- **Internal sub-phase:** P3a canonical multisig (multi/sortedmulti) → per-phase R0 → P3b general/thresh.
- **TDD (RED):** the §SPEC 7 floors — no-seed/unsupplied-slot/swapped-`@N` refuse; `@0==@1` mk1 refuse; 4-byte prefix/ambiguous/no-match refuse; the **degrade2 (BIP-84) completion succeeds, compute_default_origin_path-build FAILS** (I-A pin); multi-account own `--account 0,1,2,3` resolves all 4 own slots; address-search finds a non-zero-index target.

## 5. P4 — VERIFY-BUNDLE (Slice 3; `cmd/verify_bundle.rs`)
- **Impl:** add `--from`/`--cosigner`/the search flags to `VerifyBundleArgs`; an early short-circuit for a keyless multisig/general template bundle; `verify_multisig_template` (mirror `verify_singlesig_template:478`) running the same P3 completion engine + asserting card↔template-id binding + the completed id/address. Reuse the P1 engine + P3 origin-build.
- **TDD:** verify-bundle recomposes + asserts consistency for a template bundle (canonical + degrade2); `--expect-wallet-id`/`--search-address` parity with restore.

## 6. P5 — Funds-safety differential + property + corpus
- **(default-CI) address-equivalence:** completed addresses == an INDEPENDENT golden (the full-policy bundle of the same wallet, via rust-miniscript `derive_receive` — not md-codec reconstruction); divergent + degrade2 shapes; anti-vacuity (≠ a wrong assembly).
- **(opportunistic) bitcoind differential** corpus row (`#[ignore]`/env-gated).
- **property** (`prop_backup_restore_roundtrip.rs`): a multisig template round-trips faithfully.

## 7. P6 — Locksteps + version + ship
- **GUI schema-mirror** (`mnemonic-gui/src/schema/mnemonic.rs`): add the new restore + verify-bundle flags (`--from`/`--cosigner` on verify-bundle, `--account`/`--own-account-max`/`--search-address`/`--search-addr-min/max`/`--search-chain`/`--accept-search-time`) + any dropdown (`--search-chain` values) + pin bump; `cargo test --test schema_mirror` GREEN; GUI MINOR; **do NOT cargo fmt the GUI.**
- **Manual** (`docs/manual/src/40-cli-reference/41-mnemonic.md`): the new flags + a "Multisig template completion" section + narrow `### Unrestorable descriptor shapes` (multisig templates now restorable); `make -C docs/manual lint` GREEN.
- **Version** toolkit `0.60.0`: Cargo.toml, BOTH READMEs, install.sh self-pin, fuzz/Cargo.lock, Cargo.lock, CHANGELOG. fmt `cargo +1.95.0 fmt -p mnemonic-toolkit` then `git checkout -- …/mlock.rs` (g6). **Per-phase R0 + mandatory post-impl whole-diff adversarial exec review.** Ship (commit → ff master → tag `mnemonic-toolkit-v0.60.0` → push).
- **Housekeeping:** flip the `bundle-md1-template-only-option` umbrella + the single-sig entry; update `restore-multisig-cosigner-scope` §11 I4 + the SeedHammer `constellation-template-only-engraving`.

## 8. Risks / per-phase R0 focus
- **P3 is the load-bearing R0** (silent-wrong-wallet): the per-slot origin BUILD (I-A — NOT compute_default_origin_path), the carried-origin-never-loaded invariant (C1), the flow-inversion not weakening the full-policy path, the distinct-keys + strong-prefix floors, the swapped-`@N` reject.
- **The mk1 origin intake (I-B)** is net-new — confirm mk-codec 0.4.0 `KeyCard` exposes the fields at execution.
- **Realized-S sizing** must use the actual `--own-account-max` enumeration, not a fixed N!.
- Per-phase R0 + the post-impl adversarial exec review gate each phase before advancing.

## 9. Open execution-time items
- Re-grep ALL citations against the execution base SHA (toolkit/md-codec/mk-codec/gui — they decay).
- Confirm `mk-codec 0.4.0` `KeyCard.{origin_fingerprint,origin_path,xpub}` + `mk_codec::decode`.
- The `permutation_search` module's exact API + the `--cosigner` assigned/unassigned clap parse + the `--account` list parse.
- Whether to commit the `examples/{idsearch,addrsearch}_bench.rs` as `#[ignore]` benches.
- The exact admission predicate for "template-admissible" shapes (generalized `cli_template_from_tree`).
