# SPEC — EXPERIMENTAL depth-≥2 taproot reconstruction (never-merge proof-of-concept)

> **STALE — RECORD ONLY, 2026-08-19.** The branch this governed,
> `experimental/taproot-depth-ge2` (11 commits, 2026-06-14), has been renamed
> `stale/experimental-taproot-depth-ge2` and its CODE was deliberately **not**
> merged. Only the design record was cherry-picked here, because the reasoning
> and the six review reports are worth keeping and the implementation is not.
>
> What was left behind, and why:
> - it moved the `miniscript` pin from rev `95fdd1c` to `ff4732e` — still an
>   **unreleased** git rev. Verified 2026-08-19: `ff4732e` (PR #953, taproot tree
>   descriptor formatting, merged 2026-05-24) is **not** an ancestor of
>   `miniscript-13.1.0` (2026-06-09), so the newest release was cut from a
>   maintenance line and still does not contain #953;
> - it changed `src/cmd/restore.rs` (101 lines) to lift the depth cap;
> - the later 2026-08-18 investigation ("the codec is not the gap") concluded
>   the depth gate **stays**, superseding this branch's direction.
>
> **Lifting the depth gate and taking #953 remain a live goal** — but as a fresh
> cycle against the current tree, not by merging a June proof-of-concept whose
> own commit subjects say "never-merge".


> **⚠ EXPERIMENTAL / NEVER-MERGE / NEVER-TAG.** This spec governs work on the
> throwaway branch `experimental/taproot-depth-ge2` ONLY. It deliberately pins an
> **UNRELEASED** rust-miniscript master commit (PR #953, not in any crates.io
> release). It MUST NOT be merged to master, tagged, or used for real funds. When
> a rust-miniscript release `> 13.1.0` containing #953 ships, this branch is
> **discarded and rebuilt for real** per FOLLOWUP `taproot-coverage-cycle-on-miniscript-gt-13-1-0`.

**Source SHA:** `8da9008` (origin/master at write time). All file:line citations grep-verified against this tree; re-grep on rebase.
**Branch:** `experimental/taproot-depth-ge2` (created off `d0201d5`).
**Recon:** `cycle-prep-recon-taproot-depth-ge2-experimental.md` (2026-06-14).
**Enabler (spike-verified this session):** bump the `[patch.crates-io]` miniscript rev `95fdd1c → ff4732e` (the #953 merge commit; also carries #910). The spike confirmed: build-clean, full-suite-green (zero regression), and the depth-2 Display round-trip flips RED@`95fdd1c` → GREEN@`ff4732e`.

---

## §1 Problem / goal

`restore --md1` reconstructs taproot wallet-policy cards only up to **depth-1** (≤2
leaves); a depth-≥2 tap tree (≥3 leaves, e.g. the PDF §6 "one-tier-per-leaf"
degrading wallet) is refused (`restore.rs::ensure_taptree_depth_le_one`,
exit 2, slug-citing). The refusal exists solely because the pinned miniscript rev
`95fdd1c` mis-Displays nested taptrees as malformed `{{a,b,c}}` (the PR-#953 bug).

**Goal (experimental):** prove depth-≥2 taproot reconstruction end-to-end NOW —
build / engrave / restore / watch-only-export a depth-≥2 `tr(...)` policy — by
adopting the unreleased #953 fix, as a proof-of-concept to validate the eventual
real cycle. Funds-safety nets stay intact; nothing ships.

## §2 Decision (scope)

**In scope (experimental):**
- (a) Bump the `[patch.crates-io]` miniscript rev to `ff4732e` (enabler).
- (b) Lift the depth-≤1 ceiling in restore: relax `ensure_taptree_depth_le_one`
  from "refuse depth-≥2" to a **recursive well-formedness check** (every `TapTree`
  node has exactly 2 children, at any depth) — keeping the malformed-tree refusal,
  dropping the depth cap.
- (c) **Experimental marking**: a loud runtime advisory on every depth-≥2
  reconstruction; an `EXPERIMENTAL` banner in the patch comment + a branch
  `EXPERIMENTAL.md`; the never-merge/never-tag discipline.
- (d) Flip the two depth-2 refusal tests to reconstruction goldens; add a deeper
  (depth-≥2 / ≥3-leaf, incl. a depth-3) reconstruction cell.

**Explicitly OUT of scope (stay refused):**
- `sortedmulti_a` under a tap tree (md-codec `to_miniscript` gap,
  `md-codec-sortedmulti-a-to-miniscript-rendering-gap`) — keep
  `subtree_contains_sortedmulti_a` (`restore.rs:741`).
- `compare-cost` multi-leaf tr (`CompareCostError::MultiLeafTr`,
  `cost/strip.rs:116`; FOLLOWUP `compare-cost-single-leaf-tr-input`).
- ALL release locksteps — see §8.

## §3 Architecture

**1. Enabler — rev bump (the ONLY reason depth-≥2 was blocked).**
`Cargo.toml:29` and `fuzz/Cargo.toml:28`: `rev = "95fdd1c…"` → `rev = "ff4732e5f75aa555682343cb180fa72ee3e8e9d5"`. Refresh `Cargo.lock` + `fuzz/Cargo.lock`. #953 fixes miniscript's `taptree.rs fmt_helper` so nested taptrees Display correctly → the toolkit's `faithful_multisig_descriptor` reconstruction path (which renders via miniscript `to_string()`) survives parse→print for depth-≥2.

**2. Lift the one gate.** `restore.rs::ensure_taptree_depth_le_one` (`:819`, called at `:748`). Current behavior (`:840-846`): refuses if any `TapTree` child is itself a `TapTree`. **Change:** drop that depth refusal; instead **recurse** into the children to validate the whole binary tree is well-formed (each `TapTree` carries exactly 2 children, at every depth). Rename to `ensure_taptree_wellformed`. No depth cap. The malformed-tree refusal (`:830-838`) stays.

**3. Why this is funds-safe even at depth-≥2.** The **Display-fidelity guard**
(`restore.rs:1365`, `parsed.to_string() != descriptor`) is UNCHANGED and remains
the real net: if any reconstructed descriptor — at any depth — fails to survive
its own parse→print round-trip, restore refuses there. The v0.55.1 design
deliberately made the structural depth gate a *conservative superset* with the
fidelity guard as the true safety check; with #953 fixing Display, the structural
cap is no longer needed and the fidelity guard suffices. (If `ff4732e` still
mis-Displays some exotic shape, the guard catches it — no silently-wrong address.)

## §4 Experimental marking (the non-negotiable part)

This build adopts UNRELEASED upstream code; the marking must make that impossible
to miss:

- **Runtime advisory (stderr, non-blocking):** whenever `restore` reconstructs a
  depth-≥2 taproot tree, emit a loud advisory, e.g.: *"EXPERIMENTAL: depth-≥2
  taproot reconstruction relies on an UNRELEASED rust-miniscript commit (#953, in
  no crates.io release). This build is a proof-of-concept — do NOT use for real
  funds and do NOT merge. Rebuild when miniscript > 13.1.0 ships."*
  - **Wiring (R0-r1 I2 — `classify_taproot_restore`/`ensure_taptree_wellformed`
    are pure, no `stderr`):** add a standalone predicate `fn taproot_is_deep(tree:
    &md_codec::tree::Node) -> bool` (true iff the `Tag::Tr` node's tree child is a
    `TapTree` that has a `TapTree` child — a one-level depth-≥2 probe, NOT a full
    re-walk). Call it in `run_multisig` **inside the existing taproot/general
    reconstruction block where `stderr` is in scope — right beside the current
    `older()` advisory emit at `restore.rs:1373-1374`**
    (`crate::timelock_advisory::emit_advisories(&adv, stderr)`), guarded by
    `if is_taproot && taproot_is_deep(&d.tree) { writeln!(stderr, "<EXPERIMENTAL …>")?; }`.
    **(R0-r2 m1)** the predicate's body matches `Body::Tr { tree: Some(inner), .. }`
    on its `&d.tree` argument first (mirroring `classify_taproot_restore`), then
    returns `inner.tag == Tag::TapTree && <inner has a TapTree child>` — i.e. it
    extracts the Tr's tree child before probing, exactly like classify does.
    This runs after the descriptor is parsed + the Display-fidelity guard passes,
    so the advisory only fires on a genuinely-reconstructed depth-≥2 wallet. Do NOT
    add a third `TaprootRestore` enum variant (keeps the classify match stable).
- **`Cargo.toml` patch comment:** prepend an `EXPERIMENTAL — DO NOT MERGE` banner
  explaining the `ff4732e` (unreleased) pin and that master must stay on `95fdd1c`
  until the real release.
- **`EXPERIMENTAL.md`** at the repo root (branch-only): states the branch is a
  never-merge POC, the upstream dependency, and the discard-and-rebuild plan.
- **Discipline:** NEVER `git merge` to master; NEVER tag; NEVER push to master.
  (Pushing the experimental branch itself to origin is fine if desired.)

## §5 Components / files
- `Cargo.toml` (`:29` rev + patch comment banner), `fuzz/Cargo.toml` (`:28` rev), `Cargo.lock`, `fuzz/Cargo.lock` — the enabler.
- `crates/mnemonic-toolkit/src/cmd/restore.rs`:
  - relax `ensure_taptree_depth_le_one` (`:819`) → `ensure_taptree_wellformed` (recursive: every `TapTree` has exactly 2 children at any depth; no depth cap; keep the malformed-tree refusal). Update the call site (`:748`) and the fn's own doc (`:814-818`) to the new name/behavior.
  - add `fn taproot_is_deep(&md_codec::tree::Node) -> bool` + the EXPERIMENTAL advisory at the `:1373-1374` emit point (§4 wiring).
  - **(R0-r1 m1) refresh stale doc comments:** `TaprootRestore::GeneralFaithful` (`:668`, "single-leaf or depth-1" → "single-leaf or any-depth"), the `classify_taproot_restore` doc (`:686-695`, the "depth ≥2 … refuses" bullet), and the file-level lift note. KEEP `subtree_contains_sortedmulti_a` (`:741`) and the Display-fidelity guard (`:1365`).
- `crates/mnemonic-toolkit/tests/cli_restore_taproot.rs` module doc (`:16-19`) — **(R0-r1 m1)** the "depth ≥2 … STRUCTURAL … lift on the #953 release" bullet becomes false; update it (EXPERIMENTAL: depth-≥2 reconstructs on this branch via the ff4732e pin).
- `crates/mnemonic-toolkit/tests/cli_restore_taproot.rs` — flip `left_heavy_3leaf_tr_refuses_depth2` (`:263`) + `right_spine_3leaf_tr_also_refuses_depth2` (`:284`) to reconstruction goldens; add a depth-≥2 (and a depth-3 / 4-leaf) reconstruction cell; assert the EXPERIMENTAL advisory fires.
- `EXPERIMENTAL.md` (new, branch-only).
- **No change:** export-wallet (no explicit depth gate — miniscript Display fix handles it), `parse_descriptor::walk_tap_tree` (already multi-leaf), bundle (already engraves depth-≥2).

## §6 Retained funds-safety nets (do NOT remove)
1. **Display-fidelity guard** (`restore.rs:1365`) — the real net; catches any parse→print mismatch at any depth.
2. **`subtree_contains_sortedmulti_a`** (`:741`) — sortedmulti_a-under-taptree stays refused (md-codec gap).
3. **Malformed-tree refusal** (the exactly-2-children check) — kept inside the relaxed fn.
4. **The @-in-both structural guard** (`refuse_at_in_both`, v0.55.3) — untouched; still applies to the Template multisig path. **(R0-r1 I1) Note it is structurally unreachable at depth-≥2 and that is correct:** a depth-≥2 tree always has `inner.tag == TapTree` (not `MultiA`/`SortedMultiA`), so `classify_taproot_restore` routes it through the `_` general arm (`restore.rs:740`) and never calls `refuse_at_in_both`. No blind spot results, because the general arm reconstructs via `faithful_multisig_descriptor` → `to_miniscript_descriptor`, which reads the ACTUAL tree (the existing code comment at `restore.rs:775-776` already states this). The @-in-both trap is specific to the Template path's Cosigner "leaf = all-others" shortcut, which depth-≥2 trees never take. No extension of the guard is needed.

## §7 Testing
- **(R0-r1 m3) Ordering — enabler FIRST:** bump the rev in `Cargo.toml:29` + `fuzz/Cargo.toml:28`, refresh `Cargo.lock` + `fuzz/Cargo.lock`, and run `cargo test --workspace` + `cargo build` in `fuzz/` to confirm the spike's clean baseline on the branch BEFORE touching `restore.rs`. Then the gate/advisory changes, then the test flips.
- **Flip the 2 refusal cells → reconstruction** (`:263`, `:284`): `tr(NUMS,{{pk(K0),pk(K1)},pk(K2)})` (left-heavy) and the right-spine 3-leaf — both now reconstruct; capture goldens (derive-once-from-binary, eyeball the depth-0-xpub reconstruction; the bc1p address must match the engraved card). **(R0-r1 m4)** assert the advisory with `.success().stderr(predicate::str::contains("EXPERIMENTAL"))` (non-blocking, exit 0 — NOT the exit-2 refusal pattern the old cells used).
- **Add a depth-2 GENERAL reconstruction cell** (e.g. the PDF §6 shape, abridged) + a **depth-3** cell (4+ leaves nested deeper) → reconstruct + advisory.
- **Regression:** the v0.49.1/v0.55.1/v0.55.3 NUMS + non-NUMS + @-in-both cells stay green (the rev bump is regression-free per the spike).
- **Still-refused contracts:** `sortedmulti_a`-under-taptree still exit-2 (slug-citing); compare-cost multi-leaf still `MultiLeafTr`.
- **Full suite green on the branch** (`cargo test` whole crate + clippy + fuzz build, per the v0.55.2 lesson — even though nothing ships, the branch should be self-consistent).

## §8 NO locksteps, NO release (never-merge)
Because this branch never merges or ships:
- **No GUI `schema_mirror`** (zero clap delta anyway).
- **No manual mirror** (`docs/manual/` untouched).
- **No sibling-codec companions.**
- **No SemVer bump, no CHANGELOG release entry, no version-marker/install.sh/tag** — the release ritual does NOT run.
- The FOLLOWUPS (`upstream-miniscript-taptree-depth2-display-asymmetry`, `restore-general-and-multi-leaf-taproot-roundtrip` (i), umbrella) stay **open** — this POC does not resolve them.

## §9 R0 status / non-goals
**R0 round 1 — YELLOW → folded** (0 Critical / 2 Important / 4 Minor; review `design/agent-reports/taproot-depth-ge2-r0-round1-review.md`). The architect **confirmed the funds-safety argument is sound** (the Display-fidelity guard at `:1365` is sufficient — no Critical; no silently-wrong-address path at depth-≥2), the recursive well-formedness check is correct, `subtree_contains_sortedmulti_a` recurses correctly for deep trees, and export-wallet/parse_descriptor/bundle need no change. Folded: **I1** (§6 item 4 — the @-in-both guard's depth-≥2 unreachability is correct, justified), **I2** (§4/§5 — the advisory wiring via a `taproot_is_deep` predicate at the `:1373-1374` emit point, not in the pure classify fn), **m1–m4** (§5/§7). **R0 round 2 — GREEN** (0 Critical / 0 Important / 1 Minor; review `design/agent-reports/taproot-depth-ge2-r0-round2-review.md`). The gate is MET. R0-r2 verified all r1 folds landed correctly against source, re-confirmed the funds-safety argument (the Display-fidelity guard fires strictly later than the removed structural cap, so no silently-wrong depth-≥2 path), and confirmed the advisory fires exactly on depth-≥2 GeneralFaithful arms and never on Template arms (their `inner.tag` is `MultiA`/`SortedMultiA`, so `taproot_is_deep` returns false). **Folded R0-r2 m1** (§4: `taproot_is_deep` extracts `Body::Tr { tree: Some(inner), .. }` before probing). No further R0 round required (single doc-clarity minor on an already-GREEN gate).

**Non-goals:** resolving the upstream/md-codec FOLLOWUPs; `sortedmulti_a`-under-taptree; compare-cost multi-leaf; any merge/tag/release; production use. The depth ceiling raised here is bounded only by miniscript's own limits (BIP-341 max taptree depth 128) + the Display-fidelity guard; no artificial new cap is added (the POC's job is to show the ceiling is gone).
