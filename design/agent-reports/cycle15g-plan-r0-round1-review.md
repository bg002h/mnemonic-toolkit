# R0 REVIEW — cycle-15 Lane G PLAN-DOC (mnemonic-gui secret-residue zeroize) — Round 1

> Reconstructed from the reviewer's reported verdict (notification tangling under high parallelism). Verified against `origin/master = 5ce9d53`. The implementer's whole-diff review is the authoritative post-impl gate.

## VERDICT: GREEN — 0 Critical / 0 Important

The plan operationalizes the GREEN spec faithfully; the E0509-safe pattern and run-holder scrub are correctly pinned.

### E0509-safe pattern (priority) — sound
The `main.rs:1064` rewrite binds the WHOLE `pending` struct (single-name bind is legal for a `Drop` type — only field-move-out triggers E0509), the modal reads `pending.argv`/`pending.mask` by-ref (`:1081-1088` confirmed by-ref), the Run path passes owned `.clone()`s (consumed by the run; the original `pending` drops scrubbed), and `RunResult`'s `Drop` reads-by-ref only (E0509-clean). The "do NOT delete Drop" guard + a compile-green/T3/T4 net is the right safety.

### Run-holder scrub completeness — sound
`impl Zeroize + Drop for RunResult` (argv `Vec<String>`, stdout/stderr `String` — element bytes zeroized) + `PendingConfirm` promoted to a Zeroize+Drop struct (moved to the `runner` lib module so the `scrub_app_run_holders` seam is harness-reachable) + explicit `on_exit` sweep coverage. No replace-without-drop bypass.

### Widget masking — sound
`.password(true)` gated on `node_type_is_argv_secret` (composite, `widget.rs:647/663`) + `is_xprv_like` (tree-key, `tree_model.rs:675`; render `tree_form.rs:697/717`), via `ui.add(TextEdit::singleline(...).password(cond))` preserving the paste-warn `Response` (M3 gate-hoist drives both). Predicates mask secrets, not public fields.

### T7 test seam — sound
Drives public `tree_form::render` with a constructed Key-node `FormState` (option i), proven in-repo via `tests/tree_form.rs::tree_form_harness`/`form_with_tree`/`enabled_tree` — no `pub(crate)` widening. kittest `Role::PasswordInput` assertions for the widgets; run-holder drop-scrub/zeroize-on-replace test for the holders.

### SemVer / gates — correct
GUI MINOR 0.46.0→0.47.0; PR + 5-target CI before tag; version sites `Cargo.toml` + `README.md` self-pin (`readme_pin_coherence`); toolkit pin UNCHANGED; NO schema_mirror (no clap-flag/dropdown change); never cargo-fmt; `MNEMONIC_BIN`=the v0.60.0 binary at `…/mnemonic-toolkit/target/release/mnemonic` for the drift tests ($PATH binary is the stale v0.56.0 false-fail trigger). FOLLOWUP flips (4 GUI slugs → resolved). Mandatory whole-diff review present.

## Disposition
GREEN. The lane may proceed to TDD (P1 run-holder scrub + E0509-safe rewrite → P2 widget masking). Ships GUI v0.47.0 via PR-gate.
