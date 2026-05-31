# SPEC R2 review (re-dispatch after R1 fold) — descriptor-form symmetry (A1)

**Date:** 2026-05-31 · **Reviewer:** opus architect · **Repo SHA:** `ea8ba88` · **Target:** `design/SPEC_descriptor_form_symmetry.md`
**Verdict: RED (0C/2I/3M).** Fold + re-dispatch R3.

> Persisted verbatim before fold-and-commit.

## Pass 1 — R1 folds landed clean
- **C-A** (early-fork): `bundle.rs:223` is the `--import-json` `run()` fork ✓; `descriptor_mode` at `:227` ✓; `:338` confirmed unreachable for no-slot concrete (empty-slot gate at `:313` → `bundle_unified.rs:35-40`) ✓; `@N` path unchanged ✓. (See I2 re: body symbol.)
- **I-A** (distinctness): `check_resolved_slots_distinctness(&[ResolvedSlot])` at `bundle.rs:402` returns `ToolkitError::Bip388Distinctness{i,j}` (`:408`) — the variant Test 9 names; key `(xpub.to_string(), path)`, no entropy dependence → correct on watch-only ✓. Template runs it `:367`, `@N` runs `check_key_vector_distinctness` `:1407`, from_import_json omits it ✓.
- **M-i/M-ii/M-iii**: classifier in `pipeline.rs` reaches private `key_regex()` (`:35`) ✓; added `use`s in-crate (sibling `bsms.rs`) ✓; `descriptor_body_no_csum` (`json_envelope.rs:448`, `pub(crate)`) strips csum ✓.

## Critical
None.

## Important
**I1 — export-wallet classifier guard regresses origin-less concrete descriptors.** §3.4 routes export-wallet `--descriptor` through `classify_descriptor_form` with "`Concrete`/plain → passthrough" — but the enum has no `plain` variant and rule 4 returns `Err` (origin-required) for origin-less/keyless input. export-wallet today accepts origin-less concrete (`export_wallet.rs:328-334` = bare `MsDescriptor::from_str(desc)?.to_string()`; miniscript accepts `wpkh(xpub…/0/*)`). So `export-wallet --descriptor "wpkh(xpub…/0/*)"` works at `ea8ba88` but would be REJECTED under the guard — regression contradicting §2. *Fix:* export-wallet branches on the **`@N` probe alone** (redirect iff `AT_N_PROBE` matches); everything else → existing passthrough. Do NOT call the rule-4-bearing `classify_descriptor_form` (only bundle/verify-bundle, whose md1/BIP-388 backend needs origins, surface rule 4).

**I2 — bundle Concrete fork pinned where the descriptor body doesn't yet exist.** §3.4/§6 fork at `bundle.rs:227` calls `classify_descriptor_form(&body)?` but no `body` is in scope for `--descriptor-file` (read only at `:1058` inside `bundle_run_unified_descriptor`, and `:804` for emit). `args.descriptor` is `Some(s)` for the flag case, but `--descriptor-file` must be read by the early fork (mirroring `:1056-1064`). verify-bundle's `:614` fork is post-read (`descriptor_str` at `:603-612`) — only the bundle side has the gap. *Fix:* §3.4/§6 state the bundle early fork materializes `body` from `args.descriptor` OR `fs::read_to_string(args.descriptor_file)` (trim_end) before classifying; `bundle_run_concrete_descriptor` (or a tiny pre-helper) owns the read.

## Minor
- **M1** — `AT_N_PROBE` is described as pre-existing ("reaches … directly"); it does NOT exist in-tree — it's NEW this feature. Reword "introduce `AT_N_PROBE = @\d` in `pipeline.rs`".
- **M2** — `BundleMode` selection: reuse the `bundle_run_from_import_json` `match (n, any_secret, any_watch)` selector (`bundle.rs:1664-1670`) verbatim (collapses to the 2 watch-only arms since `any_secret` is always false) rather than hand-rolling a 2-arm picker that could drift.
- **M3** — emit side: the Concrete path has the real descriptor in `args.descriptor`/`args.descriptor_file`, so `emit_unified`'s `descriptor_field` (`:802-808`) picks it up — do NOT copy `from_import_json`'s synthetic `emit_args.descriptor` injection (`:1680-1681`) or it double-sets.

## Controller fold plan (2026-05-31)
I1 → export-wallet uses an `is_at_n_form`/`AT_N_PROBE` check, not `classify_descriptor_form`; §3.4/§3.5 reworded. I2 → bundle early fork reads `--descriptor-file` itself before classifying; §3.4/§6 updated. M1 → "introduce AT_N_PROBE". M2 → reuse the `:1664-1670` match. M3 → note no synthetic descriptor injection. C-A/I-A/M-i/M-ii/M-iii confirmed sound — untouched.
