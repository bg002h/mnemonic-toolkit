# Combined SPEC+plan re-converge (post plan-R0 C1 errata) — Opus, empirical

**Persisted per CLAUDE.md** (+ opus fold at end). Verdict: **OPEN (0C / 1I / 1M) → BOTH FOLDED → GREEN.**

## The funds-load-bearing correction — EMPIRICALLY VERIFIED CORRECT
Ran the live `0.86.0` binary (md-codec 0.40.0), seed `abandon…about` (master fp `73c5da0a`):
| Case | `origin_path` | xpub |
|---|---|---|
| no-origin `wpkh(@0)` | `m/` | `xpub661MyMwAqRbcF…` (depth-0 MASTER) |
| explicit-master `wpkh([73c5da0a]@0)` | `m/` | **byte-identical to no-origin** |
| explicit `wpkh([73c5da0a/84'/0'/0']@0)` | `m/84'/0'/0'` | `xpub6CatWdiZiodm…` (depth-3, different) |
No-origin ≡ explicit-master (byte-identical, depth-0 prefix = MASTER), NO default-path notice. PRE-repin `sh(wpkh(@0))` (non-canonical today) = `m/48'/0'/0'/1'` + notice; POST-flip analog = empty/master, no notice. **Delta = `48'/0'/0'/1'`+notice → empty/master; 49' NEVER on the emit path.** Code trace confirmed: `bundle.rs:1418` probe → `bind_descriptor_mode_paths` early-returns `:2263-2265` (canonical: no inference) → empty path_decl → `master.derive_priv(empty)` `:1562` = master. sh(wpkh) post-flip takes the identical branch (only `.is_none()` consumed; `Some(49')` discarded). 49' materialized ONLY at `restore.rs:1645` (n≥2 canonical_fallback, unreachable for n==1 sh(wpkh)) + policy-id hashing. **The corrected "empty/master, NOT 49'" claim is RIGHT.**

## I-1 (folded) — the errata was incomplete
SPEC §Testing line 63 still said "bundle default-origin 49'" (contradicting the corrected S3.1/Acceptance-#2/migration-note/plan-B1.1). FOLDED → "empty/master (wpkh-parity, NOT 49')". Grep-confirmed: every other 49' in both docs is a correct use (table value / policy-id / template-completion / `--template bip49` / "do NOT say 49'" negation). No stale 49'-emit-default remains.

## M-a (folded) — notice-cite: `emit_default_path_notice` def ~:2434, called ~:1791 (the cited `:2410` was inside `derivation_path_to_origin`). Fixed in SPEC + plan.

## Confirmed accurate (no action)
I1 scope EMPIRICALLY verified: `[Xpub,Path]` on canonical `wpkh(@0)` SUCCEEDS (exit 0); `[Phrase,Path]` REFUSES ("both secret-bearing + watch-only input"); gate at `bundle.rs:1429-1453` = `(has_phrase||has_seedqr||has_ms1)&&has_path`. M3 (repair seam) all 3 cites accurate (`bch.rs:76`, `lib.rs:50`, `prop_repair_never_wrong.rs:150-162`). M1 install.sh self-pin `:32`/sibling `:35`. M4 fixture pre-bump-inherent. **8-consumer audit complete + accurate** (all sites + the `restore.rs:319`/`verify_bundle.rs:391` `cli_template_from_tree` neutralizing conjuncts confirmed; 2 extra hits are `#[cfg(test)]` non-sh(wpkh)).

## VERDICT: GREEN (0C/0I after the I-1/M-a fold — reviewer conditional pre-bless "fix I-1 → sound to begin B0-first"). Grep-confirmed no residual drift.

---
**STATUS (opus, 2026-07-11):** Track B SPEC+plan R0 CONVERGED GREEN; the funds-flip destination (empty/master, not 49') is empirically binary-verified. Begin B0 (pin+F-A8 arm) on Opus.
