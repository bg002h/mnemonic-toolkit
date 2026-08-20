# EXPERIMENTAL depth-≥2 taproot — Implementation Plan

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


> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]` checkboxes.
> **⚠ EXPERIMENTAL / NEVER-MERGE / NEVER-TAG.** Branch `experimental/taproot-depth-ge2` only. Pins an UNRELEASED miniscript commit. Do NOT merge to master, tag, or run the release ritual.

**Goal:** lift `restore --md1`'s depth-≤1 taproot ceiling so depth-≥2 tap trees reconstruct, as a proof-of-concept, with a loud EXPERIMENTAL advisory and the Display-fidelity guard retained.

**Architecture:** bump the `[patch.crates-io]` miniscript rev to the #953 merge (`ff4732e`, fixes nested-taptree Display); relax `restore.rs::ensure_taptree_depth_le_one` → `ensure_taptree_wellformed` (recursive, no depth cap); add a `taproot_is_deep` predicate + EXPERIMENTAL stderr advisory at the existing `older()`-advisory emit point.

**Spec:** `design/SPEC_taproot_depth_ge2_experimental.md` (R0 GREEN ×2). **Source SHA:** `8da9008`. **Branch:** `experimental/taproot-depth-ge2`.

**Key facts (grep-verified):** `Cargo.toml:29` + `fuzz/Cargo.toml:28` carry `rev = "95fdd1c…"`. `restore.rs`: gate fn `ensure_taptree_depth_le_one` (`:819`), call (`:748`), `subtree_contains_sortedmulti_a` (`:741`/`:801`, KEEP), `older()` advisory emit (`:1373-1374`, `stderr` + `d.tree` + `is_taproot` in scope, after the Display-fidelity guard at `:1365`). Tests: `left_heavy_3leaf_tr_refuses_depth2` (`cli_restore_taproot.rs:263`), `right_spine_3leaf_tr_also_refuses_depth2` (`:284`). md-codec `Body::{Children(Vec<Node>), Tr{tree:Option<Box<Node>>,..}}`.

---

## Task 1: Enabler — bump the miniscript patch rev + clean baseline

**Files:** `Cargo.toml`, `fuzz/Cargo.toml`, `Cargo.lock`, `fuzz/Cargo.lock`, `EXPERIMENTAL.md` (new).

- [ ] **Step 1: Bump the rev in both manifests + add the EXPERIMENTAL banner.**
  `Cargo.toml:29` and `fuzz/Cargo.toml:28`: change `rev = "95fdd1c5773bd918c574d2225787973f63e16a66"` → `rev = "ff4732e5f75aa555682343cb180fa72ee3e8e9d5"`. Prepend to the `Cargo.toml` `[patch.crates-io]` comment:
  ```
  # ⚠ EXPERIMENTAL — DO NOT MERGE. Branch experimental/taproot-depth-ge2 only.
  # This rev (ff4732e, the PR #953 merge) is UNRELEASED upstream master; it enables
  # depth-≥2 taproot Display. master MUST stay on 95fdd1c until a crates.io release
  # > 13.1.0 ships #953 (FOLLOWUP taproot-coverage-cycle-on-miniscript-gt-13-1-0).
  ```

- [ ] **Step 2: Refresh both lockfiles.**
  A git-rev change can't use `cargo update --precise`; the resolver re-writes the
  lock on the next build. **(R0-r1 m-1) `cargo build` is the authoritative
  re-resolver** (Step 3 runs it). Run `cargo build --manifest-path crates/mnemonic-toolkit/Cargo.toml`
  (refreshes root `Cargo.lock`) and the fuzz build in Step 3 (refreshes
  `fuzz/Cargo.lock`); `cargo metadata --format-version 1` is a quicker re-resolve
  but may error "needs update" in some cargo versions — if so, the build is the
  fallback. Verify both lockfiles reference `rev=ff4732e`: `grep -n 'ff4732e' Cargo.lock fuzz/Cargo.lock`.

- [ ] **Step 3: Confirm the clean baseline (the spike result, reproduced on-branch).**
  Run: `cargo build --manifest-path crates/mnemonic-toolkit/Cargo.toml` → clean. `cargo test --manifest-path crates/mnemonic-toolkit/Cargo.toml 2>&1 | grep -E 'test result:|FAILED' | grep -v '0 failed' || echo GREEN` → GREEN (zero regression; the existing depth-2 refusal cells STILL pass at this point because the gate is unchanged). `cargo +nightly-2026-04-27 build --manifest-path fuzz/Cargo.toml --target x86_64-unknown-linux-gnu` (or the fuzz-smoke command) → builds.

- [ ] **Step 4: Add `EXPERIMENTAL.md`** (repo root, branch-only):
  ```markdown
  # EXPERIMENTAL — depth-≥2 taproot (never-merge)
  This branch (`experimental/taproot-depth-ge2`) is a throwaway proof-of-concept.
  It pins an UNRELEASED rust-miniscript commit (ff4732e = PR #953 merge, in no
  crates.io release) to prove depth-≥2 taproot reconstruction end to end.
  DO NOT merge to master. DO NOT tag. DO NOT use for real funds.
  When a rust-miniscript release > 13.1.0 containing #953 ships, DISCARD this
  branch and rebuild for real per FOLLOWUP `taproot-coverage-cycle-on-miniscript-gt-13-1-0`.
  ```

- [ ] **Step 5: Commit** (stage `Cargo.toml fuzz/Cargo.toml Cargo.lock fuzz/Cargo.lock EXPERIMENTAL.md`):
  ```
  chore(experimental): bump miniscript patch rev 95fdd1c -> ff4732e (#953, UNRELEASED)

  EXPERIMENTAL/never-merge branch only. Enables depth-≥2 taptree Display. Baseline
  build + full suite + fuzz build green (spike-reproduced). EXPERIMENTAL.md + Cargo
  banner mark the unreleased pin. master stays on 95fdd1c.

  Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
  ```

---

## Task 2: Lift the gate + EXPERIMENTAL advisory (TDD)

**Files:** `crates/mnemonic-toolkit/src/cmd/restore.rs`, `crates/mnemonic-toolkit/tests/cli_restore_taproot.rs`.

- [ ] **Step 1: Flip the 2 refusal cells to reconstruction + add a depth-3 cell (RED).**
  In `cli_restore_taproot.rs`, rewrite `left_heavy_3leaf_tr_refuses_depth2` (`:263`) and `right_spine_3leaf_tr_also_refuses_depth2` (`:284`) to assert SUCCESS + golden + the advisory (placeholders for goldens, captured in Step 4):
  ```rust
  /// (5) Left-heavy 3-leaf (depth-2) taptree: EXPERIMENTAL — now reconstructs.
  #[test]
  fn left_heavy_3leaf_tr_reconstructs_depth2_experimental() {
      let desc = format!("tr(NUMS,{{{{pk({K0}),pk({K1})}},pk({K2})}})");
      let (md1, emitted) = bundle_md1(&desc);
      assert_eq!(emitted, desc, "depth-2 card round-trips on the wire");
      Command::cargo_bin("mnemonic").unwrap().args(restore_args(&md1)).assert()
          .success()
          .stdout(predicate::str::contains(GOLDEN_DESC_LEFT_HEAVY_3LEAF)
              .and(predicate::str::contains(GOLDEN_ADDR_LEFT_HEAVY_3LEAF)))
          .stderr(predicate::str::contains("EXPERIMENTAL"));
  }
  ```
  Mirror for the right-spine cell (`tr(NUMS,{pk(K0),{pk(K1),pk(K2)}})`, depth-2).
  **(R0-r1 I-1) Add a genuine DEPTH-3 cell** — the balanced 4-leaf `{{A,B},{C,D}}`
  is only depth-2; a true depth-3 tree needs asymmetric nesting. Use
  `tr(NUMS,{{{pk(K0),pk(K1)},pk(K2)},pk(K3)})` (4 leaves; deepest at depth 3),
  which exercises a third level of the `ensure_taptree_wellformed` recursion.
  **Use 4 DISTINCT keys** to sidestep any BIP-388 distinct-key-gate ambiguity on
  reuse: add a 4th cosigner const `K3` in the SAME `[fp/87'/0'/N']xpub…/<0;1>/*`
  format as K0/K1/K2 (which are 3 distinct seeds at `87'/0'/0'`). **(plan-R0-r2 m1)**
  K0/K1/K2 are BIP-87 multisig keys, so make K3 distinct via either a 4th test
  seed at `87'/0'/0'` OR an existing seed at a different account (e.g. `87'/0'/1'`).
  `mnemonic convert --to xpub --template bip87 --account N` emits the bare xpub;
  assemble the `[fp/87'/0'/N']…/<0;1>/*` wrapper from `--to fingerprint` + the
  path (as in the Examples.pdf key derivation). Add placeholder golden consts
  (desc + addr) for all three new cells near the others.

- [ ] **Step 2: Run → RED** (`cargo test … --test cli_restore_taproot left_heavy_3leaf right_spine_3leaf depth3 -- --nocapture`). Expected FAIL: restore still exits 2 (the gate still caps depth at this point), so `.success()` fails. Confirms the gate is what's blocking.

- [ ] **Step 3: Implement — relax the gate + add the predicate + advisory.**
  In `restore.rs`, replace `ensure_taptree_depth_le_one` with `ensure_taptree_wellformed`:
  ```rust
  /// EXPERIMENTAL (branch experimental/taproot-depth-ge2): the depth-≤1 cap is
  /// LIFTED. Recursively validate the md1 tap tree is well-formed (every `TapTree`
  /// node carries exactly 2 children, at any depth) — no depth limit. The
  /// Display-fidelity guard (parse→print, ~`:1366`) remains the funds-safety net
  /// for any shape the pinned miniscript still mis-prints.
  fn ensure_taptree_wellformed(inner: &md_codec::tree::Node) -> Result<(), ToolkitError> {
      use md_codec::tree::Body;
      if inner.tag != md_codec::Tag::TapTree {
          return Ok(()); // a leaf; miniscript leaf well-formedness checked downstream
      }
      let children = match &inner.body {
          Body::Children(c) if c.len() == 2 => c,
          _ => {
              return Err(ToolkitError::ModeViolation {
                  mode: "restore",
                  flag: "--md1",
                  message: "taproot md1 tap-script tree node is malformed (a TapTree must carry exactly 2 children); refusing to reconstruct",
              })
          }
      };
      ensure_taptree_wellformed(&children[0])?;
      ensure_taptree_wellformed(&children[1])?;
      Ok(())
  }

  /// EXPERIMENTAL: true iff `tree` is a `Tag::Tr` whose script tree is depth-≥2
  /// (a `TapTree` child that itself has a `TapTree` child). Drives the advisory.
  fn taproot_is_deep(tree: &md_codec::tree::Node) -> bool {
      use md_codec::tree::Body;
      let Body::Tr { tree: Some(inner), .. } = &tree.body else { return false };
      if inner.tag != md_codec::Tag::TapTree { return false; }
      matches!(&inner.body, Body::Children(c) if c.iter().any(|x| x.tag == md_codec::Tag::TapTree))
  }
  ```
  Update the call site (`:748`): `ensure_taptree_depth_le_one(inner)?;` → `ensure_taptree_wellformed(inner)?;`. At the `older()` advisory emit (`:1373-1374`), append:
  ```rust
  if is_taproot && taproot_is_deep(&d.tree) {
      writeln!(stderr, "EXPERIMENTAL: depth-≥2 taproot reconstruction relies on an UNRELEASED rust-miniscript commit (#953, in no crates.io release) — proof-of-concept only; do NOT use for real funds and do NOT merge. Rebuild when miniscript > 13.1.0 ships.").map_err(ToolkitError::Io)?;
  }
  ```
  Refresh stale comments: `TaprootRestore::GeneralFaithful` (`:668`), `classify_taproot_restore` doc (`:686-695` depth-≥2 bullet), test module doc (`cli_restore_taproot.rs:16-19`), **(plan-R0-r2 m2)** and the Display-fidelity-guard comment at `restore.rs:~1360` (the "the known depth-2 taptree bug is structurally pre-gated in §3" parenthetical is now FALSE — depth-≥2 is no longer pre-gated; the fidelity guard is now the primary net for Display asymmetry). KEEP `subtree_contains_sortedmulti_a` (`:741`) + the Display-fidelity guard itself (`:1365`).

- [ ] **Step 4: Capture goldens, run → GREEN.** **(R0-r1 m-3) Capture method:** `assert_cmd` captures the subprocess stdout, so `--nocapture` alone won't print it. Capture by running the restore CLI **directly** for each shape — `mnemonic restore --network mainnet --md1 <chunks…>` (get the chunks from `bundle --descriptor "<shape>" --json`) — and copy the `descriptor:` + `first recv:` lines into the consts. (Eyeball: NUMS trunk `50929b74…`, real `xpub661My…` leaf keys.) Then run `cargo test … --test cli_restore_taproot` → all pass (incl. the `.stderr(contains("EXPERIMENTAL"))` advisory assertions).

- [ ] **Step 5: Full suite + clippy** (`cargo test --manifest-path crates/mnemonic-toolkit/Cargo.toml`; `cargo clippy --manifest-path crates/mnemonic-toolkit/Cargo.toml --all-targets`). Expected: green; the NUMS/non-NUMS/@-in-both/sortedmulti_a-refusal cells unchanged.

- [ ] **Step 6: Commit** (stage `restore.rs` + `cli_restore_taproot.rs`):
  ```
  feat(experimental): reconstruct depth-≥2 taproot (lift the depth cap; EXPERIMENTAL advisory)

  Relax ensure_taptree_depth_le_one -> ensure_taptree_wellformed (recursive, no
  depth cap; Display-fidelity guard retained as the net). Add taproot_is_deep +
  a loud EXPERIMENTAL stderr advisory on depth-≥2 reconstruction. Flip the 2
  depth-2 refusal cells to reconstruction + a depth-3 cell. Never-merge branch.

  Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
  ```

---

## Task 3: Reviews + proof-of-concept demo (NO merge/tag)

- [ ] **Step 1: Per-phase reviews** — spec-compliance + code-quality on Task 1 and Task 2 (persist to `design/agent-reports/taproot-depth-ge2-phase-*.md`, fold to 0C/0I).
- [ ] **Step 2: Final whole-branch review** — architect over `git diff master...HEAD`, focused on the funds-safety net + the never-merge marking. Persist + fold.
- [ ] **Step 3: POC demo (optional, branch-only):** rebuild the PDF §6 one-tier-per-leaf 4-leaf degrading wallet (the depth-2 shape that failed earlier) and show it now builds → restores → exports, with the EXPERIMENTAL advisory. Capture for the record (do NOT add to the committed docs on master).
- [ ] **Step 4: STOP. Do NOT merge to master. Do NOT tag. Do NOT run the release ritual.** Leave the branch as the proof-of-concept; the FOLLOWUPs stay open.

---

## Self-review
**Spec coverage:** enabler rev bump → T1; gate lift + advisory + comment refresh → T2; tests (flip 2 + depth-3 + advisory assert) → T2; experimental marking (Cargo banner + EXPERIMENTAL.md + advisory) → T1/T2; retained nets (fidelity guard, sortedmulti_a, malformed-tree, @-in-both) → T2 (untouched/kept); no-merge/no-lockstep → T3. All covered.
**Placeholder scan:** golden consts are PLACEHOLDER-until-captured (T2 Step 4, the derive-once discipline) — legitimate. The `cargo update -p miniscript --precise` note flags that git-rev pins don't take `--precise`; the `cargo metadata` re-resolve is the actual mechanism.
**Type consistency:** `taproot_is_deep(&md_codec::tree::Node)`, `Body::Tr { tree: Some(inner), .. }`, `Body::Children(c)` — match the md-codec enum; `writeln!(stderr, …).map_err(ToolkitError::Io)?` matches the existing restore.rs stderr pattern.
