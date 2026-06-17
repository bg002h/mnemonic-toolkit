# cycle-prep recon — 2026-06-12 — minor coverage cluster (GAP 5, 4 items)

**Origin SHAs at recon time:** descriptor-mnemonic `origin/main` = `422b049`; mnemonic-toolkit `origin/master` = `ca7d7bc`. Both local branches in sync with origin (`git status -sb`: no ahead/behind on either).

Items verified: (1) `multi` n=17..=20 never render/address-tested (md-codec); (2) `after` no positive golden (md-codec); (3) hash256/ripemd160/hash160 property-only (md-codec); (4) verify-bundle general-policy/BIP-388 cell gap (toolkit). Verdict preview: 1–3 ACCURATE; 4 is PARTIALLY STRUCTURALLY-WRONG (timelock general policies ARE already round-trip tested in verify-bundle; the real residue is hashlocks + BIP-388 intake, and the cited FOLLOWUP slug is already resolved).

---

## Per-item verification

### Item 1 — valid `multi` n=17..=20 never render/address-tested (md-codec)

**Verdict: ACCURATE — and the cheap fix is unblocked (the "key pool" rationale in the cap comment is stale).**

- `crates/md-codec/tests/common/mod.rs:884-888` T-tier cap — **ACCURATE.** Comment: "n ≤ 16 cap (the miniscript limit is 20; 17..=20 is P7 territory because the TLV-attached key pool caps n at 16)" followed by `let wide_multi = t_multi_node(Tag::Multi, 2, 16);` (:888). Enforced by the T-tier key-budget assertion at `:1024-1028` (`(1..=16).contains(&next), "T-tier key budget violated … (cap 16)"`).
- **BUT the comment's REASON is stale:** `tests/common/mod.rs:361` `test_xpubs()` returns `&'static [[u8; 65]; 32]` — the pool has **32** keys, not 16. And `descriptor_with_pubkeys` (`:388-394`) asserts `(1..=32).contains(&n)` with the doc-comment "Usable for any n in 1..=32 (P7 oversize-multi cells go above the T-tier 16 cap)". The 16 cap is a deliberate T-tier budget choice, NOT an infra limit. A deterministic 17–20 cell needs zero new key material.
- `tests/proptest_to_miniscript.rs:519-527` `p7_oversize_multi_refuses_cleanly(n in 21u8..=32, …)` — **ACCURATE** (P7 starts at 21).
- Wire-tier DOES cover 17..20: `tests/common/mod.rs:103-119` `n_strategy()` includes `Just(17)`, `Just(31)`, `Just(32)`, `2u8..=32`, consumed by the W-tier strategies at `:277/:287/:294` (encode/decode round-trip only). **No test anywhere renders / reparses / derives an address for n∈17..=20** — confirmed: the only render+address paths are the T-tier property (cap 16) and the `self_test_*` goldens (max n exercised is small).
- **Fix estimate:** 1–2 `self_test_*` cells in `tests/proptest_to_miniscript.rs`, e.g. `descriptor_with_pubkeys(wrap(Tag::Wsh, multikeys(Tag::Multi, 17, (0..20).collect())))` → `p6_chain` → golden `bc1q…` literal. ~18 LOC per cell; n=20/k=17 alone covers the upper boundary; an optional n=17 cell covers the cap+1 edge. Note: 17–20 is valid ONLY under `wsh` (legacy `sh` redeem-script 520-byte limit caps multi at 15 keys) — the cell must be wsh.
- While in there: fix the stale "key pool caps n at 16" clause at `:886` (pool is 32; the cap is the T-tier budget). One-line comment edit.

### Item 2 — `after` has no positive deterministic/golden coverage (md-codec)

**Verdict: ACCURATE.**

- `tests/proptest_to_miniscript.rs`: `Tag::After` appears ONLY in negatives — `self_test_bad_after_zero` (`:411`), `self_test_bad_after_bit31` (`:424`) — the P7 refusal property (`:494`), and the tag allow-lists (`:738`, `:774`). No positive `self_test_*` golden.
- `tests/address_derivation.rs`: Tier-3 arbitrary-miniscript golden is `wsh(and_v(v:pk(@0),older(144)))` only (`:845`, `:887`, golden `bc1qcr8te4…` at `:995`). Zero `after(` occurrences.
- `tests/bitcoind_differential.rs`: corpus entry 9 is `wsh(and_v(v:pk, older(144)))` (`:332-334`); no `after` anywhere in the corpus.
- P6 property coverage exists (typed strategy generates `After` when `abs_time` is on), so a mis-render would likely be caught by the rust-miniscript differential — the golden adds an **oracle-independent address anchor** (the stated purpose of the `self_test_*` cells, per the `p6_chain` doc-comment :40-48).
- **Fix estimate:** 1 cell, ~18 LOC: `self_test_wsh_and_v_pk_after_N` mirroring `self_test_wsh_and_v_pk_older_144` (`:136`) with `timelock(Tag::After, N)` (pick a sane height, e.g. 800000) + golden literal.

### Item 3 — hash256/ripemd160/hash160 addresses are property-only (md-codec)

**Verdict: ACCURATE.**

- Golden inventory (`grep "fn self_test_" tests/proptest_to_miniscript.rs`): the ONLY hashlock golden is `self_test_tr_nums_and_v_sha256_pk` (`:173-190`, golden `bc1psldl66…`). No hash256/ripemd160/hash160 golden anywhere (also zero hits in `address_derivation.rs` and `bitcoind_differential.rs`).
- All four hashlocks ARE in the P6 property strategies — segwit `:490-493` (`hash32(Tag::Sha256/Hash256)`, `hash20(Tag::Ripemd160/Hash160)`) and tap-leaf `:798-801` — so this is golden-anchoring, not a never-exercised window.
- **Fix estimate:** 3 cells, ~55 LOC total, mirroring the sha256 cell byte-for-byte (`descriptor_with_pubkeys(tr_node(true, 0, Some(node2(Tag::AndV, wrap(Tag::Verify, hash32/hash20(Tag::X, [0x..; N])), keyarg(Tag::PkK, 0)))))` → `p6_chain` → golden). hash256 takes `hash32`, ripemd160/hash160 take `hash20` (constructors exist: `common/mod.rs:76`/`:82`).

### Golden-cell pattern (items 1–3 share it)

`self_test_*` cells live in `tests/proptest_to_miniscript.rs` (NOT `address_derivation.rs`). Pattern: build the tree with the `common/mod.rs` constructors → `descriptor_with_pubkeys(tree)` (attaches real BIP-86 abandon-mnemonic xpubs from the 32-key OnceLock pool) → `addr = p6_chain(&d)` (asserts converter success + wire round-trip + reparse fixed-point + address derivation, `:49-…`) → `assert_eq!(addr, "bc1…")` hardcoded literal. **Derive-once-then-hardcode:** write the cell with a placeholder literal, run it, copy the address from the assertion failure, pin it. The literals are the oracle-independent anchor (deliberately NOT computed at test time).

### Item 4 — verify-bundle general-policy / BIP-388 cell (toolkit)

**Verdict: PARTIALLY STRUCTURALLY-WRONG premise; residue is real but smaller + one half is a feature gap, not a test gap.**

- **Timelock general-policy coverage ALREADY EXISTS** — the "coverage = single-sig + wsh-sortedmulti + wsh-andor multisig" framing undersells the andor cells, which carry both timelock kinds:
  - `tests/cli_verify_bundle_multi_cosigner_mk1.rs:248` — `wsh(andor(pkh(@0),after(12000000),pk(@1)))` round-trips via bundle JSON (Cell 3).
  - `:372` + `:438` — 3-cosigner `wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))` (Cells 6+; descriptor-mode with all phrases).
  - `tests/cli_verify_bundle_entropy_slot.rs:16` — `NONCANONICAL_DESC = "tr(NUMS,and_v(v:pk(@0),after(12000000)))"` (taproot + timelock).
  - A "general-policy timelock cell" sub-goal is therefore **already satisfied — drop it**.
- **Hashlock verify-bundle coverage is genuinely absent:** zero `sha256|hash256|ripemd|hash160` hits across all `tests/cli_verify_bundle*.rs`. NOT blocked: toolkit `src/parse_descriptor.rs` has Terminal arms for all four hashlocks (`:639-650`) + sha256 template tests (`:2245`, `:2271`, `:2340` `arm_sha256`). A `wsh(and_v(v:sha256(H),pk(@0)))`-class round-trip cell mirroring `non_canonical_wsh_andor_round_trips_via_bundle_json` is test-only, ~40 LOC.
- **BIP-388 policy intake is a FEATURE gap, not a test gap:** `src/cmd/verify_bundle.rs` descriptor mode goes straight to `parse_descriptor` (`:651` intake, `:709` canonicity probe, `:1017` final parse) with NO `is_bip388_policy_shape` probe. The v0.49.0 expansion (`expand_bip388_policy`) is wired into `bundle.rs`, `export_wallet.rs`, `xpub_search/descriptor_intake.rs`, and shared `wallet_import/pipeline.rs` only. A leading-`{` policy fed to `verify-bundle --descriptor` fails as a descriptor parse error today. A "BIP-388 verify-bundle cell" therefore means either (a) a pinned-refusal cell documenting current behavior (test-only, ~20 LOC) or (b) mirroring the bundle.rs probe→expand into verify_bundle's intake (small PATCH feature, ~30 LOC + cells, reuses the shared expander). `tests/cli_bip388_policy_intake.rs` covers export-wallet/bundle only — confirmed no verify-bundle cell.
- **FOLLOWUP slug `verify-bundle-descriptor-entropy-slot-gap`** (`design/FOLLOWUPS.md:350`) — location ACCURATE but **Status: `resolved` mnemonic-toolkit-v0.43.1** (Entropy arm added + `tests/cli_verify_bundle_entropy_slot.rs`, 5 tests). It is NOT an open sub-gap; do not cite it as motivation in a brainstorm. No open FOLLOWUP covers verify-bundle BIP-388 intake — if (b) is deferred, file a new slug (suggest `verify-bundle-bip388-policy-intake`).

---

## Assessment

**Ranked by real silent-mis-render risk vs theoretical completeness:**

1. **Item 1 (multi 17–20)** — the only item where a VALID window has NO render/reparse/address testing at all (P6 property never generates it; wire-tier only). A render bug in the 17–20 window would today ship silently. Highest marginal value, trivial cost. **DO.**
2. **Item 4 reduced (verify-bundle hashlock cell + BIP-388 refusal pin)** — hashlock descriptors are a shipped verify-bundle surface (v0.54.x made general-policy restore faithful) with zero verify-bundle coverage; the BIP-388 pin documents a real intake asymmetry vs bundle/export-wallet. **DO (reduced scope); drop the timelock sub-goal (already covered); BIP-388 expansion itself → FOLLOWUP, not this cycle.**
3. **Item 2 (after golden)** — P6 property + rust-miniscript differential already exercise `after`; the golden adds the oracle-independent anchor `older` already has. Near-zero cost, real (if modest) value: catches a future upstream `after`-rendering behavior shift the differential can't (both sides would move together). **DO.**
4. **Item 3 (3 hashlock goldens)** — same class as item 2 (property-covered, golden-anchoring only). Cheapest per-cell (copy the sha256 cell 3×). **DO.**

**Drop list:** nothing dropped wholesale. Dropped sub-parts: item 4's timelock-general-policy cell (already exists) and item 4's BIP-388 *expansion* (feature work → new FOLLOWUP slug; only the pinned-refusal cell ships in this cluster).

**Bundling:** TWO cycles, split by repo (different repos, different commit/CI streams; gluing them buys nothing):

- **Cycle 5a (md-codec, items 1+2+3):** one file touched (`tests/proptest_to_miniscript.rs`, + one comment line in `tests/common/mod.rs`), 5–6 new `self_test_*` cells, ~110 LOC. NO-BUMP test-only.
- **Cycle 5b (toolkit, item 4 reduced):** 1 hashlock round-trip cell (pattern: `cli_verify_bundle_multi_cosigner_mk1.rs` Cell 3) + 1 BIP-388 pinned-refusal cell + file `verify-bundle-bip388-policy-intake` FOLLOWUP. ~60 LOC + FOLLOWUP entry. NO-BUMP test-only.

Both cycles are small enough to share one working session; keep commits/tags per-repo.

---

## Recommended scope

**Verdict: all 4 items proceed, with item 4 re-scoped (hashlock cell + BIP-388 refusal pin + new FOLLOWUP; timelock sub-goal dropped as already-covered).**

- **Tier:** NO-BUMP test-only in both repos — UNLESS any new golden reveals an actual mis-render (then that item escalates to a PATCH fix cycle in md-codec; treat the golden derivation step as the detection gate). SemVer: none. No clap surface change → no GUI `schema_mirror` lockstep, no manual mirror, no sibling companions (except the one NEW toolkit FOLLOWUP slug, toolkit-local, no companion needed — the expander is already shared toolkit-side).
- **Ordering:** 5a before 5b is natural (5a's golden-derivation runs may surface md-codec issues that would inform 5b's hashlock cell), but they are independent — either order works.
- **Brainstorm-spec corrections to carry:** (i) cite the 32-key pool (`common/mod.rs:361`) and `descriptor_with_pubkeys` 1..=32 acceptance (`:388-394`) — do NOT repeat the stale "pool caps at 16" rationale; (ii) do NOT cite `verify-bundle-descriptor-entropy-slot-gap` as an open gap (resolved v0.43.1); (iii) the wsh-andor timelock cells at `cli_verify_bundle_multi_cosigner_mk1.rs:248/:372/:438` are the existing-coverage baseline for item 4. Cite source SHAs: md-codec `422b049`, toolkit `ca7d7bc`.
