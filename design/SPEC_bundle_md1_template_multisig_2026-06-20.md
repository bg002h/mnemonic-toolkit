# SPEC — `bundle --md1-form=template` MULTISIG + general-policy (#28 phase 2)

**Date:** 2026-06-20 · **UNCOMMITTED until R0-GREEN.** Pending the mandatory opus R0 (0C/0I) before the plan-doc.
**Brainstorm (R0-GREEN, 4 rounds — RED r1/r3 → GREEN r4):** `design/BRAINSTORM_bundle_md1_template_multisig_2026-06-20.md` + `design/agent-reports/template-multisig-brainstorm-r0-round{1,2,3,4}-review.md` + the C1 advisory `template-multisig-c1-origin-advisory.md`. Incorporates the origin/use-site model (D7–D10), C1 (conditional origin handling), I_new (realized-S prefix). Ingested recon: `design/cycle-prep-recon-bundle-md1-template-multisig.md`.
**Source SHAs (grep-verify at plan time):** mnemonic-toolkit `e97e8470` (master; src unchanged since `cbdadbb7`, design-only commits), md-codec **0.37.0** (`54dd765`; the toolkit's linked lib dep). **mk-codec `0.4.0`** is the toolkit's linked lib (`Cargo.toml:35`; the `KeyCard.{origin_fingerprint,origin_path,xpub}` fields the I-B intake reads are in 0.4.0); mk-cli v0.10.0 (`3258271`) is the standalone CLI binary.
**SemVer:** mnemonic-toolkit **MINOR** (additive). **md-codec / mk-codec NO-BUMP** (recon-verified: keyless multisig/general template encode + key-stable `WalletDescriptorTemplateId` + form-aware mk-cli stub already exist). **Dependency #25 SHIPPED** (per-`@N` reconstruction, v0.58.2) — phase-2 gate clear. Predecessor: single-sig template v0.59.0.

---

## 1. Scope

Extend `bundle --md1-form=template` from single-sig to **multisig and general policies** (`multi`/`sortedmulti`/`thresh`/timelocks/hashlocks/asymmetric branches; non-taproot wsh/sh(wsh) + the shipped `tr(NUMS,multi_a)`). Emit a keyless, account+key-agnostic template md1; complete a concrete watch-only wallet at `restore`/`verify-bundle` from externally-supplied cosigner keys + the operator's seed via a permutation-search engine (§6).

**Refusal floors inherited from the full-policy path:** `tr(sortedmulti_a)` refused (umbrella-gated renderer, `to_miniscript.rs:584-589`); hardened use-site refused (#25 — underivable from an xpub).

**3 slices, ordered:** Slice 1 EMIT (S) → Slice 2 RESTORE completion (M, R0-heavy) → Slice 3 VERIFY-BUNDLE (M).

---

## 2. Flag surface

**EMIT — `bundle`:** no new flag. `--md1-form=template` (exists) now admits multisig/general shapes; the loud warning (§3.4) + the `WalletPolicyId` print (§3.3) go to stderr.

**COMPLETION — `restore` AND `verify-bundle`** (both gain the same intake; `verify-bundle` has neither `--from` nor `--cosigner` today — both new there):
- `--from <seed>` — the operator's own seed (REQUIRED for a template completion; §7-floor-1).
- `--account <N[,N…]>` (LIST; D10) — the account(s) the OWN seed is used at (one own key per account; e.g. `--account 0,1,2,3` for degrade2's 4 own slots); default `0`. `--origin <path>` (exists) — an explicit own origin overriding the canonical default. `--own-account-max <K>` (new; D10) — RANGE fallback when the own accounts are unknown: derive the own seed at `0..K` and let the search select the subset used (enlarges S → I_new prefix sizing, §6.2).
- `--cosigner <mk1|xpub>` (repeatable) — **unassigned** cosigner key (search modes); `--cosigner @N=<mk1|xpub>` (repeatable) — **assigned** key (explicit mode). The presence of `@N=` selects the mode (§4.3). Cosigner origins come from the `mk1` cards (`origin_fingerprint`+`origin_path`; mk1 carries origin, NOT use-site — verified `mk-codec key_card.rs`).
- `--expect-wallet-id <hex>` (exists on BOTH restore + verify-bundle, single-sig-only today — the MULTISIG matching is the new wiring; M1) — recorded `WalletPolicyId`; triggers **id-search**. For a multisig SEARCH it MUST be a strong prefix (§6.2).
- `--search-address <addr>` (new) — a known wallet address; triggers **address-search**.
- `--search-addr-min <u32>` / `--search-addr-max <u32>` (new) — address-index range `[min,max)` (default `0..20`; §6.3).
- `--search-chain <receive|change|both>` (new, default `receive`).
- `--accept-search-time <duration>` (new) — cap override; must be ≥ the estimated exhaustive time (§6.4), which the tool prints and the operator must restate (forced acknowledgment).

**Mode/trigger precedence at completion:** (a) `--expect-wallet-id` → id-search; (b) else `--search-address` → address-search; (c) else all keys given as `--cosigner @N=` → explicit (Mode B) + warning; (d) else REFUSE (`bad`): "supply a recorded `--expect-wallet-id`, a `--search-address`, or explicit `--cosigner @N=` assignments." Sorted shapes (§6.1) skip the search.

**Locksteps:** the new completion flags on BOTH `restore` and `verify-bundle` are clap-surface changes → **GUI `schema_mirror` (`mnemonic-gui/src/schema/mnemonic.rs`) + manual mirror (`docs/manual/src/40-cli-reference/41-mnemonic.md`) updated in the same PR** (§8).

---

## 3. Slice 1 — EMIT (`synthesize_template_descriptor`, S)

### 3.1 Guard-lift + shape admission
Replace the three single-sig-only guards in `synthesize_template_descriptor` (`synthesize.rs`) with a multisig/general admission gate:
- **A** `descriptor.n != 1` (`:987-994`) — DROP the `n==1` restriction; admit `n ≥ 1`.
- **B** `cli_template_from_tree(&tree).is_none()` (`:1005-1012`) — generalize: the single-sig template-shape recognizer must become a **template-admissible** check that accepts multisig/general non-taproot shapes (and the shipped `tr(NUMS,multi_a)`), still **refusing** `tr(sortedmulti_a)` (the `to_miniscript.rs:584-589` render gap) and hardened use-site (#25).
- **C** `canonical_origin(&tree).is_none()` (`:1013-1021`) — do NOT hard-refuse on this; it drives the C1-conditional origin handling (§3.2): admit non-canonical multisig/general shapes, writing the source origins (not refusing).
- Refusals → `ToolkitError::TemplateFormUnsupportedShape` (exists; no new variant).

### 3.2 Keyless N-slot mutation — C1-CONDITIONAL origin handling (D7)
The keyless mutation currently hardcodes the single-slot `path_decl.paths = PathDeclPaths::Shared(OriginPath{empty})` (`:1023-1032`). **Eliding to empty only decodes when `canonical_origin(tree).is_some()`** — `md decode`'s `validate_explicit_origin_required` (md-codec `validate.rs:221-245`) rejects a non-canonical wrapper with empty origins (`MissingExplicitOrigin`). So generalize CONDITIONALLY per `@N`:
- `canonical_origin(tree).is_some()` (canonical multisig `wsh(multi/sortedmulti)`, `sh(wsh)`) → **elide to `Shared(empty)`** (byte-identical-shareable, like single-sig).
- `canonical_origin(tree).is_none()` (general policy — `thresh`/`or_i`/timelocks, e.g. degrade2) → **write the source's real per-`@N` origins** (`Divergent` when accounts differ; `synthesize_unified:899-908` already builds this) so decode accepts it.

In BOTH cases the carried/elided origin is **decode + display ONLY** (origins are re-supplied at completion, §4; the template-id is origin-invariant — `identity.rs:71-104` hashes only tree + use-site + use-site-overrides, pinned `wdt_id_invariant_*` — so binding is unchanged either way). Then null `tlv.pubkeys`/`tlv.fingerprints`; preserve the per-`@N` use-site structure (incl. #25 overrides — they're in the template-id). Generalize the **single-slot card back-half** (`:1047-1078`, loops `cosigners[0]`/`MkField::Single`) to N cosigners. Threshold k / sortedmulti / N slots preserved for free (unmutated `descriptor.tree`).

### 3.3 Binding + `WalletPolicyId` print (unchanged model)
Binding stays the **key-invariant `WalletDescriptorTemplateId`** (no discriminating stub — brainstorm D4): `bundle_binding_stub` is already form-generic (`bundle.rs:1151-1159`, `!is_wallet_policy()` → template-id) and the csi is already a slot loop (`:1212-1217`); mk1 reflects template-id; ms1 unchanged. Print the order-sensitive **`WalletPolicyId`** (full 16-byte hex + `to_phrase` + 4-byte prefix) on stderr — the completion checksum the creator records.

### 3.4 The loud warning (brainstorm D3 — the user's core ask)
On stderr at emit for an **order-dependent** shape (anything but `sortedmulti`/`sortedmulti_a`): the `N!` assignment count (N = distinct `@N` slots), "only one assignment reproduces this wallet," and the **asymmetric-semantics** caveat ("for a general policy a wrong assignment changes each key's SPENDING ROLE, not just the address — record the `WalletPolicyId` above and/or a receive address to complete safely"). Softened "order-independent" note for `sortedmulti*`.

---

## 4. Slice 2 — RESTORE completion (M, the R0-heavy funds-safety core)

### 4.1 Routing carve-out
Today `restore --md1` gates a keyless multisig md1 → `ModeViolation` (`restore.rs:1655-1661`; RED pin `cli_restore_md1_template.rs:173-222`). Add a template-completion branch: a keyless template md1 (`!is_wallet_policy()`) with `--from` present → the new completion path; a keyless md1 WITHOUT `--from` → refuse (§7-floor-1).

### 4.2 The flow inversion (security-load-bearing)
Today keys are BUILT from the md1 (`expand_per_at_n` over `md1.tlv.pubkeys`, `restore.rs:1751-1784`) and `--cosigner @N=` only CROSS-CHECKS (`:1987`, `supplied65 != c.key65` → exit 4). The template path must **INVERT** this: BUILD the `ResolvedSlot`s from the externally-supplied keys (`--from` own slot(s) + `--cosigner` keys) — the md1 carries no pubkeys. The inversion must not weaken any cross-check on the full-policy path (that path is untouched; the template path is a new branch).

### 4.2a Origin + use-site assembly (D7–D10; the C1 funds-safety invariant) — NET-NEW per-slot assembly (R0-r1 I-A/I-B)
The N keys = **own** (`--from` seed at each `--account` in the list / `--origin`; possibly SEVERAL — one seed → many accounts, D10) + **cosigners** (`mk1` cards). **The completion BUILDS a FRESH descriptor** from the template's tree + use-site structure (incl. #25 overrides) + the supplied keys — it does NOT consume the template's carried `path_decl` (that's decode+display-only, §3.2). Per-slot assembly under each search candidate (the permuted key→slot assignment):
- **Each key carries its OWN origin** — the search's permutable unit is the **`(key65, origin)` PAIR**, not the bare key. Own slots → `--account N`/`--origin`, **honoring the ACTUAL purpose** (degrade2 is BIP-84 `m/84'/0'/N'`, so the own origin must **NOT** be forced through `compute_default_origin_path`'s hardcoded BIP-48 `m/48'/…/2'`). Cosigner slots → each `mk1`'s `origin_fingerprint`+`origin_path`.
- **Use-sites come from the template SLOTS** (per-`@N`, fixed; mk1 carries no use-site — verified `mk-codec key_card.rs`). The search permutes the `(key,origin)` pairs across slots; use-sites stay per-slot → resolves "which cosigner is which slot" AND "which slots are mine" at once.
- **Write the assembled per-slot origins into the completion descriptor's `path_decl`** (`Divergent` when they differ — degrade2 accounts 0–3); compute the id/address on THAT (M2: `path_decl` is the write sink, matching the existing rebuild; `OriginPathOverrides` is not needed since `path_decl` is built fresh).

**NET-NEW code (I-B — flag at plan-time; does NOT exist today):** (1) the **unassigned `--cosigner <mk1>` parse** (today's `--cosigner @N=` parse `restore.rs:1955-1990` reads only `supplied65` for the cross-check, never the origin); (2) decode each `--cosigner` mk1 → `KeyCard` → `(key65, origin_fingerprint, origin_path)`; (3) the **per-slot origin BUILD from the supplied keys** — NOT `compute_default_origin_path`. The `verify_bundle.rs:~1080-1150` canonical-default rebuild is a REFERENCE for *where* `path_decl` is set, **NOT the origin SOURCE** (it hardcodes BIP-48 + reads no mk1 origins, so degrade2 would never match under it); the existing `verify_bundle.rs:1907-1944` mk1 read is an advisory cross-check on the KEYED path, not an origin source.

**C1 INVARIANT (funds-safety):** because the completion builds `path_decl` from the supplied keys' origins (never loading the template's carried `path_decl`), a carried/stale origin can NEVER reach `compute_wallet_policy_id`/derivation. A carried-vs-supplied disagreement is a SOFT advisory (D9), never a reject (degrade2 is legitimately BIP-84-in-a-multisig).

### 4.3 The three completion modes
- **id-search** (`--expect-wallet-id`): §6 engine; predicate = `compute_wallet_policy_id(candidate) == expected` (strong-prefix, §6.2).
- **address-search** (`--search-address`): §6 engine; predicate = candidate's scriptPubKey at some `(chain,idx)` in range == target's scriptPubKey.
- **explicit** (all `--cosigner @N=`): no search; build directly from the asserted assignment; fire the §3.4 warning; no verification (operator's risk).
- **sorted shapes**: no search (any assignment → same wallet); a supplied address verifies the set in one eval; a supplied recorded-id must BIP-67-normalize the supplied key order before recompute (§6.1, brainstorm I3).

On a UNIQUE strong match (search) → emit the watch-only descriptor (via the #25 per-`@N` reconstruction path, which the search already exercises). On no-match/ambiguous → refuse (§6.2).

---

## 5. Slice 3 — VERIFY-BUNDLE (M)

`verify-bundle` has `verify_singlesig_template` (`verify_bundle.rs:478`) but no multisig-template path; its multisig path sources `desc.tlv.pubkeys` → template-only failure (`:2475-2479`); `VerifyBundleArgs` (`:23-159`) has neither `--from` nor `--cosigner`. Add:
- An **early short-circuit** for a keyless multisig/general template bundle.
- `verify_multisig_template` (mirrors `verify_singlesig_template`): take the new intake (`--from` + `--cosigner` + the search options, §2), run the same completion engine (§4/§6), recompose the watch-only wallet, assert card↔template-id binding + the completed `WalletPolicyId`/address. Support `--expect-wallet-id`/`--search-address` the same way.
- The new intake flags trip the GUI + manual lockstep (§8).

---

## 6. The permutation-search engine

A reusable module (plan-time §9: shared by restore + verify-bundle, parametrized by a `MatchPredicate`), parallel across `min(20, available_parallelism())` threads (the benchmark cap).

### 6.1 Search space + sorted carve-out
- Space = the `N!` bijections of the supplied keys onto the N distinct `@N` slots (N = distinct placeholders; reused `@N` is one slot). Address-search multiplies by the index range × chain(s).
- **Sorted shapes** (`sortedmulti`/`sortedmulti_a`, detected on the tree leaf tag): order-independent → **N! collapses to 1**; complete directly. Verification: address (one eval) or **BIP-67-normalize the supplied key order before id-recompute** (`compute_wallet_policy_id` is order-sensitive — `identity.rs:172-229`, never sorts; the address routes through `new_*_sortedmulti` which DOES sort — `to_miniscript.rs:402-409`).

### 6.2 Match predicates + disambiguation (FUNDS-SAFETY)
- **id-search:** `compute_wallet_policy_id(candidate)` matches the `--expect-wallet-id` prefix. **Strong-prefix REQUIRED, sized to the REALIZED search space (I_new):** `≥ ceil((log2(S) + 32)/8)` bytes (or the full 16-byte id), where **S = realized candidate count** — `N!` for an explicit `--account` LIST, the larger `P((N−own)+K, N)` subset×permutation space for `--own-account-max K`. Sized-byte ladder (N=11, own=4): list/`N!`→8B, K=8→9B, K=16→10B, K=32→11B, K=64→13B (P(lone-spurious | true-absent) ≤ ~2e-10 across the range; a fixed 8B would hit ~1-in-275 at K=32 — must size from S). **REFUSE on ambiguous (≥2 matches)**, **REFUSE on no-match**, and on a unique match **print the completed full 16-byte `WalletPolicyId` on stderr**. id-search is runtime-WEAKER than address-search; **address-search is the recommended primary** (full 256-bit scriptPubKey — collision-free regardless of S). (Scoped to the multisig SEARCH — does NOT change #28's single-sig flexible-length `--expect-wallet-id`, a single-candidate recompute-and-match at `restore.rs:867-911`.)
- **address-search:** decode the target address → scriptPubKey ONCE; per candidate compare the candidate's `(chain,idx)` scriptPubKey. For non-sorted shapes children serialize in stored slot order (rust-miniscript `miniscript/astelem.rs:155-188` — the toolkit address-derivation dep, NOT md-codec; the md-codec WIRE-layer order property is `tree.rs:115-139`; only `Sorted*` reorders) → distinct permutations → distinct 256-bit P2WSH program → **cryptographically unique match** (no prefix issue). REFUSE on no-match.

### 6.3 Address-search range + order
- Range `[min, max)` (`--search-addr-min/max`, default `0..20`); operator deepens (`0–20`, then `20–40`, …); a narrow range expresses "I know the index."
- **Order: ascending-address-index OUTER, all permutations INNER** (the only order; no v1 toggle — brainstorm D6). Optimal by joint probability (idx skewed low, perm uniform) and implementation-cheaper (incremental per-index child precompute). Chain: default `receive` (0); `change`/`both` opt-in (doubles per-index cost).
- **Production primitive = structure-once** (build the per-index `Descriptor` template ONCE; per candidate = `translate_pk` of precomputed children + one `sha256` → scriptPubKey). ~7.4µs/candidate (vs naive 108× / rebuild-precomp ~4-5× worse).

### 6.4 Adaptive cap (brainstorm D5)
Micro-calibrate per-candidate cost on this machine (actual thread count) at search start; estimate the **exhaustive** space `N! × range × chains × per-candidate-cost`. `< ~30s` → run silently; up to the **1-hour** ceiling → run with a progress bar + ETA (early-terminate on match; Ctrl-C abort); `>` ceiling → REFUSE unless `--accept-search-time <duration ≥ estimate>` is passed, which forces the operator to restate the printed exhaustive-time estimate (acknowledgment). Time-based → auto-scales to hardware.

---

## 7. Funds-safety floors (testable; the R0 + impl-review targets)

1. **No silent wrong assembly.** Every completion = (a) UNIQUE strong search match, (b) explicit `--cosigner @N=` + warning, or (c) refuse. **Hard gates:** (i) `--from` REQUIRED (no-seed template completion → refuse); (ii) **every slot supplied** — union of `--from` own position + all `--cosigner @N` (or the search's resolved assignment) == `0..n`, else REFUSE (promote the today-advisory `all_verified` `restore.rs:2030-2042` to a hard refuse — closes the v0.44.0 "unsupplied slot marked verified" regression); (iii) a swapped `@N` is rejected by the search modes (no match), warned in explicit mode.
2. **Distinct cosigner keys (HARD floor).** REJECT duplicate supplied keys (pairwise 65-byte compare BEFORE the search) — two slots given the same key collide on BOTH address AND id (a "2-of-3" that is secretly 2-of-2). Does NOT over-reject legitimate same-`@N` multi-leaf reuse (one slot, one key).
3. **Flow inversion preserves full-policy cross-checks** (§4.2) — the full-policy path is untouched.
4. **Address-equivalence differential (TEST-side, not runtime).** The completed descriptor's first addresses == an INDEPENDENT golden (the full-policy bundle of the same wallet, via rust-miniscript — not md-codec reconstruction).
5. **Disambiguation rigor** (§6.2): id-search strong-prefix sized to the REALIZED S (I_new) + ambiguity/no-match refuse + surfaced full id; address-search full-scriptPubKey.
6. **Origin handling (D7–D9, C1).** Origin canonicality is NOT a floor — origins are supplied at completion (§4.2a) and may be non-canonical (degrade2 = BIP-84 in a multisig); the gate is the origin-canonicality-agnostic address/id match. **Per-slot origins are BUILT FRESH from the supplied keys** (own `--account`/`--origin` honoring actual purpose; cosigner mk1 `origin_path`) — NOT from `compute_default_origin_path` (which would force BIP-48 and make degrade2 never match; I-A) — and written into the completion descriptor's `path_decl` (`Divergent`). The template's carried `path_decl` is never loaded into completion → it cannot reach `compute_wallet_policy_id`/derivation (§4.2a). TDD pins: a general-policy keyless template DECODES (carried origins) while a canonical one decodes (elided); a stale/wrong carried origin cannot produce a silently-accepted wallet.

### Test inventory (RED-first)
- Emit: keyless **canonical** multisig template md1 byte-identical across two seeds + two accounts (account-agnostic, elided origins) + `md decode` round-trips; keyless **general-policy** template (degrade2) `md decode` round-trips (carried per-`@N` origins — would FAIL with empty origins, the C1 regression pin); `tr(sortedmulti_a)`/hardened refused.
- Binding: mk1/md1 share the template-id stub; template↔policy cross-reject.
- Search correctness: id-search + address-search each resolve the UNIQUE correct assignment for the degrade2-class policy; a swapped `@N` → no-match → refuse; a target at a non-zero index found within range.
- **I-A origin-build pin (load-bearing):** a **degrade2 (BIP-84-origin) completion** succeeds — the own origin built from `--account` honors purpose 84' (NOT forced BIP-48); cosigner origins read from the mk1 `origin_path`. A build that sourced origins from `compute_default_origin_path` (BIP-48) must FAIL the search (proving the SPEC's per-slot-build is implemented, not the canonical-default rebuild). Multi-account own (`--account 0,1,2,3`) resolves all 4 own slots.
- **Floor 1(i/ii/iii):** no-seed → refuse; an unsupplied slot → refuse; swapped `@N` → refuse (search) / warned (explicit).
- **Floor 2:** `@0`==`@1` mk1 → refuse.
- **Floor 5:** id-search with a 4-byte prefix → refuse (too weak); ambiguous → refuse; no-match → refuse; unique → surfaces full id.
- sortedmulti: any supplied order completes to the SAME wallet (address); a raw recorded-id check normalizes before recompute (no false-refuse).
- Differential: completed addresses == independent full-policy golden (the funds-safety gate); bitcoind `deriveaddresses` corpus row (opportunistic, `#[ignore]`).
- Non-regression: single-sig `--md1-form=template` (#28) byte-identical; `--md1-form=policy` byte-identical.

---

## 8. SemVer / locksteps / housekeeping

- **Toolkit MINOR** (additive emit + completion + verify + the new completion flags). **md-codec/mk-codec NO-BUMP** (re-pin 0.37.0 is release-ritual only). Version sites per `project_toolkit_release_ritual_version_sites`.
- **Locksteps (same PR):** the new `restore`+`verify-bundle` completion flags → GUI `schema_mirror` (`mnemonic-gui/src/schema/mnemonic.rs`) + manual mirror (`docs/manual/src/40-cli-reference/41-mnemonic.md`) + the `### Unrestorable descriptor shapes` prose (multisig templates now restorable). EMIT (`--md1-form`, existing) has no schema delta.
- **Housekeeping (on completion):** flip the `bundle-md1-template-only-option` umbrella + the single-sig phase entry (never flipped); update `restore-multisig-cosigner-scope` §11 I4; update the SeedHammer `constellation-template-only-engraving` recon (UNBLOCKS the fork-side multisig template engrave).

## 9. Open plan-time items (re-grep / decide)
- Re-grep all `synthesize.rs`/`restore.rs`/`verify_bundle.rs`/`bundle.rs`/md-codec line numbers against the plan-time base SHA (citation-decay).
- Origin handling DECIDED (C1-conditional §3.2: canonical→`Shared(empty)`; general→carry source per-`@N` origins for **DECODE-VALIDITY ONLY**). At completion the per-slot `path_decl` is **BUILT FRESH from supplied-key origins** (§4.2a) — the `verify_bundle.rs:~1080-1150` rebuild is reused as the write-SITE only, **NOT the origin source** (origins are NOT `compute_default_origin_path`). Plan-time: the Guard-C relaxation + the carried-origin-**never-loaded** invariant test (the carried template `path_decl` is never loaded into the completion descriptor, the stronger form of the overwrite invariant).
- **NET-NEW per-slot origin assembly (I-A/I-B — the plan's biggest new surface):** the unassigned `--cosigner <mk1>` parse; decode `--cosigner` mk1 → `KeyCard` → `(key65, origin_fingerprint, origin_path)`; BUILD per-slot `path_decl` from the permuted supplied-key origins (own `--account`/`--origin` honoring actual purpose; cosigner mk1 origin) — NOT `compute_default_origin_path`. The `(key65, origin)` pair is the search's permutable unit.
- The `permutation_search` module shape + the `MatchPredicate` trait; parallelism harness (`std::thread`; no rayon in deps — confirm). The realized-S prefix sizing (I_new) + the `--own-account-max` subset enumeration.
- Exact new flag names + the `--cosigner` assigned-vs-unassigned parse + the `--account` LIST parse + `--own-account-max` (D10); the `--accept-search-time` acknowledgment UX; the `0..20` default range; the soft origin advisory (canonicality + carried-vs-supplied, D9).
- The duplicate-key floor placement (toolkit restore intake vs md-codec); the sortedmulti BIP-67-normalization impl.
- Internal impl phasing (canonical multisig first, general/thresh second) even while the SPEC covers the full scope — propose at plan time.
- Whether to commit the `examples/{idsearch,addrsearch}_bench.rs` (currently local scratch) as reproducible benches with embedded vectors.
