# `mnemonic decode-address` {#mnemonic-decode-address}

Decode a Bitcoin address string into its facts: the network(s) it is
valid for, script type, witness version, and scriptPubKey. A
public-data utility — no secrets, no key material; the inverse of
`convert --to address`. The GUI exposes this as one form under the
**mnemonic** tab's subcommand selector; the address itself is supplied
as a positional argument (the form's primary input), with a single
`--json` toggle.
\index{mnemonic decode-address}

The address layer cannot disambiguate testnet / testnet4 / signet
(shared `tb1` and base58 prefixes), so `networks` reports the full set
the address is valid for; `regtest` (`bcrt1`) is distinct. An
unparseable address exits non-zero.

> **GUI form:** see [GUI Forms › mnemonic › decode-address](#gui-form-mnemonic-decode-address).

## `--json` {#mnemonic-decode-address-json}

Boolean flag. Emit a JSON envelope instead of the human-readable
block. The envelope carries `networks` (the valid-for set),
`script_type` (`p2pkh`/`p2sh`/`p2wpkh`/`p2wsh`/`p2tr`),
`witness_version` (segwit only; absent for legacy), and
`script_pubkey` (hex). The GUI renders this as a checkbox.

## Worked example — decode a P2WPKH address

1. Switch to the **mnemonic** tab; pick **decode-address** in the
   subcommand selector.
2. Type or paste the address into the positional `<ADDRESS>` field.
3. Optionally toggle `--json`.
4. Click **Run** (no run-confirm modal — an address is public).

The output reports `networks`, `script_type`, `witness_version`
(segwit only), and `script_pubkey`.
