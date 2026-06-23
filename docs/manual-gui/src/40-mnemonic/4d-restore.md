# `mnemonic restore` {#mnemonic-restore}

Take **secret seed material + an optional BIP-39 passphrase** and emit
a **watch-only "restore document"**: a verification block leading with
the master fingerprint (the passphrase-correctness oracle) and the
first receive address(es), followed by the concrete single-sig
descriptor(s) for BIP-44/49/84/86. Restore is **read-only /
watch-only-out** — it emits xpub / fingerprint / addresses / descriptor
only and NEVER any private material (`xprv` / WIF). It does not sign.

Two modes. *Single-sig* (the default, `--from <seed>`) emits the
BIP-44/49/84/86 descriptors. *Multisig-cosigner* (`--md1 <card>`)
reconstructs the concrete watch-only multisig descriptor from the
shared wallet-policy `md1` card alone — the card already carries every
cosigner's public key, so `--from` / `--cosigner` are *optional
cross-check* inputs, not build inputs. A **keyless multisig / general
TEMPLATE `md1`** (no concrete keys, from
[`bundle --md1-form=template`](#mnemonic-bundle-md1-form-template)) is
*not* a refusal — it is **completed** by re-supplying the keys via the
`--from` / `--cosigner` / `--account` and search flags below.

\index{mnemonic restore}

:::danger
The worked example uses the canonical all-`abandon` BIP-39 test vector.
**Never engrave or fund** a wallet derived from this phrase — chain
watchers have swept it continuously since 2017. The run-confirm modal
redacts secret-bearing argv tokens — the `--from` own-seed and any
`--passphrase` — as a fixed `••••` sentinel, so the literal secret is
never drawn on screen (see [§14 Defense 2](#secret-handling) for the
masking semantics and the residual flag-name exposure). Restore is
watch-only-out: no `xprv` / WIF / seed reaches stdout, stderr, or
`--json`. The cold/airgapped operational practice remains good hygiene
for every secret-bearing invocation.
:::

## Outline {#mnemonic-restore-outline}

- [`--from`](#mnemonic-restore-from) — seed source `ms1=`/`phrase=`/`entropy=`/`seedqr=` (required for single-sig)
- [`--format`](#mnemonic-restore-format) — importable wallet-software payload via an `export-wallet` emitter
- [`--template`](#mnemonic-restore-template) — restrict to a single single-sig wallet type
- [`--network`](#mnemonic-restore-network) — Bitcoin network (default `mainnet`)
- [`--language`](#mnemonic-restore-language) — BIP-39 wordlist for `phrase=` / `seedqr=` (default `english`)
- [`--account`](#mnemonic-restore-account) — BIP-32 account index(es) (default 0)
- [`--count`](#mnemonic-restore-count) — number of first-receive addresses to show per wallet type (default 1)
- [`--origin`](#mnemonic-restore-origin) — explicit BIP-32 origin path for a keyless single-sig template `md1`
- [`--expect-wallet-id`](#mnemonic-restore-expect-wallet-id) — expected `WalletPolicyId` hex prefix for template-completion
- [`--expect-fingerprint`](#mnemonic-restore-expect-fingerprint) — reference master fingerprint (mismatch → exit 4)
- [`--expect-xpub`](#mnemonic-restore-expect-xpub) — reference account xpub (requires `--template`)
- [`--allow-mismatch`](#mnemonic-restore-allow-mismatch) — emit descriptors even when a reference does not match
- [`--passphrase`](#mnemonic-restore-passphrase) — BIP-39 mnemonic-extension passphrase (XOR with `--passphrase-stdin`)
- [`--passphrase-stdin`](#mnemonic-restore-passphrase-stdin) — read `--passphrase` from stdin
- [`--output`](#mnemonic-restore-output) — write the stdout content to a file (`-` = stdout, default)
- [`--json`](#mnemonic-restore-json) — emit a single structured JSON object on stdout
- [`--md1`](#mnemonic-restore-md1) — the shared wallet-policy `md1` card chunk(s) (multisig mode; repeating)
- [`--cosigner`](#mnemonic-restore-cosigner) — cross-check / unassigned cosigner key (multisig mode; repeating)
- [`--own-account-max`](#mnemonic-restore-own-account-max) — RANGE fallback for the own seed's account(s)
- [`--search-cosigner-subset`](#mnemonic-restore-search-cosigner-subset) — opt-in bounded cosigner-subset search
- [`--search-address`](#mnemonic-restore-search-address) — a known address triggers address-search for template completion
- [`--search-addr-min`](#mnemonic-restore-search-addr-min) — inclusive lower address index for `--search-address` (default 0)
- [`--search-addr-max`](#mnemonic-restore-search-addr-max) — exclusive upper address index for `--search-address` (default 20)
- [`--search-chain`](#mnemonic-restore-search-chain) — which BIP-32 change-chain `--search-address` scans
- [`--accept-search-time`](#mnemonic-restore-accept-search-time) — override the adaptive search-time ceiling

## `--from` {#mnemonic-restore-from}

The seed source, in `<node>=<value>` form. One of `ms1=<v>` /
`phrase=<v>` / `entropy=<hex>` / `seedqr=<digits>`; the value supports
`@env:VAR` and `-` (stdin). Non-seed nodes (`xpub` / `xprv` / `wif` /
…) are refused — restore needs a master secret. REQUIRED for single-sig
restore **and for multisig-template completion** (the OWN seed);
OPTIONAL in keyed-multisig (`--md1`) mode, where it cross-checks the
own cosigner position (inferred by matching the derived key against the
md1's slots).

The GUI renders this as a NodeValueComposite widget (a Dropdown
selecting the node + a value field). Schema-`secret: true` — the value
is the master secret. The value field renders as a `SecretLineEdit`;
any non-empty value triggers the run-confirm modal, where the token is
masked as `••••`. Suffix `=-` reads from stdin.

## `--format` {#mnemonic-restore-format}

Emit an importable wallet-software payload via an
[`export-wallet`](#mnemonic-export-wallet) emitter, instead of the
plain restore document. Dropdown; eleven values. For single-sig this
REQUIRES a single `--template` (one-descriptor-in / one-out);
`--format` with no `--template` → exit 2. For multisig (`--md1`) mode
the payload class matches `export-wallet --template <multisig>
--format <X>`, built from the reconstructed cosigner keys + threshold.
When set, the importable payload goes to stdout and the verification
block goes to stderr so the payload pipes cleanly; with `--json` the
payload is embedded as the `import_payload` field. The GUI renders this
flag with a `?` help-icon.

### Outline {#mnemonic-restore-format-outline}

- [`bitcoin-core`](#mnemonic-restore-format-bitcoin-core)
- [`bip388`](#mnemonic-restore-format-bip388)
- [`coldcard`](#mnemonic-restore-format-coldcard)
- [`coldcard-multisig`](#mnemonic-restore-format-coldcard-multisig)
- [`jade`](#mnemonic-restore-format-jade)
- [`sparrow`](#mnemonic-restore-format-sparrow)
- [`specter`](#mnemonic-restore-format-specter)
- [`electrum`](#mnemonic-restore-format-electrum)
- [`green`](#mnemonic-restore-format-green)
- [`bsms`](#mnemonic-restore-format-bsms)
- [`descriptor`](#mnemonic-restore-format-descriptor)

### `bitcoin-core` {#mnemonic-restore-format-bitcoin-core}

Bitcoin Core `importdescriptors` JSON. See
[`export-wallet --format bitcoin-core`](#mnemonic-export-wallet-format-bitcoin-core).

### `bip388` {#mnemonic-restore-format-bip388}

BIP-388 wallet-policy JSON. See
[`export-wallet --format bip388`](#mnemonic-export-wallet-format-bip388).
For a general taproot policy card this format refuses (no named-template
form); such a card emits `descriptor` / `bitcoin-core` only.

### `coldcard` {#mnemonic-restore-format-coldcard}

Coldcard generic JSON. See
[`export-wallet --format coldcard`](#mnemonic-export-wallet-format-coldcard).

### `coldcard-multisig` {#mnemonic-restore-format-coldcard-multisig}

Coldcard multisig text. See
[`export-wallet --format coldcard-multisig`](#mnemonic-export-wallet-format-coldcard-multisig).

### `jade` {#mnemonic-restore-format-jade}

Blockstream Jade. See
[`export-wallet --format jade`](#mnemonic-export-wallet-format-jade).

### `sparrow` {#mnemonic-restore-format-sparrow}

Sparrow wallet JSON. See
[`export-wallet --format sparrow`](#mnemonic-export-wallet-format-sparrow).

### `specter` {#mnemonic-restore-format-specter}

Specter Desktop. In multisig restore `specter` is refused (it needs a
wallet name, which multisig restore does not take). See
[`export-wallet --format specter`](#mnemonic-export-wallet-format-specter).

### `electrum` {#mnemonic-restore-format-electrum}

Electrum's JSON wallet format. See
[`export-wallet --format electrum`](#mnemonic-export-wallet-format-electrum).

### `green` {#mnemonic-restore-format-green}

Blockstream Green. In multisig restore `green` is refused (Green has no
file-import multisig support, and explicitly refuses a general taproot
policy card because Green's file-import surface is singlesig-only). See
[`export-wallet --format green`](#mnemonic-export-wallet-format-green).

### `bsms` {#mnemonic-restore-format-bsms}

BSMS Round-2 (BIP-129). The default descriptor-driven format for the
non-taproot multisig arms. See
[`export-wallet --format bsms`](#mnemonic-export-wallet-format-bsms).

### `descriptor` {#mnemonic-restore-format-descriptor}

A bare BIP-380 descriptor. See
[`export-wallet --format descriptor`](#mnemonic-export-wallet-format-descriptor).

## `--template` {#mnemonic-restore-template}

Restrict single-sig restore to a single wallet type; omit to emit all
four (`bip44` / `bip49` / `bip84` / `bip86`). A multisig template is
refused (single-sig restore reconstructs multisig from `--md1`, not
`--template`). Same 10 values as
[`bundle --template`](#mnemonic-bundle-template); for restore only the
four single-sig values are meaningful (a multisig value refuses). The
GUI renders this flag with a `?` help-icon.

### Outline {#mnemonic-restore-template-outline}

- [`bip44`](#mnemonic-restore-template-bip44)
- [`bip49`](#mnemonic-restore-template-bip49)
- [`bip84`](#mnemonic-restore-template-bip84)
- [`bip86`](#mnemonic-restore-template-bip86)
- [`wsh-multi`](#mnemonic-restore-template-wsh-multi)
- [`wsh-sortedmulti`](#mnemonic-restore-template-wsh-sortedmulti)
- [`sh-wsh-multi`](#mnemonic-restore-template-sh-wsh-multi)
- [`sh-wsh-sortedmulti`](#mnemonic-restore-template-sh-wsh-sortedmulti)
- [`tr-multi-a`](#mnemonic-restore-template-tr-multi-a)
- [`tr-sortedmulti-a`](#mnemonic-restore-template-tr-sortedmulti-a)

### `bip44` {#mnemonic-restore-template-bip44}

See [`bundle --template bip44`](#mnemonic-bundle-template-bip44).

### `bip49` {#mnemonic-restore-template-bip49}

See [`bundle --template bip49`](#mnemonic-bundle-template-bip49).

### `bip84` {#mnemonic-restore-template-bip84}

See [`bundle --template bip84`](#mnemonic-bundle-template-bip84).

### `bip86` {#mnemonic-restore-template-bip86}

See [`bundle --template bip86`](#mnemonic-bundle-template-bip86).

### `wsh-multi` {#mnemonic-restore-template-wsh-multi}

A multisig value — refused for single-sig restore (reconstruct multisig
via `--md1`). See [`bundle --template wsh-multi`](#mnemonic-bundle-template-wsh-multi).

### `wsh-sortedmulti` {#mnemonic-restore-template-wsh-sortedmulti}

Multisig — refused for single-sig restore. See
[`bundle --template wsh-sortedmulti`](#mnemonic-bundle-template-wsh-sortedmulti).

### `sh-wsh-multi` {#mnemonic-restore-template-sh-wsh-multi}

Multisig — refused for single-sig restore. See
[`bundle --template sh-wsh-multi`](#mnemonic-bundle-template-sh-wsh-multi).

### `sh-wsh-sortedmulti` {#mnemonic-restore-template-sh-wsh-sortedmulti}

Multisig — refused for single-sig restore. See
[`bundle --template sh-wsh-sortedmulti`](#mnemonic-bundle-template-sh-wsh-sortedmulti).

### `tr-multi-a` {#mnemonic-restore-template-tr-multi-a}

Multisig — refused for single-sig restore. See
[`bundle --template tr-multi-a`](#mnemonic-bundle-template-tr-multi-a).

### `tr-sortedmulti-a` {#mnemonic-restore-template-tr-sortedmulti-a}

Multisig — refused for single-sig restore. See
[`bundle --template tr-sortedmulti-a`](#mnemonic-bundle-template-tr-sortedmulti-a).

## `--network` {#mnemonic-restore-network}

The Bitcoin network. Default `mainnet`. Same 4 values + descriptions as
[`bundle --network`](#mnemonic-bundle-network). The GUI renders this
flag with a `?` help-icon.

### Outline {#mnemonic-restore-network-outline}

- [`mainnet`](#mnemonic-restore-network-mainnet)
- [`testnet`](#mnemonic-restore-network-testnet)
- [`signet`](#mnemonic-restore-network-signet)
- [`regtest`](#mnemonic-restore-network-regtest)

### `mainnet` {#mnemonic-restore-network-mainnet}

See [`bundle --network mainnet`](#mnemonic-bundle-network-mainnet).

### `testnet` {#mnemonic-restore-network-testnet}

See [`bundle --network testnet`](#mnemonic-bundle-network-testnet).

### `signet` {#mnemonic-restore-network-signet}

See [`bundle --network signet`](#mnemonic-bundle-network-signet).

### `regtest` {#mnemonic-restore-network-regtest}

See [`bundle --network regtest`](#mnemonic-bundle-network-regtest).

## `--language` {#mnemonic-restore-language}

The BIP-39 wordlist for `phrase=` / `seedqr=` input. Default `english`.
A `mnem`-kind `ms1` carries its own wire language; a conflicting
`--language` is refused. Same 10 values as
[`bundle --language`](#mnemonic-bundle-language). The GUI renders this
flag with a `?` help-icon.

### Outline {#mnemonic-restore-language-outline}

- [`english`](#mnemonic-restore-language-english)
- [`simplifiedchinese`](#mnemonic-restore-language-simplifiedchinese)
- [`traditionalchinese`](#mnemonic-restore-language-traditionalchinese)
- [`czech`](#mnemonic-restore-language-czech)
- [`french`](#mnemonic-restore-language-french)
- [`italian`](#mnemonic-restore-language-italian)
- [`japanese`](#mnemonic-restore-language-japanese)
- [`korean`](#mnemonic-restore-language-korean)
- [`portuguese`](#mnemonic-restore-language-portuguese)
- [`spanish`](#mnemonic-restore-language-spanish)

### `english` {#mnemonic-restore-language-english}

See [`bundle --language english`](#mnemonic-bundle-language-english).

### `simplifiedchinese` {#mnemonic-restore-language-simplifiedchinese}

See [`bundle --language simplifiedchinese`](#mnemonic-bundle-language-simplifiedchinese).

### `traditionalchinese` {#mnemonic-restore-language-traditionalchinese}

See [`bundle --language traditionalchinese`](#mnemonic-bundle-language-traditionalchinese).

### `czech` {#mnemonic-restore-language-czech}

See [`bundle --language czech`](#mnemonic-bundle-language-czech).

### `french` {#mnemonic-restore-language-french}

See [`bundle --language french`](#mnemonic-bundle-language-french).

### `italian` {#mnemonic-restore-language-italian}

See [`bundle --language italian`](#mnemonic-bundle-language-italian).

### `japanese` {#mnemonic-restore-language-japanese}

See [`bundle --language japanese`](#mnemonic-bundle-language-japanese).

### `korean` {#mnemonic-restore-language-korean}

See [`bundle --language korean`](#mnemonic-bundle-language-korean).

### `portuguese` {#mnemonic-restore-language-portuguese}

See [`bundle --language portuguese`](#mnemonic-bundle-language-portuguese).

### `spanish` {#mnemonic-restore-language-spanish}

See [`bundle --language spanish`](#mnemonic-bundle-language-spanish).

## `--account` {#mnemonic-restore-account}

BIP-32 account index(es). Default 0. Single-sig restore + single-sig
template completion use one account (the first value). For **multisig
template completion** this is the comma-separated LIST of accounts the
OWN seed is used at — one own key per account (e.g. `0,1,2,3` for a
4-own-slot policy); the search places each own-derived key. Mutually
exclusive with [`--own-account-max`](#mnemonic-restore-own-account-max).
The GUI renders this as a Number widget (or a text field for the
comma-list multisig form); no `?` help-icon.

## `--count` {#mnemonic-restore-count}

Number of first-receive addresses to show per wallet type. Default 1.
The GUI renders this as a Number widget; no `?` help-icon.

## `--origin` {#mnemonic-restore-origin}

(#28) Explicit BIP-32 origin path (e.g. `m/84'/0'/7'`) for completing a
**keyless single-sig template** `md1`
([`bundle --md1-form=template`](#mnemonic-bundle-md1-form-template));
overrides the template's canonical `m/<purpose>'/<coin>'/<account>'`
default. Only meaningful for keyless single-sig template restore;
ignored otherwise. When supplied, `--expect-wallet-id` is NOT checked
(a different preimage). The GUI renders this as a Text widget.

## `--expect-wallet-id` {#mnemonic-restore-expect-wallet-id}

(#28) Expected `WalletPolicyId` hex prefix for template-completion
(single-sig phase 1 **and** multisig phase 2 id-search). Restore
recomputes the id from the completed, fully-keyed wallet and matches
its leading bytes; a **mismatch refuses loudly** (exit 4). Any-length
prefix (an advisory warns when shorter than 4 bytes — a collision
footgun). For multisig the prefix must be **strong** (sized to the
realized search space) or the search refuses an ambiguous match. **NOT**
checked when `--origin` is supplied. The GUI renders this as a Text
widget.

## `--expect-fingerprint` {#mnemonic-restore-expect-fingerprint}

Reference master fingerprint (8 lowercase hex). When the derived
material does not match → **hard error, exit 4** (`RestoreMismatch`),
the wrong-passphrase / wrong-seed guard; the verification block prints
derived-vs-expected under a `✗ MISMATCH` banner and no descriptors are
emitted (unless `--allow-mismatch`). The GUI renders this as a Text
widget.

## `--expect-xpub` {#mnemonic-restore-expect-xpub}

Reference account xpub. Requires `--template` (single-sig only).
Mismatch → exit 4 unless `--allow-mismatch`. The GUI renders this as a
Text widget.

## `--allow-mismatch` {#mnemonic-restore-allow-mismatch}

Boolean. Emit descriptors even when a reference (`--expect-fingerprint`
/ `--expect-xpub`) does not match — under a loud `✗ MISMATCH
(overridden)` stderr banner, exit 0. Use only when you know the
reference itself is wrong. The GUI renders this as a Boolean toggle.

## `--passphrase` {#mnemonic-restore-passphrase}

The BIP-39 mnemonic-extension passphrase. `@env:VAR` supported. Empty
(default) = no passphrase. The TREZOR-passphrase wallet has a different
fingerprint than the empty-passphrase wallet from the same phrase.
Schema-`secret: true`; XOR with `--passphrase-stdin`. The GUI renders
this as a `SecretLineEdit`; any non-empty value triggers the
run-confirm modal (token masked as `••••`).

## `--passphrase-stdin` {#mnemonic-restore-passphrase-stdin}

Boolean. Read the BIP-39 passphrase from stdin (raw, NULL-byte
preserving). Conflicts with `--passphrase`; mutually exclusive with
`--from <node>=-` (a single stdin per invocation — use `@env:` for one
of the two channels when both must stay off the argv). Schema-`secret:
true`. The GUI surfaces stdin routing through the secret-bearing widget.

## `--output` {#mnemonic-restore-output}

Write the stdout content to a file. Default `-` (stdout); the
verification block / banners / advisory still go to stderr. The GUI
renders this as a Path widget; `stdio_sentinel: true`.

## `--json` {#mnemonic-restore-json}

Boolean. Emit a single structured JSON object on stdout instead of the
text document. Seed material is NEVER echoed (redacted by
construction); no `xprv` / `tprv` token appears. `import_payload` is
present only when `--format` is also set. The GUI renders this as a
Boolean toggle.

## `--md1` {#mnemonic-restore-md1}

(v0.44.0; multisig mode) The shared wallet-policy `md1` card chunk(s) —
reconstructs the concrete watch-only multisig descriptor from the card
alone. Also accepts a **keyless multisig / general TEMPLATE `md1`**
([`bundle --md1-form=template`](#mnemonic-bundle-md1-form-template)),
completed via `--from` + `--account` + `--cosigner`. Repeat for chunked
cards. Covers `wsh` / `sh(wsh)`, NUMS taproot multisig, general
NUMS-taproot policies up to a depth-1 two-leaf tap tree, and non-NUMS
key-path taproot; the `@-in-both` shape (trunk key also a leaf key) or a
depth-≥2 tap tree is refused (exit 2). Watch-only (non-secret). The GUI
renders this as a Text widget with `repeating: true`.

## `--cosigner` {#mnemonic-restore-cosigner}

(v0.44.0; multisig mode) A cross-check assertion `@N=<mk1-chunk|xpub>`
— cosigner at position `N` is this public key. Repeat the same `@N=`
for each chunk of a multi-chunk `mk1`. A mismatch against the md1's
slot is a hard error (exit 4) unless `--allow-mismatch`. For
multisig-template completion the bare form (no `@N=`) supplies an
UNASSIGNED cosigner the search places; the `@N=` form assigns it
explicitly. Watch-only (non-secret). The GUI renders this as a Text
widget with `repeating: true`.

## `--own-account-max` {#mnemonic-restore-own-account-max}

(v0.70.0; #28 phase 2) RANGE fallback for the OWN seed's account(s)
when the exact accounts are unknown: derive the own seed at **every**
account in `0..K` and let the multisig-template **own-account
subset-search** select the subset actually used. Own-only — the
supplied `--cosigner` cards must be EXACT (over-supply cosigners with
[`--search-cosigner-subset`](#mnemonic-restore-search-cosigner-subset)).
Mutually exclusive with `--account` (clap `conflicts_with` —
`--own-account-max K` ALONE passes; the `--account` default is
ignored). `K ≤ 256`. The realized search space sizes the strong-prefix
requirement, so a LONGER `--expect-wallet-id` prefix (or
`--search-address`) is needed than for the exact-account path. The GUI
renders this as a Number widget.

## `--search-cosigner-subset` {#mnemonic-restore-search-cosigner-subset}

(v0.70.0; #28 phase 2) **OPT-IN bounded cosigner-subset search.** By
default (OFF) a multisig template completion requires the supplied
`--cosigner` cards to be EXACT (own-only — over-supplying cosigners
refuses). With this flag the operator MAY over-supply `--cosigner`
cards (unsure which / how many cosigners belong); the search resolves
the correct cosigner subset too. The space grows
(`S_opt = Σ_j C(K_own,j)·C(M_sup,N−j)·N!`), so a LONGER
`--expect-wallet-id` prefix is needed (a too-short prefix refuses;
`--search-address` is the recommended collision-free mode for large
opt-in pools). Bounded by the §6 hard ceiling (`S_opt ≤ 1e15`) + the
adaptive time-cap. Mutually exclusive with `--cosigner @N=`. Composes
with `--own-account-max` / `--account`. The GUI renders this as a
Boolean toggle.

## `--search-address` {#mnemonic-restore-search-address}

(#28 phase 2) A known receive (or change) ADDRESS of the wallet;
triggers **address-search** for a multisig-template completion — the
search finds the unique key→slot assignment whose scriptPubKey at some
`(chain, index)` in the range equals this address's. Recommended over
`--expect-wallet-id` (full-scriptPubKey match — collision-free). The
GUI renders this as a Text widget.

## `--search-addr-min` {#mnemonic-restore-search-addr-min}

(#28 phase 2) Inclusive lower address index for `--search-address`
(default 0). The GUI renders this as a Number widget.

## `--search-addr-max` {#mnemonic-restore-search-addr-max}

(#28 phase 2) Exclusive upper address index for `--search-address`
(default 20). Deepen (`0..20`, then `20..40`, …) if the target is not
found; a narrow range expresses "I know the index". The GUI renders
this as a Number widget.

## `--search-chain` {#mnemonic-restore-search-chain}

(#28 phase 2) Which BIP-32 change-chain branch(es) `--search-address`
scans. Dropdown; default `receive`. The GUI renders this flag with a
`?` help-icon.

### Outline {#mnemonic-restore-search-chain-outline}

- [`receive`](#mnemonic-restore-search-chain-receive)
- [`change`](#mnemonic-restore-search-chain-change)
- [`both`](#mnemonic-restore-search-chain-both)

### `receive` {#mnemonic-restore-search-chain-receive}

Scan the external (receive) chain — BIP-32 chain `0`. The default.

### `change` {#mnemonic-restore-search-chain-change}

Scan the internal (change) chain — BIP-32 chain `1`.

### `both` {#mnemonic-restore-search-chain-both}

Scan both chains (doubles the per-index search cost).

## `--accept-search-time` {#mnemonic-restore-accept-search-time}

(#28 phase 2) Override the adaptive ~1-hour search-time ceiling for a
multisig-template completion. Must be ≥ the tool's printed estimated
exhaustive time (a forced acknowledgment). Accepts a humantime duration
(e.g. `2h`, `90min`). The GUI renders this as a Text widget.

## Worked example — single-sig BIP-84 restore from the canonical phrase

1. Switch to **mnemonic** tab; pick **Restore (watch-only document)**
   in the subcommand selector.
2. In the `--from` composite, choose node `phrase` and paste the
   canonical phrase into the (masked) value field:

   ```text
   abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
   ```

3. Set `--template` to `bip84`, `--network` to `mainnet`. Optionally
   set `--expect-fingerprint` to `73c5da0a` to hard-gate the
   passphrase-correctness oracle.
4. Click **Run**. The run-confirm modal appears with the `--from`
   token masked as `••••`. Click **Run** in the modal.

The output panel renders the watch-only restore document on stdout (the
concrete `wpkh([73c5da0a/84'/0'/0']xpub6CatW…/<0;1>/*)#…` descriptor +
first receive address), with the verification block + watch-only
advisory (`note: stdout is watch-only — public keys only, cannot
spend`) on stderr. The seed is never echoed in any output mode.

A wrong `--expect-fingerprint` prints `✗ MISMATCH`, emits no
descriptor, and exits 4 — add `--allow-mismatch` to override only when
you know the reference itself is wrong.

## Refusals

| Trigger | Refusal |
|---|---|
| `--from` with a non-seed node (`xpub` / `xprv` / `wif`) | exit 1 — restore needs a master secret |
| `--format` without `--template` (single-sig) | exit 2 — `--format` needs one `--template` (one-in/one-out) |
| `--template <multisig value>` (single-sig restore) | refused — reconstruct multisig via `--md1` |
| `--expect-fingerprint` / `--expect-xpub` mismatch | exit 4 (`RestoreMismatch`) unless `--allow-mismatch` |
| `--cosigner @N=` mismatch against the md1 slot | exit 4 unless `--allow-mismatch` |
| `--md1` `@-in-both` taproot shape / depth-≥2 tap tree | exit 2 (faithful card preserved) |
| `--language` conflicting with a `mnem`-kind ms1's wire language | refused |

## Advisories

Restore emits the watch-only `note: stdout is watch-only` advisory on
every run, plus the argv-leakage advisory when a secret-bearing
`--from` value or `--passphrase` is passed inline (the GUI's preview
uses the inline form). It also emits the non-blocking consensus-masked
`older()` intake advisory in `--md1` mode (see the CLI manual). Use
`@env:VAR` or stdin to keep the seed off argv.

## See also

- [`mnemonic restore` (CLI manual)](#mnemonic-restore) — flag-by-flag
  reference, the multisig-cosigner restore walkthrough, and the
  general-policy / taproot scope matrix.
- [`mnemonic bundle --md1-form template`](#mnemonic-bundle-md1-form-template)
  — emits the keyless template `md1` that restore completes.
- [`mnemonic verify-bundle`](#mnemonic-verify-bundle) — the same
  template-completion engine, used to verify rather than reconstruct.
