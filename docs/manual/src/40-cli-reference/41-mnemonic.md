# `mnemonic` reference

The integration-layer CLI for the m-format constellation. Twenty subcommands:
[`bundle`](#mnemonic-bundle), [`verify-bundle`](#mnemonic-verify-bundle),
[`convert`](#mnemonic-convert), [`export-wallet`](#mnemonic-export-wallet),
[`import-wallet`](#mnemonic-import-wallet),
[`derive-child`](#mnemonic-derive-child),
[`electrum-decrypt`](#mnemonic-electrum-decrypt),
[`final-word`](#mnemonic-final-word), [`seed-xor`](#mnemonic-seed-xor),
[`seedqr`](#mnemonic-seedqr), [`slip39`](#mnemonic-slip39),
[`ms-shares`](#mnemonic-ms-shares),
[`nostr`](#mnemonic-nostr), [`silent-payment`](#mnemonic-silent-payment),
[`decode-address`](#mnemonic-decode-address),
[`verify-message`](#mnemonic-verify-message), [`repair`](#mnemonic-repair),
[`inspect`](#mnemonic-inspect), [`compare-cost`](#mnemonic-compare-cost),
[`xpub-search`](#mnemonic-xpub-search), and
[`gui-schema`](#mnemonic-gui-schema) (introspection only, no user-facing
semantics). Run any with `--help` for the authoritative flag set; this
reference tracks the current release.

> **Recovering a forgotten BIP-39 passphrase.** If you have your seed
> words (entropy) but not the BIP-39 passphrase (the optional "25th
> word"), `mnemonic` cannot brute-force it. A BIP-39 passphrase has no
> internal verifier — every candidate yields a valid-looking wallet — so
> correctness is only definable against a value you already know (an
> address, xpub, or master-fingerprint), which is outside this tool's
> scope. An external open-source tool does exactly this:
> [**btcrecover**](https://github.com/3rdIteration/btcrecover) (maintained
> fork; [original](https://github.com/gurnec/btcrecover)) searches
> passphrase candidates and confirms each by deriving an address / xpub /
> master-fingerprint at common default paths and matching your known
> value. Pointer current as of 2026-05-25; run untrusted recovery tools
> offline, on an air-gapped machine. This mirrors the `mnemonic --help`
> footer.

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
| `--descriptor <DESCRIPTOR>` | user-supplied descriptor; accepts either a BIP-388 `@N` template (keys supplied via `--slot`) **or a bare concrete descriptor** with inline `[fp/path]xpub` keys (watch-only output); both apostrophe and `h`-form hardened paths are accepted; mutually exclusive with `--template` and `--descriptor-file` |
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
| `--slot <SLOT>` | repeating; `@N.<subkey>=<value>` (subkey: `phrase`, `seedqr`, `entropy`, `xpub`, `master_xpub`, `fingerprint`, `path`, `wif`, `xprv`); for secret-bearing subkeys `=-` reads from stdin. `seedqr` (v0.31.3+) takes a 48- or 96-digit SeedQR string and decodes inline at slot-emit time, materializing the BIP-39 phrase identically to a `@N.phrase=` invocation. |
| `--import-json <FILE\|->` | (v0.27.0) synthesize a bundle from an `import-wallet --json` envelope rather than from `--template` / `--descriptor`; the envelope's `bundle.descriptor` carries the descriptor and `bundle.mk1` chunks decode to per-cosigner xpubs + fingerprints + paths; mutually exclusive with `--template`, `--descriptor`, `--descriptor-file`; seed overlay (`--slot @N.phrase=`) applies to slots where envelope `ms1[N] == ""` (watch-only); supplying overlay for an already-seeded slot is `BadInput` |
| `--import-json-index <N>` | (v0.27.0) pick a specific entry from a multi-entry envelope array (e.g., Bitcoin Core `listdescriptors` with multiple descriptors); required when the envelope has > 1 entry; out-of-range is `BadInput` exit 2 |
| `--help` | print help |

### Worked example

See [Your first bundle](#your-first-bundle) for a single-sig
walkthrough; [Multi-source 2-of-3 multisig](#multi-source-2-of-3-multisig)
for multisig.

### Non-English seeds: `mnem` ms1 faithful preserve

When `--language` is set to a non-English BIP-39 wordlist, `bundle`
emits a **`mnem`-kind ms1 card** (ms-codec 0.3.0+) that stores the
wordlist language on the wire. This means a future `ms decode` or
`mnemonic inspect --ms1` can recover the phrase in the original
language without the caller knowing or specifying `--language` at
decode time. English sources (the default) continue to emit the
classic `entr`-kind ms1 — byte-identical with prior toolkit versions.

See [`ms encode` auto-routing](#entr-vs-mnem-payload-kind-auto-routing)
in the `ms` reference for the full encoding spec. See FOLLOWUP
`toolkit-mnem-ms1-wire-shape-downstream-consumers` for the
downstream-compatibility note for GUI consumers.

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

The preceding `bundle` and `verify-bundle` commands emit stderr
disclosures alongside the JSON / stdout. From `bundle`:

```text
warning: secret material on argv (--slot @0.phrase=) — pipe via --slot @0.phrase=- to avoid /proc/$PID/cmdline exposure
warning: secret material on argv (--slot @1.phrase=) — pipe via --slot @1.phrase=- to avoid /proc/$PID/cmdline exposure
warning: secret material on argv (--slot @2.phrase=) — pipe via --slot @2.phrase=- to avoid /proc/$PID/cmdline exposure
info: non-canonical descriptor; defaulting origin path for @0,@1,@2 to m/48'/0'/0'/2' (BIP-48 cosigner path). Override per-placeholder with [fp/path]@N or --slot @N.path=m/...
warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
```

The `info:` line is the v0.19.0 silent-default-with-stderr-notice
feature firing on this recipe's non-canonical `wsh(andor(...))`
descriptor — the BIP-48 origin path is inferred silently and the
bundle proceeds. `verify-bundle` emits the same three secret-on-argv
warnings (no info-notice — the default-path inference fired once at
bundle-time and is now baked into the envelope).

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
| `--descriptor <DESCRIPTOR>` | user-supplied descriptor; accepts either a BIP-388 `@N` template (keys supplied via `--slot`) **or a bare concrete descriptor** with inline `[fp/path]xpub` keys (watch-only output); both apostrophe and `h`-form hardened paths are accepted |
| `--descriptor-file <DESCRIPTOR_FILE>` | descriptor read from file |
| `--threshold <THRESHOLD>` | multisig threshold |
| `--multisig-path-family <FAMILY>` | bip48 or bip87 |
| `--privacy-preserving` | match a privacy-preserving mk1 emission |
| `--language <LANGUAGE>` | BIP-39 wordlist |
| `--passphrase <PASSPHRASE>` | BIP-39 mnemonic passphrase |
| `--passphrase-stdin` | read `--passphrase` from stdin (raw, NULL-byte preserving); single stdin per invocation |
| `--account <ACCOUNT>` | BIP-32 account index |
| `--slot <SLOT>` | repeating slot input `@N.<subkey>=<value>`; subkeys mirror `mnemonic bundle --slot` (`phrase`, `seedqr`, `entropy`, `xpub`, `master_xpub`, `fingerprint`, `path`, `wif`, `xprv`); for secret-bearing subkeys `=-` reads from stdin. `seedqr` (v0.31.3+) decodes a 48- or 96-digit SeedQR string inline. |
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
silently short-circuit the entire check matrix. See the shared
[Auto-fire behavior (all three subcommands)](#auto-fire-behavior-all-three-subcommands-v0250)
section below for the cross-subcommand summary.

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

The env-var applies uniformly to all three TTY-conditional auto-fire
surfaces — `convert`, `inspect`, and `verify-bundle` — since v0.25.0
extended the v0.22.1 D18 gate to `convert` and `inspect`. It is not
part of the clap `--help` surface (env-vars are not part of
clap-derive) nor the `mnemonic gui-schema` JSON.

---

### Auto-fire behavior (all three subcommands) (v0.25.0)

The TTY-conditional auto-fire contract documented above for
`verify-bundle` applies identically to `convert` and `inspect` since
v0.25.0:

| Subcommand | Trigger | TTY-positive default | TTY-negative default |
|---|---|---|---|
| `convert` | `ms_codec` / `mk_codec` decode failure on `--from ms1=…` or `--from mk1=…` | Auto-fire (exit 5 + repair report) | Typed decode error (exit ≠ 5) |
| `inspect` | `ms_codec` / `mk_codec` / `md_codec` decode failure on any `--ms1` / `--mk1` / `--md1` input | Auto-fire (exit 5 + repair report) | Typed decode error (exit ≠ 5) |
| `verify-bundle` | as above, plus the `--bundle-json` intake path | Auto-fire (exit 5 + repair report; corrected chunk on stdout) | Legacy VerifyCheck row + exit 4 |

The TTY gate exists so scripts that parse the typed error envelope
(or, for `verify-bundle`, the `VerifyCheck` array / JSON envelope's
`checks` field) don't see a single corrupted chunk silently
short-circuit the entire flow. Interactive users see the helpful
auto-fire UX; piped consumers see the v0.22.0-and-earlier behavior
unchanged. Set `MNEMONIC_FORCE_TTY=1` in CI / scripts to opt back into
auto-fire under pipes (same mechanism `mnemonic-gui` uses).

---

## `mnemonic convert`

Single-format conversion across the typed node graph: `phrase`,
`seedqr` (input-only, v0.31.6+), `entropy`, `xpub`, `xprv`, `wif`,
`fingerprint`, `path`, `ms1`, `mk1`, `bip38`, `minikey`,
`electrum-phrase`, `address`.

### Synopsis

```sh
mnemonic convert --from <NODE>=<value> --to <NODE> [--to <NODE>]... [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--from <FROM>` | source node (`phrase=…`, `seedqr=…`, `entropy=…`, `xpub=…`, `xprv=…`, `wif=…`, `ms1=…`, `mk1=…`, `bip38=…`, `minikey=…`, `electrum-phrase=…`); `=-` reads from stdin. `seedqr=<digits>` (v0.31.6+) decodes a 48/60/72/84/96-digit SeedQR string to a BIP-39 phrase then projects to any phrase-reachable target |
| `--to <TO>` | target node; repeating for multi-output. `seedqr` is NOT a valid target (input-only); use `mnemonic seedqr encode` to emit a SeedQR digit-string |
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
| `--script-type <SCRIPT_TYPE>` | `p2pkh` / `p2wpkh` / `p2sh-p2wpkh` / `p2tr` for `(Xpub, Address)` derivation (v0.26.0: `p2pkh` added) |
| `--json` | JSON output |
| `--help` | print help |

### Worked example

See [Minimal recovery walkthrough](#minimal-recovery-walkthrough)
and [Migrating from BIP-39 to the m-format](#migrating-from-bip-39-to-the-m-format).

### Non-English ms1 output: `mnem` kind

When `--from phrase=…` is used with a non-English `--language` and `--to ms1`,
`convert` emits a **`mnem`-kind ms1** (ms-codec 0.3.0+) preserving the
wordlist language on the wire — consistent with `bundle`'s behavior.
English sources and `--from entropy=…` continue to emit the classic
`entr`-kind ms1 (byte-identical with prior versions).

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
| `--descriptor <DESCRIPTOR>` | accepts a concrete descriptor (with or without key origins); a keyless `@N` template is rejected with a pointer to `--template … --slot …` or `--from-import-json` |
| `--threshold <THRESHOLD>` | multisig threshold |
| `--multisig-path-family <FAMILY>` | bip48 or bip87 |
| `--network <NETWORK>` | default mainnet |
| `--language <LANGUAGE>` | ignored (watch-only); accepted for slot-parser symmetry |
| `--account <ACCOUNT>` | account index (default 0) |
| `--slot <SLOT>` | repeating `@N.<subkey>=<value>`; subkeys: `phrase`, `seedqr`, `entropy`, `xpub`, `master_xpub`, `fingerprint`, `path`, `wif`, `xprv` (secret-bearing subkeys, including `seedqr`, are refused by `export-wallet`'s watch-only validator per SPEC §3) |
| `--format <FORMAT>` | `bitcoin-core` (default) / `bip388` / `coldcard` / `jade` / `sparrow` / `specter` / `electrum` / `green` / `bsms` (v0.27.0) |
| `--output <OUTPUT>` | output path (`-` = stdout, default) |
| `--range <RANGE>` | Bitcoin Core `range` field; comma-separated; default `0,999` |
| `--timestamp <TIMESTAMP>` | Bitcoin Core `timestamp` field; `now` (default) or unix seconds |
| `--bitcoin-core-version <BITCOIN_CORE_VERSION>` | 24 or 25 (default 25) |
| `--wallet-name <WALLET_NAME>` | wallet name/label for formats that publish one (Coldcard generic JSON, Sparrow, Specter, Electrum); default `<template-human-name>-<account>` |
| `--taproot-internal-key <TAPROOT_INTERNAL_KEY>` | `nums` or `@N` for `tr-multi-a` / `tr-sortedmulti-a` |
| `--bsms-form <FORM>` | (v0.27.0) BSMS Round-2 emit shape — `4-line` (default; BIP-129-canonical) or `2-line` (lenient excerpt symmetric with the v0.26.0 import-side parser); ignored by every non-BSMS format per the per-format ignored-input contract |
| `--from-import-json <FILE\|->` | (v0.27.0) emit a per-format wallet config from an `import-wallet --json` envelope rather than from `--template` / `--descriptor`; the envelope's `bundle.descriptor` becomes the canonical descriptor, cosigner xpubs decode from `bundle.mk1` per SPEC §3.6.1, network derives from `bundle.network`; mutually exclusive with `--template` and `--descriptor`; `--account` is rejected (envelope's `bundle.account` is authoritative). **(v0.37.0)** for template-requiring file-import formats (`sparrow`/`coldcard`/`jade`/`electrum`) the `--template` is **auto-derived from the envelope descriptor** (so these now round-trip via `--from-import-json`); you still cannot pass `--template` explicitly here (it remains mutually exclusive) |
| `--from-import-json-index <N>` | (v0.27.0) pick a specific entry from a multi-entry envelope array; required when the envelope has > 1 entry |
| `--help` | print help |

### Notes

- **`--wallet-name` length cap.** The Coldcard multisig text (`--format coldcard` with a `wsh-*` / `sh-wsh-*` template) and the byte-identical Jade multisig text (`--format jade`) cap the `Name:` line at 20 Unicode scalar values per the Coldcard reference format. Longer names are truncated to the first 20 characters (not bytes — non-ASCII names are handled at codepoint granularity, so `🤐🤐🤐…` truncates cleanly without splitting a multi-byte sequence).
- **`@N.master_xpub=` parse vs emit.** The `master_xpub` slot subkey parses successfully under any `--format`, but `--format coldcard` with a singlesig template (`bip44` / `bip49` / `bip84`) currently refuses when the subkey is supplied because the resolution pipeline does not yet plumb the master xpub through to the Coldcard generic-JSON top-level `xpub` field (tracked by `design/FOLLOWUPS.md` entry `coldcard-master-xpub-plumbing-pending`, scheduled for v0.8.2). Re-invoke without the `master_xpub` slot to emit the JSON with the top-level `xpub` field omitted (which is what Coldcard accepts in the absence of a depth-0 xpub). Other formats silently ignore the subkey per the per-format ignored-input contract.
- **`--threshold` is REQUIRED for `--format sparrow` multisig.** Bitcoin Core / BIP-388 / Coldcard / Jade auto-default `K = N` (cosigner count) when `--threshold` is omitted, but Sparrow refuses with a missing-info error: Sparrow publishes the threshold in `defaultPolicy.miniscript.script` as `multi(K, ...)` / `sortedmulti(K, ...)`, and silently defaulting `K = N` would emit a wallet that looks like K=N was intentional rather than a missing-input default. Supply `--threshold <K>` explicitly when `--format sparrow` and the template is multisig.
- **`--wallet-name` is REQUIRED for `--format specter`.** Specter Desktop's UX requires an explicit wallet label; emitting a Specter wallet without one produces a wallet that displays as an empty string in the Specter UI (a UX regression vs. the user's likely intent). Other formats fall back to `<template-human-name>-<account>` when `--wallet-name` is omitted; Specter refuses via the SPEC §4 missing-info channel.

### Worked example

See [Exporting to Bitcoin Core / BIP-388 / Sparrow / Specter](#exporting-to-bitcoin-core-bip-388-sparrow-specter).

---

## `mnemonic import-wallet`

Import a third-party wallet blob into an m-format bundle. Parses a
foreign wallet export (BSMS Round-2 per BIP-129, or Bitcoin Core's
`listdescriptors` JSON), reconstructs the equivalent watch-only
bundle, and round-trips it back through the toolkit canonicalizer
to surface byte-exact vs semantic-only equivalence (see [foreign
wallet formats](#foreign-wallet-formats) for the format taxonomy).

v0.26.0 ships two source formats — `bsms` and `bitcoin-core` —
selectable via `--format` or auto-detected by sniff. Both formats
are watch-only by design; the resulting bundle's cosigners carry no
secret material unless the user supplies an `--ms1` / `--slot
@N.phrase=` seed overlay (see [seed overlay](#mnemonic-import-wallet-seed-overlay)).
Bitcoin Core blobs containing `xprv` extended private keys are
refused (re-run `bitcoin-cli listdescriptors` without the `true`
flag to obtain xpub-only output).

### Synopsis

```sh
mnemonic import-wallet --blob <FILE|-> [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--blob <FILE\|->` | path to the third-party wallet blob; `-` reads from stdin (required UNLESS `--bsms-round1` is supplied as a standalone Round-1 verify mode) |
| `--no-auto-repair` | (global) skip auto-fire repair on decode failures; same global flag honored by `convert` / `inspect` / `verify-bundle` |
| `--format <bsms\|bitcoin-core>` | format override; if absent, auto-detected via sniff (SPEC §6) |
| `--select-descriptor <N\|active-receive\|active-change\|all>` | multi-descriptor selector for Bitcoin Core blobs (SPEC §5.3); accepts integer index, `active-receive`, `active-change`, or `all` (default); BSMS blobs coerce non-default values to `all` with stderr NOTICE |
| `--ms1 <STRING>` | seed overlay (SPEC §8.3): supply the secret material that matches the blob's declared xpub at the cosigner's origin path; repeatable + positional cosigner-index — the i-th `--ms1` applies to cosigner i; cosigners not addressed by any `--ms1[N]` flag remain watch-only (no entropy attached); accepts the `@env:VAR` sentinel; empty-string `""` preserves the v0.25.1 watch-only sentinel |
| `--slot <@N.phrase=<phrase>>` | per-slot seed overlay; equivalent to `--ms1` but the phrase is converted to entropy and the derived xpub at the cosigner's origin path is compared against the blob's xpub; mutually exclusive with `--ms1[N]` for the same N; accepts `@env:VAR`; only the `phrase` subkey is accepted on `import-wallet` in v0.26.0 |
| `--json` | emit a JSON envelope array on stdout (SPEC §7.4) instead of the human-readable summary; the v0.27.0 envelope `bundle` field is the full toolkit-native `BundleJson` shape (was a parse-side summary in v0.26.0; the v0.27.0 wire-shape replacement is documented in CHANGELOG `### Changed`) plus a new top-level `schema_version: "1"` field |
| `--bsms-encryption-token <FILE\|->` | (v0.31.0) BIP-129 §Encryption decrypt token; reads session TOKEN from PATH (or `-` for stdin); applies PBKDF2-SHA512 + AES-256-CTR + HMAC-SHA256 per BIP-129 §Encryption. Combine with `--format bsms` to decrypt encrypted Round-2 wallet shares (`--blob`), **OR (v0.32.1)** with `--bsms-round1` to decrypt encrypted Round-1 key records. **(v0.32.2) repeatable** (BIP-129 line 74: one shared TOKEN or one per Signer): a SINGLE `--bsms-encryption-token` is SHARED — it decrypts every encrypted Round-1 record AND the Round-2 blob (backward-compatible). MULTIPLE tokens are paired POSITIONALLY with `--bsms-round1` records (the Nth token decrypts the Nth record); per-Signer mode requires every `--bsms-round1` record to be encrypted, the token count to equal the record count, and NO encrypted Round-2 `--blob` in the same invocation (a single Round-2 share carries a single token → supplying multiple tokens with an encrypted blob is refused). Token file contents: lowercase ASCII hex (16 chars STANDARD, 32 chars EXTENDED); whitespace stripped; uppercase normalized. At most one token may read from stdin (`-`). Encrypted Round-2 blobs lack the `BSMS 1.0` header so `--format bsms` is REQUIRED for the encrypted Round-2 path. MAC verify failure → exit 2 (typed `BsmsMacMismatch`). |
| `--bsms-round1 <FILE>` | (v0.27.0) BIP-129 Round-1 key record (Signer → Coordinator) for BIP-322 ECDSA signature verification; repeating flag — one per record; each record verified independently; verify state propagates to `--json` envelope's `bsms_round1_verifications` field; standalone mode (no `--blob` supplied) emits per-record verify envelope and exits 0 on verify success; v0.27.0 accepts a file path only — stdin form `-` is rejected, supply a file path per record (FOLLOWUP: multi-record stdin intake). **(v0.32.1)** the record file may be EITHER plaintext (5-line `BSMS 1.0\n…`) OR an ENCRYPTED Round-1 wire (hex `MAC \|\| ciphertext`); encrypted records are auto-detected (raw hex, no `BSMS 1.0` header) and decrypted with `--bsms-encryption-token` (MAC-verified per BIP-129 §Encryption) before the BIP-322 verify. An encrypted record supplied without `--bsms-encryption-token` → `BadInput` (exit 1); MAC verify failure → exit 2 (`BsmsMacMismatch`). |
| `--bsms-verify-strict` | (v0.27.0) make BIP-129 Round-1 SIG verification failures fatal; without this flag, verify mismatches emit a stderr NOTICE and proceed with `signature_verified: false`; requires `--bsms-round1` to be meaningful |
| `--decrypt-password <VALUE>` | (v0.33.2) password for an Electrum **BIE1** (user-password) storage-encrypted wallet file. A storage-encrypted Electrum wallet is a single base64 blob (decoded magic `BIE1`), NOT JSON; the toolkit auto-detects it and decrypts it to the wallet JSON (ECIES: PBKDF2-HMAC-SHA512 → secp256k1 key → AES-128-CBC + HMAC-SHA256 + zlib) BEFORE sniff/parse, then imports watch-only as usual. Only consumed when a `BIE1` blob is detected; ignored (with a stderr notice) otherwise. Inline form emits an argv-leakage advisory — prefer `--decrypt-password-file` / `--decrypt-password-stdin`. Wrong password → `decryption failed (wrong password or corrupted wallet file)`. Mutually exclusive with the other two `--decrypt-password*` forms. |
| `--decrypt-password-file <PATH>` | (v0.33.2) read the BIE1 decryption password from a file (one trailing newline stripped). |
| `--decrypt-password-stdin` | (v0.33.2) read the BIE1 decryption password from stdin (NULL-byte preserving). Cannot co-exist with any other stdin consumer (`--blob=-`, `--bsms-encryption-token=-`). |
| `--network <mainnet\|testnet\|signet\|regtest>` | (v0.34.6) re-bind the imported network to disambiguate **signet/regtest** from the coin-type-1→testnet collapse (BIP-129 BSMS + Bitcoin Core `listdescriptors` use coin-type `1` for testnet/signet/regtest alike, so the network is collapsed to testnet by default). Honored ONLY within the parsed coin-type class (testnet ↔ {testnet, signet, regtest}; mainnet ↔ mainnet) — a cross-class request (e.g. `--network mainnet` on a testnet-coin-type blob) is refused (exit 1, `ImportWalletNetworkClassMismatch`) because the blob's xpub prefix is coin-type-bound. Absent = use the coin-type-derived network. Note: signet shares testnet's address params (`tb1…`), so `testnet→signet` changes only the network label; `testnet→regtest` changes the HRP to `bcrt1…`. |
| `--help` | print help |

### Description

The default mode emits the synthesized engraving card(s) on stdout
— the same byte-shape `mnemonic bundle` produces — separated by
`\n;\n` when a single invocation yields multiple bundles (Bitcoin
Core blobs with `--select-descriptor all` and N ≥ 2 entries). Round-
trip discipline (SPEC §7) runs canonicalize-on-input vs canonicalize-
on-re-emit; if the comparison yields a non-byte-exact / semantic-only
match, a unified diff is printed to stderr.

`--json` mode replaces the engraving-card stdout with a JSON array,
one envelope per emitted bundle. Each envelope carries:

- `bundle` — parse-side summary of the shape `{cosigners: [{fingerprint, path_raw, xpub, has_entropy}], network, threshold}` (v0.26.0 ships this summary; the full toolkit-native `BundleJson` shape is FOLLOWUP `wallet-import-json-envelope-full-bundle`, v0.27+).
- `source_format` — `"bsms"` or `"bitcoin-core"`.
- `roundtrip` — `{byte_exact: bool, semantic_match: bool, diff: Option<String>, status: "ok" | "blocked_no_emitter" | "canonicalize_failed"}`. The `diff` field is `Some(...)` iff `byte_exact == false`; under `--json` the diff lives in the envelope only (stderr is silent).
- `bsms_audit?` — BSMS source only: `{token, signature, first_address, derivation_path, signature_verified: false}`. v0.26.0 preserves these fields verbatim from the Round-2 blob but does not verify the signature (FOLLOWUP `bsms-verify-signatures`) or the first-address (FOLLOWUP `bsms-first-address-verify`).
- `source_metadata?` — Bitcoin Core source only: per-entry `active` / `internal` / `range` / `wallet_name` preserved from the input.

### `--ms1` / `--slot @N.phrase=` seed overlay {#mnemonic-import-wallet-seed-overlay}

By default, `import-wallet` produces a watch-only bundle: each
cosigner carries its blob-declared xpub and origin path but no
entropy. To re-attach secret material to a known cosigner, pass
`--ms1 <ms1-string>` (or `--slot @N.phrase=<BIP-39 phrase>`) at the
positional cosigner-index. The toolkit derives the xpub from the
supplied entropy at the cosigner's declared origin path and asserts
equality against the blob's declared xpub. Mismatch returns exit 4
with stderr `error: import-wallet: cosigner <N>: supplied seed
produces xpub <X> at path <P>; blob declares <Y>`.

The `@env:<VAR>` sentinel (SPEC §3) resolves at clap-parse time via
`std::env::var(VAR)`. Whole-value only — `--ms1 prefix@env:VAR` is
treated as literal text. Missing or unset env-var → exit 1 with
`error: --ms1: env-var VAR referenced by sentinel is not set`.
Pipe entropy via `@env:VAR` sentinel to avoid argv-leak; the
v0.11.0 GUI emits typed values verbatim, so users must type
`@env:VAR` explicitly themselves (per FOLLOWUP
`gui-import-wallet-env-var-secret-channel` v0.12.0+ for
auto-rewriting).

### Exit codes

| Code | Meaning |
|---|---|
| `0` | success (round-trip ok; may emit WARNING for semantic-only match) |
| `1` | `ImportWalletAmbiguousFormat`, `ImportWalletFormatMismatch`, `EnvVarMissing` — user-input or generic |
| `2` | `ImportWalletParse`, `ImportWalletXprvForbidden`, `ImportWalletWatchOnlyViolation` — format-violation / refusal |
| `3` | future-format refusal (e.g., `BSMS 2.0`) — via existing `FutureFormat` From-impl |
| `4` | `ImportWalletSeedMismatch` — supplied seed does not match blob's declared xpub at the cosigner's origin path |
| `5` | repair short-circuit — BCH-correctable BSMS descriptor `mk1` chunk; see [auto-fire on decode failure](#auto-fire-on-decode-failure-v0221) |

### Stderr templates

| Class | Template |
|---|---|
| WARNING (exit 0) | `warning: import-wallet: bsms: 2-line excerpt; full BIP-129 Round-2 carries token + signature + first-address verification fields; accepting reduced form` |
| WARNING (exit 0) | `warning: import-wallet: bsms: signature present but not verified in v0.26.0; see FOLLOWUP \`bsms-verify-signatures\`` |
| WARNING (exit 0) | `warning: import-wallet: roundtrip not byte-exact; semantic equivalent; diff below` (+ unified-diff body on stderr OR in `--json` envelope, never both) |
| NOTICE (exit 0) | `notice: import-wallet: bsms: --select-descriptor <X> has no effect; BSMS Round-2 carries a single descriptor` |
| NOTICE (exit 0) | `notice: import-wallet: bitcoin-core: dropped wallet-state fields <fields>: not preserved in bundle output (key-state only)` |
| NOTICE (exit 0) | `notice: import-wallet: electrum: wallet is encrypted (use_encryption=true); importing watch-only material only (encrypted seed/xprv/passphrase/keypairs fields ignored). To extract the encrypted seed, use 'electrum --decrypt-wallet' out-of-band then re-import the plaintext wallet.` |
| NOTICE (exit 0) | `notice: import-wallet: bsms: BIP-129 encrypted Round-2 envelope decrypted (token width <N> hex chars; MAC verified)` |
| NOTICE (exit 0) | `notice: import-wallet: --bsms-round1: BIP-129 encrypted Round-1 record <i> decrypted (token width <N> hex chars; MAC verified)` (v0.32.1) |
| NOTICE (exit 0) | `notice: import-wallet: electrum: BIE1 user-password storage decrypted` (v0.33.2) |
| NOTICE (exit 0) | `notice: import-wallet: no BIE1 storage-encrypted wallet detected; --decrypt-password* ignored` (v0.33.2; emitted when a `--decrypt-password*` flag is supplied for a non-encrypted wallet) |
| Error (exit 2) | `error: import-wallet: bsms: BIP-129 MAC verification failed (token width <N> hex chars; wrong token or tampered ciphertext)` |
| Error (exit 1) | `error: import-wallet: electrum: this wallet is encrypted with a hardware-device key (BIE2 / XPUB_PASSWORD); it cannot be decrypted from a password…` (v0.33.2) |
| Error (exit 1) | `error: import-wallet: electrum: decryption failed (wrong password or corrupted wallet file)` (v0.33.2) |
| Error (exit 1) | `error: import-wallet: electrum: BIE1 storage-encrypted wallet detected; supply the wallet password via --decrypt-password, --decrypt-password-file, or --decrypt-password-stdin` (v0.33.2) |
| Error (exit 1) | `error: import-wallet: could not detect format; supply --format <bsms\|bitcoin-core>` |
| Error (exit 1) | `error: import-wallet: --format <X> supplied but blob looks like <Y>` |
| Error (exit 1) | `error: <flag>: env-var <VAR> referenced by sentinel is not set` |
| Error (exit 2) | `error: import-wallet: <format>: parse error: <detail>` |
| Error (exit 2) | `error: import-wallet: bitcoin-core: xprv-bearing descriptor refused; re-run \`bitcoin-cli listdescriptors\` without \`true\` to get xpub-only output` |
| Error (exit 3) | `error: future format: bsms: version "<V>"; toolkit supports "1.0"` |
| Error (exit 4) | `error: import-wallet: cosigner <N>: supplied seed produces xpub <X> at path <P>; blob declares <Y>` |

The first-address-mismatch WARNING is deferred to v0.27+ (FOLLOWUP
`bsms-first-address-verify`): the audit field is preserved verbatim
in `--json` envelope's `bsms_audit.first_address` for the user to
re-verify externally, but toolkit-side derivation requires a Phase-4
derivation helper not present in v0.26.0.

### Worked example — BSMS Round-2 decaying-multisig import

The kickoff seed-case for v0.26.0: a BSMS Round-2 2-line excerpt
emitted by a coordinator for a `wsh(thresh(...))` decaying-multisig
descriptor (flagship use case per SPEC §10.1).

```sh
cat > /tmp/decay-32768.bsms <<'EOF'
BSMS 1.0
wsh(thresh(2,pk([73c5da0a/48h/0h/0h/2h]xpub6E.../<0;1>/*),s:pk([4e1f...]xpub6F.../<0;1>/*),sln:older(32768)))#abcdefgh
EOF
mnemonic import-wallet --blob /tmp/decay-32768.bsms
```

Stdout (the synthesized engraving cards; the bundle is watch-only,
so the `ms1` line is the watch-only sentinel `""`):

```text
ms1: ""
mk1[0]: mk10... (cosigner @0 origin [73c5da0a/48h/0h/0h/2h])
mk1[1]: mk10... (cosigner @1 origin [4e1f.../...])
md1: md10... (decaying-multisig descriptor)
```

Stderr:

```text
warning: import-wallet: bsms: 2-line excerpt; full BIP-129 Round-2 carries token + signature + first-address verification fields; accepting reduced form
```

Exit code: `0`. Append `--ms1 <ms1-string>` (or `--slot
@0.phrase=...`) to attach entropy to cosigner @0; the toolkit will
derive the xpub at the declared origin path and assert match
against the blob's xpub.

### Worked example — Bitcoin Core `listdescriptors` multipath import

Bitcoin Core 25+ emits `listdescriptors` output with the
`<0;1>/*` multipath shape on the canonical receive/change pair.
Importing this directly yields one bundle per descriptor entry
(use `--select-descriptor active-receive` to filter to just the
external chain).

```sh
bitcoin-cli listdescriptors > /tmp/core-export.json
mnemonic import-wallet --blob /tmp/core-export.json --select-descriptor active-receive --json
```

Stdout (one envelope per emitted bundle; `[...]` collapsed for
brevity):

```json
[
  {
    "bundle": {
      "cosigners": [{"fingerprint": "73c5da0a", "path_raw": "[73c5da0a/84h/0h/0h]", "xpub": "xpub6CatWdi...", "has_entropy": false}],
      "network": "mainnet",
      "threshold": null
    },
    "source_format": "bitcoin-core",
    "source_metadata": {"wallet_name": "mywallet", "active": true, "internal": false, "range": [0, 999]},
    "roundtrip": {"byte_exact": true, "semantic_match": true, "diff": null, "status": "ok"}
  }
]
```

Stderr is silent under `--json` (the diff lives in the envelope).
Re-run without `--json` to get the human-readable engraving card
on stdout + the round-trip status on stderr.

### Refusals

| Trigger | Refusal |
|---|---|
| Bitcoin Core blob contains `xprv` | exit 2 — see `xprv-bearing descriptor refused` stderr template above |
| Cosigner in `ParsedImport.cosigners` carries entropy post-parse | exit 2 — `error: import-wallet: cosigner <N> has entropy populated post-parse; watch-only invariant violated (internal bug)` |
| BSMS line 1 is not `BSMS 1.0` | exit 2 `ImportWalletParse` |
| BSMS version > 1.0 (e.g., `BSMS 2.0`) | exit 3 via existing `FutureFormat` From-impl |
| Sniff finds no match AND no `--format` supplied | exit 1 — see `could not detect format` stderr template |
| Sniff finds positive match for format X AND `--format Y` supplied | exit 1 — see `--format X supplied but blob looks like Y` template |
| Auto-detect ambiguity (≥2 parsers' sniff return true) | exit 1 — `blob matches multiple format heuristics; supply --format <X>` |
| Supplied `--ms1` derives a different xpub than declared at cosigner's path | exit 4 `ImportWalletSeedMismatch` (see template above) |
| `@env:VAR` sentinel references unset env-var | exit 1 `EnvVarMissing` (see template above) |
| Invalid env-var name (e.g., `@env:1FOO`, `@env:`) | exit 1 `EnvVarMissing` with stderr `invalid env-var name '<VARNAME>'` |

### Advisories

The `--ms1` / `--slot @N.phrase=` overlay flags carry secret material
on argv; pipe entropy via `@env:VAR` sentinel to avoid argv-leak;
the v0.11.0 GUI emits typed values verbatim, so users must type
`@env:VAR` explicitly themselves (per FOLLOWUP
`gui-import-wallet-env-var-secret-channel` v0.12.0+ for
auto-rewriting).
Re-emitted Bitcoin Core blobs DROP `timestamp` / `next` / `next_index`
fields (wallet-state, not key-state); the dropped-fields NOTICE
template above fires when input carries any of these. BSMS Round-2
re-emission via `mnemonic export-wallet --format bsms` is FOLLOWUP
`wallet-export-bsms-emitter` (blocks the BSMS bundle round-trip
discipline; `--json` envelope reports `status: "blocked_no_emitter"`
in the interim).

### What's NOT supported

v0.26.0 ships two source formats only. Sparrow's `.json`, Specter's
`.json`, Electrum's wallet file, and Coldcard's generic JSON / multisig-
text are NOT yet importable. See [foreign wallet
formats](#foreign-wallet-formats) for the full coverage matrix and
the FOLLOWUPs queued for v0.27+ (`wallet-import-sparrow`,
`wallet-import-specter`, `wallet-import-electrum`,
`wallet-import-coldcard`).

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

## `mnemonic electrum-decrypt`

Decrypt an Electrum **field-encrypted** secret (a base64 `iv ‖
aes-256-cbc(plaintext + PKCS7)` blob, key = `sha256d(password)` per
Electrum's `_hash_password` version 1) and emit the recovered plaintext —
an Electrum-native seed phrase or a BIP-32 xprv (the keystore type
determines which; the wire carries no discriminator, so the output is
emitted opaquely). Surfaces the `electrum_crypto::decrypt_field` library
primitive (cross-impl-validated against the Python `cryptography` backend).

### Synopsis

```sh
mnemonic electrum-decrypt --ciphertext <VALUE|-> (--decrypt-password <VAL> | --decrypt-password-file <PATH> | --decrypt-password-stdin) [--json-out <PATH>]
```

### Flags

| Flag | Purpose |
|---|---|
| `--ciphertext <VALUE\|->` | the Electrum field-encrypted secret as base64; `-` reads from stdin. NOT secret (it is ciphertext) — no argv advisory |
| `--decrypt-password <VALUE>` | decryption password (inline); emits an argv-leakage advisory — prefer the stdin/file forms. Exactly one password form is required |
| `--decrypt-password-file <PATH>` | read the password from a file (single trailing newline stripped) |
| `--decrypt-password-stdin` | read the password from stdin (raw, NULL-byte preserving); single stdin per invocation (mutually exclusive with `--ciphertext -`) |
| `--json-out <PATH>` | emit a JSON envelope (`{schema_version, operation, plaintext}`; no password echo) instead of plain text on stdout; emits a world-readable-permissions advisory if the file is group/other-readable |
| `--help` | print help |

The three password forms are mutually exclusive and exactly one is
required (clap arg-group; missing/multiple → exit 64). A wrong password
(or corrupted ciphertext) surfaces as `electrum-decrypt: decryption failed
(wrong password or corrupted ciphertext)` (exit 1) — Format A field
encryption carries no MAC, so the two underlying failure modes (PKCS7
unpad refusal / non-UTF-8 result) are reported uniformly. The recovered
plaintext on stdout is private key material and emits the
output-class advisory.

### Worked example

```sh
mnemonic electrum-decrypt \
  --ciphertext ABEiM0RVZneImaq7zN3u/zY0181f7qAY/NWiVQFLdHE= \
  --decrypt-password-stdin <<<'test-password'
# → hello world
```

For a whole-file-encrypted Electrum wallet (Format B), see
[§Foreign formats](../45-foreign-formats.md) — that path is
`import-wallet`, not `electrum-decrypt`.

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
| `split` (always, unconditional) | `warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '\| age -e ...')` followed by `note: each share is secret material — distribute across separate locations; SLIP-39 shares have no authentication tag` |
| `combine` (always, unconditional) | `warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '\| age -e ...')` followed by `note: verify the recovered wallet's expected derived address before trusting` |
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

## `mnemonic ms-shares` {#mnemonic-ms-shares}

BIP-93\index{BIP-93} **codex32**\index{codex32} K-of-N share splitting
of an `ms1` secret. Two sub-subcommands: `split` (a secret →
N codex32 shares) and `combine` (≥K shares → the recovered secret).
Like [`slip39`](#mnemonic-slip39) this is a true threshold scheme — any
K-of-N subset of shares reconstructs — but the shares are `ms1`
strings (the same human-typeable codex32 alphabet as a single-string
ms1 card), produced by codex32's native `threshold(k)`+`index` Shamir
mechanism over `GF(32)`. This is the toolkit front-end for the
[`ms split` / `ms combine`](#ms-split) ms-cli surface; the recovered
`ms1` (`combine --to ms1`) composes with the rest of the toolkit (feed
it to `bundle --slot @0.ms1=…`).

The `mnem`-vs-`entr` payload kind survives the split: a non-English
`--language` source splits as a `mnem` share-set so the BIP-39 wordlist
language is preserved on the wire; an English phrase or raw entropy
splits as a plain `entr` share-set.

### Concept signposts

- **Secret** — the BIP-39 phrase or raw entropy that `split` consumes /
  `combine` recovers (the same payload an `ms1` card carries). Sizes:
  16/20/24/28/32 bytes (12/15/18/21/24 BIP-39 words).
- **Share**\index{codex32 share} — a single distributed `ms1`-format
  codex32 string emitted by `split`, carrying the threshold digit `k`,
  a random per-split identifier, and a non-`s` share index. The whole
  N-share SET is secret-equivalent.
- **Threshold (`K`)**\index{threshold} — the minimum number of shares
  that recombine (2..=9; the codex32 threshold field is a single ASCII
  digit, so `0` is the unshared single-string sentinel and `1` is
  invalid).
- **Share count (`N`)** — total shares emitted (K ≤ N ≤ 31; there are
  exactly 31 valid non-`s` codex32 share indices).
- **Identifier** — random 4-character per-split tag shared across all
  shares of one split; `combine` rejects a mixed-identifier set.
- **Secret share (index `s`)** — the codex32 secret-carrying share at
  index `s` is NEVER a valid `combine` input (it would short-circuit
  interpolation and bypass validation); `combine` rejects it.

### Synopsis

```sh
mnemonic ms-shares split   --from <phrase=…|entropy=…> --threshold K --shares N [OPTIONS]
mnemonic ms-shares combine --share <ms1-share-or-> ... [OPTIONS]
```

### `ms-shares split` flags

| Flag | Purpose |
|---|---|
| `--from <phrase=…\|entropy=…>` | secret as `phrase=<value-or->` or `entropy=<hex-or->`; `=-` reads from stdin. Inline forms emit an argv-leakage advisory |
| `--threshold <K>` | threshold K — minimum shares needed to recombine (2..=9) |
| `--shares <N>` | total shares N to emit (K ≤ N ≤ 31) |
| `--language <LANGUAGE>` | BIP-39 wordlist of the input phrase; ignored for `entropy=` inputs. A non-English language produces a `mnem` share-set so the wordlist survives the split |
| `--json` | emit a JSON object on stdout (`{"shares": [...]}`) instead of the one-share-per-line text form |
| `--no-auto-repair` | global flag; skip auto-fire BCH repair on a decode failure (see [`verify-bundle` auto-fire](#mnemonic-verify-bundle)) |
| `--help` | print help |

### `ms-shares combine` flags

| Flag | Purpose |
|---|---|
| `--share <ms1-share-or->` | repeating share input; supply at least K. At most ONE may be `-` (stdin). Inline values emit a per-occurrence argv-leakage advisory |
| `--to <phrase\|entropy\|ms1>` | output shape (default `phrase`); `phrase` emits a BIP-39 mnemonic (language per the recovered card / `--language`), `entropy` emits hex, `ms1` re-encodes a recovered single-string ms1 |
| `--language <LANGUAGE>` | BIP-39 wordlist for `--to phrase` when the recovered secret is a plain `entr` payload (no wire language); ignored for `mnem` payloads and for `--to entropy`/`--to ms1` |
| `--json` | emit a JSON object on stdout instead of the plain secret line |
| `--no-auto-repair` | global flag; skip auto-fire BCH repair on a decode failure |
| `--help` | print help |

### Worked example — 2-of-3 split + recombine

The canonical zero-entropy 24-word master `abandon × 23 + art` (matching
the [`seed-xor`](#mnemonic-seed-xor) / [`slip39`](#mnemonic-slip39)
precedent). Share text is shown as `<share-N>` placeholders because
`split` is CSPRNG-driven (the random identifier and the non-defining
share payloads are random); run the commands locally to see actual
share text.

```sh
echo "abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon art" |
  mnemonic ms-shares split --from phrase=- --threshold 2 --shares 3
```

Stdout: 3 `ms1`-format codex32 shares, one per line, each carrying the
threshold digit `2`, a shared random identifier, and a distinct non-`s`
index. Reverse with any 2:

```sh
mnemonic ms-shares combine --share "<share-1>" --share "<share-2>" \
  --to phrase --language english
```

Stdout: the original `abandon × 23 + art` 24-word phrase. (Without
`--to phrase`, `combine` defaults to `--to phrase`; use `--to entropy`
for 64 hex chars or `--to ms1` for a single recovered ms1 string.)

> **Compose with `bundle`.** A recovered single-string ms1
> (`combine --to ms1`) is a normal `ms1` card payload — feed it to
> `mnemonic bundle --slot @0.ms1=<recovered-ms1>` to rebuild the rest of
> the bundle.

### Non-English (`mnem`) split

A non-English source preserves its wordlist language across the share
set:

```sh
mnemonic ms-shares split --from phrase=- --language japanese \
  --threshold 2 --shares 3 < ja-phrase.txt
```

The shares are `mnem`-kind; `combine --to phrase` recovers the phrase in
its wire language (Japanese) regardless of the `--language` flag, which
is honored only for plain `entr` recoveries.

### Output class

Both `split` and `combine` emit private key material on stdout — the
whole N-share SET is secret-equivalent, and the recovered secret
obviously is — so both print the
`warning: stdout carries private key material (can spend) …` stderr
advisory. Entropy intermediates are held in zeroizing buffers. Engrave
each share on its own backup medium; storing K shares together
re-creates a single-point-of-failure.

### Refusals

- `--threshold` outside 2..=9, or `--shares` outside K..=31 → usage
  error (exit 64).
- `combine` with fewer than K shares → a codex32 "threshold not passed"
  refusal.
- a repeated share index, a mixed identifier/threshold/length, or the
  secret share at index `s` → a friendly codex32 / share refusal.

---

## `mnemonic seedqr`

SeedQR is an open spec originated by [SeedSigner](https://seedsigner.com/seedqr-instructions/):
a BIP-39 mnemonic encoded as a numeric-string QR payload where each
English-wordlist index is rendered as a 4-digit zero-padded decimal.
12-word phrases produce 48 digits; 24-word phrases produce 96.

`mnemonic seedqr` has two subsubcommands:

- `decode` — read a SeedQR numeric string, emit the BIP-39 phrase.
- `encode` — read a BIP-39 phrase, emit the SeedQR numeric string.

### Synopsis

```text
mnemonic seedqr decode --from seedqr=<VALUE|-> [--variant <standard|compact>] [--json-out <PATH>]
mnemonic seedqr encode --from phrase=<VALUE|-> [--variant <standard|compact>] [--json-out <PATH>]
```

### Flags

`decode`:

- `--from seedqr=<VALUE|->`: **(canonical, v0.31.6+)** the SeedQR payload. Under `--variant standard` (default) this is a numeric digit string (48, 60, 72, 84, or 96 ASCII digits — 12 / 15 / 18 / 21 / 24-word phrases). Under `--variant compact` this is lowercase hex of the raw BIP-39 entropy bytes (32 hex chars = 16 bytes = 12-word; 64 hex chars = 32 bytes = 24-word). `seedqr=-` reads from stdin. Only the `seedqr` node type is accepted.
- `--variant <standard|compact>`: **(v0.32.0+)** SeedQR variant (default `standard`). See [§Scope](#scope-v0300-widened-in-v0315-v0320) below.
- `--digits <VALUE|->`: **(DEPRECATED, v0.31.6)** the original digit-string flag (Standard variant only). Still accepted, but emits a stderr deprecation notice directing to `--from seedqr=`; will be removed in a future release. Mutually exclusive with `--from` (clap-level conflict; exit 64). Exactly one of `--from seedqr=` or `--digits` is required.
- `--json-out <PATH>`: emit a JSON envelope at PATH instead of plain text on stdout.

The equivalent Standard conversion is also reachable via `mnemonic convert --from seedqr=<digits> --to phrase` (the `seedqr` node type was unified into the shared `--from` grammar in v0.31.6).

`encode`:

- `--from phrase=<VALUE|->`: BIP-39 phrase (12, 15, 18, 21, or 24 English words for Standard; 12 or 24 only for Compact). `phrase=-` reads from stdin. The toolkit refuses non-phrase node types (`xpub=`, `ms1=`, etc.).
- `--variant <standard|compact>`: **(v0.32.0+)** SeedQR variant (default `standard`). Standard emits the decimal digit string; Compact emits lowercase hex of the entropy bytes.
- `--json-out <PATH>`: emit a JSON envelope at PATH instead of plain text on stdout. The envelope's `variant` field reflects the selected variant; the `digits` field holds the payload (decimal for standard, hex for compact).

Both subsubcommands emit an argv-leakage advisory on stderr when the
secret is supplied inline (e.g., `--from seedqr=<value>`, the deprecated
`--digits <value>`, or `--from phrase=<value>`).
Use the stdin form (`-`) to avoid the advisory.

### Scope (v0.30.0, widened in v0.31.5 + v0.32.0)

- **Variants:** Standard SeedQR (decimal digit string) + CompactSeedQR (v0.32.0+; raw BIP-39 entropy bytes, the SeedSigner binary-mode QR payload, represented on the CLI as lowercase hex). Select via `--variant <standard|compact>` (default `standard`).
  - **Standard** word counts: 12 / 15 / 18 / 21 / 24 — the complete BIP-39 word-count set (v0.30.0 shipped 12 + 24; v0.31.5 widened to all 5 per FOLLOWUP `seedqr-15-18-21-word-counts`). SeedQR encodes 4 decimal digits per BIP-39 word index, agnostic to word count.
  - **Compact** word counts: **12 and 24 only**, matching SeedSigner's `CompactSeedQrEncoder` (which strips the trailing checksum bits for exactly those two cases). 15 / 18 / 21 are refused for compact (`compact: invalid word count: N (CompactSeedQR supports only 12 or 24)`). The compact payload equals the raw BIP-39 entropy: 16 bytes (12-word) or 32 bytes (24-word).
- **Language:** English only. SeedQR's open spec defines the encoding against the BIP-39 English wordlist.

### Worked example — compact encode + binary QR render

The CLI emits the compact payload as hex; pipe through `xxd -r -p` to get
the raw bytes for a binary-mode QR:

```sh
mnemonic seedqr encode --variant compact --from phrase='abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about'
# → 00000000000000000000000000000000   (16 entropy bytes as 32 hex chars)

mnemonic seedqr encode --variant compact --from phrase='…' \
  | xxd -r -p \
  | qrencode -8 -o compact-seedqr.png   # -8 = byte mode
```

Decode a scanned CompactSeedQR (hex of the scanned bytes):

```sh
mnemonic seedqr decode --variant compact --from seedqr=00000000000000000000000000000000
# → abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

### Worked example — decode

```sh
mnemonic seedqr decode --from seedqr=000000000000000000000000000000000000000000000003
```

Stdout:

```text
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

JSON envelope form:

```sh
mnemonic seedqr decode --from seedqr=000000000000000000000000000000000000000000000003 --json-out /tmp/decode.json
cat /tmp/decode.json
```

`/tmp/decode.json` contents:

```json
{
  "schema_version": "1",
  "operation": "decode",
  "variant": "standard",
  "word_count": 12,
  "phrase": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
  "digits": "000000000000000000000000000000000000000000000003"
}
```

### Worked example — encode

```sh
mnemonic seedqr encode --from phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
```

Stdout:

```text
000000000000000000000000000000000000000000000003
```

Pipe to a QR generator:

```sh
mnemonic seedqr encode --from phrase="abandon ... about" | qrencode -o out.png -
```

### Worked example — 24-word vector

The canonical Trezor 24-word `all-abandon-art` vector encodes to 92
zero-padded digits followed by `0102` (BIP-39 English index of "art"):

```sh
mnemonic seedqr encode --from phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
```

Stdout:

```text
000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000102
```

Round-tripping through `decode` yields the original 24-word phrase
byte-for-byte.

### Cross-impl smoke recipe

`mnemonic seedqr encode` is byte-identical to SeedSigner's Python
reference encoder at `src/seedsigner/models/encode_qr.py::SeedQrEncoder`.
Verify locally:

```sh
git clone https://github.com/SeedSigner/seedsigner /tmp/ss
cd /tmp/ss
python3 -c "
import sys; sys.path.insert(0, 'src')
from seedsigner.models.encode_qr import SeedQrEncoder
phrase = 'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about'
enc = SeedQrEncoder(mnemonic=phrase.split())
print(enc.data)
"
```

Expected: `000000000000000000000000000000000000000000000003`. Compare
against the toolkit:

```sh
mnemonic seedqr encode --from phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
```

The two outputs match byte-for-byte.

### Exit codes

- `0` — success.
- `1` — `BadInput` (any `SeedqrError` variant: invalid digit count/character, word index out of range, wrong word count, BIP-39 checksum failure; OR non-phrase node passed to `encode --from`).

### Stderr templates

- `seedqr: decode: invalid digit count (expected 48, 60, 72, 84, or 96; got N)`
- `seedqr: decode: invalid character at position N: <char>`
- `seedqr: decode: invalid word index N at position M (must be 0..=2047)`
- `seedqr: decode: BIP-39 checksum failure: <bip39-crate-diagnostic>`
- `seedqr: encode: invalid word count: N (only 12, 15, 18, 21, or 24 supported)`
- `seedqr: encode: BIP-39 checksum failure: <bip39-crate-diagnostic>`
- `seedqr encode only accepts phrase=<value> or phrase=-`

---

## `mnemonic nostr`

Wrap an existing nostr key (`npub`/`nsec`, NIP-19 bech32 or 64-hex) as
Bitcoin addresses, descriptors, and (for `nsec`) a WIF. Taproot (`p2tr`)
is the default and the native x-only mapping for nostr keys — the
x-only pubkey is used directly as the taproot internal key, yielding a
key-path-only P2TR output. Non-taproot script types (`p2pkh`,
`p2wpkh`, `p2sh-p2wpkh`) use the BIP-340 even-y `02‖x` compressed
form of the x-only pubkey.

For `nsec` inputs, the secret is **normalized to even-y** (BIP-340): if
`d·G` has odd y, the toolkit uses `n−d` instead so the emitted WIF
controls the emitted address. A `notice:` is printed on stderr when
the normalization negates the key.

### Synopsis

```sh
mnemonic nostr (--pubkey <PUBKEY> | --secret <SECRET> | --secret-file <FILE> | --secret-stdin) [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--pubkey <PUBKEY>` | Public key: `npub1…` (NIP-19 bech32) or 64-hex x-only. Emits watch-only outputs (no WIF) |
| `--secret <SECRET>` | Secret key: `nsec1…` (NIP-19 bech32) or 64-hex scalar. Adds WIF + `electrum:` line (non-taproot script types only). SECRET — leaks via argv; use `--secret-stdin` or `--secret-file` |
| `--secret-file <SECRET_FILE>` | Read the secret key from a file (avoids argv exposure) |
| `--secret-stdin` | Read the secret key from stdin (avoids argv exposure) |
| `--script-type <SCRIPT_TYPE>` | Address/descriptor script type: `p2pkh` / `p2wpkh` / `p2sh-p2wpkh` / `p2tr`. Defaults to `p2tr` when neither this nor `--all-script-types` is given |
| `--all-script-types` | Emit descriptor + address for all four script types (`p2tr`, `p2wpkh`, `p2sh-p2wpkh`, `p2pkh`) |
| `--network <NETWORK>` | Bitcoin network — affects address HRP and WIF version byte. One of `mainnet` / `testnet` / `signet` / `regtest` (default `mainnet`) |
| `--json` | Emit JSON instead of the human-readable block |
| `--import <IMPORT>` | Append a ready-to-paste Bitcoin Core `importdescriptors` recipe for the derived address(es). `readonly` = watch-only (the pubkey descriptor). `spending` / `both` are reserved for a future cycle (rejected with a "deferred" message) |
| `--timestamp <TIMESTAMP>` | Bitcoin Core rescan anchor for `--import`: `now` or unix seconds. Default `0` (rescan from genesis to discover an existing key's funds) |
| `--help` | Print help |

### Bitcoin Core import (`--import readonly`)

With `--import readonly`, an `import:` line is appended carrying a ready-to-paste
**watch-only** `importdescriptors` recipe built from the address descriptor(s)
(`active: false`, `internal: false`, `timestamp` from `--timestamp`, default `0`).
With `--all-script-types`, one array carries all four watch-only descriptors —
paste it once to watch every address type.

```text
$ mnemonic nostr --pubkey npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg --script-type p2wpkh --import readonly
  …
  import:      importdescriptors '[{"active":false,"desc":"wpkh(02…)#csum","internal":false,"timestamp":0}]'
```

Paste the single-quoted array into Bitcoin Core: `bitcoin-cli importdescriptors '<array>'`.
Only the **public** descriptor is emitted (no private key); a *spending* recipe
(embedding the WIF) is deferred to a future cycle.

Exactly one of `--pubkey` / `--secret` / `--secret-file` / `--secret-stdin`
is required (clap arg-group; missing/multiple → exit 64).

### Secret-handling notes

- `--secret` passes the key via process arguments, which are visible in
  `/proc/$PID/cmdline` and `ps` output. The toolkit emits a warning:
  `warning: secret material on argv (--secret) — pipe via --secret-stdin
  to avoid /proc/$PID/cmdline exposure`. Prefer `--secret-stdin` or
  `--secret-file` in scripts and when a shoulder-surfing observer is a
  concern.
- WIF output is secret material. The toolkit always emits:
  `warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')`.

### The `electrum:` line

For `nsec` inputs, each non-taproot output row includes an `electrum:` line
of the form `<prefix>:<WIF>`, where `<prefix>` mirrors the Electrum import
convention (per Electrum's `WIF_SCRIPT_TYPES` in `bitcoin.py`). Taproot
(`p2tr`) has no Electrum WIF-import path — Electrum's `WIF_SCRIPT_TYPES`
has no `p2tr` entry — so no `electrum:` line is emitted for `p2tr`.

| Script type | Electrum prefix |
|---|---|
| `p2tr` | — (Electrum has no taproot private-key import) |
| `p2wpkh` | `p2wpkh:` |
| `p2sh-p2wpkh` | `p2wpkh-p2sh:` |
| `p2pkh` | `p2pkh:` |

Paste the `electrum:` value into Electrum ▸ Wallet ▸ Private Keys ▸ Import
to sweep the address into an Electrum wallet of the matching script type.

### Worked example — `npub` (watch-only, default `p2tr`)

```sh
mnemonic nostr --pubkey npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg
```

Stdout:

```text
nostr key (public)
  x-only:      7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e
  script-type: p2tr
  descriptor:  tr(7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e)#548pk2gr
  address:     bc1pvvymzaajnverlq90cqupmtwep2txzarvvwqfs4p8jfvkepqaws5scnww04
```

The same key, all four script types:

```sh
mnemonic nostr --pubkey npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg \
  --all-script-types
```

```text
nostr key (public)
  x-only:      7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e
  script-type: p2tr
  descriptor:  tr(7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e)#548pk2gr
  address:     bc1pvvymzaajnverlq90cqupmtwep2txzarvvwqfs4p8jfvkepqaws5scnww04
  script-type: p2wpkh
  descriptor:  wpkh(027e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e)#qayh3r2k
  address:     bc1qgyrepq5ukvwl7z7z5lk0066wx6vz75pn9ww6pv
  script-type: p2sh-p2wpkh
  descriptor:  sh(wpkh(027e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e))#kh0cqr4q
  address:     3546dKS2XmpDUbyQrA7zmrbE2fayRvHWyJ
  script-type: p2pkh
  descriptor:  pkh(027e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e)#9xj4dc7r
  address:     16vqz4S2bJ8F4r1rSrGU3RxkUReZYrr7X3
```

Note that `p2tr` uses the bare x-only key, while `p2wpkh`, `p2sh-p2wpkh`,
and `p2pkh` use the BIP-340 even-y `02‖x` compressed form
(`027e7e9c42…`).

### Worked example — `nsec` via stdin

```sh
echo 'nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5' \
  | mnemonic nostr --secret-stdin
```

Stdout:

```text
nostr key (secret)
  x-only:      7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e
  script-type: p2tr
  descriptor:  tr(7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e)#548pk2gr
  address:     bc1pvvymzaajnverlq90cqupmtwep2txzarvvwqfs4p8jfvkepqaws5scnww04
  wif:         Kzhcun32YwFnMsQGdJB5fyYTS84TmHb4hs4xQ6BL8ef94vvceGvP
```

Stderr:

```text
warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
```

(No argv warning because `--secret-stdin` was used.)

Note: taproot (`p2tr`) emits no `electrum:` line — Electrum has no taproot
private-key import path. Use `--script-type p2wpkh` to get the `electrum:`
hint for a SegWit address:

```sh
echo 'nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5' \
  | mnemonic nostr --secret-stdin --script-type p2wpkh
```

```text
nostr key (secret)
  x-only:      7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e
  script-type: p2wpkh
  descriptor:  wpkh(027e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e)#qayh3r2k
  address:     bc1qgyrepq5ukvwl7z7z5lk0066wx6vz75pn9ww6pv
  electrum:    p2wpkh:Kzhcun32YwFnMsQGdJB5fyYTS84TmHb4hs4xQ6BL8ef94vvceGvP
  wif:         Kzhcun32YwFnMsQGdJB5fyYTS84TmHb4hs4xQ6BL8ef94vvceGvP
```

### Worked example — even-y normalization notice

When the raw nostr secret has odd y, the toolkit negates the scalar and
prints a notice:

```sh
mnemonic nostr --secret-stdin <<< '0000000000000000000000000000000000000000000000000000000000000006'
```

Stderr (in addition to the output-class advisory):

```text
notice: nostr: secret normalized to even-y (BIP-340) for address consistency
```

The emitted WIF and address correspond to the normalized (even-y) key,
not the raw scalar, so `WIF → address` is always self-consistent.

### JSON output (`--json`)

`--json` emits a single JSON object on stdout:

```sh
mnemonic nostr --pubkey npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg \
  --json
```

```json
{
  "kind": "public",
  "x_only": "7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e",
  "outputs": [
    {
      "script_type": "p2tr",
      "descriptor": "tr(7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e)#548pk2gr",
      "address": "bc1pvvymzaajnverlq90cqupmtwep2txzarvvwqfs4p8jfvkepqaws5scnww04"
    }
  ]
}
```

For `nsec` inputs the object additionally carries `"wif": "<WIF>"` at the
top level and each non-taproot `outputs` entry includes
`"electrum": "<prefix>:<WIF>"` (taproot entries omit the `"electrum"` key).

---

## `mnemonic silent-payment` {#mnemonic-silent-payment}

Derive a [BIP-352](https://github.com/bitcoin/bips/blob/master/bip-0352.mediawiki) **Silent Payments** *receiver* static address from a seed-bearing secret. A silent payment address (`sp1…` mainnet, `tsp1…` testnet/signet/regtest) is published once; senders derive a unique on-chain output for each payment with no on-chain link and no sender↔receiver interaction.

```text
mnemonic silent-payment --secret <SEED> [OPTIONS]
mnemonic silent-payment --secret-stdin [OPTIONS]
```

The scan key is derived at `m/352'/<coin>'/<account>'/1'/0` and the spend key at `m/352'/<coin>'/<account>'/0'/0`; the base (unlabeled) address encodes the compressed pubkeys `B_scan ‖ B_spend`. A labeled address (`--label <m>`, m≥1) encodes `B_scan ‖ B_m` where `B_m = B_spend + hash_BIP0352/Label(b_scan ‖ m)·G`.

### Flags

| Flag | Purpose |
|---|---|
| `--secret <SEED>` | seed-bearing secret: BIP-39 phrase / ms1 / entropy-hex / master xprv. A single private key (WIF/minikey) is refused — it cannot derive `m/352'`. SECRET: leaks via argv; prefer `--secret-file` / `--secret-stdin` |
| `--secret-file <PATH>` | read the seed-bearing secret from a file (avoids argv exposure) |
| `--secret-stdin` | read the seed-bearing secret from stdin |
| `--passphrase <P>` | BIP-39 mnemonic-extension passphrase ("25th word"). Applies to phrase / ms1 / entropy-hex inputs; **ignored (with a warning) for an xprv input** (the xprv is already the master). SECRET: leaks via argv; prefer `--passphrase-stdin` |
| `--passphrase-stdin` | read the BIP-39 passphrase from stdin (whitespace-preserving — significant PBKDF2 salt). Mutually exclusive with `--passphrase`, and with `--secret-stdin` (one stdin per invocation) |
| `--network <mainnet\|testnet\|signet\|regtest>` | mainnet → `sp` address + coin-type 0; testnet/signet/regtest → `tsp` address + coin-type 1 (default mainnet) |
| `--account <N>` | BIP-32 account index `m/352'/coin'/<account>'/…` (default 0) |
| `--label <m>` | emit a labeled address for label m (repeatable); **m≥1**. `--label 0` is refused — m=0 is the reserved BIP-352 change label and must never be published |
| `--change-address` | also emit the BIP-352 **m=0 change address** — for the wallet's OWN change detection ONLY; **never hand it out as a receiving address** (additive; the base address is still emitted) |
| `--json` | emit a JSON envelope instead of the human-readable block |
| `--help` | print help |

### Output

The address(es) and the scan/spend **public** keys are publishable — hand the base address to senders. The command also emits the **scan private key** (`b_scan`, the *online / hot* key a watch-server uses to scan) and the **spend private key** (`b_spend`, the *COLD* key with full spending authority) behind the `warning: stdout carries private key material` advisory (the secret is `mlock`-pinned + zeroized). Treat them differently: never paste `b_spend` into a scanning service.

A BIP-39 `--passphrase` derives the address for the *passphrase-protected* wallet (a different wallet than the no-passphrase default); whitespace in the passphrase is significant. `--change-address` adds the m=0 change address (with a `change_address_warning` in the JSON envelope) — it is the receiver's own change-detection address and must never be published; `--label 0` remains refused as the separate publish-path guard.

### Scope

This derives the **receiver** address only. **Sender** output construction (which needs the sender's input private keys + ECDH) and **chain scanning** (which needs blockchain data) are out of scope — `mnemonic` has no transaction inputs, no chain access, and does not sign.

---

## `mnemonic addresses` {#mnemonic-addresses}

List a wallet's receive/change addresses (batch). The watch-only complement to `export-wallet --range` and the multi-address sibling of `convert --to address`. Read-only public derivation — **no private keys reach stdout, and `mnemonic` never signs.**

```text
mnemonic addresses --from <SOURCE> --address-type <T> [--account <N>] \
                   [--count <N> | --range <A,B>] [--chain <receive|change|both>] \
                   [--network <NET>] [--passphrase <V> | --passphrase-stdin] [--language <L>] [--json]
```

`--from` accepts an account `xpub=` (derived directly) or a seed source (`phrase=` / `entropy=` / `seedqr=`). For a seed source, `--address-type` selects the BIP-44/49/84/86 account path (`p2pkh`→44', `p2sh-p2wpkh`→49', `p2wpkh`→84', `p2tr`→86') at `m/<purpose>'/<coin>'/<account>'`, and the addresses are `m/<chain>/<index>` under it. For an `xpub=` source the xpub *is* the account key, so `--account` / `--passphrase` do not apply (supplying them is an error). Secret values support `@env:VAR` and `-` (stdin).

### Flags

| Flag | Purpose |
|---|---|
| `--from <SOURCE>` | `xpub=<v>` \| `phrase=<v>` \| `entropy=<hex>` \| `seedqr=<digits>`; `@env:VAR` / `-` (stdin) for secret values |
| `--address-type <T>` | `p2pkh` \| `p2sh-p2wpkh` \| `p2wpkh` \| `p2tr` (required; selects the account path for seed sources and the render type) |
| `--account <N>` | account index for seed sources (default 0; not applicable to `xpub=`) |
| `--count <N>` | number of addresses per chain, from index 0 (default 10); conflicts with `--range` |
| `--range <A,B>` | inclusive index range `A..=B`; conflicts with `--count` |
| `--chain <receive\|change\|both>` | which chain(s) to list (default `receive`) |
| `--network <NET>` | `mainnet` \| `testnet` \| `signet` \| `regtest`; defaults to the xpub's version bytes (xpub source) or mainnet (seed source); must agree with an xpub's network kind |
| `--passphrase <V>` | BIP-39 passphrase (seed sources); `@env:VAR` supported |
| `--passphrase-stdin` | read the BIP-39 passphrase from stdin (conflicts with `--passphrase`) |
| `--language <L>` | BIP-39 wordlist language for `phrase=`/`seedqr=` (default `english`) |
| `--json` | emit a JSON envelope instead of the text rows |
| `--help` | print help |

`--count`/`--range` indices are bounded by the BIP-32 normal-index ceiling (`< 2^31`); an out-of-range request is rejected (never a panic).

### Worked example

```text
mnemonic addresses --from xpub=xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a \
  --address-type p2wpkh --count 3
  0  bc1qfjxgzvdwrxh9ejp6jmdlr9tc6lfl6adcsx2z4f
  1  bc1q399ww2924rlr6xn7j0fysjxnfjuy4p2v4769p3
  2  bc1qtmra2ejp52grx486fr0nzndy8g7t4ee3amdht0
```

### Output

Text mode prints two-space-indented `<index>  <address>` rows; with `--chain both` rows are grouped by a `receive (m/0/i):` / `change (m/1/i):` header. JSON mode emits `{ "schema_version": "1", "source", "address_type", "network", "account"?, "addresses": [ { "chain", "index", "address" }, … ] }` (`account` is present only for seed sources). Because the addresses are derived keys, the non-English wordlist advisory does **not** fire here (the language is already baked into the derivation).

---

## `mnemonic decode-address` {#mnemonic-decode-address}

Decode a Bitcoin address string into its facts: the network(s) it is valid for, script type, witness version, and scriptPubKey. Public-data utility — no secrets, no key material; the inverse of `convert --to address`.

```text
mnemonic decode-address <ADDRESS> [--json]
```

The address layer cannot disambiguate testnet / testnet4 / signet (shared `tb1` and base58 prefixes), so `networks` reports the full set the address is valid for; `regtest` (`bcrt1`) is distinct.

### Flags

| Flag | Purpose |
|---|---|
| `<ADDRESS>` | the address to decode (positional); P2PKH / P2SH / P2WPKH / P2WSH / P2TR, any network |
| `--json` | emit a JSON envelope instead of the human-readable block |
| `--help` | print help |

### Output

`networks` (the valid-for set), `script_type` (`p2pkh`/`p2sh`/`p2wpkh`/`p2wsh`/`p2tr`), `witness_version` (segwit only; absent for legacy), and `script_pubkey` (hex). An unparseable address exits non-zero.

---

## `mnemonic verify-message` {#mnemonic-verify-message}

**Verify** a Bitcoin message signature (verification only — `mnemonic` never signs). Two formats are supported and partition cleanly by address type:

- **legacy** "Bitcoin Signed Message" (the `signmessage`/`verifymessage` format) — **P2PKH only**.
- **[BIP-322](https://github.com/bitcoin/bips/blob/master/bip-0322.mediawiki) simple** — **P2WPKH / P2SH-P2WPKH / P2TR**.

```text
mnemonic verify-message --address <ADDR> --message <MSG> --signature <B64> [--format <auto|legacy|bip322>]
mnemonic verify-message --address <ADDR> --message-stdin --signature <B64>
```

With `--format auto` (default) the format is chosen by address type: P2PKH → legacy, segwit/taproot → BIP-322. `--format legacy` on a non-P2PKH address is refused (legacy verification is P2PKH-only).

### Flags

| Flag | Purpose |
|---|---|
| `--address <ADDR>` | the address the message was signed by |
| `--message <MSG>` | the signed message, inline (exact bytes) |
| `--message-file <PATH>` | read the message from a file (a single trailing newline is stripped) |
| `--message-stdin` | read the message from stdin (a single trailing newline is stripped) |
| `--signature <B64>` | the signature (base64): a 65-byte recoverable sig (legacy) or a BIP-322 witness encoding |
| `--format <auto\|legacy\|bip322>` | signature format (default `auto` — legacy for P2PKH, BIP-322 otherwise) |
| `--json` | emit a JSON envelope instead of the human-readable line |
| `--help` | print help |

Exactly one of `--message` / `--message-file` / `--message-stdin` is required.

### Exit codes

A **valid** signature exits 0. A cleanly-decoded signature that simply does **not** verify exits 1 with the structured `valid: false` result on stdout (no error on stderr). Malformed input — a bad address, an undecodable signature, or `--format legacy` on a non-P2PKH address — exits 1 with an error on stderr.

### Scope

Verification only. Signing is out of scope. Taproot **script-path** and arbitrary-script (BIP-322 *full*) signatures are not yet covered (BIP-322 *simple* only).

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
| `--max-indel <N>` | search up to N (0–4, default 0) insert/delete edits to recover a chunk that failed normal repair — a single character added (too long) or dropped (too short) during transcription; ms1/mk1/md1 |
| `--max-subst <E>` | also tolerate up to E (0–4, default 0) substitution errors alongside the indels; a recovery that used a substitution is printed as a VERIFY-ME candidate (exit 4), not a confident correction |
| `--help` | print help |

### Exit codes

| Code | Meaning |
|---|---|
| `0` | all chunks already valid (no repair applied; input echoed to stdout unchanged) |
| `5` | at least one chunk corrected (`REPAIR_APPLIED`), incl. a unique `--max-indel` recovery; stdout = repair report + corrected chunks |
| `4` | ambiguous (multiple candidates) **or a candidate required ≥1 substitution** — verify each before trusting; all candidates are printed |
| `2` | unrepairable (per-chunk `RepairError`; e.g. `TooManyErrors`, `HrpMismatch`, `ReservedInvalidLength`, `UnsupportedCodeVariant`, or `--max-indel` exhausted without a recovery) |
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
warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
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
| Corrected `ms1` emitted to stdout | `warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '\| age -e ...')` |
| Repair fired and emitted ≥ 1 correction | `repair: applied K correction(s) across J chunk(s)` |

### Recovering an incorrect-length card (`--max-indel`) {#mnemonic-repair-max-indel}

`mnemonic repair` corrects substitution errors at a FIXED length. When a
character was inserted or dropped during hand-copy (so the string is the
wrong length and no longer decodes), pass `--max-indel <N>` (1–4) to also
search for that indel. The search covers the data-part (delete-and-validate
for too-long; BCH-solve the omitted symbol for too-short) and the `ms1`/`mk1`
prefix; it also considers indels split across **both** the prefix and the
data-part simultaneously (tagged `cross-region`), within the `--max-indel`
budget. Outcomes: a unique recovery prints the corrected string (exit 5, like
any repair); multiple equally-valid candidates print all of them (exit 4 —
choose manually); none within the budget exits 2. `ms1` candidates are secret
material (the usual stderr advisory applies). `md1` (chunked) recovers
per-chunk like mk1, with cross-chunk reassembly validation. Default `0`
disables the search (behavior unchanged).

### Recovering an indel that also has a wrong character (`--max-subst`) {#mnemonic-repair-max-subst}

`--max-subst <E>` (default 0) widens the indel search to also accept
candidates that have up to E **substitution** (wrong-but-in-place)
errors alongside the indel. A substitution is a position whose
corrected symbol differs from the original but is NOT one of the
inserted placeholder positions — so it required an additional BCH
correction beyond the indel itself. The shared BCH budget is
`placeholders + substitutions ≤ 4` (the `t = 4` singleton bound),
meaning `--max-indel` and `--max-subst` draw from the same pool.

Candidates that needed a substitution are printed as **VERIFY-ME**
candidates (exit 4 — same as ambiguous), NOT as confident corrections
(exit 5). This is intentional: the BCH code cannot distinguish a
genuine indel+substitution from a longer-distance all-substitution
error at the same budget; the user should verify the recovered string
against independent notes before trusting it. `--max-subst` has no
effect without `--max-indel ≥ 1` (a notice is printed to stderr if
only `--max-subst` is passed).

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

When `--max-indel ≥ 1` triggers the indel engine and produces a result,
the `--json` envelope instead has the shape:

```json
{
  "schema_version": "1",
  "status": "unique",
  "confident": true,
  "candidates": [
    {
      "recovered": "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
      "indel_count": 1,
      "subst_count": 0,
      "region": "data-part",
      "direction": "deleted"
    }
  ]
}
```

`status` is `"unique"` (one candidate, exit 5) or `"ambiguous"` (multiple,
exit 4). `confident` is `true` iff every candidate has `subst_count == 0`
(pure-indel recovery — no substitution was needed); `false` when any
candidate required a substitution (exit 4, VERIFY-ME advisory). `region` is
`"data-part"`, `"prefix"`, or `"cross-region"` (the indel spanned both the
prefix and the data-part). `direction` is `"deleted"` (removed an added
char — too-long input) or `"inserted"` (restored a dropped char — too-short
input). `subst_count` is the number of substitution corrections beyond the
indel placeholders that the BCH decoder applied for that candidate (0 for a
pure-indel recovery). The indel envelope is NOT emitted for the
`Unrecoverable` outcome — that surfaces via the normal error path (exit 2, no
JSON on stdout).

---

## `mnemonic inspect`

Describe the contents of an m-format card without performing any
conversion. Per kind:

- `ms1` — tag (`entr` for classic entropy-only ms1; `mnem` for
  language-tagged ms1 produced by ms-cli v0.2+ for non-English
  phrases), payload kind, byte length, bit strength (= 8 × bytes).
  Entropy hex is suppressed by default (sensitive material); pass
  `--reveal-secret` to print it. `mnem`-kind cards also report the
  stored wordlist language (e.g. `language: japanese`).
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
warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')
```

### JSON output (v0.27.0)

When `--json` is supplied, `inspect` emits a single JSON envelope on
stdout instead of the text-form report. The envelope carries a
top-level `schema_version: "1"` field (v0.27.0 backfill via the new
`InspectEnvelope` wrapper) followed by the kind-specific fields:

```json
{
  "schema_version": "1",
  "kind": "ms1",
  "tag": "entr",
  "payload_kind": "Entr",
  "byte_length": 16,
  "bit_strength": 128,
  "entropy_hex": null
}
```

`schema_version` is currently pinned at `"1"`; future format changes
will bump the version with explicit migration notes in the SPEC. The
same convention applies to `mnemonic repair`'s JSON output (which has
shipped `schema_version: "1"` since v0.22.0).

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
| Any `ms1` inspection (regardless of `--reveal-secret`) | `warning: stdout carries private key material (can spend) — redirect or encrypt ...` |

## `mnemonic xpub-search` (v0.26.0) {#mnemonic-xpub-search}

Umbrella subcommand for **reverse searches over a BIP-32 derivation graph** — given a seed (or xpub), find which derivation produces a target xpub / descriptor / address / passphrase. v0.26.0 ships four modes:

- **`path-of-xpub`** — given seed + target xpub (or mk1 card), find the BIP-32 path under the seed that produces it.
- **`account-of-descriptor`** — given seed + descriptor, find the cosigner role + account index.
- **`address-of-xpub`** — given xpub + address, scan child indices to a gap limit.
- **`passphrase-of-xpub`** — given seed + passphrase + target xpub, verify the passphrase produces the xpub at a standard path.

### `mnemonic xpub-search path-of-xpub`

Given a seed (BIP-39 phrase OR ms1 card) and a target xpub (or mk1 card carrying an xpub), search the standard derivation templates (BIP-44 / BIP-49 / BIP-84 / BIP-86 single-sig + BIP-48 multisig at `script_type ∈ {1', 2', 3'}`) × account range, returning the matching path on first hit. `--add-path <TEMPLATE>` extends the candidate set.

#### Synopsis

```sh
mnemonic xpub-search path-of-xpub \
    {--phrase <BIP39> | --phrase-stdin | --ms1 <MS1> | --ms1-stdin | <positional MS1>} \
    [--passphrase <P> | --passphrase-stdin] \
    --target-xpub <XPUB-OR-MK1> \
    [--language <LANG>] [--network <NET>] \
    [--min-account 0] [--number-of-accounts 20] [--max-account <N>] \
    [--add-path <TEMPLATE>]... \
    [--json]
```

#### Flags

| Flag | Purpose |
|---|---|
| `--phrase <PHRASE>` | master BIP-39 phrase (inline); emits argv-leakage advisory; prefer `--phrase-stdin` |
| `--phrase-stdin` | read master BIP-39 phrase from stdin |
| `--ms1 <MS1>` | ms1 card carrying BIP-39 entropy (inline); emits argv-leakage advisory |
| `--ms1-stdin` | read ms1 card from stdin (single chunk) |
| `<positional MS1>` | positional ms1 card (HRP-autodetect). BIP-39 phrase text is NOT accepted positionally (no HRP for autodetect) |
| `--passphrase <P>` | BIP-39 passphrase (inline); emits argv-leakage advisory |
| `--passphrase-stdin` | read BIP-39 passphrase from stdin (NULL-byte-preserving; single trailing newline stripped) |
| `--target-xpub <XPUB-OR-MK1>` | target xpub (any SLIP-0132 prefix: `xpub`/`tpub`/`ypub`/`Ypub`/`zpub`/`Zpub`/`upub`/`Upub`/`vpub`/`Vpub`) OR an `mk1...` bech32 card carrying an xpub |
| `--language <LANGUAGE>` | BIP-39 wordlist (default `english`; same options as `seed-xor`) |
| `--network <NETWORK>` | network selector: `mainnet` (default) / `testnet` / `signet` / `regtest` |
| `--min-account <N>` | lower bound of account-index iteration, inclusive (default `0`) |
| `--number-of-accounts <N>` | window size starting at `--min-account` (default `20`) |
| `--max-account <N>` | optional upper bound; effective end is `max(min_account + number_of_accounts, max_account + 1)` |
| `--add-path <TEMPLATE>` | additional derivation-path template (repeatable). Literal token `account'` (or `account`) substituted with each iterated account index. Templates without an `account` token are searched once at the literal path. Multi-occurrence within one template requires multiple `--add-path` flags |
| `--json` | emit JSON envelope on stdout instead of text-form |
| `--no-auto-repair` | (global) skip BCH auto-fire on `--ms1` decode failure; preserve typed decode error exit |
| `-h, --help` | print help |

Seed-intake mutex: exactly one of `{--phrase, --phrase-stdin, --ms1, --ms1-stdin, positional}` is required. Auto-fire BCH repair applies ONLY to the `--ms1` decode-failure path (BIP-39 phrase parse failure routes direct exit 1 — phrases have no BCH primitive).

#### Worked example

```sh
# Test BIP-39 phrase (12-word vector from BIP-39 spec)
PHRASE="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

# Derive the BIP-84 account-0 xpub for this seed (via mnemonic bundle or external tool); call it ZPUB.

# Find the path under the seed that produces ZPUB:
mnemonic xpub-search path-of-xpub --phrase "$PHRASE" --target-xpub "$ZPUB"
```

Stdout (text form):

```text
match: m/84'/0'/0'  (template=bip84, account=0)
target-xpub: xpub6... (normalized from zpub; variant=zpub)
searched: 7 templates × 20 accounts = 140 paths
```

#### JSON output

`--json` emits a versioned envelope. Schema `v1`. Match shape:

```json
{
  "schema_version": "1",
  "mode": "path-of-xpub",
  "result": "match",
  "path": "m/84'/0'/0'",
  "template": "bip84",
  "account": 0,
  "target_xpub_canonical": "xpub6...",
  "target_xpub_variant": "zpub",
  "searched_count": 140
}
```

No-match shape:

```json
{
  "schema_version": "1",
  "mode": "path-of-xpub",
  "result": "no_match",
  "target_xpub_canonical": "xpub6...",
  "target_xpub_variant": "zpub",
  "searched_count": 140
}
```

`target_xpub_variant` serializes as `null` when the target was supplied in canonical xpub/tpub form (no SLIP-0132 alt-prefix swap occurred). The field is always emitted (not skipped) to keep the JSON envelope structurally stable across runs.

**Envelope tag deviation:** `xpub-search` uses `tag = "mode"` (not the project's `tag = "kind"` used by `InspectJson` / `RepairJson`). Rationale: `mode` is the natural domain term for `xpub-search`'s four sub-modes; `kind` would conflict with `RepairJson`'s `kind: "ms1"|"mk1"|"md1"` per-card-type semantic.

#### Exit codes

| Code | Meaning |
|---|---|
| 0 | Match found |
| 1 | Bad input (BIP-39 parse failure, xpub parse failure, mk1 decode failure outside the auto-fire path, ms1 decode failure with `--no-auto-repair` or on no-TTY) |
| 4 | No match in searched set (`ToolkitError::XpubSearchNoMatch`) |
| 5 | Auto-fire BCH short-circuit on `--ms1` decode failure (TTY-gated; same contract as `convert` / `inspect` / `verify-bundle`) |
| 64 | Clap arg-parse error |

#### Refusals

| Trigger | Refusal |
|---|---|
| Positional argument with no `ms1` HRP (e.g., a BIP-39 phrase typed positionally) | `BIP-39 phrase must be supplied via --phrase or --phrase-stdin (no HRP for positional autodetect)` |
| Multiple seed-intake flags supplied (`--phrase` AND `--ms1`, etc.) | clap mutex error |
| Invalid SLIP-0132 prefix on `--target-xpub` | xpub parse error (exit 1) |

#### Advisories

| Trigger | Stderr advisory |
|---|---|
| Inline `--phrase <v>` | `warning: secret material on argv (--phrase) — pipe via --phrase-stdin to avoid /proc/$PID/cmdline exposure` |
| Inline `--ms1 <v>` | `warning: secret material on argv (--ms1) — pipe via --ms1-stdin to avoid /proc/$PID/cmdline exposure` |
| Inline `--passphrase <v>` | `warning: secret material on argv (--passphrase) — pipe via --passphrase-stdin to avoid /proc/$PID/cmdline exposure` |

#### Candidate path set

The default candidate set is the cross-product of:

- **Templates:** BIP-44 / BIP-49 / BIP-84 / BIP-86 (single-sig) + BIP-48 at `script_type ∈ {1', 2', 3'}` (sh-wsh / wsh / tr-multi-a multisig) — seven templates, fixed order
- **Accounts:** half-open range `[min_account, max(min_account + number_of_accounts, max_account + 1))`
- **Add-paths:** each `--add-path <TEMPLATE>` iterated over the same account range (or once if the template contains no `account` token)

Iteration is deterministic: templates in fixed lexical order, accounts ascending, add-paths in user-supplied order. First match wins. The matching template name is one of `bip44` / `bip49` / `bip84` / `bip86` / `bip48-sh-wsh` / `bip48-wsh` / `bip48-tr-multi-a` for standard templates, or the literal user-supplied template string (e.g. `m/87'/0'/account'`) for `--add-path` entries. The `account` field is `null` when the matched template carries no `account` token (e.g., a fully-literal `--add-path m/9999'/0'/0'`).

### `mnemonic xpub-search account-of-descriptor`

Given a seed (BIP-39 phrase OR ms1 card) + a wallet descriptor, identify which cosigner role(s) the seed plays in the descriptor and at which account index. Searches the same candidate-path set as `path-of-xpub`, run once per cosigner.

#### Synopsis

```sh
mnemonic xpub-search account-of-descriptor \
    {--phrase <BIP39> | --phrase-stdin | --ms1 <MS1> | --ms1-stdin | <positional MS1>} \
    [--passphrase <P> | --passphrase-stdin] \
    {--descriptor <VALUE> | --descriptor-from <NODE>=<VALUE>} \
    [--language <LANG>] [--network <NET>] \
    [--min-account 0] [--number-of-accounts 20] [--max-account <N>] \
    [--add-path <TEMPLATE>]... \
    [--json]
```

#### Descriptor input shapes (auto-detect tie-break order)

| Shape | Detection rule | Source |
|---|---|---|
| BIP-388 wallet-policy JSON | input (after `trim_start`) begins with `{` | reversed via `wallet_export/pipeline.rs:160-205` emitter; substitution rule `@N/**` → `keys_info[N] + "/<0;1>/*"` |
| md1 card(s) | input begins with `md1` HRP (single inline) OR `--descriptor-from md1=-` stdin (one chunk per line) | `md_codec::chunk::reassemble` tree-walk on `desc.tlv` xpub material (pubkeys + fingerprints + origin-path overrides) + `desc.path_decl.paths` |
| Toolkit `@N`-placeholder descriptor | regex `@\d+` outside string-literal context | REFUSED (synthetic xpubs are non-searchable; supply a literal-xpub descriptor / md1 card / BIP-388 JSON instead) |
| External literal-xpub descriptor | else | `rust_miniscript::Descriptor::<DescriptorPublicKey>::from_str` + `iter_pk()` walk (precedent `wallet_export/pipeline.rs:177`) |

Explicit override via `--descriptor-from <node>=<value>` where `<node>` is `literal` / `md1` / `bip388`; `<value>` is a literal string or `-` for stdin.

#### Flags

| Flag | Purpose |
|---|---|
| `--phrase` / `--phrase-stdin` / `--ms1` / `--ms1-stdin` / `<positional MS1>` | seed-intake mutex (same as `path-of-xpub`) |
| `--passphrase` / `--passphrase-stdin` | optional BIP-39 passphrase |
| `--descriptor <VALUE>` | wallet descriptor; shape auto-detected per tie-break order |
| `--descriptor-from <NODE>=<VALUE>` | explicit shape override (`literal=` / `md1=` / `bip388=`; `-` for stdin) |
| `--language` / `--network` | BIP-39 wordlist + network selector (same defaults as `path-of-xpub`) |
| `--min-account` / `--number-of-accounts` / `--max-account` / `--add-path` | candidate-set range (same as `path-of-xpub`; search runs once per cosigner) |
| `--json` | emit JSON envelope on stdout |
| `--no-auto-repair` | (global) skip BCH auto-fire on `--ms1` decode failure |
| `-h, --help` | print help |

#### v0.19.0 silent-default-path inference

Literal-xpub descriptors with missing `[fp/path]` annotations on `@N` cosigners trigger silent BIP-48 default path (`m/48'/<coin>'/<account>'/2'`) + a stderr `info:` notice mirroring `mnemonic bundle` v0.19.0 behavior. Override per-placeholder via inline `[fp/path]xpub.../<...>/*` in the descriptor.

#### NUMS sentinel

A cosigner xpub matching the BIP-341 unspendable internal-key NUMS H point (`50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`) is skipped — search does not run for that cosigner — and reported in the JSON envelope's `unspendable_internal_keys` array.

#### Output (text, multisig match)

```text
match: cosigner @0  m/48'/0'/0'/2'  (template=bip48-wsh, account=0)
descriptor: wsh(sortedmulti(2, [fp1/48h/0h/0h/2h]xpub1.../0/*, ...))
cosigners total: 3
matched cosigner indices: [0]
searched: 7 templates × 20 accounts × 3 cosigners = 420 paths
```

#### Output (`--json`)

```json
{
  "schema_version": "1",
  "mode": "account-of-descriptor",
  "result": "match",
  "matched_cosigners": [
    {"cosigner_index": 0, "path": "m/48'/0'/0'/2'", "template": "bip48-wsh", "account": 0}
  ],
  "cosigners_total": 3,
  "searched_count_per_cosigner": 140,
  "descriptor_shape": "literal_xpub",
  "unspendable_internal_keys": []
}
```

#### Exit codes

| Code | Meaning |
|---|---|
| 0 | At least one cosigner matched |
| 1 | Bad input (descriptor parse error, toolkit-@N refusal, no-xpub-keys refusal, seed-intake error) |
| 4 | No cosigner matched (`ToolkitError::XpubSearchNoMatch`) |
| 5 | Auto-fire BCH short-circuit on `--ms1` decode failure |
| 64 | Clap arg-parse error |

#### Refusals

| Trigger | Refusal |
|---|---|
| Toolkit `@N`-placeholder descriptor (e.g. `wsh(sortedmulti(2, @0[fp/...], @1[fp/...]))`) | `toolkit @N descriptors carry synthetic xpubs; supply a literal-xpub descriptor, md1 card, or BIP-388 wallet-policy JSON instead` |
| Descriptor containing no extended keys (all raw public keys) | `descriptor contains no extended keys; xpub-search requires xpub-shaped cosigners` |
| Bare `tr(...)` with no key form | rust-miniscript parse error (exit 1) |
| `--descriptor-from <unknown>=...` | `--descriptor-from: <node> must be one of literal / md1 / bip388` |

### `mnemonic xpub-search address-of-xpub`

Given a parent xpub (or an mk1 card carrying an xpub) plus one or more target addresses, scan child receive (`chain=0`) and change (`chain=1`) addresses across the gap-limit window and report which targets matched at which `(chain, index)`. Takes **no seed material** — auto-fire BCH repair does not apply, and there is no argv-leakage surface beyond the (non-secret) xpub itself.

The script-type used to render each child address comes from the xpub's SLIP-0132 prefix where unambiguous (`ypub`/`upub` → P2SH-P2WPKH; `zpub`/`vpub` → P2WPKH); for neutral `xpub`/`tpub` (and any override), supply `--address-type` explicitly. Multisig SLIP-0132 prefixes (`Ypub`/`Zpub`/`Upub`/`Vpub`) are refused — use `account-of-descriptor` instead, since single-sig address derivation from a multisig cosigner xpub is semantically wrong.

#### Synopsis

```sh
mnemonic xpub-search address-of-xpub \
    {--xpub <XPUB-OR-MK1> | --xpub-stdin} \
    --target-address <ADDR> [--target-address <ADDR>]... \
    [--gap-limit 20] \
    [--external-only] \
    [--address-type <p2pkh|p2sh-p2wpkh|p2wpkh|p2tr>] \
    [--network <NET>] \
    [--json]
```

#### Flags

| Flag | Purpose |
|---|---|
| `--xpub <XPUB-OR-MK1>` | parent xpub (any SLIP-0132 single-sig prefix: `xpub`/`tpub`/`ypub`/`upub`/`zpub`/`vpub`) OR an `mk1...` bech32 card carrying an xpub. Multisig prefixes (`Ypub`/`Zpub`/`Upub`/`Vpub`) refused |
| `--xpub-stdin` | read parent xpub from stdin (single line, trailing newline stripped); mutex with `--xpub` |
| `--target-address <ADDR>` | target address to search for; repeatable; at least one required |
| `--gap-limit <N>` | per-chain scan window, indices `0..N`. Default `20` |
| `--external-only` | restrict scan to the external (receive) chain; skip change chain. Default scans both |
| `--address-type <TYPE>` | explicit script-type for child-address rendering (`p2pkh` / `p2sh-p2wpkh` / `p2wpkh` / `p2tr`). Required for neutral `xpub`/`tpub`; overrides prefix-inferred type otherwise |
| `--network <NET>` | network selector: `mainnet` / `testnet` / `signet` / `regtest`. Default inferred from the xpub version byte; `--network signet`/`--network regtest` overrides the test/signet/regtest ambiguity collapsed by the version byte |
| `--json` | emit JSON envelope on stdout instead of text-form report |
| `-h, --help` | print help |

#### Worked example

```sh
# Take an externally-supplied account-level zpub and an address you suspect
# was derived from it. Confirm by index:
ZPUB="zpub6r..."           # account-0 zpub from a BIP-84 wallet
ADDR="bc1q..."             # candidate child address

mnemonic xpub-search address-of-xpub \
    --xpub "$ZPUB" \
    --target-address "$ADDR"
```

Stdout (text form, match):

```text
match: bc1q... → 0/5  (script_type=p2wpkh, chain=external, index=5)
targets: 1; matched: 1; unmatched: 0
```

Stdout (text form, no match):

```text
no match: bc1q... (searched 0/0..19 + 1/0..19)
targets: 1; matched: 0; unmatched: 1
```

The summary line reports total / matched / unmatched counts after all per-target lines.

#### JSON output

`--json` emits a versioned envelope. Schema `v1`. The `results` array carries one entry per `--target-address` in user-supplied order. Mixed match / no-match payloads are supported; the envelope shape stays stable.

Match entry:

```json
{
  "schema_version": "1",
  "mode": "address-of-xpub",
  "results": [
    {"target": "bc1q...", "result": "match", "chain": "external", "index": 5, "script_type": "p2wpkh"}
  ],
  "xpub_canonical": "xpub6...",
  "xpub_variant": "zpub",
  "gap_limit": 20
}
```

No-match entry (single target):

```json
{
  "schema_version": "1",
  "mode": "address-of-xpub",
  "results": [
    {"target": "bc1q...", "result": "no_match", "scanned_external": 20, "scanned_internal": 20}
  ],
  "xpub_canonical": "xpub6...",
  "xpub_variant": "zpub",
  "gap_limit": 20
}
```

`xpub_variant` serializes as `null` when the input was already-canonical `xpub`/`tpub` or an mk1 card (no SLIP-0132 alt-prefix swap occurred). When `--external-only` is supplied, `scanned_internal` is `0` for no-match entries.

#### Exit codes

| Code | Meaning |
|---|---|
| 0 | All targets matched |
| 1 | Bad input (xpub parse failure, address parse failure, multisig SLIP-0132 prefix, missing `--address-type` for neutral xpub) |
| 4 | At least one target unmatched (`ToolkitError::XpubSearchNoMatch` with `mode: "address-of-xpub"`) |
| 64 | Clap arg-parse error |

P3 takes no secret material; auto-fire BCH repair (exit 5) does not apply.

#### Refusals

| Trigger | Refusal |
|---|---|
| Multisig SLIP-0132 prefix on `--xpub` (`Ypub` / `Zpub` / `Upub` / `Vpub`) | `address-of-xpub is single-sig only; the <Ypub\|Zpub\|Upub\|Vpub> prefix is a multisig SLIP-0132 variant. Multisig address derivation requires the full descriptor — use xpub-search account-of-descriptor to find the matching account.` |
| Neutral `xpub`/`tpub` with no `--address-type` | `xpub has no SLIP-0132 single-sig prefix signal — supply --address-type <p2pkh\|p2sh-p2wpkh\|p2wpkh\|p2tr>.` |
| Both `--xpub` and `--xpub-stdin` supplied | clap mutex error |
| Neither `--xpub` nor `--xpub-stdin` supplied | `supply --xpub <VALUE> or --xpub-stdin` |
| No `--target-address` supplied | clap `required` error |

### `mnemonic xpub-search passphrase-of-xpub`

Given a seed (BIP-39 phrase OR ms1 card) **plus a specific passphrase** + a target xpub (or mk1 card carrying an xpub), verify that this passphrase produces the xpub under the seed at one of the standard derivation templates (BIP-44 / BIP-49 / BIP-84 / BIP-86 single-sig + BIP-48 multisig at `script_type ∈ {1', 2', 3'}`) × account range. Same candidate-set + first-match-wins primitive as `path-of-xpub`; the semantic difference is that this mode answers **"does THIS passphrase produce the xpub?"** rather than **"what path produced this xpub?"**.

The passphrase group is **mandatory**: exactly one of `--passphrase` / `--passphrase-stdin` must be supplied. Omitting both is a clap arg-parse error (exit 64). MVP scope is single-passphrase verification only; file-based / streamed / generated candidate sets are deferred to v0.27+ via FOLLOWUP `xpub-search-passphrase-bruteforce`.

#### Synopsis

```sh
mnemonic xpub-search passphrase-of-xpub \
    {--phrase <BIP39> | --phrase-stdin | --ms1 <MS1> | --ms1-stdin | <positional MS1>} \
    {--passphrase <P> | --passphrase-stdin} \
    --target-xpub <XPUB-OR-MK1> \
    [--language <LANG>] [--network <NET>] \
    [--min-account 0] [--number-of-accounts 20] [--max-account <N>] \
    [--add-path <TEMPLATE>]... \
    [--json]
```

#### Flags

| Flag | Purpose |
|---|---|
| `--phrase <PHRASE>` | master BIP-39 phrase (inline); emits argv-leakage advisory; prefer `--phrase-stdin` |
| `--phrase-stdin` | read master BIP-39 phrase from stdin |
| `--ms1 <MS1>` | ms1 card carrying BIP-39 entropy (inline); emits argv-leakage advisory |
| `--ms1-stdin` | read ms1 card from stdin (single chunk) |
| `<positional MS1>` | positional ms1 card (HRP-autodetect). BIP-39 phrase text is NOT accepted positionally (no HRP for autodetect) |
| `--passphrase <P>` | BIP-39 passphrase (inline); emits argv-leakage advisory. **Mandatory** (mutex with `--passphrase-stdin`) |
| `--passphrase-stdin` | read BIP-39 passphrase from stdin (NULL-byte-preserving; single trailing newline stripped). **Mandatory** (mutex with `--passphrase`) |
| `--target-xpub <XPUB-OR-MK1>` | target xpub (any SLIP-0132 prefix: `xpub`/`tpub`/`ypub`/`Ypub`/`zpub`/`Zpub`/`upub`/`Upub`/`vpub`/`Vpub`) OR an `mk1...` bech32 card carrying an xpub |
| `--language <LANGUAGE>` | BIP-39 wordlist (default `english`) |
| `--network <NETWORK>` | network selector: `mainnet` (default) / `testnet` / `signet` / `regtest` |
| `--min-account <N>` | lower bound of account-index iteration, inclusive (default `0`) |
| `--number-of-accounts <N>` | window size starting at `--min-account` (default `20`) |
| `--max-account <N>` | optional upper bound; effective end is `max(min_account + number_of_accounts, max_account + 1)` |
| `--add-path <TEMPLATE>` | additional derivation-path template (repeatable). Literal token `account'` (or `account`) substituted with each iterated account index. Templates without an `account` token are searched once at the literal path |
| `--json` | emit JSON envelope on stdout instead of text-form |
| `--no-auto-repair` | (global) skip BCH auto-fire on `--ms1` decode failure; preserve typed decode error exit |
| `-h, --help` | print help |

Seed-intake mutex (identical to `path-of-xpub`): exactly one of `{--phrase, --phrase-stdin, --ms1, --ms1-stdin, positional}` is required. Auto-fire BCH repair applies ONLY to the `--ms1` decode-failure path.

#### Stderr advisory (always emitted)

Every invocation emits the following advisory on stderr BEFORE the search starts (it does not gate on match / no-match):

```text
note: passphrase verification searches the standard BIP-44/49/84/86 + BIP-48 templates × account range; if the wallet uses a non-standard path, supply --add-path or use `xpub-search path-of-xpub` to find the path first.
```

The advisory is load-bearing UX: a "no match" result does NOT prove the passphrase is wrong — only that no standard path under the (seed, passphrase) pair produces the target. Users with non-standard paths must extend the candidate set via `--add-path`, or solve the path-lookup separately via `path-of-xpub`.

#### Worked example

```sh
# Test BIP-39 phrase (12-word vector from BIP-39 spec)
PHRASE="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

# Suppose a wallet's account-0 zpub was derived from this seed + passphrase "satoshi".
# Verify by supplying the passphrase + the target xpub:
mnemonic xpub-search passphrase-of-xpub \
    --phrase "$PHRASE" \
    --passphrase satoshi \
    --target-xpub "$ZPUB"
```

Stdout (text form, match):

```text
match: m/84'/0'/0'  (template=bip84, account=0)
target-xpub: xpub6... (normalized from zpub; variant=zpub)
searched: 140 candidate paths
```

#### JSON output

`--json` emits a versioned envelope. Schema `v1`. Same shape as `path-of-xpub` with `mode` substituted (separate `PassphraseOfXpubResult` body type keeps future divergence clean). Match shape:

```json
{
  "schema_version": "1",
  "mode": "passphrase-of-xpub",
  "result": "match",
  "path": "m/84'/0'/0'",
  "template": "bip84",
  "account": 0,
  "target_xpub_canonical": "xpub6...",
  "target_xpub_variant": "zpub",
  "searched_count": 140
}
```

No-match shape:

```json
{
  "schema_version": "1",
  "mode": "passphrase-of-xpub",
  "result": "no_match",
  "path": null,
  "template": null,
  "account": null,
  "target_xpub_canonical": "xpub6...",
  "target_xpub_variant": "zpub",
  "searched_count": 140
}
```

`target_xpub_variant` serializes as `null` when the target was supplied in canonical xpub/tpub form (no SLIP-0132 alt-prefix swap occurred). The field is always emitted (not skipped) to keep the JSON envelope structurally stable across runs.

#### Exit codes

| Code | Meaning |
|---|---|
| 0 | Match found (this passphrase produces the target xpub at one of the searched paths) |
| 1 | Bad input (BIP-39 parse failure, xpub parse failure, mk1 decode failure outside the auto-fire path, ms1 decode failure with `--no-auto-repair` or on no-TTY) |
| 4 | No match in searched set (`ToolkitError::XpubSearchNoMatch` with `mode: "passphrase-of-xpub"`) |
| 5 | Auto-fire BCH short-circuit on `--ms1` decode failure (TTY-gated; same contract as `convert` / `inspect` / `verify-bundle`) |
| 64 | Clap arg-parse error (including missing-mandatory-passphrase) |

#### Refusals

| Trigger | Refusal |
|---|---|
| Neither `--passphrase` nor `--passphrase-stdin` supplied | clap `the following required arguments were not provided` error (exit 64) |
| Both `--passphrase` and `--passphrase-stdin` supplied | clap mutex error (exit 64) |
| Positional argument with no `ms1` HRP (e.g., a BIP-39 phrase typed positionally) | `BIP-39 phrase must be supplied via --phrase or --phrase-stdin (no HRP for positional autodetect)` |
| Multiple seed-intake flags supplied | clap mutex error |
| Invalid SLIP-0132 prefix on `--target-xpub` | xpub parse error (exit 1) |

#### Advisories

| Trigger | Stderr advisory |
|---|---|
| Inline `--phrase <v>` | `warning: secret material on argv (--phrase) — pipe via --phrase-stdin to avoid /proc/$PID/cmdline exposure` |
| Inline `--ms1 <v>` | `warning: secret material on argv (--ms1) — pipe via --ms1-stdin to avoid /proc/$PID/cmdline exposure` |
| Inline `--passphrase <v>` | `warning: secret material on argv (--passphrase) — pipe via --passphrase-stdin to avoid /proc/$PID/cmdline exposure` |
| Every invocation (before search starts) | `note: passphrase verification searches the standard BIP-44/49/84/86 + BIP-48 templates × account range; if the wallet uses a non-standard path, supply --add-path or use \`xpub-search path-of-xpub\` to find the path first.` |

## `mnemonic compare-cost`

Compare per-spending-condition cost of wrapping the same miniscript as
`wsh(M)` (Segwit v0 native) versus `tr(NUMS, {M})` (Taproot,
script-path-only, with the BIP-341 H-point as the unspendable internal
key). For every minimal satisfying assignment of `M` — every distinct
"spending condition" — emit one row showing the witness-bytes cost
under each wrapper in virtual bytes, in sats at the user-supplied
feerate, and the `Δ` between the two.

Cost is computed via rust-miniscript v13's `Descriptor::plan(...)`
API; `Plan::witness_size()` returns the full witness-data byte count
(witness items + length prefixes + stack-count varint + the
serialized witnessScript or tapscript + control block). Per-input
costs include the constant 41 vB SegWit-input overhead (36-byte
outpoint + 1-byte scriptSig-length-zero + 4-byte sequence) so the
absolute `wsh vB` and `tr vB` numbers match what Sparrow / Bitcoin
Core / mempool fee-estimators report.

### Synopsis

```sh
mnemonic compare-cost {--miniscript <STR> | --descriptor <STR> | stdin (when non-TTY)} [--feerate <SATS_PER_VB>] [--max-conditions <N>] [--json]
```

### Flags

| Flag | Purpose |
|---|---|
| `--miniscript <STR>` | bare miniscript fragment with abstract labels (`pk(A)`, `pk(B)`, …) or concrete hex pubkeys; cost is key-agnostic so abstract labels auto-substitute to deterministic dummy keys. Mutually exclusive with `--descriptor`. |
| `--descriptor <STR>` | full descriptor — `wsh(M)`, `sh(wsh(M))`, or single-leaf `tr(IK, {M})` (v0.28.0). The wrapper is stripped to recover the inner miniscript `M` before the comparison. Multi-leaf `tr(IK, {M1, M2, ...})` and keypath-only `tr(IK)` are refused with exit `3`. Mutually exclusive with `--miniscript`. |
| `--feerate <SATS_PER_VB>` | decimal sats per virtual byte for the sats columns; default `1.0`, max `10000.0`. Out-of-range values exit `64`. |
| `--max-conditions <N>` | hard cap on raw enumeration size `n_abs × n_rel × 2^(\|signers\|+\|preimages\|)`; exceeding the cap exits `3` before any enumeration. Default `4096` (permits up to 10 signers+preimages). When `>256`, a soft warn-trail entry appears in `notes[]` once 256 rows are produced. Min `1`. |
| `--json` | emit a JSON envelope on stdout instead of the plaintext aligned-column table. |
| `--help` | print help. |

When neither `--miniscript` nor `--descriptor` is supplied and stdin
is not a terminal, the first non-blank line of stdin is read and
classified: if its top-level identifier is in `{wsh, sh, tr, wpkh,
pkh, combo, addr, rawtr, raw}` it routes as a descriptor, otherwise
as a miniscript. If both flags are supplied, the command exits `64`
(clap `conflicts_with`).

### Row labels

Each row is labeled by the minimal satisfying assignment that
produces it. Components are joined by ` + `:

- **Signers** — the user's input label (`A`, `Alice`, …) for the
  abstract-label case; `key[i]` (AST-order index) for concrete-key
  input where no user label is available.
- **Preimages** — `preimage(h<i>)` in AST-order, one per `sha256` /
  `hash256` / `ripemd160` / `hash160` leaf supplied.
- **Absolute timelocks** — `after(height)` for block-height locks
  (`after(N)` with `N<500_000_000`), `after(time)` for MTP-time locks
  (`N≥500_000_000`).
- **Relative timelocks** — `older(blocks)` for sequence-based locks
  (`older(N)` with the TIME_LOCK_FLAG / bit 22 clear),
  `older(512s)` for 512-second-interval locks (bit 22 set).

### Worked examples

**1. Bare miniscript with `--feerate` set:**

```sh
mnemonic compare-cost --miniscript 'or_b(pk(A),s:pk(B))' --feerate 25.0
```

Stdout:

```text
Input: or_b(pk(A),s:pk(B))
Wrapper comparison: wsh(M)  vs  tr(NUMS, {M})
Feerate: 25.0 sat/vB

Condition | wsh vB | tr vB |  Δ vB | wsh sats | tr sats | Δ sats
----------+--------+-------+-------+----------+---------+-------
A         |     60 |    84 |   +24 |     1500 |    2100 |   +600
B         |     60 |    84 |   +24 |     1500 |    2100 |   +600

note: per-condition vbytes are rounded individually; absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct
```

Either A or B can sign alone; both pay the same cost. tr costs `+24`
vB more per spend than wsh because the tr witness carries an extra
33-byte control block.

**2. Timelocked recovery path (SPEC §5 hero example):**

```sh
mnemonic compare-cost --miniscript 'or_d(pk(A),and_v(v:pk(B),older(144)))'
```

Stdout:

```text
Input: or_d(pk(A),and_v(v:pk(B),older(144)))
Wrapper comparison: wsh(M)  vs  tr(NUMS, {M})
Feerate: 1.0 sat/vB

Condition         | wsh vB | tr vB |  Δ vB | wsh sats | tr sats | Δ sats
------------------+--------+-------+-------+----------+---------+-------
A                 |     60 |    85 |   +25 |       60 |      85 |    +25
B + older(blocks) |     60 |    86 |   +26 |       60 |      86 |    +26

note: per-condition vbytes are rounded individually; absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct
```

Two rows: A can sign at any time (no timelock needed); B can sign
after 144 blocks (the recovery path costs `+1` vB more than A's
direct path on both wrappers).

**3. Descriptor input via stdin, JSON output:**

```sh
echo 'wsh(pk(02998512205ec6a5cdb77d5b4f7de63c560d1e846162612ee178c49e7b6cc44fb9))' | \
  mnemonic compare-cost --json
```

Stdout:

```json
{
  "schema_version": 1,
  "subcommand": "compare-cost",
  "input": {
    "form": "descriptor",
    "value": "wsh(pk(02998512205ec6a5cdb77d5b4f7de63c560d1e846162612ee178c49e7b6cc44fb9))"
  },
  "extracted_miniscript": "pk(02998512205ec6a5cdb77d5b4f7de63c560d1e846162612ee178c49e7b6cc44fb9)",
  "feerate_sat_per_vb": 1.0,
  "conditions": [
    {
      "label": "key[0]",
      "wsh_vbytes": 60,
      "tr_vbytes": 75,
      "delta_vbytes": 15,
      "wsh_sats": 60,
      "tr_sats": 75,
      "delta_sats": 15
    }
  ],
  "notes": [
    "per-condition vbytes are rounded individually; absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct",
    "input had concrete keys; cost is identical to the abstract case"
  ]
}
```

The stdin path auto-classifies the input as a descriptor (top-level
identifier `wsh`); the JSON envelope's `input.form` field records the
chosen path. For `--descriptor` input the `extracted_miniscript` field
holds the wrapper-stripped inner miniscript M (SPEC §5) — note the
example above shows `pk(02998512…)` in that field, not the full
`wsh(pk(02998512…))` the user supplied.

**4. Single-leaf `tr(IK, {M})` with a non-NUMS internal key (v0.28.0):**

```sh
mnemonic compare-cost --descriptor \
  'tr(f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9,pk(dff1d77f2a671c5f36183726db2341be58feae1da2deced843240f7b502ba659))' \
  --feerate 25.0
```

Stdout:

```text
Input:     tr(f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9,pk(dff1d77f2a671c5f36183726db2341be58feae1da2deced843240f7b502ba659))
Extracted: pk(02dff1d77f2a671c5f36183726db2341be58feae1da2deced843240f7b502ba659)
Wrapper comparison: wsh(M)  vs  tr(NUMS, {M})
Feerate: 25.0 sat/vB

Condition | wsh vB | tr vB |  Δ vB | wsh sats | tr sats | Δ sats
----------+--------+-------+-------+----------+---------+-------
key[0]    |     60 |    75 |   +15 |     1500 |    1875 |   +375

Keypath-spend (via IK f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9): 58 vB | 1450 sats

note: per-condition vbytes are rounded individually; absolute numbers may differ by ±1 from real-tx accounting, Δ values are correct
note: input had concrete keys; cost is identical to the abstract case
note: input had a non-NUMS internal key IK (f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9); this report compares script-path-only cost (tr modeled as tr(NUMS, {M})). Keyspend-via-IK costs ~58 vB total (under SIGHASH_DEFAULT) and is the cheapest spend if signing with IK is acceptable.
```

The per-condition table compares `wsh(M)` against the script-path of
`tr(NUMS, {M})` (the script-path is canonicalized to a NUMS internal
key for the comparison so the wsh and tr sides are like-for-like
script-spend cost). Because the user supplied a non-NUMS internal key,
the **keypath-spend** path is also available: signing with the IK
directly costs `58 vB` (Schnorr 64B + length prefix + stack-count = 66
witness bytes; `(164+66+3)/4 = 58`). That annotation line is the
cheapest spend if signing with IK is acceptable for the wallet's
spending policy.

When `IK == NUMS`, the keypath-spend cost is **not** surfaced — the
NUMS H-point has no known discrete-log so signing under it is
impossible by construction; only the script-path is meaningful.

The internal key is reverse-projected from x-only (32B) to compressed
(33B) by prepending the byte `0x02` (BIP-340 lift-x even-y LOCK; SPEC
§11.2). Cost is parity-invariant — the choice of `0x02` over `0x03`
does not affect any vbyte count — so the LOCK is a convention for
deterministic round-trips, not a cost-load-bearing decision. (Pinned
by `tests/cli_compare_cost.rs::cost_is_parity_invariant_02_vs_03`.)

Multi-leaf `tr(IK, {M1, M2, ...})` is rejected; supply one leaf at a
time via `--miniscript`. Keypath-only `tr(IK)` with no script-tree is
also rejected — there's no inner miniscript to compare.

### Notes catalog

The `notes[]` array in JSON output (and the trailing `note:` lines
in plaintext output) carry advisory text. Known entries:

| Note | Trigger |
|---|---|
| `per-condition vbytes are rounded individually; …` | always present (vbyte rounding caveat per §4). |
| `feerate is 0; sats columns will be 0` | `--feerate 0.0`. |
| `enumeration reached soft threshold; <N> conditions shown` | row count ≥ 256 (or `--max-conditions` if smaller). |
| `input had concrete keys; cost is identical to the abstract case` | input contained no abstract labels. |
| `input contains hash-preimage fragments; …` | input has at least one `sha256` / `hash256` / `ripemd160` / `hash160` leaf. |
| `input had a non-NUMS internal key IK; …` | (v0.28.0) `--descriptor tr(IK, {M})` with `IK ≠ NUMS`. The advisory carries the IK hex; the JSON envelope's `keypath_spend` field carries the keypath-spend cost (`{ internal_key_xonly_hex, vbytes: 58, sats }`); plaintext output adds a `Keypath-spend (via IK …): 58 vB \| <SATS> sats` annotation line below the per-condition table. |

### Exit codes

| Condition | Exit |
|---|---|
| success (rows emitted; advisories in `notes[]`) | `0` |
| input parse error (malformed miniscript / descriptor) | `2` |
| no input supplied (TTY stdin + no flag) | `1` |
| miniscript valid in only one of {Segwitv0, Tap} after `multi↔multi_a` rewrite | `3` |
| unsupported wrapper (pkh, wpkh, bare, keypath-only `tr(IK)` with no script-tree) | `3` |
| multi-leaf `tr(IK, {M1, M2, ...})` (one-leaf-at-a-time via `--miniscript`) | `3` |
| eager precheck exceeded `--max-conditions` cap | `3` |
| miniscript has zero satisfying conditions | `3` |
| `--miniscript` AND `--descriptor` both supplied | `64` (clap mutex) |
| `--feerate` out of `[0.0, 10000.0]` or non-numeric | `64` |
| `--max-conditions 0` | `64` |
