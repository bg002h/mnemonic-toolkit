# cycle-prep recon — 2026-06-06 — api-harvest-drift-on-synthesize-descriptor-signature + self-check-ms1-iteration-audit

**Origin/master SHA at recon time:** `8b883dd`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** `.claude/`, `CONTINUITY.md`, `cycle-prep-recon-*.md`, `feature-coverage-survey-*.md` (working artifacts) — no tracked-tree modifications.

Slug(s) verified: 2 chosen cycles. **Slug A has a STRUCTURAL error (the FOLLOWUP's stated correct signature is itself stale; the drift is broader than one line). Slug B is CONFIRMED + citations DRIFTED.**

---

## Per-slug verification

### `api-harvest-drift-on-synthesize-descriptor-signature` — OPEN, **STRUCTURALLY-WRONG + broader scope**
- **WHAT:** the technical-manual API-harvest table documents a stale `synthesize_descriptor` signature.
- **Citations (verified against `8b883dd`):**
  - `docs/technical-manual/transcripts/api-harvest-mnemonic-toolkit.md` documents `synthesize_descriptor(descriptor, cosigners, entropy, privacy_preserving)` (4-arg, cited source `:196`) — **STALE**. The FOLLOWUP says the fix is to make it 3-arg `(descriptor, cosigners, privacy_preserving)`. **BUT the FOLLOWUP's target is ITSELF now stale:** the CURRENT signature (`synthesize.rs:229`) is **4-arg `(descriptor, cosigners, privacy_preserving, run_language: bip39::Language)`** — `entropy` was dropped (v0.21.0) AND `run_language` was added (v0.47.1 synthesize_unified→synthesize_descriptor delegation). So the doc is wrong, AND the FOLLOWUP's "should be 3-arg" is wrong.
  - **Whole-table line-number drift (the FOLLOWUP only flags synthesize_descriptor):** every `synthesize_*` citation in the doc is stale — `synthesize_full :113→:142`, `synthesize_watch_only :152→:181`, `synthesize_descriptor :196→:229`, `synthesize_multisig_full :288→:344`, `synthesize_multisig_watch_only :413→:489`, `synthesize_unified :593→:745` (+ `build_descriptor :80` to re-verify). The api-harvest snapshot is broadly drifted, not just one line.
- **Action for brainstorm spec:** Regenerate (or hand-correct) the api-harvest table against current `synthesize.rs` at `8b883dd`: fix `synthesize_descriptor` to the real **4-arg `(descriptor, cosigners, privacy_preserving, run_language)`** form + correct ALL `synthesize_*` line numbers + re-verify each other signature's arg list (don't assume only synthesize_descriptor drifted). Consider the FOLLOWUP's option (b): add a grep-assert lint that each documented signature's `fn name(` exists in live source (a leading drift gate). Cite SHA `8b883dd`. **technical-manual is a SEPARATE pinned cadence (like manual-gui)** — docs-only, NO toolkit version bump/tag.

### `self-check-ms1-iteration-audit` — OPEN, **CONFIRMED gap; citations DRIFTED**
- **WHAT:** `bundle --self-check` validates md1 + mk1 but does NOT iterate ms1, so the per-slot ms1 emission rule could silently regress.
- **Citations (verified against `8b883dd`):**
  - `bundle.rs::self_check_bundle::MkField::Multi at bundle.rs:1478-1504` — **DRIFTED-by-~550**: `self_check_bundle` is at **`bundle.rs:2027-2112`**; the `MkField::Multi` arm is within it (~`:2075`). Content ACCURATE.
  - "self-check does NOT iterate ms1" — **ACCURATE/CONFIRMED**: the full `self_check_bundle` body (`:2027-2112`) decodes `md1` (`reassemble`) + `mk1` (`MkField::Single`/`Multi` via `mk_codec::decode`) + checks policy-id-stub linkage + privacy-preserving fingerprint, but has **ZERO `bundle.ms1` reference** (grep over `:2027-2112` → no ms1). The gap is real.
- **Action for brainstorm spec:** Add an ms1 iteration to `self_check_bundle` that, for every phrase-/entropy-bearing slot (non-`""` `bundle.ms1[i]`), decodes the ms1 codex32 string and asserts it round-trips to the supplied entropy (mirroring verify-bundle's ms1 validation). Skip `""` watch-only sentinels. Phased TDD: a RED cell that constructs a bundle whose ms1 emission is wrong/regressed and asserts self-check CATCHES it (currently passes → RED). Cite SHA `8b883dd`. Toolkit-only; `--self-check` already exists → **no CLI surface change, no schema_mirror, no manual mirror** (internal validation strengthening). SemVer PATCH.

---

## Cross-cutting observations
1. **Slug A's FOLLOWUP is doubly stale** — both the documented signature AND the FOLLOWUP's stated "correct" target are out of date (the `run_language` arg from v0.47.1 post-dates the 2026-05-17 filing). A cycle that naively applied the FOLLOWUP's "make it 3-arg" instruction would re-introduce a falsehood. The recon's corrected target (4-arg with `run_language`) is load-bearing. This is the decayed-citation class cycle-prep exists to catch.
2. **Two SEPARATE cadences / repos-of-concern:** slug A lives in `docs/technical-manual/` (separate pinned cadence, like manual-gui — docs-only, no toolkit version bump); slug B is toolkit crate code (`bundle.rs`) → PATCH. They do NOT belong in one cycle.
3. **Both are toolkit-repo, in sync at `8b883dd`.** No GUI/sibling lockstep for either (slug A docs; slug B internal self-check, no surface change).
4. **Slug B is the higher-value one** (closes a real silent-regression guard gap in a correctness-checking feature); slug A is trivial-but-decayed docs.

---

## Recommended brainstorm-session scope
- **TWO independent cycles** (different cadence + SemVer; do NOT combine):
  - **Cycle 1 — `self-check-ms1-iteration-audit` (toolkit PATCH).** Add ms1 round-trip validation to `self_check_bundle` (`bundle.rs:2027`). **Size: ~15-30 LOC + 1-2 RED cells.** Phased TDD (RED: self-check must catch a regressed/empty-when-expected ms1 emission). No CLI surface change → no schema_mirror / manual mirror. SemVer **PATCH** (tag-only). Mandatory R0. **Recommend FIRST** (higher value, self-contained).
  - **Cycle 2 — `api-harvest-drift-on-synthesize-descriptor-signature` (docs-only, no bump).** Regenerate/correct the api-harvest table in `docs/technical-manual/` to the CURRENT signatures (synthesize_descriptor = **4-arg with `run_language`**) + line numbers; optionally add a grep-assert drift lint. **Size: ~7 table rows + optional lint.** technical-manual separate cadence → NO toolkit version bump/tag. Mandatory R0 (small). Independent of Cycle 1.
- No inter-cycle dependency; run sequentially (Cycle 1 then Cycle 2 recommended).
