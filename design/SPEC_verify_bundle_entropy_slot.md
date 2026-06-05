# SPEC — `verify-bundle --descriptor --slot @N.entropy=` binding arm (v0.43.1)

**FOLLOWUP:** `verify-bundle-descriptor-entropy-slot-gap`
**Source SHA (origin/master at write time):** `0f404ae`
**Cycle type:** PATCH (`v0.43.0` → `v0.43.1`). Toolkit-only. **No GUI `schema_mirror` lockstep, no manual mirror** (no clap-surface change — see §6).
**Recon:** `cycle-prep-recon-restore-multisig-vbundle-entropy-schema-wireshape.md` (all citations ACCURATE).

---

## 1. Problem

`verify-bundle --descriptor <BIP-388 @N template> --slot @N.entropy=<hex>` currently fails. The descriptor-mode binding loop in `verify_bundle.rs` has arms for `Phrase`/`Seedqr` (`:788`), `Xpub` (`:830`), and `Ms1` (`:855`), then a catch-all `else` (`:885-891`) that returns `ToolkitError::DescriptorReparseFailed` (**exit 4** per `error.rs:503` — runtime-verified; the FOLLOWUP's "exit 2" is a stale citation confusing it with `DescriptorParse → 2`), detail `"--slot @{idx} subkey set {:?} not supported in descriptor verify-bundle path"`. There is **no `SlotSubkey::Entropy` arm**, so a raw-entropy cosigner falls through to the catch-all. Runtime RED baseline (unpatched, probed 2026-06-04): `bundle … --slot @0.entropy=<hex>` → exit 0 (bundles fine); `verify-bundle …` → **exit 4**, stderr `error: descriptor re-parse failed during verify-bundle: --slot @0 subkey set ["entropy"] not supported in descriptor verify-bundle path`, empty stdout.

This is a pure asymmetry, not a design boundary:

- The **`bundle`** descriptor path (`bundle.rs:1438`, `bundle_run_unified_descriptor`) **does** have an `Entropy` arm and derives the cosigner from raw entropy.
- The **`verify-bundle` TEMPLATE** path resolves `@N.entropy=` via the shared `resolve_slots` (`bundle.rs:453`, `Entropy` arm at `:610`).
- Only **`verify-bundle` + `--descriptor` + raw `entropy`** is unbindable.

`entropy` is already a documented, recognized, secret-bearing `--slot` subkey (manual `41-mnemonic.md:67`; `slot_input.rs:85` `is_secret_bearing`). The manual even documents `ms1` as materializing "the slot's entropy identically to a `@N.entropy=` invocation," and `33-taproot-multi.md:111` states verify-bundle "accepts the same `--descriptor` … the bundle was built with." So the manual already promises parity that the code does not yet honor. The fix makes code match the documented contract.

## 2. Scope

**Exactly one new arm + a small matrix of tests.** Recon + pre-flight confirmed this is the *only* instance of the gap:

- `xprv` / `wif` descriptor-verify-bundle arms remain deferred under their own scope (xprv → v0.5+; wif → multisig FOLLOWUP) — out of scope here.
- No positive per-mode allow-list of subkeys exists anywhere (only the negative catch-all at `verify_bundle.rs:888` and `bundle.rs:1503`), so there is no "second site" to update (`feedback_fix_the_class_hunt_for_second_instance` satisfied by absence).

**Out of scope:** any change to `xprv`/`wif` arms, the catch-all message wording (it is generic and remains correct for the still-unsupported sets), GUI schema, or manual prose.

## 3. The change

Insert a new arm into the `verify_bundle.rs` descriptor binding loop, **between** the `Xpub` arm (ends `:854`) and the `Ms1` arm (`:855` `else if`), mirroring the `bundle` loop's arm ordering (`Phrase/Seedqr → Xpub → Entropy → Ms1`). Placement mirrors the bundle loop **for consistency**; precedence is moot — `is_legal_set` (`slot_input.rs:343-367`) permits `[Entropy]` as a standalone-only set (no `[Entropy, *]` co-occurrence exists), so `subkeys.contains(&Entropy)` is never simultaneously true with the `Xpub`/`Ms1` `contains` checks. (Corrected per R0-r1 M1.)

The arm is the existing `Ms1` arm (`:855-884`) minus the ms1-decode step — it routes through the **same shared helper** the `Ms1` arm uses, exactly as the FOLLOWUP prescribes ("derive via `derive_slot::derive_bip32_from_entropy_at_path` at `anno_path`"):

```rust
} else if subkeys.contains(&crate::slot_input::SlotSubkey::Entropy) {
    // v0.43.1 — raw-`entropy` cosigner in descriptor verify-bundle mode
    // (FOLLOWUP `verify-bundle-descriptor-entropy-slot-gap`). Mirror of the
    // bundle-loop Entropy arm (bundle.rs:1438): hex-decode, then derive at the
    // descriptor-annotated `anno_path` via the shared helper. emit_lang = None
    // — raw entropy carries no BIP-39 wire language (symmetric with the bundle
    // Entropy arm, which returns None as its 5th element).
    let entropy_hex = slot_inputs
        .iter()
        .find(|s| s.subkey == crate::slot_input::SlotSubkey::Entropy)
        .map(|s| s.value.as_str())
        .expect("contains() asserts presence");
    let entropy_bytes = zeroize::Zeroizing::new(hex::decode(entropy_hex).map_err(|e| {
        ToolkitError::BadInput(format!("--slot @{idx}.entropy hex-decode: {e}"))
    })?);
    let language = args.language.unwrap_or_default();
    let passphrase: zeroize::Zeroizing<String> =
        zeroize::Zeroizing::new(args.passphrase.clone().unwrap_or_default());
    let acc = crate::derive_slot::derive_bip32_from_entropy_at_path(
        &entropy_bytes,
        &passphrase,
        language.into(),
        args.network,
        &anno_path,
    )?;
    let (_acc_entropy, master_fp, xpub, _xpriv, _path) = acc.into_parts();
    (
        xpub,
        master_fp,
        anno_path.clone(),
        Some((*entropy_bytes).clone()),
        None,
    )
}
```

Notes:
- `hex` is already a crate dep (`Cargo.toml:50`, `hex = "0.4"`).
- `language.into()`: `args.language` is `Option<CliLanguage>`; `.unwrap_or_default()` then `.into()` → `bip39::Language` (the helper's `language` param), identical to the `bundle` Entropy arm (`bundle.rs:1453,1456`).
- The 5-tuple element types are `(BipXpub, Fingerprint, DerivationPath, Option<Vec<u8>>, Option<bip39::Language>)` (`verify_bundle.rs:782-787`). `emit_lang = None` is forced by symmetry with the bundle Entropy arm (`bundle.rs:1470` returns `None`).
- Error behavior: hex-decode failure → `BadInput` (`--slot @N.entropy hex-decode: …`); invalid BIP-39 entropy length (not 16/20/24/28/32 B) → `ToolkitError::Bip39` (raised inside the helper's `Mnemonic::from_entropy_in`).

## 4. Output-symmetry argument (why this verifies correctly)

`verify-bundle` re-emits each cosigner's card and compares the whole card string against the supplied `--ms1/--mk1/--md1`. For a slot originally bundled from `@N.entropy=<hex>`, the `bundle` Entropy arm derived `(xpub, master_fp, anno_path, Some(entropy), None)`. This arm produces the **identical** 5-tuple from the same inputs (same helper, same `anno_path`, same `emit_lang=None`), so the re-emitted ms/mk/md cards are byte-identical to the bundle's → `result: ok`. This is the load-bearing invariant (cf. `feedback_verify_bundle_round_trip_per_phase_r0_scope`).

## 5. Test matrix (small matrix — user-selected breadth)

New dedicated file `crates/mnemonic-toolkit/tests/cli_verify_bundle_entropy_slot.rs` (keeps the new construction out of `cli_ms1_slot.rs`). All positive tests must first go **RED for the right reason** (verify step → **exit 4** `DescriptorReparseFailed`, stderr `…subkey set ["entropy"] not supported in descriptor verify-bundle path`; runtime-verified) against the unpatched binary, then GREEN after the arm lands.

Fixtures (re-declare locally as constants):
- `NONCANONICAL_DESC = "tr(NUMS,and_v(v:pk(@0),after(12000000)))"` (single `@0`; from `cli_ms1_slot.rs:294`).
- `ANDOR3_DESC = "wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))"` (3 cosigners `@0`/`@1`/`@2`; the **proven** non-canonical multi-`@N` fixture from `cli_non_canonical_descriptor.rs:22`, already shown to bundle secret slots with `.success()`).

(`CANONICAL_DESC = "wsh(sortedmulti(2,@0,@1))"` is deliberately NOT used — there is no precedent for a secret cosigner bundling on a canonical `wsh(sortedmulti)` `--descriptor` string; every such use in the suite is a refusal test. Per R0-r1 I1, the multi-`@N` cell uses the proven `ANDOR3_DESC` instead.)

Helper: a local `bundle_descriptor_cards(desc, &slot_args, lang, passphrase) -> (ms1, mk1, md1)` that runs `bundle --descriptor … --slot … [--language …] [--passphrase-env …] --json --no-engraving-card` and returns the extracted card vectors (clone of `extract_cards`, `cli_ms1_slot.rs:405`). Secret slot values pass `--slot @N.x=<value>` directly (argv); the secret-on-argv stderr warning is benign in tests.

1. **`round_trip_len32`** — `@0.entropy=<hex(32B)>` on `NONCANONICAL_DESC`: `bundle … --json` → extract cards → `verify-bundle … --slot @0.entropy=<hex> --ms1/--mk1/--md1 …` → `.code(0)` + stdout `result: ok`. (Core symmetry: the bundle Entropy arm and the new verify Entropy arm — distinct code paths — must agree.)
2. **`round_trip_len16`** — same as #1 with `@0.entropy=<hex(16B)>` (12-word equivalent) → `result: ok`. Guards `Mnemonic::from_entropy_in` length acceptance (16 vs 32 B).
3. **`nonzero_slot_multi_n`** — `ANDOR3_DESC` 3-cosigner with `@0.phrase=<12-word>` + **`@1.entropy=<hex(16B)>`** + `@2.phrase=<12-word>`: bundle (`--language english --account 0`) then verify-bundle → `result: ok`. Proves the new Entropy arm fires at a **non-`@0` position** in a **multi-`@N`** descriptor and composes with the Phrase arm handling the sibling slots. Uses the proven-bundlable `ANDOR3_DESC` (`cli_non_canonical_descriptor.rs:22`). (Phrase, not `xpub`, on the sibling slots — mixed secret+`xpub` on a `--descriptor` *string* is unproven across the canonicity boundary; phrase keeps every slot on proven ground while still exercising the entropy arm beside a different arm.)
4. **`passphrase_round_trip`** — `@0.entropy=<hex(32B)>` on `NONCANONICAL_DESC` with `--passphrase` (same value) on **both** bundle and verify → `result: ok`. Exercises the new arm's `args.passphrase` handling (positive).
5. **`passphrase_mismatch_detected`** — bundle `@0.entropy=<hex(32B)>` with `--passphrase <A>`; verify the SAME entropy slot with `--passphrase <B≠A>` + the A-cards → `.code(4)` + stdout `result: mismatch`. Proves the arm derives an **input-dependent** key and the verify comparison is **live** (not a no-op). **Replaces the originally-specified `self_check` test** (Phase-1 RED-authoring discovery): `--self-check` is a `bundle`-only flag (`bundle.rs:86`) whose `self_check_bundle` routine (`bundle.rs:2027`) re-parses the bundle internally and does **not** route through the `verify_bundle.rs` descriptor binding loop — it would not exercise the new arm at all. NB: both RED (catch-all `DescriptorReparseFailed`) and GREEN (`BundleMismatch`) are exit 4 here, so this test's GREEN assertion keys on **stdout `result: mismatch`** (only reachable post-fix), not the exit code alone.

Total: 5 tests. Each is a `Command::cargo_bin("mnemonic")` integration test (BIN target, not `--lib`). Secrets passed via argv `--slot`/`--passphrase` (test-only; the stderr secret-on-argv warning is expected and ignored).

## 6. SemVer / lockstep / non-goals

- **SemVer:** PATCH. No new flag/option/subcommand/value-enum — `entropy` is a pre-existing `--slot` subkey. Behavior-only fix (previously-erroring input now succeeds).
- **GUI `schema_mirror`:** **not triggered** (gate is clap flag-NAME + value-enum parity; no surface change). Confirmed: no new flag.
- **Manual mirror:** **not triggered.** `41-mnemonic.md:67` already lists `entropy`; no per-mode subkey enumeration excludes it; `33-taproot-multi.md:111` documents verify-bundle/bundle `--descriptor` parity. Fix aligns code to existing docs.
- **`lint_argv_secret_flags` / secret taxonomy:** unchanged — `entropy` already secret-bearing (`slot_input.rs:85`).
- **Sibling-codec FOLLOWUP companions:** none (toolkit-only).

## 7. Phased plan

**Phase 1 — TDD RED.** Add `cli_verify_bundle_entropy_slot.rs` with the 5 tests. Run against the unpatched binary; confirm each verify step fails with **exit 4** `DescriptorReparseFailed` (stderr `…subkey set ["entropy"] not supported…`; RED for the right reason — runtime-verified). Commit tests (RED documented in the commit body).

**Phase 2 — Implement + GREEN.** Insert the arm (§3) between `:854` and `:855`. Run the new file + the full `cargo test --no-fail-fast` workspace suite (grep `feedback_shared_string_change_sweep_all_tests_full_suite_per_commit` — though no shared string changes here, the full suite is mandatory per-phase). Confirm 5/5 GREEN and zero regressions. Per-phase opus architect review → persist to `design/agent-reports/verify-bundle-entropy-slot-phase-2-rN-review.md` → fold → re-dispatch until 0C/0I.

**Phase 3 — Release prep (v0.43.1).** Per `feedback_phase_6_release_prep_checklist_readme_markers`:
- `crates/mnemonic-toolkit/Cargo.toml` version `0.43.0` → `0.43.1`; **stage `Cargo.lock`** with it (`feedback_phase_6_cargo_lock_stage_with_version_bump`).
- `CHANGELOG.md` — new `v0.43.1` entry.
- README `<!-- toolkit-version: … -->` markers (both) → `0.43.1` (`readme_version_current` guard).
- `scripts/install.sh` self-pin TAG → `mnemonic-toolkit-v0.43.1` (`feedback_phase_6_install_sh_pin_bump_required`).
- `design/FOLLOWUPS.md` — `verify-bundle-descriptor-entropy-slot-gap` `Status: open` → `resolved` (`feedback_per_phase_agents_forget_followup_status_flip`).
- End-of-cycle opus architect review (full suite green + prose) → persist → fold to 0C/0I.

**Phase 4 — Ship + tag.** Clean tree first (`git status --porcelain` clean; the untracked recon/survey/CONTINUITY files stay untracked — do not stage; `feedback_clean_tree_before_ship_merge_tag`). Stage paths explicitly (no `git add -A`). `checkout master → merge --ff-only verify-bundle-entropy-slot → tag mnemonic-toolkit-v0.43.1 → push origin master --tags`. Toolkit tags are tag-only (no GitHub Release; `manual.yml` fires only on `manual-v*`).

## 8. Mandatory R0 gate

Per `CLAUDE.md`: this SPEC must pass an **opus architect R0 review to 0 Critical / 0 Important BEFORE any test/impl code is written.** Persist each round verbatim to `design/agent-reports/verify-bundle-entropy-slot-r0-rN-review.md`; fold → re-dispatch until GREEN. The architect brief must instruct reading source ground truth at `0f404ae` AND running the round-trip command end-to-end (`feedback_architect_must_run_prose_commands`, `feedback_r0_must_read_source_off_by_n`).
