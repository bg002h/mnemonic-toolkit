# R0 review — `SPEC_test_hardening_T1_ms_funds_safety.md` (round 1) — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Reviewed vs `mnemonic-secret` @ `master 9a24999`. Method: source reads + **empirical mutation execution** in a scratch copy (repo untouched).

## Ground truth
**T1-a (#12) — VERIFIED, EMPIRICALLY RED-PROVEN.** `non_s_index_pool` (`shares.rs:28`); `CODEX32_ALPHABET` (:23) `'s'` = position 16; filter yields exactly 31 entries. `encode_shares` (:101), bound `k≤n≤31` (:122), pool consumed (:127,:152,:166). Under the filter-drop `pool[16]=='s'` is emitted iff n≥17. **Mutation survives today (empirical): full `cargo test -p ms-codec` = 163/163 GREEN under `.filter(|_|true)`.** Max n in any existing test = 11; n=32 only the reject test (:451-458). Proposed n∈{16,17,31} probe (wire-position parse at `sep+6`, no pool access): GREEN unmutated, **RED under mutation with first failure at n=17, `left: 's'`** (interpolate_at short-circuits → the `s` plate IS the secret-at-S; one plate = whole seed). Oracle independent (re-parses the emitted string via `share_header`/`extract_wire_fields`, `pub(crate)` in `envelope.rs:62`). No live bug.
**T1-b (#10) — SOUND.** `purpose()` (`derive.rs:79-87`) consumed at :327 (derivation) + :341 (path string) — one fn, both sites → `Bip44=>45` self-consistent. Only bip84 pinned e2e today (`cli_derive.rs:17`). BIP-86 spec vectors use `abandon×11 about` account-0 xpub (assertable); bip44/49 → independent hardcoded constants. RED-proof holds.
**T1-c (#11) — gap confirmed; oracle has a hole → I-1.** Existing name↔code tests + `maps_to_bip39_language` (:156-165, only English+ChineseSimplified) miss a Czech↔Portuguese swap. Spot-checks vs embedded official lists: English "abandon", Czech "abdikace", Portuguese "abacate", French "abaisser", Italian "abaco", Spanish "ábaco", Japanese "あいこくしん", Korean "가격" ✓. `Language::word_list()` public; ms-cli enables `all-languages`.

## Findings
**Critical — none.**
**IMPORTANT I-1 (T1-c):** the first-word oracle is DEGENERATE for the Chinese pair — ChineseSimplified and ChineseTraditional both officially begin with `的`; lists first diverge at **index 9 (simplified 这 / traditional 這; 773/2048 differ)**. A CN-Simplified↔CN-Traditional swap stays GREEN under a first-word-only test (same class as the named Czech↔Portuguese one); the spec's "pin the actual wordlist selection for ALL 10" is unmet for 2 of 10. **Fix:** additionally pin a differing-index word for both Chinese (word_list()[9] = 这 / 這) + extend the RED-proof to a CN-S↔CN-T swap.
**MINOR M-1 (T1-b):** `ms derive` emits no addresses (fingerprint + account xpub + path only, `derive.rs:351-385`) → phrasing should be xpub + path + master-fp; `purpose()` is private → direct `==44` asserts in an in-file `#[cfg(test)]` mod or drop for the e2e pins.
**MINOR M-2 (T1-a):** "386 tests" figure corresponds to neither (163 ms-codec runtime; ~283 ms-cli static). Substance (mutation survives all) empirically true; correct the number.
**MINOR M-3 (mechanics):** `crates/ms-cli/src/mlock.rs` exists; fmt gate `rust.yml:60-86` pinned 1.95.0 `cargo fmt --all -- --check` with mlock.rs filtered out. A committed reformatted mlock.rs passes CI silently while breaking g6 byte-identity. Rule: after any fmt, `git diff crates/ms-cli/src/mlock.rs` MUST be empty.

## Rulings
- **Release: NO-BUMP** (test-only; repo precedent BCH conformance-pin NO-BUMP). STOP clause → the fix cycle re-rules.
- Acceptance #3 full-package form correct.

**VERDICT: NOT GREEN — 0 Critical / 1 Important / 3 Minor.** T1-a funds-critical design verified sound + RED-proven by execution.

---
**FOLD STATUS (opus, 2026-07-10):** I-1 folded (T1-c adds the index-9 Chinese pin 这/這 + CN-S↔CN-T RED-proof). M-1 folded (T1-b: xpub+path+master-fp, no addresses; private purpose() → in-file cfg(test) or e2e-only). M-2 folded (163/163 empirical). M-3 folded (acceptance #4: mlock.rs git-diff-empty rule). NO-BUMP ruling recorded (acceptance #5). Re-dispatched for convergence → **round-2 GREEN (0C/0I)**: all 5 folds verified faithful, Chinese index-9 pins (这/這) re-confirmed against the embedded official lists, T1-a funds-critical design untouched. 1 cosmetic Minor (heading "first-word"→"wordlist-selection") fixed. **SPEC CONVERGED — implementation begun.**
