# Plan-doc R1 review (re-dispatch after plan-R0 fold) — descriptor-form symmetry (A1)

**Date:** 2026-05-31 · **Reviewer:** opus architect · **Repo SHA:** `ea8ba88` · **Target:** `design/IMPLEMENTATION_PLAN_descriptor_form_symmetry.md`
**Verdict: RED (1C/0I/2m).** One-token fold-residue; fix + confirmation pass.

## Pass 1 — R0 folds verified GREEN against source
- **C2** (JSON wire-shape): `BundleJson` real shape confirmed (`format.rs:120-145`: `mode:"watch-only"`, `md1:Vec<String>`, `mk1:MkField` array-of-arrays `format.rs:66-70`, `ms1:MsField` ""-sentinels). P3b mirror-pattern grounded in the real reusable helper at `cli_verify_bundle_multi_cosigner_mk1.rs:114-161`. ✓
- **I1** (imports): `pipeline.rs:17/18/19/21` confirm `FromStr`/`normalize_xpub_prefix`/`Xpub` pre-exist; reduced import set exact. ✓
- **I2** (`verify_emit_from_expected`): `SuppliedCards` (`verify_bundle.rs:1114-1118`), `emit_verify_checks` 7-arg (`:1131-1139`), `VerifyBundleJson` (`format.rs:149-152`), `VerifyCheck.{name,passed,detail}`, `descriptor.n>1`, the text loop — all match `:867+:871-899`. `@N`-path re-point safe. ✓
- **I3** (fork after guards): all five mode-violation guards `:235-284` precede the `bundle_run_unified` dispatch `:286`; fork inserts at `:285`; no guard bypassed. ✓
- **I4** (version sites): `readme_version_current.rs:24-27` requires BOTH READMEs (`README.md:13` + `crates/.../README.md:9`) + Cargo.toml:3. 4-site list correct. ✓
- Other symbols verified real: `descriptor_body_no_csum` (returns whole string on no-`#`), `check_resolved_slots_distinctness` (private, same-module), `ResolvedSlot` 6 fields, `BundleMode` 5 variants, `emit_unified`/`self_check_bundle`, `Bip388VerifyDistinctness`/`Bip388Distinctness` (exit 4/2). ✓

## Critical
**C1 — the C1 fold half-landed: the dispatch call site still passes `&&BundleArgs` (E0308).** `run`'s param is `args: &BundleArgs` (`bundle.rs:170-171`). The fold fixed the fn signature (`fn bundle_run_concrete_descriptor(args: &BundleArgs, …)`) and the interior calls, but the early-fork call site read `return bundle_run_concrete_descriptor(&args, …)` — `&args` is `&&BundleArgs` vs the `&BundleArgs` param. *Fix:* drop the `&` → `bundle_run_concrete_descriptor(args, body, stdout, stderr)` (mirrors `bundle_run_from_import_json(args, …)` at `bundle.rs:224`). **[FOLDED — call site now `args`, comment added.]**

## Minor
- **m1** — guard span cited `:235-279`; last guard ends `:284`, dispatch at `:286`. Prose anchor ("immediately before the `bundle_run_unified(...)` dispatch") is unambiguous. **[FOLDED — `:235-284` + `:286`.]**
- **m2** — plan-R0 header's "exit 1" aside for `DescriptorParse` is imprecise (it's exit 2; `BadInput`=1). No plan assertion checks a specific exit code on these paths (all assert `!success`). No impact; noted so the end-of-cycle reviewer isn't misled.

## Verdict
**RED (1C/0I)** — single one-token compile fix at the dispatch call site, now folded. Every other fold verified GREEN against source. Architect noted: given the fix is mechanical/isolated, R2 may be a confirmation-only pass. → controller folds + lightweight R2 confirmation.
