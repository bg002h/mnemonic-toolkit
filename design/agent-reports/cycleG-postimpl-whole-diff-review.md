# POST-IMPL WHOLE-DIFF REVIEW — Cycle G — round 1

**Reviewer:** fresh Fable (`claude-fable-5`), adversarial, cold read of `git diff 818df179 0fdd67cc` (7 files) in `/scratch/code/shibboleth/mnemonic-toolkit-cycleG`, verified against SPEC §1/§2/§4 + IMPLEMENTATION_PLAN (both R0-GREEN).
**Dispatched:** 2026-07-09 (Cycle G, post-impl whole-diff round 1). Persisted verbatim per CLAUDE.md.

## Verdict: GREEN (0C / 0I) — 3 Minors, all non-blocking

## Independent test/clippy counts

- `cargo test -p mnemonic-toolkit`: **3671 passed / 0 failed** (handful ignored, all pre-existing), full suite, run twice with identical results.
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`: **clean**.
- `cargo fmt -p --check`: diff confined to `src/mlock.rs` only (the permanent g6 exemption) — all touched files clean.

## Item 1 — the load-bearing no-wire-leak property: CONFIRMED, byte-identical, live-verified

I built the base binary **from source at `818df179`** (verified `git diff f4461c07 818df179 -- crates/` is empty, i.e. base code = v0.81.0 exactly) and byte-compared against the cycleG binary:

| Surface | Result |
|---|---|
| `repair --ms1 <flipped> --json` | stdout, stderr, exit (4) all **byte-identical**; corrected chunk present un-redacted in `corrected_chunks`/`original_chunk`/`corrected_chunk` (D9 UX intact) |
| `repair --ms1 <flipped>` (text) | **byte-identical** incl. advisory stderr |
| `repair --mk1 <flipped> --json` | **byte-identical**, exit 4 |
| `repair --md1 ×3 chunks --json` | **byte-identical**, exit 5, blessed verdict, all 3 chunks emitted |

Mechanism verified in code:
- (a) **No emitter uses `{:?}`** of `RepairOutcome`/`RepairDetail`/`RepairJson`/`AutoFireRepairJson`. Exhaustive grep: the only `{:?}` of these types is the new redaction test (`src/repair.rs:3021`). The `{:?}` hits in `verify_bundle.rs` are on codec errors / md tree structures (pre-existing, untouched, no chunk content — `RepairError` variants carry only codec `Display` detail text and 2-char HRPs; `SetVerify.reason` is advisory prose).
- (b) Both wire structs' `*Detail.original_chunk/corrected_chunk` stay `&'a str` fed by deref-coercion (`src/cmd/repair.rs:331-332`, `src/repair.rs:1858-1859`); text emitters use `{chunk}` Display (`cmd/repair.rs:284`, `repair.rs:1827`). Exactly the SPEC M1 shape.
- (c) `SecretString` (`src/secret_string.rs`): transparent `Serialize` (`serialize_str`), transparent `Display`/`Deref<Target=str>`, length-only redacting `Debug`, `Zeroizing<String>` inner → zeroize-on-drop, including every `.clone()` (Clone clones the Zeroizing wrapper).

**Migration surface COMPLETE**: `RepairDetail` is constructed at exactly 3 sites (`repair.rs:802/1223/1685`) — all wrapped; all 6 `Vec<SecretString>` producer sites wrapped; `verify_mk1_set` widened with `&*` deref (`:1052`); both wire structs widened. Completeness is compile-enforced (a missed `String` push would not build). The SPEC's M1 mention of "the indel path" is vacuous in practice — the indel path constructs no `RepairDetail`/`RepairOutcome` (it emits via its own `IndelCandidateJson`), so no change was needed there. See Minor 1.

**`PartialEq<str>`/`PartialEq<&str>` sound**: additive impls delegating to `String::eq`; the only production content-compare (`verify_bundle.rs`) uses explicit `&**c == expected_ms1` (`&str == &str`, bypasses the new impls); no auth/timing boundary exists (documented in-code, consistent with the cycle-14 D2 ruling).

**verify_bundle compare correct + comment fixed**: `.first().is_some_and(|c| &**c == expected_ms1)` is semantically identical to the old clone-and-compare in both arms (non-empty → str eq; impossible-empty → false, same as old `unwrap_or_default()` vs non-empty expected), drops the redundant clone, avoids `SecretString: Default` (G0-3 upheld). The stale "`held in Zeroizing`" doc-comment was rewritten accurately (plan M2).

## Floor-bump ruling (implementer deviation): CORRECT AND REQUIRED, not masking

The plan didn't list `tests/lint_zeroize_discipline.rs`, but the change is **forced by the existing gate**: `src/repair.rs` now matches `SECRET_PATTERNS` (`: SecretString`, `SecretString::new(`), so `every_secret_bearing_src_file_is_declared_or_allowlisted` would FAIL without a declaration. The implementer chose the lint's own preferred mechanism ("add a canonical row (preferred)") over allowlisting — correct for a genuinely secret-bearing file — with evidence strings that verbatim-match the real field declarations. I independently replicated the partition scan (grep over `src/**.rs` for all 4 patterns): **live count = exactly 40**, so `SECRET_FILE_FLOOR 39→40` is tight, in the loss-of-coverage (`>=`) direction only, and the floor comment history was extended accurately. `src/repair.rs` is not double-listed in `NON_ROW`/`TEST_ONLY`.

## Item 2 — compare-cost multipath: CORRECT, live-verified

- **Mirror exact**: `strip.rs` split (`clone()` → `into_single_descriptors()` → `is_empty()` guard → `remove(0)`) matches `derive_address.rs:34-46` structurally line-for-line, then feeds the existing `has_wildcard`/`TryFrom`/wrapper path — split-first as SPEC M5 requires.
- **Implementer's parse-rejection claim ACCURATE**: live run of the mismatched-branch fixture → **exit 2**, stderr `error: compare-cost: parse error: descriptor parse: At least two BIP389 key expressions ... tuples of derivation indexes of different lengths` — that is rust-miniscript's `MultipathDescLenMismatch` surfacing at `from_str` (the `descriptor parse:` prefix proves it's the parse `map_err`, not the split's `multipath split failed`). The `is_empty()` guard and the split `map_err` are therefore **dead-but-defensive** — same as the shipped `derive_address.rs` precedent — and cannot panic: `remove(0)` executes only after the non-empty check. `CompareCostError::Parse → exit 2` mapping confirmed at `cost/mod.rs:93`.
- **wsh ACCEPT**: `/<0;1>/*`, `/**`, `/0/*` all exit 0 with **identical `conditions`** (live: `wsh_vbytes:60/tr_vbytes:75` across all three). Base v0.81.0 binary rejects the same multipath descriptor ("multipath key cannot be a DerivedDescriptorKey") — genuine capability add.
- **wpkh UPDATE**: both spellings exit 3 with **byte-identical stderr** = `unsupported wrapper ...` (no derivation error). The updated test retains the cross-spelling exit+stderr equality asserts and adds the three pins (past-derivation, UnsupportedWrapper, not-unexpanded) — matches SPEC I1 "UPDATE not invert".
- **Single-path regression**: `wsh(multi(2,…/0/*,…))` `--json` **byte-identical** base vs new.
- Tests are non-tautological: exit-2 assert would catch a panic (abort) or silent truncation (exit 0); the acceptance test compares against an independently-computed single-path cost.

## Regressions / collateral

- mk1/md1/ms1 repair: representation-only — proven by the live byte-compares above across all three kinds (stdout+stderr+exit).
- `mlock.rs`: not in the diff, byte-identical (g6 intact).
- No version bump, no Cargo.toml/lock change, no clap/codec surface change, no `schema_mirror` impact — the release-ritual version sites are correctly deferred to post-GREEN per the plan.
- Stale-comment updates (M3): `strip.rs` Cycle-C block, `verify_bundle.rs` doc-comment, and the test-file comment all rewritten accurately.

## Minor findings (non-blocking)

1. **[Minor] `IndelCandidate.recovered: String` (`src/indel.rs:38-40`) remains un-zeroized with a derived non-redacting Debug.** Pre-existing, never `{:?}`-printed in production, and explicitly out of SPEC scope (§0 OUT 2 — closes only the filed FOLLOWUP). Suggest filing a follow-on FOLLOWUP (`indel-candidate-zeroization`) rather than expanding this cycle.
2. **[Minor] The malformed-multipath test asserts exit 2 but no stderr fragment** (`tests/cli_bip388_double_star_shorthand.rs:459-479`). A one-word pin (e.g. `"different lengths"` or `"parse error"`) would guard against a future different-Parse-class false-pass; deliberately loose coupling to upstream wording is a defensible trade — reviewer preference only.
3. **[Minor] The SPEC §2 optional manual note** ("multipath/`/**` now accepted, costed on the receive branch") was not added. SPEC marks it non-gating "add if low-effort" — fold into the release-ritual commit if desired.

## Release-readiness statement

**Cycle G is code-complete and cleared for release as `mnemonic-toolkit` v0.82.0, toolkit-only** (md/mk/ms NO-BUMP, no GUI/schema_mirror, no crates.io publish, no re-vendor, sibling pins untouched). Both load-bearing properties are independently verified: the zeroization migration is wire-byte-identical on every emitter (live-proven against a from-source v0.81.0 build), and compare-cost multipath is additive-accept-only with wpkh/malformed/single-path behavior correct. Suite 3671/0, clippy clean, fmt clean-modulo-mlock. Proceed to the release ritual (version sites incl. `install.sh` self-pin + gen.sh ~6 pins in `.examples-build`, CHANGELOG `[0.82.0]`, flip both FOLLOWUPs in the shipping commit, regen Examples.md, tag, verify `examples`/`changelog-check`/`install-pin-check`/`sibling-pin-check` CI). The three Minors do not gate; Minor 1 warrants a new FOLLOWUP entry at ship time.
