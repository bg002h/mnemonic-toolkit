# SPEC R0 review — cycleG-zeroization-and-compare-cost-multipath — round 2

**Verdict: NOT GREEN (0 Critical / 1 Important / 2 Minor)** — one trivial one-line fold to converge.
**Reviewer:** Fable, per user directive. rev-2 @ `577abc67` vs live source.
**Dispatched:** 2026-07-09 (Cycle G, SPEC R0 round 2). Persisted verbatim per CLAUDE.md.

## Round-1 folds — ALL CORRECT (verified live)
I1 (§2 tests + §4.4/4.5 — UPDATE wpkh test to new `UnsupportedWrapper` error + ADD wsh acceptance; §2↔§4 consistent; unsupported-wrapper note added); M1 (§1 — both wire structs `RepairJson`/`AutoFireRepairJson`/`AutoFireRepairJsonDetail` + `verify_mk1_set` @:978 + `&*` @:1051, all anchors re-verified); M2 (§1 — drop Zeroizing+Option, `.first().is_some_and(|c| &**c==expected_ms1)`, guard @:2020-2025); M3 (§2 stale comments); M4 (§4.6 malformed fixture); M5 (§2 split-first mirror `derive_address.rs:34-60`); M6 (§4.7). SemVer MINOR ruled. No-wire-leak re-confirmed (both JSON paths serde-transparent SecretString; both text paths Display; zero `{:?}` in emitters).

## IMPORTANT — I1(r2): §0 item-2 scope summary still carries the stale rev-1 "INVERT the test" instruction
§0 item 2 (`:32-33`) still reads "INVERT the existing regression test that asserts multipath rejection → assert acceptance + the correct cost" — the exact instruction round-1 I1 established is IMPOSSIBLE (the test is `wpkh` = `UnsupportedWrapper` post-fix). §2 (`:80-85`) + §4.5 correctly say UPDATE-to-new-wrapper-error, but §0 was not harmonized → a direct §0↔§2 internal contradiction on the round-1 blocker (an implementer skimming §0 is re-pointed at the impossible path). **Fix (one line):** replace `:32-33` → "UPDATE the existing (`wpkh`) rejection test to assert the new `UnsupportedWrapper` error (multipath now gets past derivation), and ADD `wsh`-wrapper acceptance tests asserting the correct cost. `/**` inherits this for free (pre-expands to `/<0;1>/*` upstream, Cycle C)."

## Minors
- **M1(r2)** — count: §1 (`:24`) + Tests bullet (`:61`) say "~11 assert_eq!"; §4.3 correctly says "8 string-element". Real split (live): 8 element compares need `PartialEq<str>` (`repair.rs:1952,1973,2001,2012,2024,2108-2110`), 3 are `.len()` (`:1951,2094,2107`). Harmonize §1 "~11" → "8 string-element (+3 `.len()`)".
- **M2(r2)** — producer locals `let mut corrected_chunks: Vec<String>` @`:1098/1126/1660` push `chunk.clone()`/`corrected.clone()` (String) → become `SecretString::new(...)`. Compile-enforced + inside the named construction-sites scope → not a gap; mention only so the implementer expects it.

**Path to GREEN: fold I1(r2) (one-line §0 harmonization) + optionally M1(r2)/M2(r2); re-dispatch round 3. Design sound + complete.**
