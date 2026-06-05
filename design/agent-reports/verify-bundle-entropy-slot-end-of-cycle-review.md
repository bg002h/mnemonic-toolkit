# End-of-Cycle Architect Review — release-prep artifacts (v0.43.1)

**Reviewer:** opus `feature-dev:code-reviewer` (final gate before tag; scoped to Phase 3 release-prep prose + cycle coherence)
**Date:** 2026-06-04
**Branch:** `verify-bundle-entropy-slot` (Phase 3 commit `764d3e2`)
**Verdict:** 0 Critical / 0 Important / 1 Minor — **GATE: GREEN**

> Persisted verbatim per CLAUDE.md. The single Minor (CHANGELOG/FOLLOWUP "via the shared helper" wording) was folded after this review (commit following `764d3e2`).

---

## Review Summary

Reviewing the **v0.43.1 PATCH release-prep artifacts** (commit `764d3e2`) plus cycle coherence — scoped to the Phase 3 artifacts created after the GREEN Phase 2 code review. Every prose claim was checked against the actual source/diff/behavior, not trusted.

### Verification results

**1. CHANGELOG.md (lines 9-15)** — All factual claims verified accurate:
- "exit 4" — confirmed: `error.rs:503` maps `DescriptorReparseFailed → 4` (and `DescriptorParse → 2`).
- Catch-all message quoted verbatim — matches `verify_bundle.rs:928` exactly: `--slot @{idx} subkey set {:?} not supported in descriptor verify-bundle path`.
- "byte-identical to the bundle path" — confirmed: the new arm's helper (`derive_slot.rs:65`) runs the identical derivation spine as the inline bundle arm (`bundle.rs:1438-1470`), same inputs/path/`emit_lang=None` → same `(xpub, master_fp, anno_path, Some(entropy), None)` tuple.
- "no GUI schema_mirror or manual-mirror change" — confirmed: the change is a pure internal `else if` match arm; no `#[arg]`/clap surface change (the 18 `#[arg]` in the file are all pre-existing).
- Test-file description — confirmed: exactly 5 tests (16/32-byte lengths, non-`@0` slot in 3-cosigner `wsh(andor(...))`, passphrase round-trip, passphrase-mismatch) matching `cli_verify_bundle_entropy_slot.rs`.

**2. FOLLOWUPS.md `verify-bundle-descriptor-entropy-slot-gap` (lines 59-66)** — Status correctly flipped `open` → `resolved` at v0.43.1; exit-4 correction recorded as a stale-citation fix (the entry's own "What" line at `:63` still says "exit 2", now correctly flagged in the resolution note). Audit-trail files all exist and are cited correctly; the "R0 GREEN 0C/0I after one fold; Phase 2 GREEN 0C/0I" characterization matches the actual verdicts (r0-r1: RED 0C/1I; r0-r2: GREEN 0C/0I; phase-2-r1: GREEN 0C/0I).

**3. Version-bump completeness** — All sites consistently `0.43.1`: `Cargo.toml:3`, `Cargo.lock:706`, both README markers (`README.md:13`, `crates/mnemonic-toolkit/README.md:9`), `scripts/install.sh:32`. The three lingering `0.43.0` hits (cli_gui_schema.rs, cli_restore.rs, lint_argv_secret_flags.rs) are all legitimate historical references to the v0.43.0 `restore` feature in test comments — correctly retained.

**4. Cycle coherence** — SPEC final state matches what shipped: 5-test matrix incl. the passphrase pair that replaced the original self-check cell (§5 #5 documents the replacement rationale); exit-4 corrected throughout. FOLLOWUP/CHANGELOG/SPEC trio mutually consistent.

**5. Clean-PATCH obligations** — No manual flag-coverage or GUI schema_mirror obligation wrongly dismissed (no clap surface change, confirmed). No sibling-codec FOLLOWUP companion needed (toolkit-only).

---

## Critical
None.

## Important
None.

## Minor
- **CHANGELOG.md:13 / FOLLOWUPS.md:65 — "mirrors the bundle Entropy arm … via the shared `derive_slot::derive_bip32_from_entropy_at_path`".** Slightly loose: the *bundle* Entropy arm derives inline (`bundle.rs:1456-1469`), not via that helper; the new *verify* arm uses the helper. The claim is substantively accurate (behavioral mirror + proven byte-identical output, and the helper is a step-for-step equivalent of the bundle inline code), so this is a wording nuance only — no fix required for the release. (`/scratch/code/shibboleth/mnemonic-toolkit/CHANGELOG.md:13`)

VERDICT: 0 Critical / 0 Important
GATE: GREEN

---

## Fold note (applied after persisting)

- **Minor — FOLDED.** Reworded CHANGELOG and the FOLLOWUP resolution note to "mirrors the `bundle` Entropy arm's **behavior** … routing through the shared helper, the step-for-step equivalent of the bundle arm's inline derivation." Prose now precise about inline-vs-helper. Cycle is GREEN at every gate (R0, Phase 2, end-of-cycle); cleared for tag.
