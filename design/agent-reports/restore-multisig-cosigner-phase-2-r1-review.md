# Phase 2 Architect Review (Round 1) — implemented multisig restore (v0.44.0)

**Reviewer:** opus `feature-dev:code-reviewer` (per-phase gate, post-implementation)
**Date:** 2026-06-04
**Branch:** `restore-multisig-cosigner` (impl commit `d7c52e9`)
**Verdict:** 1 Critical / 0 Important / 1 Minor — **GATE: RED**

> Persisted verbatim per CLAUDE.md (before the fold). Fold log appended.

---

## Review summary

Reviewed the implemented v0.44.0 multisig-cosigner restore against the approved SPEC — `restore.rs` (new `run_multisig` + helpers), `bundle.rs` (`extract_multisig_threshold` → `pub(crate)`), `tests/cli_restore_multisig.rs`. All cited md-codec 0.35 APIs verified (`expand_per_at_n -> Result<Vec<ExpandedKey>>`; `ExpandedKey.xpub: Option<[u8;65]>` = "32 chain-code ‖ 33 compressed pubkey"; `OriginPath.components`; `PathComponent { hardened, value }`; `derive_address(chain, index, network)`; `Tag::Tr`); exit-code mappings verified against `error.rs`.

## Critical

**C1 — Unchecked cosigner positions are labeled "cross-checked"/"verified": the verification document asserts trust it never established.** `restore.rs:982-989` (per-cosigner note loop) + `:991-997` (global `verification_status`) + `:999-1031` (JSON propagation).

The cross-check *engine* (6a own-seed inference, 6b explicit `--cosigner`) is sound and the mismatch hard-gate fires correctly in every ordering. But the *reporting* lies. The note loop is:

```rust
for c in cosigners.iter_mut() {
    if Some(c.idx) == own_pos { c.note = "← your seed (verified)"; }
    else if has_reference && mismatch.is_none() { c.note = "cross-checked"; }
}
```

Concrete exploit: `restore --md1 <tampered> --from phrase=<your seed>` where the user is cosigner @0 and an attacker swapped @2's xpub in the md1. Own-seed matches @0 → `own_pos=Some(0)`, `mismatch=None`, `has_reference=true`. The loop then labels @1 and @2 **"cross-checked"** — though neither was ever compared to anything. The swapped @2 key is presented to the user as cross-checked, and `verification_status` becomes the global `"verified"`. Identical defect with `--cosigner @1=` alone: 6b only compares position 1, yet @0 and @2 still receive "cross-checked". The `--json` path propagates the false labels to GUI/script consumers.

This is the security twin of a silently-accepted mismatch: unverified key material presented as verified. It also diverges from SPEC §4 step 5 / §6 step 6, which require *per-cosigner* "match/UNVERIFIED" status, not a blanket label. Root cause: 6b only `break`s on mismatch and never records which `@N`s passed, so the note loop cannot know what was actually validated.

Fix: track the set of positions actually validated — `own_pos` plus the `@N`s that passed their 65-byte compare in 6b. Label only those positions "verified"/"cross-checked"; leave every other position "unverified". `verification_status` should reflect "all-cosigners-verified" only when that validated set covers all `n` positions; otherwise a partial status. Add a positive test that a non-supplied position is NOT labeled cross-checked.

## Important

None.

## Minor

**M1 — Passphrase is neither pinned nor zeroized in the multisig path (parity regression).** `restore.rs:872-886`. The single-sig `run` pins the passphrase via `_pin_pp` (`:318-322`); the duplicated 6a block resolves `passphrase` into a plain `String` and pins only `entropy`. No leak to output, but a hardening regression versus the path it mirrors.

## Items checked and confirmed CLEAN

- **Xpub reconstruction (`xpub_from_65_bytes`):** byte decomposition matches `synthesize::xpub_to_65` exactly; `network.network_kind()` is the correct `Xpub.network` type; `try_from(&bytes[0..32]).unwrap()` cannot panic.
- **Cross-check soundness (the security-critical compare):** `derive_bip32_from_entropy_at_path` derives the account xpub at each cosigner's origin; the 65-byte compare against the md1's stored per-`@N` account xpub is correct. Foreign seed → `RestoreMismatch` (exit 4) — no false "verified" at the *gate*. mk1-vs-xpub discrimination robust. The hard-gate never silently accepts a mismatch.
- **Watch-only-out invariant:** no xpriv/WIF/seed/passphrase reaches stdout/stderr/json. `account_xpriv` is read only to produce `account_xpub` then dropped. Test #8 enforces this.
- **Taproot/template-only refusal:** `Tag::Tr` gate precedes `to_miniscript_descriptor`; `!is_wallet_policy()` refusal; both `ModeViolation` (exit 2). Test #7 confirms at runtime.
- **`--from` Option change:** single-sig `expect` reached only after the `!args.md1.is_empty()` dispatch; cannot fire in multisig mode. Both consumption sites correct.
- **`origin_path_to_derivation_path`:** correct hardened/normal reconstruction; no off-by-one.
- **Descriptor build / `account` inertness:** slots carry explicit origins so `args.account` is inert. Test #10 round-trips the descriptor through `bundle --descriptor`; test #9 confirms testnet → tpub.
- **`resolve_seed_entropy`** is a faithful mirror of the single-sig block.

**VERDICT: 1 Critical / 0 Important**
**GATE: RED** — C1 must be folded and the architect re-dispatched. The cross-check engine is sound; only the verification *reporting* over-claims — a focused one-issue fix.

---

## Fold log (applied after persisting)

- **C1 — FOLDED.** Added `verified_positions: BTreeSet<u8>` recording ONLY independently-validated positions (own-seed match in 6a + each passing `--cosigner @N` in 6b). The note loop now labels own_pos "← your seed (verified)", members of `verified_positions` "cross-checked", and every other position "from md1 (not independently verified)". `verification_status` is "verified" only when `verified_positions` covers ALL cosigners, else "partial" (else "unverified"/"overridden"). Added a `PARTIAL: cross-checked X/N … positions … were NOT independently verified` stderr banner. Strengthened tests #2/#3 to assert the un-supplied positions are NOT cross-checked + PARTIAL banner, and added #3b (all-positions-verified → no PARTIAL/UNVERIFIED, no "not independently verified").
- **M1 — FOLDED.** Added `_pin_pp` passphrase page-pin in the 6a block (parity with single-sig `run`).
- Removed the unused `let secp` / `let _ = &secp;` in `run_multisig` (the architect's smell note).
- 11/11 multisig tests GREEN after the fold. Re-dispatched Phase 2 round 2.
