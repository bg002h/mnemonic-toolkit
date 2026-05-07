# mnemonic-toolkit v0.7 SPEC — `export-wallet` subcommand

**Version:** 0.7.0
**Date:** 2026-05-06
**Status:** DRAFT (converged 0C/0I after 3 user-rounds + 2 architect-rounds; ready for execution per `IMPLEMENTATION_PLAN_v0_7.md` Phase 5)
**Predecessors:** [SPEC_mnemonic_toolkit_v0_5.md](SPEC_mnemonic_toolkit_v0_5.md), [SPEC_convert_v0_6.md](SPEC_convert_v0_6.md).

## §1 Purpose

`mnemonic export-wallet` emits **watch-only** wallet artifacts in industry-standard formats consumable by reference wallet software (Bitcoin Core, hardware-wallet GUI clients, BIP-388 wallet-policy tooling). Inputs are restricted to public material (xpubs, fingerprints, paths, descriptors); secret-bearing slots (phrase, entropy, xprv, wif) are refused.

The subcommand is the spiritual successor to "give me an `importdescriptors` JSON for this xpub" — a frequent post-bundle ask that previously required hand-assembly. v0.7 ships Bitcoin Core 24+ JSON (default) and BIP-388 `wallet_policy` JSON; Sparrow / Specter formats are stubbed (return clean refusal).

## §2 Subcommand grammar

```
mnemonic export-wallet \
  --slot @N.<subkey>=<value> [--slot ...] \
  [--template <bip44|bip49|bip84|bip86|wsh-sortedmulti|...>] \
  [--descriptor <miniscript-descriptor>] \
  [--threshold <N>] \
  [--multisig-path-family <bip45|bip48|bip87>] \
  [--network <mainnet|testnet|signet|regtest>] \
  [--language <english|...>]                         # ignored (watch-only); kept for slot parser symmetry \
  [--format <bitcoin-core|bip388|sparrow|specter>]   # default: bitcoin-core \
  [--output <path|->]                                # default: - (stdout) \
  [--range <start,end>]                              # default: 0,999 (Bitcoin Core 24+ shape) \
  [--timestamp <unix|now>]                           # default: now \
  [--bitcoin-core-version <24|25>]                   # default: 25
```

Required: `--slot @N.xpub=...` (one or more); `--template` OR `--descriptor` (mutually exclusive per the existing `DESCRIPTOR_AND_TEMPLATE` guard from v0.5.1).

`--slot` parser is **shared** with `bundle` / `verify-bundle` via the `crate::slot_input::parse_slot_input` module (architect R1-N3 attribution clarification).

## §3 Watch-only refusal class

Slot inputs `phrase=` / `entropy=` / `xprv=` / `wif=` are REFUSED. Implementation: `crate::wallet_export::validate_watch_only(slots) -> Result<(), ToolkitError>` runs as a slot-set validator extension in the post-`resolve_slots` pipeline.

**Refusal stderr (byte-exact):**

```
error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret material, use 'mnemonic bundle'.
```

Exit code: 2 (`ToolkitError::ExportWalletSecretInput`). Detected before any descriptor parsing — the validator runs on the resolved-slot set, before `wallet_export::format_*` is invoked.

## §4 Descriptor pipeline (architect R1-C2 lock)

Descriptor checksum is **NOT** a public toolkit function. The export pipeline relies entirely on `miniscript::Descriptor<DescriptorPublicKey>::Display` to produce the canonical form with `#checksum` suffix:

1. **Parse:** template + slot xpubs (or raw `--descriptor`) → `miniscript::Descriptor<DescriptorPublicKey>` via the same path the existing `bundle` codepath uses (reuse `crate::parse_descriptor` module if applicable).
2. **Canonicalize:** call `descriptor.to_string()` — `miniscript`'s `Display` impl auto-appends the `#abcdef12` checksum suffix when the descriptor is well-formed (verified via spike against `rust-miniscript master`).
3. **Serialize:** dispatch to `format_bitcoin_core_importdescriptors` / `format_bip388_wallet_policy` / format-specific stubs per `--format`.

**No checksum function in the toolkit's public API.** The `miniscript` crate is the single source of truth for descriptor canonicalization. If `miniscript` ships an inert / stub descriptor (e.g., one with malformed key origin metadata), `to_string()` will either produce a non-canonical string OR panic — both signal a bug upstream that this SPEC defers to.

## §5 Bitcoin Core `importdescriptors` format (default)

Target: Bitcoin Core 24+ (default version: 25). Per architect R1-I9 hazard note, target version is locked at 24 minimum; minor JSON shape differences between 24 and 25 are exposed via `--bitcoin-core-version`.

**Output schema** (single-path descriptor):

```json
[
  {
    "desc": "wpkh([abcd1234/84h/0h/0h]xpub6.../<0;1>/*)#zzzzzzzz",
    "active": true,
    "internal": false,
    "range": [0, 999],
    "timestamp": "now"
  }
]
```

**Multi-path descriptor splitting (`<0;1>` syntax):** Bitcoin Core's `importdescriptors` requires separate entries for receive (`internal: false`) and change (`internal: true`) chains. The exporter splits a `<0;1>`-form descriptor into 2 entries:

```json
[
  {
    "desc": "wpkh([abcd1234/84h/0h/0h]xpub6.../0/*)#aaaaaaaa",
    "active": true,
    "internal": false,
    "range": [0, 999],
    "timestamp": "now"
  },
  {
    "desc": "wpkh([abcd1234/84h/0h/0h]xpub6.../1/*)#bbbbbbbb",
    "active": true,
    "internal": true,
    "range": [0, 999],
    "timestamp": "now"
  }
]
```

The receive/change checksums differ — each is computed by `miniscript`'s `Display` independently per the §4 pipeline.

**`--range` override (architect R1-I9):** default `0,999`. A user with > 999 addresses on the receive chain has invisible funds without override; the flag closes the gap. Range is half-open: `[start, end]` per Bitcoin Core's expected format.

**`--timestamp` override (architect R1-I9):** default `now`. Setting `--timestamp <unix>` triggers blocks-since-N rescan rather than a full rescan (Bitcoin Core wallet-rescan behavior; cite Bitcoin Core docs <https://github.com/bitcoin/bitcoin/blob/master/doc/release-notes/release-notes-23.0.md#updated-rpcs>).

**Reference: Bitcoin Core `importdescriptors` RPC documentation:** <https://bitcoincore.org/en/doc/24.0.0/rpc/wallet/importdescriptors/> (24.0); <https://bitcoincore.org/en/doc/25.0.0/rpc/wallet/importdescriptors/> (25.0).

## §6 BIP-388 `wallet_policy` format

Reference: BIP-388 §"Wallet policy descriptors" (<https://github.com/bitcoin/bips/blob/master/bip-0388.mediawiki>).

**Output schema:**

```json
{
  "name": "<descriptor-template-name or user-supplied>",
  "description_template": "wsh(sortedmulti(2,@0/**,@1/**,@2/**))",
  "keys_info": [
    "[fingerprint0/path0]xpub0...",
    "[fingerprint1/path1]xpub1...",
    "[fingerprint2/path2]xpub2..."
  ]
}
```

`description_template` uses the `@N` placeholder syntax per BIP-388 §"Key placeholders". `keys_info` is an array of fully-qualified key origin strings, one per `@N` placeholder, in slot-index order.

For single-sig templates (`wpkh`, `pkh`, `sh-wpkh`, `tr`), `keys_info` has length 1 and `description_template` references `@0/**`.

For multisig templates (`wsh-sortedmulti`, `wsh-multi`, `tr` with multi-key script-paths), each cosigner gets its own `@N` slot in slot-index order matching the toolkit's `--slot @0=... --slot @1=...` semantics.

## §7 Sparrow / Specter formats (stub)

`--format sparrow` and `--format specter` return a clean refusal:

```
error: --format <sparrow|specter> is deferred to v0.8 if user demand surfaces; use --format bitcoin-core or --format bip388 instead.
```

Exit code: 2 (`ToolkitError::NotSupported`). Stub returns the byte-exact stderr; no JSON emitted.

The HWI / Sparrow / Specter wallet-import JSON schemas are documented but heterogeneous (Sparrow's format is JSON-with-metadata; Specter's is a label dictionary plus descriptor). v0.7 punts on the design space pending demand signals.

## §8 Format priority

1. **Bitcoin Core `importdescriptors` (default).** Largest user base; reference RPC; canonical descriptor format upstream.
2. **BIP-388 `wallet_policy` (second).** Hardware-wallet-aligned; ledger / coldcard / passport ecosystem; growing adoption.
3. **Sparrow / Specter (deferred to v0.8+).** Single-vendor formats; design effort exceeds v0.7 scope.

The `--format <value>` enum order in clap reflects this priority (`bitcoin-core` first; `bip388` second; `sparrow` / `specter` last with stub refusals).

## §9 Test corpus (Phase 5 minimum, 6 cells)

Phase 5 RED tests cover, at minimum:

1. **Bitcoin Core importdescriptors round-trip with single-sig wpkh** (`--template bip84 --slot @0.xpub=zpub6Mu... --slot @0.fingerprint=...`).
2. **BIP-388 wallet_policy round-trip with multisig wsh-sortedmulti** (3 cosigner slots, threshold 2).
3. **Refusal stderr for `phrase=` slot input** (byte-exact; verifies §3 watch-only validator).
4. **Sparrow stub refusal stderr** (byte-exact; verifies §7).
5. **`--range 0,4999` override** exercised in Bitcoin Core format output.
6. **`--bitcoin-core-version 24` shape diff** vs. version 25 (if version 24 differs from 25 materially — confirm during impl; if no diff, document and reduce to a single-version test).

Each test pins descriptor checksums byte-exact (since `miniscript`'s `Display` is deterministic for a given input).

## §10 Implementation hooks

- `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` (~250 LOC): clap argument struct + `run()` dispatcher.
- `crates/mnemonic-toolkit/src/wallet_export.rs` (~250 LOC): format adapters (`format_bitcoin_core_importdescriptors`, `format_bip388_wallet_policy`, `format_sparrow_stub`, `format_specter_stub`) + `validate_watch_only` slot-set validator.
- Reuse: `bundle::resolve_slots` (post-v0.6.2; promoted to `pub(crate)` per architect R1-I3 in v0.5.1) handles slot resolution. `export-wallet` calls `resolve_slots` with the watch-only validator extension.

## §11 Out-of-scope for v0.7

- PSBT export / signing flows (per `bip174-psbt-signing` v1+ FOLLOWUP).
- `--format hwi` (Hardware Wallet Interface JSON) — defer to v0.8.
- `--format json` generic / `--format yaml` — defer; no demand signal.
- Slot-set validators beyond watch-only (e.g., "exactly N slots", "all slots have fingerprint subkey") — descriptor parser already enforces structural invariants downstream.
