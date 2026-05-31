# SPEC R1 review (re-dispatch after R0 fold) — descriptor-form symmetry (A1)

**Date:** 2026-05-31 · **Reviewer:** opus architect · **Repo SHA:** `ea8ba88` · **Target:** `design/SPEC_descriptor_form_symmetry.md`
**Verdict: RED (1C/1I/3M).** Fold + re-dispatch R2.

> Persisted verbatim before fold-and-commit.

## Pass 1 — R0 folds confirmed sound
- **C1** (re-cite + recovery mechanic): `key_regex` (`pipeline.rs:38`) has exactly group1=fp/group2=path/group3=xpub ✓; `ResolvedSlot` field set `{xpub,fingerprint,path,entropy,master_xpub,_entropy_pin}` (`synthesize.rs:616-644`) matches the SPEC literal exactly ✓; `xpub_to_65` exists (`synthesize.rs:98`) ✓. **Ordering hazard SAFE:** both `concrete_keys_to_placeholders` (`pipeline.rs:62`) and the §3.2 second pass use `key_regex().captures_iter(body)` over the same body → identical order/count, slot `i` ↔ `@i`/`keys[i]`. `tr(NUMS,…)` and same-xpub-twice both consistent across the two passes.
- **C2** (widen only `key_regex`): widened `((?:/\d+(?:'|h)?)+)` valid + strict superset; helper never calls `extract_origin_components` → the 4 `origin_capture_regex` copies stay untouched and can't break the new path. ✓
- **I2/I3/M1/M2/M3/M4** all folded correctly; verify-bundle fork `:614` is correct (uses `validate_slot_set`, Ok on empty — no empty-slot gate).

## Critical
**C-A — the bundle insertion point `bundle.rs:338` is UNREACHABLE for a no-`--slot` bare-concrete `--descriptor`.** Inside `bundle_run_unified`, `detect_bundle_mode(&slots)` (`bundle.rs:313`) errors on an empty slot set (`bundle_unified.rs:35-40` "no --slot inputs supplied") BEFORE control reaches the line-338 fork. The primary invocation `bundle --descriptor "<concrete>"` carries no `--slot`, so it dies at the gate. `--import-json` deliberately forks EARLIER in `run()` (`bundle.rs:223`: `if args.import_json.is_some() { return bundle_run_from_import_json(...) }`) to bypass this. *Fix:* hoist the Concrete dispatch into `run()` right after the `--import-json` fork (`bundle.rs:223-227`, where `descriptor_mode` is already computed): if `descriptor_mode && classify_descriptor_form(body)? == Concrete` → `return bundle_run_concrete_descriptor(...)`, before `bundle_run_unified`. The `@N` descriptor continues into `bundle_run_unified` unchanged. Re-spec §3.4 + §6 Phase 3 to name `bundle.rs:~223`, not `:338`.

## Important
**I-A — `bundle_run_concrete_descriptor`, mirroring the `from_import_json` tail, silently DROPS the BIP-388 distinctness check both sibling paths enforce.** Template path → `check_resolved_slots_distinctness(&resolved)` (`bundle.rs:367`); `@N` path → `check_key_vector_distinctness` (`bundle.rs:1407`); but `bundle_run_from_import_json`'s tail (`bundle.rs:1659-1690`) runs NO distinctness (its slots come from trusted mk1 cards). A bare-concrete descriptor with two identical `(xpub,path)` cosigners would be accepted by the new path while the equivalent `@N`+`--slot` is rejected — behavioral divergence + dropped invariant. *Fix:* `bundle_run_concrete_descriptor` must call `check_resolved_slots_distinctness(&resolved_slots)` (`bundle.rs:402`, the `&[ResolvedSlot]` variant) before `synthesize_descriptor`; the verify-bundle Concrete arm mirrors it. Add a negative cell (duplicate-key concrete → `Bip388Distinctness`). `validate_watch_only_resolved` is tautological here (`entropy: None` always) — optional/cosmetic.

## Minor
- **M-i** — classifier module unpinned; to reach the private `fn key_regex()` (`pipeline.rs:35`) it must be co-located in `pipeline.rs` (or `key_regex` made `pub(crate)`). Pin it.
- **M-ii** — §3.2 helper's added `use`s unstated: `crate::synthesize::ResolvedSlot`, `md_codec::Descriptor`, `crate::parse_descriptor::parse_descriptor` (all in-crate, already used by sibling `bsms.rs` — no layering violation). Note them.
- **M-iii** — checksum-stripping responsibility unstated: the caller strips via `descriptor_body_no_csum` (`json_envelope.rs:448`) before calling the helper, as `bsms.rs` does. State it.

## Controller fold plan (2026-05-31)
C-A → §3.4/§6 insertion at `run()` ~`bundle.rs:223` (after import-json fork, before bundle_run_unified). I-A → `bundle_run_concrete_descriptor` + verify Concrete arm call `check_resolved_slots_distinctness` before synth; add negative test. M-i → classifier in `pipeline.rs` (pub(crate)). M-ii → note imports. M-iii → caller strips checksum. C1/C2/verify-fork/I2/I3/M1-M4 confirmed sound — untouched.
