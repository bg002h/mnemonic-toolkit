# cycle-prep recon — 2026-06-14 — taproot depth>=2 (EXPERIMENTAL, never-merge)

**Origin/master SHA at recon time:** `8da9008`
**Local branch:** `experimental/taproot-depth-ge2` (created off `d0201d5`)
**Sync state:** `0 ahead / 1 behind` origin/master — the 1 behind is the `docs/Examples.pdf` commit (`8da9008`), irrelevant to this work.
**Untracked:** cycle-prep scratch + `.examples-build/` + `.claude/` (none touch cited paths).

Slug(s) verified: `upstream-miniscript-taptree-depth2-display-asymmetry`, `restore-general-and-multi-leaf-taproot-roundtrip`, `taproot-coverage-cycle-on-miniscript-gt-13-1-0`. **Verdict: clean — citations ACCURATE; toolkit lift surface is TINY and the Actions-on-close match exactly.** One line-drift in an adjacent file (the `[patch]` rev moved :17→:29 after this session's comment refresh).

---

## Per-slug verification

### `upstream-miniscript-taptree-depth2-display-asymmetry` (FOLLOWUPS.md:4080)
- **WHAT:** the pinned miniscript rev `95fdd1c` mis-Displays a depth-≥2 (≥3-leaf) taptree as malformed `{{a,b,c}}` that its own parser rejects; fixed upstream by PR #953 (merged to master 2026-05-25, **unreleased**).
- **Citations:**
  - **Actions-on-close** (`FOLLOWUPS.md:4087`): "LIFT `src/cmd/restore.rs::ensure_taptree_depth_le_one` + the `classify_taproot_restore` call; flip the two depth-2 refusal cells (left-heavy + right-spine) in `tests/cli_restore_taproot.rs` to reconstruction." — **ACCURATE.** Matches the live source exactly (gate fn `restore.rs:819`, call `:748`; cells `:263` `left_heavy_3leaf_tr_refuses_depth2`, `:284` `right_spine_3leaf_tr_also_refuses_depth2`).
  - #953 merged + unreleased; latest crates.io = 13.1.0 (lacks it). **VERIFIED this session** against github.com/rust-bitcoin/rust-miniscript + crates.io. The #953 merge commit = `ff4732e5f75aa555682343cb180fa72ee3e8e9d5` (also carries #910).
  - **SPIKE (this session, reverted):** bumping the `[patch]` rev `95fdd1c → ff4732e` is **build-clean + full-suite-green (zero regression)**, and the depth-2 Display round-trip is **RED@95fdd1c → GREEN@ff4732e** (a throwaway test confirmed it).
- **Action for spike/build:** bump the `[patch.crates-io]` rev to `ff4732e` (the enabler), then lift the structural depth gate. Cite SHA `8da9008`.

### `restore-general-and-multi-leaf-taproot-roundtrip` (FOLLOWUPS.md:4092)
- **WHAT:** bundle engraves general/multi-leaf tr md1 cards; restore reconstructs only the Display-safe subset (single-leaf + depth-1). Remainder item **(i)**: depth-≥2 — blocked upstream (this work).
- **Citations:**
  - depth gate `restore.rs::ensure_taptree_depth_le_one` — **ACCURATE**, fn `:819`, called from `classify_taproot_restore` `:748`. The fn refuses any `TapTree`-child-of-`TapTree` (binary md-codec trees ⟹ depth ≤1 ⟺ ≤2 leaves). Keep its malformed-tree defensive branch; remove only the deep-refusal.
  - **Display-fidelity guard** `restore.rs:1365` (`parsed.to_string() != descriptor`) — **ACCURATE, PRESENT, RETAINED.** This is the real net: with the structural depth gate gone, a genuinely mis-Displaying shape still refuses here. So lifting the structural gate is funds-safe once #953 fixes Display.
  - `subtree_contains_sortedmulti_a` (restore.rs, `:730`-ish) — **KEEP.** `sortedmulti_a`-under-a-taptree stays blocked (md-codec `to_miniscript` gap, separate slug `md-codec-sortedmulti-a-to-miniscript-rendering-gap`); NOT lifted here.
- **Action:** lift only `ensure_taptree_depth_le_one`; flip the 2 cells; keep both `subtree_contains_sortedmulti_a` and the fidelity guard.

### `taproot-coverage-cycle-on-miniscript-gt-13-1-0` (FOLLOWUPS.md:4126, umbrella)
- **WHAT:** the clean path waits for a crates.io release > 13.1.0 (with #953 + #910), then md-codec renders SortedMultiA, then toolkit drops `[patch]` + lifts the gates. **This experimental branch deliberately FRONT-RUNS step 3** using the unreleased master rev `ff4732e` — accepting the "pinning unreleased moving master" risk by being **never-merge**.
- **Action:** the experimental branch does NOT resolve this umbrella; it's a throwaway POC. When the release lands, the real cycle runs per the umbrella (and the POC is discarded/rebuilt).

---

## What does NOT need changing (verified)
- **export-wallet / `script_type_from_descriptor`:** NO explicit depth gate. The "taptree branch must have 2 children, found 1" error came from MINISCRIPT's parser re-reading the malformed Display — it disappears once the rev is bumped. Zero toolkit change. (Confirmed empirically this session: the depth-2 export-wallet failure was the miniscript parse error, not a toolkit refusal.)
- **`parse_descriptor::walk_tap_tree` (`:497`):** already folds multi-leaf taptrees via a depth-stack algorithm (1/2/3/4/5-leaf SPIKE-confirmed). bundle INTAKE of depth-≥2 already works. No change.
- **bundle:** already engraves depth-≥2 cards (intake side). No change.

## Out-of-scope (do NOT lift here)
- **compare-cost multi-leaf tr:** refused via `CompareCostError::MultiLeafTr` (`cost/strip.rs:116`); separate FOLLOWUP `compare-cost-single-leaf-tr-input`. Not needed for depth-≥2 restore/build/export. (Optional stretch only.)
- **`sortedmulti_a` under a taptree:** md-codec gap, stays refused.

---

## Cross-cutting observations
1. **`[patch]` rev line DRIFTED :17 → :29** (Cargo.toml) — this session's patch-comment refresh expanded the comment block. fuzz/Cargo.toml:28 unchanged. Both still rev `95fdd1c`.
2. The toolkit lift is **one gate fn** + a **rev bump** + **2 test flips** — astonishingly small, because the v0.55.1 design deliberately made the depth gate a conservative *superset* with the Display-fidelity guard as the real net.
3. **Experimental ⟹ NO locksteps apply.** Never-merge/never-tag means: no GUI `schema_mirror` (zero clap change anyway), no manual mirror, no sibling companions, no SemVer/version bump, no CHANGELOG release entry, no release ritual. The ONLY discipline that still applies: the mandatory R0 gate before code (CLAUDE.md) — though see scope note on rigor for a never-merge POC.

---

## Recommended scope
- **Tiny experimental POC, branch-only.** Surface: (a) bump `[patch.crates-io]` miniscript `95fdd1c → ff4732e` in `Cargo.toml:29` + `fuzz/Cargo.toml:28` + refresh `Cargo.lock` + `fuzz/Cargo.lock`; (b) lift `restore.rs::ensure_taptree_depth_le_one` (remove the deep-refusal; keep malformed-tree defensive check); (c) flip `cli_restore_taproot.rs` cells `:263`/`:284` to reconstruction goldens (+ optionally add a depth-3 cell); (d) mark EXPERIMENTAL (runtime stderr advisory on depth-≥2 reconstruct + a prominent banner/doc note); keep `subtree_contains_sortedmulti_a` + the Display-fidelity guard.
- **~30-80 LOC + goldens.** No clap delta. The spike already de-risked the rev bump (build-clean, suite-green).
- **Ordering:** brainstorm → **mandatory R0** → plan → subagent build, all on `experimental/taproot-depth-ge2`. **NEVER merge, NEVER tag.** When upstream releases > 13.1.0, discard and rebuild per the umbrella.
- **Rigor note for a never-merge POC:** R0's funds-safety purpose (don't ship unsafe) is moot here (nothing ships). Recommend a *streamlined* gate (one R0 pass on a short spec) rather than the full multi-round ceremony — at the user's discretion.
