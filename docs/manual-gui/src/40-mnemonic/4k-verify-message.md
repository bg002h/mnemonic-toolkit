# `mnemonic verify-message` {#mnemonic-verify-message}

**Verify** a Bitcoin message signature (verification only — `mnemonic`
never signs). Two formats are supported and partition cleanly by
address type: **legacy** "Bitcoin Signed Message" (the
`signmessage`/`verifymessage` format, P2PKH only) and
[BIP-322](https://github.com/bitcoin/bips/blob/master/bip-0322.mediawiki)
**simple** (P2WPKH / P2SH-P2WPKH / P2TR). The GUI exposes this as one
form under the **mnemonic** tab's subcommand selector.
\index{mnemonic verify-message}

This command takes no secret material — an address, a message, and a
signature are all public. Output is the verification verdict on stdout
(text line or JSON). A valid signature exits 0; a cleanly-decoded
signature that simply does not verify exits 1 with `valid: false` on
stdout; malformed input exits 1 with an error on stderr.

> **GUI form:** see [GUI Forms › mnemonic › verify-message](#gui-form-mnemonic-verify-message).

## Outline {#mnemonic-verify-message-outline}

- [`--address`](#mnemonic-verify-message-address) — the address the message was signed by
- [`--format`](#mnemonic-verify-message-format) — signature format (default `auto`)
- [`--json`](#mnemonic-verify-message-json) — emit JSON envelope instead of text
- [`--message`](#mnemonic-verify-message-message) — the signed message, inline (exact bytes)
- [`--message-file`](#mnemonic-verify-message-message-file) — read the message from a file
- [`--message-stdin`](#mnemonic-verify-message-message-stdin) — read the message from stdin
- [`--signature`](#mnemonic-verify-message-signature) — the signature (base64)

## `--address` {#mnemonic-verify-message-address}

The address the message was signed by. Its type selects the format
under `--format auto`: P2PKH → legacy, segwit/taproot → BIP-322. The
GUI renders this as a Text widget.

## `--format` {#mnemonic-verify-message-format}

The signature format. Default `auto` — legacy for P2PKH, BIP-322
otherwise. `--format legacy` on a non-P2PKH address is refused (legacy
verification is P2PKH-only). Three allowed values. The GUI renders
this as a Dropdown widget.

### Outline {#mnemonic-verify-message-format-outline}

- [`auto`](#mnemonic-verify-message-format-auto)
- [`legacy`](#mnemonic-verify-message-format-legacy)
- [`bip322`](#mnemonic-verify-message-format-bip322)

### `auto` {#mnemonic-verify-message-format-auto}

Choose the format by address type: P2PKH → legacy,
segwit/taproot → BIP-322. The default.

### `legacy` {#mnemonic-verify-message-format-legacy}

The legacy "Bitcoin Signed Message" format
(`signmessage`/`verifymessage`). **P2PKH only** — a non-P2PKH address
is refused. The signature is a 65-byte recoverable signature (base64).

### `bip322` {#mnemonic-verify-message-format-bip322}

BIP-322 *simple* — P2WPKH / P2SH-P2WPKH / P2TR. The signature is a
BIP-322 witness encoding (base64). Taproot script-path and
arbitrary-script (BIP-322 *full*) signatures are not yet covered.

## `--json` {#mnemonic-verify-message-json}

Boolean flag. Emit a JSON envelope instead of the human-readable
line. The GUI renders this as a checkbox.

## `--message` {#mnemonic-verify-message-message}

The signed message, inline (exact bytes). Exactly one of `--message` /
`--message-file` / `--message-stdin` is required. The GUI renders this
as a Text widget.

## `--message-file` {#mnemonic-verify-message-message-file}

Read the message from a file (a single trailing newline is stripped).
One of the required message-source group. The GUI renders this as a
Path widget.

## `--message-stdin` {#mnemonic-verify-message-message-stdin}

Boolean flag. Read the message from stdin (a single trailing newline
is stripped). One of the required message-source group. The GUI
renders this as a checkbox.

## `--signature` {#mnemonic-verify-message-signature}

The signature (base64): a 65-byte recoverable signature (legacy) or a
BIP-322 witness encoding. The GUI renders this as a Text widget.

## Worked example — verify a legacy P2PKH signature

1. Switch to the **mnemonic** tab; pick **verify-message** in the
   subcommand selector.
2. Paste the signing address into `--address`, the exact message into
   `--message`, and the base64 signature into `--signature`.
3. Leave `--format` at `auto`.
4. Click **Run**.

A valid signature exits 0 with the verdict line; an invalid (but
well-formed) signature exits 1 with `valid: false`; malformed input —
a bad address, an undecodable signature, or `--format legacy` on a
non-P2PKH address — exits 1 with an error on stderr.
