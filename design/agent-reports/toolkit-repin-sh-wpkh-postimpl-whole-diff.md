# Post-impl whole-diff R0 — Track B toolkit v0.87.0 (sh(wpkh) re-pin) — Opus, adversarial

**Persisted per CLAUDE.md.** VERDICT: **GREEN (0C / 0I).** Toolkit v0.87.0 sound to tag + ship.

## Per-phase R0 trail (all GREEN, opus; results in session transcript)
- **B0** (pin 0.40→0.41 + re-vendor from PUBLISHED 0.41.0 + F-A8 `MalformedPayloadPadding` arm + repair non-zero-pad cell): GREEN. Vendor provenance verified (0.41.0, no 0.42.0 leak).
- **B1** (4 sh(wpkh) flip tests + 2 audit guards + 6 comment updates + gui-schema cell): GREEN. Flip #1 empty/master empirically confirmed; guards MUTATION-PROVEN (adding an `Sh` arm to `cli_template_from_tree` breaks both).
- **B2** (manual 5→6 + GUI companion FOLLOWUP + v0.87.0 release ritual): GREEN. Migration note correct; all gates green.

## Whole-diff empirical (the funds crux)
- **Flip #1 destination — VERIFIED empty/master.** `bundle --descriptor "sh(wpkh(@0))"` no-origin → `origin_path: null`, decoded mk1 = empty origin / depth-0 master xpub, NO notice, byte-parity with `wpkh(@0)`. No `48'`, no `49'`.
- **Migration note — CORRECT.** CHANGELOG v0.87.0: pre `m/48'/0'/0'/1'`+notice → post elided/master (wpkh-parity); "do NOT expect a `49'` card"; pass explicit `[fp/48'/0'/0'/1']`. Reviewer empirically confirmed the inline-origin route (`sh(wpkh([73c5da0a/48'/0'/0'/1']@0))`) reproduces the pre-repin wallet from a seed.
- **Flip #4 — no false-pass.** Pre-flip fixture → `result:"mismatch"`, exit≠0; `mk1_path_match` expected `""` vs actual `48'/0'/0'/1'`; `md1_xpub_match:false`. Fail-loud, never ok.
- **Vendor provenance — CLEAN.** `vendor/md-codec` 0.41.0; `EmptyOriginOverride`×0, `DecodeOpts`×0 (no Track-A 0.42.0 leak); `.cargo-checksum.json` == Cargo.lock; ALL 111 vendored files match recorded checksums (proper cargo-vendor). Drift confined to `vendor/md-codec/`.
- **Test count — no dropped tests.** 3724/0/18; diff removes 0 `#[test]`, adds 0 `#[ignore]`, deletes no test file (+15 tests). "3619" was a narrower counting method.

## Release ritual + gates
Cargo.toml 0.87.0; both READMEs `toolkit-version:0.87.0`; root+fuzz Cargo.lock (toolkit 0.87.0, md-codec 0.41.0); install.sh SELF-pin v0.87.0, sibling `md-cli-v0.11.2` FROZEN; `.examples-build` banner-only; manual `:418` five→six; both slugs RESOLVED v0.87.0; GUI companion FOLLOWUP filed; F-A2/F-A9/F-A3/F-A4 dispositions in CHANGELOG. `git diff mlock.rs` EMPTY; no fmt collateral. Suite 3724/0; clippy clean; vendor-freshness OK.

## Findings
Critical: none. Important: none.
**Minor (non-blocking):** (1) the migration note's SECONDARY `--slot` hint refuses for `[Phrase,Path]` (I1-by-design; the PRIMARY inline-origin example covers seed + watch-only, empirically verified; refusal is fail-loud). (2) SPEC S6 cited `mnemonic-gui/design/FOLLOWUPS.md` but that repo's canonical file is repo-root `FOLLOWUPS.md` — filed correctly there (stale SPEC cite).
**Caveat (not a defect):** `make -C docs/manual lint`/`verify-examples` NOT run locally (needs the frozen sibling binaries; local checkouts ahead → false drift). Provably invariant under Track B (prose-only manual edit, no flag change, no sh(wpkh) in the corpus). CI `manual.yml` runs it on the tag push.

## VERDICT: GREEN (0C/0I) — tag + ship toolkit v0.87.0.
