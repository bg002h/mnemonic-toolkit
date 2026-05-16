# Phase P1.4 (Track G — widget_help_icon kittest RED) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-help-icons` (mnemonic-gui)
**Scope:** §3.1 P1.4 sub-phase — `mnemonic-gui/tests/widget_help_icon.rs` (NEW); plan §2.1 G7 + §3.1 P1.4 snippets corrected for `PlatformOutput::open_url` deprecation.

**Verdict:** **ITERATE 1C / 2I / 0N / 2n.**

## Critical

### C-1 — Read path `harness.ctx.output(|o| ...)` returns drained-empty PlatformOutput; URL capture impossible post-P2.2

The API-correction discovery (commands vec, not deprecated `open_url` field) is correct as far as it goes, but stops one mechanism layer too high. `Context::open_url(...)` → `send_cmd(...)` writes to `ctx.viewport().output.commands`, but `ContextImpl::end_pass` drains that field at the end of every frame (`egui-0.31.1/src/context.rs:2331`):

```rust
let mut platform_output: PlatformOutput = std::mem::take(&mut viewport.output);
```

The drained `PlatformOutput` becomes part of `FullOutput`, which `Harness::_step` stashes as `self.output: FullOutput`. By the time `harness.run()` returns, `ctx.viewport().output` has been reset to `Default::default()`, so `harness.ctx.output(|o| ...)` reads an empty PlatformOutput.

**Fix:** Read through `harness.output().platform_output.commands` (verified at `egui_kittest-0.31.1/src/lib.rs:359-361`):

```rust
let url = harness
    .output()
    .platform_output
    .commands
    .iter()
    .find_map(|cmd| match cmd {
        egui::OutputCommand::OpenUrl(open_url) => Some(open_url.url.clone()),
        _ => None,
    });
```

A pre-flight sanity test is cheap: manually call `harness.ctx.open_url(egui::OpenUrl::same_tab("test://"))`, run one frame, verify the read path captures `"test://"`.

## Important

### I-1 — Plan §2.4 ambiguous "Unicode glyph"; test asserts ASCII `?` exactly

SPEC §2.4 says "small egui::Button with `?` Unicode glyph". The test asserts `harness.query_by_label("?")` matches exactly — kittest does string-equality matching. If P2.2 implementer reads "Unicode glyph" and reaches for `？` (U+FF1F fullwidth) or `❓` (U+2753 emoji), the GREEN check breaks with no useful diagnostic.

**Fix:** Edit SPEC §2.4 to read: `small egui::Button with the ASCII character "?" (U+003F), `.small()` sizing, gray background.`

### I-2 — Test fixture uses `widget::render_with_dispatch`; SPEC §2.4 doesn't pin button render-site

The narrower probe (one `render_with_dispatch` call for `--from`) is correct only if P2.2 places the per-NVC `?` button inside that function's rendered Ui. SPEC §2.4 line 629-630 says "rendered to the right of the flag label" but does not say *which function* does the rendering.

The risk: if P2.2 implements the `?` button at the form-iteration level instead of inside `widget::render`, the probe at `render_with_dispatch` scope sees no button.

**Fix:** Add a SPEC §2.4 clarification: "The per-flag `?` button MUST be rendered inside `widget::render` or `widget::render_with_dispatch` (not at the form-iteration level), so that focused widget-level kittest probes catch it."

## Nice-to-have

(none)

## Nit

### n-1 — Comment doesn't yet mention end_pass drain mechanism

After C-1 fix lands, the test-file comment should note "`viewport.output.commands` is drained by `end_pass` at `egui-0.31.1/src/context.rs:2331`, so we read through `harness.output().platform_output.commands` instead."

### n-2 — `query_by_label` vs `get_by_label` choice

The test uses `query_by_label` plus a custom `assert!` to attach a SPEC §2.4 message — sound; better RED diagnostic than `get_by_label`'s built-in panic. Leave as-is.

---

## Verification ledger

- `PlatformOutput::open_url` deprecated: `egui-0.31.1/src/data/output.rs:115-116` ✓
- `Context::open_url` → `send_cmd(OutputCommand::OpenUrl)` → `commands.push`: `egui-0.31.1/src/context.rs:1452-1453, 1440-1441` ✓
- `PlatformOutput.commands` is modern slot: `egui-0.31.1/src/data/output.rs:108-109` ✓
- `viewport.output` drained at end_pass: `egui-0.31.1/src/context.rs:2331` ✓ (the load-bearing fact)
- `Harness::ctx` is public field: `egui_kittest-0.31.1/src/lib.rs:58` ✓
- `Harness::output() -> &FullOutput`: `egui_kittest-0.31.1/src/lib.rs:358-361` ✓
- `mnemonic convert --from` is NodeValueComposite: `mnemonic-gui/src/schema/mnemonic.rs:406-410` ✓
- Anchor formula → `mnemonic-convert-from`: plan §2.2 lines 534-537 ✓

---

**Final verdict:** **ITERATE 1C / 2I / 0N / 2n.** C-1 blocks LOCK. After folding C-1 (read FullOutput not live ctx + add sanity probe) and I-1 + I-2 (SPEC §2.4 clarifications), this phase should LOCK cleanly.
