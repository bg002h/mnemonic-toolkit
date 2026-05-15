# Phase P2.1 (Track G — Helper module + URL scheme) — R0 opus architect-reviewer

**Date:** 2026-05-15
**Branch:** `manual-gui-help-icons` (mnemonic-gui)
**Scope:** §3.2 P2.1 sub-phase — `mnemonic-gui/src/help/mod.rs` (NEW, 9 LOC) + `mnemonic-gui/src/help/url.rs` (NEW, 201 LOC including tests) + `mnemonic-gui/src/lib.rs:9` (one line `pub mod help;`).

**Verdict:** **LOCK 0C / 0I / 1N / 1n.**

All seven required source-grep verifications pass. `kebab()` in `src/help/url.rs:39-57` is byte-identical to the P1.5 reference at `tests/manual_anchor_coverage.rs:63-81`. The three helper signatures + `MANUAL_BASE_URL` const match SPEC §2.4 (plan lines 637-644) byte-for-byte. The P1.4 kittest URL pin at `tests/widget_help_icon.rs:147` matches what `manual_url_for_flag(CliTab::Mnemonic, "convert", "--from")` returns (asserted by the in-module test at url.rs:172-175). The `flag_anchor_strips_leading_dashes_only` test at url.rs:179-188 correctly distinguishes "strip leading dashes" from "kebab-fold" per SPEC §2.2 line 561.

The plan-prose at line 1000 says "Add `mod help;` declaration to `mnemonic-gui/src/main.rs`" but the executor added `pub mod help;` to `src/lib.rs` instead. This is the correct call: `src/main.rs:8-20` declares zero `mod ...;` lines and reaches everything via `use mnemonic_gui::...;` (verified at main.rs lines 12-19). Adding `mod help;` to main.rs would have either created a duplicate module or broken the lib-tests' access path. The plan-prose is mildly drift'd from the actual two-target package shape; fold inline as a minor SPEC note (see Nice-to-have N-1).

---

## Critical

None.

## Important

None.

## Nice-to-have

### N-1 — Plan-prose drift at §3.2 P2.1 bullet 4 (plan line 1000) — FOLDED

The plan says "Add `mod help;` declaration to `mnemonic-gui/src/main.rs`" but the correct site is `src/lib.rs:9` (where it now lives). `src/main.rs` declares no `mod ...;` lines (verified at main.rs:8-20 — only `use` statements). The executor's decision matches the dual-target lib+bin package shape and is consistent with how `app`, `form`, `schema`, etc. are declared in `lib.rs`. Folded inline as a one-line plan amendment in the P2 progress note.

## Nit

### n-1 — Module rustdoc at url.rs:6 mentions `main.rs` as a call site — REJECTED

R0's claim: "P2.2 button placements all live inside widget.rs" — INCORRECT. SPEC §2.4 paragraph 2 (plan lines 651-653) explicitly places the per-subcommand `?` button at "the subcommand-selector ComboBox" site, and the §2.4 render-site contract (plan lines 671-682) reinforces: "The per-subcommand `?` button lives at the subcommand-selector ComboBox site in `main.rs` per the bullet above — separate scope." The ComboBox is rendered at `main.rs:332` (`egui::ComboBox::from_label("subcommand")`), so `main.rs` IS a P2.2 call site for the per-subcommand button. The rustdoc at url.rs:7 is forward-looking and correct.

---

## Verification trace

1. **kebab() vs P1.5 reference (claim 1):** `src/help/url.rs:39-57` is character-by-character identical to `tests/manual_anchor_coverage.rs:63-81` modulo the `super::kebab` import. Both: `prev_dash=true` start, `is_ascii_alphanumeric` branch with `to_lowercase()`, dash-collapse via `prev_dash` flag, trailing-dash strip via `while out.ends_with('-')`. Agreement on all five P1.5 edge cases.

2. **Three helper signatures vs SPEC §2.4 (claim 2):** `manual_url_for_subcommand(tab: CliTab, subcommand: &str) -> String` at url.rs:82; `manual_url_for_flag(tab: CliTab, subcommand: &str, flag: &str) -> String` at url.rs:96; `manual_url_for_variant(tab: CliTab, subcommand: &str, flag: &str, variant: &str) -> String` at url.rs:106-111. Match plan lines 642-644 byte-for-byte.

3. **MANUAL_BASE_URL const (claim 3):** url.rs:31-34 byte-identical to plan lines 637-640.

4. **P1.4 kittest URL pinning (claim 4):** `tests/widget_help_icon.rs:147` pins `"https://bg002h.github.io/mnemonic-toolkit/manual-gui/#mnemonic-convert-from"`. Computed via helper composes to the identical byte sequence. The in-module test `flag_anchor_matches_p1_4_kittest_pinned_url` at url.rs:172-175 asserts this byte-for-byte. Lockstep parity holds.

5. **§3.2 P2.1 four deliverables (claim 5):** (a) `src/help/mod.rs` exists with `pub mod url;` at line 9. (b) `src/help/url.rs` exists with the three `manual_url_for_*` fns at lines 82, 96, 106-111. (c) `option_env!` at url.rs:31 with literal default at url.rs:33. (d) `pub mod help;` at `src/lib.rs:9` (correct site — main.rs declares no `mod` lines). Three of four bullets match prose exactly; bullet 4 is implementation-correct, plan-prose-drift'd (Nice-to-have N-1 folded).

6. **Flag-anchor strip-only rule (claim 6):** `anchor_for_flag` at url.rs:63-69 uses `flag.trim_start_matches('-')` — strips leading dashes only, no kebab-fold. Asserted by the `flag_anchor_strips_leading_dashes_only` test at url.rs:179-188 with `manual_url_for_flag(Md, "encode", "--key-origin")` → `#md-encode-key-origin`. Matches SPEC §2.2 line 561 and P1.5 line 95's identical `trim_start_matches('-')`.

7. **SPEC §2.4 example trace (claim 7):** `manual_url_for_variant(Mnemonic, "slip39-split", "--from", "entropy")` → `"https://bg002h.github.io/mnemonic-toolkit/manual-gui/#mnemonic-slip39-split-from-entropy"` — matches plan line 565 byte-for-byte. The `variant_anchor_kebabs_value` test at url.rs:196-199 asserts the exact URL.

8. **CliTab::bin_name() output (cross-check):** `src/app.rs:27-34` returns lowercase `"mnemonic"`, `"md"`, `"ms"`, `"mk"` — matches SPEC §2.2 `anchor(tab)` (plan line 558) byte-for-byte.

9. **RED state preservation for P1.4 + P1.5 cells:** P1.4 cell remains RED on the "? button not found" reason (not a compile error) — `cargo test --test widget_help_icon` shows `cell_help_icon_read_path_sanity_probe ... ok` + `cell_help_icon_emits_open_url_for_mnemonic_convert_from ... FAILED`. P1.5 cell `#[ignore]`-gated per its existing convention. Neither prematurely flipped GREEN.

10. **Build + unit-test confirmation (executor-run prose commands):** `cargo build --lib` clean. `cargo test --lib help::url::` → 7 passed; 0 failed. Compile-correctness end-to-end verified.

11. **Pre-existing clippy errors in `tests/manual_anchor_coverage.rs`:** `cargo clippy --all-targets -D warnings` surfaces 3 `doc_overindented_list_items` errors at lines 25-29. Verified pre-existing at P1.5 LOCK via `git stash && cargo clippy`. NOT a CI-gating failure — GUI repo CI runs `cargo build --release` (`.github/workflows/build.yml`) and `cargo test --workspace` (`.github/workflows/schema-mirror.yml`) without `-D warnings`. Out of scope for this sub-phase; will surface in P3 cycle-wide LOCK reviewer.

---

**Final verdict:** **LOCK 0C / 0I / 1N / 1n.** G-P2.1 helper module is source-faithful to SPEC §2.4 + §2.2 byte-for-byte. P1.4 kittest URL pinning + P1.5 kebab() reference are in byte-identical lockstep with the new helpers. Plan-prose drift at §3.2 P2.1 bullet 4 (main.rs vs lib.rs) folded inline as a one-line amendment. Executor proceeds to G-P2.2 (widget integration).

---

**Post-LOCK plan amendment (folded inline, additive):**

§3.2 P2.1 bullet 4 should read "Add `pub mod help;` declaration to `mnemonic-gui/src/lib.rs`" not `main.rs`. Reason: `main.rs` declares zero `mod` lines and reaches lib modules via `use mnemonic_gui::...;` imports. The lib+bin dual-target shape requires modules to be declared in `lib.rs` so both `tests/*.rs` integration tests and the binary entry can access them. All other modules (`app`, `form`, `schema`, etc.) follow this same convention.
