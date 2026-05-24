# R0 ARCHITECT REVIEW — `IMPLEMENTATION_PLAN_v0_37_0_from_import_json_template_reemit.md`

**Round:** R0 (plan-doc gate)
**Date:** 2026-05-24
**Reviewer:** feature-dev:code-reviewer (opus)
**Spec basis:** `36e6bfa` (tree at `Cargo.toml` 0.36.4, consistent)
**Verdict:** GREEN (0 Critical / 0 Important) — 4 non-blocking Minors (all folded post-review)

Every load-bearing claim verified against real source; each embedded Rust block checked for compilation against actual types; truth table traced through real emitter routing.

## CRITICAL — None.
## IMPORTANT — None.

## MINOR (all folded into the plan/spec)
- **M1** — `template_from_descriptor` `Bare(_)` arm uses `DescriptorParse` vs spec §2.2 table's `BadInput`. Both valid; arm doubly-unreachable (`script_type_from_descriptor` rejects `Bare` first at `:628`). Folded: plan notes the intentional mirror of `script_type_from_descriptor`.
- **M2** — Task 0.3 retreats from §0/§5.3 byte-equality-vs-direct-`--template` to round-trip-against-source + parse asserts. Verified technically justified: from-import-json rebuilds slots from mk1 cards (`envelope_to_resolved_slots`) vs direct path's `--slot` via `bundle::resolve_slots` — different pipelines, byte-compare could be orthogonally flaky. Folded: reconciliation note added to plan Task 0.3 AND spec §5.3.
- **M3** — module-level `SINGLESIG_SOURCES` (Task 0.2b) shadowed by an identical function-local const at `:896`. Not a compile error. Folded: plan instructs deleting the redundant inner const.
- **M4** — coldcard-multisig recipe is 3 lines (`:564-566`); strip must avoid a dangling `\`. Folded: plan Phase-2 note.

## Verified correct (positives)
- **Task 1.1 compiles:** `CliTemplate`/`MsDescriptor`/`DescriptorPublicKey`/`ToolkitError` in scope (`mod.rs:50-55`); `Sh::as_inner()→&ShInner{Wpkh,Wsh,Ms}` mirrors `script_type_from_descriptor:219-228`; `CliTemplate` derives `PartialEq+Eq+Debug+Copy` (`template.rs:14`); substring `sortedmulti(`-before-`multi(` sound (no taproot reaches path).
- **Task 1.2 compiles:** `CliExportFormat` is `Copy` (`:21`), 10 variants exact (`:23-42`); partition = the `template.ok_or_else`-refusers (sparrow `:104`, coldcard `:111`, jade `:36`, electrum `:52`, coldcard-multisig guard `:730`); passthrough formats correctly `false` (green keys on `script_type` `green.rs:36`; bip388 branches output on `template.is_some()` `:33` → MUST stay None).
- **EmitInputs edits:** `template:None` `:666`, `threshold_user_supplied:false` `:671`, `parsed_ms` `:613`, taproot refusal `:629`, `threshold` `:659`, literal `:661` — all exact. `sparrow.rs:43` sole reader of `threshold_user_supplied`.
- **Truth table 34/6:** singlesig→{sparrow✓,coldcard✓(generic),electrum✓(standard),coldcard-multisig✗(`:730`),jade✗(`jade.rs:56-62`)}; wsh-sortedmulti→all 5✓ (coldcard multisig-text `:52`, jade delegates `:46`, electrum multisig `:71`). 9+25=34 succeed; 3×2=6 refuse.
- **REFUSAL_STDERR_PATTERNS add needed:** "emits multisig wallet config only" (`jade.rs:61`) absent from current set (`:814-822`); "requires a multisig --template" already at `:817`.
- **Cell 3 rewrite sound:** `envelope_v0_27_0.json` IS `sh(multi(2,…))`→P2shMulti; both new assertion substrings appear in the new error.
- **All cited line numbers match** (p11c `:841`, p11a `:611`, Cell 3 `:96`/`:114-117`, REFUSAL `:814`, ALL_SOURCES `:563`, TEMPLATE_ONLY_DESTS `:592`, p11b `:722`, p11c_green `:892`); helper sig `run_export_from_import_envelope(&Path,&str,&str)->ExportResult` `:488` matches all call sites.
- **RED→GREEN holds:** current master refuses every template-only cell (`template:None`) → rewritten p11c 34 success-cells go RED → GREEN post-fix.
- **Release-prep citations exact:** `Cargo.toml:3` 0.36.4, `install.sh:32` self-pin, FOLLOWUPS entry `:3182` (`Status: open`), cli-ref `:669`, all recipe lines, `:577`/`:347` prose, `:352-357` taproot note (left unchanged). Manual flag-coverage lint keys off `cli-subcommands.list`, not recipes.

No spec requirement (§2.x/§5.x) lacks a task; no placeholders remain.

## VERDICT
**GREEN (0C/0I).** The plan would compile and behave correctly as written. The 4 Minors are polish (folded); none gates implementation start.
