# R0 REVIEW — cycle-14 brainstorm spec (close L22 — stdin-secret zeroize) — Round 1

Verified against `origin/master = 82c61e76` (v0.66.0).

## VERDICT: NOT GREEN — 0 Critical / 1 Important / 3 Minor

The core design (SecretString over raw Zeroizing; wrap at owned allocation) is correct and the load-bearing claims verified. One census gap (I-1) blocks: it omits secret-bearing residue sites.

## Verified-correct (the load-bearing claims)
- **Axis 1 — D2 Zeroizing-Debug-leak: CONFIRMED.** `Cargo.lock` resolves zeroize 1.8.2; `zeroize-1.8.2/src/lib.rs:622-623` = `#[derive(Debug, Default, Eq, PartialEq)] pub struct Zeroizing<Z>(Z);` with NO custom `Debug` impl → derived tuple-struct Debug prints `Zeroizing("secret")` → LEAKS into `{:?}`/`assert_eq!`. `Deref<Target=Z>` (=String, not str). The `SecretString` choice over raw `Zeroizing<String>` is fully justified (recon was wrong; spec is right).
- **Axis 2 — SecretString + plain equality: CONFIRMED SAFE.** `src/secret_string.rs` exists: newtype `SecretString(Zeroizing<String>)`, `Deref<Target=str>`, length-only redacting Debug, `#[derive(Clone)]` only (no PartialEq/Eq today → adding them is needed). Grep: no secret-vs-secret/attacker-observable/timing-sensitive compare — every non-test compare is against the public `"-"` literal or a test `assert_eq!`. Plain (non-constant-time) equality is SAFE.
- **Axis 3 — D1 fix-shape: direction CONFIRMED** (count is 14 `Zeroizing::new(read_stdin_*(…))` sites, not ~16 — m-2). Leaving readers as `String` and wrapping owners avoids the `Zeroizing<Zeroizing<String>>` foot-gun.
- **Axis 5 — Zeroize-lint gate: ACCURATE.** `SECRET_PATTERNS` (lint:426-431) includes `": SecretString"` → `slot_input.rs` newly matches → `every_secret_bearing_src_file_is_declared_or_allowlisted` (lint:482) requires a new row. `SECRET_FILE_FLOOR=35` (lint:452) → bump to 36 correct. Row-count bound `(18..=60)` (lint:375) stays in range. `secret_string.rs` allowlisted as PRIMITIVE (lint:442) → PartialEq/Eq there needs no row. `convert.rs` already has 2 rows → the proposed doc row is redundant (R0 may collapse).
- **Axis 6 — No behavior/wire change: CONFIRMED.** No `.value` is `{:?}`-logged in non-test source (all `slot @{idx}` prints the index); `SlotInput` has no serde → no `--json` surface. RED tests T1-T7 sound (T6 drop-scrub correctly deferred to the type-level guarantee).

## IMPORTANT (blocks GREEN)
**I-1 — §2.6 census omits 5 real `SlotInput.value` edit sites, 2 of which are un-wrapped secret residue.** The spec claims all non-test `.value` ops "compile unchanged (auto-deref)" — false for these `SlotInput.value` (the MIGRATING field) sites, not named anywhere:
1. `bundle.rs:2629` — `s.value = resolve_env_var_sentinel(&s.value, &flag)?;` (gated on `s.subkey.is_secret_bearing()`, resolves `@env:VAR` to the ACTUAL secret phrase and stores it back). Needs `s.value = SecretString::new(resolve_env_var_sentinel(...)?)`. **This is itself L22 residue** (secret materialized via the `@env:` channel).
2. `import_wallet.rs:1396` — same `@env:` write-back. **Residue.**
3. `verify_bundle.rs:1883` — same. **Residue.**
4. `import_wallet.rs:1233` — `phrase_overlays … .map(|s| (s.index, s.value.clone()))` filtered to `Phrase`; `.clone()` now returns `SecretString ≠ String` (won't compile) + clones the secret seed phrase. Needs `s.value.to_string()`.
5. `apply_slot_stdin`'s `slots[i].value = buf;` — spec DOES cover this (D2).
Root cause: the spec conflated `SlotInput.value` (migrating) with `FromInput.value` (separate, non-migrating String field — `from.value`/`primary.value`/`f.value`/`sh.value`/seed_xor `share` do NOT break).
**Required:** §2.6 + §2.3 + D1-scope must add the 4 omitted `SlotInput.value` sites (3× `@env:` write-backs via `SecretString::new(...)` + 1× `:1233` `.to_string()` clone), note the `@env:` write-back is a secret-residue path the wrap also closes, and disambiguate `SlotInput.value` (migrating) from `FromInput.value`. Plan-doc must re-grep these by live line number.

## MINOR (fold)
- **m-1 — SemVer rationale imprecise.** "`SlotInput` … in a `pub mod` (`lib.rs:178`) → reachable from the public API" is false for normal builds (`slot_input` is `pub mod` only under `#[cfg(fuzzing)]`; otherwise a private bin `mod` via `main.rs:30`; no `pub use SlotInput`). MINOR conclusion is CORRECT by the v0.10.1 precedent (structurally-identical `cfg(fuzzing)`-gated `derive`/`synthesize` field-type change → MINOR per FOLLOWUPS). Fix the rationale to cite precedent, not "public API reachability."
- **m-2 — census count:** "~16 already-wrapping" is actually 14; recompute "~28 edits". Direction unchanged.
- **m-3 (informational):** pre-existing stale doc comment `lint_zeroize_discipline.rs:370` says "(24..=35)" while the assert is `(18..=60)` — not this cycle's bug; don't let it confuse the floor edit.

## Path to GREEN
Fold I-1 (add the 4 omitted SlotInput.value sites + the SlotInput/FromInput disambiguation) + m-1/m-2/m-3; persist; re-dispatch R0.
