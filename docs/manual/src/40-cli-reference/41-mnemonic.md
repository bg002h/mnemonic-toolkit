# `mnemonic` reference

The integration-layer CLI for the m-format constellation. Nine subcommands:
[`bundle`](#mnemonic-bundle), [`verify-bundle`](#mnemonic-verify-bundle),
[`convert`](#mnemonic-convert), [`export-wallet`](#mnemonic-export-wallet),
[`derive-child`](#mnemonic-derive-child), [`final-word`](#mnemonic-final-word),
[`seed-xor`](#mnemonic-seed-xor), [`slip39`](#mnemonic-slip39), and
[`gui-schema`](#mnemonic-gui-schema) (introspection only, no user-facing
semantics). Run any with `--help` for the latest flag set; this chapter
mirrors v0.13.0.

---

## `mnemonic bundle`

Synthesise a 3-card engraving bundle from a phrase, entropy, or
xpub. Inputs are slotted via `--slot @N.<subkey>=<value>`, repeating.

### Synopsis

```sh
mnemonic bundle --network <NETWORK> [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--network <NETWORK>` | mainnet / testnet / signet / regtest |
| `--template <TEMPLATE>` | bip44 / bip49 / bip84 / bip86 / wsh-multi / wsh-sortedmulti / sh-wsh-multi / sh-wsh-sortedmulti / tr-multi-a / tr-sortedmulti-a |
| `--descriptor <DESCRIPTOR>` | user-supplied BIP-388 descriptor; mutually exclusive with `--template` and `--descriptor-file` |
| `--descriptor-file <DESCRIPTOR_FILE>` | descriptor read from a single-line UTF-8 file; mutually exclusive with `--descriptor` |
| `--language <LANGUAGE>` | BIP-39 wordlist for the input phrase |
| `--passphrase <PASSPHRASE>` | BIP-39 mnemonic-extension passphrase |
| `--passphrase-stdin` | read `--passphrase` from stdin (raw, NULL-byte preserving); single stdin per invocation |
| `--account <ACCOUNT>` | BIP-32 account index (default 0) |
| `--json` | emit JSON output |
| `--no-engraving-card` | suppress the stderr engraving-card layout |
| `--multisig-path-family <FAMILY>` | bip48 or bip87 (default bip87) |
| `--privacy-preserving` | suppress the master fingerprint from mk1 + engraving card |
| `--self-check` | re-parse and verify the emitted bundle round-trips |
| `--threshold <THRESHOLD>` | multisig K of N (1 ≤ K ≤ N ≤ 16) |
| `--slot <SLOT>` | repeating; `@N.<subkey>=<value>` (subkey: `phrase`, `entropy`, `xpub`, `fingerprint`, `path`, `wif`, `xprv`); for secret-bearing subkeys `=-` reads from stdin |
| `--help` | print help |

### Worked example

See [Your first bundle](#your-first-bundle) for a single-sig
walkthrough; [Multi-source 2-of-3 multisig](#multi-source-2-of-3-multisig)
for multisig.

### Non-canonical descriptor mode

A descriptor is **canonical** when it matches one of the five wrapper
shapes md-codec's `canonical_origin` table recognises — `pkh(@N)`,
`wpkh(@N)`, `tr(@N)` key-path-only, `wsh(multi/sortedmulti(...))`, or
`sh(wsh(multi/sortedmulti(...)))`. Anything else — bare `wsh(@N)`,
miniscript bodies like `wsh(andor(...))`, taproot trees with leaves
(`tr(@N, <TapTree>)`), legacy `sh(sortedmulti(...))` — is
**non-canonical**.

Non-canonical descriptors typically lack per-`@N` origin paths in the
descriptor string itself. The toolkit handles this two ways:

1. **Default path inference** — when an `@N` has no inline
   `[fingerprint/path]@N` annotation AND no `--slot @N.path=` CLI input,
   the toolkit assigns the BIP-48 cosigner path
   `m/48'/<coin>'/<account>'/2'` (Liana / Specter de-facto convention).
   `<coin>` = `0'` for mainnet, `1'` for testnet/signet/regtest;
   `<account>` consumes `--account N` (defaults to `0'`). A stderr info
   notice lists the `@N` indices that received the default.
2. **Explicit per-`@N` override** — either inline BIP-380 syntax
   `[deadbeef/48'/0'/0'/2']@N` embedded in the descriptor, or
   `--slot @N.path=m/48'/0'/0'/2'` on the CLI. Either takes precedence
   over the default. The slot-CLI form is most useful when the user
   wants distinct paths per cosigner without re-typing the descriptor.

#### Example: 3-key time-locked inheritance wallet

This descriptor expresses an inheritance flow: `@0` can spend
unconditionally after Bitcoin block 12,000,000; `@1` can spend after
a 4032-block relative timelock; `@2` after 32,768 blocks. Cosigners
`@0`, `@1`, `@2` each derive at the BIP-48 default
`m/48'/0'/0'/2'` from their respective BIP-39 phrases.

:::danger
The three BIP-39 phrases below are public test vectors; chain
watchers have long since swept anything ever derived from them.
**Never engrave or fund a wallet built from these phrases.** Generate
fresh entropy for real wallets (see
[Test seeds and example data](#appendix-f-test-seeds-and-example-data)).
:::

The miniscript body is single-line; using a shell variable keeps the
recipe readable while preserving that constraint:

```sh
DESC='wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))'
```

##### Default text-form output

Running `bundle` without `--json` prints the cards directly to stdout
in a human-readable form — each card appears both as a dense
bech32-string and as a 5-character-group line break suitable for
steel engraving:

```sh
mnemonic bundle --network mainnet --account 0 \
  --descriptor "$DESC" \
  --language english \
  --slot '@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' \
  --slot '@1.phrase=legal winner thank year wave sausage worth useful legal winner thank yellow' \
  --slot '@2.phrase=letter advice cage absurd amount doctor acoustic avoid letter advice cage above'
```

Stdout (the cards — under v0.21.0+ SPEC §5.8 per-slot emission, all three cosigners now get their own ms1 card when phrases are supplied for all three slots):

```text
# ms1[0] (entropy, BCH-checksummed)
ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f

ms10e ntrsq qqqqq qqqqq qqqqq qqqqq qqqqq qqcj9 sxraq 34v7f

# ms1[1] (entropy, BCH-checksummed)
ms10entrsqplh7lml0alh7lml0alh7lml0als5cclar2zmksh6

ms10e ntrsq plh7l ml0al h7lml 0alh7 lml0a ls5cc lar2z mksh6

# ms1[2] (entropy, BCH-checksummed)
ms10entrsqzqgpqyqszqgpqyqszqgpqyqszqqlfm7mep84hunu

ms10e ntrsq zqgpq yqszq gpqyq szqgp qyqsz qqlfm 7mep8 4hunu

# mk1[0] (cosigner 0 xpub + origin)
mk1qp40rrpqqspsrg8ml5q6p7laqxs0hltnchdq5pgy3zepu88jjutthgx8egtq4pcwl6u5p2us6r6zsnl2rd0q6gghvalgymxvy4lntk6efgf0
mk1qp40rrpp8lphut2hvvpp5wl4l0mn058ndxfl63kufyfsjwlt2vkk2nlqmlvch5n4shwf72fwktdlqfhxtswupfxql3

mk1qp 40rrp qqsps rg8ml 5q6p7 laqxs 0hltn chdq5 pgy3z epu88
jjutt hgx8e gtq4p cwl6u 5p2us 6r6zs nl2rd 0q6gg hvalg ymxvy
4lntk 6efgf 0
mk1qp 40rrp p8lph ut2hv vpp5w l4l0m n058n dxfl6 3kufy fsjwl
t2vkk 2nlqm lvch5 n4shw f72fw ktdlq fhxts wupfx ql3

# mk1[1] (cosigner 1 xpub + origin)
mk1qpxj36pqqspsrg8ml5q6p7laqxs0hldcdzxlzpgy3zepal7ec5v6wv58da6c23hjuw4ypg96ztz75f8wrrussm59fetnkggq4j8pde6hkmw0
mk1qpxj36ppag0zr8gh9upnjugr26jfvunvs35jvgdjkm3kghwnt0qqymzc0utyzxyhny9pu8c56a5k72ndqgmdftljqt

mk1qp xj36p qqsps rg8ml 5q6p7 laqxs 0hldc dzxlz pgy3z epal7
ec5v6 wv58d a6c23 hjuw4 ypg96 ztz75 f8wrr ussm5 9fetn kggq4
j8pde 6hkmw 0
mk1qp xj36p pag0z r8gh9 upnju gr26j fvunv s35jv gdjkm 3kghw
nt0qq ymzc0 utyzx yhny9 pu8c5 6a5k7 2ndqg mdftl jqt

# mk1[2] (cosigner 2 xpub + origin)
mk1qpl7wlpqqspsrg8ml5q6p7laqxs0hlfgv3gqvpgy3zepugvevsxpz2zll50ju3dcmghtxtfv0y025ltk2vc8a3ex8yqncct596tqv5z420v4
mk1qpl7wlpprja893lkxup4z7tw6q2yvs4fk9pjhxf00s49ugex8rue307wdslgcj5r8x9t5j35p6p2c22v0s30tv0s2u

mk1qp l7wlp qqsps rg8ml 5q6p7 laqxs 0hlfg v3gqv pgy3z epugv
evsxp z2zll 50ju3 dcmgh txtfv 0y025 ltk2v c8a3e x8yqn cct59
6tqv5 z420v 4
mk1qp l7wlp prja8 93lkx up4z7 tw6q2 yvs4f k9pjh xf00s 49uge
x8rue 307wd slgcj 5r8x9 t5j35 p6p2c 22v0s 30tv0 s2u

# md1 (multisig wallet policy)
md1fu39yrq9qjtvyyy5jmppp9ykcggfgp9fskxcqkudsqefnfskhqqqq8uqnxnpwwqqqtggjse9txaz6v
md1fu39yrqfqqqp0npeutks2dcdzxlrzsezsqc27rchwsv0jskp2rsal4egz4ep5859pnmq8wpsfncwhr
md1fu39yrq3l4pkhsdyytkwl5z8lphut2hvvpp5wl4l0mn058ndxfl63kufyfsjwlt2v3d70kcz8a3r42
md1fu39yrqa4j5lcxlmx9ayav9mj0jj6wv58da6c23hjuw4ypg96ztz75f8wrrussm598ryfkw5ey8h6p
md1fu39yrpzw2ua7583pn5tj7qeewyp4dfykwfkgg6fxyxetdcmythf4hsqzd3v879jpmwaykdyahtr0v
md1fu39yrpgcj7vs58sls39p0l68ewgkud5t4n95k8j84204m9xvr7cunrjqfurja8939xk8j47ndpq63
md1fu39yrpha3hqdghjmksz3ry92d3gv4ejtmu9f0zxf3clxvtlnnv86xy4qee32ay5q0e3ty49zaan43

md1fu-39yrq-9qjtv-yyy5j-mppp9-ykcgg-fgp9f-skxcq-kudsq-efnfs-khqqq-q8uqn-xnpww-qqqtg-gjse9-txaz6-v
md1fu-39yrq-fqqqp-0npeu-tks2d-cdzxl-rzsez-sqc27-rchws-v0jsk-p2rsa-l4egz-4ep58-59pnm-q8wps-fncwh-r
md1fu-39yrq-3l4pk-hsdyy-tkwl5-z8lph-ut2hv-vpp5w-l4l0m-n058n-dxfl6-3kufy-fsjwl-t2v3d-70kcz-8a3r4-2
md1fu-39yrq-a4j5l-cxlmx-9ayav-9mj0j-j6wv5-8da6c-23hju-w4ypg-96ztz-75f8w-rruss-m598r-yfkw5-ey8h6-p
md1fu-39yrp-zw2ua-7583p-n5tj7-qeewy-p4dfy-kwfkg-g6fxy-xetdc-mythf-4hsqz-d3v87-9jpmw-aykdy-ahtr0-v
md1fu-39yrp-gcj7v-s58sl-s39p0-l68ew-gkud5-t4n95-k8j84-204m9-xvr7c-unrjq-furja-8939x-k8j47-ndpq6-3
md1fu-39yrp-ha3hq-dghjm-ksz3r-y92d3-gv4ej-tmu9f-0zxf3-clxvt-lnnv8-6xy4q-ee32a-y5q0e-3ty49-zaan4-3
```

Note the two-form layout per card type: the toolkit emits a dense
single-line bech32 form first (for copy-paste and machine
consumption), then a blank line, then the same content broken into
5-character groups (for steel-plate engraving and reading aloud
during verification). The grouping separators are non-load-bearing —
either form decodes back to the same payload.

Stderr (info notice + bundle-summary engraving card):

```text
info: non-canonical descriptor; defaulting origin path for @0,@1,@2 to m/48'/0'/0'/2' (BIP-48 cosigner path). Override per-placeholder with [fp/path]@N or --slot @N.path=m/...
# === Wallet bundle: descriptor, mainnet ===
# Threshold: 3 of 3
# Cosigners:
#   @0: ms1:01a0f,mk1:01a0f (73c5da0a @ 48'/0'/0'/2')
#   @1: ms1:01a0f,mk1:01a0f (b8688df1 @ 48'/0'/0'/2')
#   @2: ms1:01a0f,mk1:01a0f (28645006 @ 48'/0'/0'/2')
# Template: descriptor
# md1: 01a0
# Recovery: any 3 of 3 signing keys + md1 (template card).
# Language: english
```

The engraving-card block on stderr is a wallet-level summary the user
copies onto a separate piece of paper kept with the bundle; it lists
the threshold, per-cosigner fingerprint+origin triples, and the
recovery rule. The `01a0f` / `01a0` short tags are chunk-set-id hex
prefixes for the corresponding cards, useful when matching a
recovered card-set back to its bundle.

##### JSON envelope form (`--json`)

For programmatic consumption — and crucially for the verify-bundle
round-trip in the next section — re-run the same invocation with
`--json` and redirect stdout to a file. Stderr is unchanged.

```sh
mnemonic bundle --network mainnet --account 0 \
  --descriptor "$DESC" \
  --language english \
  --slot '@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' \
  --slot '@1.phrase=legal winner thank year wave sausage worth useful legal winner thank yellow' \
  --slot '@2.phrase=letter advice cage absurd amount doctor acoustic avoid letter advice cage above' \
  --json > /tmp/inheritance-bundle.json
```

The resulting `/tmp/inheritance-bundle.json` envelope (pretty-printed
via `python3 -m json.tool`):

```json
{
  "schema_version": "4",
  "mode": "full",
  "network": "mainnet",
  "template": null,
  "descriptor": "wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))",
  "account": 0,
  "origin_path": "m/48'/0'/0'/2'",
  "origin_paths": null,
  "master_fingerprint": null,
  "ms1": [
    "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
    "ms10entrsqplh7lml0alh7lml0alh7lml0als5cclar2zmksh6",
    "ms10entrsqzqgpqyqszqgpqyqszqgpqyqszqqlfm7mep84hunu"
  ],
  "mk1": [
    [
      "mk1qp40rrpqqspsrg8ml5q6p7laqxs0hltnchdq5pgy3zepu88jjutthgx8egtq4pcwl6u5p2us6r6zsnl2rd0q6gghvalgymxvy4lntk6efgf0",
      "mk1qp40rrpp8lphut2hvvpp5wl4l0mn058ndxfl63kufyfsjwlt2vkk2nlqmlvch5n4shwf72fwktdlqfhxtswupfxql3"
    ],
    [
      "mk1qpxj36pqqspsrg8ml5q6p7laqxs0hldcdzxlzpgy3zepal7ec5v6wv58da6c23hjuw4ypg96ztz75f8wrrussm59fetnkggq4j8pde6hkmw0",
      "mk1qpxj36ppag0zr8gh9upnjugr26jfvunvs35jvgdjkm3kghwnt0qqymzc0utyzxyhny9pu8c56a5k72ndqgmdftljqt"
    ],
    [
      "mk1qpl7wlpqqspsrg8ml5q6p7laqxs0hlfgv3gqvpgy3zepugvevsxpz2zll50ju3dcmghtxtfv0y025ltk2vc8a3ex8yqncct596tqv5z420v4",
      "mk1qpl7wlpprja893lkxup4z7tw6q2yvs4fk9pjhxf00s49ugex8rue307wdslgcj5r8x9t5j35p6p2c22v0s30tv0s2u"
    ]
  ],
  "md1": [
    "md1fu39yrq9qjtvyyy5jmppp9ykcggfgp9fskxcqkudsqefnfskhqqqq8uqnxnpwwqqqtggjse9txaz6v",
    "md1fu39yrqfqqqp0npeutks2dcdzxlrzsezsqc27rchwsv0jskp2rsal4egz4ep5859pnmq8wpsfncwhr",
    "md1fu39yrq3l4pkhsdyytkwl5z8lphut2hvvpp5wl4l0mn058ndxfl63kufyfsjwlt2v3d70kcz8a3r42",
    "md1fu39yrqa4j5lcxlmx9ayav9mj0jj6wv58da6c23hjuw4ypg96ztz75f8wrrussm598ryfkw5ey8h6p",
    "md1fu39yrpzw2ua7583pn5tj7qeewyp4dfykwfkgg6fxyxetdcmythf4hsqzd3v879jpmwaykdyahtr0v",
    "md1fu39yrpgcj7vs58sls39p0l68ewgkud5t4n95k8j84204m9xvr7cunrjqfurja8939xk8j47ndpq63",
    "md1fu39yrpha3hqdghjmksz3ry92d3gv4ejtmu9f0zxf3clxvtlnnv86xy4qee32ay5q0e3ty49zaan43"
  ],
  "multisig": {
    "template": "descriptor",
    "threshold": 3,
    "cosigner_count": 3,
    "path_family": "bip87",
    "cosigners": [
      {
        "index": 0,
        "master_fingerprint": "73c5da0a",
        "origin_path": "m/48'/0'/0'/2'",
        "xpub": "xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf"
      },
      {
        "index": 1,
        "master_fingerprint": "b8688df1",
        "origin_path": "m/48'/0'/0'/2'",
        "xpub": "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"
      },
      {
        "index": 2,
        "master_fingerprint": "28645006",
        "origin_path": "m/48'/0'/0'/2'",
        "xpub": "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6"
      }
    ]
  },
  "privacy_preserving": false
}
```

Things to notice in the envelope:

- **`origin_path`** = `m/48'/0'/0'/2'` — the default-inferred BIP-48
  cosigner path, applied to every `@N` placeholder that lacked an
  inline `[fp/path]@N` annotation or `--slot @N.path=` override.
- **`ms1`** is a 3-element array with **all three entries populated**
  — per SPEC §5.8 emission rule (added in toolkit-v0.21.0; uniform
  across all bundle modes), every phrase-bearing slot's entropy is
  encoded as that slot's ms1 card independently. The byte values for
  `ms1[0]`, `ms1[1]`, `ms1[2]` correspond 1:1 to the
  `--slot @0.phrase=` / `--slot @1.phrase=` / `--slot @2.phrase=`
  inputs above. In a real-world multi-cosigner deployment each
  cosigner generates their own ms1 card on their own machine from
  their own phrase (they never see the other cosigners' phrases);
  the consolidated 3-entry `ms1` array shown here is what happens
  when a single operator runs the bundle with all three phrases at
  once — the typical inheritance-rehearsal or backup-audit case.
  If a slot is supplied as watch-only (e.g., `--slot @i.xpub=...`),
  its `ms1[i]` entry is `""` per §5.8 (the "hybrid" example in the
  SPEC). Each cosigner physically holds (engraves, geographically
  separates) only THEIR own ms1 card alongside the shared `md1`
  wallet-policy card and all three `mk1` watch-only cards.

  Equivalent per-cosigner conversion (each cosigner runs this on
  their own machine; produces the same `ms1[i]` byte content):

  ```sh
  # Cosigner @1 generates their personal ms1 backup
  mnemonic convert \
    --from phrase='legal winner thank year wave sausage worth useful legal winner thank yellow' \
    --to ms1
  # → ms1: ms10entrsqplh7lml0alh7lml0alh7lml0als5cclar2zmksh6

  # Cosigner @2 generates theirs
  mnemonic convert \
    --from phrase='letter advice cage absurd amount doctor acoustic avoid letter advice cage above' \
    --to ms1
  # → ms1: ms10entrsqzqgpqyqszqgpqyqszqgpqyqszqqlfm7mep84hunu
  ```

- **`mk1`** is a `Vec<Vec<String>>` — outer per cosigner, inner per
  bech32-chunk. The two-chunk shape per cosigner is the canonical
  mk1 chunking for the wrapped key card. v0.20.0's F1 fix gave each
  cosigner's chunk-set its own `chunk_set_id` (xpub-fingerprint-
  derived) so the verify-bundle intake can correctly group chunks
  back per cosigner before mk-codec decode.
- **`md1`** is a 7-chunk wallet-policy descriptor card, shared across
  all three cosigners (the descriptor body is the same — only the
  cosigner xpubs and origins differ).
- **`multisig.cosigners[]`** carries the three master-fingerprint /
  origin-path / xpub triples the toolkit derived from the supplied
  BIP-39 phrases at the inferred path. These are the watch-only
  binding records used by external wallets (Sparrow, Specter, etc.)
  when importing the descriptor.

#### Verifying the inheritance bundle (v0.20.0+)

Round-trip the emitted JSON envelope through `verify-bundle` to
confirm every card decodes back to the seed at the inferred path.
The `--bundle-json` intake reads the same three-card vector the
preceding `bundle` invocation just wrote:

```sh
mnemonic verify-bundle --network mainnet --account 0 \
  --descriptor "$DESC" \
  --slot '@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' \
  --slot '@1.phrase=legal winner thank year wave sausage worth useful legal winner thank yellow' \
  --slot '@2.phrase=letter advice cage absurd amount doctor acoustic avoid letter advice cage above' \
  --bundle-json /tmp/inheritance-bundle.json
```

Expected output (one block per cosigner; final `result: ok`):

```text
ms1_decode[0]: ok cosigner[0] ms1 decoded
ms1_entropy_match[0]: ok cosigner[0] ms1 byte-identical
mk1_decode[0]: ok cosigner[0] mk1 decoded
mk1_xpub_match[0]: ok cosigner[0] xpub matches
mk1_fingerprint_match[0]: ok cosigner[0] fingerprint matches
mk1_path_match[0]: ok cosigner[0] path matches
ms1_decode[1]: ok cosigner[1] ms1 decoded
ms1_entropy_match[1]: ok cosigner[1] ms1 byte-identical
mk1_decode[1]: ok cosigner[1] mk1 decoded
mk1_xpub_match[1]: ok cosigner[1] xpub matches
mk1_fingerprint_match[1]: ok cosigner[1] fingerprint matches
mk1_path_match[1]: ok cosigner[1] path matches
ms1_decode[2]: ok cosigner[2] ms1 decoded
ms1_entropy_match[2]: ok cosigner[2] ms1 byte-identical
mk1_decode[2]: ok cosigner[2] mk1 decoded
mk1_xpub_match[2]: ok cosigner[2] xpub matches
mk1_fingerprint_match[2]: ok cosigner[2] fingerprint matches
mk1_path_match[2]: ok cosigner[2] path matches
md1_decode: ok decoded successfully
md1_wallet_policy: ok wallet-policy mode confirmed
md1_xpub_match: ok all 3 pubkeys match expected (multiset)
result: ok
```

Per SPEC §5.8 emission rule (v0.21.0+), descriptor mode populates
`ms1[i]` for every phrase-bearing slot, so the round-trip reports
all three slots as `ok` on both `ms1_decode` and `ms1_entropy_match`.
The `skipped: watch-only slot` report appears only when a slot was
bound via `--slot @i.xpub=` rather than `@i.phrase=` (the "hybrid"
case in SPEC §5.8). Pre-v0.21.0 bundles — where `ms1[1+]` was `""`
despite phrases being supplied for those slots — are rejected by
v0.21.0 verify-bundle with `ms1_decode[i]: fail` per SPEC §5.7
case 4; the v0.21.0 migration note in SPEC §5.8 explains.

`verify-bundle` re-applies the same canonicity-aware default-path
inference on the descriptor before binding the supplied cards, so the
round-trip works without re-typing the inferred path on the CLI.

Prior to v0.20.0, this multi-cosigner round-trip failed with
`ChunkedHeaderMalformed`; the bugfix shipped in `mnemonic-toolkit-v0.20.0`
(FOLLOWUP `verify-bundle-multi-cosigner-mk1-chunk-assembly`). v0.21.0
followed with the SPEC §5.8 per-slot ms1 emission fix that this
example illustrates (FOLLOWUP `synthesize-descriptor-deduplicate-with-unified`
tracks the next-step refactor opportunity).

#### Example: script-path-only P2TR wallet (NUMS sentinel)

```sh
mnemonic bundle --network mainnet \
  --descriptor 'tr(NUMS, and_v(v:pk(@0), after(12000000)))' \
  --language english \
  --slot '@0.phrase=…'
```

`NUMS` is a literal token the toolkit substitutes with the BIP-341
unspendable internal-key hex
`50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`
before rust-miniscript parses. The resulting wallet is P2TR (bech32m
addresses) with key-path spending intentionally disabled — only the
tap-script path is spendable. The leaf-key `@0` derives at the BIP-48
default per the inference rule above.

#### Refusal cases

| Trigger | Stderr |
|---|---|
| Bare `tr(<miniscript>)` (no internal key) | `error: tr() requires an internal key. For script-path-only spending use tr(NUMS, <ms>); for full taproot use tr(@<index>, <ms>) with a slot binding for the internal key.` |
| Canonical descriptor + `--account != 0` | `error: --account != 0 is meaningful only with --template; descriptor mode encodes account index in the @i origin path.` |
| `--slot @N.fingerprint=X` AND inline `[Y/...]@N` disagree | `error: slot @{N} fingerprint mismatch: --slot says X, descriptor inline [Y/...] disagrees; supply consistent values.` |
| Phrase-derived fingerprint disagrees with inline `[Y/...]@N` | `error: slot @{N} phrase-derived fingerprint X does not match descriptor inline [Y/...]; verify the phrase or correct the descriptor.` |
| `--slot @N.path=X` AND inline `[Y/Z]@N` paths differ | `error: slot @{N} path mismatch: --slot says X, descriptor inline [.../Z] disagrees; supply consistent values or remove one source.` |
| Canonical descriptor + `--slot @N.phrase= + --slot @N.path=` | `error: slot @{N} has both secret-bearing input and watch-only input; pick one per slot.` (the `{phrase, path}` pair is legal only in non-canonical mode) |

---

## `mnemonic verify-bundle`

Re-derive expected card content from a seed (or from a partial set
of cards) and report per-card pass/fail plus the overall verdict.

### Synopsis

```sh
mnemonic verify-bundle --network <NETWORK> [OPTIONS] [--ms1 ...] [--mk1 ...] [--md1 ...]
```

### Flags

| Flag | Purpose |
|---|---|
| `--network <NETWORK>` | mainnet / testnet / signet / regtest |
| `--template <TEMPLATE>` | as for `bundle` |
| `--descriptor <DESCRIPTOR>` | user-supplied BIP-388 descriptor |
| `--descriptor-file <DESCRIPTOR_FILE>` | descriptor read from file |
| `--threshold <THRESHOLD>` | multisig threshold |
| `--multisig-path-family <FAMILY>` | bip48 or bip87 |
| `--privacy-preserving` | match a privacy-preserving mk1 emission |
| `--language <LANGUAGE>` | BIP-39 wordlist |
| `--passphrase <PASSPHRASE>` | BIP-39 mnemonic passphrase |
| `--passphrase-stdin` | read `--passphrase` from stdin (raw, NULL-byte preserving); single stdin per invocation |
| `--account <ACCOUNT>` | BIP-32 account index |
| `--slot <SLOT>` | repeating slot input; for secret-bearing subkeys `=-` reads from stdin |
| `--bundle-json <PATH>` | read the bundle from a JSON file emitted by `bundle --json` |
| `--ms1 <STRING>` | repeating; one ms1 card |
| `--mk1 <STRING>` | repeating; one mk1 card |
| `--md1 <STRING>` | repeating; one md1 card |
| `--json` | JSON output |
| `--help` | print help |

### Worked example

See [Verifying a bundle](#verifying-a-bundle).

### Auto-fire on decode failure (v0.22.1)

When `verify-bundle` encounters a `ms_codec` / `mk_codec` / `md_codec`
decode failure on the SUPPLIED side of the bundle (corrupted engraving
re-typed into `--ms1` / `--mk1` / `--md1` or supplied via
`--bundle-json`), the BCH error-correction primitive from `mnemonic
repair` auto-fires — but only when stdout is attached to a TTY (the
v0.22.1 D18 default). The behavior matrix:

| TTY? | `--no-auto-repair`? | Outcome |
|---|---|---|
| yes | no | Auto-fire (exit 5 + repair report on stderr; corrected chunk on stdout) |
| yes | yes | Legacy VerifyCheck row + `result: mismatch` + exit 4 |
| no (pipe / redirected / CI) | no | Legacy VerifyCheck row + exit 4 (preserves automation contract) |
| no | yes | Legacy VerifyCheck row + exit 4 |

The TTY gate exists so scripts that parse the `VerifyCheck` array (or
the JSON envelope's `checks` field) don't see a single corrupted chunk
silently short-circuit the entire check matrix. Interactive users see
the helpful auto-fire UX; piped consumers see the v0.22.0-and-earlier
behavior unchanged.

Under `--json` calling context (any of `convert --json`, `inspect
--json`, `verify-bundle --json`), the auto-fire emits a structured JSON
envelope per v0.22.1 D20 — see `mnemonic repair` below for the schema.

#### Environment variable `MNEMONIC_FORCE_TTY` (v0.24.0+)

The TTY-detection step above can be overridden by the environment
variable `MNEMONIC_FORCE_TTY`. This is a **first-class public-API
contract** with semver-stable semantics (promoted from test-only at
v0.24.0):

| Value | Effect |
|---|---|
| `1` | force the TTY-positive auto-fire path |
| `0` | force the TTY-negative legacy path |
| unset / any other | fall back to runtime `is_terminal()` detection |

Known consumers (the public-API contract guarantees these continue to
work through future toolkit refactors):

- **`mnemonic-gui` v0.9.0+** sets `MNEMONIC_FORCE_TTY=1` in the toolkit
  subprocess environment. The GUI pipes the toolkit's stdin/stdout
  (not a real TTY), so without the env override the GUI would never see
  auto-fire repair under `convert` / `inspect` / `verify-bundle`.
- The toolkit's own integration test suite sets it to `1` to force
  auto-fire under `cargo test` (cargo's test harness pipes stdout).

The env-var applies to `verify-bundle`'s TTY-conditional auto-fire
only. `convert` and `inspect` auto-fire unconditionally when
`--no-auto-repair=false` (no TTY gate); the env-var has no effect on
those surfaces. It is not part of the clap `--help` surface (env-vars
are not part of clap-derive) nor the `mnemonic gui-schema` JSON.

---

## `mnemonic convert`

Single-format conversion across the 13-node typed graph: `phrase`,
`entropy`, `xpub`, `xprv`, `wif`, `fingerprint`, `path`, `ms1`, `mk1`,
`bip38`, `minikey`, `electrum-phrase`, `address`.

### Synopsis

```sh
mnemonic convert --from <NODE>=<value> --to <NODE> [--to <NODE>]... [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--from <FROM>` | source node (`phrase=…`, `entropy=…`, `xpub=…`, `xprv=…`, `wif=…`, `ms1=…`, `mk1=…`, `bip38=…`, `minikey=…`, `electrum-phrase=…`); `=-` reads from stdin |
| `--to <TO>` | target node; repeating for multi-output |
| `--network <NETWORK>` | mainnet / testnet / signet / regtest |
| `--template <TEMPLATE>` | as for `bundle` |
| `--path <PATH>` | derivation path override |
| `--account <ACCOUNT>` | account index (default 0) |
| `--language <LANGUAGE>` | BIP-39 wordlist |
| `--passphrase <PASSPHRASE>` | BIP-39 passphrase |
| `--passphrase-stdin` | read `--passphrase` from stdin (raw, NULL-byte preserving); BIP-38 V3 use case |
| `--bip38-passphrase <BIP38_PASSPHRASE>` | distinct BIP-38 Scrypt passphrase channel (v0.8 BREAKING — separate from `--passphrase`) |
| `--bip38-passphrase-stdin` | read `--bip38-passphrase` from stdin (raw, NULL-byte preserving); closes the BIP-38 V3 spec NULL-byte passphrase argv gap |
| `--electrum-version <ELECTRUM_VERSION>` | Electrum seed-version selector for `(Entropy, ElectrumPhrase)` |
| `--electrum-language <ELECTRUM_LANGUAGE>` | Electrum-specific wordlist (English + 4 non-English) |
| `--fingerprint <FINGERPRINT>` | master fingerprint (input on certain edges) |
| `--xpub-prefix <XPUB_PREFIX>` | SLIP-0132 prefix selector for emitted xpubs (e.g. zpub, ypub) |
| `--script-type <SCRIPT_TYPE>` | `p2wpkh` / `p2sh-p2wpkh` / `p2tr` for `(Xpub, Address)` derivation |
| `--json` | JSON output |
| `--help` | print help |

### Worked example

See [Minimal recovery walkthrough](#minimal-recovery-walkthrough)
and [Migrating from BIP-39 to the m-format](#migrating-from-bip-39-to-the-m-format).

---

## `mnemonic export-wallet`

Emit watch-only wallet artifacts for Bitcoin Core, BIP-388, Coldcard,
Blockstream Jade, Sparrow, or Specter.

### Synopsis

```sh
mnemonic export-wallet [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--template <TEMPLATE>` | as for `bundle` |
| `--descriptor <DESCRIPTOR>` | user-supplied BIP-388 descriptor |
| `--threshold <THRESHOLD>` | multisig threshold |
| `--multisig-path-family <FAMILY>` | bip48 or bip87 |
| `--network <NETWORK>` | default mainnet |
| `--language <LANGUAGE>` | ignored (watch-only); accepted for slot-parser symmetry |
| `--account <ACCOUNT>` | account index (default 0) |
| `--slot <SLOT>` | repeating `@N.<subkey>=<value>`; subkeys: `phrase`, `entropy`, `xpub`, `master_xpub`, `fingerprint`, `path`, `wif`, `xprv` (secret-bearing subkeys refused by `export-wallet`'s watch-only validator) |
| `--format <FORMAT>` | `bitcoin-core` (default) / `bip388` / `coldcard` / `jade` / `sparrow` / `specter` / `electrum` / `green` |
| `--output <OUTPUT>` | output path (`-` = stdout, default) |
| `--range <RANGE>` | Bitcoin Core `range` field; comma-separated; default `0,999` |
| `--timestamp <TIMESTAMP>` | Bitcoin Core `timestamp` field; `now` (default) or unix seconds |
| `--bitcoin-core-version <BITCOIN_CORE_VERSION>` | 24 or 25 (default 25) |
| `--wallet-name <WALLET_NAME>` | wallet name/label for formats that publish one (Coldcard generic JSON, Sparrow, Specter, Electrum); default `<template-human-name>-<account>` |
| `--taproot-internal-key <TAPROOT_INTERNAL_KEY>` | `nums` or `@N` for `tr-multi-a` / `tr-sortedmulti-a` |
| `--help` | print help |

### Notes

- **`--wallet-name` length cap.** The Coldcard multisig text (`--format coldcard` with a `wsh-*` / `sh-wsh-*` template) and the byte-identical Jade multisig text (`--format jade`) cap the `Name:` line at 20 Unicode scalar values per the Coldcard reference format. Longer names are truncated to the first 20 characters (not bytes — non-ASCII names are handled at codepoint granularity, so `🤐🤐🤐…` truncates cleanly without splitting a multi-byte sequence).
- **`@N.master_xpub=` parse vs emit.** The `master_xpub` slot subkey parses successfully under any `--format`, but `--format coldcard` with a singlesig template (`bip44` / `bip49` / `bip84`) currently refuses when the subkey is supplied because the resolution pipeline does not yet plumb the master xpub through to the Coldcard generic-JSON top-level `xpub` field (tracked by `design/FOLLOWUPS.md` entry `coldcard-master-xpub-plumbing-pending`, scheduled for v0.8.2). Re-invoke without the `master_xpub` slot to emit the JSON with the top-level `xpub` field omitted (which is what Coldcard accepts in the absence of a depth-0 xpub). Other formats silently ignore the subkey per the per-format ignored-input contract.
- **`--threshold` is REQUIRED for `--format sparrow` multisig.** Bitcoin Core / BIP-388 / Coldcard / Jade auto-default `K = N` (cosigner count) when `--threshold` is omitted, but Sparrow refuses with a missing-info error: Sparrow publishes the threshold in `defaultPolicy.miniscript.script` as `multi(K, ...)` / `sortedmulti(K, ...)`, and silently defaulting `K = N` would emit a wallet that looks like K=N was intentional rather than a missing-input default. Supply `--threshold <K>` explicitly when `--format sparrow` and the template is multisig.
- **`--wallet-name` is REQUIRED for `--format specter`.** Specter Desktop's UX requires an explicit wallet label; emitting a Specter wallet without one produces a wallet that displays as an empty string in the Specter UI (a UX regression vs. the user's likely intent). Other formats fall back to `<template-human-name>-<account>` when `--wallet-name` is omitted; Specter refuses via the SPEC §4 missing-info channel.

### Worked example

See [Exporting to Bitcoin Core / BIP-388 / Sparrow / Specter](#exporting-to-bitcoin-core-bip-388-sparrow-specter).

---

## `mnemonic derive-child`

BIP-85 deterministic child entropy. Six in-scope applications:
`bip39`, `hd-seed`, `xprv`, `hex`, `password-base64`, `password-base85`,
plus `dice` (BIP-85 v1.3.0).

### Synopsis

```sh
mnemonic derive-child --from <FROM> --application <APP> --length <LEN> --index <INDEX> [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--from <FROM>` | `xprv=<value>` or `phrase=<bip39>` (with `--passphrase` + `--language`); `=-` to read from stdin |
| `--application <APPLICATION>` | `bip39` / `hd-seed` / `xprv` / `hex` / `password-base64` / `password-base85` / `dice` |
| `--length <LENGTH>` | application-specific size; pass `0` for `hd-seed` and `xprv` |
| `--index <INDEX>` | hardened child index (`0..2^31`) |
| `--network <NETWORK>` | for `hd-seed` / `xprv` apps; defaults to mainnet |
| `--language <LANGUAGE>` | BIP-39 wordlist + BIP-85 language code for `--application bip39` |
| `--passphrase <PASSPHRASE>` | BIP-39 passphrase, only for `--from phrase=…` |
| `--passphrase-stdin` | read `--passphrase` from stdin (raw, NULL-byte preserving); single stdin per invocation |
| `--dice-sides <DICE_SIDES>` | required for `--application dice`; range `2..=2^32-1` |
| `--help` | print help |

### Worked example

See [Deterministic child secrets via BIP-85](#deterministic-child-secrets-via-bip-85).

---

## `mnemonic final-word`

Given an N-1 word BIP-39 partial phrase, emit the lexicographically
sorted set of wordlist entries that, when appended as the Nth word,
yield a phrase with a valid BIP-39 checksum. Output set size is a
function of N alone: 128 for N=12, 64 for N=15, 32 for N=18, 16 for
N=21, 8 for N=24.

Use cases include paper-backup recovery (a smudged last word), manual
seed generation (compute the only-valid checksum-fixing word for a
hand-rolled partial), and phrase-typo verification (look up whether
your last word appears in the candidate set for the first N-1 you've
written down).

### Synopsis

```sh
mnemonic final-word --from <phrase=<value-or-->> [--language <LANGUAGE>] [--json-out <PATH>]
```

### Flags

| Flag | Purpose |
|---|---|
| `--from <phrase=<value-or-->>` | partial phrase as `phrase=<N-1 words>` (inline) or `phrase=-` to read from stdin; inline form emits a `/proc/$PID/cmdline` argv-leakage advisory on stderr |
| `--language <LANGUAGE>` | BIP-39 wordlist; one of `english` / `simplifiedchinese` / `traditionalchinese` / `czech` / `french` / `italian` / `japanese` / `korean` / `portuguese` / `spanish` (default `english`) |
| `--json-out <PATH>` | side-effect: write a versioned JSON envelope to `<PATH>` in addition to the plain candidate list on stdout; on Unix a world-readable result raises a permission-mode advisory |
| `--help` | print help |

### Worked example

```sh
echo "abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon" |
  mnemonic final-word --from phrase=- --language english
```

Stdout: 8 sorted candidate words, one per line — including `art` (the
canonical zero-entropy 24-word vector). For N=12 partial input
(`abandon × 11`), the output is 128 lines including `about` (the
canonical Trezor zero-entropy 12-word vector).

### JSON output

```json
{
  "schema_version": "1",
  "language": "english",
  "partial_word_count": 11,
  "target_word_count": 12,
  "candidate_count": 128,
  "candidates": ["abandon", "ability", "above", "..."]
}
```

Field order is part of the schema (SHA-pinned in
`tests/cli_final_word_json.rs`). `candidates` is lexicographically
sorted; `candidate_count == candidates.len()`. The plain stdout output
is emitted in parallel (the JSON file is a side-effect, not a
stdout-replacement).

### Refusals

| Trigger | Refusal |
|---|---|
| Partial word count not in `{11, 14, 17, 20, 23}` | `final-word: got K words; expected one of [11, 14, 17, 20, 23] ...` |
| Empty partial (0 words after `split_whitespace`) | `final-word: empty partial phrase; need 11/14/17/20/23 words ...` |
| Unknown word at position I | `final-word: unknown BIP-39 word at position I (not in selected wordlist; did you pick the right --language?)` |
| `--from` variant other than `phrase=` | `final-word --from only accepts phrase=<value> or phrase=-` |

### Advisories

| Trigger | Stderr advisory |
|---|---|
| Inline `--from phrase=<value>` | `warning: secret material on argv (--from phrase=) — pipe via --from phrase=- to avoid /proc/$PID/cmdline exposure` |
| Stdout is a TTY AND candidate set non-empty | `warning: candidate list is secret material — pairing the partial phrase with any candidate yields a valid seed phrase; do not paste this output into untrusted tools` |
| `--json-out PATH` with world-readable file (Unix umask 022 default) | `warning: --json-out <PATH> inherits umask (file may be world-readable, mode 644); consider --json-out /dev/stdout or chmod 0600 the path before invoking` |

---

## `mnemonic seed-xor`

Coldcard-compatible BIP-39 ↔ BIP-39 all-or-nothing XOR-based seed splitter.
Two sub-subcommands: `split` (master phrase → N BIP-39 shares) and `combine`
(N shares → master phrase). NOT a threshold scheme — ALL N shares are
required to reconstruct (for K-of-N use SLIP-39, planned for v0.13.0).

**Coldcard interop:** native at 12/18/24-word sizes (per Coldcard
`shared/xor_seed.py` accepting entropy lengths 16/24/32 bytes). 15/21-word
sizes are toolkit-only extensions; Coldcard hardware cannot round-trip
those two sizes.

**Security caveat:** Seed XOR has no authentication tag. Substitution of
a wrong-but-valid-BIP-39 share is mathematically undetectable — the
recovered phrase will validate but derive the wrong wallet. Verify the
recovered wallet's expected derived address before trusting.

### Synopsis

```sh
mnemonic seed-xor split   --from <phrase=<value-or-->> --shares <N> [OPTIONS]
mnemonic seed-xor combine --share <phrase=<value-or-->> ... --shares <N> [OPTIONS]
```

### `seed-xor split` flags

| Flag | Purpose |
|---|---|
| `--from <phrase=<value-or-->>` | master phrase as `phrase=<value>` (inline) or `phrase=-` to read from stdin; inline form emits a `/proc/$PID/cmdline` argv-leakage advisory on stderr |
| `--shares <N>` | number of shares to emit; must be >= 2 |
| `--language <LANGUAGE>` | BIP-39 wordlist: `english` (default) / `simplifiedchinese` / `traditionalchinese` / `czech` / `french` / `italian` / `japanese` / `korean` / `portuguese` / `spanish` |
| `--deterministic-from-master` | use Coldcard's SHA256d-deterministic share generation instead of OS CSPRNG; required for byte-equal Coldcard hardware interop |
| `--json-out <PATH>` | side-effect: write versioned JSON envelope to PATH (does NOT replace stdout) |
| `--help` | print help |

### `seed-xor combine` flags

| Flag | Purpose |
|---|---|
| `--share <phrase=<value-or-->>` | share phrase; repeating; at most ONE may be `phrase=-` (single stdin per invocation) |
| `--shares <N>` | asserted share count; MUST equal the number of `--share` flags (hard refusal on mismatch — catches cardinality omissions, NOT substitution) |
| `--language <LANGUAGE>` | BIP-39 wordlist of inputs + output (default `english`) |
| `--json-out <PATH>` | side-effect: write versioned JSON envelope |
| `--help` | print help |

### Worked example

```sh
# Split a 24-word seed into 3 shares (deterministic, Coldcard-interop)
echo "abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon art" |
  mnemonic seed-xor split --from phrase=- --shares 3 --deterministic-from-master
```

Stdout: 3 lines, each a 24-word BIP-39 phrase. Reverse via:

```sh
mnemonic seed-xor combine \
  --share "phrase=<share-1>" \
  --share "phrase=<share-2>" \
  --share "phrase=<share-3>" \
  --shares 3
```

Stdout: the original 24-word phrase recovered.

### JSON output

`--json-out <PATH>` writes a versioned envelope. Schema `v1`. `split`
shape:

```json
{
  "schema_version": "1",
  "operation": "split",
  "language": "english",
  "word_count": 12,
  "share_count": 3,
  "deterministic": false,
  "shares": ["phrase-1 ...", "phrase-2 ...", "phrase-3 ..."]
}
```

`combine` shape:

```json
{
  "schema_version": "1",
  "operation": "combine",
  "language": "english",
  "word_count": 12,
  "share_count": 3,
  "phrase": "reconstructed phrase ..."
}
```

Field order is part of the schema (SHA-pinned in
`tests/cli_seed_xor_json.rs`).

### Refusals

| Trigger | Refusal |
|---|---|
| `split --from` phrase word-count not in {12,15,18,21,24} | `seed-xor split: phrase must be 12/15/18/21/24 words; got K` |
| `split --shares` < 2 | `seed-xor split: --shares must be >= 2; got N` |
| `combine --share` count mismatch vs `--shares` | `seed-xor combine: --shares N requires exactly N --share arguments; got K --share values for --shares N` |
| `combine` mixed-length shares | `seed-xor combine: all shares must be the same word count; got mix of {K1, K2, ...}` |
| `combine` share at position I has BIP-39 checksum failure | `seed-xor combine: share at position I has invalid BIP-39 checksum (...)` |
| `combine` unknown word in share at position I | `seed-xor combine: share at position I: unknown BIP-39 word at index J ...` |
| `--from` or `--share` variant other than `phrase=` | `seed-xor only accepts phrase=<value> or phrase=-` |
| Two or more `--share phrase=-` (multi-stdin) | `seed-xor combine: at most one --share value may be \`-\` (single stdin per invocation)` |

### Advisories

| Trigger | Stderr advisory |
|---|---|
| Inline `--from phrase=<v>` OR inline `--share phrase=<v>` | `warning: secret material on argv (--from phrase= OR --share phrase=) — pipe via phrase=- to avoid /proc/$PID/cmdline exposure` (per-occurrence) |
| `split` AND stdout is a TTY | `warning: Seed XOR shares on stdout — each of the N=<n> lines is independently a complete BIP-39 phrase; ALL N shares are required to reconstruct the master; distribute them to N separate locations; do not paste this output into a single untrusted tool. Substitution of a wrong-but-valid-BIP-39 share is undetectable by Seed XOR — verify the recovered wallet's derived address before trusting it.` |
| `combine` AND stdout is a TTY | `warning: combined phrase is secret material — Seed XOR has no authentication tag; verify the recovered wallet's expected derived address before trusting; if a share was substituted with a wrong-but-valid one, the result will validate but derive the wrong wallet` |
| `split --deterministic-from-master` with 15/21-word input | `warning: --deterministic-from-master with 15-word input is toolkit-only — Coldcard's xor_seed.py natively supports 12/18/24 only; resulting shares will NOT round-trip a Coldcard device. For Coldcard interop, use 12/18/24-word input.` |
| `--json-out <PATH>` with world-readable file (Unix) | `warning: --json-out <PATH> inherits umask (file may be world-readable, mode 644); consider --json-out /dev/stdout or chmod 0600 the path before invoking` |

---

## `mnemonic slip39`

SLIP-39\index{SLIP-39} (Trezor's `SLIP-0039`) is the K-of-N threshold
share-splitting standard for cryptocurrency seeds. Two sub-subcommands:
`split` (master secret → groups × members of SLIP-39 mnemonic shares)
and `combine` (≥K shares → master secret). Unlike the all-N XOR
scheme in [`seed-xor`](#mnemonic-seed-xor), this IS a true threshold
scheme — any K-of-N subset of shares reconstructs.

Shares are SLIP-39 mnemonics (NOT BIP-39 — different 1024-word
wordlist, longer length, RS1024 checksum). Toolkit shares are
bit-identical to Trezor SLIP-0039 reference shares; cross-impl
verification recipe in [Trezor interop](#trezor-interop) below.

### Concept signposts

- **Master secret** — the BIP-39 phrase or raw entropy that `split`
  consumes / `combine` recovers. Sizes: 16/20/24/28/32 bytes
  (12/15/18/21/24 BIP-39 words).
- **Share**\index{SLIP-39 share} — a single SLIP-39 mnemonic produced
  by `split`. Each share is independently secret material; substitution
  with a wrong-but-valid share is undetectable until the digest check
  at `combine` (refusal row 11 in the table below).
- **Group / member** — a group is a partition of shares; a member is
  one share within a group.
- **Group threshold (`G`)**\index{group threshold} — how many groups
  must contribute ≥ their member threshold of shares to reconstruct.
- **Member threshold (`T`)**\index{member threshold} — per-group: how
  many of that group's `N` shares must combine to reconstruct that
  group's secret.
- **Identifier** — random 15-bit per-secret tag shared across all
  shares of one split; mismatch on `combine` → refusal row 7.
- **Iteration exponent (`E`)** — PBKDF2 cost; iterations = 10000 ×
  2^E. Trezor default E=1 (20000 iters); E ≥ 5 emits a perf advisory.
- **Passphrase** — SLIP-39 passphrase (NOT the BIP-39 passphrase);
  empty string is the SLIP-39 default.
- **Extendable bit** — 1-bit flag controlling whether the identifier
  participates in the PBKDF2 salt. Toolkit emits the extendable form;
  `combine` accepts both (refusal row 22 catches mixed shares).

### Synopsis

```sh
mnemonic slip39 split   --from <phrase=…|entropy=…> --group-threshold G --group N,T [--group N,T]... [OPTIONS]
mnemonic slip39 combine --share <slip39-mnemonic-or-> ... [OPTIONS]
```

### `slip39 split` flags

| Flag | Purpose |
|---|---|
| `--from <phrase=…\|entropy=…>` | master secret as `phrase=<value-or->` or `entropy=<hex-or->`; `=-` reads from stdin |
| `--passphrase <P>` | SLIP-39 passphrase (NOT the BIP-39 mnemonic-extension passphrase) |
| `--passphrase-stdin` | read `--passphrase` from stdin (single stdin per invocation) |
| `--group-threshold <G>` | groups required to reconstruct (1 ≤ G ≤ group count) |
| `--group <N,T>` | repeating group spec (`<member_count>,<member_threshold>`); position in argv = SLIP-39 `group_idx` |
| `--iteration-exponent <E>` | PBKDF2 cost; iterations = 10000 · 2^E (range 0..=15, default 0); E ≥ 5 emits a perf advisory |
| `--language <LANGUAGE>` | BIP-39 wordlist of input phrase; ignored for `entropy=` inputs |
| `--json-out <PATH>` | side-effect: write versioned JSON envelope to `<PATH>` (in addition to plain-stdout shares) |
| `--help` | print help |

### `slip39 combine` flags

| Flag | Purpose |
|---|---|
| `--share <slip39-mnemonic-or->` | repeating share input; at most ONE may be `-` (stdin) |
| `--passphrase <P>` | SLIP-39 passphrase used at split time |
| `--passphrase-stdin` | read `--passphrase` from stdin (incompatible with any `--share -`) |
| `--to <entropy\|phrase>` | output shape (default `entropy`); `phrase` emits a BIP-39 mnemonic per `--language` |
| `--language <LANGUAGE>` | BIP-39 wordlist for `--to phrase`; ignored for `--to entropy` |
| `--json-out <PATH>` | side-effect: write versioned JSON envelope to `<PATH>` (in addition to plain-stdout secret) |
| `--help` | print help |

### Worked examples

The four examples below build progressively from the simplest case to
a realistic multi-group setup. All use the canonical zero-entropy
24-word master `abandon × 23 + art` (matching the
[`seed-xor` chapter's](#mnemonic-seed-xor) precedent for reader
recognition); share text is shown as `<share-N>` placeholders because
`split` is CSPRNG-driven (run the commands locally to see actual
share text).

#### Example 1 — smallest legal 2-of-2 single group, no passphrase

Smallest legal split (the toolkit refuses `--group 1,1` per refusals
row 5 AND `--group N,1` with N>1 per row 25 — the python `split_ems`
algorithm replicates the group share to all N members so any T=1
spec is degenerate; `--group 2,2` is the smallest non-degenerate
form). Two shares, BOTH required to recover.

```sh
echo "abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon art" |
  mnemonic slip39 split --from phrase=- --group-threshold 1 --group 2,2
```

Stdout: 2 shares, each a 33-word SLIP-39 mnemonic (33 words for the
32-byte master entropy at default `iter_exp=0`). Reverse with both:

```sh
mnemonic slip39 combine --share "<share-1>" --share "<share-2>" \
  --to phrase --language english
```

Stdout: the original `abandon × 23 + art` 24-word phrase. (Without
`--to phrase`, `combine` defaults to `--to entropy` and emits 64 hex
chars — `0000000000000000000000000000000000000000000000000000000000000000`
for the canonical zero-vector master.)

> Alternative master input via raw hex entropy:
>
> ```sh
> mnemonic slip39 split --from entropy=0102030405060708090a0b0c0d0e0f10 \
>   --group-threshold 1 --group 2,2
> ```
>
> Produces 2 shares of 20 words each (16-byte entropy maps to 20-word
> shares). The JSON envelope's `identifier` + `iteration_exponent`
> shape is the same regardless of `phrase=` vs `entropy=` input.

#### Example 2 — 2-of-2 single group, with passphrase

Adds a SLIP-39 passphrase. Same threshold shape as example 1; only
the passphrase differs.

```sh
echo "abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon art" |
  mnemonic slip39 split --from phrase=- --group-threshold 1 --group 2,2 \
    --passphrase TREZOR
```

Stdout: 2 shares. Reverse with both + the matching passphrase:

```sh
mnemonic slip39 combine --share "<share-1>" --share "<share-2>" \
  --passphrase TREZOR --to phrase --language english
```

Stdout: the original 24-word phrase.

> **Passphrase has no authentication tag.** `combine` with the WRONG
> passphrase silently recovers a DIFFERENT entropy — the digest check
> (refusal row 11) only fires when the recovered secret fails its
> internal HMAC, which the wrong-passphrase result will pass for any
> non-empty input. Same security model as the BIP-39 passphrase. Always
> verify the recovered wallet's expected derived address before
> trusting.
>
> **Argv-leakage advisory:** `--passphrase TREZOR` is on argv and
> visible in `/proc/$PID/cmdline`; the toolkit emits
> `warning: secret material on argv (--passphrase) — pipe via
> --passphrase-stdin to avoid /proc/$PID/cmdline exposure` on stderr.
> For sensitive use, pipe via `--passphrase-stdin`.

#### Example 3 — standard 2-of-3 single group, no passphrase

Introduces the K-of-N\index{K-of-N} threshold (the headline SLIP-39
feature). 1 group with 3 members at threshold 2: any 2 shares
reconstruct; losing 1 share is recoverable; losing 2 of 3 is total
loss.

```sh
echo "abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon art" |
  mnemonic slip39 split --from phrase=- --group-threshold 1 --group 3,2
```

Stdout: 3 shares `<share-1>`, `<share-2>`, `<share-3>`. Reverse with
any 2:

```sh
mnemonic slip39 combine --share "<share-1>" --share "<share-2>" \
  --to phrase --language english
```

Equivalent recoveries with `--share "<share-1>" --share "<share-3>"`
or `--share "<share-2>" --share "<share-3>"`. (Without `--to phrase`,
`combine` defaults to `--to entropy` and emits 64 hex chars.)

> Attempting recovery with only 1 share: `mnemonic slip39 combine
> --share "<share-1>"` exits 1 with stderr `slip39 combine: insufficient
> shares for group 0: need 2, got 1` (refusal row 12).

#### Example 4 — multi-group 2-of-3 of 2-of-3, with passphrase

The comprehensive case: 3 groups, each with 3 members at 2-of-3 member
threshold; 2 of 3 groups required (group threshold). 9 shares total.

This shape is "social-recovery"-style: 3 trustees each hold 3 shares;
any 2 trustees with ≥2 of their 3 shares can cooperate. A trustee
losing 1 share is not catastrophic; an entire trustee being unavailable
is also recoverable as long as the other 2 trustees can each contribute
their 2-of-3.

```sh
echo "abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon art" |
  mnemonic slip39 split --from phrase=- \
    --group-threshold 2 \
    --group 3,2 --group 3,2 --group 3,2 \
    --passphrase TREZOR
```

Stdout: 9 shares in group-major order, with a blank-line separator
between groups (the Trezor interop recipe below relies on this layout
when slicing shares with `sed -n`):

```text
<g0-m0>
<g0-m1>
<g0-m2>

<g1-m0>
<g1-m1>
<g1-m2>

<g2-m0>
<g2-m1>
<g2-m2>
```

Reverse with 2 shares from group 0 + 2 shares from group 1 (group 2
unused — the group threshold of 2 is satisfied by groups 0 + 1):

```sh
mnemonic slip39 combine \
  --share "<g0-m0>" --share "<g0-m1>" \
  --share "<g1-m0>" --share "<g1-m1>" \
  --passphrase TREZOR --to phrase --language english
```

Stdout: the original 24-word phrase. Many valid 4-share subsets exist
(any 2 from 2 of the 3 groups). (Without `--to phrase`, `combine`
defaults to `--to entropy`.)

> **Note:** to exercise the iteration-exponent perf advisory below,
> append `--iteration-exponent 5` to the `split` invocation; stderr
> will print `warning: --iteration-exponent E=5 yields 320000 ×
> PBKDF2-HMAC-SHA-256 iterations; ...`. The exponent is encoded in
> each share's `id_exp` field, so the matching `combine` invocation
> needs no extra flag — it reads the exponent from the shares
> automatically.

This example's combine recipe is also the input to the
[Trezor interop](#trezor-interop) cross-impl recipe below.

### JSON output

`--json-out <PATH>` writes a versioned JSON envelope (in addition to
the plain-stdout shares/secret). Schema `v1`. Field order is part of
the schema (SHA-pinned in `tests/cli_slip39_json.rs`).

`split` envelope (using example 4's shape):

```json
{
  "schema_version": "1",
  "operation": "split",
  "identifier": <u64>,
  "iteration_exponent": 0,
  "group_threshold": 2,
  "groups": [
    {"member_count": 3, "member_threshold": 2, "shares": ["<g0-m0>", "<g0-m1>", "<g0-m2>"]},
    {"member_count": 3, "member_threshold": 2, "shares": ["<g1-m0>", "<g1-m1>", "<g1-m2>"]},
    {"member_count": 3, "member_threshold": 2, "shares": ["<g2-m0>", "<g2-m1>", "<g2-m2>"]}
  ]
}
```

Each group entry is `{member_count, member_threshold, shares}` in that
exact order (mirrors the `seed_xor` envelope precedent). NO top-level
`language` field, NO `master_word_count` field — those are conveyed
via the `--language` and `--from` CLI flags out of band.

`combine` envelope (`--to entropy` shape, default):

```json
{
  "schema_version": "1",
  "operation": "combine",
  "identifier": <u64>,
  "iteration_exponent": 0,
  "output_shape": "entropy",
  "entropy_hex": "0000000000000000000000000000000000000000000000000000000000000000",
  "phrase": null
}
```

`combine` envelope (`--to phrase` shape):

```json
{
  "schema_version": "1",
  "operation": "combine",
  "identifier": <u64>,
  "iteration_exponent": 0,
  "output_shape": "phrase",
  "entropy_hex": null,
  "phrase": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
}
```

Both `entropy_hex` and `phrase` are always present; one carries the
value, the other is `null`, selected by `output_shape`. The
`--language` flag controls which BIP-39 wordlist `phrase` uses
(English / Czech / Korean / etc.) but is not itself reflected in the
envelope.

`--json-out` to a Unix world-readable path triggers the `mode 644`
permission advisory on stderr (advisories table below).

### Refusals

All refusals exit 1 with the stem on stderr. Mirror of SPEC §2.5
(25 classes; row 24 added at v0.13.0 P2.2 GREEN per Q3 fold; row 25
added at v0.13.0 P3 R1 fold for the toolkit-policy refusal of any
`--group N,T` with `T==1 AND N>1` — surfaced when chapter examples
1+2 attempted `--group 2,1` and the lib refused per the python
`split_ems` rule).

| Trigger | Refusal stem |
|---|---|
| `--from phrase` word-count not in {12,15,18,21,24} | `slip39 split: input phrase must be 12/15/18/21/24 words; got K` |
| `--from entropy=` hex not parseable / odd length / length not in {16,20,24,28,32} bytes | `slip39 split: entropy hex must decode to 16/20/24/28/32 bytes; got K bytes` |
| `--group-threshold` outside `1..=group_count` | `slip39 split: --group-threshold must be in 1..=K (number of --group flags); got G` |
| `--group N,T` with `T > N` OR `T < 1` OR `N > 16` | `slip39 split: --group N,T requires 1 <= T <= N <= 16; got group <idx>=N,T` |
| Any `--group 1,1` (toolkit usability policy) | `slip39 split: 1-of-1 group offers no recovery benefit; use --group N,T with N >= 2 (toolkit policy)` |
| `--iteration-exponent` outside 0..=15 | `slip39 split: --iteration-exponent must be 0..=15 (4-bit field); got E` |
| `combine` shares: identifier mismatch across shares | `slip39 combine: shares disagree on identifier; shares must come from the same secret` |
| `combine` shares: iteration-exponent mismatch | `slip39 combine: shares disagree on iteration-exponent` |
| `combine` shares: RS1024 checksum failure on share I | `slip39 combine: share at position I has invalid SLIP-39 checksum (RS1024)` |
| `combine` shares: unknown SLIP-39 word at position I in share J | `slip39 combine: share at position J: word at index I not in SLIP-39 wordlist` |
| `combine` shares: digest verification failure | `slip39 combine: reconstructed master digest mismatch — wrong --passphrase OR a share was substituted` |
| `combine` shares: insufficient share count for one or more required groups | `slip39 combine: insufficient shares for group <idx>: need <member_threshold>, got <K>` |
| `combine` shares: mismatching group thresholds across shares | `slip39 combine: shares disagree on group_threshold` |
| `combine` shares: mismatching group counts across shares | `slip39 combine: shares disagree on group_count` |
| `combine` shares: duplicate member index within a single group | `slip39 combine: duplicate member index <I> in group <G>` |
| Invalid padding bits in encoded share | `slip39 combine: share at position I has non-zero padding bits (encoding violation)` |
| `--from` variant other than `phrase=` / `entropy=` | `slip39 split: --from only accepts phrase=<value-or-> or entropy=<hex-or->; got <node>=` |
| Multi-stdin contention (e.g. `--passphrase-stdin` + `--share -`) | `slip39: at most one stdin consumer per invocation (across --share, --from, and --passphrase-stdin)` |
| `combine` called with empty share list | `slip39 combine: at least one share required` |
| `combine` shares: share at position I has value-byte length L not in {16,20,24,28,32} | `slip39 combine: share at position I has value length L (must be 16/20/24/28/32 bytes)` |
| `combine` shares: shares disagree on value-byte length | `slip39 combine: shares disagree on value length` |
| `combine` shares: shares disagree on the `extendable` (ext) bit | `slip39 combine: shares disagree on the extendable bit` |
| `combine` shares: parse-time refusal — share at position J encodes `group_count < group_threshold` | `slip39 combine: share at position J: group_threshold T exceeds group_count N` |
| `combine` shares: shares within a single group disagree on `member_threshold` | `slip39 combine: shares within a group disagree on member_threshold` |
| Any `--group N,T` with `T==1 AND N>1` (toolkit policy; python `split_ems` rule — algorithm replicates the group share to all N members so T=1+N>1 is degenerate; jointly with row 5 means smallest legal split is `--group 2,2`) | `slip39 split: --group N,T requires 1 <= T <= N <= 16; got group <idx>=N,T` |

### Advisories

Stderr advisories are non-fatal and do not change exit code (0 on
success). Mirror of SPEC §2.6 (6 rows).

| Trigger | Stderr advisory |
|---|---|
| Inline secret on argv (`--from`, `--share`, `--passphrase`) | per-occurrence `warning: secret material on argv (<flag>) — pipe via <alternative> to avoid /proc/$PID/cmdline exposure` |
| `split` AND stdout is a TTY | `warning: SLIP-39 shares on stdout — N=<n> shares emitted across <g> groups (group-threshold <G>); each share is independently secret material; distribute per your group/member-threshold policy; do not paste this output into a single untrusted tool` |
| `combine` AND stdout is a TTY | `warning: reconstructed secret material on stdout — verify the recovered wallet's expected derived address before trusting` |
| `--json-out` to a world-readable path (Unix) | `warning: --json-out <PATH> inherits umask (file may be world-readable, mode 644); consider --json-out /dev/stdout or chmod 0600 the path before invoking` |
| `--iteration-exponent E` where E ≥ 5 | `warning: --iteration-exponent E=<E> yields <iters> × PBKDF2-HMAC-SHA-256 iterations; split + combine performance may be observably slow (sub-second to multi-second); Trezor's reference uses E=1 (20000 iters) as default; the SLIP-0039 spec gives no recommended values; E ≥ 10 may exceed 30s on weak hardware` |
| Either `MNEMONIC_SLIP39_TEST_RNG` OR `MNEMONIC_SLIP39_TEST_IDENTIFIER` env-var set on a `split` invocation (always-on; not suppressible) | `warning: MNEMONIC_SLIP39_TEST_RNG set — output is deterministic and INSECURE; do not use for real shares` |

> **Note:** the warning string names `MNEMONIC_SLIP39_TEST_RNG` even
> when only the companion `MNEMONIC_SLIP39_TEST_IDENTIFIER` is set —
> both env-vars trigger the same single-string advisory; see SPEC §6
> for both env-var definitions.

### Trezor interop

Toolkit shares are bit-identical to Trezor SLIP-0039
interop\index{Trezor SLIP-0039 interop}. The recipe below proves this
via cross-implementation verification against `shamir-mnemonic`, the
Python reference implementation maintained by the Trezor team
(reproduces without hardware).

**Recipe** (validated 2026-05-14 against `shamir-mnemonic` 0.3.0 on
Linux x86_64; toolkit reference baseline is `python-shamir-mnemonic`
upstream commit `17fcce14`):

```sh
pipx install 'shamir-mnemonic[cli]==0.3.0'

# Produce shares with the toolkit (using example 4's shape: multi-group
# 2-of-3 of 2-of-3 with passphrase=TREZOR, master = abandon × 23 + art)
printf 'TREZOR' | mnemonic slip39 split \
  --from phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art" \
  --group-threshold 2 \
  --group 3,2 --group 3,2 --group 3,2 \
  --passphrase-stdin > /tmp/shares.txt

# Recover via shamir-mnemonic — pipe 4 shares (2 from group 0, 2 from
# group 1), then the passphrase twice (shamir prompts for confirmation).
# NOTE: multi-group split output is group-major with a BLANK LINE
# between groups; for 3 groups of 3 members each the file layout is
# lines 1-3 = group 0, line 4 = blank, lines 5-7 = group 1, line 8 =
# blank, lines 9-11 = group 2.
SHARE_G0_M0=$(sed -n 1p /tmp/shares.txt)
SHARE_G0_M1=$(sed -n 2p /tmp/shares.txt)
SHARE_G1_M0=$(sed -n 5p /tmp/shares.txt)
SHARE_G1_M1=$(sed -n 6p /tmp/shares.txt)
printf '%s\n%s\n%s\n%s\nTREZOR\nTREZOR\n' \
  "$SHARE_G0_M0" "$SHARE_G0_M1" "$SHARE_G1_M0" "$SHARE_G1_M1" |
  shamir recover -p
```

Expected output (last 2 lines):

```text
SUCCESS!
Your master secret is: 0000000000000000000000000000000000000000000000000000000000000000
```

That hex (32 zero bytes) is the BIP-39 entropy of `abandon × 23 + art`
— the same master `mnemonic slip39 combine` recovers from the same
shares + passphrase. Convert to phrase form via
`mnemonic convert --from entropy=00...00 --to phrase` if desired.

**Version-pin caveat:** the recipe pins `shamir-mnemonic==0.3.0` (the
latest released PyPI version at chapter-write 2026-05-14). The
toolkit's library bit-exact verification baseline is upstream commit
`17fcce14`; if the recipe fails for you, the released PyPI version
may have diverged. The version-pinned PyPI archive is at
<https://pypi.org/project/shamir-mnemonic/0.3.0/>; file a toolkit
issue with the failing share text + python error if encountered.

**Trezor hardware compatibility note:** SLIP-39 is supported on
Trezor Model T and the Trezor Safe family — NOT on Trezor One (which
predates SLIP-39 and uses raw BIP-39 only, per SPEC §3 OOS row
`OOS-slip39-import-trezor-onev-format`). SLIP-39 has two backup-type
modes: `slip39-basic` for single-group splits (examples 1-3 above)
and `slip39-advanced` for multi-group splits (example 4 above).
Consult Trezor's current docs for the exact `trezorctl
recovery-device --backup-type` flag value, which has historically
varied by firmware version.

---

## `mnemonic gui-schema`

Emit the SPEC §7 machine-readable schema of every existing
subcommand's flag surface as JSON to stdout. Companion to the
`mnemonic-gui` v0.2 schema-mirror contract — the GUI consumes this
output to render forms and refuses to launch on `version != 1`.

The schema is generated by walking the clap-derive `Command` tree
via `clap::CommandFactory`; the `gui-schema` subcommand itself is
filtered out (self-reference suppression).

### Synopsis

```sh
mnemonic gui-schema
mnemonic gui-schema --classify-descriptor <DESCRIPTOR>
```

### Flags

| Flag | Purpose |
|---|---|
| `--classify-descriptor <DESCRIPTOR>` | diagnostic: print `canonical` or `non-canonical` for `<DESCRIPTOR>` per md-codec's canonical-origin table; suppresses JSON schema |
| `--help` | print help |

### `--classify-descriptor`

When `--classify-descriptor <DESCRIPTOR>` is supplied, the JSON schema
is suppressed and a single line is printed to stdout:

- `canonical\n` (exit 0) — the descriptor maps to one of the canonical
  shapes in md-codec's `canonical_origin` table (`pkh / wpkh / tr (keypath-only) /
  wsh(multi|sortedmulti) / sh(wsh(multi|sortedmulti))`); its origin path is
  inferred from BIP-44/49/84/86 or BIP-48 conventions.
- `non-canonical\n` (exit 0) — the descriptor parses but does not map to a
  canonical shape. The `mnemonic bundle` default-path inference per
  SPEC §4.12.b applies (BIP-48 cosigner path `m/48'/<coin>'/<account>'/2'`).
- exit 2 with empty stdout — descriptor failed to parse
  (`DescriptorParse` error variant).

```sh
$ mnemonic gui-schema --classify-descriptor 'pkh(@0)'
canonical
$ echo $?
0
$ mnemonic gui-schema --classify-descriptor 'wsh(andor(pkh(@0),after(12000000),pk(@1)))'
non-canonical
$ echo $?
0
$ mnemonic gui-schema --classify-descriptor 'this is not a descriptor'
$ echo $?
2
```

This is the toolkit-side authority used by `mnemonic-gui` v0.8.1 (and
later) to detect non-canonical descriptors and surface the appropriate
default-path-inference banner + slot-editor placeholder. The drift gate
at `mnemonic-gui/tests/canonicity_drift.rs` pins agreement between the
GUI's regex classifier and this toolkit verdict on every canonicity-corpus
fixture.

### Output shape

```json
{
  "version": 1,
  "cli": "mnemonic",
  "subcommands": [
    {
      "name": "bundle",
      "flags":       [ {"name": "--network", "required": true, "kind": "dropdown", "choices": ["mainnet","testnet","signet","regtest"]} ],
      "positionals": []
    }
  ]
}
```

`kind` is one of `text` / `boolean` / `number` / `dropdown` / `path`.
`choices` is non-null only when `kind == "dropdown"`. Complex
GUI-side variants (NodeValueComposite, TaggedOrIndexed, Range,
Timestamp) intentionally collapse to `"text"` upstream and are
re-parsed client-side per the SPEC §7 lossy-mapping contract.

---

## `mnemonic repair`

BCH error-correct a corrupted m-format card (`ms1` / `mk1` / `md1`).
All three formats share the BIP-93 codex32 BCH code family — regular
`BCH(93,80,8)` for data-parts of 14–93 symbols (every `ms1`, every
`md1`, and short `mk1` chunks), long `BCH(108,93,8)` for data-parts of
96–108 symbols (the xpub-bearing first chunk of typical `mk1`
emissions). Both codes correct up to four substitution errors per
chunk (singleton bound `t=4`).

Use cases include recovery of a corroded engraving (one or two letters
unreadable), salvage of a hand-copied card with a single typo, or
sanity-checking a freshly engraved card against its source bundle
before committing to steel.

### Synopsis

```sh
mnemonic repair {--ms1 <MS1> | --mk1 <MK1> [--mk1 <MK1>...] | --md1 <MD1> [--md1 <MD1>...]} [--json]
```

### Flags

| Flag | Purpose |
|---|---|
| `--ms1 <MS1>` | single `ms1` chunk to repair; use `-` to read one chunk from stdin; mutually exclusive with `--mk1` / `--md1` |
| `--mk1 <MK1>` | one or more `mk1` chunks (repeating flag); use `-` to read chunks from stdin (one per line); mutually exclusive with `--ms1` / `--md1` |
| `--md1 <MD1>` | one or more `md1` chunks (repeating flag); use `-` to read chunks from stdin (one per line); mutually exclusive with `--ms1` / `--mk1` |
| `--json` | emit a single JSON envelope on stdout instead of the text-form repair report |
| `--help` | print help |

### Exit codes

| Code | Meaning |
|---|---|
| `0` | all chunks already valid (no repair applied; input echoed to stdout unchanged) |
| `5` | at least one chunk corrected (`REPAIR_APPLIED`); stdout = repair report + corrected chunks |
| `2` | unrepairable (per-chunk `RepairError`; e.g. `TooManyErrors`, `HrpMismatch`, `ReservedInvalidLength`, `UnsupportedCodeVariant`) |
| `1` | I/O error or other generic failure |

### Worked example

```sh
# A valid ms1 chunk with one character corrupted (position 17 'q' → 'z'):
mnemonic repair --ms1 ms10entrsqqqqqqqqqqqzqqqqqqqqqqqqqqqqcj9sxraq34v7f
```

Stdout (the corrected chunk is on the LAST line; comment lines describe the fix):

```text
# Repair report
#   ms1 chunk 0: 1 correction at position 17: 'z' -> 'q'
ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
```

Stderr:

```text
repair: applied 1 correction across 1 chunk
warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')
```

Exit code: `5`.

### JSON output

```json
{
  "schema_version": "1",
  "kind": "ms1",
  "corrected_chunks": ["ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"],
  "repairs": [
    {
      "chunk_index": 0,
      "original_chunk": "ms10entrsqqqqqqqqqqqzqqqqqqqqqqqqqqqqcj9sxraq34v7f",
      "corrected_chunk": "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
      "corrected_positions": [{"position": 17, "was": "z", "now": "q"}]
    }
  ]
}
```

### Per-chunk atomic semantics

For multi-chunk inputs (`--mk1 <c0> --mk1 <c1> --mk1 <c2>` or the `md1`
analog), if ANY chunk fails to repair (e.g. > 4 errors), the WHOLE
call fails with the offending `chunk_index` named. Partial repair of
sibling chunks is NOT returned — this avoids surfacing a half-fixed
card that could mislead the user into committing it. Re-run with
better data for the failing chunk.

### Refusals

| Trigger | Refusal |
|---|---|
| `chunk_index N` has more than 4 substitutions | `repair: chunk N has too many errors to correct uniquely (exceeds singleton bound = 8); cannot suggest correction` |
| `chunk_index N` HRP is not the expected one | `repair: chunk N HRP mismatch — expected 'XX', found 'YY' (HRP is not BCH-protected; re-type the prefix)` |
| `chunk_index N` data-part length is 94 or 95 | `repair: chunk N data-part length L is in BIP-93's reserved-invalid band [94, 95]; re-type the chunk` |
| `chunk_index N` data-part length triggers long code for an HRP whose codec doesn't define one (`ms` / `md`) | `repair: chunk N data-part length L would require the long BCH code, which is not defined for HRP 'X' in this codec version` |
| No chunks supplied | `repair: no chunks supplied` |
| Post-correction sibling-codec decode failed (`ms1` / `md1` only, v0.23.0+) | `repair: chunk N post-correction decode failed: <upstream codec Display>` (chunk index `N` is omitted when atomic-fail context lost the offending chunk's position). |

#### `PostCorrectionDecodeFailed` (v0.23.0)

At v0.23.0, the `ms1` and `md1` repair branches delegate to the
sibling codecs' native `decode_with_correction` APIs
(`ms_codec::decode_with_correction` from ms-codec v0.2.0 +
`md_codec::decode_with_correction` from md-codec v0.34.0) instead of
the v0.22.x toolkit-side BCH primitive (which vendored
`MS_NUMS_TARGET` + `MD_NUMS_TARGET` constants). Because the
sibling-codec wrappers run BCH correction AND the full §4-rule
wire-format decoder in one call, decoder errors that occur AFTER
BCH correction (e.g. ms-codec's `ThresholdNotZero` / `TagInvalidAlphabet`
/ `PayloadLengthMismatch` orphan §4-rule variants, or md-codec's
`BitStreamTruncated` / `WireVersionMismatch` wire-format variants)
surface through a new `RepairError::PostCorrectionDecodeFailed { chunk_index: Option<usize>, detail: String }` variant.

This is the catch-all for sibling-codec error variants that the
toolkit's per-variant translation table does not enumerate individually.
The `detail` field is the upstream codec's `Display`-rendered error,
verbatim. Mk1 repair is unaffected (mk-codec primitives are still
consumed natively per the unchanged Mk1 branch).

### Advisories

| Trigger | Stderr advisory |
|---|---|
| Corrected `ms1` emitted to stdout | `warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '\| age -e ...')` |
| Repair fired and emitted ≥ 1 correction | `repair: applied K correction(s) across J chunk(s)` |

### `--no-auto-repair` interaction

The standalone `mnemonic repair` subcommand IGNORES the global
`--no-auto-repair` flag (the whole point of this subcommand IS repair).
The flag applies only to the auto-fire short-circuit on the OTHER
subcommands (`convert`, `inspect`, `verify-bundle`).

### HRP "did you mean" (v0.22.1)

When the user supplies a chunk whose human-readable prefix is one
substitution away from a known HRP, the `HrpMismatch` error appends a
`; did you mean '<suggestion>'?` suffix:

```sh
mnemonic repair --ms1 ns10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
# stderr: error: repair: chunk 0 HRP mismatch — expected 'ms', found 'ns'
#   (HRP is not BCH-protected; re-type the prefix); did you mean 'ms'?
```

The suggestion is OMITTED when the input is ambiguous (e.g., `mb` is
1-sub from all three known HRPs) or has no Levenshtein-1 neighbor in
`{"ms", "mk", "md"}`. The HRP is not part of the BCH-protected payload,
so the suggestion is purely informational — the user must re-type the
prefix manually.

**Scope:** D19 is observable via the standalone `mnemonic repair`
error path only. Auto-fire (`convert` / `inspect` / `verify-bundle`)
falls through to the typed sibling-codec error on repair-failure (per
the v0.22.0 fall-through discipline), so the auto-fire path surfaces
the codec's own message — NOT this suggestion.

### JSON-context auto-fire envelope (v0.22.1 D20)

When auto-fire fires under any `--json` calling context (`convert
--json`, `inspect --json`, `verify-bundle --json`), the stdout is a
structured JSON envelope instead of the text-form repair report. Schema:

```json
{
  "schema_version": "1",
  "auto_repair_short_circuit": true,
  "exit_code": 5,
  "kind": "ms1",
  "corrected_chunks": ["ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"],
  "repairs": [
    {
      "chunk_index": 0,
      "original_chunk": "ms10entrsqqqqqqqqqqqzqqqqqqqqqqqqqqqqcj9sxraq34v7f",
      "corrected_chunk": "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
      "corrected_positions": [{"position": 17, "was": "z", "now": "q"}]
    }
  ]
}
```

The two top-level fields `auto_repair_short_circuit: true` and
`exit_code: 5` discriminate the envelope from the standalone
`mnemonic repair --json` envelope (which is structurally similar but
omits those fields). Stderr summary and D9 sensitive-secret warning
remain identical regardless of stdout format.

The standalone `mnemonic repair --json` invocation still emits the
v0.22.0 `RepairJson` envelope (without the D20 discriminator fields) —
the discriminator marks emission as "auto-fire short-circuit" vs
"user-invoked repair subcommand."

---

## `mnemonic inspect`

Describe the contents of an m-format card without performing any
conversion. Per kind:

- `ms1` — tag (`entr` for v0.1 ms-codec), payload kind, byte length,
  bit strength (= 8 × bytes). Entropy hex is suppressed by default
  (sensitive material); pass `--reveal-secret` to print it.
- `mk1` — policy-id-stub count, origin fingerprint (or `<absent>`
  for the privacy-preserving emission mode), origin path, xpub.
- `md1` — placeholder count (`n`), root-tree tag (`Wpkh` / `Tr` /
  `Wsh` / …), wallet-policy-mode flag, path-decl shape (`Shared` vs
  `Divergent`).

### Synopsis

```sh
mnemonic inspect {--ms1 <MS1> | --mk1 <MK1> [--mk1 <MK1>...] | --md1 <MD1> [--md1 <MD1>...]} [--json] [--reveal-secret]
```

### Flags

| Flag | Purpose |
|---|---|
| `--ms1 <MS1>` | single `ms1` chunk to inspect; use `-` to read one chunk from stdin; mutually exclusive with `--mk1` / `--md1` |
| `--mk1 <MK1>` | one or more `mk1` chunks (repeating flag); use `-` for stdin |
| `--md1 <MD1>` | one or more `md1` chunks (repeating flag); use `-` for stdin |
| `--json` | emit a single JSON envelope on stdout instead of the text-form report |
| `--reveal-secret` | reveal `ms1` entropy hex on stdout (no effect for `mk1` / `md1`, which carry no secret material) |
| `--help` | print help |

### Worked example

```sh
mnemonic inspect --ms1 ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
```

Stdout:

```text
kind: ms1
tag: entr
payload_kind: Entr
byte_length: 16
bit_strength: 128
entropy_hex: <suppressed; pass --reveal-secret to print>
```

Stderr:

```text
warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')
```

### Auto-fire short-circuit

When a corrupted card is supplied to `inspect`, the sibling-codec
decode fails and v0.22.0 auto-fire kicks in: instead of surfacing the
typed decode error, the toolkit attempts BCH correction and — on
success — prints the corrected card and exits with code `5`. Pass
the global `--no-auto-repair` flag to opt out and restore the
pre-v0.22 behavior (typed sibling-codec error, exit `1` or `2`).

For `mk1` specifically, the toolkit's auto-fire is essentially
redundant: `mk-codec` performs INTERNAL BCH correction at the same
`t=4` capacity inside `mk_codec::decode`, so corrupted `mk1` chunks
within capacity are silently fixed before reaching the auto-fire
boundary. Auto-fire is the user-visible repair path for `ms1`
(codex32-delegated; no internal correction) and `md1` (no internal
correction in `md-codec`).

### Refusals

`inspect` surfaces whatever the underlying sibling-codec `decode`
returns; consult the per-codec chapters (`md`, `ms`, `mk-cli`) for
the full per-error taxonomy.

### Advisories

| Trigger | Stderr advisory |
|---|---|
| Any `ms1` inspection (regardless of `--reveal-secret`) | `warning: secret material on stdout — consider redirecting ...` |
