# `mnemonic electrum-decrypt` {#mnemonic-electrum-decrypt}

Decrypt an Electrum **field-encrypted** secret — a base64
`iv ‖ aes-256-cbc(plaintext + PKCS7)` blob whose key is
`sha256d(password)` per Electrum's version-1 password hashing — and emit
the recovered plaintext. The plaintext is an Electrum-native seed phrase
or a BIP-32 xprv (the keystore type determines which; the wire carries
no discriminator, so the output is emitted opaquely). The GUI exposes
the toolkit's `electrum-decrypt` subcommand as a flat **Electrum
Decrypt** form on the `mnemonic` tab. Exactly one password form is
required.

:::danger
The worked example decrypts a throwaway demonstration blob. **Never
engrave or fund** any wallet recovered from demonstration ciphertext.
The recovered plaintext on stdout is private key material; the
`--decrypt-password` field is secret-bearing. The run-confirm modal
redacts the secret-bearing password argv token as a fixed `••••`
sentinel (see [§14 Defense 2](#secret-handling)). For a whole-file
encrypted Electrum wallet (Format B) use [`mnemonic
import-wallet`](#mnemonic-import-wallet), not this subcommand.
:::

> **GUI form:** see [GUI Forms › mnemonic › electrum-decrypt](#gui-form-mnemonic-electrum-decrypt).

## Outline {#mnemonic-electrum-decrypt-outline}

- [`--ciphertext`](#mnemonic-electrum-decrypt-ciphertext) — the base64 field-encrypted value (`-` reads from stdin; required)
- [`--decrypt-password`](#mnemonic-electrum-decrypt-decrypt-password) — decryption password inline (leaks via argv; prefer the file/stdin forms)
- [`--decrypt-password-file`](#mnemonic-electrum-decrypt-decrypt-password-file) — read the password from a file
- [`--decrypt-password-stdin`](#mnemonic-electrum-decrypt-decrypt-password-stdin) — read the password from stdin
- [`--json-out`](#mnemonic-electrum-decrypt-json-out) — write a JSON envelope to PATH instead of plain stdout

## `--ciphertext` {#mnemonic-electrum-decrypt-ciphertext}

Text field. The Electrum field-encrypted secret as base64. Required.
Use `-` to read the ciphertext from stdin. This value is NOT secret (it
is ciphertext), so the GUI renders it as a plain text field with no
argv-leakage advisory. `--ciphertext -` consumes the single stdin
channel, so it is mutually exclusive with `--decrypt-password-stdin`.

## `--decrypt-password` {#mnemonic-electrum-decrypt-decrypt-password}

Text field (masked). The decryption password supplied inline. The GUI
renders this as a `SecretLineEdit`; a non-empty value triggers the
run-confirm modal and emits an argv-leakage advisory — prefer
[`--decrypt-password-file`](#mnemonic-electrum-decrypt-decrypt-password-file)
or [`--decrypt-password-stdin`](#mnemonic-electrum-decrypt-decrypt-password-stdin).
The three password forms are mutually exclusive and exactly one is
required (clap arg-group; missing or multiple → exit 64).

## `--decrypt-password-file` {#mnemonic-electrum-decrypt-decrypt-password-file}

Path widget. Read the decryption password from a file (one trailing
newline stripped). The OS file picker is not yet wired (FOLLOWUP
`gui-file-picker-affordance`); the field accepts a path string. Keeps
the password off argv. One of the three mutually-exclusive password
forms.

## `--decrypt-password-stdin` {#mnemonic-electrum-decrypt-decrypt-password-stdin}

Boolean flag. When set, the password is read from stdin (raw bytes,
NULL-byte preserving). Keeps the password off argv. Single stdin per
invocation — mutually exclusive with `--ciphertext -`. One of the three
mutually-exclusive password forms; the schema marks the flag
secret-bearing because its presence implies a stdin secret.

## `--json-out` {#mnemonic-electrum-decrypt-json-out}

Path widget. When set, the toolkit writes a JSON envelope
(`{schema_version, operation, plaintext}`; no password echo) to the
given path instead of emitting plaintext on stdout. Emits a
world-readable-permissions advisory if the target file is group- or
other-readable. The written file holds private key material — emit it
only to encrypted, access-controlled storage.

## Failure mode

A wrong password (or corrupted ciphertext) surfaces as
`electrum-decrypt: decryption failed (wrong password or corrupted
ciphertext)` (exit 1). Field encryption carries no MAC, so the two
underlying failure modes (PKCS7 unpad refusal / non-UTF-8 result) are
reported uniformly — the GUI shows this string in the output panel's
stderr region. A successful decrypt emits the recovered plaintext on
stdout with the private-key-material output advisory.

## Worked example — decrypt with a stdin password

1. Switch to the **mnemonic** tab; pick **Electrum Decrypt** in the
   subcommand selector.
2. Paste the base64 ciphertext into the `--ciphertext` plain field:

   ```text
   ABEiM0RVZneImaq7zN3u/zY0181f7qAY/NWiVQFLdHE=
   ```

3. Tick `--decrypt-password-stdin` (so the password stays off argv) and
   supply the password through the form's stdin channel.
4. The `Preview:` line resembles:

   ```text
   mnemonic electrum-decrypt --ciphertext ABEi…dHE= --decrypt-password-stdin
   ```

5. Click **Run**; redact-confirm in the modal.

The output panel renders the recovered plaintext on stdout. Because the
plaintext is private key material, redirect or encrypt it rather than
leaving it in the output panel; the output-class advisory fires on
stderr.

\index{mnemonic electrum-decrypt}
