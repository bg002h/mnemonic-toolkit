# `mk encode` {#mk-encode}

Encode an extended public key (xpub) plus origin metadata
(master fingerprint, derivation path) plus at least one
`policy_id_stub` into one or more `mk1` strings. The largest
mk-tab subcommand: 9 flags + 0 positionals. The encoder may emit
a single `mk1` (regular BCH variant) or split the payload across
multiple chunks (chunked-with-long-leading-code variant) when the
single-string size would exceed the BCH budget.

Conditional input rule: `--origin-fingerprint` and
`--privacy-preserving` are mutually exclusive — enforced by a
runtime guard at `crates/mk-cli/src/cmd/encode.rs:58-62` (the
clap `--arg` attributes do NOT carry a `conflicts_with`, despite
the doc-comments' "Mutually exclusive with …" phrasing). The
conditional-visibility engine at
`mnemonic-gui/src/form/conditional::mk_encode` mirrors the
runtime guard: setting either flag disables the other.

Additional runtime requirement: at least one of
`--policy-id-stub` (raw 4-byte hex stub) **or** `--from-md1`
(derive the stub from a supplied `md1` per SPEC §3.5.1) must be
present. The GUI does not enforce this at form-fill time — the
clap surface marks neither as required individually — so the
runtime UsageError surfaces if both are empty.

## Outline {#mk-encode-outline}

- [`--xpub`](#mk-encode-xpub) — BIP-32 extended public key (required)
- [`--origin-fingerprint`](#mk-encode-origin-fingerprint) — master fingerprint (XOR with `--privacy-preserving`)
- [`--origin-path`](#mk-encode-origin-path) — BIP-32 derivation path (required)
- [`--policy-id-stub`](#mk-encode-policy-id-stub) — explicit 4-byte hex policy-id stub (repeating)
- [`--from-md1`](#mk-encode-from-md1) — derive `--policy-id-stub` from an `md1` (repeating)
- [`--privacy-preserving`](#mk-encode-privacy-preserving) — omit fingerprint from the mk1 (XOR with `--origin-fingerprint`)
- [`--force-chunked`](#mk-encode-force-chunked) — force chunked output even when single-string would fit (reserved for v0.2)
- [`--force-long-code`](#mk-encode-force-long-code) — force long-code BCH variant (reserved for v0.2)
- [`--json`](#mk-encode-json) — emit a single JSON object instead of one mk1 string per line

## `--xpub` {#mk-encode-xpub}

The BIP-32 extended public key to encode. **Required.** Plain
Text widget. Accepts the canonical xpub-prefixed form
(`xpub6…` for mainnet; `tpub…` for testnet/signet/regtest; the
mk1 format encodes the network indicator internally). The mk1
encoder embeds the 33-byte compressed pubkey + 32-byte chaincode
from the parsed BIP-32 envelope.

## `--origin-fingerprint` {#mk-encode-origin-fingerprint}

The master-key fingerprint, 8 lowercase hex chars (= 4 bytes).
Optional. Mutually exclusive with
[`--privacy-preserving`](#mk-encode-privacy-preserving) — the
conditional-visibility engine `Disables` whichever flag the user
did not select.

When omitted (and `--privacy-preserving` is also omitted), the
encoded mk1 carries `origin_fingerprint = None`. At the wire
format the two omission paths produce identical bytes — per
`encode.rs:77-82` both `(None, true)` and `(None, false)` resolve
to the same `KeyCard::new(...)` call argument. The
`--privacy-preserving` flag exists to make operator intent
explicit and to gate the runtime mutual-exclusion check; it does
not change the encoded bytes for an already-omitted fingerprint.

## `--origin-path` {#mk-encode-origin-path}

The BIP-32 derivation path from the master xprv to the supplied
xpub. **Required.** Plain Text widget. Accepts the conventional
form `m/<index>'/<index>'/...` where `'` (or `h`) marks
hardened children. Example: `m/84'/0'/0'` for a single-sig
mainnet BIP-84 account.

The encoder validates the path against mk-codec's path-indicator
dictionary; unknown or unrepresentable paths produce
`error: invalid path indicator …` or `error: path too deep …` at
exit 2.

## `--policy-id-stub` {#mk-encode-policy-id-stub}

Explicit 4-byte policy-id stub, supplied as 8 lowercase hex
chars. **Repeating** — pass once per stub. The GUI renders this
as a multi-row text widget; the cross-chunk binding header
preserves stub order.

At least one stub MUST be present (either supplied directly via
this flag or derived from a supplied `--from-md1`). The runtime
emits `error: at least one of --policy-id-stub or --from-md1 is
required` (exit 64) if both are empty.

## `--from-md1` {#mk-encode-from-md1}

Derive a `--policy-id-stub` from a supplied `md1` string. The
encoder calls `md_codec::policy_id_stub_from_md1` per SPEC §3.5.1
to compute the stub from the md1's encoded wallet-policy template
bytes. **Repeating** — pass once per md1.

Mixing `--policy-id-stub` and `--from-md1` is allowed; the
encoder concatenates them in supplied-order (explicit stubs
first, derived stubs second; per
`crates/mk-cli/src/cmd/encode.rs:64-70`). The resulting stub
order is the order the mk1 records; downstream
[`mk verify`](#mk-verify) is order-sensitive.

## `--privacy-preserving` {#mk-encode-privacy-preserving}

Boolean. Omits the master fingerprint from the encoded `mk1`.
Default off. Mutually exclusive with
[`--origin-fingerprint`](#mk-encode-origin-fingerprint).

Use case: cold-card scenarios where the master fingerprint is
considered metadata-leakage (it bridges this card to other cards
under the same master) and the operator prefers a wallet-policy-only
binding via `--policy-id-stub` or `--from-md1`. The decoded card
shows `origin_fingerprint: (omitted, privacy-preserving mode)`
for these.

## `--force-chunked` {#mk-encode-force-chunked}

Boolean. **Reserved for v0.2 — no-op at v0.3.1.** mk-codec
auto-dispatches the chunked-vs-single decision based on payload
size at v0.3.1 (see help string at
`crates/mk-cli/src/cmd/encode.rs:42-43`). The flag remains in the
schema for forward-compatibility.

## `--force-long-code` {#mk-encode-force-long-code}

Boolean. **Reserved for v0.2 — no-op at v0.3.1.** mk-codec
auto-dispatches the BCH-code variant (regular vs long) based on
chunk count today. The flag remains in the schema for
forward-compatibility.

## `--json` {#mk-encode-json}

Boolean. Emit a single JSON object on stdout (fields:
`schema_version`, `mk1_strings`, `chunk_count`, `code_variant`)
instead of one mk1 string per stdout line. Default off.

The `code_variant` field reports the first chunk's variant
(`regular` for single-chunk emission; `long` for chunked
emission's leading chunk).

## Worked example — typical mainnet mk1 with fingerprint

1. **mk** tab; pick **Encode (xpub → mk1)**.
2. `--xpub`:
   `xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a`
3. `--origin-fingerprint`: `deadbeef`
4. `--origin-path`: `m/84'/0'/0'`
5. `--policy-id-stub`: `c0ffee00`
6. **Run**. No run-confirm modal — `mk encode` operates on
   public material.

The output panel emits two `mk1` strings, one per chunk, to
stdout (the payload exceeds the single-chunk size budget so the
encoder picks chunked emission):

```text
mk1qpydzkpqqsq…    (leading chunk; long BCH variant)
mk1qpydzkpp…       (trailing chunk; regular BCH variant)
```

The exact 5-char prefix-after-`mk1qp` differs per invocation
because the encoder generates a fresh 4-byte `chunk_set_id` (the
cross-chunk binding header) each time; the pinned fixture's
strings (chunk_set_id `144470`) are shown verbatim in
[§71](#mk-per-tab-reference). Decode the freshly-emitted pair via
[`mk decode`](#mk-decode) to confirm the round-trip.

## Worked example — privacy-preserving mode

1. **mk** tab; **Encode** subcommand.
2. `--xpub`: same canonical xpub.
3. `--privacy-preserving`: checked.
4. Note that the conditional-visibility engine `Disables`
   `--origin-fingerprint` once `--privacy-preserving` is set.
5. `--origin-path`: `m/84'/0'/0'`.
6. `--policy-id-stub`: `c0ffee00`.
7. **Run**.

The output panel emits the chunked pair under privacy-preserving
mode; decode via `mk decode` to confirm the
`origin_fingerprint` field reads `(omitted, privacy-preserving
mode)`.

## Refusals

| Trigger | Refusal |
|---|---|
| `--xpub` missing | clap-level `required` error |
| `--origin-path` missing | clap-level `required` error |
| Both `--origin-fingerprint` and `--privacy-preserving` supplied | exit 64 with `error: --privacy-preserving and --origin-fingerprint are mutually exclusive` (runtime guard at `encode.rs:58-62`; no clap `conflicts_with`, so the runtime always fires when both are present) |
| Neither `--policy-id-stub` nor `--from-md1` supplied | exit 64 with `error: at least one of --policy-id-stub or --from-md1 is required` |
| `--policy-id-stub <value>` not 8 lowercase hex chars | exit 64 with `error: …` per `parse_stub_hex` |
| `--from-md1 <value>` not a parseable md1 | exit 2 with `error: md1 input rejected: …` per `md-codec` |
| `--origin-fingerprint <value>` not 8 hex chars | exit 64 with `error: …` per `parse_fingerprint` |
| `--xpub <value>` not a parseable BIP-32 extended public key | exit 64 with `error: …` per `parse_xpub` |
| `--origin-path <value>` not a parseable BIP-32 derivation path string | exit 64 with `error: …` per `parse_derivation_path` (caught before mk-codec dispatch) |
| `--origin-path <value>` parses but is rejected by mk-codec's path-indicator dictionary | exit 2 with `error: invalid path indicator …` or `error: path too deep …` |
