# SPEC — `mnemonic import-wallet` (v0.26.0)

**Status:** Phase 0 — SPEC author + R0 reviewer-loop.
**Cycle:** v0.26.0 (toolkit minor bump + `mnemonic-gui` v0.11.0 lockstep).
**Predecessor:** v0.25.1 (`7c1f874`, 2026-05-18).
**Brainstorm companion:** [`BRAINSTORM_wallet_import_v0_26_0.md`](BRAINSTORM_wallet_import_v0_26_0.md).
**External authorities:**
- [BIP-129 BSMS specification](https://github.com/bitcoin/bips/blob/master/bip-0129.mediawiki)
- [BIP-380 Output Script Descriptors](https://github.com/bitcoin/bips/blob/master/bip-0380.mediawiki)
- [BIP-389 Multipath descriptors](https://github.com/bitcoin/bips/blob/master/bip-0389.mediawiki)
- Bitcoin Core `listdescriptors` RPC (Core ≥ 0.21)

---

## §1 Purpose

Add `mnemonic import-wallet` subcommand: ingest third-party wallet blobs (BSMS Round-2 + Bitcoin Core `listdescriptors` xpub-only form) and emit toolkit bundle(s) (`ms1` ×N + `mk1` ×N + `md1` ×1). Inverse direction of the v0.7..v0.8.1 `mnemonic export-wallet` surface; first cycle of a multi-cycle multi-format initiative.

Also introduces a cross-cutting CLI value-source sentinel `@env:<VAR>` for all 6 inline-value secret-bearing flags (`--passphrase`, `--bip38-passphrase`, `--ms1`, `--share`, plus slot-subkey forms `--slot @N.phrase=`, `--slot @N.ms1=`).

## §2 Functional surface

### §2.1 CLI

```
mnemonic import-wallet [OPTIONS]

OPTIONS:
  --blob <FILE|->                                             (required)  third-party wallet blob; `-` reads from stdin
  --format <bsms|bitcoin-core>                                (optional)  format override; default = auto-detect via sniff
  --ms1 <STRING>                                              (repeatable, positional cosigner-index) seed overlay
  --slot @<N>.phrase=<STRING>                                 (existing slot-subkey pattern) per-cosigner phrase
  --select-descriptor <N|active-receive|active-change|all>    (default `all`) multi-descriptor selector
  --json                                                      emit JSON envelope array on stdout
  --no-auto-repair                                            (global, auto-attached) suppress repair short-circuit
  --help / -h
```

Any inline-value secret flag accepts `@env:<VAR>` sentinel resolved at clap-parse time (§3).

### §2.2 Default output

- **Stdout (default mode):** human-readable engraving card(s) — exactly the byte-shape produced by `mnemonic synthesize`. When N > 1 descriptors emit N bundles, cards are separated by a single line `;` (newline + literal semicolon + newline; i.e., `\n;\n`).
- **Stdout (`--json` mode):** JSON array of bundle envelopes, one per emitted bundle. Each envelope includes:
  - `bundle: {...}` — **v0.26.0 ships a parse-side summary** (per Phase 5 R0 I2 fold) of the shape `{ cosigners: [{fingerprint, path_raw, xpub, has_entropy}], network, threshold }`. The full toolkit-native `BundleJson` shape (the one `verify-bundle --bundle-json` consumes, with synthesized ms1/mk1/md1 cards) requires running the synthesizer post-parse and is tracked at FOLLOWUP `wallet-import-json-envelope-full-bundle` (v0.27+). Consumers writing against v0.26.0 should encode against the summary shape; the envelope key remains `bundle`, the shape itself extends in v0.27.
  - `roundtrip: { byte_exact: bool, semantic_match: bool, diff: Option<String>, status: "ok" | "blocked_no_emitter" | "canonicalize_failed", error?: String }` — `status` is the v0.26.0 envelope-extension key (per Phase 5 R0): `"ok"` for the standard case, `"blocked_no_emitter"` for BSMS until FOLLOWUP `wallet-export-bsms-emitter` lands, `"canonicalize_failed"` when canonicalize errors post-parse-success. **v0.27.1 amendment (per FOLLOWUP `pr-26-roundtrip-warning-suppression`):** `error: String` field added in the `canonicalize_failed` branch only (omitted in other status values); carries the typed `ToolkitError` Display form. See §7.4.
  - `bsms_audit: { token, signature, first_address, derivation_path, signature_verified: false }` (BSMS only; absent for Core).
  - `source_format: "bsms" | "bitcoin-core"`.
- **Stderr:** progress / NOTICEs / WARNINGs / round-trip diff (when bytes differ and `--json` is NOT set).

### §2.3 Exit codes

Per `error.rs:296-328` tier discipline (D17 in brainstorm):

| Exit | Variant(s) | Trigger |
|---|---|---|
| 1 | `ImportWalletAmbiguousFormat`, `ImportWalletFormatMismatch`, `EnvVarMissing` (cross-cutting) | User-input/generic |
| 2 | `ImportWalletParse`, `ImportWalletXprvForbidden`, `ImportWalletWatchOnlyViolation` | Format-violation/refusal |
| 3 | (existing) `FutureFormat` via From-impl | BSMS 2.0+ |
| 4 | `ImportWalletSeedMismatch` | Supplied seed ↮ blob xpub at declared path |
| 5 | (existing) repair short-circuit | BCH-correctable BSMS descriptor mk1 chunk |

ToolkitError variant naming: no `Error` suffix (matches `DescriptorParse`/`ConvertRefusal`).

### §2.4 Stderr messages (normative templates)

| Class | Template |
|---|---|
| WARNING (exit 0) | `warning: import-wallet: bsms: 2-line excerpt; full BIP-129 Round-2 carries token + signature + first-address verification fields; accepting reduced form` |
| WARNING (exit 0) | `warning: import-wallet: bsms: signature present but not verified in v0.26.0; see FOLLOWUP \`bsms-verify-signatures\`` |
| WARNING (exit 0) | `warning: import-wallet: bsms: first-address mismatch at path <P>: computed <C>, blob declares <D>` — restored at v0.27.0 Phase 3 via `crate::derive_address::derive_first_address`. Mismatch is informational (exit 0); v0.27.0 toolkit derives at canonical `/0/0` from the parsed descriptor. Taproot descriptors are skipped (BIP-129 §1 prerequisites pre-date BIP-386). Closes FOLLOWUP `bsms-first-address-verify`. |
| NOTICE (exit 0) | `notice: import-wallet: bsms: --select-descriptor <X> has no effect; BSMS Round-2 carries a single descriptor` |
| NOTICE (exit 0) | `notice: import-wallet: bitcoin-core: dropped wallet-state fields <fields>: not preserved in bundle output (key-state only)` |
| WARNING (exit 0) | `warning: import-wallet: roundtrip not byte-exact; semantic equivalent; diff below`<br>(+ unified-diff body on stderr OR in `--json` envelope, never both) |
| Error (exit 1) | `error: import-wallet: could not detect format; supply --format <bsms\|bitcoin-core>` |
| Error (exit 1) | `error: import-wallet: --format <X> supplied but blob looks like <Y>` |
| Error (exit 1) | `error: <flag>: env-var <VAR> referenced by sentinel is not set` (cross-cutting `EnvVarMissing` variant; emitted by any subcommand consuming a secret flag) |
| Error (exit 2) | `error: import-wallet: <format>: parse error: <detail>` |
| Error (exit 2) | `error: import-wallet: bitcoin-core: xprv-bearing descriptor refused; re-run \`bitcoin-cli listdescriptors\` without \`true\` to get xpub-only output` |
| Error (exit 3) | (via `FutureFormat` From-impl) `error: future format: bsms: version "<V>"; toolkit supports "1.0"` |
| Error (exit 4) | `error: import-wallet: cosigner <N>: supplied seed produces xpub <X> at path <P>; blob declares <Y>` |

## §3 Env-var sentinel `@env:<VAR>` (cross-cutting)

### §3.1 Surfaces covered

| # | Flag | Subcommands |
|---|---|---|
| 1 | `--passphrase` | bundle, verify-bundle, convert, derive-child (covers BIP-85 path), slip39-{split,combine} |
| 2 | `--bip38-passphrase` | convert |
| 3 | `--ms1` | verify-bundle, **import-wallet (new)** |
| 4 | `--share` | slip39-combine, seed-xor-combine |
| 5 | `--slot @N.<subkey>=` (secret-bearing subkeys: `phrase`, `entropy`, `wif`, `xprv`) | bundle, verify-bundle (via slot-subkey infra at `slot_input.rs`) |
| 6 | `--from <node>=` (secret-bearing nodes: `phrase`, `entropy`, `wif`, `xprv`, `minikey`, `electrum-phrase`) | convert, derive-child, slip39-{split,combine}, seed-xor-{split,combine} (via composite-node infra at `from_input.rs`) |

Stdin-form variants (`--passphrase-stdin`, `--bip38-passphrase-stdin`, `--from <node>=-`, `--slot @N.<subkey>=-`) are unaffected — already non-argv. Note: `SlotSubkey` enum (`slot_input.rs:17-32`) does NOT have an `Ms1` variant; prior plan-doc references to `--slot @N.ms1=` were inaccurate (the `--ms1` direct flag is row 3 above; per-cosigner ms1 material is supplied via `--slot @N.entropy=` for raw-hex form or via `import-wallet --ms1` for the import-side overlay).

### §3.2 Grammar

Sentinel: `@env:<VARNAME>` where `<VARNAME>` matches the POSIX env-var-name regex `[A-Z_][A-Z0-9_]*`.

- **Resolution scope (NORMATIVE):** sentinel resolution applies ONLY at the 6 secret-flag-surface classes enumerated in §3.1. Non-secret flags treat `@env:VAR` as literal text (no resolution attempted). This is the locked rule per §7.0.d. Row 5 (`--slot @N.<subkey>=`) and row 6 (`--from <node>=`) are composite forms covering all secret-bearing subkey/node variants enumerated in the row; resolution applies per-element on the composite form's secret-bearing values.
- Whole-value sentinel (no concatenation): `--ms1 @env:MNEMONIC_MS1_0` ✓; `--ms1 prefix@env:VAR` ✗ (treated as literal).
- Resolution: clap-parse-time substitution via `std::env::var(VARNAME)` invoked from the 6 enumerated callsites.
- Missing/unset env-var → exit 1 with cross-cutting `EnvVarMissing` variant.
- Empty-string env-var (`VAR=""`) → preserves v0.25.1 watch-only sentinel semantics: substituted value is `""` and proceeds through validation (e.g., `validate_flag_hrp("--ms1", "ms", "")` early-returns Ok per v0.25.1).
- Invalid `<VARNAME>` (e.g., `@env:foo bar`, `@env:1FOO`, `@env:`) → exit 1 with `EnvVarMissing` and stderr template "invalid env-var name `<VARNAME>`".
- Literal `@env:<text>` cannot be escaped in v0.26.0 (FOLLOWUP `env-var-sentinel-literal-escape`).

### §3.3 Interaction with stdin sentinel

- `--ms1 -` (stdin) and `--ms1 @env:VAR` are mutually exclusive at the per-invocation level: only one stdin reader per invocation per `verify_bundle.rs:876` precedent.
- Multiple `@env:VAR` sentinels on a repeating flag are allowed: `--ms1 @env:MS1_0 --ms1 @env:MS1_1`.
- Mixed forms allowed: `--ms1 ms1xxx... --ms1 @env:MS1_1 --ms1 -` (one literal + one env + one stdin).
- **Env-var name collision rule (cross-cutting):** Referencing the same env-var multiple times within a single invocation is **explicitly allowed** and resolves to the same value (e.g., `--ms1 @env:WALLET_SEED --ms1 @env:WALLET_SEED` resolves both to `std::env::var("WALLET_SEED")`). The resolver reads the env-var per-sentinel-occurrence (no caching); calling `std::env::var` repeatedly is `O(1)` per call. Use case: cosigners sharing entropy in test fixtures or pathological cases. No-op for the resolver; useful for user clarity.
- **Variant naming (canonical):** the `ToolkitError::EnvVarMissing` variant is cross-cutting (not import-wallet-specific); applies uniformly across all 6 secret-flag surfaces. No `ImportWalletEnvVarMissing` variant exists. Error template carries the offending `<flag>` name for disambiguation.

### §3.4 SPEC §5.11 placement (in `SPEC_mnemonic_toolkit_v0_5.md`)

Real-anchor verification (grep `^## ` against v0.5 SPEC TOC, 2026-05-18): existing §5-cluster sections are §5.5, §5.5.a, §5.6, §5.7, §5.8 (numerically discontiguous due to delta-only ordering). New section anchor: **§5.11**, textually inserted after the §5-cluster:
> §5.11 CLI value-source sentinels (NEW)
> Generalizes the empty-string sentinel from §5.8 + the stdin sentinel `-` (existing in CLI grammar). Adds `@env:<VAR>` as a third value-source. Future sentinels (`@file:<PATH>`, etc.) accumulate here.

Similarly:
- **§6.11 (NEW)** — `import-wallet` CLI grammar, placed after existing §6.10 (Conditional-applicability projection).
- **§6.11.a (NEW)** — `wallet_import` round-trip discipline. Per §7.0.b: this is a sub-section of §6.11 (not a new §7 top-level) to preserve v0.5 SPEC's delta-only ordering convention. Mirrors `§4.12.a-g` precedent established by the v0.19.0 non-canonical descriptor cycle.

## §4 BSMS Round-2 parser

### §4.1 Accepted shapes (lenient)

**2-line shape** (kickoff seed-case form):
```
BSMS 1.0
<descriptor>#<checksum>
```
- Stderr WARNING per §2.4.

**6-line shape** (full BIP-129 Round-2):
```
BSMS 1.0
<TOKEN>
<descriptor>#<checksum>
<DERIVATION_PATH>
<FIRST_ADDRESS>
<SIGNATURE>
```
- No WARNING about reduced form.
- First-address verification: **deferred to v0.27+ per Phase 2 I1 fold.** `<FIRST_ADDRESS>` is preserved verbatim in `ParsedImport.bsms_audit.first_address` for the `--json` envelope; toolkit-side derivation + mismatch WARNING tracked in FOLLOWUP `bsms-first-address-verify`. Rationale: descriptor-at-derivation-path → address rendering is non-trivial absent a derivation helper that doesn't exist in v0.26.0 toolkit surface; the WARNING was informational-only (not hard-error), so deferral does not weaken the import-path correctness contract — concrete-keys checksum (BIP-380) + xpub parse (`MsDescriptor::from_str`) + watch-only invariant remain load-bearing.
- `<TOKEN>` + `<SIGNATURE>` preserved in `ParsedImport.bsms_audit` for `--json` envelope; not verified in v0.26.0 (FOLLOWUP `bsms-verify-signatures`).

**Note on the 6-line shape vs BIP-129 (Phase 4 R0 M4 fold).** BIP-129 §6 specifies a 4-line Round-2 plaintext (version + descriptor + derivation_path + first_address); the HMAC token + signature live OUTSIDE the plaintext (in the encryption envelope). The toolkit's 6-line shape above is a **toolkit-specific lenient input shape** consolidating the BIP-129 plaintext lines + the envelope-side HMAC/signature into a single flat blob, so an importer that doesn't decrypt the envelope can still preserve and audit those fields. The lenient consolidation is sound for v0.26.0's parse-only contract; full BIP-129 §5 HMAC token + signature verification (which requires the coordinator's HMAC key material) is tracked at FOLLOWUP `bsms-verify-signatures` and will revisit this shape if verification needs a stricter input form.

### §4.2 Parse pipeline

1. Split blob bytes on `\n`. Normalize CRLF → LF before split.
2. Verify first line == `BSMS 1.0`. Other versions (`BSMS 2.0`, etc.) → `FutureFormat` → exit 3.
3. Detect 2-line vs 6-line by line count.
4. Extract descriptor body (line 2 in 2-line, line 3 in 6-line).
5. **Adapter step:** lex concrete `[fp/path]xpub` occurrences from the descriptor body via regex `\[([0-9a-fA-F]{8})((?:/\d+'?)+)\]([xtuvyzYZ]pub[A-HJ-NP-Za-km-z1-9]+)`. For each match: assign sequential `@N` placeholder; collect `(ParsedKey, ParsedFingerprint)` pair preserving declaration order.
6. Substitute concrete keys with `@N` placeholders in the descriptor body, producing a placeholder-form descriptor.
7. Call `parse_descriptor::parse_descriptor(placeholder_descriptor, &parsed_keys, &parsed_fingerprints)` (existing pipeline at `parse_descriptor.rs:747-751`). BIP-380 checksum auto-validated via `MsDescriptor::from_str`.
8. **Network detection from origin paths.** Extract the `coin_type` child number (BIP-48 path component index 1, hardened) from the first parsed cosigner's `[fp/path]` origin annotation. Map: hardened `0'` → `bitcoin::Network::Bitcoin`; hardened `1'` → `bitcoin::Network::Testnet`. Signet and regtest are not distinguishable from testnet via origin-path inspection in either BIP-129 BSMS or Bitcoin Core `listdescriptors` — both use coin-type `1`. Wallets intrinsically running on signet/regtest are imported as testnet; users running signet/regtest workflows must supply `--network signet|regtest` post-import via a downstream subcommand if signet/regtest semantics are required (FOLLOWUP: `wallet-import-signet-regtest-disambiguation`, v0.27+). **Cosigner-to-cosigner coin-type heterogeneity** (e.g., cosigner 0 has `m/48'/0'/...`, cosigner 1 has `m/48'/1'/...`) → exit 2 `ImportWalletParse` per §2.3 with stderr template `error: import-wallet: <format>: cosigner <i> has coin-type <c1>, cosigner 0 has coin-type <c0>; all cosigners must share a coin-type`. The single-`Network` field on `ParsedImport` (per §8.1) permits no heterogeneity. (SLIP-132 prefixes ypub/zpub/upub/vpub remain handled by existing `slip0132.rs::normalize_xpub_prefix` for xpub-string canonicalization — orthogonal to network inference.)
9. Construct `ParsedImport { descriptor, cosigners, network, threshold, bsms_audit }`. Enforce watch-only invariant: every `cosigners[i].entropy == None`.

### §4.3 Cosigner ordering

`multi(N, @0, @1, ..., @M)` preserves declaration order at `@N` substitution time. `sortedmulti(N, @0, @1, ..., @M)` ALSO preserves the user-supplied input order at the `@N` placeholder level (the discriminator between `multi` and `sortedmulti` is the function name; the SORT in `sortedmulti` is a render-time operation, not a wire-level reordering). No new TLV needed.

### §4.4 Checksum

BIP-380 8-character polymod checksum. **Validated UP-FRONT by `wallet_import::bsms::parse` via `miniscript::descriptor::checksum::verify_checksum` on the concrete-keys descriptor body, BEFORE the `concrete_keys_to_placeholders` adapter rewrites the body to `@N` placeholder form for `parse_descriptor`.** The downstream `MsDescriptor::from_str` inside `parse_descriptor` operates on the synthetic-xpub-substituted form (per `parse_descriptor::substitute_synthetic` at `parse_descriptor.rs:776`) and cannot reach the original checksum — so up-front validation is the load-bearing path.

(Resolves FOLLOWUP `wallet-import-bsms-checksum-delegation-note` per Phase 2 R0 architect-review wording fix. The prior SPEC wording "auto-validated when `MsDescriptor::from_str` is called by `parse_descriptor`" was structurally inaccurate; the implementation at `wallet_import/bsms.rs:26-27,140-145` has been correct since Phase 2 close.)

## §5 Bitcoin Core `listdescriptors` parser

### §5.1 Accepted shape

Top-level JSON wrap: `{"wallet_name": "<name>", "descriptors": [<entry>, ...]}`. `wallet_name` is metadata-only.

Per-entry shape (each entry):
```json
{
  "desc": "wsh(sortedmulti(2,[fp1/48'/0'/0'/2']xpub.../<0;1>/*,[fp2/48'/0'/0'/2']xpub.../<0;1>/*))#abcdefgh",
  "timestamp": <int|"now">,
  "active": <bool>,
  "internal": <bool>,
  "range": [<int>, <int>],
  "next": <int>,
  "next_index": <int>
}
```

### §5.2 Parse pipeline

1. JSON-parse via `serde_json`.
2. For each `descriptors[i]`:
   a. Reject if `desc` contains `xprv` → `ImportWalletXprvForbidden` (exit 2 per §2.3).
   b. Extract `desc` field; same adapter + `parse_descriptor` pipeline as BSMS (§4.2 steps 5-9).
   c. Preserve `active`, `internal`, `range` in per-entry metadata (drives `--select-descriptor active-*` filtering).
   d. Drop `timestamp`, `next`, `next_index`; if any are present, emit stderr NOTICE per §2.4.

### §5.3 `--select-descriptor` filtering

- `all` (default): emit one bundle per entry; output stream uses `\n;\n` separator (cards) or JSON array (`--json`).
- `N` (integer): emit only `descriptors[N]`; error if N out of range.
- `active-receive`: emit entries with `active: true, internal: false`. Multiple matches → emit all; zero matches → exit 1 error "no active-receive descriptor found".
- `active-change`: emit entries with `active: true, internal: true`. Same multi/zero handling.

Under `--format bsms`, any non-default `--select-descriptor` value emits stderr NOTICE (BSMS has single descriptor) and is treated as `all`.

## §6 Sniff (format auto-detect)

### §6.1 Heuristics

`sniff(blob)` consults each parser's `WalletFormatParser::sniff` in fixed order:

1. **BSMS**: blob starts with the literal byte sequence `BSMS 1.0\n` (or `BSMS 1.0\r\n` after CRLF normalize). Exact prefix match; no fuzzing.
2. **Bitcoin Core**: blob trimmed-leading-whitespace starts with `{`; `serde_json::from_slice::<serde_json::Value>` succeeds; top-level JSON value is an object with a `descriptors` key whose value is a non-empty array; each `descriptors[i]` is an object with a `desc: String` field; AND **NO vendor-specific marker keys present** at top level (e.g., `chain`, `policy`, `version` indicate Specter; `bipname`, `extendedPublicKey` indicate other vendors). The absence-check is conservative and conservative-only; tightening to a positive Core-marker check (presence of `wallet_name` + `timestamp` or `next_index`) is FOLLOWUP `wallet-import-sniff-bitcoin-core-tighten-heuristic`.

### §6.2 Ambiguity handling

- If 0 parsers' `sniff` returns true: exit 1 `ImportWalletAmbiguousFormat` with stderr template "could not detect format; supply --format <bsms|bitcoin-core>".
- If ≥2 parsers' `sniff` returns true (e.g., contrived JSON blob containing `BSMS 1.0` as a string value AND a valid `descriptors` array): exit 1 `ImportWalletAmbiguousFormat` with stderr template "blob matches multiple format heuristics; supply --format <X>".
- If `--format <X>` is supplied AND a DIFFERENT format's sniff returns true (positive-sniff-for-other-format): exit 1 `ImportWalletFormatMismatch` with stderr template "--format <X> supplied but blob looks like <Y>".
- **If sniff returns `NoMatch` or `Ambiguous` AND `--format <X>` is supplied: the explicit `--format` is honored unconditionally** (per Phase 5 R0 I3 fold). Parse proceeds with the supplied format; any downstream parse error surfaces as `ImportWalletParse` (exit 2), not `ImportWalletFormatMismatch` (exit 1). Rationale: a user explicitly overriding sniff (e.g., a legitimate Bitcoin Core blob that happens to contain a `chain` key, or a hand-rolled BSMS variant) deserves to have their override respected; mismatch-exit-1 fires only when sniff has positive evidence of a DIFFERENT format. The cell `sniff_explicit_format_honored_when_blob_has_vendor_markers` (`tests/cli_import_wallet_sniff.rs:142-160`) pins this contract.

## §7 Round-trip discipline

### §7.1 Bundle round-trip

For each format `F` and each cosigner-count / template combination in the test corpus:
```
let bundle_synth = mnemonic synthesize <toolkit-args>;
let blob = mnemonic export-wallet --format F < bundle_synth;
let bundle_imp = mnemonic import-wallet --format F --blob blob;
assert bundle_imp == bundle_synth  // full struct equality on `Bundle`
```

### §7.2 Semantic blob round-trip

For each third-party fixture blob `B` in the corpus:
```
let bundle = mnemonic import-wallet --blob B;
let blob_re = mnemonic export-wallet --format F < bundle;
assert canonicalize(B) == canonicalize(blob_re)
if B != blob_re bytewise:
  stderr WARNING (default mode) OR
  --json envelope `roundtrip: {byte_exact: false, semantic_match: true, diff: "..."}`
```

### §7.3 `canonicalize()` per format

#### §7.3.1 BSMS

```
canonicalize(bsms_blob):
  1. Normalize CRLF → LF.
  2. Strip trailing whitespace per line.
  3. Parse descriptor body via MsDescriptor::from_str; re-render via to_string(); re-checksum via miniscript.
  4. Drop token, signature, first_address, derivation_path lines from compare (semantic round-trip).
  5. Re-emit: "BSMS 1.0\n<re-rendered-descriptor>#<re-checksum>\n"
```

**Policy:** semantic round-trip is **descriptor-only**. Audit fields (token, signature, first_address, derivation_path) are coordinator-output-side metadata not regeneratable from a bundle alone. Importing a 6-line BSMS Round-2 → bundle drops audit metadata; re-exporting bundle → BSMS Round-2 emits a 2-line shape (no synthesis of fresh token/signature/first-address; that requires the coordinator's HMAC keying material which is not part of bundle state). The `--json` envelope `bsms_audit` field preserves the original audit metadata for the user to re-attach manually if they choose. Future FOLLOWUP `bsms-audit-field-regeneration` may add a `--coordinator-key <FILE>` flag enabling re-signed Round-2 export.

#### §7.3.2 Bitcoin Core

```
canonicalize(core_blob):
  1. Parse JSON via serde_json.
  2. For each descriptors[i]:
     - desc: byte-equality after re-checksum (parse via MsDescriptor::from_str; to_string()).
     - active, internal, range: byte-equality.
     - timestamp, next, next_index: DROPPED from compare.
  3. wallet_name: preserved (metadata).
  4. Re-serialize with keys sorted alphabetically + 2-space indent + trailing newline.
```

### §7.4 `--json` envelope `roundtrip` field

```json
{
  "roundtrip": {
    "byte_exact": false,
    "semantic_match": true,
    "diff": "--- input\n+++ output\n@@ -3,1 +3,1 @@\n-old line\n+new line\n",
    "status": "ok"
  }
}
```

`diff` is `Some(...)` iff `byte_exact == false`. Format: unified-diff (RFC standard). When `--json` is set, the diff goes ONLY in the envelope; stderr is silent. When `--json` is NOT set, diff goes ONLY on stderr; stdout cards are unaffected.

**`status` extension key (v0.26.0 lock per Phase 5 R0 fold).** Values:

- `"ok"`: standard case; `byte_exact` + `semantic_match` reflect the canonicalize comparison faithfully.
- `"blocked_no_emitter"`: applies to BSMS until FOLLOWUP `wallet-export-bsms-emitter` lands. The re-emit side of the round-trip cannot run because `export-wallet --format bsms` is unimplemented in v0.26.0. Envelope reports `byte_exact: false`, `semantic_match: false`, `diff: null`; the discriminator informs downstream consumers (mnemonic-gui, automation) that the round-trip discipline is not yet evaluable for this blob — NOT that the blob is malformed.
- `"canonicalize_failed"`: applies when `canonicalize_bsms` or `canonicalize_bitcoin_core` errors after parse-success (e.g., exotic descriptor accepted by parse but rejected by `MsDescriptor::from_str` in the canonicalize path). Envelope reports `byte_exact: false`, `semantic_match: false`, `diff: null`. **v0.27.1 amendment (per FOLLOWUP `pr-26-roundtrip-warning-suppression`):** the envelope carries an additional `error: String` field with the typed `ToolkitError` Display form, scoped to the `canonicalize_failed` branch only (omitted in other status values). Example:
  ```json
  "roundtrip": {
    "byte_exact": false,
    "semantic_match": false,
    "diff": null,
    "status": "canonicalize_failed",
    "error": "canonicalize_bitcoin_core: miniscript: unexpected token"
  }
  ```
  The canonicalize-failure-with-parse-success class is a candidate for FOLLOWUP `wallet-import-roundtrip-canonicalize-edge-cases` if it surfaces in real fixtures. **Backward compatibility:** consumers parsing the prior `{byte_exact, semantic_match, diff, status}` fields are unaffected; the `error` key is additive in a closed-enum branch.

## §8 Module layout

```
crates/mnemonic-toolkit/src/
├── cmd/
│   └── import_wallet.rs              — CLI entry; clap glue; trait dispatch
├── wallet_import/
│   ├── mod.rs                        — pub(crate) trait WalletFormatParser + struct ParsedImport
│   ├── sniff.rs                      — auto-detect; ambiguity → exit 1
│   ├── bsms.rs                       — pub(super) struct BsmsParser; impl WalletFormatParser
│   ├── bitcoin_core.rs               — pub(super) struct BitcoinCoreParser; impl WalletFormatParser
│   ├── pipeline.rs                   — concrete-keys → @N-placeholder adapter
│   └── roundtrip.rs                  — canonicalize + unified-diff helper
├── secrets.rs                        — extension: env-var sentinel resolution (cross-cutting)
└── error.rs                          — new ToolkitError variants (§2.3 mapping)
```

### §8.1 Trait surface

```rust
pub(crate) trait WalletFormatParser {
    fn sniff(blob: &[u8]) -> bool;
    fn parse(blob: &[u8]) -> Result<Vec<ParsedImport>, ToolkitError>;
}

pub(crate) struct ParsedImport {
    pub(crate) descriptor: md_codec::Descriptor,
    pub(crate) cosigners: Vec<ResolvedSlot>,    // INVARIANT: all entropy == None
    pub(crate) network: bitcoin::Network,
    pub(crate) threshold: Option<u8>,
    pub(crate) bsms_audit: Option<BsmsAuditFields>,
}

pub(crate) struct BsmsAuditFields {
    pub(crate) token: String,
    pub(crate) signature: String,
    pub(crate) first_address: String,
    pub(crate) derivation_path: String,
    pub(crate) signature_verified: bool,    // always false in v0.26.0
}
```

**Dispatch:** The trait has associated-function signatures (no `&self`), matching the existing `WalletFormatEmitter` non-`&self` shape at `wallet_export/mod.rs:322`. The dispatcher uses `match format { ... }` enum-style dispatch (NOT `dyn WalletFormatParser`); trait is not object-safe by design.

**Field-name discipline:** Use the canonical name **`ResolvedSlot`** in new wallet-import code. `CosignerKeyInfo` is a deprecated type alias for `ResolvedSlot` retained for backward-compatibility (`synthesize.rs:182-188`, re-exported via `parse_descriptor.rs:12`); the alias remains importable but should not be used in new code (§7.0.c). Field names per `ResolvedSlot`: `.xpub` (xpub bytes / typed `Xpub`), `.fingerprint` (`Fingerprint`), `.path` (`DerivationPath` — typed origin path), `.path_raw` (`String` — raw `[fp/path]` text), `.entropy` (`Option<Zeroizing<Vec<u8>>>`), `.master_xpub` (`Option<Xpub>`). Code accessing the origin path uses `.path` for typed comparison or `.path_raw` for byte-exact equality against the blob's input text. NO field named `origin_path` exists.

### §8.2 Watch-only invariant enforcement

New `ToolkitError` variant: `ImportWalletWatchOnlyViolation(usize)` (carrying the offending cosigner index). Tier-2 routing per §2.3 (format-violation/refusal; mirrors `ExportWalletSecretInput` discipline at `error.rs:93,308,354,417`).

```rust
fn validate_watch_only_resolved(cosigners: &[ResolvedSlot]) -> Result<(), ToolkitError> {
    for (i, c) in cosigners.iter().enumerate() {
        if c.entropy.is_some() {
            return Err(ToolkitError::ImportWalletWatchOnlyViolation(i));
        }
    }
    Ok(())
}
```
Called by each `WalletFormatParser::parse` impl post-construction. Mirrors `wallet_export/mod.rs:117-124`. Stderr template: `error: import-wallet: cosigner <N> has entropy populated post-parse; watch-only invariant violated (internal bug)`.

### §8.3 Seed overlay

Seed overlay happens AFTER `WalletFormatParser::parse` and BEFORE bundle synthesis:
```
fn apply_seed_overlay(
    parsed: &mut Vec<ParsedImport>,
    ms1_args: &[String],
    phrase_overlays: &[(usize, String)],    // from --slot @N.phrase=
) -> Result<(), ToolkitError> {
    for (n, ms1_or_phrase) in collect_overlays(ms1_args, phrase_overlays) {
        let entropy = decode_ms1_or_phrase(ms1_or_phrase)?;
        let cosigner = &parsed[bundle_idx].cosigners[n];
        let derived_xpub = derive_xpub_at_path(&entropy, &cosigner.path)?;    // .path is typed DerivationPath per §8.1
        if derived_xpub != cosigner.xpub {
            return Err(ToolkitError::ImportWalletSeedMismatch {
                cosigner_index: n,
                derived_xpub: derived_xpub.to_string(),
                blob_xpub: cosigner.xpub.to_string(),
                path: cosigner.path_raw.clone(),
            });
        }
        parsed[bundle_idx].cosigners[n].entropy = Some(entropy);
    }
    Ok(())
}
```

Note: derivation uses `.path` (typed `DerivationPath`) for the cryptographic operation; the error report uses `.path_raw` for human-readable byte-exact text matching the blob input.

If overlay applies to N=K cosigner: that cosigner's `entropy` becomes `Some(...)`; remaining cosigners stay `None` (watch-only sentinels in emitted bundle).

## §9 GUI lockstep (v0.11.0)

### §9.1 SubcommandSchema entry (`mnemonic-gui/src/schema/mnemonic.rs`)

New entry mirroring §2.1 surface:

```rust
SubcommandSchema {
    name: "import-wallet",
    flags: &[
        FlagSchema { name: "--blob", kind: FlagKind::FilePath, ... },
        FlagSchema { name: "--format", kind: FlagKind::TaggedOrIndexed(&["bsms", "bitcoin-core"]), ... },
        FlagSchema { name: "--ms1", kind: FlagKind::Text { repeatable: true }, ... },
        FlagSchema { name: "--slot", kind: FlagKind::SlotSubkey, ... },  // existing infra
        FlagSchema { name: "--select-descriptor", kind: FlagKind::TaggedOrIndexed(&["active-receive", "active-change", "all"]), default: Some("all"), ... },
        FlagSchema { name: "--json", kind: FlagKind::Bool, ... },
        // --no-auto-repair auto-attaches via global-flag projection
    ],
    ...
}
```

### §9.2 Surface placement

Lives in the existing `mnemonic` top-level tab's subcommand combobox at `main.rs:359-371`. **No new top-level tab.** Schema version stays **v5** (additive subcommand entry).

### §9.3 Env-var seed channel (subprocess spawn)

`mnemonic-gui/src/runner.rs::run` extended:
1. Collect per-cosigner-index secret values from the GUI form.
2. Set per-secret env-vars on the spawned subprocess: `MNEMONIC_MS1_<i>=<value>`, `MNEMONIC_PHRASE_<i>=<value>`, `MNEMONIC_PASSPHRASE=<value>`, etc.
3. Replace argv flag values with `@env:MNEMONIC_<KIND>_<i>` sentinels.
4. Run-confirm-modal at `main.rs:686-688` renders the sentinel-bearing argv — secrets never appear.
5. Subprocess inherits env, reads via `std::env::var`, processes, exits; env cleared with process tree.

### §9.4 kittest coverage

- 6-8 cells exercising:
  - Form rendering with new SubcommandSchema entry visible in combobox.
  - File-picker → `--blob <path>` argv construction.
  - Repeating `--ms1` text inputs → multiple `--ms1 @env:MNEMONIC_MS1_<i>` argv tokens.
  - `--select-descriptor` dropdown → correct TaggedOrIndexed argv value.
  - `--json` checkbox → correct argv toggle.
  - Run-confirm-modal shows sentinel form (no raw seed visible).
  - Subprocess receives env-vars; toolkit-side reads value correctly (integration test against built toolkit binary).
  - **Env-var-failure path:** empty seed input → GUI sends `--ms1 @env:MNEMONIC_MS1_0` with `MNEMONIC_MS1_0` unset → toolkit exits 1 `EnvVarMissing` → GUI displays the error message in the output preview pane.
  - **Env-var-lifecycle gate:** after subprocess exits, assert env-var keys cleared from parent GUI process state (no env-var persistence in `std::env::vars()` for the secrets just used). Per `[[project_v0_22_1_verify_bundle_auto_fire_shipped]]` MNEMONIC_FORCE_TTY discipline, env-var-lifecycle gates have been load-bearing before.

### §9.5 Schema-mirror drift gate

The existing `mnemonic-gui/tests/schema_mirror_secret_drift.rs` gate is version-tolerant (`>=1`); additive SubcommandSchema entries are auto-validated.

**Phase 0 verification step (load-bearing per `[[feedback-build-rs-stub-fallback-security-audit]]`):** before Phase 1 implementation begins, verify that the `secret: bool` field flows correctly from the toolkit-side `gui_schema.rs` JSON output through `mnemonic-gui`'s `secret_flag_keys()` consumer for the new `@env:VAR`-bearing flags. The drift gate's predicates rely on `.contains()` against the secret-flag set; if the env-var sentinel changes the way secret flags are identified at parse time, the paste-warn / run-confirm-modal secret detection could fail open on `--ms1`. Recon: `cargo run -- gui-schema --classify-flags` + grep new entries for `"secret": true` on `--ms1`, `--passphrase`, `--bip38-passphrase`, `--share`, `--slot @N.phrase=`, `--slot @N.ms1=`.

## §10 Test corpus

### §10.1 BSMS fixtures (12-15 per round-trip side)

Format diversity:
- 2-line vs 6-line wire shape (×2)
- `wsh(sortedmulti(2, …))` 2-of-2, 2-of-3, 3-of-5 (×3)
- `wsh(multi(...))` non-sortedmulti (×1)
- **Decaying-multisig (driving seed-case): `wsh(thresh(2, pk, s:pk, sln:older(N)))` with N = 144 (1-day), N = 4032 (~28-day), N = 32768 (~227-day, matches user's blob) (×3)** — promoted to dedicated test class given this is the user's flagship use case.
- `tr(NUMS, ...)` taproot (×1) — if rust-miniscript supports
- `sh(wsh(...))` legacy compatibility (×1)
- Mainnet + testnet (×2 axis)
- SLIP-132 variants (ypub, zpub, upub) (×3)
- Edge: 1-of-1 single-sig wsh (×1)

**Per-format budget clarification:** "12-15 per round-trip side" means **12-15 distinct input fixtures**, each exercised in BOTH the bundle round-trip (§7.1) AND the semantic blob round-trip (§7.2) directions — so 12-15 inputs × 2 directions = 24-30 round-trip cells per format (matches Phase 4 budget in BRAINSTORM §5).

**v0.26.0 shipped subset (per Phase 4 R0 fold).** v0.26.0 ships a reduced fixture corpus selected for the load-bearing canonicalize / round-trip paths:

- BSMS (8 fixtures shipped): 2-line decay-32768 (kickoff seed-case), 2-line decay-144, 2-line sortedmulti-2of2, 2-line sortedmulti-2of3, 2-line multi-2of2, 2-line multi-2of3 (declaration-order assertion — Phase 4 fold close M3), testnet-tpub-2of2, sh(wsh)-2of3 legacy + 1-of-1 single-sig.
- Bitcoin Core (5 fixtures shipped): BIP-84 P2WPKH single-sig (mainnet), BIP-49 P2SH-P2WPKH (mainnet), wsh-sortedmulti 2-of-3 (mainnet), receive+change BIP-84 pair (mainnet, ×4 entries), testnet BIP-84.

Items deferred to FOLLOWUP `wallet-import-fixture-corpus-expansion`: BSMS decay-4032, 6-line sortedmulti-2of3, sortedmulti-3of5, mainnet+ypub, mainnet+zpub, tr(NUMS,...) taproot; Core BIP-44 P2PKH, BIP-86 P2TR, wsh-sortedmulti 3-of-5, native `<0;1>/*` multipath shape, explicit `active: false` cell name. Rationale: shipped subset exercises the canonicalize discipline + idempotency + both Core envelope shapes (object + bare-array) + declaration-order preservation invariant; missing fixtures are coverage-expansion targets, not load-bearing for v0.26.0's correctness contract. Full corpus tracked at v0.27+.

### §10.2 Bitcoin Core fixtures (12-15 per round-trip side)

- Single-sig P2PKH (BIP-44), P2WPKH (BIP-84), P2SH-P2WPKH (BIP-49), P2TR (BIP-86) (×4)
- Multisig wsh-sortedmulti 2-of-3 + 3-of-5 (×2)
- Multipath `<0;1>/*` (default in `listdescriptors`) (×1)
- Receive + change pairs (4 entries per wallet) (×1)
- `active: true` and `active: false` mix (×1)
- Mainnet + testnet (×2)

**See §10.1 v0.26.0 shipped subset note** for the actual v0.26.0 Bitcoin Core fixture set + deferral list.

### §10.3 Negative-path fixtures

- BSMS bad checksum (×1)
- BSMS unsupported version (`BSMS 2.0`) (×1)
- BSMS non-`BSMS 1.0` line 1 (×1)
- Core blob with `xprv` (×1)
- Core blob with non-JSON top level (×1)
- Core blob missing `descriptors` key (×1)
- Core blob with empty `descriptors: []` (×1)
- Auto-detect ambiguity (contrived JSON containing `"BSMS 1.0"` as string value AND `descriptors` array) (×1)
- Seed mismatch: `--ms1 <S>` where supplied seed derives different xpub at declared path (×1)
- Env-var missing: `--ms1 @env:UNSET` (×1)

## §11 Reviewer-loop gates (per phase)

Per `[[feedback-opus-primary-review-agent]]`: every phase dispatches opus-model `feature-dev:code-architect` agent for R0 review of the per-phase plan/SPEC slice, iterates R0 → R1 → R2 until 0 Critical / 0 Important. Per-phase agent reports persist to `design/agent-reports/`.

## §12 Cycle close

- Tag `mnemonic-toolkit-v0.26.0` (toolkit; git+tag — crates.io publish blocked on miniscript [patch.crates-io] per `[[project_v0_24_0_cycle_shipped]]`).
- Tag `mnemonic-gui-v0.11.0` (GUI; lockstep static-form).
- 11 new FOLLOWUPs filed per BRAINSTORM §6.
- CHANGELOG.md entries (toolkit + GUI) with byte-exact migration notes for env-var sentinel + `import-wallet` subcommand.
- GitHub Release notes draft (both repos).
- End-of-cycle architect review (holistic) dispatched on the merged release branch.

---

**End of SPEC.**
