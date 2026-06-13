# R0 round 3 — architect review (verbatim) — SPEC_older_timelock_advisory.md — **GREEN**

> Persisted verbatim per CLAUDE.md. Dispatched via Agent tool (feature-dev:code-architect,
> inherited session default model Opus 4.8; the body's self-attribution line is the agent's own).
> **Verdict: GREEN (0 Critical, 0 Important).** The R0 gate PASSES — implementation may proceed
> (the implementation plan-doc gets its own R0). Two optional Minors (m1, m2) folded post-GREEN as
> pure clarifications (no logic change). Source SHA `3235431` / md-codec 0.35.3 / HEAD `53d9d79`.

---

# R0 REVIEW — SPEC_older_timelock_advisory.md — Round 3

**Reviewer:** Claude Sonnet 4.6 (Fable 5)
**Source SHA verified:** `3235431` (spec self-declares; branch `older-timelock-advisory`, HEAD `53d9d79`)
**Date:** 2026-06-13
**Scope:** Verify r2 folds landed; grep-verify citations; check generic core type-correctness and `translated.segv0` reachability; confirm internal consistency.

---

## Critical

None found.

---

## Important

None found.

---

## Minor

### m1 — §3.2 adapter description omits the `translated.tap` coverage question for single-leaf `tr` input to `compare-cost`

**Evidence:** `cost/strip.rs:100-145` shows that when `--descriptor` input is a single-leaf `tr(IK, M)`, `translate_descriptor_tr_single_leaf` performs a reverse projection: the `Tap`-context miniscript is re-projected to Segwitv0 (string round-trip, `from_str`), and both `translated.segv0` (reverse-projected) and `translated.tap` (original Tap) are populated. The `older()` operand values survive the string round-trip, so walking `translated.segv0` catches them. This is correct and the spec's single-hook claim is valid.

However, §3.2 says "a SINGLE hook on `translated.segv0` after the dispatch covers BOTH paths" without acknowledging the `tr` leaf case. The statement is true, but an implementer who notices that `translated.tap` holds the "original" Tap miniscript (not the projected one) might incorrectly conclude they need to walk `translated.tap` as well. A single sentence noting that the reverse projection preserves `older()` operands — or equivalently, that `segv0` and `tap` hold structurally equivalent miniscripts with identical `older()` values — would preempt that confusion.

**Fix (optional):** Add to the `compare-cost` row in §3.2 or §4: "Walking only `translated.segv0` is sufficient because both the `wsh`-path and the `tr`-leaf reverse-projection path populate `segv0` with the same `older()` operand values; `translated.tap` contains structurally identical `older()` nodes (just different key types), and the dedup-by-value means a second walk would produce no new advisories."

This is a Minor — the spec is not wrong, only incompletely reasoned for this sub-case. The implementation will work correctly as specified. Not a blocker.

### m2 — §3.2 adapter B description uses "Wsh / Sh(Wsh) / Tr leaf scripts" without noting that Sh(Wsh) does not appear in any of the seven surfaces

**Evidence:** None of the §4 surfaces produces a `Sh(Wsh)` descriptor. `export-wallet`, `restore --md1`, and `xpub-search` literal funnel all hold `wsh(...)` or `tr(...)` descriptors from `from_str`. The `Sh(Wsh)` case is a valid miniscript descriptor type (confirmed at `strip.rs:40-43`), but including it in the `older_advisories_descriptor` unwrap description without noting that no current surface produces it could lead an implementer to write dead code or an incomplete unwrap list if they omit it for that reason.

**Fix (optional):** The spec can leave it in as forward-compatible robustness (the adapter is correct to handle it). No change strictly required. Alternatively add a parenthetical: "(`Sh(Wsh)` included for completeness; not produced by any current surface)."

Not a blocker.

---

## Fold verification — r2 findings

### I1 (Adapter B bifurcation) — VERIFIED CORRECT AND TYPE-SOUND

**Fold present:** §3.2 now describes (a) a generic core `older_advisories_ms<Pk: MiniscriptKey, Ctx: ScriptContext>(&Miniscript<Pk, Ctx>)`, (b) a thin `older_advisories_descriptor` unwrap adapter for `export-wallet`/`restore --md1`/`xpub-search` literal, and (c) `compare-cost` calling the core directly on `translated.segv0`.

**Type-correctness confirmed:**

- `Terminal<Pk, Ctx>::Older(RelLockTime)` — `RelLockTime` is a concrete type (not parameterized by `Pk` or `Ctx`). `lt.to_consensus_u32()` is available on `RelLockTime` at `primitives/relative_locktime.rs:48`. A generic `match self.node { Terminal::Older(lt) => ... }` compiles for any `Pk: MiniscriptKey, Ctx: ScriptContext`. **Generic core is type-correct.**
- `DefiniteDescriptorKey` implements `MiniscriptKey` (confirmed at `descriptor/key.rs:1455`). `Segwitv0` implements `ScriptContext`. So `Miniscript<DefiniteDescriptorKey, Segwitv0>` satisfies the generic bounds. **`compare-cost`'s `translated.segv0` can be passed to the generic core.**
- `Wsh::as_inner()` at `descriptor/segwitv0.rs:38` returns `&Miniscript<Pk, Segwitv0>`. `TapTree::leaves()` at `descriptor/tr/taptree.rs:110` yields `Arc<Miniscript<Pk, Tap>>`. Both can be passed to the generic core. **`older_advisories_descriptor` unwrap is feasible.**
- `translated.segv0` is accessible after `cost/mod.rs:136`; both dispatch arms produce a `Translated` before `?` propagation. **Hook insertion point confirmed reachable.**
- For single-leaf `tr` input via `--descriptor`: `strip.rs:125` reverse-projects Tap→Segwitv0 via `Miniscript::from_str`, which validates `older()` via `TryFrom<Sequence>`. Bit-31/zero remain unreachable; `older()` operand values survive the round-trip. Walking only `translated.segv0` is complete. **Single-hook sufficiency confirmed.**

### I2 (compare-cost test coverage) — VERIFIED CORRECT

§6 now exercises BOTH `--descriptor wsh(andor(...))` AND `--miniscript andor(...)`. Both parse (`older(65536)` passes `from_str`: bit-31 clear, non-zero Sequence), `older_consensus_masked(65536)` → `Some(Masked{0, Blocks})`. Advisory fires on both. **Fold confirmed.**

### m1 (cost/mod.rs line range) — VERIFIED CORRECT

`cost/mod.rs:128` is the `let (translated, advisory) = match &args.input {`; line 136 is the `};`. Spec cites `:128-136`. **Correct.**

---

## Additional correctness checks

- Bit-math re-verified at `gate.rs:264`: `older(65536)`→`Masked{0,Blocks}`; `older(1)`→`None`; `older(0x80000001)`→`Bit31Disabled`; `older(0x400064)`→`None` (clean 512s, value 100). All CORRECT.
- SemVer PATCH (zero clap delta, no schema_mirror). CORRECT.
- Manual lockstep: `41-mnemonic.md`, one shared paragraph x-ref'd from 7 sections. CORRECT.
- FOLLOWUPS Where-list `:140` (4→7, mark RESOLVED). CORRECT.
- Gate comment reword `gate.rs:262`. CORRECT.
- §3.3 three-regime analysis unchanged & consistent (`relative_locktime.rs:72-77` filter; `tree.rs:293-295` no-validation read; restore fail-closed).
- `main.rs` placement: `secret_advisory` already exists at `main.rs:25` — `timelock_advisory` follows the same pattern; no collision; no `lib.rs` change.

---

## Verdict

**GREEN** — 0 Critical, 0 Important. The gate passes.

The r2 folds (I1 Adapter B bifurcation, I2 compare-cost dual-path test, m1 line range) landed correctly. The generic core architecture is type-sound. `translated.segv0` is reachable at the hook insertion point and sufficient for all compare-cost input paths. All bit-math, regime analysis, SemVer, lockstep, and citation claims are correct. Two Minors (m1: `tr`-path reasoning completeness; m2: `Sh(Wsh)` scope note) are optional quality improvements, neither blocking.

**Implementation may proceed.** The implementation plan gets its own R0.
