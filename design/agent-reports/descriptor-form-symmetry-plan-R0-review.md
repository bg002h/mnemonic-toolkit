# Plan-doc R0 review — descriptor-form symmetry (A1)

**Date:** 2026-05-31 · **Reviewer:** opus architect · **Repo SHA:** `ea8ba88` · **Target:** `design/IMPLEMENTATION_PLAN_descriptor_form_symmetry.md`
**Verdict: RED (2C/4I/3m).** Fold + re-dispatch.

> Persisted verbatim before fold-and-commit. SPEC mapping confirmed complete; defects are in the pasted code/tests.

## Verified real (no change): `ToolkitError::DescriptorParse(String)`/`BadInput(String)` exit 1/`Bip388VerifyDistinctness` exit 4/`Bip388Distinctness{i,j}` exit 2/`message()`; `xpub_to_65(&Xpub)->[u8;65]` (synthesize.rs:98); `ResolvedSlot` 6-field set (synthesize.rs:616-643, `entropy: Option<Zeroizing<Vec<u8>>>`, `_entropy_pin: Option<Rc<PinnedPageRange>>`); `parse_descriptor(&str,&[ParsedKey],&[ParsedFingerprint])->Result<md_codec::Descriptor>`; `normalize_xpub_prefix(&str)->Result<(String,Option<&'static str>)>`; `CosignerKeyInfo = ResolvedSlot`; `descriptor_body_no_csum(&'a str,&str)->Result<&'a str>`; `emit_unified(&BundleArgs,&Bundle,&[ResolvedSlot],BundleMode,&[(u8,&'static str)],W,E)`; `self_check_bundle(&Bundle,&BundleArgs)`. P0 regex widening + h-form preservation + two-pass ordering all GREEN.

## Critical
**C1 — `bundle_run_concrete_descriptor` by-value/by-ref mismatch won't compile.** `run`'s `args` is `&BundleArgs`; plan declares `fn (args: BundleArgs, …)` and calls it passing `&BundleArgs`. *Fix:* `args: &BundleArgs` (mirror `bundle_run_from_import_json`, bundle.rs:1491); call `emit_unified(args, …)` + `self_check_bundle(&bundle, args)` (drop the extra `&` — `&args` would be `&&BundleArgs`).

**C2 — pasted `--json` test assertions are wrong against the real wire-shape; P3b panics.** `BundleJson` (format.rs:119-145): `md1: Vec<String>` (array, not string), `mk1: MkField` (multisig = array-of-arrays), `ms1: MsField` (length-N array, `""` watch-only sentinel — present, not null), `mode: &str` ("full"|"watch-only"). So `v["md1"].as_str()` is always `None` (P3 assert fails; P3b `.unwrap()` panics), `v["mk1"][0].as_str().unwrap()` panics, the ms1-null check fails. *Fix:* assert `v["mode"]=="watch-only"` + `v["md1"].as_array().unwrap()` non-empty + `v["ms1"].as_array().unwrap().iter().all(|s| s=="")`; for P3b's card round-trip, MIRROR an existing verify-bundle test's produce→extract→verify pattern (`tests/cli_verify_bundle_full.rs`) instead of hand-navigating JSON. P5's `assert_eq!(cv["md1"],av["md1"])` Value-equality is correct.

## Important
**I1 — duplicate imports → E0252/E0254.** `pipeline.rs` already has `use std::str::FromStr;` (:21), `use bitcoin::bip32::Xpub;` (:19), `use crate::slip0132::normalize_xpub_prefix;` (:18). *Fix:* add only `use bitcoin::bip32::{DerivationPath, Fingerprint};` + `use crate::synthesize::{ResolvedSlot, xpub_to_65};` + `use crate::parse_descriptor::parse_descriptor;`.

**I2 — verify-bundle "synthesize+compare tail" extraction under-specified.** `verify_bundle.rs:856-866` is `@N`-specific (re-parses the `@N` form — FAILS on concrete; mutates `path_decl` from `descriptor_resolved` built only on the `@N` path). The Concrete path owns its parsed `descriptor` (from §3.2 helper) and must SKIP :856 re-parse + :864-866 path_decl propagation. The genuinely reusable tail is :867 (`synthesize_descriptor`) + :871-902 (`SuppliedCards`+`emit_verify_checks`+output+`Ok(if any_fail{4}else{0})`). The plan's call drops `no_auto_repair` (required by `emit_verify_checks`, :876) and must return `Result<u8>`. *Fix:* specify the extracted helper `fn verify_emit_from_expected(&VerifyBundleArgs, descriptor: md_codec::Descriptor, cosigners: &[ResolvedSlot], no_auto_repair: bool, json_context: bool, W, E) -> Result<u8, ToolkitError>` = :867 + :871-902; Concrete path skips path_decl mutation (explicit-origin → no-op; state it); thread `no_auto_repair`. The `@N` path calls the same helper after its parse+path_decl.

**I3 — Concrete early-fork bypasses the descriptor-mode mutex guards.** `--descriptor` has no clap `conflicts_with` for `--template`; the mutexes are code-level at bundle.rs:235-279 (DESCRIPTOR_AND_TEMPLATE :236, etc.), AFTER the plan's `:227` insertion. Concrete `--descriptor --template` would `return bundle_run_concrete_descriptor` and silently ignore `--template` (vs the `@N` path which errors). *Fix:* place the Concrete fork AFTER the mode-violation guards (after :279, before the `bundle_run_unified(...)` dispatch), or replicate the 3 guards in `bundle_run_concrete_descriptor`.

**I4 — version-bump site list incomplete → `readme_version_current.rs` reds the suite.** The test (`tests/readme_version_current.rs:27`) requires `<!-- toolkit-version: X -->` in BOTH `crates/mnemonic-toolkit/README.md` (:9) AND repo-root `README.md`. Plan's grep scopes only repo-root README, omits `crates/mnemonic-toolkit/README.md` + `CHANGELOG.md`. *Fix:* update both READMEs + Cargo.toml + CHANGELOG; widen the grep to all four.

## Minor
- **m1** — `--test cli_verify_bundle` is non-existent; real targets are `cli_verify_bundle_{full,multi_cosigner_mk1,watch_only,forensics,seedqr_slot}`. Use `cli_verify_bundle_full` or `ls tests | grep verify_bundle` first.
- **m2** — `--test cli_import_wallet` non-existent (plan already hedges with `ls … grep import`).
- **m3** — `debug_assert_eq!(xpub_to_65(&xpub), keys[idx].payload)` verified (types + ordering sound). No change.

## Controller fold plan (2026-05-31)
C1 → `args: &BundleArgs`. C2 → fix assertions (mode/md1-array/ms1-empty) + P3b mirror cli_verify_bundle_full. I1 → 3 new imports only. I2 → `verify_emit_from_expected` helper spec (:867+:871-902, skip @N path_decl, thread no_auto_repair, Result<u8>). I3 → fork after :279 guards. I4 → both READMEs+Cargo+CHANGELOG. m1/m2 → real test targets.
