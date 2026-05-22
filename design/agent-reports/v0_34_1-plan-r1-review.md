# v0.34.1 import-wallet hygiene — plan-doc opus R1 verification review (verbatim)

**Date:** 2026-05-22
**Reviewer:** opus `feature-dev:code-reviewer` (agent `a5efbb2487cfcf90e`)
**Target:** revised plan (commit `554c11a`) vs R0 report + source `6576cbf`.
**Verdict:** **YELLOW** — 0 Critical, 1 Important (NEW, from the fold), 1 Minor.

## R0 fold verification — all confirmed resolved
- **C1 (stale-munlock) RESOLVED + correctness confirmed.** Single `let mut _pin_blob` reassigned at each site is sound: (a) Rust assignment evaluates RHS (pins new) before dropping the old place value (munlocks old); (b) the munlocked original is freed-but-not-realloc'd (the new buffer = pre-existing `plaintext`/`to_vec()` allocated before the `blob = …` free; `pin_pages_for` allocates nothing); (c) no path leaves two live guards on overlapping pages or a stale guard at end-of-`run()`; (d) end-of-`run()` drop order `_pin_blob` (`:391`) before `blob` (`:390`) is fine — munlock never dereferences and `blob` is still allocated. No residual hole.
- **I1 (testability) RESOLVED.** The `:391` pin is genuinely unconditional (executes right after `read_blob`, before the `:400` match / any branch), so plaintext coverage is by-construction; `run()` has no harness + `ImportWalletArgs` has no `Default` → counter-test cost disproportionate for a PATCH. Corrected justification is accurate.
- **M1 RESOLVED.** `raw_text` at `:2312`.

## Important

### I1-NEW — Task 1 Step 4's `#[allow(unused_assignments)]` placement won't suppress the lint it targets; the plan's own `-D warnings` gate fails as written. Confidence 88.
The reassignments at `:435` + `:1043`-after assign `_pin_blob` a value never read before the next drop/reassign; `unused_assignments` doesn't model `Drop` as a read (rust#126743) and points at the REASSIGNMENT statement. A `#[allow]` on the `:391` `let` does NOT cover later separate statements in nested scopes → clippy `-D warnings` still fails.
**Fix (preferred, no attribute):** at both reassign sites use
`drop(std::mem::replace(&mut _pin_blob, mnemonic_toolkit::mlock::pin_pages_for(&blob)));`
— reads the old guard (satisfies the lint), explicitly drops it (self-documenting munlock), same pin-new-before-munlock-old ordering. No direct reassignment statement → `unused_assignments` never fires; `_pin_blob` used via `&mut` → no `unused_variables`. Pin a concrete idiom; don't leave as "resolve per clippy output".

## Minor
- **M1-NEW** (Task 2 Step 4): Round-1 consumer cited `:2299`/`:2310`; the call spans `:2299`-`:2303` yielding at `:2310`. Unambiguous; compiler names any missed consumer. No action. Confidence 70.

## New-issue scan — clean
`_pin_pw` (`:418`) untouched; no use-after-move (reassign, not move); Task 1+2 compose at `:1043` (`Zeroizing::new(plaintext.as_bytes().to_vec())` then re-pin); `parse_round1(&Zeroizing<String>)` via deref; SemVer/lockstep/install.sh/CHANGELOG/FOLLOWUPS all correct.

## Verdict: YELLOW → GREEN after I1-NEW fold
Fix I1-NEW (pin the `mem::replace`-drop idiom at both reassign sites + correct Step 4's clippy note). Core pinning + zeroize design sound by construction. No other task affected.
