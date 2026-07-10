# SPEC R0 review — F5+F6 GUI recovery wiring — round 1

**Reviewer:** Fable (SPEC R0, read-only), per user directive. SPEC @ GUI `f5cb11f` / toolkit `3d985798`.
**Dispatched:** 2026-07-10 (F5+F6, SPEC R0 round 1). Persisted verbatim per CLAUDE.md.

**VERDICT: NOT GREEN — 0 Critical / 3 Important / 3 Minor.** Core design verified correct + complete (5-parent-prefix split, PREPEND sentinel, surgical scope). The Importants are completeness gaps, each with a mechanical fold; I-3 needs a real design decision.

## Verified GREEN (load-bearing)
- **#1 F5 split rule COMPLETE + UNAMBIGUOUS.** Exactly 5 child `#[command(subcommand)]` (`cmd/seed_xor.rs:28`, `seedqr.rs:41`, `slip39.rs:68`, `ms_shares.rs:30`, `xpub_search/mod.rs:55`); no grandchildren. GUI mirror = 32 subs (20 flat + 12 nested, `schema/mnemonic.rs:4389-4658`). NO flat name begins with any `<parent>-` prefix (checked all 20). No cross-match (`seed-xor-*` vs `seedqr-*` diverge at char 4). `strip_prefix(p).and_then(strip_prefix('-'))` correct incl. the exact-parent edge (→passthrough). Flatten `gui_schema.rs:1017`.
- **#2 Fix location + mask.** Single push `invocation.rs:161-162`; sole assembler `assemble_argv_with_secret_mask:152` called once (`app_window.rs:898`) feeding preview + both copy flavors + both Run legs (`:1038`/`:1094`). Mask parallel. No prod code assumes argv[1]=subcommand (only FLAT-sub tests). Index-sensitive mask tests use token-lookup → survive.
- **#4 F6 PREPEND.** Restore APPENDED `""` (census `restore_template_none.rs:265-311` scoped to restore/bundle — won't trip on `_INFER`). `emit_one` guards `!v.is_empty()`; materialization chain confirmed. `NETWORKS`/`XPUB_SEARCH_ADDRESS_TYPES` match the `_INFER` variants. Widget selection value-keyed → no index breakage.
- **#5 Surgical scope RIGHT.** Sibling xpub-search modes' `--network` = `default_value:Some("mainnet")` (suppressed) + no `--address-type` dropdown; `import-wallet --network` loudly refuses cross-class (exit 1); `convert` agree-checked (F3); `bundle`/`verify-bundle --network` clap-REQUIRED (visible). Full 33-flag census → NO second silent inference-override.
- **#7 Release/CI.** `schema_mirror` names-only (`tests/schema_mirror.rs:90-120`) → F6 choices change invisible. GUI-only, no pin bump, MINOR v0.58.0. Tutorial gates unaffected.

## IMPORTANT (fold)
- **I-1 — F5 test-update list incomplete.** 4 more flattened-token argv assertions FAIL post-fix: `tests/widget_interaction.rs:287` (slip39-split), `:335` (slip39-combine), `:381` (seed-xor-split), `:432` (seed-xor-combine). Complete set (all other refs are lookup keys/kebab — unaffected). Fold: add to §4.
- **I-2 — F6 reds the required `snapshots` CI gate.** `tests/snapshots/forms/mnemonic-xpub-search-address-of-xpub.png` renders the VIRGIN form; F6 flips both dropdowns' text `p2pkh`/`mainnet`→`(none)` → PNG diverges → the required `snapshots` context fails until regenerated (`UPDATE_SNAPSHOTS=1 cargo test --test gui_form_snapshots`) + committed. F5 churns NO PNG (preview/action-bar is bin-crate-only). Fold: add the single-PNG regen to §4/§5.
- **I-3 (DESIGN DECISION) — persisted stale materialized values survive the consts-only fix.** Autosave persists per-subcommand non-secret `values` incl. Dropdowns + rehydrates (`persistence.rs:79-120,:342`). Any user who EVER opened `address-of-xpub` on ≤v0.57.0 has `("--address-type",Dropdown("p2pkh"))` + `("--network",Dropdown("mainnet"))` persisted → the consts fix leaves those EMITTING → the funds false-negative PERSISTS post-upgrade for exactly the at-risk users. The existing load-time hook `normalize_loaded_form_values` (`persistence.rs:342-401`, hint-text precedent) is Text/Path-scoped + default_value-keyed → does NOT cover this. Fold: SPEC MUST decide — **(a) extend the load-time normalization to DROP the two flags' persisted values when equal to the old materialized `opts[0]`** (`p2pkh`/`mainnet`) → reset to inference (fail-safe + re-selectable; can't distinguish a deliberate `p2pkh` but reset-to-inference is the safe direction — RECOMMENDED), or (b) explicitly accept + document the carryover. Silence is not an option in a funds-safety SPEC.

## MINOR
- **M-1 — #13 caveat incomplete.** Beyond `""`: `--separator` is GUI `Dropdown(["space","hyphen","comma"])` over a toolkit `text` kind (JSON `choices:null`) → naive choices-equality reds even after stripping `""`. Scope the comparison to flags whose pinned JSON carries NON-NULL choices (or allowlist). The "if it balloons → FOLLOWUP" hatch keeps this Minor.
- **M-2 — code nits.** `vec![p, child]` mixes `&&str`/`&str` → needs `*p`; the assembler doc-comment invariant "argv[1] = subcommand.name" (`invocation.rs:116`) must update in the same PR.
- **M-3 — G6 FOLLOWUP wording.** Name `verify-bundle --template` (materialized `bip44` vs md1-carried type → false verify-FAIL; fail-SAFE, keyless-template dispatches first `verify_bundle.rs:379-400`) as the top-priority item in `gui-dropdown-none-opts0-materialization-audit`.

**Recon cross-check:** all load-bearing recon facts re-verified accurate. (Recon's "~20 Dropdown+None" undercounts — census=33 — immaterial to scope.)

**Fold I-1/I-2/I-3 (+ minors), re-dispatch for convergence.**
