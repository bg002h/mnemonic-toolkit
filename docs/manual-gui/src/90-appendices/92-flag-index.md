# Flag index {#flag-index}

Alphabetical cross-reference for every schema-enumerated flag
across the four CLIs, grouped by subcommand. Source of truth is
`mnemonic-gui/src/schema/{mnemonic,md,ms,mk}.rs`. The leading
`--` is omitted in the link text. Click through for the full
syntax and refusal table at the matching per-flag section.

## `mnemonic` tab

10 subcommands. The per-subcommand chapters are at
[§42](#mnemonic-bundle) through [§4b](#mnemonic-slip39-combine).

### `mnemonic bundle` {#flag-index-mnemonic-bundle}

[`--account`](#mnemonic-bundle-account) ·
[`--descriptor`](#mnemonic-bundle-descriptor) ·
[`--descriptor-file`](#mnemonic-bundle-descriptor-file) ·
[`--json`](#mnemonic-bundle-json) ·
[`--language`](#mnemonic-bundle-language) ·
[`--multisig-path-family`](#mnemonic-bundle-multisig-path-family) ·
[`--network`](#mnemonic-bundle-network) ·
[`--no-engraving-card`](#mnemonic-bundle-no-engraving-card) ·
[`--passphrase`](#mnemonic-bundle-passphrase) ·
[`--passphrase-stdin`](#mnemonic-bundle-passphrase-stdin) ·
[`--privacy-preserving`](#mnemonic-bundle-privacy-preserving) ·
[`--self-check`](#mnemonic-bundle-self-check) ·
[`--slot`](#mnemonic-bundle-slot) ·
[`--template`](#mnemonic-bundle-template) ·
[`--threshold`](#mnemonic-bundle-threshold)

### `mnemonic verify-bundle` {#flag-index-mnemonic-verify-bundle}

[`--account`](#mnemonic-verify-bundle-account) ·
[`--bundle-json`](#mnemonic-verify-bundle-bundle-json) ·
[`--descriptor`](#mnemonic-verify-bundle-descriptor) ·
[`--descriptor-file`](#mnemonic-verify-bundle-descriptor-file) ·
[`--json`](#mnemonic-verify-bundle-json) ·
[`--language`](#mnemonic-verify-bundle-language) ·
[`--md1`](#mnemonic-verify-bundle-md1) ·
[`--mk1`](#mnemonic-verify-bundle-mk1) ·
[`--ms1`](#mnemonic-verify-bundle-ms1) ·
[`--multisig-path-family`](#mnemonic-verify-bundle-multisig-path-family) ·
[`--network`](#mnemonic-verify-bundle-network) ·
[`--passphrase`](#mnemonic-verify-bundle-passphrase) ·
[`--passphrase-stdin`](#mnemonic-verify-bundle-passphrase-stdin) ·
[`--privacy-preserving`](#mnemonic-verify-bundle-privacy-preserving) ·
[`--slot`](#mnemonic-verify-bundle-slot) ·
[`--template`](#mnemonic-verify-bundle-template) ·
[`--threshold`](#mnemonic-verify-bundle-threshold)

### `mnemonic convert` {#flag-index-mnemonic-convert}

[`--account`](#mnemonic-convert-account) ·
[`--bip38-passphrase`](#mnemonic-convert-bip38-passphrase) ·
[`--bip38-passphrase-stdin`](#mnemonic-convert-bip38-passphrase-stdin) ·
[`--electrum-language`](#mnemonic-convert-electrum-language) ·
[`--electrum-version`](#mnemonic-convert-electrum-version) ·
[`--fingerprint`](#mnemonic-convert-fingerprint) ·
[`--from`](#mnemonic-convert-from) ·
[`--json`](#mnemonic-convert-json) ·
[`--language`](#mnemonic-convert-language) ·
[`--network`](#mnemonic-convert-network) ·
[`--passphrase`](#mnemonic-convert-passphrase) ·
[`--passphrase-stdin`](#mnemonic-convert-passphrase-stdin) ·
[`--path`](#mnemonic-convert-path) ·
[`--script-type`](#mnemonic-convert-script-type) ·
[`--template`](#mnemonic-convert-template) ·
[`--to`](#mnemonic-convert-to) ·
[`--xpub-prefix`](#mnemonic-convert-xpub-prefix)

### `mnemonic export-wallet` {#flag-index-mnemonic-export-wallet}

[`--account`](#mnemonic-export-wallet-account) ·
[`--bitcoin-core-version`](#mnemonic-export-wallet-bitcoin-core-version) ·
[`--descriptor`](#mnemonic-export-wallet-descriptor) ·
[`--format`](#mnemonic-export-wallet-format) ·
[`--language`](#mnemonic-export-wallet-language) ·
[`--multisig-path-family`](#mnemonic-export-wallet-multisig-path-family) ·
[`--network`](#mnemonic-export-wallet-network) ·
[`--output`](#mnemonic-export-wallet-output) ·
[`--range`](#mnemonic-export-wallet-range) ·
[`--slot`](#mnemonic-export-wallet-slot) ·
[`--taproot-internal-key`](#mnemonic-export-wallet-taproot-internal-key) ·
[`--template`](#mnemonic-export-wallet-template) ·
[`--threshold`](#mnemonic-export-wallet-threshold) ·
[`--timestamp`](#mnemonic-export-wallet-timestamp) ·
[`--wallet-name`](#mnemonic-export-wallet-wallet-name)

### `mnemonic derive-child` {#flag-index-mnemonic-derive-child}

[`--application`](#mnemonic-derive-child-application) ·
[`--dice-sides`](#mnemonic-derive-child-dice-sides) ·
[`--from`](#mnemonic-derive-child-from) ·
[`--index`](#mnemonic-derive-child-index) ·
[`--language`](#mnemonic-derive-child-language) ·
[`--length`](#mnemonic-derive-child-length) ·
[`--network`](#mnemonic-derive-child-network) ·
[`--passphrase`](#mnemonic-derive-child-passphrase) ·
[`--passphrase-stdin`](#mnemonic-derive-child-passphrase-stdin)

### `mnemonic final-word` {#flag-index-mnemonic-final-word}

[`--from`](#mnemonic-final-word-from) ·
[`--json-out`](#mnemonic-final-word-json-out) ·
[`--language`](#mnemonic-final-word-language)

### `mnemonic seed-xor-split` {#flag-index-mnemonic-seed-xor-split}

[`--deterministic-from-master`](#mnemonic-seed-xor-split-deterministic-from-master) ·
[`--from`](#mnemonic-seed-xor-split-from) ·
[`--json-out`](#mnemonic-seed-xor-split-json-out) ·
[`--language`](#mnemonic-seed-xor-split-language) ·
[`--shares`](#mnemonic-seed-xor-split-shares)

### `mnemonic seed-xor-combine` {#flag-index-mnemonic-seed-xor-combine}

[`--json-out`](#mnemonic-seed-xor-combine-json-out) ·
[`--language`](#mnemonic-seed-xor-combine-language) ·
[`--share`](#mnemonic-seed-xor-combine-share) ·
[`--shares`](#mnemonic-seed-xor-combine-shares)

### `mnemonic slip39-split` {#flag-index-mnemonic-slip39-split}

[`--from`](#mnemonic-slip39-split-from) ·
[`--group`](#mnemonic-slip39-split-group) ·
[`--group-threshold`](#mnemonic-slip39-split-group-threshold) ·
[`--iteration-exponent`](#mnemonic-slip39-split-iteration-exponent) ·
[`--json-out`](#mnemonic-slip39-split-json-out) ·
[`--language`](#mnemonic-slip39-split-language) ·
[`--passphrase`](#mnemonic-slip39-split-passphrase) ·
[`--passphrase-stdin`](#mnemonic-slip39-split-passphrase-stdin)

### `mnemonic slip39-combine` {#flag-index-mnemonic-slip39-combine}

[`--json-out`](#mnemonic-slip39-combine-json-out) ·
[`--language`](#mnemonic-slip39-combine-language) ·
[`--passphrase`](#mnemonic-slip39-combine-passphrase) ·
[`--passphrase-stdin`](#mnemonic-slip39-combine-passphrase-stdin) ·
[`--share`](#mnemonic-slip39-combine-share) ·
[`--to`](#mnemonic-slip39-combine-to)

## `md` tab

8 subcommands. The per-subcommand chapters are at
[§52](#md-inspect) through [§59](#md-address).

### `md inspect` {#flag-index-md-inspect}

[`--json`](#md-inspect-json)

### `md encode` {#flag-index-md-encode}

[`--context`](#md-encode-context) ·
[`--fingerprint`](#md-encode-fingerprint) ·
[`--force-chunked`](#md-encode-force-chunked) ·
[`--force-long-code`](#md-encode-force-long-code) ·
[`--from-policy`](#md-encode-from-policy) ·
[`--json`](#md-encode-json) ·
[`--key`](#md-encode-key) ·
[`--network`](#md-encode-network) ·
[`--path`](#md-encode-path) ·
[`--policy-id-fingerprint`](#md-encode-policy-id-fingerprint) ·
[`--unspendable-key`](#md-encode-unspendable-key)

### `md decode` {#flag-index-md-decode}

[`--json`](#md-decode-json)

### `md verify` {#flag-index-md-verify}

[`--fingerprint`](#md-verify-fingerprint) ·
[`--json`](#md-verify-json) ·
[`--key`](#md-verify-key) ·
[`--network`](#md-verify-network) ·
[`--template`](#md-verify-template)

### `md bytecode` {#flag-index-md-bytecode}

[`--json`](#md-bytecode-json)

### `md vectors` {#flag-index-md-vectors}

[`--out`](#md-vectors-out)

### `md compile` {#flag-index-md-compile}

[`--context`](#md-compile-context) ·
[`--json`](#md-compile-json) ·
[`--unspendable-key`](#md-compile-unspendable-key)

### `md address` {#flag-index-md-address}

[`--chain`](#md-address-chain) ·
[`--change`](#md-address-change) ·
[`--count`](#md-address-count) ·
[`--fingerprint`](#md-address-fingerprint) ·
[`--index`](#md-address-index) ·
[`--json`](#md-address-json) ·
[`--key`](#md-address-key) ·
[`--network`](#md-address-network) ·
[`--template`](#md-address-template)

## `ms` tab

5 subcommands. The per-subcommand chapters are at
[§62](#ms-inspect) through [§66](#ms-vectors).

### `ms inspect` {#flag-index-ms-inspect}

[`--json`](#ms-inspect-json)

### `ms encode` {#flag-index-ms-encode}

[`--hex`](#ms-encode-hex) ·
[`--json`](#ms-encode-json) ·
[`--language`](#ms-encode-language) ·
[`--no-engraving-card`](#ms-encode-no-engraving-card) ·
[`--phrase`](#ms-encode-phrase)

### `ms decode` {#flag-index-ms-decode}

[`--json`](#ms-decode-json) ·
[`--language`](#ms-decode-language)

### `ms verify` {#flag-index-ms-verify}

[`--json`](#ms-verify-json) ·
[`--language`](#ms-verify-language) ·
[`--phrase`](#ms-verify-phrase)

### `ms vectors` {#flag-index-ms-vectors}

[`--pretty`](#ms-vectors-pretty)

## `mk` tab

5 subcommands. The per-subcommand chapters are at
[§72](#mk-inspect) through [§76](#mk-vectors).

### `mk inspect` {#flag-index-mk-inspect}

[`--json`](#mk-inspect-json)

### `mk encode` {#flag-index-mk-encode}

[`--force-chunked`](#mk-encode-force-chunked) ·
[`--force-long-code`](#mk-encode-force-long-code) ·
[`--from-md1`](#mk-encode-from-md1) ·
[`--json`](#mk-encode-json) ·
[`--origin-fingerprint`](#mk-encode-origin-fingerprint) ·
[`--origin-path`](#mk-encode-origin-path) ·
[`--policy-id-stub`](#mk-encode-policy-id-stub) ·
[`--privacy-preserving`](#mk-encode-privacy-preserving) ·
[`--xpub`](#mk-encode-xpub)

### `mk decode` {#flag-index-mk-decode}

[`--json`](#mk-decode-json)

### `mk verify` {#flag-index-mk-verify}

[`--from-md1`](#mk-verify-from-md1) ·
[`--json`](#mk-verify-json) ·
[`--origin-fingerprint`](#mk-verify-origin-fingerprint) ·
[`--origin-path`](#mk-verify-origin-path) ·
[`--policy-id-stub`](#mk-verify-policy-id-stub) ·
[`--xpub`](#mk-verify-xpub)

### `mk vectors` {#flag-index-mk-vectors}

[`--out`](#mk-vectors-out) ·
[`--pretty`](#mk-vectors-pretty)
