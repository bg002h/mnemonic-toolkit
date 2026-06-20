# BRAINSTORM — `bundle --md1-form=template` MULTISIG + general-policy (#28 phase 2)

**Date:** 2026-06-20 · Working draft; decisions LOCKED (user-approved). Pending the mandatory opus R0 (0C/0I gate) before SPEC.
**Source SHAs (grep-verify at SPEC time):** mnemonic-toolkit `cbdadbb7` (master, v0.59.1), descriptor-mnemonic md-codec **0.37.0** (`54dd765`), mnemonic-key mk-cli `3258271` (v0.10.0).
**Ingested recon:** `design/cycle-prep-recon-bundle-md1-template-multisig.md` (primary-source-verified; the other instance's recon — its FACTS are folded here; where its DECISIONS conflict with the decisions below, **the decisions below supersede** per the user directive).
**Predecessor:** single-sig template shipped v0.59.0 (#28 phase 1). **Dependency #25 (per-`@N` faithful reconstruction) SHIPPED** (v0.58.2) → the phase-2 gate is clear.

---

## 1. What this is + the C1 "inversion" it resolves

Extend `bundle --md1-form=template` from single-sig to **multisig and general-policy** wallet shapes: emit a keyless, account+key-agnostic template md1; teach `restore`/`verify-bundle` to **complete** a concrete watch-only wallet from the template + externally-supplied cosigner keys + the operator's own seed.

**Why phase 1 deferred it (C1):** single-sig completion is safe (the seed re-derives the one key). **Multisig completion grafts N keys the operator supplies externally**, and for any order-dependent shape there are **N! ways to assign N keys to the N `@N` slots — only one is correct**, and the keyless md1 carries no pubkeys/fingerprints to validate against. A wrong assignment **silently builds a different wallet**. This brainstorm resolves C1 with a loud operator warning **plus** an active **search** that *finds and verifies* the correct assignment.

**The ordering crux (decisive):** order-dependence is **sorted-vs-unsorted**, not multi-vs-everything. `sortedmulti`/`sortedmulti_a` (BIP-67 sort) → all N! assignments collapse to ONE wallet (safe, no search). `multi`/`multi_a`/**`thresh`**/general miniscript with ≥2 keys → N! distinct wallets. For **asymmetric general policies** (e.g. `wsh(or_d(pk(@0),and_v(v:pk(@1),older(144))))`) a wrong assignment changes each key's **spending semantics** (which key is timelock-gated), not merely the address — strictly more dangerous than symmetric multisig.

---

## 2. Decisions (LOCKED)

- **D1 — Scope = canonical multisig + general policies (incl. `thresh`, timelocks, asymmetric branches).** (Broader than the recon's "canonical multisig only" — supersedes.) **Inherits the full-policy refusal floors:** `tr(sortedmulti_a)` refused (umbrella-gated renderer, `to_miniscript.rs:584-586`), hardened use-site refused (#25 — underivable from an xpub). In-scope: non-taproot wsh/sh(wsh) general+multisig, and the shipped `tr(NUMS,multi_a)`.
- **D2 — Completion = THREE resolution paths feeding ONE permutation-search engine** (supersedes the recon's explicit-only model):
  - **id-search** — match the recorded `WalletPolicyId` (`--expect-wallet-id`).
  - **address-search** — match a known address among a configurable index range (`--search-address` + range; §3). The pragmatic path: a user *always* has a receive address but may never have recorded the id.
  - **explicit** — `--cosigner @N=<mk1>` per-slot assertion + the loud warning; no search/verification.
  - `sortedmulti`/`sortedmulti_a` → order-free → no search (any assignment is the same wallet). A supplied ADDRESS verifies the key *set* in one evaluation; a supplied recorded-id must FIRST BIP-67-normalize the key order before recomputing (R0-I3 — the id is order-sensitive even though the address isn't).
- **D3 — Loud stderr warning** (the user's core ask) at BOTH emit and restore for order-dependent shapes: the `N!` count (N = distinct `@N` slots), "only one assignment is correct," and the asymmetric-semantics caveat for general policies. Softened "order-independent" note for `sortedmulti*`.
- **D4 — Binding stays the key-invariant `WalletDescriptorTemplateId`** (same as single-sig). **Discriminating stub `H(template-id ‖ sorted fingerprints)` REJECTED** — it would bake the cosigner set into the card, defeating the shareable "one engraving for thousands" goal. Cosigner-set discrimination happens at *completion* (id/address match), not in the card.
- **D5 — Adaptive search cap (§3.3).** Default ceiling = **1 hour estimated exhaustive time**, calibrated on the actual machine; progress bar + ETA; early-terminate on match; Ctrl-C abort; **override allowed but forces an explicit acknowledgment of the full exhaustive-search time.** Time-based → auto-scales to hardware.
- **D6 — Address-search refinements (§3.2):** configurable index **range `[lo,hi]`** (iterative deepening); default **ascending-address-index-outer** order (low indices first — provably optimal, §3.2); **no user order-toggle in v1** (YAGNI; future multi-address feature is where permutation-outer earns its keep); default **receive chain (0)**, change (chain 1) opt-in.

---

## 3. The permutation-search engine

One engine; the search *space* and the *match predicate* differ per strategy. Parallel across `min(20, available_parallelism())` threads (the cap that bound the benchmarks).

### 3.1 Strategies + disambiguation (funds-safety)
- **id-search:** space = the `N!` permutations; predicate = `compute_wallet_policy_id(candidate) == expected`. **Disambiguation (R0-I2, load-bearing — id-search is runtime-WEAKER than address-search):** `--expect-wallet-id` is a flexible-length prefix (#28 D7). A SEARCH over `N!` candidates can yield a LONE spurious prefix-match even when the *true* assembly is ABSENT (wrong key set) — which would be silently accepted as a wrong wallet (the refuse-on-≥2 rule only fires if a *second* candidate also collides). So a search REQUIRES a **strong prefix: `≥ ceil((log2(N!) + 32)/8)` bytes, or simply the full 16-byte id** (= **8 bytes at N=11, 9 at N=13**; margin ≥32 bits → P(spurious) ≤ ~2e-12 even at 11!; a 4-byte prefix is ~1-in-110, unacceptable). **REFUSE on ambiguous (≥2 matches), REFUSE on no-match, and on a unique match SURFACE the completed full 16-byte `WalletPolicyId` on stderr** for an out-of-band eyeball check. id-search is the opt-in/recorded-id path; **address-search is the RECOMMENDED primary** (runtime collision-free — next bullet).
- **address-search:** space = `N!` × index range × chain(s); predicate = candidate's address at `(chain,idx)` equals the target's **scriptPubKey** (decode the target once, compare the 256-bit P2WSH program). For non-sorted shapes distinct assignments → distinct scripts → distinct addresses (R0-verified: children serialize in *stored slot order*, `astelem.rs:155-188`; only `Sorted*` reorders via BIP-67), so a match is **cryptographically unique** (no prefix-length issue) — strictly more robust at runtime than a short id prefix.
- **Sorted shapes (R0-I3):** for `sortedmulti`/`sortedmulti_a` the id is permutation-SENSITIVE (`compute_wallet_policy_id` never BIP-67-sorts at hash time, `identity.rs:189-228`) while the address is permutation-INVARIANT. So a recorded-id verify on a sorted shape MUST first **BIP-67-normalize the supplied key order before recomputing the id** (else a correct wallet supplied in a different valid order FALSE-REFUSES); the address path needs no normalization (one evaluation) → prefer it for sorted shapes.
- Both: a UNIQUE strong match → complete that assignment; ZERO matches → refuse loudly (wrong keys / wrong id / out-of-range index); never guess.

### 3.2 Address-search range + order
- **Range `[lo,hi]`** (`--search-addr-min`/`--search-addr-max`); operator deepens (`0–N`, then `N–2N`, …). A narrow range also expresses "I know the index" (e.g. `48–52`).
- **Default order = ascending-address-index OUTER, all permutations INNER.** Optimal by joint probability: the match cell `(perm*, idx*)` has `idx*` heavily skewed low (sequential issuance) and `perm*` uniform → visiting low indices first minimizes expected time-to-match; permutation-outer wastes a full range-sweep on each of ~`N!/2` wrong permutations (≈50× worse for a depth-100 range). Also implementation-cheaper: incremental per-index child precompute (derive the N keys' children at the current index once, sweep all permutations at the structure-once cost, advance).
- **Chain:** default receive (0); change (1) opt-in (`--search-chain`; doubles per-index cost).

### 3.3 Adaptive cap (D5)
At search start, micro-calibrate per-candidate cost on this machine (actual thread count), estimate the **exhaustive** space `N! × (hi−lo) × chains × per-candidate-cost`. `< ~30s` → run silently; up to the **1-hour** ceiling → run with progress/ETA; `>` ceiling → refuse unless overridden, and the override prints + forces acknowledgment of the estimated exhaustive time. Early-termination makes the *typical* case far faster than the worst-case estimate the cap guards.

---

## 4. Funds-safety model + floors

The completion is the silent-wrong-wallet surface. Floors (non-negotiable, R0 targets):
1. **No silent wrong assembly.** Every order-dependent completion is either (a) search-resolved to a UNIQUE strong match (id per §3.1-I2, or address), (b) operator-asserted via explicit `--cosigner @N=` *with* the loud warning, or (c) refused. **Two HARD gates:** (i) a **swapped `@N` must be rejected** by the search modes (no match) and is the operator's risk in explicit mode (warned); (ii) **every slot must be supplied** — the union of the `--from`-resolved own position + all `--cosigner @N` must equal `0..n`, ELSE REFUSE (R0-MINOR-2: in the inverted BUILD flow an unsupplied slot has NO source; promote the today-advisory `all_verified` check `restore.rs:2032-2034` to a hard refuse — closes the v0.44.0 Phase-2-R1 "unsupplied slot marked verified" regression at the structural level). TDD pins both.
2. **Distinct cosigner keys (R0-I1, HARD floor).** REJECT duplicate supplied cosigner keys (pairwise-compare the N supplied 65-byte keys before the search). Two slots given the SAME key (paste one mk1 twice) collide on BOTH address AND id — swapping them is a no-op → a degenerate/insecure multisig (a "2-of-3" that is really 2-of-2 with a reused signer) would build with no warning, and the ambiguity rule would false-refuse or silently pick one. No dedup guard exists today (md-codec `canonicalize.rs:420-474`; toolkit restore). TDD: `@0`==`@1` mk1 → refused.
3. **Flow inversion is security-load-bearing.** Today `--cosigner @N=`/`--from` *cross-check* md1-sourced pubkeys (`restore.rs:1987` vs `c.key65`); the template path must make them the **build source** (the md1 is keyless). The inversion must not weaken any existing cross-check on the *full-policy* path.
4. **Address-equivalence differential (TEST-side).** The completed descriptor's first addresses verified against an INDEPENDENT golden (the full-policy bundle of the same wallet, through rust-miniscript — not md-codec's reconstruction). This is a differential TEST that a wrong assembly cannot pass — **NOT a runtime guard** (the runtime guards are floors 1+2 + the §3.1 disambiguation rigor).
5. **Disambiguation rigor** (§3.1): id-search strong-prefix requirement (margin ≥32 bits or full id) + ambiguity-refusal + surfaced-full-id; address-search full-scriptPubKey match (collision-free, recommended primary).

---

## 5. As-built facts (recon, folded — verified at the SHAs above)

- **md-codec + mk-cli NO CHANGE (proven).** `md encode 'wsh(sortedmulti(2,@0/**,@1/**,@2/**))'` (no `--key`) emits a keyless template (`pubkeys:None`); `Body::MultiKeys{k,indices}` key-independent (`tree.rs:115-139`); `compute_wallet_descriptor_template_id` (`identity.rs:71-104`) recurses multisig → stable distinct ids (live: 2-of-3=`b02b4403`, 3-of-3=`a227f95e`). `to_miniscript_descriptor` errors `MissingPubkey` (`to_miniscript.rs:122`) on a keyless template (the recompose boundary). mk-cli `derive_stub_from_md1` already form-aware (template-id for `!is_wallet_policy()`, N-agnostic).
- **Emit delta (Slice 1, S):** lift the THREE guards in `synthesize_template_descriptor` (A `descriptor.n != 1`, `synthesize.rs:987-994`; B `cli_template_from_tree(&tree).is_none()`, `:1005-1012`; **C `canonical_origin(&tree).is_none()`, `:1013-1021`** — rejects non-canonical/custom-origin wrappers, R0-MINOR-1) → replace with a multisig-shape admission gate; **generalize the keyless origin-elide from the single-slot `PathDeclPaths::Shared(empty)` (`:1023-1032`) to N slots** — reuse what `synthesize_unified:899-908` ALREADY builds (`Shared` if origins equal, else `Divergent(origin_paths)`), then null pubkeys/fingerprints; also **generalize the single-slot card back-half** (`:1047-1078`, loops `cosigners[0]`/`MkField::Single`) to N cosigners. Threshold k / sortedmulti / N slots preserved for free (unmutated `descriptor.tree`). Card binding stub already form-generic + a slot loop (`bundle.rs:1151-1159`, csi `:1212-1217`).
- **Restore (Slice 2, M, R0-heavy):** keyless multisig md1 is NOT reconstructible from the md1 alone — `restore.rs:1655-1661` gates `!is_wallet_policy()` → `ModeViolation`; `expand_per_at_n` + `e.xpub.ok_or` fails (`:1752-1761`); RED pin `cli_restore_md1_template.rs:206`. N cosigner keys MUST come externally. The flow-inversion (§4.2). `mk1` carries no `@N` (`mk-codec key_card.rs:24-54`) → operator-asserted `@N=` is the only key→slot source for explicit mode; the search modes try all assignments.
- **verify-bundle (Slice 3, M):** has `verify_singlesig_template` (`verify_bundle.rs:478-608`) to mirror; the multisig path (`:883-976`) sources keys from `md1.tlv.pubkeys` and fails on a template (`:2474-2489`); verify-bundle has NEITHER `--from` NOR `--cosigner` → needs `verify_multisig_template` + early short-circuit + new external-key intake.

---

## 6. Benchmark data (drives the cap; `examples/idsearch_bench.rs` + `examples/addrsearch_bench.rs`, degrade2.desc 11-key timelock multisig, 20 threads / 24 ncpu)

| Strategy | µs/candidate | 11! (39.9M) @ 20 threads |
|---|---|---|
| id-search (`WalletPolicyId`) | 6.9 | ~14 s |
| address-search `addr[0]` | 7.4 (1.1×) | ~15 s |
| address-search first-10 | 75 | ~2.5 min |
| address-search first-100 | 750 | ~25 min |

Realistic address-search uses **structure-once** (build the per-index `Descriptor` template once; per candidate = `translate_pk` of precomputed children + one `sha256` → scriptPubKey) — 108× faster than naive per-candidate secp derivation, ~4-5× faster than rebuild-precomp; cost ~linear in addresses-per-candidate. **The SPEC pins structure-once as the production primitive** (R0-MINOR-4) so the adaptive-cap estimate uses the right per-candidate cost. At ~170M candidates/min the 1-hour ceiling ≈ ~13.5! on this box and auto-scales down on slow hardware. **Both primitives are pure structure+hash (no per-candidate secp on the hot path)** — confirmed faithful (addresses byte-identical to `md address`/rust-miniscript; permutations yield distinct ids/addresses; a target at index 50 is found by the depth-100 search).

---

## 7. SemVer / locksteps / housekeeping

- **Toolkit MINOR** (additive: multisig template emit + completion + verify; new restore/verify-bundle flags — `--search-address`, `--search-addr-min/max`, `--search-chain`, verify-bundle's `--from`/`--cosigner` intake; `restore` may reuse its existing `--cosigner`/`--from` — confirm). **md-codec/mk-codec NO-BUMP** (re-pin 0.37.0 is a release-ritual touch only).
- **Locksteps:** EMIT none. **verify-bundle's new flags → GUI `schema_mirror` (`mnemonic-gui/src/schema/mnemonic.rs`) + manual mirror (`docs/manual/src/40-cli-reference/`) paired in the SAME PR** (CLAUDE.md). Any new restore flag likewise.
- **Housekeeping (on completion):** flip the `bundle-md1-template-only-option` umbrella + the single-sig entry (never flipped); update `restore-multisig-cosigner-scope` §11 I4 carve-out; update the SeedHammer `constellation-template-only-engraving` recon (this UNBLOCKS the fork-side multisig template engrave).
- **3-slice ordering:** Slice 1 EMIT → Slice 2 RESTORE completion (the R0-heavy funds-safety core) → Slice 3 VERIFY-BUNDLE. Sizing M–L.

## 8. Open SPEC-time items (re-grep / decide at SPEC)
- Re-grep all `synthesize.rs`/`restore.rs`/`verify_bundle.rs`/`bundle.rs` line numbers (they decay).
- Confirm whether `restore` reuses existing `--cosigner`/`--from` (no new flag) or needs new ones; settle the exact new flag names + the address-range default (e.g. `0..20` snappy default vs `0..100`).
- Settle the cap-override-acknowledgment UX (D5). *(The id-search strong-prefix rule is DECIDED — §3.1-I2: margin ≥32 bits / full id, ambiguity-refuse, surface-full-id.)*
- Where the **duplicate-key rejection** (floor 2 / I1) lives — toolkit restore intake (preferred: it owns the external-key intake) vs a md-codec distinctness check.
- The **sortedmulti BIP-67-normalize-before-id** (I3) implementation — reuse rust-miniscript's BIP-67 ordering, or use the address path for sorted shapes.
- Decide the origin-elide generalization: Divergent-arm support vs canonical-shared-origin gate; account for Guard C (canonical-origin) + the N-slot card back-half (MINOR-1).
- Engine placement: a reusable `permutation_search` module (shared by restore + verify-bundle, parametrized by match predicate) vs inline.
- The address-search `--search-chain both` + multi-address (future — where a permutation-outer order toggle would earn its keep) — note as deferred.
