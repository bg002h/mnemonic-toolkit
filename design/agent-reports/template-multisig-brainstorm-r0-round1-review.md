# Brainstorm R0 round 1 — bundle-md1-template-multisig (#28 phase 2) (opus architect, verbatim)

> Reviewer: opus architect (read+bash; toolkit working tree `99052af6` == recon commit, no drift in reviewed files; md-codec 0.37.0 registry copy; mk-cli v0.10.0). **Verdict: RED — 0 Critical, 3 Important.** Architecture sound; 3 funds-safety floor-tightening folds + 4 Minors. Folded → re-dispatch.

---

**Verdict: RED — 0 Critical, 3 Important.**

## Verified correct

- **NO md-codec/mk-codec change — keyless MULTISIG *and* GENERAL-POLICY template round-trips (§5).** `Body::MultiKeys{k,indices}` key-independent (`tree.rs:34-40`). `compute_wallet_descriptor_template_id` → `tree::write_node` whose `Body` match is exhaustive, NO `_ =>` arm, handles `Variable`(thresh `:90-114`), `Timelock`(`:160-162`), `Children`, `Hash256/160Body`, `KeyArg`, `Tr`. Encode/TLV-write (`tlv.rs:149-171`, `None` pubkeys omits entry)/decode/validate all round-trip a keyless thresh+timelock+hashlock policy. `to_miniscript_descriptor` errors `MissingPubkey{idx}` cleanly (`to_miniscript.rs:122`). `tr(sortedmulti_a)` refused (`:584-589`). **D1 broad scope well-founded.**
- **Emit guard-lift + origin-elide feasible (§5 Slice 1).** Guard A `n!=1` (`synthesize.rs:987-995`), Guard B `cli_template_from_tree().is_none()` (`:1005-1012`). Elide hardcodes single-slot `Shared(empty)` (`:1030`). NO structural blocker to N slots: `PathDeclPaths::Divergent(Vec<OriginPath>)` validates `len()==n` + round-trips (`origin_path.rs:90-146`); `synthesize_unified:899-908` ALREADY builds `Shared`-or-`Divergent` N-slot path_decls. *(Third guard omitted — MINOR 1.)*
- **Flow-inversion (§4.2/§5 Slice 2).** Today keys BUILT from md1 (`expand_per_at_n` over `md1.tlv.pubkeys`, `restore.rs:1751-1784`); `--cosigner` only compares `supplied65 != c.key65` (`:1987`) → exit 4; keyless gate `:1655-1661` → `ModeViolation`; RED pin `cli_restore_md1_template.rs:173-222` `.code(2)`. Inversion sound as a mechanism.
- **verify-bundle gap + lockstep (§5 Slice 3).** `verify_singlesig_template` (`:478`); multisig path sources `desc.tlv.pubkeys` → template-only failure (`:2475-2479`); `VerifyBundleArgs` has NEITHER `--from` NOR `--cosigner`. GUI mirror `mnemonic-gui/src/schema/mnemonic.rs:662-882` + manual `41-mnemonic.md:581-602` → new flags trip both. Lockstep holds.
- **Disambiguation math (§3.1) correct.** log2(11!)≈25.25 bits; formula gives ≥5B (margin 8) to ≥8B (margin 32), correctly excluding 4B. Expected false prefix-collisions among N! candidates: 4B→0.0093, 5B→3.6e-5, 8B→2e-12.
- **Address-search cryptographically unique for non-sorted shapes.** `multi`/`multi_a`/`thresh`/combinators serialize children in STORED slot order (`astelem.rs:155-188`; only `Sorted*` reorders via BIP-67) → distinct perms → distinct script bytes → distinct 256-bit P2WSH program → distinct address. Full-scriptPubKey predicate = no prefix-collision. Asymmetric `or_d(pk(@0),and_v(v:pk(@1),older(144)))`: wrong `@0↔@1` → different script + semantics → different id AND address → caught. `compute_wallet_policy_id` hashes per-`@N` in slot order (`identity.rs:189-228`), never BIP-67-sorts → order-sensitive id for all shapes.
- **N!-count / search-order / cap sound.** N = distinct `@N`; reused `@N` doesn't inflate; cap is a benign upper bound. Order optimality = perf heuristic, not correctness.
- **Benchmark §6 reproduces exactly.** 6.9µs→14s, 7.4µs→15s, 75µs→2.5min, 750µs→25min at 11!/20-thread; 1-hr ceiling ~13.5!. Faithfulness asserts present. Not fabricated.
- **Housekeeping refs exist.** v0.44.0 regression record real (`design/agent-reports/restore-multisig-cosigner-phase-2-r1-review.md`); umbrella `FOLLOWUPS.md:29`, `restore-multisig-cosigner-scope:321`; `degrade2.desc` present.

## CRITICAL
None.

## IMPORTANT

### I1 — Duplicate supplied cosigner keys collide on BOTH address AND id; unguarded.
If the operator supplies the SAME key for two distinct `@N` slots (paste one mk1 twice — a common error), swapping those slots is a literal no-op: byte-identical script → identical address AND identical per-slot id records (`identity.rs:189-228`) → identical `WalletPolicyId`. Result: a degenerate/insecure multisig (a "2-of-3" that is really 2-of-2 with a reused signer) built with NO warning. No dedup guard in md-codec (`canonicalize.rs:420-474`) or toolkit (`restore.rs` grep `dup`/`distinct`/`HashSet` → empty); `to_miniscript.rs:648-660` looks up `keys[index]` per slot, no uniqueness check. The "refuse on ambiguous" rule (§3.1) would even false-refuse the (degenerate) wallet or silently pick one. **Fix:** hard floor — reject duplicate supplied cosigner keys (pairwise 65-byte compare before the search). Add to §4 floors + §8 + a TDD case (`@0`==`@1` mk1 → refused).

### I2 — id-search acceptance rests entirely on the unspecified prefix margin; the single-spurious-match case has no runtime backstop.
Scenario the §3.1 ambiguity rule does NOT cover: operator supplies a WRONG key set → the true assembly is ABSENT from the N! space → a SINGLE wrong candidate spuriously matches the short id prefix → exactly one match → "unique" → silently accepted as a wrong wallet. Refuse-on-≥2 only fires if a second candidate also collides. P(≥1 spurious unique match | true absent), N=11: 4B→0.0093, 5B(margin 8)→3.6e-5, 8B(margin 32)→2e-12. id-search has NO secondary runtime verification (accepts on prefix alone); §4 floor 3 (address-equivalence differential) is TEST-side, not a runtime guard. The doc defers `safety_margin` to §8 — with margin 8 a wrong-key operator faces ~1-in-28,000 of a silent wrong wallet. **Fix:** (a) pin `safety_margin ≥ ~32 bits` (or require the full 16-byte id for searches) in the SPEC BODY, not §8; (b) document id-search is runtime-WEAKER than address-search (full-scriptPubKey is collision-free) → recommend address-search primary, id-search opt-in with full id; (c) optionally surface the completed full 16-byte id on a unique match for an out-of-band eyeball check.

### I3 — sortedmulti id-search over-constrains (id permutation-SENSITIVE, address permutation-INVARIANT).
§2 D2/§3.1 say for sortedmulti "a supplied id/address still verifies the key set, one evaluation." True for the ADDRESS (all perms → one address). FALSE for the id: `compute_wallet_policy_id` never BIP-67-sorts pubkeys (`identity.rs:189-228`), so the id is reproduced ONLY by the exact slot order used when recorded. A one-shot raw-id check on a sortedmulti template against a key set supplied in a different (but equally valid BIP-67) order would REFUSE a correct wallet. **Fix:** specify sortedmulti/sortedmulti_a verification is address-based (one eval), OR BIP-67-normalize the supplied key order before recomputing the id. Never raw-id-check a sortedmulti. Add to §8.

## MINOR
1. **Third emit guard omitted (§5):** Guard C `canonical_origin(&tree).is_none()` → `TemplateFormUnsupportedShape` (`synthesize.rs:1013-1021`) rejects non-canonical/custom-origin wrappers; the N-slot generalization must account for it + the single-slot card back-half (`:1047-1078`, loops `cosigners[0]`/`MkField::Single`).
2. **§4 floor 1 → make "every slot supplied" a HARD GATE:** today the cross-check is optional (an unsupplied slot degrades the LABEL but exits 0 — safe today only because the build source is the authoritative md1). In the inverted BUILD flow an unsupplied slot has NO source. Promote `all_verified` (`restore.rs:2032-2034`) to a hard refuse for the keyless-template path: union of `--from` own position + all `--cosigner @N` must == `0..n`, else refuse. (Recommended to fold now.)
3. **§4 floor 3 wording:** it is a test-side golden — clarify "a wrong assembly cannot pass" is a differential TEST, not a runtime guarantee; runtime guarantees = floors 1 + the disambiguation rigor (which I2 shows needs the margin pinned).
4. **§6 "7.4µs (1.1×)"** is the structure-once primitive (production floor); ensure the SPEC pins structure-once (rebuild-precomp is ~4-5× worse) so the cap estimate uses the right cost.

## To turn GREEN
1. I1 — duplicate-key hard floor + TDD.
2. I2 — pin `safety_margin ≥ ~32 bits` (or full 16-byte id for searches) in the SPEC body; document id-search runtime-weaker than address-search; optionally surface the full id on unique match.
3. I3 — sortedmulti id-check BIP-67-normalizes (or use address path).
4. MINOR 2 — unsupplied slot → refuse (hard gate).
5. MINOR 1/3/4 — fold for completeness.
Architecture sound; floor-tightening folds, not a redesign. Persist verbatim, re-dispatch, converge 0C/0I before SPEC.
