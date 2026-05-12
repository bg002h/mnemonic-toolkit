# mnemonic-toolkit v0.8 SPEC — `export-wallet` multi-format expansion

**Version:** 0.8.0 (extension)
**Date:** 2026-05-11
**Status:** DRAFT (post-plan-R1; awaiting SPEC-level reviewer-loop)
**Predecessors:** [SPEC_export_wallet_v0_7.md](SPEC_export_wallet_v0_7.md)

## §1 Purpose

`mnemonic export-wallet` extends from emitting two reference-tooling formats (Bitcoin Core, BIP-388) to also emit six vendor-targeted wallet-import artifacts (Coldcard, Blockstream Jade, Sparrow, Specter, Electrum, Blockstream Green). All formats remain **watch-only by construction**; secret-bearing slots (`phrase=`, `entropy=`, `xprv=`, `wif=`) are refused at the validator layer before any emitter runs.

Scope boundary: this SPEC handles file emission only. No PSBT construction, no signing, no transaction broadcast, no address discovery beyond per-format "first receive address" fields, no balance lookup, no network I/O of any kind, no encryption-at-rest of emitted files. Vendor-firmware-version probing and seed-import (as opposed to xpub-import) are out of scope.

## §2 Subcommand grammar (extended)

```
mnemonic export-wallet \
  --slot @N.<subkey>=<value> [--slot ...] \
  [--template <bip44|bip49|bip84|bip86|wsh-sortedmulti|wsh-multi|sh-wsh-sortedmulti|sh-wsh-multi|tr-multi-a|tr-sortedmulti-a>] \
  [--descriptor <miniscript-descriptor>] \
  [--threshold <N>] \
  [--multisig-path-family <bip45|bip48|bip87>] \
  [--network <mainnet|testnet|signet|regtest>] \
  [--language <english|...>]                          # ignored (watch-only); kept for slot parser symmetry \
  [--format <bitcoin-core|bip388|coldcard|jade|sparrow|specter|electrum|green>]  # default: bitcoin-core \
  [--output <path|->]                                 # default: - (stdout) \
  [--range <start,end>]                               # default: 0,999 (Bitcoin Core 24+ shape) \
  [--timestamp <unix|now>]                            # default: now \
  [--bitcoin-core-version <24|25>]                    # default: 25 \
  [--wallet-name <STRING>]                            # default: <template-human-name>-<account> \
  [--taproot-internal-key <NUMS|@N>]                  # v0.8 carry-in (existing flag from v0.8 phase-1)
```

`--slot` parser is shared with `bundle` / `verify-bundle` via `crate::slot_input::parse_slot_input` (no change).

`--wallet-name` is **new** in this cycle: optional, defaults to `<template-human-name>-<account>` (e.g., `bip84-0`). Used as the wallet `name`/`label` field in Coldcard generic JSON, Sparrow, Specter, Electrum. Ignored by Bitcoin Core / BIP-388 / Jade text / Green (those formats have no name slot).

The `--format` enum gains six values; existing `bitcoin-core` and `bip388` remain default-priority. Stub arms for `sparrow` / `specter` at `src/cmd/export_wallet.rs:148-153` are removed incrementally per §12 (Phase 2 deletes the Sparrow stub; Phase 3 deletes the Specter stub).

## §3 Watch-only refusal class (unchanged)

Slot inputs `phrase=` / `entropy=` / `xprv=` / `wif=` continue to be REFUSED via `crate::wallet_export::validate_watch_only` and `validate_watch_only_resolved` (unchanged byte-exact refusal `REFUSAL_SECRET_INPUT` in `src/wallet_export.rs:17-18`). All new emitters call both validators before touching `EmitInputs`.

## §4 Missing-info refusal class (new)

When the resolved slot set or descriptor does not carry enough information to populate a target format's required fields, the emitter returns ONE byte-exact refusal that enumerates **every** missing field in deterministic order. Exit code 2. No partial JSON / text on stdout.

**Field enumeration order** (locked by `MissingField` enum discriminant):
1. `MasterFingerprint`
2. `DerivationPath`
3. `Xpub`
4. `ScriptType`
5. `Threshold`
6. `WalletName`
7. `IncompatibleFormatForTemplate` (always last; e.g., requesting `--format coldcard` with a multisig template that the generic JSON skeleton cannot represent)

`Account` and `Network` are NOT `MissingField` variants: both have clap defaults (`--account` default `0`, `--network` default `mainnet`), so the resolved `EmitInputs` is always populated for them and they cannot be missing.

Per-slot fields (e.g., per-cosigner missing fingerprint) are ordered by slot index ascending after the global enum-discriminant ordering.

**Refusal-message shape** (byte-exact):

```
error: mnemonic export-wallet --format <FORMAT> requires the following missing fields:
  - <field_name> for slot @<N> (<one-line explanation pointing at the supply mechanism>)
  - <field_name> for slot @<N> (...)
Re-invoke with all missing fields supplied.
```

**Implementation hooks:**
- `MissingField` enum + `build_missing_fields_refusal(format, &[MissingField]) -> String` in `crate::wallet_export` (the module root after the split — see §12). This function is the **sole** site of message construction: `user_text()` for the new error variant calls it directly and does NOT concatenate the header constant separately.
- `ToolkitError::ExportWalletMissingFields { format: &'static str, missing: Vec<MissingField> }` variant + `user_text()` arm in `src/error.rs`.
- One `REFUSAL_<FORMAT>_MISSING_FIELDS_HEADER` constant per format. These constants exist for **test-pinning only** (so a refusal-header byte-pin test can change without churning the dynamic-bullet portion). `build_missing_fields_refusal` reads the matching constant via `match format { … }` and prepends it. The constant + builder pair must agree by construction.

**Per-slot vs global field ordering:** when the missing set contains both global fields (e.g., `Threshold`, `WalletName`) and per-slot fields (e.g., per-cosigner `MasterFingerprint` for slots 0, 1, 2), the deterministic order is: ALL global-discriminant entries first in enum-discriminant order; THEN per-slot entries in (enum-discriminant, slot-index) tuple order. For example, a missing-set of {`Threshold` (global), `MasterFingerprint` for slots @0/@1, `DerivationPath` for slots @0/@1} emits in order: `Threshold`, `MasterFingerprint for slot @0`, `MasterFingerprint for slot @1`, `DerivationPath for slot @0`, `DerivationPath for slot @1` — globals first, then per-slot entries **grouped by enum discriminant, then ordered by slot index within each discriminant** (NOT interleaved across slots). Pin this ordering in `tests/export_wallet/multi_missing_fields_aggregate_refusal.stderr` (Phase 1).

## §5 Coldcard format (`--format coldcard`)

Two artifact flavors selected by the resolved template's multisig predicate:

### §5.1 Generic JSON skeleton (singlesig templates: bip44/bip49/bip84)

Format reference: <https://github.com/Coldcard/firmware/blob/master/docs/generic-wallet-export.md>.

```json
{
  "chain": "BTC",
  "xfp": "ABCD1234",
  "xpub": "xpub6...",
  "account": 0,
  "bip84": {
    "name": "p2wpkh",
    "deriv": "m/84'/0'/0'",
    "xfp": "DEADBEEF",
    "xpub": "xpub6...",
    "_pub": "zpub6...",
    "first": "bc1q..."
  }
}
```

- `chain`: `"BTC"` (mainnet), `"XTN"` (testnet/signet), `"XRT"` (regtest).
- `xfp`: top-level master fingerprint, 8 uppercase hex.
- `xpub`: top-level master xpub (BIP-32 base58).
- `account`: integer account index (0-indexed; clap default 0).
- Per-derivation sub-objects: `bip44` / `bip49` / `bip84` populated for the matching template only (single sub-object per emit). The upstream Coldcard `generic-wallet-export.md` documents these three sub-objects; **`bip86` is NOT in the upstream schema**. Until Coldcard firmware ships a documented `bip86` sub-object, `--template bip86 --format coldcard` REFUSES with the byte-exact pointer below (the emitted string has NO leading whitespace — the markdown-fenced-block indent under this bullet is presentation only):

  ```
  error: --format coldcard does not yet support BIP-86 (P2TR) — Coldcard's generic-wallet-export schema documents only bip44/bip49/bip84. Use --format bitcoin-core (descriptor) or --format sparrow for taproot watch-only setup.
  ```

  Tracked by FOLLOWUPS entry `coldcard-bip86-generic-export-pending-firmware` (introduced by this cycle).
- `_pub`: SLIP-132 variant matching script-type × network (`ypub`/`zpub`/`Upub`/`Vpub`/`vpub`/`upub`). Conversion via existing `crate::slip0132`.
- `first`: `m/<deriv>/0/0` address. Derived via `bitcoin::Address` constructors (`p2pkh`/`p2sh_wpkh`/`p2wpkh`).

### §5.2 Multisig text (multisig templates)

Format reference: <https://coldcard.com/docs/multisig> (Coldcard's published spec; the firmware repo does not host this doc under `docs/`); identical bytes accepted by Jade — see §6.

```
Name: <wallet-name, ≤20 chars>
Policy: <K> of <N>
Derivation: m/<shared-or-divergent>
Format: P2WSH | P2SH-P2WSH | P2SH
<XFP>: xpub6...
<XFP>: xpub6...
...
```

- `Name`: from `--wallet-name`, truncated to 20 chars.
- `Policy`: `<threshold> of <cosigner_count>`.
- `Derivation`: shared origin path if all cosigners share; for divergent paths use `m/0'/0'` (zeros-template per Coldcard convention) and rely on the per-cosigner `[fp/path]` embedded in canonical descriptor (Coldcard ingests both shapes — see source reference).
- `Format`: `P2WSH` (wsh / wsh-sortedmulti), `P2SH-P2WSH` (sh-wsh / sh-wsh-sortedmulti), `P2SH` (legacy multisig — not currently in toolkit templates; reserved).
- One `<XFP>: xpub` line per cosigner; XFP UPPERCASE 8-hex, xpub BIP-32 base58 (NOT SLIP-132). Order matches slot index for `multi(...)`; sorted by xpub lex for `sortedmulti(...)`.
- `tr-multi-a` / `tr-sortedmulti-a` templates: REFUSE with vendor-firmware-pending pointer; FOLLOWUPS entry `coldcard-tr-multi-a-pending-firmware` records the gate.

## §6 Jade format (`--format jade`)

Multisig: byte-identical to Coldcard's §5.2 multisig text (Jade's `register_multisig.multisig_file` alternative; reference: <https://github.com/Blockstream/Jade/blob/master/docs/index.rst>). Implementation delegates to the Coldcard multisig text emitter — see §12.

Singlesig: no native file-import surface (Jade selects address type on-device after seed restore). REFUSE singlesig with the byte-exact pointer below (no leading whitespace on the emitted line):

```
error: mnemonic export-wallet --format jade emits multisig wallet config only; for singlesig setups Jade reads the seed on-device. Use --format coldcard for a singlesig JSON or --format bitcoin-core for a descriptor.
```

Taproot multisig: REFUSE pending firmware (FOLLOWUPS entry `jade-tr-multi-a-pending-firmware`).

## §7 Sparrow format (`--format sparrow`)

Format reference: <https://github.com/sparrowwallet/drongo/blob/master/src/main/java/com/sparrowwallet/drongo/wallet/Wallet.java> (canonical model used by Sparrow's wallet-import path).

```json
{
  "name": "<wallet-name>",
  "network": "mainnet",
  "policyType": "SINGLE",
  "scriptType": "P2WPKH",
  "defaultPolicy": {
    "name": "Default",
    "miniscript": { "script": "wpkh(@0/**)" }
  },
  "keystores": [
    {
      "label": "<wallet-name>",
      "source": "SW_WATCH",
      "walletModel": "SPARROW",
      "keyDerivation": {
        "masterFingerprint": "abcd1234",
        "derivation": "m/84'/0'/0'"
      },
      "extendedPublicKey": "xpub6..."
    }
  ]
}
```

- `network`: `"mainnet"` / `"testnet"` / `"signet"` / `"regtest"` (Sparrow accepts these exact strings).
- `policyType`: `"SINGLE"` (singlesig templates) / `"MULTI"` (multisig templates).
- `scriptType`: `"P2PKH"` / `"P2SH_P2WPKH"` / `"P2WPKH"` / `"P2TR"` (singlesig) or `"P2SH"` / `"P2SH_P2WSH"` / `"P2WSH"` (multisig).
- `defaultPolicy.miniscript.script`: `wpkh(@0/**)` (singlesig wpkh) / `wsh(sortedmulti(K, @0/**, @1/**, ...))` (multisig wsh-sortedmulti) / `wsh(multi(K, @0/**, ...))` (multisig wsh-multi) / `tr(@0/**)` (singlesig p2tr); the per-template miniscript expression is derived from `EmitInputs.script_type` + `threshold` + cosigner count. Threshold is conveyed implicitly via the `multi(K, ...)` / `sortedmulti(K, ...)` argument count — Sparrow's `Policy` class (`Policy.java`) has only two serialized fields (`name`, `miniscript`); `numSignaturesRequired` is a derived getter, NOT a JSON field, and must not appear in the emitted shape.
- `keystores`: 1 element for `SINGLE`, K elements for `MULTI` (one per cosigner, slot-index order).
- `masterFingerprint`: lowercase 8-hex (Sparrow convention).
- `extendedPublicKey`: BIP-32 xpub form (Sparrow refuses SLIP-132 in import path).

Taproot multisig (`tr-multi-a` / `tr-sortedmulti-a`): supported by Sparrow as descriptor-passthrough; emit `miniscript.script` directly from canonical descriptor. Rides on existing v0.8 taproot-internal-key flag.

## §8 Specter Desktop format (`--format specter`)

Format reference: <https://github.com/cryptoadvance/specter-desktop/blob/master/src/cryptoadvance/specter/util/wallet_importer.py> (canonical import-shape authority — the REST GET schema at <https://docs.specter.solutions/desktop/api/ep_wallets_wallet/> documents a different shape and is not authoritative for file-import).

```json
{
  "label": "<wallet-name>",
  "blockheight": 0,
  "descriptor": "wpkh([abcd1234/84h/0h/0h]xpub6.../<0;1>/*)#zzzzzzzz",
  "devices": ["unknown"]
}
```

- `label`: from `--wallet-name`.
- `blockheight`: 0 by default; future flag `--blockheight <N>` deferred to FOLLOWUPS.
- `descriptor`: canonical BIP-380 descriptor with `#checksum` suffix, produced via the existing miniscript Display pipeline (`SPEC_export_wallet_v0_7.md §4`).
- `devices`: array of vendor strings; length matches cosigner count for multisig. Toolkit emits `"unknown"` placeholders since cosigner-vendor metadata is not threaded through the codecs.

Taproot multisig: supported as descriptor-passthrough (same as Sparrow).

## §9 Electrum format (`--format electrum`)

Format reference: <https://github.com/spesmilo/electrum/blob/master/electrum/wallet_db.py> (authoritative schema; `FINAL_SEED_VERSION` is currently `71` on master).

**`seed_version` policy.** Electrum's `wallet_db.py` upgrades wallet files in place on load, walking each `_convert_version_<N>` migration. Older `seed_version` values are accepted; the loader applies migrations and rewrites the file on next save. The toolkit emits the **minimum `seed_version` that current Electrum (>= 4.5.x) imports cleanly for watch-only wallets** — to be locked by a Phase 4 spike (read-only; produce a reference watch-only wallet via Electrum's CLI, observe the `seed_version` value it writes, lock the constant). The Coldcard sample fixtures at `firmware/docs/sample-electrum-wallets/` are NOT authoritative for the toolkit (they are Coldcard-generated stale files); they remain useful as a structural reference only. The `ELECTRUM_SEED_VERSION_PIN` constant has a doc-comment citing the Phase 4 spike report. FOLLOWUPS entry `electrum-final-seed-version-drift` tracks ongoing upstream drift.

### §9.1 Singlesig

```json
{
  "seed_version": <ELECTRUM_SEED_VERSION_PIN>,
  "wallet_type": "standard",
  "use_encryption": false,
  "keystore": {
    "type": "bip32",
    "xpub": "zpub6...",
    "derivation": "m/84'/0'/0'",
    "root_fingerprint": "abcd1234",
    "label": "<wallet-name>"
  }
}
```

- `seed_version`: `ELECTRUM_SEED_VERSION_PIN` (locked by Phase 4 spike against current Electrum >= 4.5.x; see Phase 4 step 0 below). FOLLOWUPS entry `electrum-final-seed-version-drift` tracks upstream drift.
- `wallet_type`: `"standard"` for singlesig.
- `keystore.type`: `"bip32"` (toolkit emits xpub-based watch-only; never `"hardware"` since we don't know which HW).
- `keystore.xpub`: **SLIP-132 form** matching script-type × network (`vpub`/`zpub`/`upub`/`ypub`/`tpub`). Conversion via `crate::slip0132`.
- `keystore.derivation`: BIP-32 origin path as string.
- `keystore.root_fingerprint`: lowercase 8-hex.

### §9.2 Multisig

```json
{
  "seed_version": <ELECTRUM_SEED_VERSION_PIN>,
  "wallet_type": "2of3",
  "use_encryption": false,
  "x1/": { "type": "bip32", "xpub": "Zpub6...", "derivation": "m/48'/0'/0'/2'", "root_fingerprint": "abcd1234", "label": "<wallet-name>-1" },
  "x2/": { "type": "bip32", "xpub": "Zpub6...", "derivation": "m/48'/0'/0'/2'", "root_fingerprint": "deadbeef", "label": "<wallet-name>-2" },
  "x3/": { "type": "bip32", "xpub": "Zpub6...", "derivation": "m/48'/0'/0'/2'", "root_fingerprint": "cafebabe", "label": "<wallet-name>-3" }
}
```

- `wallet_type`: `"<K>of<N>"` (e.g., `"2of3"`).
- One `"xN/"` keystore per cosigner; key naming is `"x1/"` ... `"xN/"` in slot-index order.
- Each cosigner xpub in SLIP-132 multisig form (`Zpub` for mainnet wsh; `Vpub` for testnet/signet wsh; `Ypub`/`Upub` for sh-wsh). Conversion via `crate::slip0132`.

Taproot multisig: REFUSE pending Electrum libsecp-taproot support (FOLLOWUPS entry `electrum-tr-multi-a-pending-libsecp-taproot`).

## §10 Blockstream Green format (`--format green`)

Green has no native descriptor-import file shape; the Help Center documents pasting the descriptor or xpub into Green's "Import from file" dialog. Reference: <https://help.blockstream.com/hc/en-us/articles/19340800530713-Set-up-watch-only-wallet> (Zendesk-hosted; programmatic fetchers may receive 403, verify in a browser).

Toolkit emits a thin 3-line text file:

```
# Blockstream Green — Watch-only import (singlesig)
# Help: https://help.blockstream.com/hc/en-us/articles/19340800530713-Set-up-watch-only-wallet
<canonical-descriptor-or-xpub>
```

- Descriptor preferred; falls back to xpub if `--descriptor` was supplied without a template.
- Multisig: REFUSE with pointer at Green's server-mediated multisig surface (FOLLOWUPS `green-native-multisig-pending-server-support`).

## §11 Format priority + default

Priority order in the `--format` clap enum:

1. `bitcoin-core` (default; v0.7 baseline)
2. `bip388` (v0.7 baseline)
3. `coldcard`
4. `jade`
5. `sparrow`
6. `specter`
7. `electrum`
8. `green`

Default unchanged: `bitcoin-core`. The new variants appear after the existing pair to preserve the v0.7 stable order.

## §12 Module reorganization

`src/wallet_export.rs` is 442 LOC today (v0.7), mixing the descriptor pipeline, validators, and two formatters. Adding six more emitters in-file pushes it past 1500 LOC. Split into a submodule:

```
crates/mnemonic-toolkit/src/wallet_export/
    mod.rs            # re-exports + REFUSAL_* constants + validate_watch_only*
                      # + EmitInputs struct + WalletFormatEmitter trait
                      # + MissingField enum + build_missing_fields_refusal
                      # + TaprootInternalKey (existing)
    pipeline.rs       # build_descriptor_string, descriptor_to_bip388_wallet_policy (moved)
    bitcoin_core.rs   # format_bitcoin_core_importdescriptors (moved)
    bip388.rs         # format_bip388_wallet_policy (moved)
    coldcard.rs       # emit_coldcard_generic_json + emit_coldcard_multisig_text
    jade.rs           # emit_jade_multisig_text (delegates to coldcard text emitter)
    sparrow.rs        # emit_sparrow_wallet_json
    specter.rs        # emit_specter_wallet_json
    electrum.rs       # emit_electrum_standard_json + emit_electrum_multisig_json
    green.rs          # emit_green_descriptor_text
```

Migration is a file-move + extractions. All `pub(crate)` symbols keep their names; no callsite churn outside the submodule. The existing `format_stub_message` helper moves to `wallet_export/mod.rs` and stays there (still used by future stubs and by the `bip86`/`tr-multi-a` per-format refusal arms introduced in this cycle).

**Shared trait** (in `wallet_export/mod.rs`). Return type is `String` (all six new formats and the two existing formats produce text). The existing `format_bitcoin_core_importdescriptors` and `format_bip388_wallet_policy` functions today return `serde_json::Value`; Phase 0 thin-wraps them as trait impls that call `serde_json::to_string_pretty(&value).map_err(...)?` and return the resulting `String`. The pretty-print indentation matches v0.7's existing call-site at `cmd::export_wallet::run` (which today does `let serialized = serde_json::to_string_pretty(&value)?; writeln!(stdout, "{serialized}")`). Phase 0 deletes the call-site pretty-print and writes `stdout.write_all(emitted.as_bytes())?` instead. The byte-exact v0.7 fixtures for Bitcoin Core and BIP-388 remain valid (pretty-print is deterministic for a given `Value` input).

```rust
pub(crate) trait WalletFormatEmitter {
    // collect_missing: per-format predicate (which fields does THIS format require?).
    // build_missing_fields_refusal (free fn in mod.rs): cross-format formatter that
    // turns the collected list into the byte-exact refusal text per §4.
    fn collect_missing(inputs: &EmitInputs) -> Vec<MissingField>;
    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError>;
    fn extension() -> &'static str;
}
```

**`EmitInputs`** is built once in `cmd::export_wallet::run` after template + slot resolution and watch-only validation:

```rust
pub(crate) struct EmitInputs<'a> {
    pub canonical_descriptor: &'a str,        // canonical with #checksum
    pub resolved_slots: &'a [ResolvedSlot],   // xpub + fingerprint + path per slot
    pub template: Option<CliTemplate>,
    pub script_type: WalletScriptType,        // see "WalletScriptType" note below
    pub network: CliNetwork,
    pub account: u32,
    pub threshold: Option<u8>,                // multisig only
    pub wallet_name: &'a str,                 // resolved (template-default if no --wallet-name)
    pub taproot_internal_key: Option<TaprootInternalKey>,
    pub range: (u32, u32),
    pub timestamp: TimestampArg,
    pub bitcoin_core_version: u8,
}
```

**`WalletScriptType`**. The existing `crate::cmd::convert::ScriptType` enum has only three single-sig variants (`P2wpkh`, `P2shP2wpkh`, `P2tr`) — confirmed by reading `cmd/convert.rs:224`. It is intentionally narrow because `convert`'s `(Xpub, Address)` edge only supports single-sig address derivation. The new emitters need a richer enum covering single AND multisig:

```rust
// crates/mnemonic-toolkit/src/wallet_export/mod.rs
pub(crate) enum WalletScriptType {
    P2pkh,             // bip44
    P2shP2wpkh,        // bip49
    P2wpkh,            // bip84
    P2tr,              // bip86 (singlesig only — refused for Coldcard per §5.1)
    P2shMulti,         // legacy multisig (sh-multi)
    P2shP2wshMulti,    // sh-wsh-multi / sh-wsh-sortedmulti
    P2wshMulti,        // wsh-multi / wsh-sortedmulti
    P2trMulti,         // tr-multi-a / tr-sortedmulti-a (Sparrow + Specter only this cycle)
}

pub(crate) fn script_type_from_template(t: &CliTemplate) -> WalletScriptType { ... }
pub(crate) fn script_type_from_descriptor(d: &Descriptor<DescriptorPublicKey>) -> Result<WalletScriptType, ToolkitError> { ... }
```

`WalletScriptType` is local to `crate::wallet_export` and does NOT collide with `cmd::convert::ScriptType` (which keeps its single-sig-only role). No cross-module rename is needed. Phase 0 adds `WalletScriptType` + the two derivation functions; no existing callsites are affected.

Format-emitter dispatch in `cmd::export_wallet::run`:

```rust
let emitted: String = match args.format {
    CliExportFormat::BitcoinCore => BitcoinCoreEmitter::emit(&inputs),
    CliExportFormat::Bip388      => Bip388Emitter::emit(&inputs),
    CliExportFormat::Coldcard    => ColdcardEmitter::emit(&inputs),
    CliExportFormat::Jade        => JadeEmitter::emit(&inputs),
    CliExportFormat::Sparrow     => SparrowEmitter::emit(&inputs),
    CliExportFormat::Specter     => SpecterEmitter::emit(&inputs),
    CliExportFormat::Electrum    => ElectrumEmitter::emit(&inputs),
    CliExportFormat::Green       => GreenEmitter::emit(&inputs),
}?;
write_output(&args.output, emitted.as_bytes(), stdout)?;
```

The existing stub arms for `Sparrow` / `Specter` at `src/cmd/export_wallet.rs:148-153` are removed incrementally: Phase 2 deletes the `Sparrow` stub arm; Phase 3 deletes the `Specter` stub arm. Phase 1 does NOT delete them. Between phases, the stub for the not-yet-shipped format continues to return its v0.7 byte-exact refusal so callers see a clean error rather than a panic.

## §13 Test corpus

Per-format byte-exact fixtures pinned under `tests/export_wallet/`:

| File | Phase | Format | Template | Coverage |
|---|---|---|---|---|
| `coldcard_generic_bip84_mainnet.json` | 1 | Coldcard JSON | bip84 mainnet | singlesig wpkh, SLIP-132 zpub |
| `coldcard_generic_bip49_testnet.json` | 1 | Coldcard JSON | bip49 testnet | sh-wpkh, SLIP-132 upub |
| `coldcard_generic_bip44_mainnet.json` | 1 | Coldcard JSON | bip44 mainnet | legacy p2pkh |
| `coldcard_multisig_2of3_wsh.txt` | 1 | Coldcard text | wsh-sortedmulti | 2-of-3 multisig |
| `jade_multisig_2of3_wsh.txt` | 1 | Jade text | wsh-sortedmulti | byte-equal to coldcard |
| `sparrow_single_wpkh.json` | 2 | Sparrow JSON | bip84 | singlesig |
| `sparrow_multi_2of3_wsh_sortedmulti.json` | 2 | Sparrow JSON | wsh-sortedmulti | multisig |
| `sparrow_single_tr.json` | 2 | Sparrow JSON | bip86 | p2tr singlesig |
| `specter_single_wpkh.json` | 3 | Specter JSON | bip84 | singlesig |
| `specter_multi_2of3.json` | 3 | Specter JSON | wsh-sortedmulti | multisig |
| `electrum_single.json` | 4 | Electrum JSON | bip84 | pinned to Phase 4 step 0 spike-observed byte shape (singlesig) |
| `electrum_multi_2of4.json` | 4 | Electrum JSON | wsh multi 2-of-4 | pinned to Phase 4 step 0 spike-observed byte shape (2-of-4 multisig) |
| `green_descriptor.txt` | 5 | Green text | bip84 | thin descriptor file |

Refusal-text fixtures (one per phase, byte-exact):

- `coldcard_missing_xfp_refusal.stderr` (Phase 1) — single missing field
- `coldcard_multisig_template_skeleton_mismatch_refusal.stderr` (Phase 1) — IncompatibleFormatForTemplate
- `coldcard_bip86_pending_firmware_refusal.stderr` (Phase 1) — §5.1 bip86 refusal text
- `jade_singlesig_refusal.stderr` (Phase 1)
- `jade_tr_multi_a_refusal.stderr` (Phase 1)
- `sparrow_missing_threshold_refusal.stderr` (Phase 2)
- `electrum_tr_multi_a_refusal.stderr` (Phase 4)
- `green_multisig_refusal.stderr` (Phase 5)
- `multi_missing_fields_aggregate_refusal.stderr` (Phase 1) — exercises 3+ missing fields across global + per-slot to verify deterministic ordering per §4

**Specter `--wallet-name` requirement:** locked **required** for `--format specter` (Specter's UX requires a wallet label; emitting a Specter wallet without a label produces a wallet that displays as an empty string in the Specter UI, which is a UX regression vs. the user's likely intent). Phase 3 RED test pins `specter_missing_wallet_name_refusal.stderr`. The fixture row is no longer conditional.

Round-trip helpers (where format is structured enough to invert):

- `tests/helpers/coldcard_parse.rs` — parses Coldcard generic JSON and multisig text back into `xfp`/`xpub`/`deriv`/cosigner-list for equality assertion.
- `serde_json::Value` deep-equality for all JSON formats (compare emitter output to golden file).
- For Electrum SLIP-132 round-trip: supply `--slot @0.xpub=vpub...`, verify emitted `keystore.xpub` matches the script-type-correct variant.

## §14 Out-of-scope (reaffirmed)

This cycle does NOT add: PSBT construction or signing, transaction broadcast, address discovery beyond per-format `first` fields, balance lookup, encryption-at-rest of emitted files (no `.mv.db` / no AES envelope), vendor-firmware-version probing, network I/O of any kind, seed-import-style files (we only emit xpub-based watch-only artifacts), descriptor → BIP-388 placeholder re-parsing (the existing `export-wallet-descriptor-bip388-interop` FOLLOWUPS entry stays deferred), SLIP-39 / Liquid / MuSig2 / FROST / vault covenants (existing FOLLOWUPS classes remain deferred).

The watch-only refusal class (§3) is the trip-wire: no entropy / phrase / xprv / wif material is ever rendered into any of the new formats, by construction at the validator layer and reinforced at the emitter layer.

---

## Iterative-review log (SPEC-level)

This section records SPEC-level reviewer-loop rounds. Plan-level R1 resolutions were folded silently during promotion; the audit trail lives in `IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md`'s own `## Iterative-review log`. SPEC-level reviewer-loop rounds are recorded here.

- 2026-05-11 — Promoted from plan Part A (`/home/bcg/.claude/plans/we-need-to-make-recursive-pnueli.md`).
- 2026-05-11 — SPEC-level architect review **R1** returned **2 Critical / 6 Important / 2 Low / 1 Nit** (`design/agent-reports/v0_8-spec-r1.md`). Resolutions folded inline:
  - **C-1.** §5.2 Coldcard multisig URL `github.com/Coldcard/firmware/.../docs/multisig-wallets.md` is a 404; that doc does not exist in the firmware repo. Re-cited to `https://coldcard.com/docs/multisig` (Coldcard's published spec).
  - **C-2 (cross-cut with IMPLEMENTATION_PLAN R1 I-1).** §3 line-ref `src/wallet_export.rs:17-25` was off-by-7 (the constant occupies lines 17-18; lines 20-25 are `format_stub_message`). Corrected to `17-18` (mirrored in IMPLEMENTATION_PLAN).
  - **I-1.** §7 Sparrow JSON+bullets emitted a spurious `defaultPolicy.numSignaturesRequired` field; Sparrow's `Policy.java` has only `name` and `miniscript` as serialized fields and `numSignaturesRequired` is a derived getter. Field removed; bullet rewritten to note that threshold is implicit in the miniscript `multi(K,...)` argument count.
  - **I-2.** §7 used a fabricated token `wpkh(bip39)`; no such literal exists in Sparrow's source. Replaced with `wpkh(@0/**)` and used `@N/**` placeholder syntax consistently across the singlesig + multisig miniscript bullet.
  - **I-3.** §8 cited the Specter REST GET schema URL as authoritative for the import shape; the GET response has a different shape than the import file. Re-cited to `src/cryptoadvance/specter/util/wallet_importer.py` (canonical import-shape authority); kept the REST URL as a "different shape" cross-reference.
  - **I-4 / I-3 (cross-cut with IMPLEMENTATION_PLAN).** §2 and §12 inconsistently cited stub-arm line range as `148-154` and `148-155`. Walked the current source: Sparrow arm 148-150, Specter arm 151-153, wildcard 154, match-close 155. Normalized all citations to `148-153` and clarified that the wildcard at 154 and match-close at 155 remain after the per-phase stub deletions.
  - **I-5.** §2 said stub arms are "deleted" (no phase qualifier); §12 said they "remain until each format's phase replaces them" with `Phase 1 does NOT delete them`. Direct contradiction. §2 reworded to defer the deletion narrative to §12's incremental model (Phase 2 deletes Sparrow; Phase 3 deletes Specter).
  - **I-6.** §4 `MissingField` enum listed `Account` and `Network` as variants, but both have clap defaults and can never be missing. Removed those two variants (enum shrinks from 9 to 7); added a paragraph documenting why `Account` and `Network` are NOT `MissingField` variants.
  - **I-7.** §4 refusal-shape ended with `Re-invoke with all missing fields supplied. (exit 2)` — embedding the exit code in stderr text violates the v0.7 §3 precedent (exit code is process-status metadata, not message text). Removed ` (exit 2)`; the `Exit code 2.` callout earlier in §4 already documents the exit code.
  - **L-1.** Green Help-Center URL returns 403 to non-browser clients. Added a note clarifying the URL is Zendesk-hosted and should be verified in a browser.
  - **L-2.** Stripped `(R1-XN hardening)` inline parentheticals from 8 sites — the v0.7 SPEC convention is silent fold-in; the resolutions are normative regardless of which round produced them, and the audit trail lives in this log section.
  - **L-3.** §5.1 + §6 fenced refusal blocks have ambiguous indentation in markdown (under a bullet vs. column 0). Added "no leading whitespace on the emitted text" clarifying prose just before each fenced block.
  - **N-1.** Added inline comment to the `WalletFormatEmitter` trait block distinguishing `collect_missing` (per-format predicate) from `build_missing_fields_refusal` (cross-format formatter).
  - Cross-fold from IMPLEMENTATION_PLAN R1 I-5: §13 fixture-table rows for `electrum_single.json` / `electrum_multi_2of4.json` still cited Coldcard's stale sample fixtures as Coverage authority; corrected to "pinned to Phase 4 step 0 spike-observed byte shape" to match §9's already-corrected narrative.
- 2026-05-11 — SPEC-level architect review **R2** verified all 13 R1 resolutions resolved (`design/agent-reports/v0_8-spec-r2.md`). Two new findings surfaced (0C/0I/1L/1N): **L-4** — §4 per-slot ordering paragraph used "interleaved" terminology that contradicted the body's `(enum-discriminant, slot-index)` tuple-order rule and the example, which is actually grouped-by-discriminant. Reworded to "grouped by enum discriminant, then ordered by slot index within each discriminant (NOT interleaved across slots)" and extended the example to include a global `Threshold` entry so the global-first rule is observable. **N-2** — earlier `**C-2 / I-1 (cross-cut).**` review-log label conflated two unrelated R1 findings; the line-ref correction is mirrored in IMPLEMENTATION_PLAN R1 (not in SPEC I-1, which is the Sparrow `numSignaturesRequired` removal). Relabeled to `**C-2 (cross-cut with IMPLEMENTATION_PLAN R1 I-1).**`. Convergence: **0C/0I/0L/0N** after R2 folds applied.
