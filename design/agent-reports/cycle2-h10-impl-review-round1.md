# cycle-2 WS-B / H10 — per-phase implementation review (round 1)

**Reviewer:** opus adversarial execution review (post-implementation, whole-diff).
**Scope:** refuse unsorted `multi(...)` export to the field-less electrum / coldcard(-multisig) / jade formats (pure refusal).
**Worktree:** `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/cycle2-h10`, branch `fix/cycle2-h10`, commit `29b39723`.
**Baseline:** toolkit `origin/master` = `f9467cc5` (0.61.0). Diff = `git diff origin/master...HEAD`.
**Design:** `design/IMPLEMENTATION_PLAN_cycle2_funds_loss_fixes.md` §2 (WS-B/H10) + `design/BRAINSTORM_cycle2_funds_loss_fixes.md`.

---

## VERDICT: GREEN — 0 Critical / 0 Important.

Implementation matches the R0-GREEN plan §2 exactly. The guard is a structured check on the resolved typed `CliTemplate`, placed at the single shared `emit_payload` chokepoint after `collect_missing` and before the per-format `emit` match. It refuses exactly `{WshMulti, ShWshMulti} × {Electrum, Coldcard, ColdcardMultisig, Jade}` and nothing else. No over-refuse, no under-refuse, restore-path coverage is a funds-safe free consequence, and the pre-existing by-design-coercion test was correctly flipped to assert the refusal. Full suite GREEN, clippy clean, no out-of-scope churn.

---

## Critical

None.

---

## Important

None.

---

## Over-refuse / under-refuse check

**Predicate (export_wallet.rs:131-141):**
```
matches!(inputs.template, Some(CliTemplate::WshMulti | CliTemplate::ShWshMulti))
  && matches!(format, Electrum | Coldcard | ColdcardMultisig | Jade)
  → Err(ExportWalletUnsortedMultisigUnsupported { format: format_name })
```

**Over-refuse — NONE (verified).** `CliTemplate` (template.rs:16-42, master) has EXACTLY 10 variants: `Bip44/Bip49/Bip84/Bip86`, `WshMulti`, `WshSortedMulti`, `ShWshMulti`, `ShWshSortedMulti`, `TrMultiA`, `TrSortedMultiA`. The two named in the guard (`WshMulti`, `ShWshMulti`) are precisely the unsorted-segwit-multi variants; NO sorted variant, no bare `Multi`/general variant, no taproot variant is in the set. Confirmed the guard NEVER fires for:
- `WshSortedMulti`/`ShWshSortedMulti` — unit `sorted_multi_not_refused_by_h10_guard` + integration `sorted_multi_template_still_exports_to_fieldless_vendors`/`from_import_json_sorted_wsh_multi_still_exports` (exit 0). RED-discriminator confirmed on master: sorted export already succeeds and still does.
- `TrMultiA`/`TrSortedMultiA` (taproot) — disjoint variant set; passes the H10 guard untouched and hits the EXISTING per-emitter taproot refusal. Unit `taproot_multi_hits_existing_taproot_guard_not_h10` + integration `taproot_multi_a_hits_existing_taproot_refusal_not_h10` (asserts the stderr does NOT contain the H10 wording).
- single-sig `Bip44/49/84/86` — unit `single_sig_not_refused_by_h10_guard` + integration `single_sig_to_fieldless_vendor_still_exports` (exit 0).
- faithful formats `Descriptor`/`Sparrow`/`BitcoinCore` (also `Bip388`/`Bsms`/`Green`/`Specter`) — excluded from the format set; unit `faithful_formats_not_refused_for_unsorted_multi` + integration `faithful_formats_still_export_unsorted_multi` (descriptor emits the literal `multi(`). Note: `Sparrow` is in `format_requires_template` but is deliberately NOT in the guard's format set, so `--from-import-json --format sparrow` of an unsorted multi (which `template_from_descriptor` resolves to `Some(WshMulti)`) is correctly allowed to pass through faithfully.

**Under-refuse — NONE (all three entry routes verified against master control-flow):**
- **`--template wsh-multi`/`sh-wsh-multi` (`run`):** `resolved_template = Some((…, template, k))` (export_wallet.rs:542 region) → `template_opt = Some(WshMulti)` → `inputs.template = Some(WshMulti)`. Guard fires (typed exit 2). ✓ — integration `template_wsh_multi_refused_…` / `template_sh_wsh_multi_refused_…`.
- **`--from-import-json` unsorted (`run_from_import_json`):** general-policy refused first (`descriptor_is_general_policy`, export_wallet.rs:798), taproot refused upstream, then `template_from_descriptor` (mod.rs:259-292) computes `is_sorted = to_string().contains("sortedmulti(")` and maps `Wsh(_) → WshMulti` / `Sh(Wsh) → ShWshMulti` for the unsorted case → `Some(WshMulti)`. Guard fires (typed exit 2). ✓ — integration `from_import_json_unsorted_wsh_multi_refused_exit2` / `…sh_wsh_multi…` + unit `template_from_descriptor_preserves_unsorted_distinction` (asserts the live `Wsh(multi)→WshMulti`, `Wsh(sortedmulti)→WshSortedMulti` mapping). The `sortedmulti(`-as-substring false-match a naive `.contains("multi(")` would hit is structurally avoided (the guard reads the typed enum, not the string).
- **Direct `--descriptor 'wsh(multi(…))'` (`run`):** `@N` form rejected at the descriptor branch; a concrete inline-key descriptor never assigns `resolved_template` → `template_opt = None` → guard does NOT fire (and need not). The field-less emitter's own generic `BadInput`/`collect_missing` refuses it. REFUSED, never silently coerced. ✓ — integration `direct_descriptor_unsorted_multi_refused_not_silently_coerced` (asserts failure) + unit `template_none_falls_through_to_generic_badinput_not_h10` (asserts the kind is NOT the typed H10). This is the §8.1 optional FOLLOWUP (typed-upgrade), deliberately out of cycle-2 scope and funds-safe today.

The guard sits AFTER `collect_missing` and BEFORE the `match format` (export_wallet.rs:139). For all four field-less vendors `collect_missing` is empty for a fully-slotted multisig, so the missing-fields short-circuit does not pre-empt the guard. Placement correct.

**RED-discriminator proof (built the `f9467cc5` master binary and ran the offending case):** on master, `export-wallet --format coldcard-multisig --template wsh-multi …` returns **exit 0** and emits a coldcard-multisig file (the silent-sortedmulti-coercion bug — the importer interprets it as sortedmulti, giving a different witnessScript/address). On the fix branch the identical invocation returns **exit 2** with the typed message. The tests are genuine fix-vs-bug discriminators, not made-to-pass. The sorted case returns exit 0 on BOTH branches, confirming the false-refuse guard tests are non-vacuous.

---

## Restore-path interaction verdict — SAFE.

`emit_payload` has 5 call sites across 4 logical entry contexts (`run`, `run_from_import_json`, restore `build_import_payload`, restore `build_multisig_import_payload`). The guard at the chokepoint now ALSO fires on the restore/import builders.

- **No legitimate flow wrongly refused.** An unsorted `multi(...)` → a field-less BIP-67-sortedmulti-only vendor is ALWAYS wrong (it silently reorders keys → different address), so there is no legitimate restore/import scenario that should export an unsorted-multi to electrum/coldcard/jade. The added restore refusals are strictly funds-safe (more refusals, never fewer). Restore of a SORTED md1 STILL emits — integration `restore_md1_sorted_multi_to_fieldless_vendor_still_emits` (exit 0), so the restore-path guard does NOT over-refuse either.
- **`restore.rs` is NOT edited** — confirmed: 0 occurrences of `restore.rs` in the diff (`git diff --name-only` = `error.rs`, `cmd/export_wallet.rs`, `tests/cli_export_wallet_unsorted_multi_refusal.rs`, `tests/cli_wallet_cross_format_convergence.rs`). The restore multisig builder takes `template: Option<CliTemplate>` and forwards it to `emit_payload`; the coverage is a genuine free consequence of the shared chokepoint, file-disjointness holds.
- **The "restore coverage = extra funds-safe refusals" claim is real, not hand-waving** — pinned by integration `restore_md1_unsorted_multi_to_fieldless_vendor_refused` (builds the md1 via `bundle --template wsh-multi`, restores it to each field-less vendor, asserts exit 2) and the sorted counterpart asserting exit 0.

---

## Per-checklist confirmations (review item 4 & 5)

**Item 4 — rewritten `c4_unsorted_multi_order_preservation` (cli_wallet_cross_format_convergence.rs):** the previous coldcard-multisig probe RECORDED the silent sortedmulti coercion as "expected by-design" (asserted the key-set round-trips, logged the tag). The rewrite builds a real `export-wallet --format coldcard-multisig --template wsh-multi` CLI invocation, asserts `!ok` (exit ≠ 0) AND `stderr.contains("UNSORTED multisig") && stderr.contains("sortedmulti-only")`. This is a correct lockstep update (the old behavior is now a refusal), NOT a made-to-pass hack. The faithful-format anchor is PRESERVED unchanged: the `order_preserving = [bitcoin-core, bsms, sparrow, specter]` block still asserts all four converge on the UNSORTED "Multi" tag and preserve declaration order (anchor `md1_multi_tag == Some("Multi")`, `ordered_xpubs` equality). Only the coldcard-multisig probe hunk changed.

**Item 5 — new variant + arms + tests + scope:**
- New variant `ExportWalletUnsortedMultisigUnsupported { format: &'static str }` — **struct form** (per the plan's pinned choice, §2.1). Alphabetical: sits after `ExportWalletTaprootMultisigUnsupported` (T < U) and before `FutureFormat` (E < F) in the enum (error.rs:177) AND in all three exhaustive `match self` arms: `exit_code` → `2` (error.rs:554), `kind` → `"ExportWalletUnsortedMultisigUnsupported"` (error.rs:619-620), `message` (error.rs:764). All placements verified.
- `exit_code` = 2, mirroring the taproot/every-export-refusal precedent. ✓
- The two exhaustive-style error tests (`exit_code_table_per_variant` :980, `kind_strings_stable` :1287) are SAMPLING tables, not exhaustive enumerations, so they correctly do not require a new row; the exhaustive `match` arms enforce completeness (plan §2.1 anticipated this).
- Message wording (§2.4) is byte-acceptable: names the offending format and points to faithful formats (`descriptor` / `bitcoin-core` / `sparrow`) for recovery (anti-dead-end). Inlined in the `message()` arm (implementer's-choice per §2.4). Pinned by unit `unsorted_multi_refusal_message_points_to_faithful_format` (`descriptor` + `electrum`) and integration stderr asserts.
- Tests are real RED→GREEN discriminators (RED-proof above). Count matches plan: **8 unit** (in-crate `h10_unsorted_multi_refusal_tests` module) + **12 integration** (`cli_export_wallet_unsorted_multi_refusal.rs`) + the rewritten c4. All pass.
- **Scope:** only `error.rs` + `cmd/export_wallet.rs` + the 2 test files. No `Cargo.toml`/`Cargo.lock`/`fuzz`/`mlock`/README/`schema`/manual churn (grep-confirmed empty) — correct for a pure refusal with no new clap flag (no GUI schema-mirror leg, no manual flag-table leg per plan §8/§9).
- **Full suite GREEN** (`cargo test -p mnemonic-toolkit`: 60 `test result: ok`, 0 FAILED, exit 0). **Clippy clean** (`cargo clippy -p mnemonic-toolkit --tests`: no warnings/errors).

---

## Minor

- (informational) The integration `FIELDLESS = ["electrum", "coldcard-multisig", "jade"]` omits the generic `coldcard` single-sig alias from the CLI tests, while the unit-module `FIELDLESS` includes all 4 (incl. `Coldcard`). This is consistent with plan §2.2 / Open-Q1: gating `Coldcard` into the guard is harmless future-proofing, and the generic `coldcard` alias cannot legitimately carry a multisig descriptor, so it is not separately CLI-exercised. The unit test covers the `Coldcard` guard arm directly. No action needed.
- (informational) The doc-comment on the new enum variant says `"electrum" / "coldcard" / "coldcard-multisig" / "jade"` for the `format` payload; in practice `format_name` for the generic-coldcard path is `"coldcard"`, which is correct. No action.

---

_Review only — no code written, no source edited. Persisted before any fold. VERDICT: GREEN (0C/0I)._
