# Brainstorm R0 round 4 — bundle-md1-template-multisig (#28 phase 2) — C1 + I_new fold (opus architect, verbatim)

> Reviewer: opus architect (read+bash; toolkit `f1b0bb1f`/src `cbdadbb7`, md-codec 0.37.0 registry copy + descriptor-mnemonic src, mk-cli v0.10.0). **Verdict: GREEN — 0 Critical, 0 Important.** Round-3 C1 + I_new CLOSED; no prior finding reopened; 3 non-blocking Minors. Brainstorm R0 converged → SPEC-ready.

---

**Verdict: GREEN — 0 Critical, 0 Important.**

## C1 — CLOSED
Round-3 Critical (elide-ALL breaks general-policy decode) closed by the conditional model (§2a-D7, §3-Slice-1, §4-floor-6, §8). Source-verified:
1. **Decode for both branches.** `validate_explicit_origin_required` (`validate.rs:221-245`, called `decode.rs:75`) → `Ok` when `canonical_origin(&tree).is_some()` (`:222-224`), else requires every `@N` non-empty origin (`:225-244`). Canonical multisig → `Shared(empty)`: `canonical_origin` Some for `wsh(multi/sortedmulti)→m/48'/0'/0'/2'` (`canonical_origin.rs:60-62`), `sh(wsh)` (`:67`) → empty elide decodes (test `..._passes_canonical_wpkh:513`). General policy (degrade2 `wsh(or_i(...))`) → `_ => None` (`canonical_origin.rs:78`) → carried per-`@N` origins satisfy it (test `..._passes_divergent_when_all_populated:643`; the empty-elide failure pinned `..._fails_divergent_when_one_idx_empty:668-691` → `MissingExplicitOrigin{idx:1}`). Defect eliminated.
2. **Template-id origin-invariant under carried origins.** `compute_wallet_descriptor_template_id` (`identity.rs:71-104`) hashes only `use_site_path` + `tree::write_node` + `use_site_path_overrides`; never `path_decl`. Carrying real origins → binding byte-identical.
3. **Carried origin cannot reach the id/address preimage as a wrong wallet (no silent-wrong-wallet).** (a) `compute_wallet_policy_id` reads `e.origin_path` from `expand_per_at_n` (`identity.rs:178,191-201`); INVARIANT docblock (`:160-171`): reads `OriginPathOverrides[idx]` first, else `path_decl`, never `canonical_origin` at hash time. (b) `expand_per_at_n` (`canonicalize.rs:436-444`) gives `origin_path_overrides` PRECEDENCE over `path_decl`; completion rebuild overwrites `path_decl.paths` from supplied keys (`verify_bundle.rs:1080-1150` pattern) before the binding loop. (c) The gate is recompute-and-match against the operator's out-of-band record (`restore.rs:867-915`), not the carried origin → a stale carried origin that survived → changed derived id/address → mismatch → refuse. No self-consistent wrong-wallet path.
4. **Carried-vs-supplied disagreement = ADVISORY (D9), not reject.** Necessary (degrade2 = BIP-84 in a multisig). Gate stays the origin-canonicality-agnostic address/id match. mk1 cross-checks (`verify_bundle.rs:1900/1907/1937`) are the advisory surface.

## I_new — CLOSED
§3.1 sizes the id-search prefix to `≥ ceil((log2(S)+32)/8)` bytes, S = realized candidate count (`N!` for explicit `--account` list; `P((N−own)+K,N)` for `--own-account-max K`). Math (N=11, own=4): K=4(list→N!)→8B→2.2e-12; K=8→9B; K=16→10B; K=32→11B→2.2e-10 (vs round-3 fixed-8B ~1-in-275); K=64→13B→5e-12 (vs >1). Sizing-from-S restores the ≥32-bit margin at every K (P ≤ ~2e-10); explicit list collapses S→N! (no over-charge). Address-search unaffected (full 256-bit P2WSH, non-sorted serialize in stored slot order `astelem.rs:155-188` → distinct (subset,perm) → distinct program, prefix-independent). CLOSED.

## Drift/regression
- C1/I_new edits consistent with D1-D6, D8 (override-precedence `canonicalize.rs:436-444` makes supplied-key origins win), D10 (realized-S = D10's enlarged space; distinct-keys floor holds — different account ⇒ different child xpub ⇒ distinct 65-byte key), §4 floors, §3 engine, §5 as-built.
- §3-Slice-1 emit-delta internally consistent — no leftover "elide ALL"; every `Shared(empty)` gated on `canonical_origin==Some` (line 77 even names the reject reason).
- Round-1/2/3 closed findings NOT reopened (I1 distinct-keys, I2 strong-prefix mechanism, I3 sortedmulti, D8/D9). I_new is a SIZING delta on I2, not a reopen. MINOR-1/2 intact.

## Minors (non-blocking, SPEC-time)
1. Line-decay: toolkit citations vs `cbdadbb7`; `verify_bundle.rs:1074-1146` rebuild lives at `:1080-1150` in HEAD. §8 mandates re-grep at SPEC.
2. The `verify_bundle.rs:1080-1150` rebuild is the EXISTING non-canonical multisig pattern; SPEC must make explicit the NEW template-completion path REUSES it (overwrite-before-gate) rather than trusting as-decoded `path_decl`.
3. Add the per-K sized-byte ladder (8/9/10/11/13) to the SPEC for an at-a-glance check.

## Verdict
**GREEN — 0 Critical, 0 Important.** C1 conditional origin model decodes both branches, template-id origin-invariant, carried origin provably overwritten before the recompute-and-match gate → no silent-wrong-wallet; I_new realized-S sizing restores the margin across the `--own-account-max` range, collapses to N! for explicit lists. No prior finding reopened. Clear to proceed to SPEC.
