<!-- VERBATIM opus-architect Phase-2 per-phase review, round 2, descriptor-builder gate. Persisted per CLAUDE.md. gate.rs @ post-I1-fold; source b596d3f. Verdict: GREEN 0C/0I — Phase 2 cleared, Phase 3 may begin. -->

# Phase-2 Per-Phase Architect Review, Round 2 — `descriptor_builder/gate.rs` — **GREEN** (0 Critical / 0 Important)

I1 is closed; the M1/M3 minors are folded soundly; no regression or new C/I introduced by the fold. **Phase 2 is cleared. Phase 3 (emit + clap surface) may begin.**

## CRITICAL
None. No production path returns `Ok` for a sanity-unsafe tree. Step 3 (`sanity_check`) unchanged, remains the sole funds-footgun gate; the fold touched only the step-4 count. Fail-closed holds.

## IMPORTANT
None.

**I1 — CLOSED.** Confirmed on all four sub-points:
1. **Leaf-count, not distinct.** `hash_and_timelock_counts` increments `n_hash_leaves += 1` per `Terminal::Sha256/Hash256/Ripemd160/Hash160` over `ms.iter()` — the `BTreeSet` dedup is gone. Mirrors enumerate's `walk_segv0_for_hash_leaves` (`enumerate.rs:422-444`) + `n_hashes = assets.hashes.len()` (`:90`). Same recursive `ms.iter()`; identical leaf counts for the same descriptor string.
2. **All three count axes agree.** Keys `BTreeSet<DescriptorPublicKey>` both sides; hashes leaf-count both sides (the fold); timelocks `n_abs × n_rel` same classification. No remaining divergence.
3. **Regression test genuine + non-vacuous.** `repeated_digest_cap_agrees_with_enumerate`: `and_v(v:sha256(H),and_v(v:sha256(H),pk(A)))` (1 key, same H twice) → raw=8. The discriminating assertion is the **cap-4 refusal** — under the old dedup n_hashes=1 → raw=4, `4>4` false → Ok → the `.err().expect()` would panic → test fails under old code. Also pins gate-raw == enumerate-raw == 8 across the `run_compare_cost` boundary (ok at 8, ConditionsTooMany at 7).
4. **No-regression clincher.** The change is **monotonically more conservative**: `n_hashes_new ≥ n_hashes_old` always → `check_cap` can only refuse more, never newly pass — cannot introduce a funds-unsafe Ok.

## MINOR
None blocking.
- **M1 (folded) — sound, not vacuous.** `sanity_dispatch_maps_each_rule_to_its_kind` iterates all 5 `AnalysisError` variants, genuinely executes the previously-untested Malleable + ResourceLimit arms, asserts each → its kind + the clean-tree root fallback. Dropping the end-to-end malleable RED cell is acceptable: the acceptance argument is fail-closed (even a malleable-but-parseable tree would be rejected by `sanity_check`; worst case is `node_path="root"`, a precision miss not a safety miss); the 3 cross-branch kinds prove `localize()` end-to-end.
- **M3 (folded) — accurate.** Comment correctly states the step-2 invariant + relaxation condition; property is fail-closed regardless.

## What passes
- I1 closed — leaf-count parity across all three axes; monotonically-conservative fold; genuine cap-4 discriminating regression cell.
- M1/M3 folded soundly.
- No regression / no new C/I — fold swapped only the hash counter; steps 1-3, localization, timelock axis untouched; fail-closed intact.
- State: descriptor_builder = 33 green; clippy = 0 warnings. (Verdict rests on source-level analysis; the discrimination proof is the cap-4 arithmetic, not the pass count.)

## Carry-forward for Phase 3
- **M2 (deferred, must not be dropped):** SPEC §9 assigns the one-xpub-two-origins key-dedup cap test to Phase 3. The gate's `distinct_keys` dedups by full `DescriptorPublicKey` (origin-bearing), so two origins on the same xpub count as 2 keys. Phase 3 must include the cap test pinning this — do not drop it.

Phase 2 is cleared at 0 Critical / 0 Important. Phase 3 may begin.
