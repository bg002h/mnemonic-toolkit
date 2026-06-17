# cycle-prep recon — 2026-06-13 — sortedmulti_a-under-taptree + non-NUMS-internal-key

**Origin/master SHA at recon time:** `29613f3`
**Local branch:** `master`
**Sync state:** up-to-date (0 ahead / 0 behind)
**Untracked:** cycle-prep scratch + `.claude/` (none touch cited paths)

Slug(s) verified: (#2) `md-codec-sortedmulti-a-to-miniscript-rendering-gap` (sortedmulti_a as a leaf inside a multi-leaf taptree) and (#3) the non-NUMS / cosigner taproot internal-key (`is_nums:false`) case deferred by `restore-multisig-taproot-reconstruction` (v0.49.1). Expectation going in: both partly blocked beyond the toolkit. Finding: **#3 is toolkit-only/buildable now; #2 is cross-repo (md-codec) + (for depth≥2) upstream-miniscript blocked.**

---

## Per-slug verification

### #2 — `sortedmulti_a` under a tap-script tree (cross-repo + partial upstream)

- **WHAT:** A wallet-policy md1 whose tap tree contains `sortedmulti_a` as a non-root leaf is refused; only a **single-leaf** `tr(NUMS, sortedmulti_a(...))` works (the byte-identical Template path, routing around md-codec).
- **Citations (toolkit, current `29613f3`):**
  - `cmd/restore.rs:720` `Tag::SortedMultiA => Ok(TaprootRestore::Template(TrSortedMultiA))` — **ACCURATE** (single-leaf sortedmulti_a → works).
  - `cmd/restore.rs:722-727` `subtree_contains_sortedmulti_a(inner)` → `ModeViolation`, message cites `md-codec-sortedmulti-a-to-miniscript-rendering-gap` — **ACCURATE** (the refusal of sortedmulti_a under a taptree).
  - `cmd/restore.rs:689-691` doc: "sortedmulti_a anywhere under a TapTree refuses — md-codec's [gap]" — **ACCURATE.**
- **Root blocker (cross-repo, descriptor-mnemonic):** md-codec's `to_miniscript` cannot build `SortedMultiA` as a non-root tap leaf. Slug `md-codec-sortedmulti-a-to-miniscript-rendering-gap` / obs `sortedmulti-a-derive-gap-fenced` (descriptor-mnemonic FOLLOWUPS). The original `to_miniscript.rs:406-411` line citation is **DRIFTED** in md-codec 0.35.3 (that range is now the `Multi`/`MultiA` build arms) — re-locate the SortedMultiA arm before citing.
- **Upstream interaction:** sortedmulti_a-under-a-**multi-leaf** taptree is depth ≥1; a depth-≥2 placement ALSO trips `upstream-miniscript-taptree-depth2-display-asymmetry` (the same Display bug demonstrated this session: pinned `95fdd1c` Displays a depth-2 taptree as a string its own parser rejects — `taptree.rs:87-114 fmt_helper` comma-before-close-brace defect, per the sibling entry at `descriptor-mnemonic/design/FOLLOWUPS.md:1928-1939`).
- **Action for brainstorm spec:** NOT toolkit-only. The clean dissolution is upstream: a miniscript bump **> 13.1.0** (PR #953) dissolves the depth-2 bug AND enables md-codec option (a) AND drops the toolkit `[patch.crates-io]`. Sequence: (1) miniscript release → (2) md-codec renders SortedMultiA (sibling cycle) → (3) toolkit lifts the `subtree_contains_sortedmulti_a` + depth gates. Do NOT build #2 toolkit-side first. Cite SHA `29613f3`.

### #3 — non-NUMS (cosigner) taproot internal key (`is_nums:false`) (toolkit-only, buildable)

- **WHAT:** A `tr(<real cosigner key>, …)` policy (key-path spend present) is refused on restore; only `is_nums:true` (NUMS, script-path-only) is reconstructed.
- **Citations (toolkit, current `29613f3`):**
  - `template.rs:213,450,579,628` — bundle/`wrapper_node` emits `is_nums: true` (NUMS) since v0.48.0; the placeholder `is_nums:false` was RESOLVED — **ACCURATE** (so a non-NUMS card only ever arises from user `--descriptor` intake via `substitute_nums_sentinel`, not from `bundle`).
  - `cmd/restore.rs:700-703` `Body::Tr { is_nums: false, .. } => ModeViolation` — **ACCURATE** (the refusal).
  - `cmd/restore.rs:676-678` doc: "Supports only is_nums:true … is_nums:false (genuine cosigner-internal — **reconstructable but deferred**)" — **ACCURATE.** Confirms it is NOT upstream/cross-repo blocked — purely a deferred toolkit scope decision.
- **Upstream interaction:** independent of the SortedMultiA gap; a non-NUMS internal key with a **single leaf or depth-1** tap tree is buildable now. Depth ≥2 still hits the upstream Display bug (same ceiling as NUMS).
- **Action for brainstorm spec:** Toolkit-only, **buildable now.** Reuse the existing general-faithful arm; add an `is_nums:false` reconstruction branch that carries the cosigner internal key (`Body::Tr.key_index` → `TaprootInternalKey::Cosigner(i)`) into `build_descriptor_string(..., Some(Cosigner(i)))` and derives the key-path-inclusive descriptor. Watch-only, PATCH. Cite SHA `29613f3`.

- **DEEPER VERIFICATION (2026-06-13, the "do we need more cycle-prep" pass) — reconstruction mechanism is SHAPE-DEPENDENT (this is the load-bearing brainstorm decision, NOT more archaeology):**
  - **Encode-reachability CONFIRMED:** `bundle --descriptor "tr(<real cosigner xpub>, multi_a(2,B,C))"` → exit 0, emits a real `is_nums:false` md1. So it's a genuine reachable round-trip gap (engrave-yes / restore-no), not hypothetical.
  - **The gate is ONE common-tr check** (`restore.rs:693-716`, the `Body::Tr` extractor): `is_nums:false` is refused there for EVERY tr — so lifting #3 = lift this single gate AND supply faithful reconstruction for BOTH the Template (multi_a/sortedmulti_a) and General (route-around) arms.
  - **`build_tr_multi_a_descriptor`'s `Cosigner(idx)` mode (`pipeline.rs:134-156`) hard-codes `leaf = key_segs \ {idx}`** (internal key designated from the cosigner table; ALL OTHER cosigners go in the leaf). FAITHFUL for `tr(cosigner_i, multi_a(k, {all others}))` (the internal-distinct shape bundle produces). **BREAKS for:** (1) the legacy `@-in-both` shape `tr(@0, multi_a(k, @0,…))` — drops @0 from the leaf (the same defect as the v0.49.1 R0-r1 C2); (2) any leaf that is a strict subset of `table \ {internal}`.
  - **General (non-multi_a) non-NUMS tr** routes to the general-faithful arm, which builds the descriptor STRING directly from the tree — so it can write the real internal key naturally; needs a one-spot verify that the route-around reads `Body::Tr{is_nums:false,key_index}` and emits `tr(<internal>,…)` rather than assuming NUMS.
  - **VERDICT:** no further recon round needed. The remaining open item is a **SCOPE DECISION for the brainstorm**, not unknown code: which non-NUMS shapes to faithfully reconstruct — recommended (a) general single-leaf/depth-1 non-NUMS (route-around, cleanest), (b) multisig `tr(cosigner_i, multi_a/sortedmulti_a(k, {the others}))` via the existing Cosigner mode; **refuse/defer** the legacy `@-in-both` shape (Cosigner mode can't represent it; bundle never emits it post-v0.48.0, only external/legacy cards could) and depth-≥2 (upstream-blocked, unchanged). Sizing unchanged (~80-150 LOC + goldens). This is the R0-question set the brainstorm must close.

---

## Cross-cutting observations

1. **Neither item has a dedicated toolkit FOLLOWUP slug.** #2 lives in the sibling (descriptor-mnemonic) registry; #3 is the deferred half of the RESOLVED `restore-multisig-taproot-reconstruction` (a code-comment note at `restore.rs:676-678`, not a `###` entry). If #3 is greenlit, file a proper toolkit slug first.
2. **Cross-repo line drift:** the md-codec `to_miniscript.rs:406-411` SortedMultiA-refusal citation is stale in 0.35.3 — re-grep before any spec.
3. **One upstream release dissolves three things at once:** miniscript > 13.1.0 (#953) clears the depth-2 Display bug, the SortedMultiA rendering path, and the toolkit `[patch.crates-io]`. That release is the natural trigger for a combined taproot-coverage cycle.
4. Sync clean; no DRIFTED-by-N in toolkit-side citations (only the cross-repo md-codec line).

---

## Recommended brainstorm-session scope

- **#3 (non-NUMS internal key): a standalone, buildable PATCH cycle now.** Toolkit-only, ~80–150 LOC (a new `is_nums:false` reconstruction branch + golden tests; mirrors the v0.49.1 NUMS arm). Watch-only, no clap delta → no `schema_mirror`; manual prose under `docs/manual/src/40-cli-reference/41-mnemonic.md` (restore taproot section). Depth ceiling unchanged (single-leaf/depth-1 only). **R0-gateable today.**
- **#2 (sortedmulti_a under a taptree): NOT independently buildable — gate it behind the upstream miniscript release.** It is cross-repo (md-codec must render SortedMultiA) and, for depth≥2, upstream-blocked. Track it as the toolkit's slice of a future combined cycle triggered by miniscript > 13.1.0: (a) bump miniscript + drop `[patch]`; (b) md-codec sibling cycle renders SortedMultiA; (c) toolkit lifts the `subtree_contains_sortedmulti_a` + depth-≥2 gates together (they share the same upstream root). Building #2 toolkit-side now would only add dead refusal-removal that nothing downstream can satisfy.
- **Ordering/dependency:** #3 independent, ship anytime. #2 blocked on upstream→md-codec; do not start until the miniscript bump lands.
- **Mandatory R0 gate** before any implementation on #3 (0C/0I), per CLAUDE.md.
