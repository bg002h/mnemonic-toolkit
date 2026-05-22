# v0.34.1 import-wallet hygiene — plan-doc opus R0 review (verbatim)

**Date:** 2026-05-22
**Reviewer:** opus `feature-dev:code-reviewer` (agent `ae37aa92b090f25d6`)
**Target:** `design/IMPLEMENTATION_PLAN_v0_34_1_import_wallet_secret_hygiene.md` (commit `e188c5e`) vs recon + FOLLOWUPs + source (working tree, Cargo.toml 0.34.0).
**Verdict:** **YELLOW** — 1 Critical, 1 Important, 3 Minor.

## Citation verification — all accurate
`decrypt_bsms_record` def `:2161` returns `String`, built `:2186`; EXACTLY two consumers (Round-2 `:1033`/`:1043`; Round-1 `:2299` into the `if/else` at `:2289`-`:2313`; the `:2440` hit is a comment). blob binding `:390`; BIE1 reassign `:434` + re-pin `:435` (`_pin_pt`); password pin `:418`; `read_blob -> Result<Zeroizing<Vec<u8>>>` `:2082`; `pin_pages_for(&[u8]) -> PinnedPageRange` `mlock.rs:90`; `parse_round1(&str)` `bsms_round1.rs:84`; `Zeroizing` import `:88`. All confirmed.

## Task 2 (zeroize) — SOUND
`Zeroizing<String>` blocks `.into_bytes()` (Drop move-out); `Zeroizing::new(plaintext.as_bytes().to_vec())` leaves no un-scrubbed copy (the `to_vec()` transient becomes the `blob` buffer); Round-1 `else` → `Zeroizing::new(raw_text)` unifies both arms; `parse_round1(&text)` compiles via deref. No missed consumers. No finding.

## Critical

### C1 — run-scoped `_pin_blob` guard issues a stale `munlock` on freed/reallocated pages after the `:434`/`:1043` reassign (cross-buffer un-pin hazard; a hardening regression). Confidence 80.
`_pin_blob` at `:391` lives to end of `run()`. On BIE1/Round-2 paths, `blob = …` (`:434`/`:1043`) drops + frees the original buffer; the allocator may re-hand those pages to a live secret buffer (the replacement `blob`, pinned by `_pin_pt`/`_pin_round2`). `_pin_blob`'s end-of-`run()` Drop then `sys_munlock`s that range — munlock is by page address with a per-page lock count, so it silently decrements a live secret's pin (including the BIE1 seed-bearing recovered JSON) → worse than today's `:435` pin. The plan resolves only the "re-pin the new buffer" direction, not the stale-guard teardown. (Not memory-unsafe — munlock never dereferences.)
**Fix (preferred):** single `let mut _pin_blob = pin_pages_for(&blob);` at `:391`, then at each reassign site reassign it (`_pin_blob = pin_pages_for(&blob);`) so the stale guard drops + munlocks the freed original immediately (before realloc can alias under a live pin). DELETE the separate `_pin_pt` (`:435`) to avoid two live guards on the same buffer. Result: exactly one live blob guard at any time.

## Important

### I1 — "mlock pin cannot be asserted in a test" is contradicted by the codebase's own `attempts_for_test()` pattern. Confidence 80.
`mlock::attempts_for_test()` (`mlock.rs:235`, increments on every `pin_pages_for` call) + asserting tests at `slip39/mod.rs:613`, `bip85.rs:411`, `derive_child.rs:463`, `derive.rs:225`, `synthesize.rs:1527`. A test that drives a plaintext Electrum import and asserts the counter incremented WOULD lock the headline fix (the plaintext seed-bearing path currently has zero pin coverage; a future deletion would be silent). Either add that focused test or amend the justification to "we decline the attempts-counter assertion because X" (the "cannot be asserted" claim is factually wrong).

## Minor
- **M1** — Round-1 `else`-arm line label off by one: `raw_text` is `:2312` (not `:2313`). Cosmetic. Confidence 95.
- **M2** — Round-2 "low sensitivity" framing accurate; zeroize still worthwhile. No action.
- **M3** — SemVer PATCH v0.34.1 + no GUI/manual/sibling lockstep + install.sh self-pin bump (`install.sh:32`, `Cargo.toml:3`) all CORRECT. No action.

## Verdict: YELLOW
Fold C1 (stale-munlock teardown — re-pin the single guard, drop `_pin_pt`) + I1 (add attempts-counter test or correct the justification) + M1, then re-dispatch R0. Citations accurate; Task 2 sound; SemVer/lockstep/install-pin correct.
