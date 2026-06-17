# C1 (unrestorable-shape advisory) impl review — code reviewer (verbatim)

> Reviewer: opus code reviewer (general-purpose, full tools — audited md-codec 0.36.0 source,
> ran suite/clippy/fmt/manual-lint, empirically proved non-vacuity). Branch
> `feature/unrestorable-shape-advisory` @ v0.57.1. Verdict GREEN (0C/0I).

---

**Verdict: GREEN (0C/0I)**

### Critical
None.

### Important
None.

### Minor
- **M1 (test isolation, informational).** The module unit tests (`unrestorable_advisory::tests`) only run via the `--bin mnemonic` test target (plain `mod` in main.rs); under lib they are gated behind `#[cfg(fuzzing)]`. This is the established dual-home pattern that `timelock_advisory` uses, so it's consistent — but the 4 unit tests do not run in a default `cargo test --lib` invocation. They DO run in `cargo test -p mnemonic-toolkit` (binary unittests target), confirmed (4 passed). No action needed.
- **M2 (cosmetic).** Hook site comments label the three bundle sites "(Site 1)", "(Site 3)", "(Site 2)" in source order — the numbers are out of order relative to placement. Harmless; mirrors the plan's site-id scheme. No action needed.

### What I verified (evidence)

1. **Parity oracle — HOLDS.** A deep audit of md-codec 0.36.0 enumerated every `Tag::SortedMulti` accept position: exactly THREE (`wsh(sortedmulti)` @to_miniscript.rs:205, `sh(wsh(sortedmulti))` @231, bare `sh(sortedmulti)` @248), all gated on `Body::Children` len==1 — matching `is_accepted_sole_child_sortedmulti` exactly. Every other position (combinator-nested, tr-scriptpath, bare top-level @174, `sh(wsh(combinator))`, thresh-nested) ERRORS at to_miniscript.rs:417 ("must be the sole child of wsh/sh"). The predicate fires TRUE on all of these. `Body::Variable`/`Body::Tr` recursion covers thresh and taproot; `Tag::SortedMultiA` (0x09) is distinct from `Tag::SortedMulti` (0x07) and correctly not matched. **No false positive, no false negative, no Body variant that hides a SortedMulti.**

2. **Hooks — correct.** All 4 sites pass the in-scope `stderr: &mut E` directly, placed immediately after the older() emit, using `&descriptor`/`&p.descriptor`. Non-blocking (no `?`, no exit-code path). import-wallet hook is inside `for p in &parsed`. **Scoping is complete:** of 7 `older_advisories_tree` sites, the 2 unpaired ones (`verify_bundle.rs:1096`, `xpub_search/descriptor_intake.rs:234`) were verified to NOT engrave an md1 — correct exclusion, no parity gap.

3. **Module decls — correct.** `pub mod unrestorable_advisory;` under `#[cfg(fuzzing)]` in lib.rs (alphabetical); plain `mod unrestorable_advisory;` in main.rs (alphabetical).

4. **Tests — non-vacuous + real parity.** 8 integration + 4 unit tests pass. Empirically proved non-vacuity: removing the import-wallet hook made `import_wallet_hardened_wildcard_fires_advisory` go RED (the older() prefix never overlaps `ADVISORY_PREFIX`, so tests can't pass for the wrong reason). Each positive feeds bundle AND restore (real parity). Negatives include all 3 sole-child shapes — crucially bare `sh(sortedmulti)` (the R0-I1 false-positive guard) — plus multi-in-combinator and shared-suffix. Restore assertions match restore.rs:1247/1254 wording exactly.

5. **Lockstep — complete.** All version sites at 0.57.1: Cargo.toml, README.md:13, crates/.../README.md:9 (both enforced by `readme_version_current` — passes), install.sh:32, main Cargo.lock, fuzz/Cargo.lock. CHANGELOG [0.57.1] block present. `git grep 0.57.0` leaves only legit historical/prose refs. Manual subsection `### Unrestorable descriptor shapes {#unrestorable-shapes}` + import-wallet cross-ref added (all anchors resolve; markdownlint 0 errors); cspell `unrestorable` added. FOLLOWUP umbrella flipped to RESOLVED with the 2 reconstruction-halves correctly left open. No ToolkitError variant, no clap flag, no schema_mirror surface.

6. **Build hygiene.** clippy clean (`--all-targets`, no warnings); fmt-check shows ONLY the known g6-exempt mlock.rs diff; full suite GREEN.

This is ready to tag `mnemonic-toolkit-v0.57.1`.
