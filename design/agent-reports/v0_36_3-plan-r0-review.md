# v0.36.3 — Plan-doc R0 architect review (opus) — MANDATORY pre-implementation gate

**Date:** 2026-05-24
**Cycle:** v0.36.3 documentation refresh (README + manual hygiene)
**Branch:** `v0.36.3-docs-refresh`
**Reviewer:** opus (feature-dev:code-reviewer), R0 (agentId a1863d287505203ff)
**Target:** `design/IMPLEMENTATION_PLAN_v0_36_3_docs_refresh.md`

---

## Critical

**C1 — TWO READMEs; the crate's PUBLISHED readme (`crates/mnemonic-toolkit/README.md`, per `Cargo.toml:11 readme="README.md"`, resolved relative to the manifest dir) is ALSO stale at v0.8.0 (`:22` clone-tag v0.8.0, `:182` v0.8.0 entry) and the plan + guard touch ONLY the repo-root README.** The guard reads `CARGO_MANIFEST_DIR/../../README.md` (repo root) — so the cycle would ship "green" with the crate's own canonical front door still v0.8.0, defeating the cycle's thesis. **Fix:** refresh BOTH READMEs + version-marker in BOTH + guard checks BOTH (OR collapse to one canonical + thin pointer; note the crate is git+tag-only so docs.rs/registry don't consume it, but the file is still the `readme=` target).

**C2 — intro link `#mnemonic-xpub-search` will NOT resolve + the claimed lint backstop is false.** Heading is `## \`mnemonic xpub-search\` (v0.26.0)` (`41-mnemonic.md:2553`) → GFM auto-slug `mnemonic-xpub-search-v0260`, not `mnemonic-xpub-search`. The other 5 (repair/inspect/compare-cost/electrum-decrypt/seedqr) auto-slug cleanly. Backstop is illusory: lychee runs `--offline` WITHOUT `--include-fragments` (`lint.sh:57` → fragment anchors unchecked); index-bidirectional (`lint.sh:114-137`) checks `\index{}`, unrelated; no mdBook build-link gate. **Fix:** add explicit `{#mnemonic-xpub-search}` anchor to the heading (mirror silent-payment/decode-address/verify-message `:2061/:2100/:2124`); remove the false "lint backstops anchors" claim; hand-verify all 6 slugs.

## Important

**I1 — G2's gate is stage-4 flag-coverage, NOT index-bidirectional.** `lint.sh:62-98` reads `cli-subcommands.list`, runs `$MNEMONIC_BIN $sub --help`, greps flags into `41-mnemonic.md`; CI builds a fresh binary (`manual.yml:94/103`). State this precisely so the implementer runs the right gate (the C2 conflation risks "verifying" the wrong stage).

**I2 — Phase-3 GREEN is self-contradictory.** The guard asserts README contains `<!-- toolkit-version: {env!("CARGO_PKG_VERSION")} -->`; `CARGO_PKG_VERSION` = Cargo.toml at test-compile. Phase 3 writes marker `0.36.3` while Cargo.toml is still `0.36.2` (bump deferred to Phase 4) → guard FAILS at end of Phase 3. **Fix:** bump `Cargo.toml` → 0.36.3 FIRST (early phase), so `CARGO_PKG_VERSION`=0.36.3 throughout and the marker matches.

## Minor
- **M1** — README SPEC-pointer block (`README.md:50-59`) is also stale (newest `SPEC_export_wallet_v0_7.md`); the restructure should subsume/replace it, not leave a stale SPEC list beside a fresh status line.
- **M2** — `manual.yml:77/84/88` pins stale sibling tags (mk v0.4.1, md v0.6.0, ms v0.4.0) vs `install.sh` v0.4.2/v0.6.1/v0.4.1. Pre-existing, doesn't affect the mnemonic flag-coverage this cycle wires; one-line FOLLOWUP note.

## Verification summary (confirmed correct)
- **A(a)** README v0.8.0 vs Cargo 0.36.2 (28-version gap); install pin v0.13.0; 5/20 subcommands — CONFIRMED.
- **A(b)** cli-subcommands.list lacks electrum-decrypt + seedqr; correct form `mnemonic electrum-decrypt` (flat, main.rs:79) + `mnemonic seedqr encode`/`decode` (seedqr.rs:48/50, mirror seed-xor split/combine); flag-coverage reads this file → G2 wires the gate. CONFIRMED.
- **A(c)** intro "Fourteen" omits the 6; 20 live top-level variants (main.rs:63-104); all 20 have chapters. CONFIRMED.
- **A(d)** both stamps CONFIRMED.
- **B(a)** guard path resolves to repo root (matches `tests/design_artifacts_presence.rs:13` idiom) — but wrong/incomplete target (C1).
- **B(b)** marker mechanism feasible (CARGO_PKG_VERSION=crate version), modulo I2 ordering.
- **B(c)** DISPOSITION: guard is net-good (kills status-line decay, low-friction) but insufficient alone (doesn't police feature narrative/inventory) — acceptable FLOOR provided C1 fixed; marker adjacent to status line.
- **B(d)** no published-crate-build risk — tests run only via repo `cargo test -p mnemonic-toolkit` (rust.yml:54); toolkit git+tag-only; `../../README.md` always present. CONFIRMED.
- **C** DISPOSITION: point-at-authoritative-surfaces is correct (full 20-bullet duplication is what decayed + violates manual-is-SOT). Recommend: status line + marker + brief capability paragraph + grouped one-liner inventory + "see docs/manual/ + CHANGELOG.md".
- **D** DISPOSITION sound: v0.28.5 was a docs-only PATCH w/ toolkit tag (CHANGELOG:772). PATCH v0.36.3 = single coherent tag; needs Cargo.toml/lock/install.sh:32 bump (install-pin-check greps install.sh on `mnemonic-toolkit-v*` tag); NO GUI lockstep; manual workflow fires on docs/manual/**.
- **E** 5/6 anchors auto-slug clean; xpub-search broken (C2).
- **F** lint mechanics CONFIRMED; README not in lint scope (markdownlint/cspell/lychee scan docs/manual/src only) so README prose can't trip those; the new manual intro text CAN.

VERDICT: RED (2C/2I)

---

## Fold disposition (controller) — R0 → R1
- **C1:** address BOTH READMEs. Refresh repo-root + `crates/mnemonic-toolkit/README.md` to current; put the `<!-- toolkit-version: X -->` marker in BOTH; the guard test asserts the marker in BOTH files (`CARGO_MANIFEST_DIR/README.md` + `CARGO_MANIFEST_DIR/../../README.md`). (Considered collapsing the crate README to a thin pointer — but keeping both current + guarded is simpler + safe; the restructure makes both point at manual/CHANGELOG so they're short, low-drift.)
- **C2:** add explicit `{#mnemonic-xpub-search}` anchor to the `## \`mnemonic xpub-search\` (v0.26.0)` heading; hand-verify all 6 new intro slugs; DELETE the false claim that the lint backstops anchor resolution.
- **I1:** Phase 1/2 state G2's gate = stage-4 flag-coverage vs a fresh binary.
- **I2:** move the `Cargo.toml`→0.36.3 bump to the FIRST phase (new Phase 0 release-prep-start: Cargo.toml + Cargo.lock + install.sh:32), so the guard's `CARGO_PKG_VERSION`=0.36.3 matches the marker in every subsequent phase.
- **M1:** restructure subsumes the README SPEC-pointer block. **M2:** file `manual-yml-sibling-cli-pin-staleness` FOLLOWUP.
Re-dispatch R1.
