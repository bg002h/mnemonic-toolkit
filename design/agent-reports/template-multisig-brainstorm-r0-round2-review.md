# Brainstorm R0 round 2 — bundle-md1-template-multisig (#28 phase 2) (opus architect, verbatim)

> Reviewer: opus architect (read+bash; toolkit working tree `99052af6`, md-codec 0.37.0 registry copy, mk-cli v0.10.0). **Verdict: GREEN — 0 Critical, 0 Important.** All 3 round-1 Importants CLOSED + 4 Minors folded + no new C/I. 3 presentational Minors (non-blocking). Brainstorm R0 converged → SPEC-ready.

---

**Verdict: GREEN — 0 Critical, 0 Important.**

## I1 — duplicate supplied cosigner keys collide on both address AND id — CLOSED
§4 floor 2 adds the HARD floor (pairwise-compare the N supplied 65-byte keys before search; TDD `@0`==`@1`→refused). Collision mechanism verified real: `compute_wallet_policy_id` builds each per-`@N` record `presence ‖ paths ‖ fp ‖ xpub` iterating `expanded` in slot order (`identity.rs:189-228`) → two slots with the same key emit byte-identical records → identical id; address BIP-67-collapses too (`to_miniscript.rs:402-409`). Swapping the equal slots is a no-op on BOTH axes. Pairwise 65-byte compare BEFORE the search (load-bearing: poisoned space never enumerated; match-uniqueness invariant preserved) deterministically refuses it. No dedup guard today (`canonicalize.rs:420-474`; restore grep empty). Residual (two DIFFERENT xpubs colliding at one index) is a ~2^-256 accident, id still differs, correctly out-of-scope. Does NOT over-reject legitimate same-`@N` multi-leaf reuse (that's ONE distinct slot, supplied once; N = distinct `@N`).

## I2 — id-search lone-spurious-match silently accepts a wrong wallet — CLOSED
In the SPEC-bindable body (§3.1 + §4 floor 5), NOT §8: search REQUIRES `≥ ceil((log2(N!)+32)/8)` bytes or the full 16-byte id; REFUSE on ambiguous/no-match; SURFACE the completed full 16-byte id on a unique match; id-search documented runtime-WEAKER than address-search (address-search recommended primary). Math re-checked: N=11 → 8 bytes → P(lone spurious | true absent) ≈ 11!/2^64 ≈ 2.2e-12; N=13 → 9 bytes → ≈ 1.3e-12; the `+32` margin is preserved across N (prefix bytes step with N!). The rejected 4-byte ≈ 1-in-108, correctly excluded. Surfaced full id = a genuine 128-bit out-of-band backstop. Address-search uniqueness basis verified (non-sorted serialize in stored slot order; only `Sorted*` reorders).

## I3 — sortedmulti id order-sensitive while address order-invariant — CLOSED
§2 D2 + §3.1 require a sorted-shape recorded-id verify to BIP-67-normalize the supplied order before recomputing (else false-refuse), address path preferred (sort-invariant, one eval). Confirmed: `compute_wallet_policy_id` NEVER BIP-67-sorts at hash time (`identity.rs:172-229`; canonicalizes only placeholder indices `:176`; hashes per-slot bytes positionally `:223-228`; `grep -i 'sort|bip67|lexico'` empty); the address routes through `new_*_sortedmulti` (`to_miniscript.rs:366/392/409`) which DOES BIP-67-sort. Asymmetry exactly as stated; normalize-before-recompute (or address path) resolves it.

## Folded Minors
- **MINOR 2 (every-slot-supplied HARD gate) — CLOSED structurally.** §4 floor 1(ii) promotes the today-advisory `all_verified` (`restore.rs:2030-2042` — only selects a label string, never errors) to a hard refuse (union of `--from` own position + all `--cosigner @N` == `0..n`). Closes the v0.44.0-class "unsupplied slot marked verified" hole at the structural level.
- **MINOR 1 (Guard C + N-slot back-half) — CLOSED; both sites exist.** Guard C `canonical_origin(&tree).is_none()` `synthesize.rs:1013-1021`; single-slot back-half `cosigners[0]`/`MkField::Single` `:1047-1078`. §5 names both.
- **MINOR 4 (structure-once primitive) — CLOSED.** §6 pins structure-once (7.4µs floor; naive 108× / rebuild-precomp ~4-5× worse) for the cap estimate.

## Drift check
- **Full/long-id-for-search vs #28 flexible-length `--expect-wallet-id` — NO collision, correctly scoped.** The single-sig path (`restore.rs:867-911` + `decode_wallet_id_prefix:978-997`) is a direct recompute-and-match against ONE completed wallet (one candidate, <4 bytes advisory not refusal). The strong-prefix rule is explicitly scoped to the id-SEARCH over the N! multisig space — different threat model (1 vs ≤11! candidates). No breaking change to single-sig.
- **Distinct-keys floor does NOT over-reject legitimate key reuse.** Multi-leaf reuse = same `@N` = one slot, one key; the floor compares the N keys for N distinct slots, fires only on the same 65-byte key in TWO different slots (the degenerate case). No over-rejection.
- No internal contradiction (id-weaker/address-primary consistent; sorted carve-outs consistent across §2/§3.1/§8; floors 1/2/5 orthogonal).

## Minors (non-blocking, SPEC-time)
1. §3.1's `ceil((log2(N!)+32)/8)` "bytes" — a reader could mis-skim "margin ≥32 bits" as "+4 bytes"; a one-line "= 8 bytes at N=11, 9 at N=13" in the SPEC forestalls an off-by-a-byte. (Numeric examples already correct.)
2. §6 "~13.5! / ~170M/min" are box-specific; SPEC re-derives from the §3.3 per-machine calibration (presentational).
3. Doc cites master `cbdadbb7`, tree at `99052af6`; §8 already mandates SPEC-time re-grep. No reviewed citation drifted; re-anchor at the chosen base SHA.

## Verdict
**GREEN — 0 Critical, 0 Important.** All 3 round-1 Importants CLOSED with source-verified funds-safety reasoning; 4 Minors folded with extant citations; no new C/I or contradiction; strong-prefix correctly scoped (no #28 single-sig break). Clear to proceed to SPEC.
