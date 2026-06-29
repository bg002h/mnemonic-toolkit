# `mk verify` {#mk-verify}

Verify that one or more `mk1` strings decode cleanly (BCH check
passes, payload-format is valid) and optionally that their
decoded content matches user-supplied expected fields. **Exit 0**
on success; **exit 2** on decode failure; **exit 3** on
future-format (`UnsupportedVersion`); **exit 4** on content
mismatch; **exit 64** on user-input error.

The content-match flags are all optional. A bare `mk verify
<mk1-strings>` exits 0 iff the strings decode cleanly. Supplying
any of `--xpub` / `--origin-fingerprint` / `--origin-path` /
`--policy-id-stub` / `--from-md1` adds an equality assertion on
the matching decoded field; mismatch returns exit 4 with the
field name and the actual-vs-expected values.

> **GUI form:** see [GUI Forms › mk › verify](#gui-form-mk-verify).

## Outline {#mk-verify-outline}

- [`--xpub`](#mk-verify-xpub) — expected xpub (exit 4 on mismatch)
- [`--origin-fingerprint`](#mk-verify-origin-fingerprint) — expected master fingerprint
- [`--origin-path`](#mk-verify-origin-path) — expected derivation path
- [`--policy-id-stub`](#mk-verify-policy-id-stub) — expected stub(s); repeating, order-sensitive
- [`--from-md1`](#mk-verify-from-md1) — derive expected `--policy-id-stub` from an `md1`; repeating
- [`--json`](#mk-verify-json) — emit JSON envelope on stdout

## `--xpub` {#mk-verify-xpub}

Expected extended public key. Optional. When supplied, the
binary parses it via `parse_xpub` and compares against the
decoded card's xpub field. Mismatch returns exit 4 with
`error: verify mismatch on xpub: expected <expected>, got
<actual>` (per `crates/mk-cli/src/error.rs:65-75` ContentMismatch
formatting).

## `--origin-fingerprint` {#mk-verify-origin-fingerprint}

Expected master fingerprint, 8 lowercase hex chars. Optional.
The comparison handles the privacy-preserving case: if the
decoded card has `origin_fingerprint: None`, the mismatch
message reports `actual: (omitted, privacy-preserving mode)`
(per `crates/mk-cli/src/cmd/verify.rs:74-80`).

## `--origin-path` {#mk-verify-origin-path}

Expected BIP-32 derivation path (e.g. `m/84'/0'/0'`). Optional.
Compared against the decoded card's `origin_path` after parsing
via `parse_derivation_path`. Mismatch returns exit 4.

## `--policy-id-stub` {#mk-verify-policy-id-stub}

Expected `policy_id_stub`, 8 lowercase hex chars. **Repeating**;
**order-sensitive**. Compared element-wise against the decoded
card's `policy_id_stubs` array. Mismatch emits the comma-joined
expected vs actual lists.

May be combined with [`--from-md1`](#mk-verify-from-md1); the two
flag sets concatenate in supplied-order (explicit stubs first,
md1-derived stubs second) per `crates/mk-cli/src/cmd/verify.rs:95-101`.

## `--from-md1` {#mk-verify-from-md1}

Derive an expected `--policy-id-stub` from a supplied `md1`
string. **Repeating**; order-sensitive (appended after
`--policy-id-stub` values).

## `--json` {#mk-verify-json}

Boolean. Emit a JSON envelope on stdout. On success: a single
JSON object with `schema_version`, `ok: true`, `chunks`, and
`policy_id_stubs`. On failure: the standard `mk-cli` error
envelope on stdout (the mismatch-detail rides in the envelope's
`details` field per `error.rs:91-105`).

## Positional `mk1-strings`

One or more `mk1` strings. **Repeating** positional. The literal
`-` causes the binary to read one string per line from stdin
until EOF.

## Worked example — bare verify (no content-match)

1. **mk** tab; pick **Verify (mk1 content-match)**.
2. Paste both canonical mk1 strings into the `mk1-strings` field
   (one per row).
3. Leave all `--xpub` / `--origin-*` / `--policy-id-stub` /
   `--from-md1` fields empty.
4. **Run**.

The output panel emits exit 0 and stdout:

```{.text include="75-mk-verify-bare.out"}
OK: mk1 string(s) decode cleanly (and any --xpub / --origin-* / --policy-id-stub / --from-md1 inputs match)
```

(The suffix mentions the content-match flags unconditionally per
`cmd/verify.rs:128-137`; the trailing parenthetical is literal
even when no content-match flags were supplied.)

## Worked example — content-match all fields

1. **mk** tab; **Verify** subcommand.
2. Paste both canonical mk1 strings.
3. `--xpub`: paste the canonical xpub
   (`xpub6BmeGmRo4Los…oCp2z6a`).
4. `--origin-fingerprint`: `deadbeef`.
5. `--origin-path`: `m/84'/0'/0'`.
6. `--policy-id-stub`: `c0ffee00`.
7. **Run**.

All four fields match the decoded values; stdout emits the same
`OK:` line and exit 0. Changing any field to a non-matching
value (e.g. `--origin-fingerprint cafebabe`) exits 4 with stderr
`error: verify mismatch on origin_fingerprint: expected cafebabe,
got deadbeef`.

## Refusals

| Trigger | Refusal |
|---|---|
| No positional `mk1-strings` provided AND stdin not used | clap-level `required` error |
| Any positional that does not parse as `mk1` | exit 2 with the matching `mk-codec` error |
| Supplied chunks have inconsistent `chunk_set_id` headers | exit 2 with `error: chunk_set_id mismatch` |
| `mk1` future-format (`UnsupportedVersion`) | exit 3 with `error: unsupported version …` |
| `--xpub <value>` does not match the decoded card | **exit 4** with `error: verify mismatch on xpub: expected <expected>, got <actual>` |
| `--origin-fingerprint <value>` does not match the decoded card | **exit 4** with `error: verify mismatch on origin_fingerprint: …` (privacy-preserving mode reports `actual: (omitted, privacy-preserving mode)`) |
| `--origin-path <value>` does not match | **exit 4** with `error: verify mismatch on origin_path: …` |
| `--policy-id-stub` / `--from-md1` produces an expected stub list that does not equal the decoded `policy_id_stubs` (order-sensitive) | **exit 4** with `error: verify mismatch on policy_id_stubs: expected <comma-joined>, got <comma-joined>` |
| `--xpub` / `--origin-fingerprint` / `--policy-id-stub` / `--from-md1` value fails its own parse (bad hex, malformed xpub, etc.) | exit 64 with `error: …` per the matching `parse_*` helper |
