# mnemonic-toolkit v0.5.1 — legacy CLI flag deletion + test rewrite

**Status:** CONVERGED (architect r1: 0C/3I/4L → all addressed inline; r2: 0C/1I/3L → all addressed inline; ready for execution).

## Context

v0.5.0 (shipped 2026-05-06) closed 13 of 15 planned FOLLOWUPS. The remaining 2 — `legacy-cli-flag-deletion` and `legacy-flag-deprecation` — were deferred to v0.5.1 per the plan's scope-reduction trigger. They represent ~2500 LOC of mechanical-but-error-prone churn (CLI surface contraction + ~25-test rewrite). v0.5.1 is the focused-rewrite cycle that ships them.

The v0.5.0 SPEC document already specifies the v0.5.1 target state in §6.6 (with a "partial delivery" note that v0.5.1 will remove). v0.5.1 implementation aligns the binary to the SPEC.

Per "no users yet → break anything" license: no compatibility shims, no deprecation aliases. Direct deletion + test rewrite.

## Inventory (re-verified 2026-05-06)

**Legacy CLI flags to delete (6):**
- `--phrase`, `--xpub`, `--master-fingerprint`, `--cosigner`, `--cosigners-file`, `--cosigner-count`.

Source files affected (`crates/mnemonic-toolkit/src/`):
- `cmd/bundle.rs::BundleArgs` (6 fields + clap attrs).
- `cmd/verify_bundle.rs::VerifyBundleArgs` (same 6 fields + clap attrs).
- Alias shims: `expand_legacy_to_slots` (in `slot_input.rs`) and `bundle_args_to_slots` (in `cmd/bundle.rs`).

**Mode-violation guards (per v0.5.0 SPEC §6.6):**
- DELETE 9 guards: `XPUB_AND_COSIGNER`, `COSIGNER_AND_COSIGNERS_FILE`, `XPUB_NEEDS_FINGERPRINT`, `FINGERPRINT_WITHOUT_XPUB`, `XPUB_STDIN`, `PASSPHRASE_WITH_XPUB`, `LANGUAGE_WITH_XPUB`, `PRIVACY_WITH_XPUB`, `COSIGNER_COUNT_WITHOUT_MULTISIG`.
- RETAIN 3 guards (still meaningful): `THRESHOLD_WITHOUT_MULTISIG`, `PATH_FAMILY_WITHOUT_MULTISIG`, `DESCRIPTOR_AND_TEMPLATE`.

**Test file deletions:**
- `cli_mode_violations.rs` (192 lines, 21 legacy-flag references).
- `cli_mode_violations_v0_2.rs` (181 lines, 24 references).
- `cli_mode_violations_v0_3.rs` (211 lines, 16 references).
- Total whole-file delete: ~584 lines + 61 references.

**New test file:**
- `cli_mode_violations_v0_5.rs` covering 3 retained guards under `--slot` invocations. Estimated 6 tests (one happy-path + one violation per guard).

**Test rewrites (13 files, 41 legacy-flag occurrences):**
- `cli_descriptor_mode.rs` (7).
- `cli_bip388_distinctness.rs` (6).
- `cli_verify_bundle_watch_only.rs` (4).
- `cli_bundle_json_intake.rs` (4).
- `cli_verify_bundle_forensics.rs` (3).
- `cli_unified_slot.rs` (3).
- `cli_help_fixtures.rs` (3).
- `cli_bundle_watch_only.rs` (3).
- `cli_self_check.rs` (2).
- `cli_json_envelopes.rs` (2).
- `cli_bundle_multisig.rs` (2).
- `cli_verify_bundle_full.rs` (1).
- `cli_bundle_full.rs` (1).

## Bundle decisions

**Single-cycle ship:** A.1 + A.2 land together. Splitting them would require a transitional state where the source-side flags are deleted but tests still reference them (compile errors). Conversely, keeping flags but rewriting tests buys nothing. Atomic deletion.

**No alias period:** the v0.4.2 deprecation step routed legacy flags through `expand_legacy_to_slots`. v0.5.1 deletes both the flags AND the shim. No staged transition.

**Test-rewrite mechanical approach:** for each legacy invocation pattern, apply the v0.5.0 plan's mapping table:
- `--phrase X` → `--slot @0.phrase=X`.
- `--xpub X --master-fingerprint Y` → `--slot @0.xpub=X --slot @0.fingerprint=Y` (path defaulted from `--template`).
- `--cosigner Xpub:fp:path` → `--slot @1.xpub=Xpub --slot @1.fingerprint=fp --slot @1.path=path` (per-cosigner subkeys).
- `--cosigners-file F` → preprocess F to expand into `--slot @N.xpub=` + `--slot @N.fingerprint=` + `--slot @N.path=` per cosigner.
- `--cosigner-count N` → drop entirely (derived from slot count).

## Cycle naming

**v0.5.1** — patch bump (the v0.5 cycle's second release). Architectural intent shipped in v0.5.0 SPEC; binary alignment shipped in v0.5.1.

---

# SPEC amendments (v0.5.0 → v0.5.1)

Minimal — the v0.5.0 SPEC already specifies the v0.5.1 target state in §6.6. v0.5.1 only removes the "partial delivery" note added to v0.5.0 to acknowledge the deferral.

## §6.6 — partial-delivery note removal

**v0.5.0 text:**
> **v0.5.0 partial delivery:** legacy flag deletion is described in this section but **deferred to v0.5.1** (`legacy-cli-flag-deletion` FOLLOWUP at tier `v0.5.1`). The v0.5.0 binary retains `--phrase`, `--xpub`, `--cosigner`, `--master-fingerprint`, `--cosigner-count`, `--cosigners-file` from `BundleArgs` + `VerifyBundleArgs` plus all 9 mode-violation guards from v0.4. The §6.6 table below reflects the v0.5.1 target state. v0.5.0 ships the SPEC text and the architectural intent; v0.5.1 ships the actual deletion + ~25-test rewrite.

**v0.5.1 amendment:** the partial-delivery paragraph is **DELETED**. The §6.6 table now reflects shipped state.

## §6.6.b Per-slot subkey-set validity matrix — wif/xprv/passphrase notes

The v0.5.0 SPEC mentions `{phrase, passphrase}` as "v0.5+" — confirm this is shipped in v0.5.1 (was already shipped in v0.4.2). No SPEC amendment needed; just verify the v0.5.0 §6.6.b text describes the v0.5.1 reality accurately.

## SPEC v0.5 changelog header

The "v0.4 → v0.5 amendments (delta-only summary)" header at the top of `SPEC_mnemonic_toolkit_v0_5.md` lists 6 amendments. Update bullet 5 to remove the "deferred to v0.5.1" note implicit in the v0.5.0 SPEC header (the partial-delivery acknowledgment lived only in §6.6).

---

# Implementation plan

**Status:** DRAFT (pre-architect-review-r1).

## Phase order

```
A.1a (source-side deletions: BundleArgs + VerifyBundleArgs flag fields, expand_legacy_to_slots shim, bundle_args_to_slots, 9 mode-violation guards)
  ↓
A.1b (whole-file test deletes: cli_mode_violations*.rs ×3)
  ↓
A.1c (new cli_mode_violations_v0_5.rs covering 3 retained guards)
  ↓
A.2 (rewrite ~13 consumer test files to use --slot syntax exclusively)
  ↓
A.3 (SPEC §6.6 partial-delivery note removal)
  ↓
R (release prep + final review + tag + GitHub release)
```

The atomic-deletion constraint (legacy fields + tests both go in the same commit to avoid compile errors) means A.1a + A.1b + A.1c + A.2 land together as a single commit. The phase order above describes the within-commit edit sequence, not separate commits.

## Phase A.1a — source-side deletions

**File:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::BundleArgs` (lines ~28-98 currently).

- Remove fields: `phrase`, `xpub`, `master_fingerprint`, `cosigner`, `cosigners_file`, `cosigner_count`.
- Remove `clap` attribute lines for each.
- Remove the alias-handler bridge code that calls `expand_legacy_to_slots(args)` to fold legacy flags into `args.slot`. The `--slot` vec is now the sole input.

**File:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::VerifyBundleArgs` (lines ~31-115 currently).

Confirmed inventory (architect r1 I-3): all 6 legacy fields are present (`phrase` line 33, `xpub` line 36, `master_fingerprint` line 39, `cosigner` line 94, `cosigners_file` line 98, `cosigner_count` line 114). VerifyBundleArgs has NO `--slot` field — it currently uses legacy-flag-based dispatch exclusively. v0.5.1 adds `pub slot: Vec<SlotInput>` field + reshapes the entire run() dispatch to be slot-based.

Note: there is no `verify_args_to_slots` shim — `expand_legacy_to_slots` is unit-test-only (no production call sites). It can be deleted along with bundle's `bundle_args_to_slots`.

**Mode-violation guard sweep:**

In both `bundle.rs::run` and `verify_bundle.rs::run` pre-check ladders, the 9 deleted guards have their `if`-branches removed. The 3 retained guards are tightened to no longer reference the deleted flag fields:

- `THRESHOLD_WITHOUT_MULTISIG`: `if args.threshold.is_some() && !template.is_multisig()`.
- `PATH_FAMILY_WITHOUT_MULTISIG`: `if args.multisig_path_family.is_some() && !template.is_multisig()`.
- `DESCRIPTOR_AND_TEMPLATE`: unchanged (already references only `args.descriptor` + `args.template`).

**Mode-text constants:**

In `bundle.rs::mode_text` module, the 9 deleted guards' `pub const` strings are removed. The 3 retained constants remain.

**Alias shim deletion:**

- `crates/mnemonic-toolkit/src/slot_input.rs::expand_legacy_to_slots` — function deleted (only unit-test callers; no production sites).
- `crates/mnemonic-toolkit/src/cmd/bundle.rs::bundle_args_to_slots` — function deleted (single call site at bundle.rs:327 updated to use `args.slot.clone()` directly).
- `slot_input.rs` file-level `#![allow(dead_code)]` comment at line 6 references `expand_legacy_to_slots` by name — drop or update the comment in this same pass (architect r2 L-1).

**Compile-time impact:** any consumer code (in tests or other internal call sites) that references a deleted field will fail to compile. The plan addresses these in A.2.

## Phase A.1d — verify-bundle slot-dispatch wiring (added per architect r1 I-3)

`VerifyBundleArgs::run()` (and its delegates `run_full`, `run_watch_only`, `run_multisig`, `descriptor_mode_verify_run`) currently dispatch on `args.phrase.is_some()` / `args.xpub.is_some()` / `args.cosigner.is_empty()` / `args.cosigners_file.is_some()`. After A.1a deletes these fields, the dispatch must be rewritten to consume `args.slot: Vec<SlotInput>` instead.

**Approach:**

1. Add `pub slot: Vec<SlotInput>` to `VerifyBundleArgs` with the same clap attribute as `BundleArgs::slot`.
2. Refactor `bundle::resolve_slots` (currently private; takes `&BundleArgs` per architect r2 I-1) to take an explicit args-tuple (the fields it actually consults: `template`, `network`, `account`, `language`, `passphrase`, `multisig_path_family`, `privacy_preserving`) instead of `&BundleArgs`. Promote to `pub(crate)`. Both `bundle.rs` and `verify_bundle.rs` then call the same helper. This is a parameter-list change, not an API break — the function body is unchanged. Estimated 5-10 LOC of signature-update churn at the call site in bundle.rs.
3. Reshape `run()` dispatch:
   - Detect mode from `--slot` shape via `bundle_unified::detect_bundle_mode(&args.slot)`.
   - Dispatch to:
     - `run_full(args, ...)`: SingleSigFull mode (read phrase from slot @0).
     - `run_watch_only(args, ...)`: SingleSigWatchOnly mode (xpub + fingerprint from slot @0).
     - `run_multisig(args, ...)`: MultisigMultiSource / MultisigWatchOnly / MultisigHybrid (per-cosigner slots).
4. `run_full` body: replace `args.phrase` with the `phrase` subkey of slot @0 (after resolve). Use `derive_full(...)` as before.
5. `run_watch_only` body: replace `args.xpub` + `args.master_fingerprint` with the corresponding slot @0 subkeys. The `XPUB_NEEDS_FINGERPRINT` guard is no longer needed — confirmed via inspection of `is_legal_set` (slot_input.rs:298-310): SPEC §6.6.b's validity matrix permits `[Xpub]` alone (no fingerprint), so a slot with just `--slot @0.xpub=X` is legitimate. v0.5.1 synthesis emits such a slot as a privacy-preserving watch-only (mk1.origin_fingerprint = None). XPUB_NEEDS_FINGERPRINT was a v0.4-era quirk specific to legacy `--xpub` semantics; deletion is safe.
6. `run_multisig` body: replace `args.cosigner` parsing with the resolved per-slot cosigner vec.
7. `descriptor_mode_verify_run` body: replace `phrase_owned` extraction from `args.phrase` and `cosigner_specs` from `args.cosigner` with their slot-derived equivalents.

**Estimated LOC:** verify_bundle.rs run() + 4 dispatch helpers shrink by ~80-120 lines (the legacy-flag-extraction pre-checks delete; slot-resolved equivalents are 1-2 lines each).

**Test impact:** all integration tests using `--phrase`/`--xpub`/`--cosigner` against `verify-bundle` are rewritten to `--slot` in A.2. The tests' assertion shape (stderr text + exit codes) is unchanged.

## Phase A.1b — whole-file test deletes

```
rm crates/mnemonic-toolkit/tests/cli_mode_violations.rs
rm crates/mnemonic-toolkit/tests/cli_mode_violations_v0_2.rs
rm crates/mnemonic-toolkit/tests/cli_mode_violations_v0_3.rs
```

Total: ~584 lines deleted, 61 legacy-flag references swept.

## Phase A.1c — new cli_mode_violations_v0_5.rs

New file covering the 3 retained guards (`THRESHOLD_WITHOUT_MULTISIG`, `PATH_FAMILY_WITHOUT_MULTISIG`, `DESCRIPTOR_AND_TEMPLATE`) under `--slot` invocations.

**Estimated 6 tests** (one violation case per guard + happy-path counterpart):

1. `threshold_without_multisig_template_rejected` — `bundle --slot @0.phrase=X --template bip84 --threshold 2` → exit 2 with byte-exact stderr `mode_text::THRESHOLD_WITHOUT_MULTISIG`.
2. `threshold_with_multisig_template_accepted` — `bundle --slot @0.phrase=X --slot @1.phrase=Y --template wsh-sortedmulti --threshold 2` → success.
3. `path_family_without_multisig_template_rejected` — `bundle --slot @0.phrase=X --template bip84 --multisig-path-family bip48` → exit 2 with byte-exact `mode_text::PATH_FAMILY_WITHOUT_MULTISIG`.
4. `path_family_with_multisig_template_accepted` — corresponding happy path.
5. `descriptor_and_template_rejected` — `bundle --slot @0.phrase=X --template bip84 --descriptor 'wpkh(@0/...)'` → exit 2 with byte-exact `mode_text::DESCRIPTOR_AND_TEMPLATE` (the existing source const, verbatim: `"--descriptor and --template are mutually exclusive; pick descriptor passthrough or template, not both."`). Note (architect r1 I-1): SPEC §6.6 row 2 lists slightly different shorthand text (`"error: --template and --descriptor are mutually exclusive"`); the SPEC text is sub-normative descriptive, the source const is authoritative byte-exact. Test pins the source const.
6. `descriptor_without_template_accepted` — corresponding happy path.

## Phase A.2 — consumer test rewrites (13 files)

For each test file, mechanical rewrite per the mapping table above. The 41 legacy-flag occurrences cluster around 5-8 distinct invocation shapes; rewrites should consolidate into shared test helpers where possible.

**Per-file approach:**

1. **cli_descriptor_mode.rs (7 occurrences):** descriptor-mode invocations with `--phrase` for full-mode and `--xpub`/`--cosigner` for watch-only/multisig. Rewrite to `--slot @N.<subkey>=`.
2. **cli_bip388_distinctness.rs (6):** typed-DerivationPath equality tests. May overlap with v0.5.0 Phase C.1 audit; verify v0.5.0 fix didn't already touch these.
3. **cli_verify_bundle_watch_only.rs (4):** the spurious-ms1 + happy-path tests use `--xpub --master-fingerprint`. Rewrite to `--slot @0.xpub=X --slot @0.fingerprint=Y`.
4. **cli_bundle_json_intake.rs (4):** verify-bundle round-trip + schema-3 fail tests use `--phrase`. Rewrite to `--slot @0.phrase=`.
5. **cli_verify_bundle_forensics.rs (3):** forensic integration tests use `--phrase` for the bundle generation. Same rewrite.
6. **cli_unified_slot.rs (3):** confirmed inventory (architect r1 I-2): one `--phrase`-using test at lines 70-95 (`unified_slot_phrase_collides_with_legacy_phrase_emits_row6`) — DELETE the entire test. The `TREZOR_BIP84_XPUB` constant at line 11 is silenced via `let _ = TREZOR_BIP84_XPUB;` at line 93 inside that test; after the test deletion, the constant is dead — DELETE it too. The `TREZOR_FP_HEX` constant at line 12 is still actively used at line 357 — KEEP.
7. **cli_help_fixtures.rs (3):** asserts on help-text content. Update help-text fixtures to match the v0.5.1 reduced flag set.
8. **cli_bundle_watch_only.rs (3):** watch-only single-sig + watch-only multisig fixtures. Rewrite.
9. **cli_self_check.rs (2):** `--self-check` round-trip uses `--phrase`. Rewrite.
10. **cli_json_envelopes.rs (2):** JSON envelope shape tests use `--phrase`. Rewrite.
11. **cli_bundle_multisig.rs (2):** `--phrase --cosigner-count N` self-multisig tests (BIP-388 row-13 collision test). Rewrite to multi-source `--slot @0.phrase=X --slot @1.phrase=Y` or accept that BIP-388 row-13 self-multisig is exhibited via duplicate-xpub two-slot bundles.
12. **cli_verify_bundle_full.rs (1):** full-mode round-trip. Rewrite.
13. **cli_bundle_full.rs (1):** single-sig full-mode round-trip. Rewrite.

**Consolidation opportunities:** several tests construct the same `--phrase TREZOR_24 --template bip84 --network mainnet` invocation. A shared `bip84_full_args()` helper in a `tests/common/mod.rs` could reduce churn. v0.5.1 stays scope-tight: prefer in-file consolidation over cross-test-file extraction unless 3+ tests share the same shape.

## Phase A.3 — SPEC §6.6 partial-delivery note removal

Single edit in `design/SPEC_mnemonic_toolkit_v0_5.md`:

```diff
-**v0.5.0 partial delivery:** legacy flag deletion is described in this section but **deferred to v0.5.1** (...). The v0.5.0 binary retains [...] all 9 mode-violation guards from v0.4. The §6.6 table below reflects the v0.5.1 target state. [...]
-
 v0.5 deletes the legacy CLI flags entirely. [...]
```

## Phase R — release prep

1. Bump version: `crates/mnemonic-toolkit/Cargo.toml` `0.5.0` → `0.5.1`.
2. CHANGELOG.md `[0.5.1] — 2026-05-06` entry summarizing A.1+A.2 + scope-completion note.
3. FOLLOWUPS.md: mark `legacy-cli-flag-deletion` and `legacy-flag-deprecation` resolved (cite shipping commit).
4. Final cross-phase architect review (transcript-only).
5. `git tag -a mnemonic-toolkit-v0.5.1 -m "..."` + `git push origin master mnemonic-toolkit-v0.5.1`.
6. `gh release create mnemonic-toolkit-v0.5.1`.
7. Memory update: append v0.5.1 section to `mnemonic_toolkit_v0_5_state.md`; update MEMORY.md index entry.

---

## Verification — end-to-end

After all phases land:

1. **Single-sig full bundle:**
   ```
   mnemonic bundle --slot @0.phrase="..." --template bip84 --network mainnet --json
   ```
   → schema-4 envelope, byte-identical to v0.5.0 emission for the same inputs.

2. **Watch-only single-sig:**
   ```
   mnemonic bundle --slot @0.xpub=<xpub> --slot @0.fingerprint=<fp> --template bip84 --network mainnet --json
   ```
   → schema-4 envelope.

3. **Multi-source multisig 2-of-2:**
   ```
   mnemonic bundle --slot @0.phrase=A --slot @1.phrase=B --threshold 2 --template wsh-sortedmulti --network mainnet
   ```
   → schema-4 envelope.

4. **CLI surface negative tests:**
   - `mnemonic bundle --phrase X` → clap rejects `--phrase` as unknown-arg, exit 64.
   - `mnemonic bundle --xpub X` → exit 64.
   - `mnemonic bundle --cosigner ...` → exit 64.

5. **Mode-violation retained guards:**
   - `mnemonic bundle --slot @0.phrase=X --template bip84 --threshold 2` → exit 2 with byte-exact `THRESHOLD_WITHOUT_MULTISIG` stderr.
   - Same for `--multisig-path-family` on a single-sig template.
   - `mnemonic bundle --template bip84 --descriptor 'wpkh(@0/**)'` → exit 2 with byte-exact `DESCRIPTOR_AND_TEMPLATE` stderr.

## Estimated test-count delta

- v0.5.0 baseline: 236 lib + 22 integration suites.
- A.1b: -3 integration suites (whole-file delete: cli_mode_violations*.rs).
- A.1c: +1 integration suite (cli_mode_violations_v0_5.rs).
- A.2: -1 integration test in cli_unified_slot.rs (the row-6 conflict test at lines 70-95; confirmed inventory per architect r2 L-2).
- Lib unit tests: +0 to -2 (deleted alias-handler unit tests in slot_input.rs).
- Net: ~234 lib + ~20 integration suites at v0.5.1 ship.

## Out-of-scope for v0.5.1

- `unified-slot-xprv-resolution-needs-ms-codec-extension` — blocked on ms-codec sibling repo (cross-repo dependency). Stays at v0.5+ tier.

That's the only remaining open FOLLOWUP after v0.5.1.
