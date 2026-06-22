# PLAN — GREEN the live-RED `sibling-pin-check` gate (md-cli pin alignment)

**Date:** 2026-06-22  **Cycle:** NO-BUMP CI chore (no toolkit version change)
**Source SHA pinned:** `origin/master` = `7cd3ccf1` (v0.70.1 tip). Citations grep-verified at this SHA.
**Parent:** open-followups maturity program; broken-gate-first (the same principle that drove the Wave-1 fmt fix). The fmt gate is now GREEN in CI; `sibling-pin-check` is the *second* live-RED gate.

---

## 1. Problem (CI-confirmed)
`sibling-pin-check.yml` fails on every recent master push (`gh run list`). Its logic (sibling-pin-check.yml:60-116): parse `scripts/install.sh` `component_info` canonical pins, scan every `.github/workflows/*.yml` `cargo install --git … --tag <tag> <pkg>` line, `exit 1` if any `<tag>` ≠ canonical. **Drift:** two md-cli sites pin `descriptor-mnemonic-md-cli-v0.6.2` while install.sh:35 canonical = `descriptor-mnemonic-md-cli-v0.7.1`:
- `.github/workflows/manual.yml:86` (md-cli install for the manual flag-coverage audit)
- `.github/workflows/cross-tool-differential.yml:46` (the walker-divergence comparison baseline)

mk-cli (v0.8.0) and ms-cli (v0.7.0) sites are aligned with canonical → not the cause.

## 2. Decision: Option A (align both md sites to canonical v0.7.1) — NOT exclude-from-gate
The `cross-tool-differential.yml` baseline comment (`:40-46`) explicitly states the pin **"matches scripts/install.sh:35's pin"** and pre-documents the "assess whether the md-codec delta is wire-neutral, so it does not confound the walker-divergence signal" reasoning. So the differential baseline is **designed to track install.sh canonical** and be moved **deliberately** (the gate enforces "not *silently*" — exactly what it caught here). Therefore: align, do not exclude. (Excluding a frozen baseline from the gate would be the wrong model — the comment shows the author intends alignment + a deliberate wire-neutrality check on each bump.)

**Why not de-stale all the way to latest (md v0.9.2 / ms v0.10.0 / mk v0.10.1)?** Recon confirmed that is a *separate, larger* cycle: it moves ms off v0.7.0, which trips the **frozen g6 mlock byte-equality anchor** (rust.yml:43-48 — ms-cli-v0.7.0's `mlock.rs` is not 1.95.0-formatted; moving it requires the coordinated mlock reformat + dropping the fmt exemption). That de-stale is best paired with the **codex32-vendor ms-cli publish** (the architect's Q1 carrier for the mlock re-baseline). This cycle only stops the RED bleeding; Option A leaves ms at v0.7.0 → the g6 anchor is untouched.

## 3. The funds-adjacent check (the one real risk)
md v0.6.2 → v0.7.1 bumps md-codec to `=0.37.0` (the #25 per-cosigner use-site override **funds-safety derivation fix**); md-cli's own source is **byte-identical** across the two tags (R0-verified `git diff --stat md-cli/` empty — only the Cargo md-codec pin moves). **Decisive wire-neutrality (R0 M1, corpus-structural):** the differential compares **encode-side ids** (`wallet_policy_id` + `wallet_descriptor_template_id`), **not** derived addresses, and **no corpus row has divergent per-cosigner suffixes** (every multi-key entry uses an identical `/<0;1>/*` on all keys) — so the #25 fix (which only affects cards with *divergent* per-cosigner use-site suffixes) has **no behavioral trigger** in this corpus on either side → Match holds by construction. **Mitigation (still mandatory before ship, as confirmation):** install md-cli v0.7.1 locally and run the differential test — it is a tool-vs-tool comparison (`cross-tool-differential.yml:51-55`: `MD_BIN=md cargo test -p mnemonic-toolkit --test cli_cross_tool_differential -- --ignored`), NOT bitcoind-gated, so it runs locally. **Must report pass/Match at v0.7.1 before committing.** (Expected neutral: the toolkit already carries the funds-fix via its own md-codec ≥0.39.0, so toolkit-vs-md-v0.7.1 should still match; the differential was green at v0.6.2 too.)

## 4. The change (no manual cascade — recon-verified)
The flag-coverage gate is one-directional (binary flag ⟹ documented in manual); the manual is already *ahead* of the pins (kept current in the display-grouping cycle), so bumping the md install forward documents nothing new → **zero manual-chapter edits**. The change is exactly two lines:
- `manual.yml:86`: `…md-cli-v0.6.2 md-cli` → `…md-cli-v0.7.1 md-cli`
- `cross-tool-differential.yml:46`: same bump. **(R0 M2 — non-optional):** refresh the baseline comment's stale `md-codec 0.35.0→0.35.1` skew note (`:43-44`) to cite the `=0.37.0` skew + its corpus-structural wire-neutrality — that comment is the design-intent the gate keys off; keep it honest.

## 5. Verification (all must pass before commit)
1. **Differential Match at v0.7.1** (§3) — local run, pass.
2. **sibling-pin-check GREEN** — run the gate's comparison logic locally (or re-grep: all workflow `--tag` md/ms/mk now == install.sh canonical).
3. **manual flag-coverage still GREEN** at md v0.7.1 (`make -C docs/manual audit` with MD_BIN=the-v0.7.1-md) — sanity, recon says no new flags.
4. CI post-push: `sibling-pin-check` + `cross-tool-differential` + `technical-manual` all green.

## 6. Out of scope (surfaced, not done here)
- **Full de-stale to latest** (md v0.9.2 / ms v0.10.0 / mk v0.10.1) — separate cycle, gated by the mlock-reformat coordination; pair with the codex32 ms-cli publish.
- **manual.yml `verify-examples` `41-inheritance.cmd` transcript drift (R0 I1 — reconciled):** R0 round-1 claimed this gate is live-RED, but it cited a PRE-SESSION stale run; current `gh run list` shows **`technical-manual` = success on HEAD** (the `verify_bundle.rs:2861` enriched string is on a code path `41-inheritance.cmd` does not exercise). No action. **The genuine second red gate was `changelog-check`** (the v0.70.1 tag lacked a `## mnemonic-toolkit [0.70.1]` CHANGELOG section — a v0.70.1 release-ritual miss) — **FIXED separately** (master `a662a8ce`, CHANGELOG section added + tag re-pointed), NOT part of this pin chore.
- **GUI sibling pin** (`mnemonic-gui-v0.40.0` in install.sh:44) + the Q4 MSRV "gate + raise to 1.88" decision — Wave-3 GUI pin work.

## 7. SemVer / ship
NO-BUMP (CI-only, no toolkit source/behavior change). Branch `chore/sibling-pin-check-green` off `7cd3ccf1`; verify §5; commit; FF to master; push; confirm CI green. No tag.

## 8. R0
One round requested (funds-adjacency of the md-codec derivation bump warrants a second opinion on the wire-neutrality reasoning + the Option-A-vs-exclude design call). Gate: 0C/0I before edits.
