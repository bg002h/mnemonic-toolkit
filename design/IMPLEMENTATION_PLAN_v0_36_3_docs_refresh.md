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
G1 **BOTH** READMEs (Phase 3) · G2 cli-subcommands.list (Phase 1) · G3 intro + G4 stamps + C2 anchor (Phase 2) · G5 prose-exec gate → FOLLOWUP (Phase 4).

**R0 C1 — TWO READMEs.** There are two distinct, both-stale READMEs: repo-root `README.md` (GitHub landing) AND `crates/mnemonic-toolkit/README.md` (the crate's PUBLISHED `readme=`, `Cargo.toml:11`, stale at v0.8.0 `:22`/`:182`). Both must be refreshed + marker'd + guarded — a guard on only the root README ships "green" with the canonical published README still v0.8.0.

---

## Phase 0 — version bump FIRST (R0 I2: the guard reads `CARGO_PKG_VERSION`)

**Files:** `crates/mnemonic-toolkit/Cargo.toml`, `Cargo.lock`, `scripts/install.sh`

The Phase-3 README guard asserts `<!-- toolkit-version: {env!("CARGO_PKG_VERSION")} -->`; `CARGO_PKG_VERSION` is read at test-COMPILE from `Cargo.toml`. So the crate version must be 0.36.3 BEFORE the marker is written, else the guard is unsatisfiable mid-cycle.

- [ ] **Step 1** — `Cargo.toml` 0.36.2 → 0.36.3; `cargo update -p mnemonic-toolkit` (Cargo.lock); `scripts/install.sh:32` self-pin `mnemonic-toolkit-v0.36.3`. Commit (`chore(v0.36.3): version bump first — guard reads CARGO_PKG_VERSION`).

---

## Phase 1 — G2: wire the flag-coverage gate over electrum-decrypt + seedqr

**Files:** `docs/manual/tests/cli-subcommands.list`

**Context:** the audit DRY-RUN confirmed both chapters are flag-complete — adding them is a clean gate-extension (no drift to fix). seedqr = `encode`/`decode` sub-subcommands (mirror `seed-xor split/combine`).

- [ ] **Step 1** — add to `cli-subcommands.list` (after `mnemonic nostr` / `mnemonic silent-payment`): `mnemonic electrum-decrypt`, `mnemonic seedqr encode`, `mnemonic seedqr decode`.
- [ ] **Step 2** — `make -C docs/manual lint MNEMONIC_BIN=<v0.36.3 binary> MD_BIN=md MS_BIN=ms MK_BIN=mk` → flag-coverage now validates those chapters + still GREEN (6/6). **If flag-coverage NOW fails, a chapter flag drifted → fix the chapter (escalation).** (Audit dry-run says it won't.)
- [ ] **Step 3** — commit.

## Phase 2 — G3 + G4: manual intro completeness + version stamps

**Files:** `docs/manual/src/40-cli-reference/41-mnemonic.md`, `docs/manual/src/60-appendices/68-release-history.md`

- [ ] **Step 1a — C2 anchor FIRST:** the `## \`mnemonic xpub-search\` (v0.26.0)` heading (`41-mnemonic.md:2553`) auto-slugs to `mnemonic-xpub-search-v0260` (version suffix) — a link `#mnemonic-xpub-search` would DANGLE. Add an explicit `{#mnemonic-xpub-search}` anchor to that heading (mirror the silent-payment/decode-address/verify-message explicit-anchor convention `:2061/:2100/:2124`). The other 5 (`repair`/`inspect`/`compare-cost`/`electrum-decrypt`/`seedqr`) auto-slug cleanly (clean headings) → `#mnemonic-repair` etc. resolve.
- [ ] **Step 1b — G3:** rewrite the `41-mnemonic.md:3` intro to enumerate ALL 20 subcommands (add `electrum-decrypt`, `seedqr`, `repair`, `inspect`, `compare-cost`, `xpub-search` as links) + correct the count ("Twenty subcommands"). **HAND-VERIFY all 6 new slugs** by deriving the GFM auto-slug for each (backticks stripped, lowercased, spaces→hyphens) — do NOT rely on the lint to catch a dangling fragment: lychee runs WITHOUT `--include-fragments` (`lint.sh:57`) and index-bidirectional checks `\index{}` not anchors, so a broken intra-doc `#…` ships silently. (A `--include-fragments` lint upgrade is out of scope → note in the G5/M-class FOLLOWUP set.)
- [ ] **Step 2 — G4:** `41-mnemonic.md:14` "this chapter mirrors v0.13.0" → version-agnostic ("Run any with `--help` for the authoritative flag set; this reference tracks the current release."); `68-release-history.md:66` "as of v0.1's tag." → generalize or refresh.
- [ ] **Step 3** — `make -C docs/manual lint …` GREEN (markdownlint + cspell + lychee + index-bidirectional must still pass with the new intro links). Commit.

## Phase 3 — G1: refresh BOTH READMEs + anti-decay version guard (R0 C1)

**Files:** `README.md` (repo root); `crates/mnemonic-toolkit/README.md` (crate-published, `Cargo.toml:11`); `crates/mnemonic-toolkit/tests/readme_version_current.rs` (new)

**Context:** the READMEs decayed BECAUSE they duplicated drift-prone detail (per-version narrative + per-subcommand sections) with no gate. Restructure BOTH to point at authoritative surfaces (manual + CHANGELOG) — keeping them short + low-drift — + add a machine-checked version marker to BOTH. (R0 disposition C: do NOT re-create the manual; manual is SOT per CLAUDE.md.)

- [ ] **Step 1 — RED: the freshness guard (checks BOTH READMEs).** New `tests/readme_version_current.rs`:

```rust
//! Anti-decay guard: BOTH READMEs (repo-root GitHub landing + the crate's
//! published `readme=` at crate root) must carry the current toolkit-version
//! marker. The READMEs silently decayed v0.8.0 → v0.36.2 (28 versions) for lack
//! of this gate. Marker form (one place the release bump touches per file):
//!   <!-- toolkit-version: 0.36.3 -->
use std::fs;
#[test]
fn both_readmes_carry_current_version_marker() {
    let want = env!("CARGO_PKG_VERSION"); // = Cargo.toml (0.36.3 after Phase 0)
    let marker = format!("<!-- toolkit-version: {want} -->");
    for rel in ["README.md", "../../README.md"] { // crate-dir + repo-root
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/").to_string() + rel;
        let body = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read {path}: {e}"));
        assert!(
            body.contains(&marker),
            "{path} must carry `{marker}` (READMEs decayed to v0.8.0 once); update \
             the README status + marker in lockstep with the crate version",
        );
    }
}
```
  (R0 B(d): no published-crate-build risk — integration tests run only via repo `cargo test -p mnemonic-toolkit`; toolkit is git+tag-only; both paths always present. `crates/mnemonic-toolkit/` → `README.md` = crate dir, `../../README.md` = repo root.)
- [ ] **Step 2** — run; FAIL (no markers yet). Cargo.toml is already 0.36.3 (Phase 0), so `CARGO_PKG_VERSION`=0.36.3 and the marker target is consistent (R0 I2 resolved).
- [ ] **Step 3 — refresh BOTH READMEs** (repo-root `README.md` + `crates/mnemonic-toolkit/README.md`), each:
  - **Status line:** → current; replace the v0.7/v0.8 feature narrative with a concise capability summary + pointers to `CHANGELOG.md` (version history) + `docs/manual/` (authoritative CLI reference). Add the `<!-- toolkit-version: 0.36.3 -->` marker (adjacent to the status line).
  - **Install:** point at `scripts/install.sh` as the canonical version-pinned installer (R0 disposition (f) — avoid a second hand-maintained tag pin); if a literal example tag is kept, it must read `mnemonic-toolkit-v0.36.3`.
  - **Subcommand inventory:** replace the partial list with a GROUPED one-liner inventory of all 20 (R0 disposition C: grouped categories + terse one-liners + "see the manual chapter for detail" — NOT per-flag detail, NOT a manual re-creation). Subsume the stale SPEC-pointer block (`README.md:50-59`, R0 M1) into the "see docs/manual + design/" pointer.
- [ ] **Step 4** — run the guard test → GREEN (both READMEs carry the 0.36.3 marker); commit.

## Phase 4 — FOLLOWUP + release + end-of-cycle

- [ ] **Step 1 — FOLLOWUPS:** file `manual-prose-command-execution-gate` (G5: a lint stage / integration test that EXECUTES the documented round-trip recipes vs the pinned binary — the v0.28.1 breakage shipped because the lint only checks flag NAMES; could also add lychee `--include-fragments` for anchor validation per C2; meatier, own cycle + R0). File `manual-yml-sibling-cli-pin-staleness` (R0 M2: `manual.yml:77/84/88` pins mk v0.4.1/md v0.6.0/ms v0.4.0 vs install.sh v0.4.2/v0.6.1/v0.4.1 — pre-existing, doesn't affect this cycle's mnemonic flag-coverage).
- [ ] **Step 2 — release-prep:** version bump already done in **Phase 0** (Cargo.toml/lock/install.sh = 0.36.3). Here: `CHANGELOG.md` [0.36.3] PATCH entry (docs-refresh).
- [ ] **Step 3 — full suite + clippy** (the new README guard runs under the crate test target) + `make -C docs/manual lint` GREEN.
- [ ] **Step 4 — end-of-cycle opus review** → persist `design/agent-reports/v0_36_3-end-of-cycle-review.md` → fold → GREEN.
- [ ] **Step 5 — ship toolkit:** merge→master (ff), tag `mnemonic-toolkit-v0.36.3`, push, GH release; verify rust + manual + install-pin-check CI. No GUI cycle.

---

## Self-review / R0 dispositions folded (all RESOLVED)
- **Spec coverage:** Phase 0 version bump (I2), G2 (P1), C2 anchor + G3 + G4 (P2), G1 BOTH READMEs + guard-both (P3), G5/M2 FOLLOWUPs + CHANGELOG + ship (P4). ✓
- **C1 (FOLDED):** BOTH READMEs (repo-root + crate-published `crates/mnemonic-toolkit/README.md`) refreshed + marker'd; the guard checks BOTH.
- **C2 (FOLDED):** explicit `{#mnemonic-xpub-search}` anchor added before linking; hand-verify all 6 slugs; the false "lint backstops anchors" claim removed (lychee no `--include-fragments`).
- **I1 (FOLDED):** G2 gate = stage-4 flag-coverage vs a freshly-built binary.
- **I2 (FOLDED):** Cargo.toml→0.36.3 in Phase 0 FIRST, so the guard's `CARGO_PKG_VERSION` matches the marker.
- **Dispositions:** (a) tagging = toolkit PATCH v0.36.3 (v0.28.5 precedent); (b/C) README = grouped one-liner inventory + "see manual/CHANGELOG", NOT 20-bullet duplication; (c) marker `<!-- toolkit-version: X -->` + `CARGO_PKG_VERSION` is the right low-friction floor (kills status-line decay; doesn't police narrative — acceptable); (d) per-release marker bump justified by the 28-version decay; (f) install → `scripts/install.sh` canonical. M1 (SPEC-block subsumed), M2 (manual.yml FOLLOWUP).
- **No placeholder:** README capability summary must be real + current vs the live 20-subcommand surface.
