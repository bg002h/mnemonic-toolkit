# PLAN R0 review — bip388-double-star-shorthand-support — round 3

**Verdict: GREEN (0 Critical / 0 Important)**
**Reviewer:** opus architect, source basis `0964462d` + live-binary verification.
**Dispatched:** 2026-07-06 (Cycle C, IMPLEMENTATION_PLAN R0 loop round 3 — scoped convergence on the compare-cost I3 decision). Persisted verbatim per CLAUDE.md.

The round-2 I3 blocker (compare-cost false-acceptance) is correctly resolved via option (B) — equivalence, not acceptance. Re-verified against documents + live binary. No residual findings. Ready for implementer dispatch.

## I3 → RESOLVED (coherent, satisfiable, non-tautological)
Empirical confirmation of the two premises:
- Pre-fix `compare-cost --descriptor "…xpub/**"` → `"at derivation index '**': invalid child number format"` (from_str@strip.rs:21).
- Explicit `"…xpub/<0;1>/*"` → `"multipath key cannot be a DerivedDescriptorKey"` (derive_at_index@strip.rs:27).

Distinct error strings, so §7.11 is:
- **Satisfiable:** after expansion `/**`→`/<0;1>/*` produces the multipath error == the explicit form. ✓
- **Non-tautological:** the `!= "invalid child number format"` clause proves the expander fired (a no-op would leave the pre-fix error, failing the cell). Asserts (i) `/**` ≡ `/<0;1>/*` AND (ii) `/**` differs from its own pre-fix behavior. ✓
- **Version-robust:** compares two live outputs through the same miniscript at test time; holds regardless of the exact error string, and stays valid if a future dep makes compare-cost accept multipath (both accept identically). ✓

## Mutual consistency → verified
- SPEC §0 item 5 (export-wallet genuinely ACCEPTS — verified live: `wpkh([…]xpub…/<0;1>/*)#zlgwg2zt`) ↔ §7.11 export cell ↔ plan `export_wallet.rs:517` row. ✓
- SPEC §0 item 6 (compare-cost EQUIVALENCE-only, FOLLOWUP `compare-cost-multipath-descriptor-unsupported`) ↔ §6 (invariant spans successful outputs AND error/exit) ↔ §7.11 ↔ plan Task-1 row + §7.11. Identical framing + error strings matching the live binary. ✓

## No other residual (Q3) → confirmed
The I3 fold reframed the semantics of an already-present IN site (`cost/strip.rs:21`) — NO new call site → no new surface to re-audit. Expander no-op on non-`/**` (borrowed Cow) → compare-cost's non-`/**` + `--miniscript` paths unaffected. FOLLOWUP referenced consistently across SPEC §0 item 6, §7.11, plan rows.

## Everything else (GREEN in round 2) still holds
Complete call-site set (parse_descriptor:875, bundle:1389, verify_bundle:1375, descriptor_intake:297, bsms:300, roundtrip:231, export_wallet:517, cost/strip:21; gui_schema:1319 chokepoint-covered); full user-`--descriptor` sweep found no further surface; roundtrip.rs:241 sole production from_str (sibling canonicalize_* audit empty); M1-M3 folded; TDD ordering, per-phase full-suite gate, whole-diff endpoint, v0.78.0 version-site ritual all present.

**Conclusion:** SPEC (rev-5) + IMPLEMENTATION_PLAN (rev-3) converged to 0C/0I across three plan-R0 rounds (following spec-R0 convergence). Both gates GREEN. Per CLAUDE.md phase-3, dispatch to a single implementer subagent in a worktree (TDD, per-phase R0, mandatory post-impl whole-diff review).
