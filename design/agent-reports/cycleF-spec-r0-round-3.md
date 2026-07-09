# SPEC R0 review — ms1-repair-demote-to-candidate — round 3

**Verdict: GREEN (0 Critical / 0 Important / 0 Minor)**
**Reviewer:** Fable architect (funds-weighted, read-only), per user directive. SPEC @ `da7be631`; source @ toolkit `4c554295`, ms `c2fd4eb`, mk `1c9fbf7`.
**Dispatched:** 2026-07-09 (Cycle F, SPEC R0 loop round 3 — convergence). Persisted verbatim per CLAUDE.md.

All round-2 findings correctly folded, no new drift. Design converged: C1's ground-truth compare closes the funds hole; I1's indel carve-out is funds-sound and correctly scoped; the 3 doc-accuracy defects fixed. Clear to implementation.

## Fold verification
- **I-R2-1 → §4 + §3 RESOLVED.** The 3 false indel cells now `n/a — no indel path` (§4 `ms repair`/`toolkit auto-repair`/`verify-bundle`); only `mnemonic repair --ms1` keeps `exit 5 (§3 indel; --max-indel only)`. §3 opens by scoping indel to `mnemonic repair --max-indel≥1` alone (ms-cli `RepairArgs={ms1,json}` no indel flag; inline sites `repair_card` substitution-only; length-corrupted→`Err→Ok(())` @:1684). No false exit-code statement for §6 to mirror.
- **M-R2-1 → §0.3 RESOLVED.** Advisory scoped to convert/inspect/xpub ONLY, explicitly NOT the 2 verify-bundle sites; corrected string via a direct `repair_card` call. Cross-check: §4 `toolkit auto-repair` cell = "no short-circuit + stderr advisory", `verify-bundle` cell = "checks evaluated vs expected" NO advisory → no §0.3↔§4 contradiction on the MATCH path.
- **M-R2-2 → §6 RESOLVED.** `RepairJson.verdict` = `{blessed(clean 0-corr), candidate(subst)}`; indel via `IndelJson` (`confident:bool`, no verdict). Refs check (`RepairJson`@:279, `IndelJson`@:350-365).

## Regression on prior folds — intact
C1 (ground-truth compare both sites, watch-only-skip → `expected.ms1[i]` present), I1 (full-checksum per-candidate + dedup→Unique/exit-5, multi-hit→Ambiguous/exit-4), I3 (mismatch=failed check row→exit 4, no typed error) all untouched. §4 internally coherent (verify-only columns `n/a` for non-verify surfaces). Funds-attack matrix stands; wrong-bundle attack pinned §5.5.

## SemVer / NO-BUMP — holds
toolkit MINOR v0.81.0 + ms-cli MINOR 0.14.0; ms-codec/mk-codec (not even read post-C1)/md* NO-BUMP; no clap surface → no GUI/schema_mirror; 4-site ms-cli pin advance complete.

## Two non-blocking notes for the plan/implementer (advisory — do NOT gate)
1. **§8.6 secret-hygiene at the C1 mismatch row:** the existing byte-mismatch path (`verify_bundle.rs:2063-2073`) populates `VerifyCheck.expected`/`.actual` with the ms1 seed STRINGS. If the C1 mismatch row mirrors it, both seed strings appear in the check output — pre-existing accepted verify-bundle behavior (not a new leak class), but the plan should CONSCIOUSLY decide echo-vs-redact per §8.6.
2. **verify-bundle `--json` check-row value shift:** post-C1, `ms1_decode` can be `pass` after auto-repair (previously always `fail` in that arm). Wire-value change (not schema_mirror-gated); consumers self-update per paired-PR rule; any corrupted-ms1-bundle verify-examples golden would churn (likely none exist).

**R0 gate satisfied — 0C/0I. Implementation may begin (single-subagent TDD in a worktree, then the mandatory post-impl whole-diff adversarial review).**
