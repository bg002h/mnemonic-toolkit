# mnemonic-toolkit v0.8 — `export-wallet` multi-format expansion implementation plan

**Status:** DRAFT (post-plan-R1, 0C/0I after architect round; awaiting IMPLEMENTATION_PLAN-level reviewer-loop).
**Predecessors:** [IMPLEMENTATION_PLAN_v0_7.md](IMPLEMENTATION_PLAN_v0_7.md), [SPEC_export_wallet_v0_8.md](SPEC_export_wallet_v0_8.md), [SPEC_export_wallet_v0_7.md](SPEC_export_wallet_v0_7.md).
**Sibling FOLLOWUPS touched:** `wallet-export-industry-formats` (already `Status: resolved 3821f66` by v0.7 Phase 5; v0.8.1 cycle extends coverage from 2 → 8 formats via `Resolution-extended:` notes appended to the existing entry — no reopen), `export-wallet-descriptor-bip388-interop` (stays deferred), `tr-multi-a-tr-sortedmulti-a-export-wallet-support` (partial cover via Sparrow + Specter descriptor passthrough).
**Sibling FOLLOWUPS introduced by this cycle:** `coldcard-tr-multi-a-pending-firmware`, `coldcard-bip86-generic-export-pending-firmware`, `jade-tr-multi-a-pending-firmware`, `electrum-final-seed-version-drift`, `electrum-tr-multi-a-pending-libsecp-taproot`, `green-native-multisig-pending-server-support`.

## Context

`mnemonic export-wallet` (v0.7) already emits **watch-only** wallet artifacts in Bitcoin Core (`importdescriptors` JSON) and BIP-388 (`wallet_policy` JSON) formats; Sparrow and Specter slots are present in the `--format` enum but return `format_stub_message` refusals (`src/cmd/export_wallet.rs:148-153`). The recurring user ask — "make it work with the wallet I actually own" — is unmet for everything outside the descriptor-JSON pair.

This cycle adds first-class emit support for five additional vendor formats so a toolkit user can hand a single file to their target wallet and complete a watch-only setup with no hand-assembly: **Coldcard** (generic JSON skeleton + multisig text), **Blockstream Jade** (multisig text, byte-identical to Coldcard's), **Sparrow Wallet** (native JSON), **Specter Desktop** (native JSON), **Electrum** (`seed_version`-pinned JSON, single + multisig), and **Blockstream Green** (thin descriptor-text file with Help-Center pointer comment). No tx construction, no signing, no network. The existing watch-only refusal class (`REFUSAL_SECRET_INPUT`, `src/wallet_export.rs:17-18`) remains the trip-wire.

The work folds into the **v0.8 series** as the **v0.8.1** cut. v0.8.0 (commit `7bb722a`, 2026-05-07) shipped taproot-internal-key, electrum-version-info-stderr, and 12 other v0.8 FOLLOWUPS as a `[BREAKING]` cut; v0.8.1 ships these six new export-wallet emitters as **additive** (no breaking change to v0.7-stable `bitcoin-core` / `bip388` byte-exact fixtures).

## Locked decisions (from this planning conversation)

1. **Integration:** extend the existing `export-wallet` subcommand — add five new `CliExportFormat` variants; delete the existing Sparrow/Specter stub arms.
2. **Coverage:** Coldcard + Jade + Sparrow + Specter + Electrum + Green (Green ships a thin descriptor-text file, NOT a blanket refusal).
3. **Missing-info UX:** byte-exact refusal listing **all** missing fields in one message, deterministically ordered by `MissingField` enum discriminant then by slot index. Exit code 2. Never partial JSON.
4. **Phases:** five per-format phases. Phase 1 — Coldcard + Jade (text format byte-identical, ride along). Phase 2 — Sparrow. Phase 3 — Specter. Phase 4 — Electrum (SLIP-132 round-tripping is the heaviest piece). Phase 5 — Green (thin descriptor-text emitter). Each phase = TDD before impl + opus reviewer-loop until 0C/0I + report under `design/agent-reports/`.
5. **Version:** v0.8 series. SPEC + IMPLEMENTATION_PLAN files become `design/SPEC_export_wallet_v0_8.md` + `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md` (folded out from the plan file once the user approves).

## TDD discipline

Per CLAUDE.md and `IMPLEMENTATION_PLAN_v0_7.md` precedent: each phase begins with RED tests, ships impl in the same phase commit, and runs an opus reviewer-loop until 0C/0I — report persisted to `design/agent-reports/v0_8-phase-N-<name>-r{N}.md`. No stub semantics that flip GREEN twice; each phase is atomic (variants + fixtures + emitter + dispatch in one commit).

## Phase 0 — Preconditions + module reorganization

Pre-code:
1. Promote this plan file's Part A (SPEC) to `design/SPEC_export_wallet_v0_8.md`. Run opus reviewer-loop on the SPEC until 0C/0I; persist `design/agent-reports/v0_8-spec-r{N}.md`.
2. Promote this plan file's Part B (this section onward) to `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md`. Run opus reviewer-loop on the IMPLEMENTATION_PLAN; persist `design/agent-reports/v0_8-impl-plan-r{N}.md`.

Code (module reorganization only, no behavior change):
3. Split `src/wallet_export.rs` → `src/wallet_export/` submodule tree per SPEC §12. Move existing `format_bitcoin_core_importdescriptors` to `bitcoin_core.rs` (wrap as `BitcoinCoreEmitter::emit` returning `String` via `serde_json::to_string_pretty`), `format_bip388_wallet_policy` to `bip388.rs` (same wrapping), descriptor pipeline to `pipeline.rs`. Move `format_stub_message` helper to `wallet_export/mod.rs`. Preserve all `pub(crate)` names.
4. Update `cmd::export_wallet::run` call site: replace the `serde_json::to_string_pretty(&value)?` block with the trait dispatch `let emitted: String = match args.format { ... }?; write_output(&args.output, emitted.as_bytes(), stdout)?;` (per SPEC §12 dispatch snippet). v0.7 byte-exact fixtures for bitcoin-core and bip388 remain valid (pretty-print is deterministic).
5. Add `WalletScriptType` enum + `script_type_from_template` + `script_type_from_descriptor` to `wallet_export/mod.rs` (per SPEC §12 R1-I1 hardening). Do NOT touch `cmd::convert::ScriptType` (stays single-sig-only).
6. Add `MissingField` enum (7 variants per SPEC §4) + `build_missing_fields_refusal` + `ToolkitError::ExportWalletMissingFields { format, missing }` variant. Define the `EmitInputs` struct per SPEC §12 — including the `wallet_name_was_user_supplied: bool` field that `SpecterEmitter::collect_missing` consumes in Phase 3 (and that Phase 1 step 7's default-resolution logic sets). Zero new behavior tests (validator does not fire yet in v0.7 paths).

**Phase 0 exit gate:**

```fish
cargo build --workspace
cargo build --bin mnemonic                # local binary for the manual-mirror gate below
cargo test --workspace --no-fail-fast     # all v0.7 tests still GREEN; zero new tests yet
cargo clippy --workspace -- -D warnings   # workspace excludes example crates
make -C docs/manual lint MNEMONIC_BIN="$(pwd)/target/debug/mnemonic" MD_BIN=true MS_BIN=true MK_BIN=mk
                                          # bidirectional CLI↔manual mirror; exercises the real
                                          # `mnemonic --help` (NOT `true`, which would be vacuous —
                                          # see FOLLOWUPS lint-md-flag-coverage-vacuous-with-md_bin-true).
                                          # CI continues to use MNEMONIC_BIN=true per manual.yml.
```

## Phase 1 — Coldcard + Jade

RED first:
1. Pin byte-exact fixtures: `tests/export_wallet/coldcard_generic_bip84_mainnet.json`, `coldcard_generic_bip49_testnet.json`, `coldcard_generic_bip44_mainnet.json`, `coldcard_multisig_2of3_wsh.txt`, `jade_multisig_2of3_wsh.txt`. (No bip86 fixture — refused per SPEC §5.1, R1-I2.)
2. Write `tests/helpers/coldcard_parse.rs` — parses generic JSON + multisig text back into structural fields; assert round-trip equality against the canonical inputs.
3. Pin refusal fixtures: `coldcard_missing_xfp_refusal.stderr`, `coldcard_multisig_template_skeleton_mismatch_refusal.stderr`, `coldcard_bip86_pending_firmware_refusal.stderr`, `jade_singlesig_refusal.stderr`, `jade_tr_multi_a_refusal.stderr`, `multi_missing_fields_aggregate_refusal.stderr`.

Code:
4. Implement `coldcard.rs` — `emit_coldcard_generic_json` (singlesig sub-object per template: bip44/bip49/bip84 only; bip86 → byte-exact refusal per SPEC §5.1) + `emit_coldcard_multisig_text`. Wire SLIP-132 `_pub` via `crate::slip0132`. XFP rendered UPPERCASE.
5. Implement `jade.rs` — `emit_jade_multisig_text` delegates to `coldcard::emit_coldcard_multisig_text`. Singlesig + tr-multi-a refuse.
6. Wire `CliExportFormat::Coldcard` + `Jade` into `cmd::export_wallet::run`. **Sparrow + Specter stub arms at `cmd/export_wallet.rs:148-153` remain untouched** (replaced in Phase 2 / Phase 3 respectively, not earlier). v0.7 tests asserting Sparrow/Specter clean-refusal stderr continue to pass through Phase 1.
7. Add `--wallet-name` clap flag as `Option<String>` (clap-derive). Default resolution happens in `cmd::export_wallet::run` AFTER template + account are resolved: `let wallet_name = args.wallet_name.clone().unwrap_or_else(|| format!("{}-{}", template_human_name(template), account));`. The specter-required check happens later in `SpecterEmitter::collect_missing` via a `wallet_name_was_user_supplied: bool` field on `EmitInputs` (add this field in Phase 0 step 6 alongside the rest of `EmitInputs`). Update `docs/manual/src/40-cli-reference/41-mnemonic.md` for both the new flag and the new `--format` values.
8. Run `make -C docs/manual lint MNEMONIC_BIN="$(pwd)/target/debug/mnemonic" MD_BIN=true MS_BIN=true MK_BIN=mk` — confirm CLI↔manual mirror green against the real `mnemonic` binary (CI substitutes `MNEMONIC_BIN=true` per `.github/workflows/manual.yml`, which renders the per-flag check vacuous for `mnemonic`; local exit gate must exercise the real binary to catch drift before push).

Phase 1 reviewer-loop:
9. Run opus reviewer-loop against the phase delta until 0C/0I; persist `design/agent-reports/v0_8-phase-1-coldcard-jade-r{N}.md`.
10. Update `design/FOLLOWUPS.md`: add `coldcard-tr-multi-a-pending-firmware`, `coldcard-bip86-generic-export-pending-firmware`, `jade-tr-multi-a-pending-firmware`. Append a `Resolution-extended (v0.8.1 Phase 1):` line to the existing (already-resolved) `wallet-export-industry-formats` entry listing Coldcard + Jade shipped — do NOT reopen the entry; the FOLLOWUPS schema has no "reopen" state, only additive `Resolution-extended:` notes on a resolved entry.

**Phase 1 exit gate:** Coldcard + Jade emit byte-exact fixtures; missing-field refusals deterministic; SLIP-132 `_pub` round-trips; reviewer-loop 0C/0I.

## Phase 2 — Sparrow

RED first:
1. Pin fixtures: `sparrow_single_wpkh.json`, `sparrow_multi_2of3_wsh_sortedmulti.json`, `sparrow_single_tr.json`.
2. Pin refusal fixture: `sparrow_missing_threshold_refusal.stderr`.

Code:
3. Implement `sparrow.rs` — `emit_sparrow_wallet_json`. `policyType` from `template.is_multisig()`; `scriptType` mapped from `EmitInputs.script_type` (`WalletScriptType`); `defaultPolicy.miniscript.script` built per script type; `keystores` length = 1 (single) or N (multi). XFP lowercase. xpub BIP-32 form (never SLIP-132).
4. Wire `CliExportFormat::Sparrow`. **Delete the v0.7 Sparrow stub arm at `cmd/export_wallet.rs:148-150`** (now replaced by real impl). The Specter stub arm at `cmd/export_wallet.rs:151-153` remains untouched until Phase 3. Update manual mirror.

Phase 2 reviewer-loop:
5. Run reviewer-loop until 0C/0I; persist `design/agent-reports/v0_8-phase-2-sparrow-r{N}.md`.

**Phase 2 exit gate:** Sparrow emits byte-exact fixtures (singlesig wpkh + multisig wsh-sortedmulti + singlesig p2tr); reviewer-loop 0C/0I.

## Phase 3 — Specter

RED first:
1. Pin fixtures: `specter_single_wpkh.json`, `specter_multi_2of3.json`.

Code:
2. Implement `specter.rs` — `emit_specter_wallet_json`. `descriptor` field MUST round-trip through `pipeline::build_descriptor_string` so `#checksum` matches the bitcoin-core branch byte-exact. `devices` array length = cosigner count, filled with `"unknown"`. `--wallet-name` is REQUIRED for `--format specter` (R1-L1 hardening); `collect_missing` returns `MissingField::WalletName` when absent.
3. Pin `specter_missing_wallet_name_refusal.stderr` fixture.
4. Wire `CliExportFormat::Specter`. **Delete the (now sole-remaining) v0.7 Specter stub arm at `cmd/export_wallet.rs:151-153`** (now replaced by real impl). The stub-arm match block (and the wildcard `_ => {}` arm at line 154) is fully gone after this phase since every `CliExportFormat` variant is dispatched. Update manual mirror.

Phase 3 reviewer-loop:
5. Run reviewer-loop until 0C/0I; persist `design/agent-reports/v0_8-phase-3-specter-r{N}.md`.

**Phase 3 exit gate:** Specter emits byte-exact fixtures; descriptor `#checksum` cross-verifies with bitcoin-core emitter output; reviewer-loop 0C/0I.

## Phase 4 — Electrum

Phase 4 step 0 — **Electrum seed-version spike** (read-only):
- Install current Electrum (>= 4.5.x) in a scratch venv.
- Create a watch-only wallet by importing a known xpub via `electrum --offline restore <xpub> --wallet_path /tmp/electrum-spike-single.json` (pin the path explicitly — without `--wallet_path` Electrum 4.5.x writes to a default location whose name varies by version, which makes the spike report ambiguous).
- Read `/tmp/electrum-spike-single.json` directly; extract the `seed_version` value.
- Repeat for a multisig wallet (e.g., `electrum --offline restore "<xpub1>;<xpub2>;<xpub3>" --wallet_path /tmp/electrum-spike-multi.json` — verify multisig restore syntax against `https://electrum.readthedocs.io/en/latest/cmdline.html` before running).
- Persist the spike report to `design/agent-reports/v0_8-phase-4-electrum-seed-version-spike.md` with: Electrum version, observed `seed_version` for singlesig, observed for multisig, the exact keystore shape Electrum wrote.
- Lock `ELECTRUM_SEED_VERSION_PIN` to the observed value. If singlesig and multisig disagree, use the higher (Electrum's loader walks all `_convert_version_N` migrations idempotently).

RED first:
1. Pin fixtures: `electrum_single.json` and `electrum_multi_2of4.json` to match the **spike-observed** byte shape (NOT Coldcard's stale sample fixtures, per R1-C1).
2. Pin refusal fixture: `electrum_tr_multi_a_refusal.stderr`.
3. Write a SLIP-132 round-trip test: supply `--slot @0.xpub=vpub...`, verify emitted `keystore.xpub` matches the script-type-correct SLIP-132 variant.

Code:
4. Add constant `ELECTRUM_SEED_VERSION_PIN` in `electrum.rs` with a doc-comment citing the Phase 4 step 0 spike report and the rationale (Electrum upgrades older versions on load; we pin to the value current Electrum writes for watch-only).
5. Implement `electrum.rs` — `emit_electrum_standard_json` (singlesig) + `emit_electrum_multisig_json` (multisig with `"x1/"`..`"xN/"` keystores). SLIP-132 variant selection via `crate::slip0132::variant_for(script_type, network)`. **Verify before coding** whether `variant_for` already exists in `slip0132.rs`; if not, add it as a pure addition (no API churn to existing exports).
6. Wire `CliExportFormat::Electrum`. Update manual mirror.

Phase 4 reviewer-loop:
7. Run reviewer-loop until 0C/0I; persist `design/agent-reports/v0_8-phase-4-electrum-r{N}.md`.
8. Update `design/FOLLOWUPS.md`: add `electrum-final-seed-version-drift`, `electrum-tr-multi-a-pending-libsecp-taproot`. **No `Companion:` cross-cite to mk-codec** (R1-N2 hardening: `crate::slip0132` manipulates base58check bytes directly and does not cross the codec boundary).

**Phase 4 exit gate:** Electrum emits byte-exact fixtures; SLIP-132 conversion verified for at least one round-trip; reviewer-loop 0C/0I.

## Phase 5 — Green

RED first:
1. Pin fixture: `green_descriptor.txt` (3 lines: 2 comment lines + canonical descriptor).
2. Pin refusal fixture: `green_multisig_refusal.stderr`.

Code:
3. Implement `green.rs` — `emit_green_descriptor_text`. Singlesig emits the 3-line file. Multisig REFUSES with byte-exact pointer.
4. Wire `CliExportFormat::Green`. Update manual mirror.

Phase 5 reviewer-loop:
5. Run reviewer-loop until 0C/0I; persist `design/agent-reports/v0_8-phase-5-green-r{N}.md`.
6. Update `design/FOLLOWUPS.md`: add `green-native-multisig-pending-server-support`. Append a final `Resolution-extended (v0.8.1 Phase 5):` line to the (already-resolved) `wallet-export-industry-formats` entry listing all six new formats shipped this cycle; entry stays `Status: resolved` (do NOT flip its status — there is no "RESOLVED-again" state).

**Phase 5 exit gate:** Green emits byte-exact fixture; multisig refusal byte-exact; reviewer-loop 0C/0I.

## Phase 6 — Release roll-up

1. End-to-end smoke (per Verification section below).
2. Update `CHANGELOG.md` with `## mnemonic-toolkit [0.8.1] — 2026-05-??` entry listing six new formats (`coldcard`, `jade`, `sparrow`, `specter`, `electrum`, `green`), `--wallet-name` flag (optional, required-for-specter), module reorganization (`wallet_export.rs` → `wallet_export/` submodule, internal-only). **No breaking changes** — v0.7 stable `--format bitcoin-core` / `--format bip388` byte-exact fixtures continue to pass through the new submodule dispatch. (v0.8.0 at commit `7bb722a` was `[BREAKING]` per the v0.8 series header in `CHANGELOG.md`; this cut is additive.)
3. Final cargo gauntlet:

```fish
cargo build --workspace
cargo build --bin mnemonic
cargo test --workspace --no-fail-fast
cargo clippy --workspace -- -D warnings
cargo doc --workspace --no-deps
make -C docs/manual lint MNEMONIC_BIN="$(pwd)/target/debug/mnemonic" MD_BIN=true MS_BIN=true MK_BIN=mk
```

4. Add a per-format worked example to `docs/manual/src/30-workflows/` for Coldcard multisig text (the workflow with the most unique-vendor knowledge).
5. Tag `mnemonic-toolkit-v0.8.1`. CI workflow `.github/workflows/manual.yml` attaches the manual PDF to the release.

## Critical files

**New (in submodule):**
- `crates/mnemonic-toolkit/src/wallet_export/mod.rs` — `MissingField`, `build_missing_fields_refusal`, `WalletFormatEmitter` trait, `EmitInputs`, re-exports.
- `crates/mnemonic-toolkit/src/wallet_export/pipeline.rs` — moved from `wallet_export.rs`.
- `crates/mnemonic-toolkit/src/wallet_export/bitcoin_core.rs` — moved.
- `crates/mnemonic-toolkit/src/wallet_export/bip388.rs` — moved.
- `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs` — Phase 1.
- `crates/mnemonic-toolkit/src/wallet_export/jade.rs` — Phase 1.
- `crates/mnemonic-toolkit/src/wallet_export/sparrow.rs` — Phase 2.
- `crates/mnemonic-toolkit/src/wallet_export/specter.rs` — Phase 3.
- `crates/mnemonic-toolkit/src/wallet_export/electrum.rs` — Phase 4.
- `crates/mnemonic-toolkit/src/wallet_export/green.rs` — Phase 5.

**Removed:**
- `crates/mnemonic-toolkit/src/wallet_export.rs` (replaced by submodule).

**Modified:**
- `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` — extend `CliExportFormat`; add `--wallet-name`; delete stub arms at 148-153 incrementally (Phase 2 deletes Sparrow at 148-150, Phase 3 deletes Specter at 151-153 plus wildcard at 154); thread `EmitInputs` through dispatch.
- `crates/mnemonic-toolkit/src/error.rs` — new `ExportWalletMissingFields { format, missing }` variant + `user_text()` arm.
- `crates/mnemonic-toolkit/src/slip0132.rs` — add `variant_for(script_type, network)` helper if not present (pure addition).
- `docs/manual/src/40-cli-reference/41-mnemonic.md` — mirror invariant (one update per phase).
- `design/FOLLOWUPS.md` — per-phase entries.

**New design artifacts:**
- `design/SPEC_export_wallet_v0_8.md` (from Part A above).
- `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md` (from Part B above).
- `design/agent-reports/v0_8-{spec,impl-plan,phase-1-coldcard-jade,phase-2-sparrow,phase-3-specter,phase-4-electrum,phase-5-green}-r{N}.md`.

**No new Cargo deps anticipated** (`serde_json`, `bitcoin`, `miniscript`, `bip39`, `hex`, `sha3` cover everything).

## Reuse opportunities

- `crate::wallet_export::validate_watch_only*` — watch-only refusal class. Unchanged.
- `crate::wallet_export::pipeline::build_descriptor_string` (moved from `wallet_export.rs`) — canonical descriptor + miniscript-computed `#checksum`. Used by Specter + Bitcoin Core + BIP-388 + Green.
- `crate::slot_input::parse_slot_input` — shared `--slot` parser (per `SPEC_export_wallet_v0_7.md §2` architect R1-N3 attribution).
- `crate::slip0132::*` — SLIP-132 variant conversion. Used by Coldcard `_pub` + Electrum keystore xpub.
- `crate::network::CliNetwork` — coin-type derivation + `known_hrp`.
- `crate::template::CliTemplate` — BIP-44/49/84/86/multisig variants + `origin_path_str` + `is_multisig` predicate + `human_name`.
- `mk_codec::KeyCard` (xpub + fingerprint + path), `md_codec::compute_wallet_policy_id` — already used in `synthesize.rs`. No new codec API consumption needed; `EmitInputs` is built from already-resolved data.
- `miniscript::Descriptor<DescriptorPublicKey>::Display` — canonical descriptor with `#checksum` (per `SPEC_export_wallet_v0_7.md §4`).
- BIP-84 reference vectors (`slip0132.rs:169` `BIP84_REF_ZPUB`) — reusable for Coldcard `_pub` fixture cross-check.

## Verification

Automated, per-phase + final:

```fish
cd /scratch/code/shibboleth/mnemonic-toolkit
cargo build --workspace
cargo build --bin mnemonic
cargo test --workspace --no-fail-fast
cargo clippy --workspace -- -D warnings
cargo doc --workspace --no-deps
make -C docs/manual lint MNEMONIC_BIN="$(pwd)/target/debug/mnemonic" MD_BIN=true MS_BIN=true MK_BIN=mk
```

Expected: ~430 baseline tests + ~25 new (5 phases × ~5 tests each) = ~455 GREEN.

Manual smoke (after Phase 5):

```fish
# Coldcard singlesig
mnemonic export-wallet --format coldcard --template bip84 \
  --slot @0.xpub=zpub6Mu... --slot @0.fingerprint=ABCD1234 \
  --output /tmp/cc-generic.json
# Inspect /tmp/cc-generic.json — verify `xfp`, `xpub`, `bip84._pub` populated.

# Coldcard multisig (+ Jade byte-equal)
mnemonic export-wallet --format coldcard --template wsh-sortedmulti --threshold 2 \
  --slot @0.xpub=xpub6... --slot @0.fingerprint=ABCD1234 \
  --slot @1.xpub=xpub6... --slot @1.fingerprint=DEADBEEF \
  --slot @2.xpub=xpub6... --slot @2.fingerprint=CAFEBABE \
  --wallet-name "2-of-3 vault" --output /tmp/cc-multi.txt
mnemonic export-wallet --format jade --template wsh-sortedmulti --threshold 2 \
  --slot @0.xpub=xpub6... --slot @0.fingerprint=ABCD1234 \
  --slot @1.xpub=xpub6... --slot @1.fingerprint=DEADBEEF \
  --slot @2.xpub=xpub6... --slot @2.fingerprint=CAFEBABE \
  --wallet-name "2-of-3 vault" --output /tmp/jade-multi.txt
diff /tmp/cc-multi.txt /tmp/jade-multi.txt   # must be byte-identical

# Sparrow
mnemonic export-wallet --format sparrow --template bip84 \
  --slot @0.xpub=xpub6... --slot @0.fingerprint=abcd1234 \
  --wallet-name "Daily" --output /tmp/sparrow.json

# Specter
mnemonic export-wallet --format specter --template bip84 \
  --slot @0.xpub=xpub6... --slot @0.fingerprint=abcd1234 \
  --wallet-name "Daily" --output /tmp/specter.json

# Electrum singlesig + multisig
mnemonic export-wallet --format electrum --template bip84 \
  --slot @0.xpub=zpub6... --slot @0.fingerprint=abcd1234 \
  --output /tmp/electrum-single.json
mnemonic export-wallet --format electrum --template wsh-sortedmulti --threshold 2 \
  --slot @0.xpub=Zpub6... --slot @0.fingerprint=abcd1234 \
  --slot @1.xpub=Zpub6... --slot @1.fingerprint=deadbeef \
  --slot @2.xpub=Zpub6... --slot @2.fingerprint=cafebabe \
  --output /tmp/electrum-multi.json
# Verify /tmp/electrum-multi.json contains "wallet_type": "2of3" and three "xN/" keystores.

# Green
mnemonic export-wallet --format green --template bip84 \
  --slot @0.xpub=xpub6... --output /tmp/green.txt

# Missing-info smoke (no fingerprint supplied)
mnemonic export-wallet --format coldcard --template bip84 \
  --slot @0.xpub=xpub6... 2>&1
# Expect: error message listing master_fingerprint as missing; exit code 2; nothing on stdout.

# Watch-only refusal smoke (secret-bearing slot)
mnemonic export-wallet --format coldcard --template bip84 \
  --slot @0.entropy=... 2>&1
# Expect: REFUSAL_SECRET_INPUT byte-exact; exit code 2.
```

End-to-end with the bundle path (proves the new formats consume real bundle output):

```fish
mnemonic bundle --phrase "abandon abandon abandon ... about" --template bip84 --json | \
  mnemonic export-wallet --format coldcard --template bip84 --slot @0=- --output /tmp/cc-from-bundle.json
```

---

## Iterative-review log

- 2026-05-11 — Initial plan draft (post-brainstorm; user-locked decisions on integration / coverage / missing-info UX / phasing / version).
- 2026-05-11 — Architect review round R1 returned **2 Critical / 3 Important / 3 Low / 2 Nit**. Resolutions applied in-plan:
  - **C1 (Electrum `seed_version` authority).** The plan originally pinned `seed_version: 17` citing Coldcard's sample fixture as authoritative. Reviewer correctly noted Electrum's `FINAL_SEED_VERSION` is 71 on master, that Coldcard's sample is stale (Coldcard-generated, not Electrum-canonical), and that Electrum upgrades older seed_versions on load. Resolution: §9 reframed to defer the `seed_version` constant to a **Phase 4 step 0 spike** against current Electrum (>= 4.5.x) — toolkit produces what current Electrum produces for watch-only. Coldcard sample fixtures demoted to "structural reference only, not authoritative." `ELECTRUM_SEED_VERSION_PIN` becomes a spike-locked constant.
  - **C2 (WalletFormatEmitter return type / serialization ownership).** Reviewer noted the proposed trait returned `Vec<u8>` but existing Bitcoin Core / BIP-388 formatters return `serde_json::Value`; Phase 0 migration ambiguity. Resolution: §12 trait now returns `Result<String, ToolkitError>`; Phase 0 step 3 specifies `BitcoinCoreEmitter::emit` / `Bip388Emitter::emit` thin-wrap the moved functions with `serde_json::to_string_pretty(&value)`; Phase 0 step 4 specifies the `cmd::export_wallet::run` call-site is updated to `let emitted: String = match args.format { … }?; write_output(&args.output, emitted.as_bytes(), stdout)?;` — eliminating the ambiguity about who owns serialization.
  - **I1 (`ScriptType` enum module location).** Reviewer noted the existing `cmd::convert::ScriptType` has only single-sig variants (`P2wpkh`, `P2shP2wpkh`, `P2tr`) — confirmed at `cmd/convert.rs:224`. Resolution: §12 introduces a NEW `WalletScriptType` enum local to `crate::wallet_export` covering singlesig + multisig variants (`P2pkh`, `P2shP2wpkh`, `P2wpkh`, `P2tr`, `P2shMulti`, `P2shP2wshMulti`, `P2wshMulti`, `P2trMulti`). `cmd::convert::ScriptType` stays untouched (still single-sig-only for `(Xpub, Address)` edge). Phase 0 step 5 adds `WalletScriptType` + `script_type_from_template` + `script_type_from_descriptor`.
  - **I2 (Coldcard `bip86` sub-object fabricated).** Reviewer verified Coldcard's `generic-wallet-export.md` documents only `bip44` / `bip49` / `bip84` sub-objects; `bip86` is not in the upstream schema. Resolution: §5.1 now lists templates as bip44/bip49/bip84 only; `--template bip86 --format coldcard` REFUSES with a byte-exact pointer text; new FOLLOWUPS entry `coldcard-bip86-generic-export-pending-firmware` introduced. Fixture `coldcard_generic_bip86_mainnet.json` removed from §13; replaced with `coldcard_generic_bip44_mainnet.json` (legacy p2pkh coverage). New refusal fixture `coldcard_bip86_pending_firmware_refusal.stderr` added.
  - **I3 (Phase 1 deletes stub arms prematurely).** Reviewer noted Phase 1 step 6 deleted Sparrow/Specter stub arms but Phase 2/3 had not yet replaced them — would leave `unreachable!` arms live and panic on `--format sparrow` in the v0.7 production path. Resolution: Phase 1 step 6 reworded to keep stub arms untouched in Phase 1; Phase 2 step 4 deletes the Sparrow stub arm (replaced by real impl); Phase 3 step 4 deletes the Specter stub arm (replaced by real impl). v0.7 tests asserting clean Sparrow/Specter refusal continue to pass through Phase 1.
  - **I4 (`electrum-version-info-stderr` already resolved).** Reviewer confirmed at `FOLLOWUPS.md:895` the entry is `resolved 5dc83eb (v0.8 Phase 2)`. Resolution: removed from the plan header "Sibling FOLLOWUPS touched" line; replaced with the entries this cycle actually introduces (`coldcard-tr-multi-a-pending-firmware`, `coldcard-bip86-generic-export-pending-firmware`, `jade-tr-multi-a-pending-firmware`, `electrum-final-seed-version-drift`, `electrum-tr-multi-a-pending-libsecp-taproot`, `green-native-multisig-pending-server-support`).
  - **L1 (Specter `--wallet-name` required-or-not ambiguous).** Resolution: §13 + Phase 3 locked `--wallet-name` as REQUIRED for `--format specter` (UX rationale: Specter displays an empty label otherwise). Fixture `specter_missing_wallet_name_refusal.stderr` is no longer conditional. `collect_missing` returns `MissingField::WalletName` when absent.
  - **L2 (constant vs function ambiguity in §4).** Resolution: §4 clarified — `build_missing_fields_refusal` is the SOLE site of message construction; the `REFUSAL_<FORMAT>_MISSING_FIELDS_HEADER` constants exist for test-pinning the header literal only, and are read inside the builder via a `match format` block. They cannot drift by construction.
  - **L3 (Part C log format).** Resolution: this section now mirrors `IMPLEMENTATION_PLAN_v0_7.md:311-331`'s inline-bulleted-resolution-per-finding-code format. This R1 entry IS the new format.
  - **N1 (`format_stub_message` disposition).** Resolution: §12 module reorganization narrative explicitly states `format_stub_message` moves to `wallet_export/mod.rs` and stays there (used by the new per-format refusal arms in this cycle and any future stubs).
  - **N2 (mk-codec `Companion:` cross-cite unwarranted).** Reviewer noted `crate::slip0132` manipulates base58check bytes directly and does not cross the mk-codec API boundary. Resolution: Phase 4 step 8 reworded to drop the mk-codec companion cross-cite. The cross-cite was a planning artifact slip, not a real boundary impact.

  Additional hardening surfaced while folding R1:
  - **N3 (per-slot vs global field ordering specificity).** §4 expanded with an explicit deterministic-order rule for the case where missing-set contains both global fields (`Threshold`, `WalletName`) and per-slot fields (per-cosigner `MasterFingerprint`). Order: global-discriminant entries first in enum-discriminant order; per-slot entries in (enum-discriminant, slot-index) tuple order. Pinned in `multi_missing_fields_aggregate_refusal.stderr` (Phase 1).

- 2026-05-11 — Architect review round R2 (out-of-band check pending if user requests). Default convergence: R1 resolutions cover 0C/0I; plan ready for ExitPlanMode → execution begins at Phase 0.
- 2026-05-11 — Promoted from plan Part B (`/home/bcg/.claude/plans/we-need-to-make-recursive-pnueli.md`). Plan-level R1 resolutions folded inline above; awaiting IMPLEMENTATION_PLAN-level reviewer-loop R1.
- 2026-05-11 — IMPLEMENTATION_PLAN-level architect review **R1** returned **2 Critical / 7 Important / 2 Low / 1 Nit** (`design/agent-reports/v0_8-impl-plan-r1.md`). Resolutions folded inline:
  - **C-1.** Every per-phase exit gate ran `bash tests/lint.sh`, but no such file exists at repo root — the lint script lives at `docs/manual/tests/lint.sh` and requires positional `MNEMONIC_BIN=...` etc. arguments (it is called via `make -C docs/manual lint ...` in both CI workflows). The Phase 0 exit gate, Phase 1 step 8, Phase 6 final gauntlet, and Verification block all updated to `make -C docs/manual lint MNEMONIC_BIN="$(pwd)/target/debug/mnemonic" MD_BIN=true MS_BIN=true MK_BIN=mk`. (CLAUDE.md carries the same `tests/lint.sh` misstatement — a sibling cleanup, out of scope here.)
  - **C-2.** Phase 6 said "tag `mnemonic-toolkit-v0.8.X`" and `CHANGELOG.md [0.8.X]` entry — but `mnemonic-toolkit-v0.8.0` is ALREADY tagged (commit `7bb722a`, 2026-05-07, `[BREAKING]` cut closing 14 v0.8 FOLLOWUPS). The Context paragraph also called taproot-internal-key + electrum-version-info-stderr "in-flight v0.8 work," but both shipped in v0.8.0. Context rewritten to clarify this cycle is the **v0.8.1** additive cut; Phase 6 step 2 changed to `[0.8.1]`; Phase 6 step 5 changed to `mnemonic-toolkit-v0.8.1`.
  - **I-1 / I-3 (cross-cut with SPEC).** Line refs `wallet_export.rs:17-25` (off-by-7) and `cmd/export_wallet.rs:148-155`/`148-154` (inconsistent) corrected to `17-18` and `148-153` (with explicit per-phase sub-ranges: Sparrow 148-150, Specter 151-153, wildcard 154). Mirrored fix in SPEC.
  - **I-2.** Citation `slip0132.rs:138 BIP84_REF_ZPUB` was stale (inherited from `IMPLEMENTATION_PLAN_v0_7.md`); the declaration is at line **169** in current source. Corrected.
  - **I-4.** `MNEMONIC_BIN=true` in CI renders flag-coverage vacuous for `mnemonic` (per `FOLLOWUPS.md` entry `lint-md-flag-coverage-vacuous-with-md_bin-true`). Phase 0 + Phase 1 + Verification now build `target/debug/mnemonic` and pass it to `make lint` so local per-phase exit gates catch CLI↔manual drift before push; CI continues to use `true` as documented.
  - **I-5 (cross-cut with SPEC).** SPEC §13 table rows for `electrum_single.json` / `electrum_multi_2of4.json` still cited Coldcard's stale samples as Coverage authority. Folded into the SPEC (already-corrected §9 narrative now matches §13 table rows).
  - **I-6.** Phase 6 CHANGELOG entry lacked the v0.8.0 `[BREAKING]` precedent disambiguation. Phase 6 step 2 reworded to call out "No breaking changes" explicitly and cross-reference v0.8.0's `[BREAKING]` cut.
  - **I-7.** `wallet-export-industry-formats` FOLLOWUPS handling was contradictory across header line 5 ("resolved → partial-progress reopen"), Phase 1 step 10 ("partial-progress note"), Phase 5 step 6 ("Flip to RESOLVED"). The entry is already at `Status: resolved 3821f66` (v0.7 Phase 5 close); the FOLLOWUPS schema has no "reopen" state. All three references converted to a single coherent model: append `Resolution-extended (v0.8.1 Phase N):` notes to the already-resolved entry; never reopen, never re-flip status.
  - **L-1.** Phase 3 had two steps numbered `4.` (the second was the reviewer-loop step). Renumbered the reviewer-loop step to `5.`.
  - **L-2.** Phase 1 step 7 said only "Add `--wallet-name` clap flag." with no shape detail. Expanded to specify `Option<String>` with cross-flag-dependency defaults resolved post-parse, and the specter-required check in `SpecterEmitter::collect_missing` via a new `wallet_name_was_user_supplied: bool` field on `EmitInputs` (added in Phase 0 step 6).
  - **N-1.** Phase 4 step 0 spike instructions did not pin `--wallet_path`; Electrum 4.5.x's default-wallet-path varies. Updated the spike CLI to `electrum --offline restore <xpub> --wallet_path /tmp/electrum-spike-single.json` so the reader can locate the file deterministically.
- 2026-05-11 — IMPLEMENTATION_PLAN-level architect review **R2** verified all 12 R1 resolutions resolved (`design/agent-reports/v0_8-impl-plan-r2.md`). One new finding surfaced (0C/0I/1L/0N): **R2-L1** — Phase 0 step 6 listed `MissingField` / `build_missing_fields_refusal` / `ToolkitError` additions but did not mention `EmitInputs`, while Phase 1 step 7 cross-referenced "Phase 0 step 6 alongside the rest of `EmitInputs`". Phase 0 step 6 expanded to explicitly define `EmitInputs` per SPEC §12 with the `wallet_name_was_user_supplied: bool` field used by Phase 3's `SpecterEmitter::collect_missing`. Convergence: **0C/0I/0L/0N** after R2 fold applied.
