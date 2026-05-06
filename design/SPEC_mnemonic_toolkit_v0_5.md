# mnemonic-toolkit v0.5 SPEC

**Version:** 0.5.0
**Date:** 2026-05-06
**Status:** APPROVED (in-plan-mode brainstorm + SPEC + plan all converged 0C/0I)
**Predecessor:** [SPEC_mnemonic_toolkit_v0_4.md](SPEC_mnemonic_toolkit_v0_4.md) (v0.4.0 + v0.4.1 + v0.4.2 + v0.4.3 + v0.4.4 + v0.4.5)

## v0.4 → v0.5 amendments (delta-only summary)

- **§4.11.b — DELIBERATE REVERSAL.** Typed-`DerivationPath` equality (folds `h` → `'`) replaces v0.4 raw-string equality for BIP-388 distinctness. `48h/0h/0h/2h` and `48'/0'/0'/2'` now compare EQUAL.
- **§5.7 line 103 — multiset semantics for `md1_xpub_match`.** Set-equality with multiplicity (sort-then-compare).
- **§5.7 line 104 — four-case ms1 short-circuit table** (watch-only / full-supplied-decodes / full-supplied-malformed / full-supplied-absent) with byte-exact `decode_error` strings.
- **§5.7 (new paragraph) — mk1 cosigner-mapping diagnostic** distinguishing `NotSupplied` / `DecodeFailed` / `XpubNotInPolicy`.
- **§6.6 — legacy CLI flag deletion.** `--phrase`, `--xpub`, `--cosigner`, `--master-fingerprint`, `--cosigner-count`, `--cosigners-file` removed entirely; only `--slot @N.<subkey>=<value>` survives for slot-bearing data.
- **JSON envelope** — `engraving_card` field deletion; `origin_path` `null` unification.

## Carry-forward from v0.4

Unless explicitly noted as DELTA below, all sections of v0.4 SPEC carry forward unchanged. This SPEC writes only the delta sections.

**Carry forward unchanged:** §1 (Metadata), §2 (Versioning), §3 (Wire format — md1/mk1/ms1 codecs), and most of §4–§6 (see "v0.4 → v0.5 amendments" header above for the full delta list). See [SPEC_mnemonic_toolkit_v0_4.md](SPEC_mnemonic_toolkit_v0_4.md) for the full v0.4 baseline text and [SPEC_mnemonic_toolkit_v0_3.md](SPEC_mnemonic_toolkit_v0_3.md) for the §1–§3 wire-format origin.

**Delta sections in v0.5 (rewritten / added / removed below):** §4.11.b (REVERSAL: typed-DerivationPath equality), §5.5 (engraving_card field deletion note), §5.7 (multiset md1_xpub_match + four-case ms1 + mk1-mapping diagnostic), §6.6 (legacy flag deletion + alias-table removal + retained-flag enumeration). All other v0.4 delta sections (§4.9.a, §4.11.a/c/d, §5.6, §5.8, §6.7, §6.9, §8–§11) carry forward unchanged from v0.4.

## §4.9.a Layer 1 fragments — multi-leaf taproot (DELTA: deferred → SUPPORTED)

`Tr-multileaf` (descriptor `tr(K, {leaf1, leaf2, ...})` with ≥2 leaves) — **SUPPORTED in v0.4** via `walk_tap_tree`. Encoding via md-codec `Tag::TapTree` branch nodes (each branch has two children; leaves contain miniscript via `walk_miniscript_node`). Round-trip tests required for ≥2-leaf and ≥3-leaf shapes.

**0-leaf branch:** unreachable per BIP-341 (TapTree::leaves builder requires ≥1 leaf) + rust-miniscript invariant. v0.4 walker carries a one-line invariant comment citing BIP-341; no defensive guard.

## §4.11 BIP-388 distinct-key conformance (NEW; subsumes deleted §4.7/§4.12 SELF-MULTISIG WARNING)

The toolkit enforces BIP-388's "key information vector" distinct-key rule symmetrically across bundle creation and verify-bundle.

### §4.11.a Per-`@N` annotation consistency

Repeated occurrences of `@N` within one descriptor MUST share byte-identical annotations. Enforced at resolve-phase by `parse_descriptor.rs:166-180` (existing check; v0.4 confirms via test). Mismatch → exit 2 with byte-exact stderr `error: descriptor placeholder @N has inconsistent annotations across occurrences`.

### §4.11.b Key-vector distinctness across `@N` slots

Distinct `@N` slots MUST resolve to distinct `(xpub, derivation_path)` tuples. Enforced post-binding by `check_key_vector_distinctness(&binding) -> Result`. Pairwise comparison across all slot pairs. Collision → exit 2 with byte-exact stderr `error: BIP-388 distinct-key violation: slot @{i} and slot @{j} resolve to identical (xpub, path)`.

**Normalization domain (v0.5 REVERSAL):** BIP-388 distinctness operates on the typed canonical form of `derivation_path` — the form produced by `bitcoin::bip32::DerivationPath::from_str(...)`'s normalization, which folds `h`-notation into `'`-notation. Under typed equality, `48h/0h/0h/2h` and `48'/0'/0'/2'` compare EQUAL and produce a BIP-388 row-13 collision.

If a slot has no `path` subkey supplied (watch-only slot with degenerate origin metadata, or single-key `wif` slot), `derivation_path` is treated as the empty string `""` for collision comparison. Two slots with identical xpubs and both lacking `path` subkeys ARE considered colliding. Two slots with identical xpubs but different non-empty paths are NOT colliding (BIP-388 letter; different key-vector entries).

Raw user input (e.g., from `--slot @N.path=`) is preserved separately in the `path_raw: String` field of the binding and engraving-card emission for round-trip fidelity. `path_raw` is not consulted for distinctness or collision detection.

**Rationale:** BIP-388 itself uses only `'`-notation; treating `h` and `'` as the same hardened indicator is the more correct interpretation. The v0.4 raw-string-equality rule was a Phase A implementation expedient, not a semantic intent.

**v0.4 → v0.5 migration cross-reference:** v0.4 SPEC §4.11.b (raw-string-equality) is REPLACED. Any test or fixture that used `h`/`'` notation differences as a distinctness lever will start colliding in v0.5; tests must be rewritten to use genuinely distinct paths if the test intended distinct slots.

### §4.11.c Symmetric verify-bundle enforcement

Verify-bundle re-runs `check_key_vector_distinctness` on every parsed bundle (template-mode and descriptor-mode). Bundles that violated BIP-388 at creation time (e.g., v0.2 self-multisig artifacts produced by `multisig-full --cosigner-count > 1 --phrase ...`) fail v0.4 verify-bundle with byte-exact stderr `error: bundle violates BIP-388 distinct-key rule; regenerate with distinct keys`. Exit 4. No backward-compat exception.

### §4.11.d Migration guidance

CHANGELOG callout: users with v0.2 self-multisig bundles must regenerate using the v0.4 unified `bundle` command with `--slot @N.phrase=...` per cosigner (multi-source full multisig) or `--slot @N.xpub=... --slot @N.fingerprint=... --slot @N.path=...` per cosigner (watch-only multisig). The mock multisig pattern is no longer supported.

## §5.8 `MsField` type definition (NEW)

```rust
/// Schema 4 ms1 field shape. Always an array of length N (number of slots);
/// dense layout with empty-string placeholders for watch-only slots.
///
///   - len(MsField) == N (always)
///   - ms1[i] == "<ms1-string>"  : slot @i is secret-bearing; this is the encoded ms1 card
///   - ms1[i] == ""              : slot @i is watch-only (no secret material; placeholder)
///
/// Examples:
///   - Single-sig full (N=1, secret):       ["ms1abc..."]
///   - Pure watch-only multisig (N=3):      ["", "", ""]
///   - Multi-source full 3-of-3 (N=3):      ["ms1...", "ms1...", "ms1..."]
///   - Hybrid (slot 0 phrase, slots 1-2 watch-only, N=3): ["ms1...", "", ""]
pub type MsField = Vec<String>;
```

NOT a discriminated union; NOT `#[serde(untagged)]`. Schema_version is the dispatch discriminator.

**Length invariant:** `len(ms1) == N` always in schema 4. `len(mk1) == N` always. Verify-bundle asserts both invariants and exits 4 with `error: bundle schema-4 length invariant violated: expected N={N} ms1+mk1 entries; got ms1_len={ms1_len}, mk1_len={mk1_len}` if either is wrong. The dense layout preserves the slot-index correspondence `ms1[i] ↔ mk1[i] ↔ slot @i` by invariant — verify-bundle and engraving card both rely on this. Watch-only slots have `ms1[i] == ""` (empty string sentinel; a real ms1 string is never empty since it's a bech32-family encoding with mandatory HRP `"ms"` + separator). Verify-bundle skips ms1 checks for empty-string elements.

## §5.5 Engraving card layout under unified bundle (NEW)

**Card cardinality:** 1 master card per bundle (locked 2026-05-05 brainstorm). v0.5: emitted to stderr only (the v0.4 `BundleJson.engraving_card: Option<String>` JSON-envelope field is DELETED in v0.5; consumers read the card from stderr).

**Card sections (in order):**

1. **Header line:** `# === Wallet bundle: <template-or-descriptor-summary>, <network> ===`
2. **Threshold line** (multisig only): `# Threshold: {T} of {N}`
3. **Cosigners block:** `# Cosigners:` followed by N indented lines, one per slot:
   - `#   @{i}: <ms1-card-id>{,}<mk1-card-id> (<fp-or-anon> @ <path>)` where:
     - `<ms1-card-id>` = the ms1 card's `chunk_set_id` (4 hex chars) if present; OR `(no ms1; watch-only)` if `ms1[i] == ""`
     - `<mk1-card-id>` = the mk1 card's `chunk_set_id` (4 hex chars)
     - `<fp-or-anon>` = master fingerprint hex OR `anon` under `--privacy-preserving`
     - `<path>` = origin derivation path; `(no path)` if absent
4. **Template line** (template mode): `# Template: <template-name>` (e.g., `wsh-sortedmulti`)
5. **Descriptor line** (descriptor mode): `# Descriptor: <descriptor-string-or-summary>`
   - **Truncation policy:** if descriptor string length > 80 characters, render as `<first 60 chars>... [md1: <chunk-set-id>] (<descriptor_len> chars total)` and rely on the md1 card to carry the full descriptor. If ≤ 80 chars, render verbatim.
6. **Md1 reference line** (always): `# md1: <chunk_set_id>`
7. **Recovery hint line** (always): `# Recovery: any {T} of {N} signing keys + md1 (template card).`

**Privacy-preserving rendering:** under `--privacy-preserving`, fingerprint columns render as `anon` (3 ASCII chars); ms1/mk1 chunk_set_ids still render (they don't leak provenance). Card layout shape is otherwise identical.

## §5.6 Wire-bit-identical guarantee (DELTA: schema-4 carry rules)

**Schema 2 / 3 fixtures:** byte-identical regression invariant carries forward UNCHANGED (subject to §10 exclusions).

**Schema 4:** new fixtures introduced; not back-compatible with schemas 2/3 in shape (ms1 array vs flat string). Verify-bundle reads schema_version and routes accordingly.

**Cross-schema invariant for the SAME LOGICAL BUNDLE:** if a v0.2 single-sig full bundle (schema 2) is regenerated under v0.4 (schema 4), the encoded ms1 / mk1 / md1 card strings are byte-identical to the v0.2 emission; only the JSON envelope shape differs (ms1 wrapped in 1-element array; schema_version "4"). Engraving card text is also byte-identical for these carry-forward cases (single-sig full, watch-only multisig with distinct cosigners).

## §5.7 Verify-bundle conditional guarantee (TIGHTENED to 9 / 3+6N parity)

v0.3's "3-element coarse ladder" for descriptor-mode is REPLACED by full 9 / 3+6N parity. Both template-mode and descriptor-mode emit the same check schema:

- **Single-sig (N=1, secret-bearing):** **9 checks** total: `ms1_decode(1) + ms1_entropy_match(1) + mk1_decode(1) + mk1_xpub_match(1) + mk1_fingerprint_match(1) + mk1_path_match(1) + md1_decode(1) + md1_wallet_policy(1) + md1_xpub_match(1) = 9`.
- **Multisig (N>1):** **3 + 6N checks** total = 3 shared (`md1_decode` + `md1_wallet_policy` + `md1_xpub_match`) + 6 per cosigner (`ms1_decode[i]` + `ms1_entropy_match[i]` + `mk1_decode[i]` + `mk1_xpub_match[i]` + `mk1_fingerprint_match[i]` + `mk1_path_match[i]`). The "6N" already includes per-slot ms1 checks; schema 4's per-slot ms1 array is what drives the 2 ms1 checks per slot in the 6N count. The `md1_xpub_match` check passes if and only if the multiset of expected pubkeys equals the multiset of decoded md1 pubkeys (i.e., set-equality with multiplicity, sort-then-compare). Multiplicity matters because `wsh(multi(K,@0,@0))` (degenerate) would otherwise compare equal to `wsh(multi(K,@0,@1))`.
- **Per-cosigner ms1 semantics (v0.5 four-case table).** The `ms1_decode[i]` and `ms1_entropy_match[i]` checks divide into four cases:
  1. **Watch-only slot** (`expected.ms1[i] == ""`): both checks pass-vacuously regardless of whether the user supplied an `ms1[i]` value. `passed: true`, all forensic fields null, `decode_error = "skipped: watch-only slot"`. (Supplying `--ms1[i]` to a watch-only slot is silently absorbed.)
  2. **Full-mode slot, supplied present, decodes successfully** (`expected.ms1[i]` non-empty AND `supplied.ms1[i]` non-empty AND `ms_codec::decode(supplied)` returns Ok): substantive byte-equality comparison. `ms1_decode[i]: passed=true`. `ms1_entropy_match[i]` passes if `supplied == expected` byte-for-byte; otherwise fails with `expected/actual/diff_byte_offset` populated.
  3. **Full-mode slot, supplied present but malformed** (decode returns Err): `ms1_decode[i]: passed=false, decode_error=<error message>`. `ms1_entropy_match[i]: passed=true, decode_error="skipped: ms1 decode failed"` (cascade-skip; cannot byte-compare a non-decoding payload).
  4. **Full-mode slot, supplied absent** (`expected.ms1[i]` non-empty AND `supplied.ms1[i]` empty/missing): both checks `passed: false`. For `ms1_decode[i]`: `decode_error = "error: ms1[{i}] expected (full-mode bundle) but not supplied"` (byte-exact, no period). For `ms1_entropy_match[i]`: `decode_error = "skipped: ms1[{i}] not supplied"`.
- **Pure watch-only multisig** (all slots xpub-only, ms1 all empty): same 3+6N schema; all per-cosigner ms1 checks short-circuit per case 1.
- **`wif` slot handling:** verify-bundle treats `wif` slots as watch-only for ms1 check purposes (`expected.ms1[i] == ""`, both ms1 checks short-circuit per case 1). The mk1 card for a wif slot is still a real engraved card; `mk1_decode[i]` and `mk1_xpub_match[i]` checks run normally against the supplied wif's derived public point.
- **Multisig cosigner-mapping diagnostic.** When a supplied `--mk1` card cannot be associated with a cosigner index, the verify-bundle helper distinguishes three failure modes:
  - **Card not supplied** — no `--mk1` group decoded successfully into a card mapped to slot `i`. `mk1_decode[i]: passed=false, decode_error="skipped: mk1[{i}] not supplied"`.
  - **Card supplied but does not decode** — a `--mk1` group exists for the slot but `mk_codec::decode` rejects it. `mk1_decode[i]: passed=false, decode_error=<mk_codec error message>`.
  - **Card xpub not in policy** — a supplied `--mk1` group decoded but its xpub is absent from the descriptor's pubkeys-TLV (wrong-key attack indicator or user supplied a card from a different wallet). `mk1_decode[i]: passed=false, decode_error="supplied mk1 card xpub absent from descriptor policy"`.

  In all three cases, the dependent `mk1_xpub_match[i]` / `mk1_fingerprint_match[i]` / `mk1_path_match[i]` checks cascade-skip with `passed: true, decode_error="skipped: mk1[{i}] decode failed"` (vacuous-skip semantics — these checks have no oracle to evaluate against).

  *Implementation note (informative):* the helper requires an intermediate per-slot `MappingFailure` enum tracking `{NotSupplied, DecodeFailed(String), XpubNotInPolicy}` to disambiguate at emission time. Precedence when multiple failure modes apply: `XpubNotInPolicy > DecodeFailed > NotSupplied`.

### §5.7 Per-cell forensic diagnostics (NEW)

Mismatch identifies the failing field within a card, not just the failing card. JSON shape:

```json
{
  "name": "mk1_xpub_match[1]",
  "passed": false,
  "expected": "xpub6...abc",
  "actual": "xpub6...xyz",
  "diff_byte_offset": 31
}
```

`diff_byte_offset` is the first byte position where expected and actual differ (UTF-8 byte index in the encoded string).

**Forensic field rules:**

- **Pass cases** (`passed: true`): all forensic fields are `null`.
- **String-mismatch checks** (where both sides decode but produce different content): all four forensic fields populated.
- **Decode-failure checks** (e.g., `ms1_decode`, `mk1_decode`, `md1_decode`, `md1_wallet_policy`):
  ```json
  {
    "name": "ms1_decode[0]",
    "passed": false,
    "expected": null,
    "actual": null,
    "diff_byte_offset": null,
    "decode_error": "invalid checksum at position 73"
  }
  ```
  `expected: null`, `actual: null`, `diff_byte_offset: null`. Optional `decode_error: <error message>` field carries the underlying decode error string (best-effort; from md-codec / mk-codec / ms-codec error types).
- **Length-mismatch** (e.g., `mk1_path_match[1]` where the supplied and expected path strings have different lengths): `diff_byte_offset` set to `min(len(expected), len(actual))`; full strings in `expected`/`actual`.

## §6.6 Mode-violation pre-check ladder (v0.5 — legacy flags fully deleted)

v0.5 deletes the legacy CLI flags entirely. The unified `--slot @N.<subkey>=<value>` syntax is the sole input shape for slot-bearing data. The mode-violation ladder is correspondingly trimmed.

**Deleted flags (v0.4.2 alias-routed; v0.5 hard-removed):** `--phrase`, `--xpub`, `--cosigner`, `--master-fingerprint`, `--cosigner-count`, `--cosigners-file`. Clap rejects them as unknown args (exit 2 with the standard "unknown argument" message).

**Retained first-class flags (NOT deleted; serve distinct non-slot purposes):** `--template`, `--network`, `--account`, `--language`, `--passphrase`, `--threshold`, `--multisig-path-family`, `--privacy-preserving`, `--no-engraving-card`, `--self-check`, `--descriptor`, `--descriptor-file`, `--bundle-json` (verify-bundle only), `--ms1`, `--mk1`, `--md1`, `--json`, `--slot` (bundle only).

**Trap deletion:** the v0.4.2 `detect_removed_subcommand` trap (which courtesy-rejected `bundle multisig-full` etc.) is deleted. The clap fallback (unknown subcommand → exit 2) is the surviving rejection path.

| Row | Condition | Exit | Stderr (byte-exact) |
|---|---|---|---|
| 2 | Both `--template` and `--descriptor` supplied | 2 | `error: --template and --descriptor are mutually exclusive` |
| 3 | Neither `--template` nor `--descriptor` supplied AND no slot inputs | 2 | `error: missing --template or --descriptor` |
| 4 | A slot has both secret-bearing subkey (`{phrase\|entropy\|xprv\|wif}`) and watch-only subkey (`{xpub}`) | 2 | `error: slot @{N} has both secret-bearing input and watch-only input; pick one per slot.` |
| 8 | Slot indices have a gap (e.g., `@0` and `@2` supplied but `@1` absent) | 2 | `error: slot indices must be contiguous starting at @0; missing @{i}` |
| 9 | Threshold T > N or T < 1 | 2 | `error: threshold {T} out of range for N={N} cosigners (must be 1..={N})` |
| 9.5 | Multisig template mode AND `--threshold` absent | 2 | `error: --threshold required for multisig template '{template}'` |
| 10 | Single-sig template (e.g., `wpkh`) with N > 1 slots | 2 | `error: single-sig template '{template}' incompatible with N={N} slots; use a multisig template or --descriptor` |
| 11 | Multisig template with N == 1 | 2 | `error: multisig template '{template}' requires N > 1; use a single-sig template for N=1` |
| 12 | Descriptor mode AND `--threshold` supplied (descriptor encodes its own threshold) | 2 | `error: --threshold conflicts with --descriptor (descriptor encodes its own threshold)` |
| 13 | BIP-388 violation: any two slots resolve to the same (xpub, path) tuple per §4.11.b typed-DerivationPath equality | 2 | `error: BIP-388 distinct-key violation: slot @{i} and slot @{j} resolve to identical (xpub, path)` |
| 14 | Per-`@N` annotation inconsistency (descriptor mode) | 2 | `error: descriptor placeholder @{N} has inconsistent annotations across occurrences` |
| T1 | `--threshold` supplied with single-sig `--template` (no multisig context) | 2 | `error: --threshold supplied but --template '{template}' is single-sig` |
| T2 | `--multisig-path-family` supplied with single-sig `--template` | 2 | `error: --multisig-path-family supplied but --template '{template}' is single-sig` |

Rows 2-12 fire pre-synthesis; rows 13-14 fire post-binding. Rows T1-T2 are the v0.5 retained guards on first-class flags that conflict with single-sig template selection.

**Removed in v0.5 (rows 1, 5, 6, 7 from v0.4):** the trap row, `--cosigner-count` consistency, `--phrase`/`--slot` conflict, `--cosigner`/`--slot` conflict — all gated legacy flags that no longer exist.

### §6.6.a Legacy flag deletion (v0.5 — table removed)

The v0.4.2 alias mapping table (`--phrase X` → `--slot @0.phrase=X`, etc.) is removed in v0.5 because the legacy flags are themselves removed. Users must invoke the unified `--slot @N.<subkey>=<value>` syntax directly.

### §6.6.b Per-slot subkey-set validity matrix

Each slot's subkey set must be one of the following exact shapes (any other combination → exit 2 row 4):

- `{phrase}` or `{phrase, passphrase}` (passphrase is v0.5+) → secret-bearing, BIP-39 derivation
- `{entropy}` → secret-bearing, raw entropy
- `{xprv}` → secret-bearing, xpriv-direct
- `{xpub}`, `{xpub, fingerprint}`, `{xpub, path}`, `{xpub, fingerprint, path}` → watch-only with origin metadata at the granularity supplied
- `{wif}` → degenerate single-key (no ms1, no extended-key derivation)

**Conflict cases** (REJECT exit 2 row 4): any slot with both a secret-bearing subkey AND a watch-only subkey.

**Mixed-types-across-slots is fine (hybrid):** slot 0 with `{phrase}` and slot 1 with `{xpub, fingerprint, path}` is a legitimate hybrid (own seed + watch-only cosigner) — auto-detected as hybrid mode.

## §6.7 verify-bundle CLI grammar (NEW)

`mnemonic verify-bundle` is the v0.4 verify-bundle command. CLI accepts:

```
mnemonic verify-bundle [--template X | --descriptor Y] \
    --slot @N.<subkey>=<value> ... \
    [ --ms1 <ms1-card> --mk1 <mk1-card> [--mk1 <mk1-card> ...] --md1 <md1-card>
    | --bundle-json <path> ] \
    --network <mainnet|testnet|signet|regtest> \
    [--threshold T] \
    [--json] \
    [--privacy-preserving]
```

**v0.4.3 §6.7 amendment — `--bundle-json <path>`:** v0.4.3 adds an alternative card-input form: `--bundle-json <path>` reads a JSON-envelope bundle (the output of `bundle --json`) from the named file and extracts the supplied ms1/mk1/md1 from its envelope (same wire bytes as if the user had typed `--ms1 ... --mk1 ... --md1 ...` directly). Mutually exclusive with the explicit `--ms1` / `--mk1` / `--md1` flag triplet (clap `conflicts_with`). The re-derivation inputs (`--slot` / `--phrase` / `--xpub` / `--cosigner` / etc.) are STILL required — `--bundle-json` only supplies the `supplied` side of the verification; the `expected` side is re-derived from user inputs as before. Schema-version dispatch: `--bundle-json` only accepts schema-4 envelopes in v0.4.3; schema-2/3 retro-compat intake is tracked at FOLLOWUP `bundle-json-schema-2-3-retro-compat` and deferred to v0.4.4+ pending real need.

**Flag semantics:**

- `--slot @N.<subkey>=<value>`: re-derivation inputs (same `--slot @N.<subkey>=<value>` vocabulary as `bundle`; v0.5 sole input shape). Used to RE-derive the expected ms1/mk1/md1 from user inputs for cross-comparison against the supplied cards.
- `--bundle-json <path>` (v0.4.3+): supplies `--ms1`/`--mk1`/`--md1` from a JSON envelope file. Mutually exclusive with the explicit triplet. Schema-4 only in v0.4.3.
- `--ms1 <card>`: supplied ms1 card(s).
  - **Schema 4 (v0.4 bundles): `--ms1` MUST repeat exactly N times** (where N = number of slots, derived from `--slot @N` indices), in slot index order. The CLI shape mirrors the JSON `MsField` shape verbatim (1:1 positional correspondence with `ms1[i]`). For watch-only slots, the empty-string sentinel is supplied as a literal CLI argument: `--ms1 ""`. Example: `--ms1 "ms1abc..." --ms1 "" --ms1 "ms1xyz..."` (slot 0 secret-bearing, slot 1 watch-only, slot 2 secret-bearing). NO inference from `--slot @N.<subkey>=` presence; verify-bundle does NOT skip absent flags or auto-fill empty strings — the user is required to be explicit. Mismatch (`len(--ms1) != N`) triggers row 16 below.
  - **Schemas 2 and 3 (v0.2 / v0.3 bundles, ms1 = flat string)**: `--ms1 <card>` accepts exactly ONE value (the flat-string ms1). Repeating `--ms1` under schema 2/3 is a CLI error (exit 2 with `error: --ms1 may appear at most once for schema-2/3 bundles`).
- `--mk1 <card>`: supplied mk1 cards. Always repeats per slot under all schemas (mirrors v0.2 behavior).
- `--md1 <card>`: ONE supplied md1 card.
- `--threshold T`: optional in template mode (re-derived from descriptor / template); required only if user wants to assert the threshold matches. Conflict with descriptor mode per §6.6 row 12.
- `--privacy-preserving`: re-derivation suppresses fingerprints (matches `bundle --privacy-preserving`); on a privacy-preserving bundle, expected fingerprints are not generated, and `mk1_fingerprint_match` checks pass-vacuously.

**Mode determination for verify-bundle:** identical to `bundle` — auto-detect from per-slot subkeys + presence of `--template` vs `--descriptor`. Schema-version dispatch happens AFTER mode determination: `verify_bundle::run_*` reads `schema_version` from the supplied `--ms1`/`--mk1`/`--md1` (decoding header bits) and routes to schema-2 / schema-3 / schema-4 handler.

**verify-bundle pre-check ladder:** SAME 14 rows as §6.6 plus the schema-4-specific rows below. Pre-check fires before re-derivation.

| Row | Condition | Exit | Stderr |
|---|---|---|---|
| 15 | `len(--mk1)` ≠ `len(--ms1)` | 4 | `error: bundle schema-4 length invariant violated: --mk1 count {mk1_count} ≠ --ms1 count {ms1_count}` |
| 16 | `len(--mk1)` ≠ derived N from slot indices | 4 | `error: bundle schema-4 length invariant violated: expected N={derived_N} cards; got {actual_count}` |
| 17 | Bundle violates BIP-388 distinct-key rule (per re-binding) | 4 | `error: bundle violates BIP-388 distinct-key rule; regenerate with distinct keys` |

verify-bundle's exit codes: `0` (all checks pass), `4` (bundle decode failure or check failure or invariant violation; details in `checks` array under `--json`).

## §6.9 Byte-exact error text reference

Rows 1-14 in §6.6 plus rows 15-17 in §6.7 are the canonical byte-exact texts. Implementation must use these strings verbatim (consts, not format strings except for the bracketed substitutions). SPEC author copies into impl directly; tests assert byte-exact.

## §8 Out of scope (DELTA)

Carry-forward from v0.3 §8 plus v0.4 additions:

- **`tr-sortedmulti-a-via-upstream`** (v0.3.2 cleanup) — drop `[patch.crates-io]` once miniscript publishes a post-#910+#915 release. Not gated on v0.4.
- **`walker-backport-to-md-cli`** (v0.4-cross-repo) — md-cli backport of toolkit's expanded walker (now includes v0.3.0-NEW arms + v0.3.1 sortedmulti_a + v0.4 multi-leaf walker). Cross-repo coordination cycle pending.
- **Per-slot `passphrase` subkey** (v0.5+) — BIP-39 passphrase per slot. v0.4 supports global `--passphrase` only.
- **Per-slot `language` subkey** (v0.5+) — BIP-39 wordlist per slot. v0.4 supports global `--language` only.
- **K-of-N share encoding** (gated on ms-codec v0.2; multi-week external dependency).
- **`--output <dir>`** (per-card files instead of stdout sections). Future cycle.
- **Recovery flow** (3 strings → wallet artifact). Future cycle.
- **SLIP-39 / non-BIP-39 secret formats** (future).

## §9 CHANGELOG

### v0.4.0 — 2026-05-05 (foundation release; full v0.4 deliverables in v0.4.0 + v0.4.1)

**Note:** v0.4.0 ships the BIP-388 hard-reject + `--slot` CLI surface + multi-leaf taproot walker + foundation primitives (MsField type, BundleMode helpers). The full schema-4 cutover, multi-source synthesis wiring, unified engraving card, and verify-bundle 9/3+6N forensics are deferred to v0.4.1 — see `design/FOLLOWUPS.md` entries `bundle-json-schema-4-cutover`, `engraving-card-unified-1-master-card`, `verify-bundle-9-3plus6n-forensics`. The breaking-changes and new-features lists below describe the full v0.4 cycle (v0.4.0 + v0.4.1 combined).

**Breaking changes:**

1. **Subcommands removed:** `bundle multisig-full` and `bundle multisig-watch-only` removed entirely (no deprecation aliases). Use unified `bundle` with `--slot @N.<subkey>=<value>` inputs; mode auto-detected.
2. **BIP-388 distinct-key rule enforced** (exit 2 at bundle creation, exit 4 at verify-bundle, symmetric). v0.2 self-multisig artifacts (any bundle produced by `multisig-full --cosigner-count > 1`) now fail both creation and verification.
3. **`BundleJson` schema bump** `"3"` → `"4"`. `ms1` field migrates `Option<String>` → `MsField` (= `Vec<String>`, length-N invariant, dense with empty-string placeholders for watch-only slots).
4. **`--cosigner-count` flag deprecated.** N derived from slot indices; conflict if K ≠ max(slot_indices)+1.
5. **`--cosigner` and standalone `--phrase` flags deprecated** as aliases mapping to `--slot @N.<subkey>=` per §6.6.a.

**New features:**

- **Multi-source full multisig:** N seeds → N distinct (ms1, mk1) pairs + 1 md1. Each cosigner has its own secret card.
- **Hybrid mode:** mix secret-bearing slots (own seed) with watch-only slots (cosigner xpubs) in template mode. Previously descriptor-mode-only.
- **`@N`-pattern unified CLI:** single `--slot @N.<subkey>=<value>` flag across multisig-full, multisig-watch-only, and descriptor-mode template binding. Subkey vocabulary: `phrase`, `entropy`, `xpub`, `fingerprint`, `path`, `wif`, `xprv`.
- **Multi-leaf taproot:** `tr(K, {leaf1, leaf2, ...})` with ≥2 leaves now SUPPORTED (was deferred in v0.3).
- **Verify-bundle 9 / 3+6N parity for descriptor mode:** v0.3's 3-element coarse ladder replaced with full 9 / 3+6N schema matching template mode.
- **Per-cell forensic diagnostics:** verify-bundle JSON now includes `expected`, `actual`, `diff_byte_offset`, `decode_error` fields per `VerifyCheck` (mismatch identifies the failing field within a card, not just the failing card).
- **Schema-4 dispatch in verify-bundle:** schemas 2, 3, 4 all supported.
- **1 master engraving card per bundle** with new unified layout supporting all bundle shapes (single-sig / multisig / hybrid / descriptor).

**Bug fixes / cleanups:**

- Deleted dead `SELF_MULTISIG_WARNING` constant family (both `cmd/bundle.rs:639` and `parse_descriptor.rs:1054` stale duplicate).
- Deleted `check_self_multisig_warning` function (replaced by `check_key_vector_distinctness`).
- Removed stale `#[allow(dead_code)]` on `synthesize.rs::CosignerKeyInfo`.
- Cleaned up stale `ctx_for_descriptor` comment in `parse_descriptor.rs`.

**Cross-phase invariant carry:**

- v0.2 single-sig full + multisig-watch-only fixtures continue to pass byte-identically (subject to new schema-4 envelope wrapping per §5.6).
- v0.2 multisig-full fixtures EXCLUDED from byte-identical regression matrix (BIP-388 violation).
- v0.3 fixtures continue to pass byte-identically except cells violating BIP-388 distinctness (enumerated in §10).

## §10 Fixture corpus + exclusions

**v0.4 new fixtures:** ≥40 cells covering:

- Single-sig full (carry from v0.2 logic; new schema-4 envelope)
- Multi-source full multisig (the new flagship; covers wsh-sortedmulti and tr-sortedmulti-a templates × 3 cosigners × different SlotSubkey input combinations)
- Watch-only multisig (carry from v0.2 logic with distinct cosigners; new schema-4 envelope)
- Hybrid mode (one seed + watch-only cosigners; per-slot SlotSubkey variation)
- Multi-leaf taproot (≥2-leaf, ≥3-leaf shapes; descriptor mode)
- Per-SlotSubkey input type coverage (phrase, entropy, xpub, fingerprint, path, wif, xprv at least one fixture per subkey)
- BIP-388 violation rejection cases (creation-time + verify-time symmetric)

**v0.2 fixture exclusions** (excluded from byte-identical regression matrix; documented exhaustively in Phase A):

- All cells from `bundle multisig-full --cosigner-count > 1` invocations (the deprecated self-multisig pattern). Phase A greps tests/vectors/v0_2/ for these cells.
- Single-sig full cells: carry forward (no exclusion)
- multisig-watch-only cells: carry forward (no exclusion; these already use distinct keys per BIP-388)

**v0.3 fixture exclusions:**

- Any descriptor-mode cells where the descriptor causes two slots to resolve to identical (xpub, path) tuples. Phase A audits the v0.3 fixture corpus.
- Most v0.3 fixtures expected to carry forward unchanged.

## §11 Release process (carry-forward)

Carry-forward from v0.3 §11 unchanged. v0.4-specific gates: cycle plan exit criteria 1-10 must all be green; v0.4 fixture SHA pin computed and recorded before tag push; user approval at the v0.4.0 tag-push gate per the autonomous-mode handoff.
