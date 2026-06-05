# Phase 2 Architect Review (Round 2) — multisig restore (v0.44.0)

**Reviewer:** opus `feature-dev:code-reviewer` (per-phase re-dispatch after C1/M1 fold)
**Date:** 2026-06-04
**Verdict:** 0 Critical / 0 Important / 1 Minor — **GATE: GREEN**

> Persisted verbatim per CLAUDE.md. The sole Minor (no `--json` envelope assertion) was folded after this review.

---

### C1 fold verification (the security-critical one)

The `verified_positions: BTreeSet<u8>` discipline closes the round-1 exploit. Trace per the six required points:

1. **Exploit closed** (`--md1 <tampered @2> --from <your @0 seed>`): 6a matches @0 → `own_pos=Some(0)`, `verified_positions.insert(0)`, then `break` — @1/@2 never inserted. 6b skipped. Note loop: @0 → "← your seed (verified)"; @2 (tampered) → "from md1 (not independently verified)", NOT "cross-checked". `all_verified` false → status "partial". Closed.
2. **`--cosigner @1=` alone**: 6a skipped. 6b inserts `*n` only after the `find` real-idx guard AND the passing 65-byte compare — the mismatch arm `break`s before the insert. Only @1 enters the set; @0/@2 → "not independently verified"; status "partial".
3. **All-verified path** (own @0 + `--cosigner @1` + `@2`): `verified_positions={0,1,2}`, `all_verified` true, mismatch None → status "verified"; no PARTIAL/UNVERIFIED branch; no "not independently verified" label. Test #3b asserts this.
4. **No new drift / off-by-one**: `all_verified` requires every `cosigners[i].idx ∈ verified_positions` — correct. No spurious idx can enter (6a inserts only a matched `c.idx`; 6b inserts only a `find`-validated real idx). The mismatch hard-gate (exit 4) runs BEFORE the note loop + status and returns early, so a mismatch never reaches "partial"/"verified"; under `--allow-mismatch` it yields "overridden".
5. **M1 — passphrase pinned** (`_pin_pp`): parity with single-sig `run`. Resolved.
6. **Tests catch the C1 regression**: pre-fold labeled all positions "cross-checked" with no PARTIAL banner. Tests #2/#3 assert stdout "not independently verified" AND stderr "PARTIAL" — both post-fold-only, so both would FAIL against pre-fold code. #3b guards against over-triggering PARTIAL.

**Fold-drift re-scan:** `Secp256k1` import still used by single-sig `run`; `PublicKey` used in `xpub_from_65_bytes`; the removed `run_multisig` `secp` was genuinely unused. The `--json` envelope reads the same corrected `c.note` + `verification_status` as the text path.

## Critical
None.

## Important
None.

## Minor
- **No test asserts the `--json` envelope's `verification.status` / per-position `note` fields.** Round-1 C1 named the JSON path as part of the attack surface. Fix: add a `--json` assertion that `verification.status == "partial"` and a `cosigners[i].note` contains "not independently verified". Non-blocking: the JSON and text paths read identical variables, so the text-mode tests transitively guard those values.

---

**VERDICT: 0 Critical / 0 Important**
**GATE: GREEN**

C1 and M1 both resolved with no fold-introduced drift; `status=="partial"` ⟺ PARTIAL banner ⟺ mismatch-None ∧ has-reference ∧ !all_verified — the exact property that closes C1.

---

## Fold note (applied after persisting)

- **Minor — FOLDED.** Added test #3c (`md1_json_partial_status_and_notes`): `--from @0 --json` asserts `verification.status == "partial"`, the @0 cosigner note contains "your seed", and an un-supplied position is flagged "not independently verified". 12/12 multisig GREEN. Phase 2 GATE GREEN — proceeding to Phase 3 lockstep.
