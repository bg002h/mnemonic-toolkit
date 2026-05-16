# Phase P2.2 (Track G — Widget integration) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-help-icons` (mnemonic-gui)
**Scope:** §3.2 P2.2 sub-phase — `mnemonic-gui/src/form/widget.rs` (signature edits + `render_help_icon` helper + `needs_help_icon` predicate), `mnemonic-gui/src/main.rs` (per-subcommand `?` at ComboBox + per-`--slot` `?` at "Slot rows:" label + plumbing of `(active_tab, &active_sub_name)` into `render_with_dispatch`), `mnemonic-gui/tests/widget_help_icon.rs` (call-site signature update only — step()-vs-run() amendment deferred to G-P2.3).

**Verdict:** **LOCK 0C / 0I / 0N / 0n.**

All eight required source-grep verifications pass. The §2.4 render-site contract (plan lines 671-682) is honored: `render_help_icon` is called inside both `render`'s `ui.horizontal` row (widget.rs:141, 143) and `render_with_dispatch`'s secret-path `ui.horizontal` row (widget.rs:76, 79). The per-subcommand `?` button lives at main.rs:354-362 next to the ComboBox per §2.4 paragraph 2's explicit `main.rs` placement. The per-`--slot` button at main.rs:432-448 is correctly gated on `sub.allows_slots` and correctly routed around the widget.rs early-`continue` at main.rs:400-402. The `needs_help_icon` predicate at widget.rs:27-32 evaluates the exact §1.6 Option C union (Dropdown ∪ NodeValueComposite ∪ TaggedOrIndexed ∪ `repeating == true`). Button arithmetic 28 + 43 + 20 = 91 verifies against source: 28 SubcommandSchema constructors across 4 schema modules, 43 enumerated `FlagKind::{Dropdown,NodeValueComposite,TaggedOrIndexed}(` occurrences across 3 modules (mk has 0), 20 FlagSchema-bearing `repeating: true` occurrences (positional-arg `repeating: true` excluded). The variant axis is correctly elided: widget.rs:48 calls `manual_url_for_flag` (NOT `manual_url_for_variant`), one `?` per enumerated flag, never per-variant.

---

## Critical

None.

## Important

None.

## Nice-to-have

None.

## Nit

None.

---

## Verification trace

1. **§2.4 render-site contract (claim 1).** `render_help_icon` is called inside `render`'s `ui.horizontal` block at widget.rs:143 (immediately after `ui.label(flag.name).on_hover_text(flag.help)` at line 142), and inside `render_with_dispatch`'s secret-path `ui.horizontal` block at widget.rs:79 (between `widget.show(...)` at line 78 and the required-asterisk at line 80-82). Both render-sites satisfy the §2.4 contract paragraph requirement that the per-flag `?` button live "in the same row as the flag label." The per-subcommand `?` button lives at main.rs:354-362 directly inside the same `ui.horizontal(...)` closure that hosts the ComboBox (opened at main.rs:333) — separate scope per §2.4 paragraph 2. Plan amendment at lines 671-682 is fully honored.

2. **§1.6 button-count audit (claim 2).** Per-subcommand: 28 `SubcommandSchema {` constructors verified by Grep. Per-Dropdown/NVC/TaggedOrIndexed: 43 schema occurrences = mnemonic 35 (28 Dropdown + 6 NodeValueComposite + 1 TaggedOrIndexed) + md 5 + ms 3 + mk 0. Per-repeating FlagSchema: 20 = mnemonic 10 + mk 4 + md 6. PositionalArgSchema repeating-true correctly excluded because positionals go through main.rs:452-465 rendering, not through widget.rs. Total = 28 + 43 + 20 = 91. Matches plan §1.6 line 253.

3. **`needs_help_icon` predicate exactness (claim 2 cont'd).** widget.rs:27-32 `matches!(flag.kind, FlagKind::Dropdown(_) | FlagKind::NodeValueComposite(_) | FlagKind::TaggedOrIndexed(_)) || flag.repeating` — verified against FlagKind enum at schema/mod.rs:84-106. The three enumerated variants are the only enumerated-display kinds, matching §1.6 line 240 "Per-dropdown / per-NodeValueComposite / per-TaggedOrIndexed".

4. **`--slot` placement (claim 3).** All 3 `--slot` FlagSchema occurrences (mnemonic.rs:235, 395, 611) have `repeating: true`. All 3 corresponding subcommands are `allows_slots: true`. Therefore widget.rs's render loop ALWAYS short-circuits on `--slot` via main.rs:400-402 `if flag.name == "--slot" && sub.allows_slots { continue; }`. The `?` button for `--slot` is rendered at main.rs:432-448, gated on `sub.allows_slots`, calling `help_url::manual_url_for_flag(active_tab, &active_sub_name, "--slot")`. Anchor scheme is identical for any other repeating flag — only the render-site differs. Consistent with §2.4's render-site-contract purpose.

5. **Per-subcommand URL composition (claim 4).** main.rs:359 calls `help_url::manual_url_for_subcommand(active_tab, &active_sub)`. Verified against help/url.rs:82-87 which composes `format!("{MANUAL_BASE_URL}#{tab.bin_name()}-{kebab(subcommand)}")` per §2.2 anchor scheme + §2.4 line 651-653. Both call-site (main.rs:354 button creation + 357-361 click handler) and URL helper agree on `OpenUrl::new_tab(...)` per §3.2 P2.2 bullet 4.

6. **Variant axis behavior (claim 5).** widget.rs:48 calls `url::manual_url_for_flag(tab, subcommand, flag.name)` — NOT `manual_url_for_variant`. Correct per §1.6 + §2.4: ONE `?` button per flag, linking to the flag anchor, not per-variant. The `manual_url_for_variant` helper at url.rs:106-115 is currently unused by widget.rs (future P2.4-or-later use only).

7. **Conditional-visibility interaction (claim 6).** main.rs:412-423 wraps the `widget::render_with_dispatch` call in `ui.add_enabled_ui(!matches!(v, Visibility::Disabled), |ui| { ... })`. The `?` button rendered inside `render_help_icon` (widget.rs:43-50) is inside that wrapper's closure, so egui's `add_enabled_ui` semantics correctly grey it out when the flag is `Disabled`. `Visibility::Hidden` is handled by main.rs:404-406 `continue` before the wrapper executes, so the `?` button is correctly skipped for hidden flags.

8. **Secret-path `?` button (claim 7).** widget.rs:75 branch (`flag_is_secret(flag) && matches!(flag.kind, FlagKind::Text)`) calls `render_help_icon` at line 79. Since the branch additionally requires `FlagKind::Text`, `needs_help_icon`'s enumerated-kind disjunct is always false on this path; only `flag.repeating` can make `needs_help_icon` true. Verified secret-AND-repeating-AND-Text FlagSchemas exist (mnemonic.rs:319-326 `--ms1`, 328-335 `--mk1`, 336-340 `--md1`, 841-846 `--share` of `slip39-combine`). For each of these the `?` button correctly renders. Non-repeating Text-secret flags (e.g., `--passphrase`) correctly get NO `?` button — they remain tooltip-only per §1.6.

9. **Pre-existing test scope (claim 8).** Grep `widget::render_with_dispatch` returns exactly 2 hits: main.rs:415 (call-site, updated to pass `active_tab, &active_sub_name`) and tests/widget_help_icon.rs:131 (updated to pass `CliTab::Mnemonic, "convert"`). All other tests (argv_assembler.rs, widget_secret.rs, secrets.rs, persistence.rs) use `form::secret_widget::SecretLineEdit` — they do NOT touch `widget::render*`, so the signature change does not affect them.

10. **Test-side call-site update (claim 8 cont'd).** tests/widget_help_icon.rs:131 passes `CliTab::Mnemonic, "convert"` — the same `(tab, subcommand)` pair the helper-side test at help/url.rs:172-175 pins to URL `"https://bg002h.github.io/mnemonic-toolkit/manual-gui/#mnemonic-convert-from"`. The test's assertion at widget_help_icon.rs:166 expects exactly this URL byte-for-byte. With G-P2.2 widget code wired, accesskit-mapped `?` button → click → `ctx.open_url(...)` produces the asserted URL when run via the G-P2.3 step()-not-run() amendment.

11. **Glyph + button construction (cross-check).** widget.rs:43-45 builds `egui::Button::new("?").small().fill(egui::Color32::from_gray(96))`. Identical construction at main.rs:354-356 (subcommand) and main.rs:436-438 (`--slot`). Glyph is the ASCII `?` (U+003F) per the SPEC §2.4 byte-exactness contract (plan lines 662-664). Three call sites are byte-for-byte consistent — no glyph drift.

12. **CliTab Copy-ness + parameter passing.** CliTab at src/app.rs:16-22 derives `Copy`. Pass-by-value `tab: CliTab` at widget.rs:39 (`render_help_icon`), 68 (`render_with_dispatch`), 134 (`render`) is the correct Rust idiom. `subcommand: &str` borrowed at the same sites avoids unnecessary clones; the caller's `&active_sub_name` at main.rs:418 borrows the BTreeMap-cloned String, sustained for the duration of the dispatch call. No lifetime hazards.

13. **Executor's cargo verification (closing the R0 prose-command gap).** The reviewer was tool-scoped (read/grep only) so couldn't run cargo; the executor independently verified the prose-command suite before dispatch: `cargo build --lib` clean, `cargo build --bin mnemonic-gui` clean, `cargo build --all-targets` clean, `cargo test --lib help::url::` 7/7 passed, `cargo test --test widget_help_icon` 2/2 GREEN (after the G-P2.3 step() amendment), all other non-stub tests (argv_assembler, conditional_visibility, copy_command, dropdown_id_salt, path_detect, widget_interaction, widget_secret) all-green. Pre-existing failures in persistence/schema_mirror/runner_integration/secrets confirmed unchanged on baseline via `git stash && cargo test`.

---

**Final verdict:** **LOCK 0C / 0I / 0N / 0n.** G-P2.2 widget integration is source-faithful to §1.6 Option C (91 buttons; 28 + 43 + 20 arithmetic verified against schema source byte-for-byte) and to §2.4's render-site contract (per-flag inside widget.rs render bodies; per-subcommand at main.rs ComboBox; per-`--slot` at main.rs "Slot rows:" label per the early-`continue` interaction). The signature plumbing of `(tab: CliTab, subcommand: &str)` through `render` and `render_with_dispatch` is contained to two live call-sites (main.rs:415, tests/widget_help_icon.rs:131); no transitive test-file breakage. The variant axis is correctly elided (helper not called from widget.rs). The conditional-visibility `Disabled` wrapper at main.rs:412-423 correctly disables the `?` button transitively. The secret-path `?` button renders iff the secret-Text flag is also repeating, matching §1.6's load-bearing-dropdown-or-repeating predicate. Executor proceeds to G-P2.3 (kittest GREEN — including the step()-vs-run() amendment, properly scoped out of this review).
