# Plan-doc R0 round 1 — architect review (verbatim) — PLAN_taproot_depth_ge2_experimental.md

> Persisted verbatim per CLAUDE.md. Dispatched via Agent (feature-dev:code-architect, Opus 4.8).
> **Verdict: YELLOW (0 Critical, 1 Important, 4 Minor).** Source SHA `8da9008`. EXPERIMENTAL never-merge POC.
> I-1 (depth-3 cell was actually depth-2) + actionable minors folded before re-dispatch.

---

## Pre-verdict working notes (verbatim, abridged)

`cargo metadata` without `--locked` does refresh the Cargo.lock when the manifest changes; `cargo build` is more reliable. The plan's note is adequest. [m-1]

[taproot_is_deep traces] After the guard `if inner.tag != TapTree { return false }`, `matches!(&inner.body, Body::Children(c) if c.iter().any(|x| x.tag == TapTree))`. For a depth-2 TapTree, children include TapTree nodes → true. For a depth-1 TapTree (children are pk leaves) → false. For the Template path (`inner.tag == MultiA`) → early `return false`. All correct — no advisory on depth-1 or Template.

[is_taproot/d.tree scope] `is_taproot` set at `:1281`; the advisory block is inside `if is_taproot || template_opt.is_none()` (`:1354`); `d.tree` is the field of `d` (assembled at `:1228`). Both in scope at `:1373-1374`. Placement AFTER `emit_advisories` + AFTER the fidelity guard — correct.

[format-string] `format!("tr(NUMS,{{{{pk({K0}),pk({K1})}},pk({K2})}})")` → `tr(NUMS,{{pk(…),pk(…)},pk(…)})` — correct Rust `{{`/`}}` escaping for the left-heavy depth-2 tree.

---

## CRITICAL — None.

## IMPORTANT

**I-1. The "depth-3 cell" is actually depth-2 (balanced 4-leaf tree).** (Task 2 Step 1)
The plan specifies `tr(NUMS,{{pk(K0),pk(K1)},{pk(K2),pk(K0)}})` labeled "depth-3 4-leaf." A balanced 4-leaf binary TapTree `{{A,B},{C,D}}` has TapTree nesting depth **2**, not 3. Spec §7 explicitly asks for "a **depth-3** cell (4+ leaves nested deeper)" — needed to exercise the RECURSION in `ensure_taptree_wellformed` past the level the flipped depth-2 cells already cover. **Fix:** use an asymmetric depth-3 shape, e.g. `tr(NUMS,{{{pk(K0),pk(K1)},pk(K2)},pk(K0)})` (innermost TapTree [K0,K1]; outer [inner,K2]; outermost [that,K0]). Key reuse (K0) yields only 3 distinct key-table slots and does not trigger the distinct-key gate (which checks cosigner xpubs, not leaf occurrences); the shape is miniscript-valid and round-trips through `bundle --descriptor`.

## MINOR

**m-1.** Lockfile refresh under-specified for the fuzz lock. `cargo metadata` may refuse to update with a "needs update" error in some cargo versions. The reliable mechanism is `cargo build` (Step 3 already runs it). Treat `cargo metadata` as a diagnostic shortcut; the Step 3 build is authoritative.

**m-2.** `ensure_taptree_wellformed` needs `use md_codec::tree::Body;` inside the fn — the snippet already shows it (matches the existing fn at `:820`). Non-issue.

**m-3.** Test Step 4 "capture goldens": `assert_cmd` captures subprocess stdout, so `--nocapture` alone won't print it. Capture by running the restore CLI directly (`mnemonic restore --network mainnet --md1 <chunks>`) and copy stdout, OR temporarily add a `println!` in the test. Specify this.

**m-4.** The depth-2 cell's `(depth-2)` labeling is correct once the depth-3 cell (I-1) is separate. Informational only.

## VERDICT: YELLOW
1 Important (I-1: the "depth-3 cell" is depth-2; needs a genuine depth-3 asymmetric shape to exercise full recursion). All code snippets compile; `taproot_is_deep` fires correctly (false on depth-1/Template); `is_taproot`/`d.tree`/`stderr` in scope at `:1373-1374`; TDD ordering sound; lockfile mechanism adequate. Fold the shape correction and re-dispatch.
