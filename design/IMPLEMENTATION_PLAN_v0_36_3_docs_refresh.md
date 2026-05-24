# v0.36.3 Implementation Plan — documentation refresh (README + manual hygiene)

> **For agentic workers:** per-phase TDD where a gate exists; per-phase opus reviewer-loop until 0C/0I; persist reviews to `design/agent-reports/` before the fold. NO parallel code-gen. Steps use `- [ ]`.

**Goal:** Remedy the documentation gaps found by the full doc audit (`cycle-prep-recon-documentation-audit.md`): refresh the severely-stale README (v0.8.0 → v0.36.2), wire the flag-coverage gate over the 2 un-validated chapters, fix the manual intro completeness + stale version stamps, and (anti-recurrence) add a lightweight README-version freshness guard. File a FOLLOWUP for the systemic prose-command-execution gate.

**Architecture:** Docs + one lint-input + one small new test. NO toolkit crate-logic change. The README is restructured to POINT to the authoritative surfaces (manual for the CLI reference, CHANGELOG for version history) instead of duplicating drift-prone per-version/per-subcommand detail — minimizing future re-decay — plus a machine-checked version marker.

**Tech Stack:** Markdown; `docs/manual/tests/lint.sh` (6-stage); a new `tests/readme_version_current.rs` (compares a README marker to `env!("CARGO_PKG_VERSION")`).

**Source SHA:** audit + citations vs `origin/master` @ `b2806d6` (2026-05-24).

---

## SemVer + lockstep
- **PATCH → v0.36.3** (docs/test-only; toolkit PATCH per the v0.28.5 docs-only-PATCH precedent — single coherent tag; README + cli-subcommands.list are toolkit-repo artifacts). Requires `Cargo.toml`/`Cargo.lock`/`install.sh:32` bump in lockstep (install-pin-check).
- **NO GUI schema_mirror lockstep** (no clap surface change). **Manual workflow** fires on the `docs/manual/**` change (validates 6 stages incl. the newly-wired electrum-decrypt/seedqr flag-coverage).
- No sibling-codec companions.

## Audit findings → phases
G1 README (Phase 3) · G2 cli-subcommands.list (Phase 1) · G3 intro + G4 stamps (Phase 2) · G5 prose-exec gate → FOLLOWUP (Phase 4).

---

## Phase 1 — G2: wire the flag-coverage gate over electrum-decrypt + seedqr

**Files:** `docs/manual/tests/cli-subcommands.list`

**Context:** the audit DRY-RUN confirmed both chapters are flag-complete — adding them is a clean gate-extension (no drift to fix). seedqr = `encode`/`decode` sub-subcommands (mirror `seed-xor split/combine`).

- [ ] **Step 1** — add to `cli-subcommands.list` (after `mnemonic nostr` / `mnemonic silent-payment`): `mnemonic electrum-decrypt`, `mnemonic seedqr encode`, `mnemonic seedqr decode`.
- [ ] **Step 2** — `make -C docs/manual lint MNEMONIC_BIN=<v0.36.3 binary> MD_BIN=md MS_BIN=ms MK_BIN=mk` → flag-coverage now validates those chapters + still GREEN (6/6). **If flag-coverage NOW fails, a chapter flag drifted → fix the chapter (escalation).** (Audit dry-run says it won't.)
- [ ] **Step 3** — commit.

## Phase 2 — G3 + G4: manual intro completeness + version stamps

**Files:** `docs/manual/src/40-cli-reference/41-mnemonic.md`, `docs/manual/src/60-appendices/68-release-history.md`

- [ ] **Step 1 — G3:** rewrite the `41-mnemonic.md:3` intro to enumerate ALL 20 subcommands (add `electrum-decrypt`, `seedqr`, `repair`, `inspect`, `compare-cost`, `xpub-search` as links to their `{#mnemonic-…}` anchors) + correct the count ("Twenty subcommands"). Confirm each anchor exists (the 6 added chapters have `## \`mnemonic <x>\`` headings — `repair`/`inspect`/`compare-cost`/`xpub-search` use auto-anchors; verify the link targets resolve via the index-bidirectional lint stage).
- [ ] **Step 2 — G4:** `41-mnemonic.md:14` "this chapter mirrors v0.13.0" → version-agnostic ("Run any with `--help` for the authoritative flag set; this reference tracks the current release."); `68-release-history.md:66` "as of v0.1's tag." → generalize or refresh.
- [ ] **Step 3** — `make -C docs/manual lint …` GREEN (markdownlint + cspell + lychee + index-bidirectional must still pass with the new intro links). Commit.

## Phase 3 — G1: README refresh + anti-decay version guard

**Files:** `README.md`; `crates/mnemonic-toolkit/tests/readme_version_current.rs` (new)

**Context:** the README decayed BECAUSE it duplicated drift-prone detail (per-version narrative + per-subcommand bullets) with no gate. Restructure to point at authoritative surfaces + add a machine-checked version marker.

- [ ] **Step 1 — RED: the freshness guard.** New `tests/readme_version_current.rs`:

```rust
//! Anti-decay guard: the README's stated toolkit version must match the crate
//! version. The README silently decayed v0.8.0 → v0.36.2 (28 versions) for lack
//! of this gate. Marker form (single source the release bump touches):
//!   <!-- toolkit-version: 0.36.3 -->  somewhere in README.md
use std::fs;
#[test]
fn readme_version_matches_crate() {
    let readme = fs::read_to_string(
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../README.md")
    ).expect("read README.md");
    let want = env!("CARGO_PKG_VERSION"); // 0.36.3
    let marker = format!("<!-- toolkit-version: {want} -->");
    assert!(
        readme.contains(&marker),
        "README.md must carry `{marker}` (it decayed to v0.8.0 once); update the \
         README status + marker in lockstep with the crate version",
    );
}
```
  (Confirm the README path relative to `CARGO_MANIFEST_DIR` = `crates/mnemonic-toolkit/` → `../../README.md`. R0 to verify the workspace layout / path.)
- [ ] **Step 2** — run; FAIL (no marker yet).
- [ ] **Step 3 — refresh README.md:**
  - **Status line** (`:13`): → `v0.36.2`-current; replace the v0.7/v0.8 feature narrative with a concise capability summary + a pointer to `CHANGELOG.md` for version history and `docs/manual/` for the authoritative CLI reference. Add the `<!-- toolkit-version: 0.36.3 -->` marker.
  - **Install** (`:35`): bump the example tag to `mnemonic-toolkit-v0.36.3` (or point at `scripts/install.sh` as the canonical, version-pinned installer to avoid a second hand-maintained pin).
  - **Subcommand inventory** (`:40-44`): replace the 5-bullet partial list with the full 20, grouped (bundle/verify-bundle/convert/import-wallet/export-wallet/decode-address/verify-message/…), each a terse one-liner pointing to the manual chapter for detail — NOT a re-creation of the manual (avoid re-drift). R0 to settle: full-20 terse bullets vs grouped-categories + "see the manual".
- [ ] **Step 4** — run the guard test → GREEN; commit.

## Phase 4 — FOLLOWUP + release + end-of-cycle

- [ ] **Step 1 — FOLLOWUP:** file `manual-prose-command-execution-gate` (G5: a lint stage / integration test that EXECUTES the documented round-trip recipes vs the pinned binary — the v0.28.1 breakage shipped because the lint only checks flag NAMES; meatier, own cycle + R0).
- [ ] **Step 2 — release-prep:** `Cargo.toml` 0.36.2 → 0.36.3; `Cargo.lock` regen; `scripts/install.sh:32` self-pin `mnemonic-toolkit-v0.36.3`; `CHANGELOG.md` [0.36.3] PATCH entry (docs-refresh).
- [ ] **Step 3 — full suite + clippy** (the new README guard runs under the crate test target) + `make -C docs/manual lint` GREEN.
- [ ] **Step 4 — end-of-cycle opus review** → persist `design/agent-reports/v0_36_3-end-of-cycle-review.md` → fold → GREEN.
- [ ] **Step 5 — ship toolkit:** merge→master (ff), tag `mnemonic-toolkit-v0.36.3`, push, GH release; verify rust + manual + install-pin-check CI. No GUI cycle.

---

## Self-review / open R0 questions
- **Spec coverage:** G2 (P1), G3+G4 (P2), G1 + guard (P3), G5 FOLLOWUP + release (P4). ✓
- **R0 must resolve:** (a) tagging model — PATCH v0.36.3 (proposed, per v0.28.5 precedent) vs manual-namespace; (b) README rewrite scope — full-20 terse bullets vs grouped + "see manual" (anti-re-drift); (c) the README freshness guard — is `<!-- toolkit-version: X -->` + `env!("CARGO_PKG_VERSION")` the right low-friction marker, and is the `CARGO_MANIFEST_DIR/../../README.md` path correct for the workspace layout (verify README is at repo root, crate is at `crates/mnemonic-toolkit/`)? (d) does adding a per-release README-version-bump obligation create undue lockstep friction, or is it justified given the 28-version decay? (e) confirm the 6 intro-link anchors resolve (the index-bidirectional lint stage is the backstop); (f) should the install example point at `scripts/install.sh` (single canonical pin) rather than a second hand-maintained tag in the README?
- **Type consistency:** the guard reads `README.md` via `CARGO_MANIFEST_DIR/../../README.md` + `env!("CARGO_PKG_VERSION")`.
- **No placeholder:** the README capability summary must be real + current (verified against the live 20-subcommand surface), not aspirational.
