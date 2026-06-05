# R0 Architect Review (Round 2) — `SPEC_verify_bundle_entropy_slot.md` (v0.43.1)

**Reviewer:** opus `feature-dev:code-reviewer` (R0 mandatory pre-impl gate, re-dispatch after R0-r1 fold)
**Date:** 2026-06-04
**Source SHA:** `0f404ae`
**Verdict:** 0 Critical / 0 Important / 1 Minor — **GATE: GREEN**

> Persisted verbatim per CLAUDE.md. Fold note appended.

---

## Critical

None.

## Important

None.

The two round-1 findings are resolved cleanly, with no fold-introduced drift:

- **I1 (test #4 retarget) — RESOLVED.** §5 now declares `ANDOR3_DESC` matching `crates/mnemonic-toolkit/tests/cli_non_canonical_descriptor.rs:22` character-for-character, and that fixture is genuinely proven `.success()` with three secret slots (`cli_non_canonical_descriptor.rs:21-43`). Retargeted test #4 (`@0.phrase` / `@1.entropy=<hex>` / `@2.phrase`) is sound: the bundle descriptor loop (`bundle.rs:1438`) binds each `@N` independently via per-slot `if/else if` dispatch with **no cross-slot type-uniformity guard**, so the phrase+entropy+phrase mix bundles successfully; the new Entropy arm then fires at the non-`@0` `@1` slot during verify; against the unpatched binary the `@1` slot hits the catch-all (`verify_bundle.rs:885-892`, exit 2 `DescriptorReparseFailed`) → RED-for-the-right-reason, GREEN after. No `CANONICAL_DESC` secret-cosigner dependency remains in §5 (§5 explicitly excludes it).
- **M1 (arm-order rationale) — RESOLVED.** §3's reworded claim is accurate against source: `is_legal_set` (`slot_input.rs:342-367`) permits `[Entropy]` only as a standalone set (line 348); no `[Entropy, *]` co-occurrence exists anywhere in the matrix (verified through the full `matches!` body, lines 344-366), so precedence relative to the `Xpub`/`Ms1` arms is genuinely moot.

No regression elsewhere: the §3 arm code is unchanged from round 1 and remains compile-correct — the helper call matches `derive_slot.rs:65-71` `derive_bip32_from_entropy_at_path(entropy: &[u8], passphrase: &str, language: Bip39Language, network: CliNetwork, path: &DerivationPath)`, and the 5-tuple shape matches the destructuring target at `verify_bundle.rs:782-787`. §6 lockstep claims still hold (no clap-surface change; `entropy` pre-existing secret-bearing subkey at `slot_input.rs:85`; no `schema_mirror`/manual trigger).

## Minor

- **M2 — §3 citation range is short of the matrix it quantifies over** — `design/SPEC_verify_bundle_entropy_slot.md:33` cites `slot_input.rs:343-359` for the "no `[Entropy, *]` co-occurrence" claim, but the `is_legal_set` `matches!` arms run through line 365 (lines 360-365: `[Phrase,Path]`…`[Ms1,Path]`). The cited negative is true (none of 360-365 involve `Entropy`), but a reader can't confirm the claim from `343-359` alone. Fix: widen to `342-367` (or `344-366`). Non-blocking; worth the one-character fix given this project's citation-drift convention.

---

VERDICT: 0 Critical / 0 Important
GATE: GREEN

---

## Fold note (applied after persisting)

- **M2 — FOLDED.** Widened the §3 `is_legal_set` citation from `slot_input.rs:343-359` to `slot_input.rs:343-367` to cover the full `matches!` body. No other change. R0 gate is GREEN at 0C/0I — implementation may begin.
