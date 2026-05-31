# SPEC R3 review (re-dispatch after R2 fold) — descriptor-form symmetry (A1)

**Date:** 2026-05-31 · **Reviewer:** opus architect · **Repo SHA:** `ea8ba88` · **Target:** `design/SPEC_descriptor_form_symmetry.md`
**Verdict: GREEN (0C/0I/2m).** SPEC R0 gate SATISFIED — proceed to the implementation plan-doc (own R0 gate).

> Persisted verbatim. This closes the SPEC reviewer-loop (R0 RED 2C/3I/4M → R1 RED 1C/1I/3M → R2 RED 0C/2I/3M → R3 GREEN).

## Pass 1 — R2 folds correct
- **I1 (export-wallet `@N`-probe-only):** §3.4 (`SPEC:114`) guards with `AT_N_PROBE.is_match(desc)` → redirect, else `MsDescriptor::from_str` passthrough; explicitly does NOT call `classify_descriptor_form`. `export_wallet.rs:328-334` confirmed a bare passthrough (no origin requirement) → origin-less-concrete regression fully removed. §3.5 + §6 Phase 4 consistent. ✓
- **I2 (bundle early fork reads body):** §3.4 (`SPEC:111`) materializes `body` via `match (&args.descriptor, &args.descriptor_file)` — byte-identical to `bundle.rs:1056-1064`; double-read on `@N --descriptor-file` harmless; `bundle_run_concrete_descriptor(body)` consistent across §3.4/§3.2/§6. verify-bundle `:614` fork confirmed post-read (`verify_bundle.rs:603-612`). ✓
- **M1/M2/M3:** `AT_N_PROBE` framed NEW everywhere; BundleMode reuses `bundle.rs:1664-1670` verbatim; no synthetic `emit_args.descriptor` injection (`:1680-1681`). ✓

## Pass 2 — whole-spec coherence
- §1–§9 end-to-end: no contradiction, no symbol-before-definition, no orphaned R0/R1/R2 claim. `DescriptorForm { AtN, Concrete }` exactly two variants everywhere; "plain" removed. ✓
- Phase plan §6 ordering correct, each phase independently testable; Test 9 (distinctness) correctly under Phase 3. ✓
- SemVer §4: `synthesize_descriptor` (`bundle.rs:1424/1659`) + `emit_unified` shared by `@N` and Concrete paths → Concrete normalizes to identical `md_codec::Descriptor`+`ResolvedSlot` → `--json` byte-identical. No flag/value added → `schema_mirror`/`cli-subcommands.list` untouched. PATCH + no-GUI-lockstep + manual-prose-only correct. ✓
- No unaddressed implementer edge: the lossy-`[u8;65]` → re-recover-`(xpub,fp,path)`-from-base58 mechanic fully specified (§3.2 step 3-4); checksum-strip ordering, error-prefix remap, mixed-form non-false-fire (`pipeline.rs:155-156`), empty-slot-gate bypass (`bundle.rs:313`) all captured. ✓

## Minor (non-blocking, documentation cite-drift)
- **m1** — §3.2 cited `bsms.rs:256` for the `debug_assert_eq!`; actual is `bsms.rs:254`. FIXED in fold.
- **m2** — `bundle.rs:1664-1670`/`:1680-1681` cites exact at HEAD; decay on merge (plan-doc re-grep discipline covers it). No action.

## Verdict
**R3 GREEN (0C/0I).** SPEC approved. Both R2 Importants folded correctly + completely; all Minors folded; R0/R1 folds remain sound and unbroken. Proceed to the implementation plan-doc — which gets its own R0 gate (re-grep citations against current `origin/master` at plan-write time).
