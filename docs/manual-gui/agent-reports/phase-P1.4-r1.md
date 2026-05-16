# Phase P1.4 (Track G — widget_help_icon kittest RED) — R1 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-help-icons` (mnemonic-gui)
**Scope:** Re-verify R0's C-1, I-1, I-2 folds.

**Verdict:** **LOCK 0C / 0I / 0N / 1n.**

## Folds verified

### C-1 (end_pass drain — corrected read path)

- `tests/widget_help_icon.rs:91-99` (sanity probe) and `:136-144` (real cell) both use `harness.output().platform_output.commands.iter().find_map(...)` — the `Harness::output()` accessor returning `&FullOutput`. Deprecated `PlatformOutput.open_url` no longer accessed.
- Sanity probe (`tests/widget_help_icon.rs:84-105`) empirically verifies the chain `Context::open_url` → `send_cmd` → `OutputCommand::OpenUrl` → end_pass drain into `FullOutput.platform_output` → `Harness.output` accessor. If the read path is broken, the sanity probe goes RED first.
- Source citations verified:
  - `egui-0.31.1/src/context.rs:2331` — end_pass drain ✓
  - `egui-0.31.1/src/context.rs:1452-1453` — Context::open_url → send_cmd ✓
  - `egui-0.31.1/src/data/output.rs:115` — deprecated field ✓
  - `egui_kittest-0.31.1/src/lib.rs:359` — Harness::output() accessor ✓
  - `egui_kittest-0.31.1/src/lib.rs:58` — Harness::ctx public field ✓
- Empirical: `cargo test --test widget_help_icon` shows sanity probe PASS + real cell FAIL at the `assert!(button.is_some(), ...)` line.
- Test independence: each `#[test]` fn constructs its own Harness; no shared global state.

**C-1: PASS.**

### I-1 (ASCII `?` U+003F)

- Plan §2.4 (lines 662-665): "small `egui::Button` with the ASCII character `?` (U+003F — NOT the fullwidth `？` U+FF1F or the emoji `❓` U+2753; the P1.4 G7 smoke test asserts `harness.query_by_label("?")` with byte-exact label-match semantics)." Explicit forbid; consistent with test's literal `query_by_label("?")` argument.
- Plan §3.1 P1.4 reinforces: "the affordance label is the ASCII `?` glyph, U+003F".

**I-1: PASS.**

### I-2 (render-site contract)

- Plan §2.4 (lines 671-681) "Render-site contract" paragraph: per-flag `?` button MUST be rendered inside `widget::render` or `widget::render_with_dispatch` in the same `ui.horizontal(...)` row as the flag label. Cites `widget.rs:78` (existing horizontal). Excludes the per-subcommand `?` button (lives at subcommand-selector ComboBox in main.rs).
- Source-confirmed: `mnemonic-gui/src/form/widget.rs:78` is the existing `ui.horizontal(|ui| {` open.

**I-2: PASS.**

## Off-by-N / path-drift check

- Panic-message line `tests/widget_help_icon.rs:126:5` — matches empirical panic.
- Plan §2.1 G7 + §3.1 P1.4 snippets mutually consistent on read path + expected URL.
- No internal SPEC contradiction (per-subcommand vs per-flag button contracts cleanly separated).

## Nit

### n-1 — Sanity-probe pattern is reusable; consider shared helper when second consumer appears

The push-known-URL-via-modern-API + read-back-via-corrected-path pattern catches "API looks correct on paper but doesn't work in practice" failures. When a second kittest test in this repo needs to read emitted commands, extract to `mnemonic-gui/tests/_kittest_helpers.rs`. **No action required this round** — one consumer; don't extract prematurely. Surface as deferred FOLLOWUP when P2.2 lands.

## Architect-must-run discipline assessment

R0 round 1 caught the deprecation via source-grep alone — insufficient. R0 round 2 caught the end_pass-drain only after running the test. The sanity-probe cell institutionalizes the discipline: any future egui internal change that breaks the read path will trip the sanity probe FIRST, surfacing the API drift before the real-cell assertion. This operationalizes `[[feedback-architect-must-run-prose-commands]]` as a durable test fixture.

## Verdict

**LOCK 0C / 0I / 0N / 1n.**

Folds complete, accurate, source-citation-grounded. C-1 sanity-probe is a meaningful improvement beyond just fixing the broken read path — it's a continuing guardrail. I-1 and I-2 SPEC clarifications unambiguously gate P2.2 implementation. No path-drift introduced. The single nit (n-1) is a forward-looking observation; no action required for P1.4 close.
