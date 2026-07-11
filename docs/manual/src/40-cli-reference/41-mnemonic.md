# `mnemonic` reference

The integration-layer CLI for the m-format constellation. The subcommands:
[`bundle`](#mnemonic-bundle), [`verify-bundle`](#mnemonic-verify-bundle),
[`convert`](#mnemonic-convert), [`export-wallet`](#mnemonic-export-wallet),
[`restore`](#mnemonic-restore),
[`import-wallet`](#mnemonic-import-wallet),
[`derive-child`](#mnemonic-derive-child),
[`electrum-decrypt`](#mnemonic-electrum-decrypt),
[`final-word`](#mnemonic-final-word), [`seed-xor`](#mnemonic-seed-xor),
[`seedqr`](#mnemonic-seedqr), [`slip39`](#mnemonic-slip39),
[`ms-shares`](#mnemonic-ms-shares),
[`nostr`](#mnemonic-nostr), [`silent-payment`](#mnemonic-silent-payment),
[`addresses`](#mnemonic-addresses),
[`decode-address`](#mnemonic-decode-address),
[`verify-message`](#mnemonic-verify-message), [`repair`](#mnemonic-repair),
[`inspect`](#mnemonic-inspect), [`compare-cost`](#mnemonic-compare-cost),
[`xpub-search`](#mnemonic-xpub-search),
[`build-descriptor`](#mnemonic-build-descriptor), and
[`gui-schema`](#mnemonic-gui-schema) (introspection only, no user-facing
semantics). Run any with `--help` for the authoritative flag set; this
reference tracks the current release.

> **Recovering a forgotten BIP-39 passphrase.** If you have your seed
> words (entropy) but not the BIP-39 passphrase (the optional "25th
> word"): a BIP-39 passphrase has no internal verifier — every candidate
> yields a valid-looking wallet — so correctness is only definable against
> a value you already know (an address, xpub, or master-fingerprint). If
> you have a **list of likely passphrases**, `mnemonic xpub-search
> passphrase-of-xpub --passphrase-candidates-file <file> --target-xpub <a
> known xpub>` (see [that mode](#mnemonic-xpub-search-passphrase-of-xpub))
> tests each candidate against that known value. To **generate or mutate a
> keyspace** (wordlists, masks, typo models), `mnemonic` does not — an
> external open-source tool does:
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

### Consensus-masked relative timelocks {#consensus-masked-relative-timelocks}

If a descriptor's `older(N)` value is BIP-68 consensus-masked (stray bits,
or a zero 16-bit value such as `older(65536)`), this command prints a
non-blocking advisory to stderr noting the effective (weaker) value. The
command still succeeds — it never refuses to back up or inspect an
already-deployed wallet.

### Unrestorable descriptor shapes {#unrestorable-shapes}

Some descriptor shapes engrave a wire-faithful `md1` card that
[`mnemonic restore`](#mnemonic-restore) cannot yet mechanically reconstruct
(it refuses loudly rather than silently rebuild a different wallet). When the
descriptor has one of these shapes, this command prints a non-blocking advisory
to stderr at engrave time — the card is still emitted (a faithful backup); keep
the full descriptor to restore. The shapes are:

- `sortedmulti()` **inside a combinator** (not the sole child of `wsh`/`sh`);
- a **hardened use-site** — a hardened wildcard (`/*h`) on the shared suffix or
  inside a per-cosigner override (a hardened child cannot be derived from an xpub);
- **per-cosigner use-site overrides on a `tr(sortedmulti_a)` card** — the
  `tr(sortedmulti_a)` reconstruction renderer is gated on the next rust-miniscript
  release (`Terminal::SortedMultiA`, > 13.1.0); interim loud-refuse;
- **per-cosigner use-site overrides on a taproot card whose internal key is a
  real key (non-NUMS)** — the faithful override path (v0.59.1) covers only
  NUMS-internal `tr(multi_a)`, so a non-NUMS internal key combined with
  *divergent* per-cosigner suffixes is still refused. This is the **override**
  case only: a **baseline** non-NUMS key-path taproot card (no divergent
  per-cosigner suffixes) **is** restorable since v0.55.3 — see
  [restore](#mnemonic-restore).

Both pending taproot legs are tracked by FOLLOWUP
`restore-md1-taproot-use-site-override-arm`.

Non-hardened **per-cosigner use-site path overrides** (cosigners with divergent
derivation suffixes) are **reconstructed faithfully** and no longer fire the
*unrestorable*-shape advisory for: non-taproot `wsh`/`sh` multisig (since
v0.58.2), and NUMS-keyed single-leaf `tr(multi_a)` taproot cards (since v0.59.1)
— both via md-codec 0.37.0's per-`@N` multipath reconstruction. The taproot one,
however, now fires a separate **loud funds-safety warning** — see
[Custom use-site on a NUMS-taproot card](#custom-use-site-nums-taproot) below.

These are the shapes the [multisig-cosigner restore](#multisig-cosigner-restore)
path refuses to reconstruct. The same advisory fires on
[`mnemonic import-wallet`](#mnemonic-import-wallet) (the other surface that
engraves an `md1` from a descriptor).

### Custom use-site on a NUMS-taproot card {#custom-use-site-nums-taproot}

A `tr(NUMS, multi_a)` multisig card with **custom per-cosigner use-site
derivation paths** — divergent derivation suffixes per cosigner, e.g. `@0` on
`/<0;1>/*` but `@1` on `/<2;3>/*` — **restores faithfully** (since v0.59.1; see
above). Unlike the unrestorable shapes, this card is **not** refused at restore.

But **no known wallet produces this shape**: every standard wallet uses one
**uniform** `<0;1>/*` suffix across all cosigners. A user on this path has almost
certainly misconfigured, and the reconstructed addresses will not match any
standard wallet software — a funds-loss risk. So, rather than refuse (which would
strand the rare legitimate user who deliberately chose divergent paths), the
toolkit **reconstructs the card faithfully and warns loudly**. At engrave
([`mnemonic bundle`](#mnemonic-bundle) and
[`mnemonic import-wallet`](#mnemonic-import-wallet)) *and* at restore
([`mnemonic restore --md1`](#mnemonic-restore)), it prints a non-blocking stderr
line beginning:

> `WARNING (funds-safety): this card is a tr(NUMS, multi_a) multisig with CUSTOM
> per-cosigner use-site derivation paths …`

— going on to state that no known wallet produces the shape, that the
reconstructed addresses will **not** match your wallet software, that you risk
**permanent loss of funds** if the divergence was unintended, and that you should
**verify the descriptor against your wallet** before relying on the card. The
operation still succeeds (exit 0) and the addresses are still emitted; if you
*did* deliberately intend divergent per-cosigner paths the warning is benign.

A **baseline** `tr(NUMS, multi_a)` card with the same uniform `<0;1>/*` suffix on
every cosigner is **not** custom, so it carries **no** use-site overrides and
fires **no** funds-safety warning.

This list is about **keyed wallet-policy `md1` cards**. A **keyless
multisig / general TEMPLATE `md1`** (`bundle --md1-form=template`) is a
different artifact: it carries no keys by design and **is restorable** —
you complete it by re-supplying the keys (see [Multisig template
completion](#multisig-template-completion)). Only `tr(sortedmulti_a)` and
hardened use-sites are refused as templates (they are refused at *emit*
time, so no such template card exists to restore).

### Non-representable use-site steps {#non-representable-use-site-steps}

md1's use-site path can only encode two shapes for the derivation steps
that follow a key placeholder (an `@N` slot, or an inline
`[fp/path]xpub` key): the BIP-389 **multipath** form `/<a;b>/*` (receive
and change sharing one card), or a **bare** `/*` wildcard. A **fixed
single step** — `/0/*`, `/0h/*`, or any other literal index in place of
the multipath alternatives — is *not representable* in md1.

The BIP-388 **combined-wildcard shorthand** `/**` is **accepted**: it is
defined as an exact synonym for `/<0;1>/*` (receive = index 0, change =
index 1), so a final-use-site `/**` is **rewritten to `/<0;1>/*` before
the descriptor reaches the parser** and behaves byte-for-byte
identically to the explicit multipath spelling. This happens on every
literal-descriptor surface — `mnemonic bundle --descriptor`, `mnemonic
verify-bundle --descriptor`, `mnemonic import-wallet` (every source
format whose descriptor carries an `@N`/bracketed key), `mnemonic
export-wallet --descriptor`, `mnemonic xpub-search
account-of-descriptor`, and `mnemonic gui-schema
--classify-descriptor`. (Only the *exact* `/**` is expanded; `/***` and
`/**'` are not the shorthand and are left untouched.)

A **fixed step** is a different matter. It used to be *silently dropped*
rather than rejected: `@0/0/*` lexed identically to bare `@0`, so a
receive-only (`/0/*`) and a change-only (`/1/*`) descriptor for the
*same* key collapsed to a byte-identical wallet-policy card — a
funds-loss bug, since the card derives a *different* address space than
the wallet actually in use. This is now refused rather than silently
collapsed: `mnemonic bundle --descriptor`, `mnemonic import-wallet`
(every source format whose descriptor carries an `@N`/bracketed key —
Sparrow is unaffected, see below), and `mnemonic verify-bundle` all
**reject** (exit 2, `DescriptorParse`) a use-site path ending in a fixed
step. (A fixed step *combined* with the shorthand, e.g. `/0/**`, still
rejects: the `/**` expands to `/0/<0;1>/*`, whose leading `/0` fixed
step is un-representable.)

**Remedy:** rewrite the use-site path as the explicit multipath form —
e.g. replace separate `/0/*` (receive) and `/1/*` (change) entries for
the same key with one `/<0;1>/*` entry (alternative order follows the
receive/change convention: index 0 = receive, index 1 = change), or use
the `/**` shorthand for the same thing.

**Sparrow is unaffected.** Sparrow's own `@N/**` template placeholder is
expanded to the multipath form internally before this check runs, so a
normal Sparrow import is unaffected by this rule.

**Bitcoin Core auto-recombination.** `bitcoin-cli listdescriptors`
exports the receive and change branches of an account as two *separate*
descriptor entries (`/0/*` + `/1/*`), never as one combined multipath
entry. Rather than hard-fail this check on each entry, `import-wallet
--format bitcoin-core` **auto-recombines** a same-key receive/change
pair into one `/<0;1>/*` multipath bundle at parse time (a
receive/change-shaped pair whose keys differ is still refused —
distinct keys are different wallets). See
[Foreign wallet formats → Bitcoin Core](#bitcoin-core-listdescriptors)
for the guard matrix.

**`verify-bundle`'s two intake paths differ.** A **concrete** descriptor
(inline `[fp/path]xpub` keys) rejects at exit 2 (`DescriptorParse`)
*before* comparing against the supplied cards — this closes a
false-pass: previously, a `/0/*`-collapsed descriptor verified
successfully against the *wrong* card, because both sides collapsed
identically. An **`@N`-template** descriptor (keys supplied via
`--slot`) instead rejects at exit 4 (`DescriptorReparseFailed`) when the
completed wallet is re-parsed.

### Synopsis

```sh
mnemonic bundle --network <NETWORK> [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--network <NETWORK>` | mainnet / testnet / signet / regtest |
| `--template <TEMPLATE>` | bip44 / bip49 / bip84 / bip86 / wsh-multi / wsh-sortedmulti / sh-wsh-multi / sh-wsh-sortedmulti / tr-multi-a / tr-sortedmulti-a |
| `--descriptor <DESCRIPTOR>` | user-supplied descriptor; accepts either a BIP-388 `@N` template (keys supplied via `--slot`) **or a bare concrete descriptor** with inline `[fp/path]xpub` keys (watch-only output); **(v0.49.0) or a BIP-388 wallet-policy JSON** `{name, description_template, keys_info}` (auto-detected by a leading `{`, origin-annotated `keys_info` required), expanded to the concrete descriptor — the inverse of `export-wallet --format bip388`; both apostrophe and `h`-form hardened paths are accepted; mutually exclusive with `--template` and `--descriptor-file` |
| `--descriptor-file <DESCRIPTOR_FILE>` | descriptor read from a single-line UTF-8 file; mutually exclusive with `--descriptor` |
| `--language <LANGUAGE>` | BIP-39 wordlist for the input phrase |
| `--passphrase <PASSPHRASE>` | BIP-39 mnemonic-extension passphrase |
| `--passphrase-stdin` | read `--passphrase` from stdin (raw, NULL-byte preserving); single stdin per invocation |
| `--account <ACCOUNT>` | BIP-32 account index (default 0) |
| `--md1-form <policy\|template>` | (#28) what the `md1` card encodes (default `policy`). `policy` = the full keyed wallet-policy `md1` (pre-#28; identifies **this** wallet). `template` = a **keyless**, fingerprint-stripped, origin-conditional template `md1` — a backup of the wallet **type**, byte-identical (for a canonical type) across all users of that type ("one engraving for thousands"); keys/accounts are supplied at restore and the specific wallet is identified out-of-band by the `WalletPolicyId` printed on **stderr**. **(phase 1, v0.59.0)** canonical single-sig (`bip44` / `bip84` / `bip86`). **(phase 2, v0.60.0)** ALSO multisig (`wsh(multi/sortedmulti)`, `sh(wsh)`), general / thresh policies, and `tr(NUMS, multi_a)` — emitting one keyless template `md1` + N keyless cosigner `mk1` stubs with a loud key-ordering warning. Still refused: `tr(sortedmulti_a)`, hardened use-sites, and `bip49` (nested-segwit) — use `--md1-form=policy`. See [Template-only md1](#template-only-md1) and [Multisig template completion](#multisig-template-completion) |
| `--json` | emit JSON output |
| `--no-engraving-card` | suppress the stderr engraving-card layout |
| `--group-size <N>` | mstring display grouping: insert a separator every N characters in the emitted `ms1`/`mk1`/`md1` card strings; `0` = unbroken (default 5). Display only — `--json` and `verify-bundle` forensic strings always stay unbroken. The same flag (with `--separator`) is also accepted on `convert` (when emitting an `ms1`/`mk1` card) and on `ms-shares split` / `ms-shares combine --to ms1`. |
| `--separator <space\|hyphen\|comma>` | the grouping separator for `--group-size` (default `space`); accepts the keyword or the literal `-` / `,` or a space. |
| `--multisig-path-family <FAMILY>` | bip48 or bip87 (default bip87) |
| `--privacy-preserving` | suppress the master fingerprint from mk1 + engraving card |
| `--self-check` | re-parse and verify the emitted bundle round-trips |
| `--threshold <THRESHOLD>` | multisig K of N (1 ≤ K ≤ N ≤ 16) |
| `--slot <SLOT>` | repeating; `@N.<subkey>=<value>` (subkey: `phrase`, `seedqr`, `entropy`, `ms1`, `xpub`, `master_xpub`, `fingerprint`, `path`, `wif`, `xprv`); for secret-bearing subkeys `=-` reads from stdin. `seedqr` (v0.31.3+) takes a 48- or 96-digit SeedQR string and decodes inline at slot-emit time, materializing the BIP-39 phrase identically to a `@N.phrase=` invocation. `ms1` (v0.41.0+) takes a raw BIP-93 codex32 secret string and decodes it inline, materializing the slot's entropy identically to a `@N.entropy=` invocation; it is **language-preserving** — a `mnem`-kind ms1 carries its BIP-39 wordlist language on the wire, so the emitted card round-trips that language without re-specifying `--language`. Supplying `--language` whose wordlist disagrees with the slot's wire language is refused (`SlotInputViolation` `kind:"language-conflict"`, exit 2); omit `--language` or set it to match. A K-of-N codex32 **share** (not a single-string secret) is rejected with a pointer to `ms-shares combine` — reassemble the secret first, then slot the combined `ms1`. |
| `--import-json <FILE\|->` | (v0.27.0) synthesize a bundle from an `import-wallet --json` envelope rather than from `--template` / `--descriptor`; the envelope's `bundle.descriptor` carries the descriptor and `bundle.mk1` chunks decode to per-cosigner xpubs + fingerprints + paths; mutually exclusive with `--template`, `--descriptor`, `--descriptor-file`; seed overlay (`--slot @N.phrase=`) applies to slots where envelope `ms1[N] == ""` (watch-only); supplying overlay for an already-seeded slot is `BadInput`. The replayed `bundle.descriptor` re-parses through the same pipeline as `--descriptor`, so an old envelope carrying a fixed use-site step (`/0/*`) now hits the [use-site residue reject](#non-representable-use-site-steps) too (exit 2) |
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

### Template-only md1 {#template-only-md1}

`--md1-form=template` (#28) emits a **keyless single-sig template
`md1`** instead of the full keyed wallet-policy card. The template is
fingerprint-stripped and canonical-origin-elided, so it is
**byte-identical across every user of the same wallet type** — a backup
of the wallet *type* (`bip44` / `bip84` / `bip86`), not of one specific
wallet ("one engraving for thousands"). It is account-agnostic: no
fingerprint, no account, no origin is baked into the card.

Because the card carries no keys, the toolkit prints the wallet's
**`WalletPolicyId`** on **stderr** at emit time. Record it out-of-band
(it is the only thing that pins the engraved template to *your* wallet);
the printed convenience prefix is 4 bytes, but you may keep and later
match a prefix of any length.

Completion happens at restore time. Re-supply the seed plus the account
or origin to turn the keyless template back into a fully-keyed wallet:

```sh
# canonical account (default origin m/<purpose>'/<coin>'/<account>'):
mnemonic restore --md1 <template-md1> --from phrase=- --account 7 \
    --expect-wallet-id <id-from-stderr>
# arbitrary explicit origin (overrides the canonical account default):
mnemonic restore --md1 <template-md1> --from phrase=- \
    --origin "m/84'/0'/7'"
```

The same completion works on [`verify-bundle`](#mnemonic-verify-bundle)
via its `--from` + `--account` / `--origin` / `--expect-wallet-id`
flags. `--expect-wallet-id` recomputes the `WalletPolicyId` from the
completed wallet and refuses loudly on mismatch (exit 4); it is **not**
checked when `--origin` is supplied (the explicit-origin id is a
different preimage from the canonical-account one).

The single-sig template form covers the canonical shapes `bip44` /
`bip84` / `bip86`; `bip49` (nested-segwit) and non-canonical single-sig
are refused — use `--md1-form=policy` for those. **Multisig and general
policies** have their own template form (phase 2, v0.60.0) — see
[Multisig template completion](#multisig-template-completion) below.

### Multisig template completion {#multisig-template-completion}

**(#28 phase 2, v0.60.0.)** `--md1-form=template` also admits **multisig**
(`wsh(multi/sortedmulti)`, `sh(wsh)`), **general / thresh** policies, and
**`tr(NUMS, multi_a)`**. Such a bundle emits ONE keyless template `md1`
(the shared policy, keys stripped) plus **N keyless cosigner `mk1` stub
cards** — a reusable "engraving of the wallet *type*". `tr(sortedmulti_a)`
and hardened use-sites are refused (use `--md1-form=policy`).

Because a `multi()` descriptor is **order-sensitive** — N keys can be
assigned to the N slots N! ways and only one assignment is the wallet you
funded — `bundle` prints a **loud key-ordering warning** plus the
order-sensitive **`WalletPolicyId`** (the completion checksum) on
**stderr** at emit time. (For order-independent `sortedmulti` the warning
is softened — slot order does not change the wallet.) Record the
`WalletPolicyId` out-of-band; it (or a known address) is what lets restore
pick the correct assignment.

**Completion** re-supplies the keys and lets a parallel permutation-search
engine place them. Provide:

- your **OWN seed** via `--from <seed>` and the account(s) it is used at
  via `--account <list>` (one own key per account — e.g. `--account 0,1,2,3`
  for four own slots);
- each **cosigner key** via a `--cosigner <mk1>` card — unassigned
  (search-placed) or explicit (`--cosigner @N=<mk1|xpub>`).

The engine then resolves the unique key→slot assignment via one of three
**completion modes**:

1. **id-search** — `--expect-wallet-id <prefix>` (a **strong** prefix
   sized to the realized search space) recomputes the `WalletPolicyId` for
   each candidate assignment and keeps the unique match.
2. **address-search** — `--search-address <addr>` (collision-free,
   recommended) matches a known receive/change scriptPubKey across the
   `--search-addr-min`..`--search-addr-max` index range on the
   `--search-chain` branch(es) (`receive` (default) / `change` / `both`).
3. **explicit assignment** — pin every cosigner with `--cosigner @N=` (no
   search).

The engine carries an adaptive **~1-hour search-time ceiling**; if the
realized space would exceed it the tool refuses with a printed exhaustive-
time estimate. Override with **`--accept-search-time <duration>`** (a
humantime duration that must be ≥ the estimate — a forced acknowledgment).

**Funds-safety floors (all refuse loudly — never a silent wrong wallet):**
distinct-keys (no slot may collide), every-slot-supplied (own + cosigner
keys must fill all N slots), a strong `--expect-wallet-id` prefix, and an
**ambiguity / no-match** refusal (≥2 assignments match, or none does).
Per-slot origins are **built fresh** from the supplied keys — the origin
carried in the template card is never loaded for derivation.

When the exact own account(s) are unknown, **`--own-account-max K`**
(v0.70.0) over-supplies the own candidates and the engine selects the
subset actually used — see
[Subset-search / over-supply completion](#subset-search-over-supply-completion)
below.

The same completion intake is available on
[`verify-bundle`](#mnemonic-verify-bundle) (`--from` / `--cosigner` /
`--search-address` + range + `--search-chain` / `--accept-search-time`,
plus `--own-account-max` / `--search-cosigner-subset`), which verifies the
card↔template-id binding and recomposes the wallet via the same engine.

#### Subset-search / over-supply completion {#subset-search-over-supply-completion}

The exact-account path above assumes the operator knows precisely which
account(s) the own seed is used at (`--account <list>`) and supplies
exactly the right cosigner cards. When that knowledge is incomplete,
**over-supply** the candidates and let the engine resolve the unique
assignment by an **own-anchored k-permutation subset-search** (v0.70.0).

- **Own-only by default — `--own-account-max K`.** The common case: the
  operator does **not recall their account index**. Deriving the own seed
  at every account `0..K-1` over-supplies `K` own candidates; the search
  picks the subset actually placed in the template. The supplied
  `--cosigner` cards are still matched **exactly** (own-only). Mutually
  exclusive with `--account` (a fixed index needs no search). `K ≤ 256`.
- **Opt-in cosigner subset — `--search-cosigner-subset`.** For uncertain
  **cosigner** cards (the operator over-supplies `--cosigner` cards, unsure
  which/how many belong), this opt-in flag extends the search to resolve
  the cosigner subset too. Bounded (below). Mutually exclusive with
  `--cosigner @N=` (explicit placement); composes with `--own-account-max`
  / `--account`.

Over-supply enlarges the search space, so the **strong-prefix requirement
scales with it**: the realized space `realized_s` (`= S_own =
C(K_own,j)·N!` for own-only, `= S_opt = Σ_j C(K_own,j)·C(M_sup,N−j)·N!`
when cosigner-subset is engaged) sizes the `--expect-wallet-id` prefix the
engine demands. A too-short prefix **refuses an ambiguous match** (never a
silent wrong wallet). For large pools, **`--search-address` is recommended**
(full-scriptPubKey match — collision-free, no prefix-length tuning).

**Bounds (§6 ceilings).** Own pool `K_own ≤ 256`; the optional-cosigner
search space `S_opt ≤ 1e15` (a hard ceiling); the adaptive **~1-hour**
time-cap applies on top (override with `--accept-search-time`). Inputs that
would exceed a ceiling **refuse** (exit ≠ 0) with a printed estimate rather
than run unbounded. The own candidate pool is derived **public-only** (the
own xpriv is scrubbed by-move, never lingering un-scrubbed). All refusals
exit ≠ 0.

```sh
# 2-of-3 wsh(multi) template; operator does not recall their own account
# index — over-supply own accounts 0..4 and id-search the result:
mnemonic restore --md1 <template-md1> \
    --from phrase=- --own-account-max 5 \
    --cosigner <cosigner-mk1-A> --cosigner <cosigner-mk1-B> \
    --expect-wallet-id <strong-id-prefix-from-stderr>
```

```sh
# 2-of-3 wsh(multi) template: own key at account 0, two cosigner stubs,
# id-search against the WalletPolicyId printed by `bundle --md1-form=template`:
mnemonic restore --md1 <template-md1> \
    --from phrase=- --account 0 \
    --cosigner <cosigner-mk1-A> --cosigner <cosigner-mk1-B> \
    --expect-wallet-id <id-from-stderr>
```

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
in a human-readable form — by default each card is printed once, broken
into space-separated groups of five characters, suitable for steel
engraving and reading aloud during verification (`--group-size 0` gives
the dense single-line form; `--separator hyphen` groups with dashes):

```sh
mnemonic bundle --network mainnet --account 0 \
  --descriptor "$DESC" \
  --language english \
  --slot '@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about' \
  --slot '@1.phrase=legal winner thank year wave sausage worth useful legal winner thank yellow' \
  --slot '@2.phrase=letter advice cage absurd amount doctor acoustic avoid letter advice cage above'
```

Stdout (the cards — under v0.21.0+ SPEC §5.8 per-slot emission, all three cosigners now get their own ms1 card when phrases are supplied for all three slots):

```{.text include="41-bundle-inheritance-cards.out" lines="1-29"}
PLACEHOLDER — generated from transcripts/41-bundle-inheritance-cards.out lines 1-29 at build
```

Each card type is printed once, in 5-character groups separated by
spaces (the default display grouping), suitable for steel-plate
engraving and reading aloud during verification. The grouping
separators are non-load-bearing — pass `--group-size 0` for the dense
single-line form, or any grouping you like; intake strips the
separators, so every form decodes back to the same payload.

Stderr (info notice + bundle-summary engraving card):

```{.text include="41-bundle-inheritance-cards.err" lines="4-14"}
PLACEHOLDER — generated from transcripts/41-bundle-inheritance-cards.err lines 4-14 at build
```

The engraving-card block on stderr is a wallet-level summary the user
copies onto a separate piece of paper kept with the bundle; it lists
the threshold, per-cosigner fingerprint+origin triples, and the
recovery rule. The `6e6be` / `6e6b` short tags are chunk-set-id hex
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

```{.text include="41-bundle-inheritance-json.out"}
PLACEHOLDER — generated from transcripts/41-bundle-inheritance-json.out at build
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
  mk1 chunking for the wrapped key card. Each cosigner's chunk-set gets
  its own `chunk_set_id` — the policy stub XORed with the cosigner slot
  index — so the verify-bundle intake can correctly group chunks back
  per cosigner before mk-codec decode. (This superseded v0.20.0's
  xpub-fingerprint-derived scheme, which collided for two cosigners
  reusing one xpub at different paths.)
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

```{.text include="41-inheritance.out" lines="1-5"}
PLACEHOLDER — generated from transcripts/41-inheritance.out lines 1-5 at build
```

The `info:` line is the v0.19.0 silent-default-with-stderr-notice
feature firing on this recipe's non-canonical `wsh(andor(...))`
descriptor — the BIP-48 origin path is inferred silently and the
bundle proceeds. `verify-bundle` emits the same three secret-on-argv
warnings (no info-notice — the default-path inference fired once at
bundle-time and is now baked into the envelope).

Expected output (one block per cosigner; final `result: ok`):

```{.text include="41-inheritance.out" lines="9-30"}
PLACEHOLDER — generated from transcripts/41-inheritance.out lines 9-30 at build
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
| `@N`/key use-site path ends in a fixed step (`/0/*`, `/0h/*`) | refused, exit 2 (`DescriptorParse`) — message names the offending residue and the multipath remedy; see [Non-representable use-site steps](#non-representable-use-site-steps). The BIP-388 `/**` shorthand is **accepted** (expanded to `/<0;1>/*`), not refused |

---

## `mnemonic verify-bundle`

Re-derive expected card content from a seed (or from a partial set
of cards) and report per-card pass/fail plus the overall verdict.

See [Consensus-masked relative timelocks](#consensus-masked-relative-timelocks)
for the non-blocking `older()` advisory this command emits on intake.

### Synopsis

```sh
mnemonic verify-bundle --network <NETWORK> [OPTIONS] [--ms1 ...] [--mk1 ...] [--md1 ...]
```

### Flags

| Flag | Purpose |
|---|---|
| `--network <NETWORK>` | mainnet / testnet / signet / regtest |
| `--template <TEMPLATE>` | as for `bundle` |
| `--descriptor <DESCRIPTOR>` | user-supplied descriptor; accepts either a BIP-388 `@N` template (keys supplied via `--slot`) **or a bare concrete descriptor** with inline `[fp/path]xpub` keys (watch-only output); **(v0.57.0) or a BIP-388 wallet-policy JSON** `{name, description_template, keys_info}` (auto-detected by a leading `{`), expanded to the concrete descriptor before verifying — parity with `bundle`/`export-wallet --descriptor`; both apostrophe and `h`-form hardened paths are accepted |
| `--descriptor-file <DESCRIPTOR_FILE>` | descriptor read from file (also accepts a BIP-388 wallet-policy JSON, same as `--descriptor`) |
| `--threshold <THRESHOLD>` | multisig threshold |
| `--multisig-path-family <FAMILY>` | bip48 or bip87 |
| `--privacy-preserving` | match a privacy-preserving mk1 emission |
| `--language <LANGUAGE>` | BIP-39 wordlist |
| `--passphrase <PASSPHRASE>` | BIP-39 mnemonic passphrase |
| `--passphrase-stdin` | read `--passphrase` from stdin (raw, NULL-byte preserving); single stdin per invocation |
| `--account <ACCOUNT>` | BIP-32 account index |
| `--origin <ORIGIN>` | (#28) explicit BIP-32 origin path (e.g. `m/84'/0'/7'`) for verifying + recomposing a **keyless single-sig template** bundle (`bundle --md1-form=template`); overrides the canonical `m/<purpose>'/<coin>'/<account>'` default, mirroring [`restore --origin`](#mnemonic-restore). Only meaningful for a keyless single-sig template bundle; ignored otherwise. See [Template-only md1](#template-only-md1) |
| `--expect-wallet-id <PREFIX>` | (#28) expected `WalletPolicyId` hex prefix for a keyless single-sig template bundle. When set, verify-bundle recomputes the id from the completed, fully-keyed, explicit-origin wallet and matches its leading bytes; a mismatch is a **failed check** (overall `mismatch`, exit 4). Only meaningful for a template bundle; ignored otherwise. **NOT** checked when `--origin` overrides the canonical account path (a different preimage). See [Template-only md1](#template-only-md1) |
| `--from <FROM>` | (#28 phase 2) the operator's OWN seed for completing a keyless **multisig / general** TEMPLATE bundle (`bundle --md1-form=template`, n≥2). Same grammar + semantics as [`restore --from`](#mnemonic-restore) (`ms1=` / `phrase=` / `entropy=` / `seedqr=`; `@env:VAR` or stdin). REQUIRED to complete a multisig template; ignored for a single-sig template or keyed wallet-policy bundle. The own key is derived at `--account` (a single own account for verify) honoring `--origin`. See [Multisig template completion](#multisig-template-completion) |
| `--cosigner <COSIGNER>` | (#28 phase 2) an UNASSIGNED cosigner key (`mk1` / xpub) for completing a keyless multisig / general TEMPLATE bundle; repeat per cosigner card. Same grammar + semantics as [`restore --cosigner`](#mnemonic-restore): the bare form is search-placed, `@N=<mk1\|xpub>` assigns it explicitly. Only meaningful with `--from` + a keyless multisig template. Distinct from `--mk1` (the engraved STUB cards the binding check validates) |
| `--own-account-max <OWN_ACCOUNT_MAX>` | (v0.70.0; #28 phase 2) RANGE fallback for the OWN seed's account when the exact account is unknown: derive the own seed at every account in `0..K` and let the multisig-template **own-account subset-search** select the account actually used (own-only — the `--cosigner` cards must be EXACT; over-supply cosigners with `--search-cosigner-subset`). NEW on `verify-bundle` (mirrors [`restore --own-account-max`](#mnemonic-restore)). Mutually exclusive with `--account` (clap `conflicts_with` — `--own-account-max K` ALONE passes; the scalar `--account` default is ignored). `K ≤ 256`. Threaded into the SAME shared completion engine `restore` uses (verify == restore). See [Subset-search / over-supply completion](#subset-search-over-supply-completion) |
| `--search-cosigner-subset` | (v0.70.0; #28 phase 2) **OPT-IN bounded cosigner-subset search.** By default (OFF) a multisig template completion requires the supplied `--cosigner` cards to be EXACT (own-only — over-supplying cosigners refuses). With this flag the operator MAY over-supply `--cosigner` cards (unsure which/how many cosigners belong); the search resolves the correct cosigner subset too. NEW on `verify-bundle` (mirrors [`restore --search-cosigner-subset`](#mnemonic-restore)). The space grows, so a LONGER `--expect-wallet-id` prefix may be needed; bounded by the §6 hard ceiling + the adaptive time-cap. Mutually exclusive with `--cosigner @N=`. Threaded into the SAME shared completion engine `restore` uses (verify == restore). See [Subset-search / over-supply completion](#subset-search-over-supply-completion) |
| `--search-address <SEARCH_ADDRESS>` | (#28 phase 2) a known receive (or change) ADDRESS of the wallet; triggers **address-search** for a multisig-template completion (mirrors [`restore --search-address`](#mnemonic-restore)). Recommended over `--expect-wallet-id` (full-scriptPubKey match — collision-free) |
| `--search-addr-min <SEARCH_ADDR_MIN>` | (#28 phase 2) inclusive lower address index for `--search-address` (default 0; mirrors `restore`) |
| `--search-addr-max <SEARCH_ADDR_MAX>` | (#28 phase 2) exclusive upper address index for `--search-address` (default 20; mirrors `restore`) |
| `--search-chain <SEARCH_CHAIN>` | (#28 phase 2) which BIP-32 change-chain branch(es) `--search-address` scans: `receive` (chain 0, default), `change` (chain 1), or `both` (mirrors `restore`) |
| `--accept-search-time <ACCEPT_SEARCH_TIME>` | (#28 phase 2) override the adaptive ~1-hour search-time ceiling for a multisig-template completion (mirrors [`restore --accept-search-time`](#mnemonic-restore)). Must be ≥ the printed exhaustive-time estimate (a forced acknowledgment). Humantime duration (e.g. `2h`, `90min`) |
| `--slot <SLOT>` | repeating slot input `@N.<subkey>=<value>`; subkeys mirror `mnemonic bundle --slot` (`phrase`, `seedqr`, `entropy`, `ms1`, `xpub`, `master_xpub`, `fingerprint`, `path`, `wif`, `xprv`); for secret-bearing subkeys `=-` reads from stdin. `seedqr` (v0.31.3+) decodes a 48- or 96-digit SeedQR string inline. `ms1` (v0.41.0+) decodes a raw BIP-93 codex32 secret inline (language-preserving; `--language` conflicting with the slot's wire language is refused with exit 2; a K-of-N share is rejected with a pointer to `ms-shares combine`), mirroring `mnemonic bundle --slot @N.ms1=`. |
| `--bundle-json <PATH>` | read the bundle from a JSON file emitted by `bundle --json` |
| `--ms1 <STRING>` | repeating; one ms1 card |
| `--mk1 <STRING>` | repeating; one mk1 card |
| `--md1 <STRING>` | repeating; one md1 card |
| `--json` | JSON output |
| `--help` | print help |

### Non-representable use-site steps

`verify-bundle` applies the same [use-site residue reject](#non-representable-use-site-steps)
as `bundle` / `import-wallet` when re-parsing a `--descriptor` /
`--descriptor-file` input, but the exit code differs by intake path: a
**concrete** descriptor (inline keys) rejects at exit 2
(`DescriptorParse`) *before* any comparison against the supplied cards
runs, while an **`@N`-template** descriptor (keys supplied via `--slot`)
rejects at exit 4 (`DescriptorReparseFailed`) when the completed wallet
is re-parsed.

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
| yes | no | Auto-fire attempts a correction: **mk1** (a full `chunk_set_id` group that reassembles cleanly) / **md1 chunked** (multi-chunk, or chunked-of-1 — e.g. the `mnemonic bundle` / `--md1-form=template` shape) — exit 5 + repair report on stderr, corrected chunk on stdout. **md1 non-chunked** (v0.86.0 — the plain `md encode` single-string form; no cross-chunk/content-id oracle, see [md1 non-chunked demotion](#mnemonic-repair-md1-non-chunked-demotion) below) and **ms1** (Cycle F) — NO short-circuit. The md1 non-chunked candidate falls through with a one-line stderr advisory pointing at `mnemonic repair --md1`; the ms1 corrected candidate is instead compared to the expected (typed) seed via the `ms1_decode` / `ms1_entropy_match` check rows below |
| yes | yes | Legacy VerifyCheck row + `result: mismatch` + exit 4 |
| no (pipe / redirected / CI) | no | Legacy VerifyCheck row + exit 4 (preserves automation contract) |
| no | yes | Legacy VerifyCheck row + exit 4 |

**Principled distinction across all four CLIs (SPEC §4):** exit-5
`REPAIR_APPLIED` means a correction is **verified now** (an mk1/**chunked**
md1 cross-chunk reassembly hash or content-id check; a unique
full-checksum `--max-indel` recovery) **or verifiable-by-reassembly
later** (an mk1 single-plate chunk under the standalone `mk repair` codec
CLI, once the rest of its set is supplied — note that `mnemonic repair`
*itself* instead demotes an incomplete mk1 group to exit-4, per the
Cycle-E note below). exit-4 `VERIFY-ME` means a bounded-distance BCH
SUBSTITUTION correction spent the checksum's error-detection budget and
has **no self-oracle** — this is ms1's case always, a **non-chunked**
md1 single-string correction (v0.86.0 — see [md1 non-chunked
demotion](#mnemonic-repair-md1-non-chunked-demotion) below), an
incomplete mk1 `chunk_set_id` group, or an ambiguous `--max-indel`
recovery. Exit-5 is never "an oracle verified it" — that phrasing is
false for the mk1 single-plate case, which is merely *not yet*
falsified.

**mk1 set-level re-verify (Cycle E):** for an `--mk1` / `--bundle-json`
mk1 payload, the "Auto-fire" outcome above (exit 5, corrected chunk
applied to stdout) only occurs when the FULL `chunk_set_id` group is
supplied AND the correction reassembles cleanly. If the correction
would alias to a different card (a complete group whose reassembly
fails), auto-fire does **not** apply it — the original decode error
surfaces instead (the Legacy VerifyCheck row / `result: mismatch`
outcome), never a silent short-circuit onto a wrong card. If only a
partial group is supplied, auto-fire likewise does not blindly apply
the correction (it cannot be set-verified); the original error
surfaces the same way. See [mk1 set-level
re-verify](#mnemonic-repair-mk1-set-level-reverify) under `mnemonic
repair` below.

**ms1 ground-truth compare (Cycle F funds fix):** ms1 is a single-string
bearer secret with no cross-chunk hash and no internal redundancy beyond
the BCH checksum itself, so — unlike mk1/**chunked** md1 — a
substitution-correction can never be confirmed by reassembly (a
**non-chunked** md1 shares ms1's structural gap here; see [md1
non-chunked demotion](#mnemonic-repair-md1-non-chunked-demotion) below,
v0.86.0); every other surface demotes it to
an unverified Candidate (see [ms1 substitution-correction
demotion](#mnemonic-repair-ms1-substitution-demotion) under `mnemonic
repair` below). Here, uniquely, the user's own TYPED seed
(`--slot @N.phrase=…` / `--slot @N.ms1=…`) is already the ground truth,
so `verify-bundle` feeds the corrected candidate into the existing
`ms1_decode` / `ms1_entropy_match` check-row comparison against it
instead of emitting a stderr advisory: **match** ⇒ both checks PASS
("recovered via auto-repair, confirmed against expected seed") and the
run can still finish `exit 0`; **mismatch** ⇒ `ms1_entropy_match` FAILS
("auto-repair candidate did not match the expected seed — this card is
not a card for this seed"), the full check table is still emitted, and
the run exits `4` — never a silent "recovered", never an abort. This
closes the wrong-bundle attack a derived-xpub oracle could not: a
corroded `--ms1` that happens to correct to a DIFFERENT wallet's seed is
caught by the comparison rather than accepted. No mk1/xpub derivation is
involved, and the compare is skipped identically to the mk1/md1 paths
when the corresponding slot is watch-only (no typed seed to compare
against).

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
| `convert` | `mk_codec` decode failure on `--from mk1=…` | Auto-fire (exit 5 + repair report) | Typed decode error (exit ≠ 5) |
| `convert` | `ms_codec` decode failure on `--from ms1=…` (Cycle F) | **NO short-circuit** — the original decode error surfaces (its own exit code) + a one-line stderr advisory pointing at `mnemonic repair --ms1` | Typed decode error (no advisory) |
| `inspect` | `mk_codec` decode failure on `--mk1`, or `md_codec` decode failure on a **chunked** `--md1` (multi-chunk, or chunked-of-1) | Auto-fire (exit 5 + repair report) | Typed decode error (exit ≠ 5) |
| `inspect` | `md_codec` decode failure on a **non-chunked** `--md1` (v0.86.0 — see [md1 non-chunked demotion](#mnemonic-repair-md1-non-chunked-demotion) below) | **NO short-circuit** — same advisory-and-fall-through shape as ms1 below, pointing at `mnemonic repair --md1` | Typed decode error (no advisory) |
| `inspect` | `ms_codec` decode failure on `--ms1` (Cycle F) | **NO short-circuit** — same advisory-and-fall-through as `convert` above | Typed decode error (no advisory) |
| `verify-bundle` | `mk_codec` decode failure, or `md_codec` decode failure on a **chunked** md1 (as above, plus the `--bundle-json` intake path) | Auto-fire (exit 5 + repair report; corrected chunk on stdout) | Legacy VerifyCheck row + exit 4 |
| `verify-bundle` | `md_codec` decode failure on a **non-chunked** md1 (v0.86.0, plus `--bundle-json`) | **NO short-circuit** — falls through to the Legacy VerifyCheck row + a one-line stderr advisory pointing at `mnemonic repair --md1` | Legacy VerifyCheck row + exit 4 |
| `verify-bundle` | `ms_codec` decode failure (as above, plus `--bundle-json`) (Cycle F) | `ms1_decode` / `ms1_entropy_match` check-row compare against the typed seed — PASS on match, `ms1_entropy_match` fail → exit 4 on mismatch (see [ms1 ground-truth compare](#auto-fire-on-decode-failure-v0221) above); NO stderr advisory on this path | Legacy VerifyCheck row + exit 4 |

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
| `--network <NETWORK>` | mainnet / testnet / signet / regtest. For `(Xpub, Address)` derivation, an explicit `--network` disagreeing with the xpub's own version bytes is refused fail-closed (exit 2) rather than rendering a wrong-network address; omit `--network` to infer it from the xpub instead |
| `--template <TEMPLATE>` | as for `bundle` |
| `--path <PATH>` | derivation path override |
| `--account <ACCOUNT>` | account index (default 0) |
| `--language <LANGUAGE>` | BIP-39 wordlist |
| `--passphrase <PASSPHRASE>` | BIP-39 passphrase |
| `--passphrase-stdin` | read `--passphrase` from stdin (raw, NULL-byte preserving); BIP-38 V3 use case |
| `--bip38-passphrase <BIP38_PASSPHRASE>` | distinct BIP-38 Scrypt passphrase channel (v0.8 BREAKING — separate from `--passphrase`). On a composite `(seedqr\|phrase\|entropy)→bip38` edge, `--bip38-passphrase` is **required**; an unset value is refused (it would otherwise encrypt the BIP-38 layer with the empty passphrase, since `--passphrase` feeds only BIP-39 PBKDF2). Pass `--bip38-passphrase ""` to deliberately use an empty BIP-38 passphrase. |
| `--bip38-passphrase-stdin` | read `--bip38-passphrase` from stdin (raw, NULL-byte preserving); closes the BIP-38 V3 spec NULL-byte passphrase argv gap |
| `--electrum-version <ELECTRUM_VERSION>` | Electrum seed-version selector for `(Entropy, ElectrumPhrase)` |
| `--electrum-language <ELECTRUM_LANGUAGE>` | Electrum-specific wordlist (English + 4 non-English) |
| `--fingerprint <FINGERPRINT>` | master fingerprint (input on certain edges) |
| `--xpub-prefix <XPUB_PREFIX>` | SLIP-0132 prefix selector for emitted xpubs (`xpub`/`ypub`/`Ypub`/`zpub`/`Zpub`; requires `--network`). **(v0.58.1)** Reading an `mk1` card (`--from mk1= --to xpub`) prints a non-blocking stderr note naming the SLIP-0132 variant the card's derivation path conventionally implies (e.g. `m/84'` → zpub) and pointing here — stdout stays the BIP-32-neutral xpub. The mk1 card stores only the neutral xpub (the variant is normalized away on intake and is not recoverable exactly); pass `--xpub-prefix <variant>` to emit the SLIP-0132 form. |
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

See [Consensus-masked relative timelocks](#consensus-masked-relative-timelocks)
for the non-blocking `older()` advisory this command emits on intake.

### Synopsis

```sh
mnemonic export-wallet [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--template <TEMPLATE>` | as for `bundle` |
| `--descriptor <DESCRIPTOR>` | accepts a concrete descriptor (with or without key origins); **(v0.49.0) also a BIP-388 wallet-policy JSON** `{name, description_template, keys_info}` (auto-detected by a leading `{`), expanded to the concrete descriptor — the inverse of `--format bip388`; a keyless `@N` template is rejected with a pointer to `--template … --slot …` or `--from-import-json` |
| `--threshold <THRESHOLD>` | multisig threshold |
| `--multisig-path-family <FAMILY>` | bip48 or bip87 |
| `--network <NETWORK>` | default mainnet. Every resolved `--slot` xpub (and `@N.master_xpub=`) must agree with `--network`'s version-byte family, and a concrete `--descriptor`'s extended keys must agree with it for `--format bsms` 4-line's first-address line — a disagreement (including against the clap default, with no explicit flag) is refused fail-closed (exit 2) rather than re-emitting a wrong-network xpub / minting a wrong-network address |
| `--language <LANGUAGE>` | ignored (watch-only); accepted for slot-parser symmetry |
| `--account <ACCOUNT>` | account index (default 0) |
| `--slot <SLOT>` | repeating `@N.<subkey>=<value>`; subkeys: `phrase`, `seedqr`, `entropy`, `xpub`, `master_xpub`, `fingerprint`, `path`, `wif`, `xprv` (secret-bearing subkeys, including `seedqr`, are refused by `export-wallet`'s watch-only validator per SPEC §3) |
| `--format <FORMAT>` | `bitcoin-core` (default) / `bip388` / `coldcard` / `jade` / `sparrow` / `specter` / `electrum` / `green` / `bsms` (v0.27.0) / `descriptor` (v0.42.0) — bare canonical descriptor string + BIP-380 checksum (`<descriptor>#<checksum>`), no wallet-file wrapper |
| `--output <OUTPUT>` | output path (`-` = stdout, default) |
| `--range <RANGE>` | Bitcoin Core `range` field; comma-separated; default `0,999` |
| `--timestamp <TIMESTAMP>` | Bitcoin Core `timestamp` field; `0` (default; rescan from genesis to discover an existing key's funds), `now`, or unix seconds |
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

## `mnemonic restore` {#mnemonic-restore}

Take **secret seed material + an optional BIP-39 passphrase** and emit
a **watch-only "restore document"**: a verification block leading with
the master fingerprint (the passphrase-correctness oracle) and the first
receive address(es), followed by the concrete single-sig descriptor(s)
for BIP-44/49/84/86. Restore is **read-only / watch-only-out** — it
emits xpub / fingerprint / addresses / descriptor only and NEVER any
private material (`xprv` / WIF). It does NOT sign: the toolkit stops at
key material and read-only derivation (see
[Watch-only operation](../30-workflows/34-watch-only.md)).

See [Consensus-masked relative timelocks](#consensus-masked-relative-timelocks)
for the non-blocking `older()` advisory `restore --md1` emits on intake.

**Two modes.** *Single-sig* (the default, `--from <seed>`) emits the
BIP-44/49/84/86 descriptors. *Multisig-cosigner* (v0.44.0, `--md1 <card>`)
reconstructs the concrete watch-only **multisig** descriptor from the shared
wallet-policy `md1` card **alone** — the card already carries every cosigner's
public key, so `--from`/`--cosigner` are *optional cross-check* inputs, not
build inputs. Multisig mode covers `wsh`, `sh(wsh)`, **NUMS taproot**
multisig (`tr-multi-a` / `tr-sortedmulti-a`), (v0.55.1) **general
NUMS-taproot policies** with a single script leaf or a depth-1 two-leaf tap
tree, and (v0.55.3) **non-NUMS key-path taproot** — a real cosigner key at
the trunk — for general single-leaf/depth-1 policies and distinct-trunk
multisig. A **keyless multisig / general TEMPLATE `md1`** (no concrete
keys) is *not* a refusal — it is **completed** by re-supplying the keys
(see [Multisig template completion](#multisig-template-completion)). Still
refused (exit 2): the `@-in-both` shape (the trunk key is *also* a leaf
key) and a depth-≥2 tap tree.

### Synopsis

```sh
# single-sig
mnemonic restore --from <node>=<value> [--template <TEMPLATE>] [OPTIONS]
# multisig-cosigner (reconstruct from the shared md1; cross-check is optional)
mnemonic restore --md1 <card> [--md1 <card> …] \
    [--from <node>=<value>] [--cosigner @N=<mk1|xpub> …] [OPTIONS]
```

`<node>` is one of `ms1` / `phrase` / `entropy` / `seedqr` (seed-bearing
only — a non-seed node such as `xpub=` / `xprv=` / `wif=` is refused with
exit 1). The value supports `@env:VAR` and `-` (stdin), the two secret
channels that keep the seed off the argv.

### Flags

| Flag | Purpose |
|---|---|
| `--from <FROM>` | seed source `ms1=<v>` / `phrase=<v>` / `entropy=<hex>` / `seedqr=<digits>`; value supports `@env:VAR` and `-` (stdin). Non-seed nodes (`xpub` / `xprv` / `wif` / …) are refused (restore needs a master secret). REQUIRED for single-sig restore **and for multisig-template completion** (the OWN seed); OPTIONAL in keyed-multisig (`--md1`) mode, where it cross-checks the own cosigner position (inferred by matching the derived key against the md1's slots). See [Multisig template completion](#multisig-template-completion) |
| `--md1 <MD1>` | (v0.44.0; multisig mode) the shared wallet-policy `md1` card chunk(s) — reconstructs the concrete watch-only multisig descriptor from the card alone. **(#28 phase 2) also accepts a keyless multisig / general TEMPLATE `md1`** (`bundle --md1-form=template`), completed via `--from` + `--account` + `--cosigner` (see [Multisig template completion](#multisig-template-completion)). Repeat for chunked cards. `wsh` / `sh(wsh)`, taproot NUMS multisig (`tr-multi-a` / `tr-sortedmulti-a`), (v0.55.1) general NUMS-taproot policies up to a depth-1 two-leaf tap tree, and (v0.55.3) non-NUMS key-path taproot (a real cosigner trunk key) for general single-leaf/depth-1 + distinct-trunk multisig; the `@-in-both` shape (trunk key also a leaf key) or a depth-≥2 tap tree is refused (exit 2). Watch-only (non-secret) |
| `--cosigner <@N=KEY>` | (v0.44.0; multisig mode) cross-check assertion `@N=<mk1-chunk\|xpub>` — cosigner at position `N` is this public key. Repeat the same `@N=` for each chunk of a multi-chunk `mk1`. A mismatch against the md1's slot is a hard error (exit 4) unless `--allow-mismatch`. **(#28 phase 2) for multisig-template completion** the bare form (`--cosigner <mk1>`, no `@N=`) supplies an UNASSIGNED cosigner the search places; the `@N=` form assigns it explicitly. Watch-only (non-secret) |
| `--passphrase <PASSPHRASE>` | BIP-39 mnemonic-extension passphrase; `@env:VAR` supported. Empty (default) = no passphrase |
| `--passphrase-stdin` | read the BIP-39 passphrase from stdin (conflicts with `--passphrase`; mutually exclusive with `--from <node>=-`) |
| `--language <LANGUAGE>` | BIP-39 wordlist for `phrase=` / `seedqr=` (default `english`); one of `english` / `simplifiedchinese` / `traditionalchinese` / `czech` / `french` / `italian` / `japanese` / `korean` / `portuguese` / `spanish`. A `mnem`-kind ms1 carries its own wire language; a conflicting `--language` is refused |
| `--network <NETWORK>` | `mainnet` (default) / `testnet` / `signet` / `regtest` |
| `--account <ACCOUNT>` | BIP-32 account index(es) (default 0). Single-sig restore + single-sig template completion: one account (the first value). **(#28 phase 2) MULTISIG template completion**: the comma-separated LIST of accounts the OWN seed is used at — one own key per account (e.g. `--account 0,1,2,3` for a 4-own-slot policy); the search places each own-derived key. See [Multisig template completion](#multisig-template-completion) |
| `--origin <ORIGIN>` | (#28) explicit BIP-32 origin path (e.g. `m/84'/0'/7'`) for completing a **keyless single-sig template** `md1` (`bundle --md1-form=template`); overrides the template's canonical `m/<purpose>'/<coin>'/<account>'` default. Only meaningful for keyless single-sig template restore; ignored otherwise. See [Template-only md1](#template-only-md1) |
| `--expect-wallet-id <PREFIX>` | (#28) expected `WalletPolicyId` hex prefix for template-completion (single-sig phase 1 **and** multisig phase 2 id-search). Restore recomputes the id from the completed, fully-keyed wallet and matches its leading bytes; a **mismatch refuses loudly** (exit 4). Any-length prefix (an advisory warns when shorter than 4 bytes — a collision footgun — but does not enforce it; the convenience prefix the `bundle` advisory prints is 4 bytes). For multisig the prefix must be **strong** (sized to the realized search space) or the search refuses an ambiguous match. **NOT** checked when `--origin` is supplied (a different preimage). See [Template-only md1](#template-only-md1) and [Multisig template completion](#multisig-template-completion) |
| `--own-account-max <OWN_ACCOUNT_MAX>` | (v0.70.0; #28 phase 2) RANGE fallback for the OWN seed's account(s) when the exact accounts are unknown: derive the own seed at **every** account in `0..K` and let the multisig-template **own-account subset-search** select the subset actually used. Own-only — the supplied `--cosigner` cards must be EXACT (over-supply cosigners with `--search-cosigner-subset`). Mutually exclusive with `--account` (clap `conflicts_with` — `--own-account-max K` ALONE passes; the `--account` default is ignored). `K ≤ 256`. The realized search space sizes the strong-prefix requirement, so a LONGER `--expect-wallet-id` prefix (or `--search-address`) is needed than for the exact-account path. See [Subset-search / over-supply completion](#subset-search-over-supply-completion) |
| `--search-cosigner-subset` | (v0.70.0; #28 phase 2) **OPT-IN bounded cosigner-subset search.** By default (OFF) a multisig template completion requires the supplied `--cosigner` cards to be EXACT (own-only — over-supplying cosigners refuses). With this flag the operator MAY over-supply `--cosigner` cards (unsure which/how many cosigners belong); the search resolves the correct cosigner subset too. The space grows to `S_opt = Σ_j C(K_own,j)·C(M_sup,N−j)·N!`, so a LONGER `--expect-wallet-id` prefix is needed (a too-short prefix refuses; `--search-address` is the recommended collision-free mode for large opt-in pools). Bounded by the §6 hard ceiling (`S_opt ≤ 1e15`) + the adaptive time-cap. Mutually exclusive with `--cosigner @N=` (explicit placement). Composes with `--own-account-max` / `--account`. See [Subset-search / over-supply completion](#subset-search-over-supply-completion) |
| `--search-address <SEARCH_ADDRESS>` | (#28 phase 2) a known receive (or change) ADDRESS of the wallet; triggers **address-search** for a multisig-template completion — the search finds the unique key→slot assignment whose scriptPubKey at some `(chain, index)` in the range equals this address's. Recommended over `--expect-wallet-id` (full-scriptPubKey match — collision-free). See [Multisig template completion](#multisig-template-completion) |
| `--search-addr-min <SEARCH_ADDR_MIN>` | (#28 phase 2) inclusive lower address index for `--search-address` (default 0) |
| `--search-addr-max <SEARCH_ADDR_MAX>` | (#28 phase 2) exclusive upper address index for `--search-address` (default 20). Deepen (`0..20`, then `20..40`, …) if the target is not found; a narrow range expresses "I know the index" |
| `--search-chain <SEARCH_CHAIN>` | (#28 phase 2) which BIP-32 change-chain branch(es) `--search-address` scans: `receive` (chain 0, the **default**), `change` (chain 1), or `both` (doubles the per-index search cost) |
| `--accept-search-time <ACCEPT_SEARCH_TIME>` | (#28 phase 2) override the adaptive ~1-hour search-time ceiling for a multisig-template completion. Must be ≥ the tool's printed estimated exhaustive time (a forced acknowledgment). Accepts a humantime duration (e.g. `2h`, `90min`) |
| `--template <TEMPLATE>` | restrict to a single wallet type (`bip44` / `bip49` / `bip84` / `bip86`); omit = emit all four. A multisig template is refused (restore is single-sig) |
| `--expect-fingerprint <EXPECT_FINGERPRINT>` | reference master fingerprint (8 lowercase hex); mismatch → exit 4 (unless `--allow-mismatch`) |
| `--expect-xpub <EXPECT_XPUB>` | reference account xpub (requires `--template`); mismatch → exit 4 (unless `--allow-mismatch`) |
| `--allow-mismatch` | emit descriptors even when a reference does not match (loud `✗ MISMATCH (overridden)` banner, exit 0) |
| `--count <COUNT>` | number of first-receive addresses to show per wallet type (default 1) |
| `--format <FORMAT>` | emit an importable wallet-software payload via an `export-wallet` emitter (`descriptor`, `bitcoin-core`, `bip388`, `coldcard`, `sparrow`, `specter`, `jade`, `electrum`, `green`, `bsms`, …). REQUIRES a single `--template` (one-descriptor-in/one-out); `--format` with no `--template` → exit 2. When set, the importable payload goes to stdout and the verification block goes to stderr so the payload pipes cleanly. With `--json` the payload is embedded as the `import_payload` field instead |
| `--json` | emit a single structured JSON object on stdout instead of the text document; seed material is NEVER echoed (redacted by construction). `import_payload` is present only when `--format` is also set |
| `--output <OUTPUT>` | write the stdout content to `<FILE>` (`-`, the default, → stdout); the verification block / banners / advisory still go to stderr |
| `--no-auto-repair` | (global) skip auto-fire repair on decode failures; same global flag honored by `convert` / `inspect` / `verify-bundle` |
| `--help` | print help |

### Verification policy

Restore is built around the master fingerprint as the
**passphrase-correctness oracle**:

- **Reference present** (`--expect-fingerprint` / `--expect-xpub`) and
  the derived material **matches** → emit, exit 0.
- **Reference present and it does NOT match** → **hard error, exit 4**
  (`RestoreMismatch`); the verification block prints derived-vs-expected
  under a `✗ MISMATCH` banner and **no descriptors are emitted**. This is
  the wrong-passphrase / wrong-seed guard.
- **`--allow-mismatch`** override → emit the descriptors the supplied
  seed+passphrase produced under a loud `✗ MISMATCH (overridden)` stderr
  banner, exit 0.
- **No reference at all** → emit, with a loud `UNVERIFIED` stderr banner
  pointing at the fingerprint to verify against your own records.

### Worked example

Single-sig BIP-84 restore from the public zero-entropy test seed
(`abandon` × 11 + `about`), with the fingerprint hard-gated against a
known reference:

```sh
seed="abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon about"
printf '%s' "$seed" |
  mnemonic restore --from phrase=- --template bip84 \
    --expect-fingerprint 73c5da0a
```

Stdout:

```{.text include="41-restore-bip84.out"}
PLACEHOLDER — generated from transcripts/41-restore-bip84.out at build
```

Stderr carries the watch-only advisory (`note: stdout is watch-only —
public keys only, cannot spend`). Piping the seed via `--from phrase=-`
keeps it off the argv; passing `--from phrase="$seed"` inline instead
raises a `/proc/$PID/cmdline` argv-leakage advisory on stderr.

**All four wallet types** (omit `--template`):

```sh
printf '%s' "$seed" | mnemonic restore --from phrase=-
```

emits a `bip44` / `bip49` / `bip84` / `bip86` block each (same
master fingerprint, four descriptors + first-recv addresses).

**The hard-gate (exit 4) on a wrong reference:**

```sh
printf '%s' "$seed" |
  mnemonic restore --from phrase=- --template bip84 \
    --expect-fingerprint deadbeef ; echo "exit=$?"
```

prints `✗ MISMATCH` + `error: restore: fingerprint mismatch — derived
73c5da0a, expected deadbeef` on stderr, **emits no descriptor**, and
`exit=4`. Add `--allow-mismatch` to override (descriptors emitted under
`✗ MISMATCH (overridden)`, exit 0) — only when you know the reference
itself is wrong.

**Passphrase via stdin**, seed via `@env:` (both secret channels — the
TREZOR-passphrase wallet has a *different* fingerprint, `b4e3f5ed`):

```sh
export RSEED="$seed"
printf 'TREZOR' |
  mnemonic restore --from phrase=@env:RSEED --template bip84 \
    --passphrase-stdin --expect-fingerprint b4e3f5ed
```

`--passphrase-stdin` and `--from <node>=-` cannot both read stdin in one
invocation; use `@env:` for one of the two channels (as above) when you
need both off the argv.

**Importable payload** (`--format`) — the payload pipes from stdout, the
verification block goes to stderr:

```sh
printf '%s' "$seed" |
  mnemonic restore --from phrase=- --template bip84 \
    --format descriptor --expect-fingerprint 73c5da0a
```

Stdout is the bare BIP-380 descriptor
`wpkh([73c5da0a/84'/0'/0']xpub6CatW…/<0;1>/*)#hpg6d6w2`. With `--json`
the payload is embedded as the `import_payload` field of the structured
object.

**Structured output** (`--json`):

```sh
printf '%s' "$seed" |
  mnemonic restore --from phrase=- --template bip84 \
    --expect-fingerprint 73c5da0a --json
```

emits a single object:

```{.text include="41-restore-bip84-json.out"}
PLACEHOLDER — generated from transcripts/41-restore-bip84-json.out at build
```

The seed is never echoed in any output mode (redacted by construction);
no `xprv` / `tprv` token appears in restore's stdout or `--json`.

### Multisig-cosigner restore (`--md1`) {#multisig-cosigner-restore}

A wallet-policy `md1` card (the multisig descriptor card the toolkit emits
for any multisig bundle) carries **every cosigner's public key**, so the
concrete watch-only multisig descriptor is reconstructible from the card
alone — you do not need any seed to recover the *watch-only* wallet:

```sh
mnemonic restore --md1 md1f5przzs... --md1 md1f5przzs...   # all chunks
```

emits the concrete 2-of-3 descriptor, a first receive address, and a
per-cosigner table, with a loud `UNVERIFIED` stderr banner (nothing was
cross-checked):

```text
2-of-3 multisig restore
CONFIRM: verify each cosigner fingerprint against your records before importing.
  descriptor: wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub6.../<0;1>/*,[b8688df1/87'/0'/0']xpub6.../<0;1>/*,[28645006/87'/0'/0']xpub6.../<0;1>/*))#yjp7hj7w
  first recv: bc1q...
  cosigner @0: 73c5da0a [87'/0'/0']  from md1 (not independently verified)
  cosigner @1: b8688df1 [87'/0'/0']  from md1 (not independently verified)
  cosigner @2: 28645006 [87'/0'/0']  from md1 (not independently verified)
```

**Cross-checking** is optional and *per-position*. Add `--from <your seed>`
to prove which cosigner is yours (the position is inferred by matching the
derived key against the md1 slots), and/or `--cosigner @N=<mk1|xpub>` to
assert another cosigner's key. Only the positions you actually supply are
marked verified; the rest stay `from md1 (not independently verified)`, and
the verdict is `PARTIAL` until **every** position is cross-checked:

```sh
mnemonic restore --md1 md1f5przzs... \
    --from phrase=- --cosigner @1=mk1qp... --cosigner @2=xpub6...
```

A supplied key (own seed or `--cosigner`) that does **not** match the md1's
slot is a hard error (`✗ MISMATCH`, exit 4, `RestoreMismatch`) unless
`--allow-mismatch`. Restore stays watch-only-out in multisig mode too: no
`xprv` / WIF / seed reaches stdout, stderr, or `--json`.

**General wallet policies (v0.54.0; taproot v0.55.1).** Beyond plain
`multi`/`sortedmulti`, an `md1` for a *general* policy — timelocks
(`older`/`after`), hashlocks (`sha256`/`hash256`/`ripemd160`/`hash160`),
`andor`/`and_v`/`or_*`/`thresh`, or a decaying-quorum vault — reconstructs
**faithfully**, preserving the full policy (it is no longer collapsed to
plain multisig). These are labeled `miniscript policy restore (N cosigners)`
(and `--json` `wallet_type: "miniscript-policy"` with a null top-level
`threshold`, since a general policy has no single k-of-n threshold).
Descriptor-driven `--format`s (`bitcoin-core` / `descriptor`) emit the
faithful descriptor (`bsms` for the non-taproot arms; `bip388` for a
multipath `/<0;1>/*` non-taproot card — it refuses a wildcard-only one);
template-requiring k-of-n formats refuse. (A policy whose keys appear as
bare `pk(@N)`/`pkh(@N)` *outside* a `multi()` is refused with a clear message
pending a sibling md-codec rendering fix; likewise a card with a hardened
wildcard `/*h` (or a hardened per-cosigner override), or with taproot
per-cosigner use-site overrides, is refused rather than mis-rendered —
non-taproot non-hardened per-cosigner use-site overrides reconstruct faithfully
since v0.58.2. The engraved card remains a faithful backup either way.)

Since v0.55.1 the general arm also covers **NUMS taproot** policies whose tap
tree is a single general miniscript leaf (e.g. `tr(NUMS,and_v(v:pk(K),
after(N)))`) or a depth-1 **two-leaf** tree (e.g. `tr(NUMS,{pk(K0),pk(K1)})`,
including a `multi_a` leaf alongside another leaf). The reconstructed
descriptor prints the NUMS internal key as its x-only **hex**
(`50929b74…803ac0`), not the engraving-side literal `NUMS` token — the two
spellings are the same key. Deeper trees (**≥3 leaves / depth ≥2**) are
refused (exit 2) until an upstream miniscript printing fix ships, and a
`sortedmulti_a` leaf *inside* a multi-leaf tree is refused (exit 2) pending a
sibling md-codec rendering fix — the engraved card remains a faithful backup
in both cases.

Since v0.55.3 the trunk (internal) key may also be a **real cosigner key**
(non-NUMS, "key-path") rather than the NUMS sentinel: a general single-leaf /
depth-1 policy (e.g. `tr(D,and_v(v:pk(K),older(N)))`) and a distinct-trunk
multisig (e.g. `tr(D,multi_a(2,K0,K1))`, trunk `D` not one of the leaf keys)
both reconstruct faithfully, with the live key-path internal key rendered as
its x-only xpub. The one shape that stays **refused** (exit 2) is `@-in-both`
— the trunk key is *also* one of the leaf keys (e.g. `tr(@0,multi_a(2,@0,@1))`)
— because reconstructing it needs a leaf-membership-aware rebuild not yet
supported; the toolkit refuses rather than emit a silently-different multisig
(FOLLOWUP `restore-non-nums-tr-internal-key-also-in-leaf`). The engraved card
remains a faithful backup.

**Importable wallet payloads (`--format`).** As with single-sig restore,
`--format <X>` emits an importable wallet-software payload instead of the
plain descriptor doc — the same payload class as `mnemonic export-wallet
--template <multisig> --format <X>`, built from the reconstructed cosigner
keys + threshold. The payload pipes from stdout while the verification doc
(descriptor + cosigner table) moves to stderr:

```sh
mnemonic restore --md1 md1f5przzs... --format bitcoin-core > import.json
```

Supported for plain k-of-n cards: `bitcoin-core`, `bip388`, `coldcard`,
`coldcard-multisig`, `jade`, `sparrow`, `electrum`, `bsms`, `descriptor`
(general-policy cards narrow this — descriptor-driven formats emit, k-of-n
template formats refuse; see the general-policies section above). `specter`
is refused (it needs a wallet name, which multisig restore does not take)
and `green` is refused — for k-of-n cards because Green has no file-import
multisig support (identically to `export-wallet`), and (v0.55.1) for a
general taproot policy card explicitly (NUMS or non-NUMS), because Green's
file-import surface is singlesig-only and must not receive a script-tree
policy dressed as singlesig. `bip388` likewise refuses a **general** taproot
card (a tap-script-tree reconstructed via the general route-around has no
named-template form): such a card emits `descriptor` / `bitcoin-core` only.
A **distinct-trunk taproot multisig** card (NUMS *or* non-NUMS), by contrast,
takes the template path and *does* emit `bip388` faithfully — `tr(@idx/**,
multi_a(k,…))`. No `--template` is needed (the threshold and script type come
from the `md1`). A cross-check `✗ MISMATCH` still hard-fails (exit 4)
**before** any payload is emitted unless `--allow-mismatch`.

**Scope.** `wsh`, `sh(wsh)`, **NUMS taproot** (`tr-multi-a` /
`tr-sortedmulti-a`) multisig, (v0.55.1) **general NUMS-taproot policies**
up to a depth-1 two-leaf tap tree (single general leaf or two leaves;
reconstructed with the NUMS H-point hex internal key), and (v0.55.3)
**non-NUMS key-path taproot** — a real cosigner key at the trunk — for
general single-leaf/depth-1 policies and distinct-trunk multisig. The
`@-in-both` shape (the trunk key is *also* a leaf key) is refused (exit 2,
citing `restore-non-nums-tr-internal-key-also-in-leaf`), as are a depth-≥2
(≥3-leaf) tap tree and a `sortedmulti_a` leaf inside a multi-leaf tree (both
exit 2, each citing its tracking slug). A **keyless multisig / general
TEMPLATE `md1`** (no concrete keys, `bundle --md1-form=template`) is
**completed** rather than refused — see [Multisig template
completion](#multisig-template-completion). A non-NUMS **general** tr
emits `descriptor` / `bitcoin-core` only (`bip388` / `green` refused), while a
non-NUMS distinct-trunk **multisig** also emits `bip388`. `--template` and
`--expect-xpub` are single-sig only.

---

## `mnemonic import-wallet`

Import a third-party wallet blob into an m-format bundle. Parses a
foreign wallet export (BSMS Round-2 per BIP-129, or Bitcoin Core's
`listdescriptors` JSON), reconstructs the equivalent watch-only
bundle, and round-trips it back through the toolkit canonicalizer
to surface byte-exact vs semantic-only equivalence (see [foreign
wallet formats](#foreign-wallet-formats) for the format taxonomy).

See [Consensus-masked relative timelocks](#consensus-masked-relative-timelocks)
for the non-blocking `older()` advisory this command emits on intake.

v0.26.0 ships two source formats — `bsms` and `bitcoin-core` —
selectable via `--format` or auto-detected by sniff. Both formats
are watch-only by design; the resulting bundle's cosigners carry no
secret material unless the user supplies an `--ms1` / `--slot
@N.phrase=` seed overlay (see [seed overlay](#mnemonic-import-wallet-seed-overlay)).
Bitcoin Core blobs containing `xprv` extended private keys are
refused (re-run `bitcoin-cli listdescriptors` without the `true`
flag to obtain xpub-only output).

Because import-wallet engraves an `md1` from the imported descriptor, it
emits the same non-blocking advisories `bundle` does when the descriptor
carries a [consensus-masked `older()`](#consensus-masked-relative-timelocks)
or an [unrestorable shape](#unrestorable-shapes). It is also subject to
the same [use-site residue reject](#non-representable-use-site-steps): a
descriptor whose key placeholder is followed by a fixed derivation step
(`/0/*`) — rather than the multipath `/<a;b>/*` (or bare `/*`, or the
BIP-388 `/**` shorthand, which is expanded to `/<0;1>/*` and accepted) —
is refused (exit 2) instead of silently collapsed.
Bitcoin Core's receive/change split export is the main surface this
would affect — but `--format bitcoin-core` **auto-recombines** a
same-key receive/change pair into one `/<0;1>/*` multipath bundle before
this check runs, so a standard split Core export imports cleanly; see
[foreign wallet formats](#foreign-wallet-formats) for the guard matrix.

### Synopsis

```sh
mnemonic import-wallet --blob <FILE|-> [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--blob <FILE\|->` | path to the third-party wallet blob; `-` reads from stdin (required UNLESS `--bsms-round1` is supplied as a standalone Round-1 verify mode) |
| `--no-auto-repair` | (global) skip auto-fire repair on decode failures; same global flag honored by `convert` / `inspect` / `verify-bundle` |
| `--format <bitcoin-core\|bsms\|coldcard\|coldcard-multisig\|descriptor\|electrum\|jade\|sparrow\|specter>` | format override; if absent, auto-detected via sniff (SPEC §6). **`descriptor` (v0.58.0)** reads a watch-only descriptor from a text file, tolerating leading `#`-comment lines + blank lines (so it accepts `export-wallet --format green` / `--format descriptor` output and any hand-written/foreign commented descriptor); singlesig **and** multisig; the BIP-380 checksum is validated if present (tolerated if absent). `descriptor` is **explicit-only** — it is never auto-sniffed (a bare descriptor is too generic), so `--format descriptor` is required |
| `--select-descriptor <N\|active-receive\|active-change\|all>` | multi-descriptor selector for Bitcoin Core blobs (SPEC §5.3); accepts integer index, `active-receive`, `active-change`, or `all` (default); BSMS blobs coerce non-default values to `all` with stderr NOTICE |
| `--ms1 <STRING>` | seed overlay (SPEC §8.3): supply the secret material that matches the blob's declared xpub at the cosigner's origin path; repeatable + positional cosigner-index — the i-th `--ms1` applies to cosigner i; cosigners not addressed by any `--ms1[N]` flag remain watch-only (no entropy attached); accepts the `@env:VAR` sentinel; empty-string `""` preserves the v0.25.1 watch-only sentinel |
| `--slot <@N.phrase=<phrase>>` | per-slot seed overlay; equivalent to `--ms1` but the phrase is converted to entropy and the derived xpub at the cosigner's origin path is compared against the blob's xpub; mutually exclusive with `--ms1[N]` for the same N; accepts `@env:VAR`; only the `phrase` subkey is accepted on `import-wallet` in v0.26.0 |
| `--json` | emit a JSON envelope array on stdout (SPEC §7.4) instead of the human-readable summary; the v0.27.0 envelope `bundle` field is the full toolkit-native `BundleJson` shape (was a parse-side summary in v0.26.0; the v0.27.0 wire-shape replacement is documented in CHANGELOG `### Changed`) plus a new top-level `schema_version: "1"` field |
| `--bsms-encryption-token <FILE\|->` | (v0.31.0) BIP-129 §Encryption decrypt token; reads session TOKEN from PATH (or `-` for stdin); applies PBKDF2-SHA512 + AES-256-CTR + HMAC-SHA256 per BIP-129 §Encryption. Combine with `--format bsms` to decrypt encrypted Round-2 wallet shares (`--blob`), **OR (v0.32.1)** with `--bsms-round1` to decrypt encrypted Round-1 key records. **(v0.32.2) repeatable** (BIP-129 line 74: one shared TOKEN or one per Signer): a SINGLE `--bsms-encryption-token` is SHARED — it decrypts every encrypted Round-1 record AND the Round-2 blob (backward-compatible). MULTIPLE tokens are paired POSITIONALLY with `--bsms-round1` records (the Nth token decrypts the Nth record); per-Signer mode requires every `--bsms-round1` record to be encrypted, the token count to equal the record count, and NO encrypted Round-2 `--blob` in the same invocation (a single Round-2 share carries a single token → supplying multiple tokens with an encrypted blob is refused). Token file contents: lowercase ASCII hex (16 chars STANDARD, 32 chars EXTENDED); whitespace stripped; uppercase normalized. At most one token may read from stdin (`-`). Encrypted Round-2 blobs lack the `BSMS 1.0` header so `--format bsms` is REQUIRED for the encrypted Round-2 path. MAC verify failure → exit 2 (typed `BsmsMacMismatch`). |
| `--bsms-round1 <FILE>` | (v0.27.0) BIP-129 Round-1 key record (Signer → Coordinator) for BIP-322 ECDSA signature verification; repeating flag — one per record; each record verified independently; verify state propagates to `--json` envelope's `bsms_round1_verifications` field; standalone mode (no `--blob` supplied) emits per-record verify envelope and exits 0 when every record verifies. v0.27.0 accepts a file path only — stdin form `-` is rejected, supply a file path per record (FOLLOWUP: multi-record stdin intake). **(v0.32.1)** the record file may be EITHER plaintext (5-line `BSMS 1.0\n…`) OR an ENCRYPTED Round-1 wire (hex `MAC \|\| ciphertext`); encrypted records are auto-detected (raw hex, no `BSMS 1.0` header) and decrypted with `--bsms-encryption-token` (MAC-verified per BIP-129 §Encryption) before the BIP-322 verify. An encrypted record supplied without `--bsms-encryption-token` → `BadInput` (exit 1); MAC verify failure → exit 2 (`BsmsMacMismatch`). **(v0.85.0)** in the default lenient mode, if ANY record's `signature_verified` is `false`, the full per-record report/envelope is still printed but the invocation now exits **4** (VERIFY-ME — do not trust) instead of 0 — this applies in BOTH standalone mode (no `--blob`) AND combined mode (`--blob` + `--bsms-round1` together; the parsed bundle/card is still synthesized and emitted, only the exit code changes). `$?`-gated scripts that previously treated exit 0 as "all signatures verified" must check for exit 4. `--bsms-verify-strict` is unaffected — a failed signature under strict mode was already fatal (exit 2) before this point. |
| `--bsms-verify-strict` | (v0.27.0) make BIP-129 Round-1 SIG verification failures fatal; without this flag, verify mismatches emit a stderr NOTICE and proceed with `signature_verified: false` (exit 4 per the `--bsms-round1` row above, v0.85.0); requires `--bsms-round1` to be meaningful |
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
| `4` | `ImportWalletSeedMismatch` — supplied seed does not match blob's declared xpub at the cosigner's origin path; OR (v0.85.0) `--bsms-round1` lenient-default mode with any record's `signature_verified: false` (standalone OR combined with `--blob`) — report/envelope still printed |
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
| NOTICE (decrypt succeeded; overall exit is 0 or 4 — v0.85.0) | `notice: import-wallet: --bsms-round1: BIP-129 encrypted Round-1 record <i> decrypted (token width <N> hex chars; MAC verified)` (v0.32.1). This NOTICE only reports that the record's ENCRYPTION layer decrypted and MAC-verified; it says nothing about the record's own BIP-322 SIGNATURE. Since v0.85.0, if that signature then fails to verify (lenient mode), the overall invocation exits `4` despite this NOTICE having fired — see the `--bsms-round1` exit-code row above. |
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

### Worked example — Bitcoin Core `listdescriptors` split import

Bitcoin Core emits `listdescriptors` output as **two separate
descriptor entries** for the canonical receive/change pair — `.../0/*`
(`"internal": false`) and `.../1/*` (`"internal": true`) — never a
combined `<0;1>/*` multipath (multipath is import-only for Core). The
toolkit **auto-recombines** this same-key pair into **one** `<0;1>/*`
multipath bundle at parse time, so a standard split export imports
directly and yields a single merged bundle (not one-per-entry):

```sh
bitcoin-cli listdescriptors > /tmp/core-export.json
mnemonic import-wallet --blob /tmp/core-export.json --format bitcoin-core --json
```

Stdout — one merged envelope (`[...]` collapsed for brevity). The
merged descriptor carries the `<0;1>/*` multipath use-site and a freshly
recomputed BIP-380 checksum; its `source_metadata.internal` is `null`
("both chains"), so the single bundle satisfies both
`--select-descriptor active-receive` and `active-change`:

```json
[
  {
    "bundle": {
      "cosigners": [{"fingerprint": "73c5da0a", "path_raw": "[73c5da0a/84h/0h/0h]", "xpub": "xpub6CatWdi...", "has_entropy": false}],
      "descriptor": "wpkh([73c5da0a/84h/0h/0h]xpub6CatWdi.../<0;1>/*)#........",
      "network": "mainnet",
      "threshold": null
    },
    "source_format": "bitcoin-core",
    "source_metadata": {"wallet_name": "mywallet", "active": true, "internal": null, "range": [0, 999]},
    "roundtrip": {"byte_exact": false, "semantic_match": true, "diff": "...", "status": "ok"}
  }
]
```

A receive/change-*shaped* pair whose keys or origins differ is **not**
merged — it is refused (exit 2) with a distinct-key message, since
distinct keys are different wallets. Stderr is silent under `--json`
(the diff lives in the envelope).
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
| Descriptor's key placeholder use-site path ends in a fixed step (`/0/*`, `/0h/*`) | exit 2 `ImportWalletParse` — message names the offending residue and the multipath remedy; see [Non-representable use-site steps](#non-representable-use-site-steps). The BIP-388 `/**` shorthand is **accepted** (expanded to `/<0;1>/*`), not refused. Bitcoin Core's separate receive/change entries are **auto-recombined** into one `/<0;1>/*` bundle before this check, so a standard split export imports cleanly; a lone fixed-step entry with no receive/change partner still rejects — see [foreign wallet formats](#foreign-wallet-formats) |
| Bitcoin Core receive/change-shaped pair whose keys/origins differ | exit 2 `ImportWalletParse` — not merged (distinct keys are different wallets); see [foreign wallet formats](#foreign-wallet-formats) |

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
| `combine` shares: digest verification failure | `slip39 combine: reconstructed master digest mismatch — a share was substituted, corrupted, or the shares do not reconstruct the intended secret; this check runs BEFORE passphrase decryption and does NOT verify --passphrase (a wrong --passphrase still exits 0 with a different, incorrect master)` |
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

```{.text include="41-seedqr-decode.out"}
PLACEHOLDER — generated from transcripts/41-seedqr-decode.out at build
```

JSON envelope form:

```sh
mnemonic seedqr decode --from seedqr=000000000000000000000000000000000000000000000003 --json-out /tmp/decode.json
cat /tmp/decode.json
```

`/tmp/decode.json` contents:

```{.text include="41-seedqr-decode-json.out"}
PLACEHOLDER — generated from transcripts/41-seedqr-decode-json.out at build
```

### Worked example — encode

```sh
mnemonic seedqr encode --from phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
```

Stdout:

```{.text include="41-seedqr-encode.out"}
PLACEHOLDER — generated from transcripts/41-seedqr-encode.out at build
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

```{.text include="41-seedqr-encode-24.out"}
PLACEHOLDER — generated from transcripts/41-seedqr-encode-24.out at build
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

```{.text include="41-nostr-npub.out"}
PLACEHOLDER — generated from transcripts/41-nostr-npub.out at build
```

The same key, all four script types:

```sh
mnemonic nostr --pubkey npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg \
  --all-script-types
```

```{.text include="41-nostr-npub-all.out"}
PLACEHOLDER — generated from transcripts/41-nostr-npub-all.out at build
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

```{.text include="41-nostr-nsec-stderr.out"}
PLACEHOLDER — generated from transcripts/41-nostr-nsec-stderr.out at build
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

```{.text include="41-nostr-evenY-stderr.out" lines="1-1"}
PLACEHOLDER — generated from transcripts/41-nostr-evenY-stderr.out line 1 at build
```

The emitted WIF and address correspond to the normalized (even-y) key,
not the raw scalar, so `WIF → address` is always self-consistent.

### JSON output (`--json`)

`--json` emits a single JSON object on stdout:

```sh
mnemonic nostr --pubkey npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg \
  --json
```

```{.text include="41-nostr-json.out"}
PLACEHOLDER — generated from transcripts/41-nostr-json.out at build
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
| `--network <mainnet\|testnet\|signet\|regtest>` | mainnet → `sp` address + coin-type 0; testnet/signet/regtest → `tsp` address + coin-type 1 (default mainnet). For an xprv/tprv `--secret`, `--network` (including the default) must agree with the key's own version bytes — a disagreement is refused fail-closed (exit 2) rather than deriving at the wrong coin-type; phrase/ms1/entropy-hex secrets are network-agnostic (the master mints AT `--network`) and are unaffected |
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

`--from` accepts an account `xpub=` (derived directly) or a seed source (`phrase=` / `entropy=` / `seedqr=` / `electrum-phrase=`). For a BIP-39 seed source, `--address-type` selects the BIP-44/49/84/86 account path (`p2pkh`→44', `p2sh-p2wpkh`→49', `p2wpkh`→84', `p2tr`→86') at `m/<purpose>'/<coin>'/<account>'`, and the addresses are `m/<chain>/<index>` under it. For an `xpub=` source the xpub *is* the account key, so `--account` / `--passphrase` do not apply (supplying them is an error). Secret values support `@env:VAR` and `-` (stdin).

`electrum-phrase=` (v0.47.0+) derives Electrum's **own** native-seed addresses (NOT BIP-39/BIP-44): `PBKDF2-HMAC-SHA512(seed, "electrum"+passphrase, 2048)` → BIP-32 root → for a **standard** seed `m/<chain>/<index>` (P2PKH), for a **segwit** seed `m/0'/<chain>/<index>` (P2WPKH). The script type and derivation are **fixed by the Electrum seed version**, so `--address-type` must match it (`p2pkh` for standard, `p2wpkh` for segwit — a mismatch is refused), `--account` does not apply (refused if non-zero), and `--language` is ignored (the seed is stretched from the raw phrase string, not decoded via a wordlist). 2FA seeds (versions 101/102) are refused. `--passphrase` is the Electrum seed-extension passphrase. The `<chain>` is `0` (receive) / `1` (change) per `--chain`, as in Electrum.

### Flags

| Flag | Purpose |
|---|---|
| `--from <SOURCE>` | `xpub=<v>` \| `phrase=<v>` \| `entropy=<hex>` \| `seedqr=<digits>` \| `electrum-phrase=<v>`; `@env:VAR` / `-` (stdin) for secret values |
| `--address-type <T>` | `p2pkh` \| `p2sh-p2wpkh` \| `p2wpkh` \| `p2tr` (required; selects the account path for BIP-39 seed sources and the render type. For `electrum-phrase=` it must match the seed version: `p2pkh` standard / `p2wpkh` segwit) |
| `--account <N>` | account index for BIP-39 seed sources (default 0; not applicable to `xpub=` or `electrum-phrase=`) |
| `--count <N>` | number of addresses per chain, from index 0 (default 10); conflicts with `--range` |
| `--range <A,B>` | inclusive index range `A..=B`; conflicts with `--count` |
| `--chain <receive\|change\|both>` | which chain(s) to list (default `receive`) |
| `--network <NET>` | `mainnet` \| `testnet` \| `signet` \| `regtest`; defaults to the xpub's version bytes (xpub source) or mainnet (seed source); must agree with an xpub's network kind |
| `--passphrase <V>` | BIP-39 passphrase (seed sources); `@env:VAR` supported |
| `--passphrase-stdin` | read the BIP-39 passphrase from stdin (conflicts with `--passphrase`) |
| `--language <L>` | BIP-39 wordlist language for `phrase=`/`seedqr=` (default `english`); ignored for `electrum-phrase=` (the Electrum seed is stretched from the raw phrase string, not decoded via a wordlist) |
| `--json` | emit a JSON envelope instead of the text rows |
| `--help` | print help |

`--count`/`--range` indices are bounded by the BIP-32 normal-index ceiling (`< 2^31`); an out-of-range request is rejected (never a panic).

### Worked example

```sh
mnemonic addresses --from xpub=xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8petHexjjn5WbQ9PriVrRhphw4oCp2z6a \
  --address-type p2wpkh --count 3
```

```{.text include="41-addresses-xpub.out"}
PLACEHOLDER — generated from transcripts/41-addresses-xpub.out at build
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
mnemonic repair [--ms1 <MS1>] [--mk1 <MK1> [--mk1 <MK1>...]] [--md1 <MD1> [--md1 <MD1>...]] [--json]
```

### Flags

| Flag | Purpose |
|---|---|
| `--ms1 <MS1>` | single `ms1` chunk to repair; use `-` to read one chunk from stdin; may be combined with `--mk1` / `--md1` (one HRP per card; per D35) |
| `--mk1 <MK1>` | one or more `mk1` chunks (repeating flag); use `-` to read chunks from stdin (one per line); may be combined with `--ms1` / `--md1` (one HRP per card; per D35) |
| `--md1 <MD1>` | one or more `md1` chunks (repeating flag); use `-` to read chunks from stdin (one per line); may be combined with `--ms1` / `--mk1` (one HRP per card; per D35) |
| `--json` | emit a single JSON envelope on stdout instead of the text-form repair report |
| `--max-indel <N>` | search up to N (0–4, default 0) insert/delete edits to recover a chunk that failed normal repair — a single character added (too long) or dropped (too short) during transcription; ms1/mk1/md1 |
| `--max-subst <E>` | also tolerate up to E (0–4, default 0) substitution errors alongside the indels; a recovery that used a substitution is printed as a VERIFY-ME candidate (exit 4), not a confident correction |
| `--help` | print help |

### Exit codes

| Code | Meaning |
|---|---|
| `0` | all chunks already valid (no repair applied; input echoed to stdout unchanged) |
| `5` | at least one chunk corrected AND self-verified (`REPAIR_APPLIED`) — mk1 (full `chunk_set_id` group reassembles) / **chunked** md1 (multi-chunk, or chunked-of-1 — content-id check passes), incl. a unique full-checksum `--max-indel` recovery that re-validates by reassembly (a non-chunked md1 indel cannot reassemble, so it is not among these — it exits 2); stdout = repair report + corrected chunks |
| `4` | ambiguous (multiple `--max-indel` candidates), **or a candidate required ≥1 substitution with no self-oracle** — **every `--ms1` substitution correction (Cycle F — see [ms1 substitution-correction demotion](#mnemonic-repair-ms1-substitution-demotion) below)**, **every non-chunked `--md1` single-string correction (v0.86.0 — see [md1 non-chunked demotion](#mnemonic-repair-md1-non-chunked-demotion) below)**, **or (mk1 only, Cycle E) a corrected chunk set is INCOMPLETE and so cannot be set-verified** — verify each before trusting; all candidates are printed |
| `2` | unrepairable (per-chunk `RepairError`; e.g. `TooManyErrors`, `HrpMismatch`, `ReservedInvalidLength`, `UnsupportedCodeVariant`, or `--max-indel` exhausted without a recovery) **or (mk1 only, Cycle E) a COMPLETE corrected chunk set that fails cross-chunk reassembly** (`SetReassemblyMismatch` — the correction aliased to a different, wrong card; auto-repair does NOT apply it). See [mk1 set-level re-verify](#mnemonic-repair-mk1-set-level-reverify) |
| `1` | I/O error or other generic failure |

### mk1 set-level re-verify (Cycle E funds fix) {#mnemonic-repair-mk1-set-level-reverify}

BCH correction is a best-fit operation: it returns the codeword within
Hamming distance 4 of the corrupted input, and for a genuine ≤4-error
corruption that is provably the originally-encoded chunk. Beyond that
bound (5 or more substitution errors in one chunk), a correction can
still *succeed* — the corrected chunk passes its own BCH check — while
actually **aliasing to a different, valid-but-wrong codeword** rather
than recovering the original. This PARTIAL-SET failure mode matters
specifically for **mk1**, whose chunks are repaired and reported
per-chunk: a **chunked** `md1` (multi-chunk, or chunked-of-1) always
carries the content-id and rejects a full-set alias on its own
(unchanged by this cycle). A **non-chunked** `md1` single-string card
has no such check to fall back on at all — see [md1 non-chunked
demotion](#mnemonic-repair-md1-non-chunked-demotion) below (v0.86.0)
for that card's own, structurally-analogous-to-ms1 gap. `ms1` has no
chunk-set at all (it is always a single string), so this particular
partial-set gap never applied to it — but ms1 has a **worse,
undetectable variant** of the same underlying substitution-aliasing
risk with no self-oracle whatsoever; see [ms1
substitution-correction demotion](#mnemonic-repair-ms1-substitution-demotion)
below.

An empirically measured rate for this failure mode — a 5-substitution
corruption of an mk1 regular-code chunk aliasing to a different, valid
codeword — is on the order of **7.2 × 10⁻⁵** (a 95% Clopper-Pearson
upper confidence bound, measured by a seeded, reproducible harness in
the toolkit test suite; not a theoretical estimate). Small, but not
zero — and the corrected chunk alone cannot distinguish the two cases;
only reassembling the full card (its cross-chunk hash) can.

`mnemonic repair --mk1` (and the auto-fire short-circuit on `convert` /
`inspect` / `verify-bundle`, [above](#auto-fire-on-decode-failure-v0221))
re-verify every full `chunk_set_id` group before reporting a confident
fix:

- **A COMPLETE group (every chunk of the card supplied) reassembles
  cleanly** — reported as repaired, exit `5`, as before.
- **A COMPLETE group's correction FAILS reassembly** — the per-chunk
  correction has aliased to a different card. `mnemonic repair` REJECTS
  it outright: the un-repaired decode error surfaces (exit `2`); no
  corrected chunk is printed, and — critically — **auto-repair does NOT
  apply the miscorrection either**; it falls through to the original
  error. This is a breaking exit-code change from pre-Cycle-E behavior,
  where such a miscorrection could be reported (or silently applied via
  auto-fire) as a confident fix.
- **An INCOMPLETE group is supplied** (a single plate of a multi-chunk
  card, or otherwise fewer chunks than `total_chunks`) — reassembly
  cannot be checked, because the other chunks aren't present.
  `mnemonic repair` reports it as an exit-`4` VERIFY-ME candidate (the
  same precedence tier as an ambiguous `--max-indel` recovery) with an
  `UNVERIFIED` advisory: reassemble the full card (e.g. `mnemonic
  inspect --mk1` for every chunk, or `mk decode`) before trusting the
  correction. A batch call mixing an incomplete group with a fully-
  reassembling group still reports the incomplete group's candidate —
  but a batch containing a REJECTED (aliased) group fails the WHOLE
  call (reject dominates candidate and bless; the rejected group's
  chunks are never presented as recovered).

BIP-93 itself recommends confirming a corrected codex32 string before
relying on it; this advisory operationalizes that recommendation for
the one case the re-verify cannot resolve on its own.

### ms1 substitution-correction demotion (Cycle F funds fix) {#mnemonic-repair-ms1-substitution-demotion}

`ms1` encodes raw BIP-39 entropy as a SINGLE codex32 string — a bearer
secret with no cross-chunk hash, no fingerprint, and no internal
redundancy beyond the BCH checksum itself. A bounded-distance (≤4-error)
BCH substitution-correction is provably the original chunk, but beyond
that bound the "correction" can still *succeed* while **aliasing to a
DIFFERENT, valid-but-wrong seed** — and unlike mk1/**chunked** md1, there
is no cross-chunk hash or content-id to catch it (a **non-chunked** md1
shares this exact gap — v0.86.0, see [md1 non-chunked
demotion](#mnemonic-repair-md1-non-chunked-demotion) below). A miscorrection *presents*
as an ordinary small correction; the BCH code cannot distinguish a
genuine ≤4-error fix from a longer-distance aliasing event at the same
apparent edit distance.

Every `ms1` substitution-correction is therefore demoted to an
exit-`4` **VERIFY-ME Candidate** — **never** a silent exit-`5`
"recovered" — across every surface that touches it:

- **`mnemonic repair --ms1`** (and **`ms repair`**, ms-cli's standalone
  binary): any touched correction reports the corrected string as a
  Candidate, exit `4`, plus a stderr advisory recommending the user
  confirm the derived address/xpub against a known-good copy before
  trusting it. A clean (already-valid) decode is unaffected: exit `0`.
- **Auto-fire on `convert` / `inspect` / `xpub-search`:** a corrected
  `ms1` no longer short-circuits (no silent apply). The caller's
  ORIGINAL decode error surfaces unchanged, plus a one-line stderr
  advisory ("a candidate correction exists but a seed card cannot be
  self-verified — run `mnemonic repair --ms1 …` to inspect it") so the
  withheld candidate is not silently invisible.
- **`verify-bundle`:** the one surface with an actual ground truth
  available — the user's own TYPED seed. See [ms1 ground-truth
  compare](#auto-fire-on-decode-failure-v0221) above: match against the
  expected seed ⇒ checks pass; mismatch ⇒ `ms1_entropy_match` fails ⇒
  exit `4`.

**The one carve-out — indel recovery keeps exit `5`:** `mnemonic repair
--ms1 --max-indel <N>` (standalone only; `ms repair` has no `--max-indel`
flag, and no auto-fire site has indel plumbing) is a DIFFERENT
mechanism: it enumerates candidate insert/delete edits and RE-VALIDATES
the FULL BCH checksum on each, rather than spending the checksum's
substitution budget. A UNIQUE full-checksum indel candidate is
trustworthy to within the checksum's own false-accept rate —
cryptographically stronger than the reassembly/content-id check mk1 /
chunked md1 are ALREADY blessed on — so it remains a genuine
self-verification and stays exit `5`. A multi-hit (ambiguous) indel
recovery is not unique and falls to exit `4` like any other ambiguous
candidate.

BIP-93 itself recommends against automatically proceeding with a
corrected codex32 string without user confirmation; this demotion
operationalizes that recommendation for the one card kind with no
self-oracle at all.

### md1 non-chunked demotion (v0.86.0 funds fix) {#mnemonic-repair-md1-non-chunked-demotion}

`md1` has TWO wire shapes sharing the same 37-bit chunk header: a
**chunked** form (multi-chunk, or **chunked-of-1** — `count == 1` with
the chunked-flag bit set, the shape `mnemonic bundle` /
`--md1-form=template` always emits via `md_codec::chunk::split`) and a
**non-chunked** single-string form (the chunked-flag bit clear — the
compact form plain `md encode` emits for a small-enough payload). Both
decode through the SAME `md_codec::decode_with_correction` delegate, but
only the chunked form re-derives and compares the content-id
(`vendor/md-codec/src/chunk.rs:379-387`) — md-codec 0.35.0 added a
bypass (`chunk.rs:615-631`) that routes a non-chunked single string
straight to `decode_md1_string`, skipping that check entirely. A
non-chunked md1 therefore has the SAME structural gap as `ms1` (above):
a bounded-distance (≤4-error) correction is provably the original
payload, but beyond that bound it can still *succeed* while **aliasing
to a DIFFERENT, valid-but-wrong descriptor**, with no cross-chunk hash
or content-id to catch it.

From v0.86.0, every TOUCHED non-chunked md1 correction is therefore
demoted to an exit-`4` **VERIFY-ME Candidate** — **never** a silent
exit-`5` "recovered" — mirroring the ms1 treatment exactly:

- **`mnemonic repair --md1`:** any touched correction on a non-chunked
  single string reports the corrected string as a Candidate, exit `4`,
  plus a stderr `UNVERIFIED` advisory recommending the user re-derive
  the wallet/address to confirm before trusting it. A clean
  (already-valid) decode, and any CHUNKED correction (multi-chunk or
  chunked-of-1), are unaffected: still exit `0` / exit `5` respectively.
- **Auto-fire on `convert`\* / `inspect` / `verify-bundle`:** a
  corrected non-chunked md1 no longer short-circuits (no silent apply).
  The caller's ORIGINAL decode error surfaces unchanged, plus a
  one-line stderr advisory ("a candidate correction exists but a
  non-chunked descriptor cannot be self-verified — run `mnemonic repair
  --md1 …` to inspect it") so the withheld candidate is not silently
  invisible. \*`convert` has no `--from md1=…` target, so this applies
  to `inspect` / `verify-bundle` only.
- md1 has no `--max-indel` carve-out distinct from the above. A
  `--max-indel` recovery re-validates the whole checksum **by
  reassembly**, so it succeeds (and correctly stays exit 5) only for a
  **chunked** md1 (chunked-of-1 or multi-chunk); a **non-chunked** md1
  indel cannot reassemble and is not recoverable at all (exit 2), so
  this substitution-correction demotion leaves the md1 indel path
  untouched (see [Recovering an incorrect-length
  card](#mnemonic-repair-max-indel) below).

**Sibling-CLI divergence:** the standalone `md repair` (md-cli, see
`42-md.md`) does NOT yet apply this demotion — it still reports exit
`5` for every correction, chunked or not. `mnemonic repair --md1` and
`md repair` therefore currently disagree on a non-chunked md1's exit
code; see FOLLOWUP `md-cli-non-chunked-single-string-repair-demote`.

BIP-93 itself recommends against automatically proceeding with a
corrected codex32 string without user confirmation; this demotion
closes the one md1 shape where the earlier content-id-based reasoning
did not actually apply.

### Worked example

```sh
# A valid ms1 chunk with one character corrupted (position 17 'q' → 'z'):
mnemonic repair --ms1 ms10entrsqqqqqqqqqqqzqqqqqqqqqqqqqqqqcj9sxraq34v7f
```

Stdout (the corrected chunk is on the LAST line; comment lines describe the fix):

```{.text include="41-repair-ms1.out"}
PLACEHOLDER — generated from transcripts/41-repair-ms1.out at build
```

Stderr:

```{.text include="41-repair-ms1.err"}
PLACEHOLDER — generated from transcripts/41-repair-ms1.err at build
```

Exit code: `4` (Candidate — see [ms1 substitution-correction
demotion](#mnemonic-repair-ms1-substitution-demotion) above; the
corrected string is printed, but a seed card cannot self-verify, so it
is never a confident exit-`5` "recovered").

### JSON output

The `verdict` field (Cycle F) is `"blessed"` for a clean or confidently-
recovered card, `"candidate"` for a touched-but-unverified correction —
reachable for every `ms1` substitution-correction, an incomplete mk1
partial-plate group, and (v0.86.0) every non-chunked `md1` single-string
correction (see [md1 non-chunked
demotion](#mnemonic-repair-md1-non-chunked-demotion) above). `ms-cli`'s
standalone `ms repair --json` byte-matches this field's position.

```{.text include="41-repair-ms1-json.out"}
PLACEHOLDER — generated from transcripts/41-repair-ms1-json.out at build
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
| Inline `--ms1 <value>` or an `ms1`-classified positional on argv (v0.53.2) | `warning: secret material on argv (--ms1) — pipe via --ms1 - to avoid /proc/$PID/cmdline exposure` (per-occurrence; positionals are labelled `(positional ms1)`; fires for `--ms1` even under `--max-indel` relaxation, where the corrupted value no longer HRP-classifies) |

### Recovering an incorrect-length card (`--max-indel`) {#mnemonic-repair-max-indel}

`mnemonic repair` corrects substitution errors at a FIXED length. When a
character was inserted or dropped during hand-copy (so the string is the
wrong length and no longer decodes), pass `--max-indel <N>` (1–4) to also
search for that indel. The search covers the data-part (delete-and-validate
for too-long; BCH-solve the omitted symbol for too-short) and the `ms1`/`mk1`
prefix; it also considers indels split across **both** the prefix and the
data-part simultaneously (tagged `cross-region`), within the `--max-indel`
budget. Outcomes: a unique recovery prints the corrected string (exit 5 — a
unique indel candidate re-validates the full checksum, so it self-verifies,
unlike a substitution-correction which is demoted to an exit-4 candidate);
multiple equally-valid candidates print all of them (exit 4 — choose
manually); none within the budget exits 2. `ms1` candidates are secret
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
structured JSON envelope instead of the text-form repair report. Schema
(the `kind: "ms1"` value below illustrates the field SHAPE only — since
Cycle F, an `ms1` substitution-correction can no longer produce this
envelope; it always falls through to the original decode error plus a
stderr advisory instead, see [ms1 substitution-correction
demotion](#mnemonic-repair-ms1-substitution-demotion). The only kinds
that can still emit `auto_repair_short_circuit: true` / `exit_code: 5`
are `mk1` — a full, cleanly-reassembling `chunk_set_id` group — and
`md1`):

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
- `md1` — the keyless BIP-388 `@N` wallet-policy template (the full
  miniscript expression with `@N` key placeholders, e.g.
  `wsh(or_i(and_v(v:pk(@0/<0;1>/*),after(1000000)),multi(2,@1/<0;1>/*,@2/<0;1>/*)))`;
  rendered identically to `md decode`), placeholder count (`n`),
  root-tree tag (`Wpkh` / `Tr` / `Wsh` / …), wallet-policy-mode flag,
  path-decl shape (`Shared` vs `Divergent`).

### Synopsis

```sh
mnemonic inspect [--ms1 <MS1>] [--mk1 <MK1> [--mk1 <MK1>...]] [--md1 <MD1> [--md1 <MD1>...]] [--json] [--reveal-secret]
```

### Flags

| Flag | Purpose |
|---|---|
| `--ms1 <MS1>` | single `ms1` chunk to inspect; use `-` to read one chunk from stdin; may be combined with `--mk1` / `--md1` (one HRP per card; per D35) |
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

```{.text include="41-inspect-ms1.out"}
PLACEHOLDER — generated from transcripts/41-inspect-ms1.out at build
```

Stderr:

```{.text include="41-inspect-ms1.err" lines="2-2"}
PLACEHOLDER — generated from transcripts/41-inspect-ms1.err line 2 at build
```

### JSON output (v0.27.0)

When `--json` is supplied, `inspect` emits a single JSON envelope on
stdout instead of the text-form report. The envelope carries a
top-level `schema_version: "2"` field (v0.27.0 backfill via the new
`InspectEnvelope` wrapper; bumped `"1"`→`"2"` in v0.75.0 when the md1
body gained the `template` field) followed by the kind-specific fields
(the md1 body leads with `template`, the keyless `@N` wallet-policy
expression):

```{.text include="41-inspect-ms1-json.out"}
PLACEHOLDER — generated from transcripts/41-inspect-ms1-json.out at build
```

`schema_version` is the shared top-level envelope field, so the ms1 /
mk1 / md1 envelopes all report `"2"` even though only the md1 body
changed. It is currently pinned at `"2"` (v0.75.0); future format
changes will bump the version with explicit migration notes in the
SPEC. `mnemonic repair`'s JSON output carries its own, independent
`schema_version` (still `"1"` since v0.22.0) — the inspect bump does
not touch it.

### Auto-fire short-circuit

When a corrupted card is supplied to `inspect`, the sibling-codec
decode fails and v0.22.0 auto-fire kicks in. For `md1`, instead of
surfacing the typed decode error, the toolkit attempts BCH correction
and — on success — prints the corrected card and exits with code `5`.
For **`ms1` (Cycle F)**, auto-fire no longer short-circuits: a
substitution-correction is a Candidate with no self-oracle (see [ms1
substitution-correction
demotion](#mnemonic-repair-ms1-substitution-demotion)), so the
ORIGINAL typed decode error surfaces unchanged, plus a one-line stderr
advisory pointing at `mnemonic repair --ms1` to inspect the withheld
candidate. Pass the global `--no-auto-repair` flag to suppress even
that advisory and restore the pre-v0.22 behavior verbatim.

For `mk1` specifically, the toolkit's auto-fire is essentially
redundant: `mk-codec` performs INTERNAL BCH correction at the same
`t=4` capacity inside `mk_codec::decode`, so corrupted `mk1` chunks
within capacity are silently fixed before reaching the auto-fire
boundary. Auto-fire (the exit-`5` short-circuit) is reachable only for
`md1` (no internal correction in `md-codec`); `ms1` (codex32-delegated;
no internal correction) never short-circuits per the above.

### Refusals

`inspect` surfaces whatever the underlying sibling-codec `decode`
returns; consult the per-codec chapters (`md`, `ms`, `mk-cli`) for
the full per-error taxonomy.

### Advisories

| Trigger | Stderr advisory |
|---|---|
| Any `ms1` inspection (regardless of `--reveal-secret`) | `warning: stdout carries private key material (can spend) — redirect or encrypt ...` |
| Inline `--ms1 <value>` or an `ms1`-classified positional on argv (v0.53.2) | `warning: secret material on argv (--ms1) — pipe via --ms1 - to avoid /proc/$PID/cmdline exposure` (per-occurrence; positionals are labelled `(positional ms1)`) |

Card intake is case-tolerant (v0.53.3 routing; `ms1` end-to-end since v0.53.5):
all-uppercase cards — the BIP-173 QR alphanumeric form — are routed to the
right codec, and `mk1`/`md1`/`ms1` all decode end-to-end. The codecs remain
the authority on case — `mk1`/`ms1` reject mixed-case input, `md1` is lenient.

## `mnemonic xpub-search` (v0.26.0) {#mnemonic-xpub-search}

Umbrella subcommand for **reverse searches over a BIP-32 derivation graph** — given a seed (or xpub), find which derivation produces a target xpub / descriptor / address / passphrase. v0.26.0 ships four modes:

- **`path-of-xpub`** — given seed + target xpub (or mk1 card), find the BIP-32 path under the seed that produces it.
- **`account-of-descriptor`** — given seed + descriptor, find the cosigner role + account index.
- **`address-of-xpub`** — given xpub + address, scan child indices to a gap limit.
- **`passphrase-of-xpub`** — given seed + passphrase + target xpub, verify the passphrase produces the xpub at a standard path.

See [Consensus-masked relative timelocks](#consensus-masked-relative-timelocks)
for the non-blocking `older()` advisory the descriptor-bearing modes
(`account-of-descriptor`) emit on intake.

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
| 1 | Bad input (BIP-39 parse failure, xpub parse failure, mk1 decode failure, or an `--ms1` decode failure — since Cycle F an `--ms1` substitution-correction never short-circuits here; the original decode error always surfaces, with a stderr advisory when a candidate correction exists) |
| 4 | No match in searched set (`ToolkitError::XpubSearchNoMatch`) |
| 64 | Clap arg-parse error |

Exit `5` is **not reachable** by this subcommand (Cycle F): the seed's
`--ms1` intake previously auto-fired a short-circuit on TTY-positive
decode failure; it now always falls through to the typed decode error
(exit `1`) plus the advisory above. See [ms1 substitution-correction
demotion](#mnemonic-repair-ms1-substitution-demotion).

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
| 1 | Bad input (descriptor parse error, toolkit-@N refusal, no-xpub-keys refusal, seed-intake error — since Cycle F an `--ms1` decode failure always surfaces here, with a stderr advisory when a candidate correction exists) |
| 4 | No cosigner matched (`ToolkitError::XpubSearchNoMatch`) |
| 64 | Clap arg-parse error |

Exit `5` is **not reachable** here (Cycle F) — see [ms1
substitution-correction demotion](#mnemonic-repair-ms1-substitution-demotion).

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
| `--network <NET>` | network selector: `mainnet` / `testnet` / `signet` / `regtest`. Default inferred from the xpub version byte; `--network signet`/`--network regtest` overrides the test/signet/regtest ambiguity collapsed by the version byte. An explicit `--network` that disagrees with the (possibly mk1-decoded) xpub's own version bytes is refused fail-closed (exit 2) rather than scanning wrong-network addresses; omit `--network` to infer it instead |
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

The passphrase source is **mandatory**: exactly one of `--passphrase` / `--passphrase-stdin` / `--passphrase-candidates-file` must be supplied. Omitting all three is a clap arg-parse error (exit 64).

**Candidate-list scan (`--passphrase-candidates-file`, v0.46.0).** Instead of one passphrase, supply a text file with **one candidate passphrase per line** (no argv exposure). The command derives the master seed per candidate and stops at the **first** that produces `--target-xpub`, reporting the matching **file line number** to stdout (the matching passphrase appears only under `--json`). Blank lines are skipped; each non-blank line is a literal candidate (only the trailing newline/CR is stripped — no other trimming, since a passphrase is an exact byte string). No match ⇒ exit 4 with the count of candidates tried. This is bounded **verification of a list you supply** — for keyspace *generation* (wordlists, masks, typo models) use [`btcrecover`](https://github.com/3rdIteration/btcrecover) (see the passphrase-recovery note at the top of this chapter). The candidate file is sensitive (holds secret candidates); it is classified as a path (non-secret) flag.

#### Synopsis

```sh
mnemonic xpub-search passphrase-of-xpub \
    {--phrase <BIP39> | --phrase-stdin | --ms1 <MS1> | --ms1-stdin | <positional MS1>} \
    {--passphrase <P> | --passphrase-stdin | --passphrase-candidates-file <PATH>} \
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
| `--passphrase <P>` | BIP-39 passphrase (inline); emits argv-leakage advisory. One of the mandatory passphrase-source group |
| `--passphrase-stdin` | read BIP-39 passphrase from stdin (NULL-byte-preserving; single trailing newline stripped). One of the mandatory passphrase-source group |
| `--passphrase-candidates-file <PATH>` | scan a text file of candidate passphrases (one per line, no argv exposure); first match wins, reports the file line (passphrase only in `--json`); exit 4 if none match. One of the mandatory passphrase-source group |
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

**Candidate-list scan (`--passphrase-candidates-file`).** Supply a text file of
candidate passphrases (one per line) instead of a single `--passphrase`. The
scan stops at the first candidate that produces the target and reports the
matching **file line** (the passphrase itself is shown only under `--json`):

```sh
printf 'hunter2\nsatoshi\ncorrect horse battery staple\n' > candidates.txt
mnemonic xpub-search passphrase-of-xpub \
    --phrase "$PHRASE" \
    --passphrase-candidates-file candidates.txt \
    --target-xpub "$ZPUB"
```

Stdout (text form, match on line 2) — plus a `note: candidates.txt holds candidate passphrases — treat as sensitive` advisory on stderr:

```text
match: candidate on line 2 derives the target xpub at m/84'/0'/0' (template=bip84, account=0)
target-xpub: xpub6... (normalized from zpub; variant=zpub)
searched: 140 candidate paths per passphrase
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

**Candidate-list scan fields.** With `--passphrase-candidates-file`, the `match`
body additionally carries `matched_candidate_line` (1-indexed file line) and
`matched_passphrase` (the winning candidate — present only for the scan), and the
`no_match` body carries `candidates_tried` (#non-blank lines tried). These fields
use `skip_serializing_if` — they are ABSENT (not `null`) on the single-`--passphrase`
path, so that envelope is byte-unchanged:

```json
{ "schema_version": "1", "mode": "passphrase-of-xpub", "result": "match",
  "path": "m/84'/0'/0'", "template": "bip84", "account": 0,
  "target_xpub_canonical": "xpub6...", "target_xpub_variant": "zpub",
  "searched_count": 140, "matched_candidate_line": 2, "matched_passphrase": "satoshi" }
```

#### Exit codes

| Code | Meaning |
|---|---|
| 0 | Match found (this passphrase produces the target xpub at one of the searched paths) |
| 1 | Bad input (BIP-39 parse failure, xpub parse failure, mk1 decode failure, or an `--ms1` decode failure — since Cycle F an `--ms1` substitution-correction never short-circuits here; the original decode error always surfaces, with a stderr advisory when a candidate correction exists) |
| 4 | No match — single passphrase (`XpubSearchNoMatch`), OR no candidate in `--passphrase-candidates-file` produced the target (`XpubSearchPassphraseCandidatesExhausted`, with the count of candidates tried; an all-blank/empty file gets a tailored "no candidates" note) |
| 64 | Clap arg-parse error (missing/duplicate passphrase source — exactly one of `--passphrase` / `--passphrase-stdin` / `--passphrase-candidates-file`) |

Exit `5` is **not reachable** by this subcommand (Cycle F) — see [ms1
substitution-correction demotion](#mnemonic-repair-ms1-substitution-demotion).

#### Refusals

| Trigger | Refusal |
|---|---|
| None of `--passphrase` / `--passphrase-stdin` / `--passphrase-candidates-file` supplied | clap `the following required arguments were not provided` error (exit 64) |
| More than one of the three passphrase sources supplied | clap `passphrase_source` group mutex error (exit 64) |
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

See [Consensus-masked relative timelocks](#consensus-masked-relative-timelocks)
for the non-blocking `older()` advisory this command emits on intake.

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

```{.text include="41-compare-cost-orb.out"}
PLACEHOLDER — generated from transcripts/41-compare-cost-orb.out at build
```

Either A or B can sign alone; both pay the same cost. tr costs `+24`
vB more per spend than wsh because the tr witness carries an extra
33-byte control block.

**2. Timelocked recovery path (SPEC §5 hero example):**

```sh
mnemonic compare-cost --miniscript 'or_d(pk(A),and_v(v:pk(B),older(144)))'
```

Stdout:

```{.text include="41-compare-cost-ord.out"}
PLACEHOLDER — generated from transcripts/41-compare-cost-ord.out at build
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

```{.text include="41-compare-cost-json.out"}
PLACEHOLDER — generated from transcripts/41-compare-cost-json.out at build
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

```{.text include="41-compare-cost-tr-ik.out"}
PLACEHOLDER — generated from transcripts/41-compare-cost-tr-ik.out at build
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

## `mnemonic build-descriptor` (v0.50.0; archetype presets v0.51.0; `--allow` v0.52.0) {#mnemonic-build-descriptor}

Build a **validated `wsh(...)` descriptor** + its **BIP-388 wallet-policy**
from a **versioned JSON policy-tree spec**. The spec is a fragment-level
miniscript tree (keys, multisig, timelocks, hashlocks, combinators); the
engine renders it to a concrete multipath descriptor and runs it through a
**fail-closed validation gate** before emitting anything. Like `restore`
/ `export-wallet`, build-descriptor is **watch-only-out** — it takes
cosigner **xpub**s and NEVER accepts secret material (an extended private
key `xprv` / WIF in a key node is refused, exit 2, without ever echoing the
key). It does not sign.

The engine is a deterministic renderer + validator, **not** a policy
compiler — you author the exact fragment tree (wrappers explicit), and a
mis-typed or unsafe tree is rejected with a node-addressed diagnostic.

### Synopsis

```sh
mnemonic build-descriptor --spec <FILE|-> [--allow <RULE>]… [--network <NET>] [--format <FMT>] [--json]
mnemonic build-descriptor --archetype <NAME> <PARAMS…> [--allow <RULE>]… [--format <FMT>] [--json]
mnemonic build-descriptor --archetype <NAME> <PARAMS…> --emit-spec   # print the lowered spec JSON
mnemonic build-descriptor --spec-schema      # dump the node-tree grammar and exit
```

### The spec (`--spec`)

A versioned JSON document `{"schema_version": 1, "wrapper": "wsh", "root": <node>}`
(`deny_unknown_fields` — typo'd fields are rejected). Each `<node>` is an
**externally-tagged** object (exactly one key). The fragment set (run
`mnemonic build-descriptor --spec-schema` for the machine-readable grammar):

| Node | miniscript | Notes |
|---|---|---|
| `{"pk": "<key>"}` / `{"pkh": "<key>"}` | `pk` / `pkh` | `<key>` = a concrete `[fp/path]xpub`; the engine appends the `/<0;1>/*` multipath suffix |
| `{"multi": {"k": K, "keys": [..]}}` / `{"sortedmulti": …}` | `multi` / `sortedmulti` | `1 ≤ k ≤ n` |
| `{"older": N}` / `{"after": N}` | `older` / `after` | relative / absolute timelock |
| `{"sha256": "<hex>"}` (+ `hash256`/`hash160`/`ripemd160`) | hashlocks | 64-hex (sha256/hash256) or 40-hex (hash160/ripemd160) |
| `{"and_v": [A,B]}` / `{"or_d": …}` / `{"or_i": …}` / `{"or_b": …}` | binary combinators | exactly 2 children |
| `{"andor": [A,B,C]}` | `andor` | exactly 3 children |
| `{"thresh": {"k": K, "subs": [..]}}` | `thresh` | k-of-n over sub-policies |
| `{"wrap": {"w": "<wrappers>", "sub": <node>}}` | `<w>:<frag>` | explicit miniscript wrapper(s) (`v`,`s`,`a`,…) on a child |

### Archetype presets (`--archetype`, v0.51.0)

Five curated vault shapes over the same engine — no JSON authoring. Each
preset lowers your flag parameters into the canonical policy tree below and
runs the full validation gate; the result is byte-identical to authoring the
equivalent `--spec` by hand.

| Archetype | Shape | Parameters |
|---|---|---|
| `simple-timelocked-inheritance` | `or_d(pk(P), and_v(v:pkh(H), older(N)))` — owner spends anytime; heir after `N` blocks | `--key` ×1, `--recovery-key` ×1, `--older` |
| `kofn-recovery` | `or_d(multi(k,K…), and_v(v:pk(R), older(N)))` — k-of-n multisig; single recovery key after `N` blocks | `--key` ×n (≥2), `--threshold`, `--recovery-key` ×1, `--older` |
| `decaying-multisig` | `andor(multi(k1,T1…), older(N1), andor(multi(k2,T2…), older(N2), and_v(v:pk(F), after(T))))` — quorum decays through a recovery quorum to a final key | `--key` ×n (≥2), `--threshold`, `--older`, `--recovery-key` ×n (≥2), `--recovery-threshold`, `--recovery-older` (> `--older`), `--final-key` ×1, `--after` |
| `tiered-recovery` | `or_i(sortedmulti(k1,P…), and_v(v:older(N), thresh(k2, pk, s:pk…)))` — primary sorted multisig OR a timelocked recovery threshold of distinct keys | `--key` ×n (≥2), `--threshold`, `--older`, `--recovery-key` ×n (≥2), `--recovery-threshold` |
| `hashlock-gated` | `andor(pk(A), sha256(H), and_v(v:pk(B), older(N)))` — primary key + SHA-256 preimage; recovery key after `N` blocks | `--key` ×1, `--hash`, `--recovery-key` ×1, `--older` |

Notes:

- **Key order is significant** — argv order maps into the quorum untouched.
- `--older` means "the timelock gating the recovery path" everywhere except
  `decaying-multisig`, where it is the **tier-1** timelock (the table above
  is the disambiguator).
- A parameter that does not belong to the chosen archetype, a missing
  required parameter, too few keys, or `--recovery-older` ≤ `--older` is
  refused with a `param`-kind diagnostic naming the flag (exit 2). Everything
  else — k > n, duplicate keys, bad hex, timelock bounds, an `xprv` — flows
  to the SAME validation gate as `--spec` and is refused there; in preset
  mode those node-addressed diagnostics also name the responsible flag
  (`(from --key)` in the human output, `flag` in `--json`).
- A deliberately unusual variant (inverted decay ordering, same-key
  degradation, …) is out of preset scope — author it as a `--spec` tree, or
  take the raw `--descriptor` door on `export-wallet` / `bundle`.

```sh
# 2-of-3 multisig, single recovery key after ~1 year — no JSON authoring:
mnemonic build-descriptor --archetype kofn-recovery \
  --key "[11111111/48h/0h/0h/2h]xpubA…" --key "[22222222/48h/0h/0h/2h]xpubB…" \
  --key "[33333333/48h/0h/0h/2h]xpubC…" --threshold 2 \
  --recovery-key "[44444444/48h/0h/0h/2h]xpubD…" --older 52560 \
  --format descriptor

# Review the generated tree first, then build from it:
mnemonic build-descriptor --archetype kofn-recovery … --emit-spec > policy.json
mnemonic build-descriptor --spec policy.json --format descriptor
```

### Flags

| Flag | Purpose |
|---|---|
| `--spec <SPEC>` | the JSON node-tree spec — a file path, or `-` for stdin. If omitted, stdin is read when it is not a TTY |
| `--network <NETWORK>` | `mainnet` (default) / `testnet` / `signet` / `regtest`. Used only for the human-view first-receive-address rendering; the descriptor / bip388 / cost output is network-agnostic (the xpubs carry the network) |
| `--format <FORMAT>` | emit a single bare artifact instead of the rich human view: `descriptor` = the concrete `wsh(M)#checksum`; `bip388` = the BIP-388 wallet-policy JSON. Omit `--format` for the human view (descriptor + first receive address + cost table). Overridden by `--json` |
| `--json` | emit a structured JSON envelope `{descriptor, bip388, cost, diagnostics}` for the GUI (the `cost` field is the embedded `compare-cost --json` object). On a gate failure: `{diagnostics: [{node_path, kind, message}]}` with exit 2. In preset mode each diagnostic may additionally carry `flag` — the CLI flag it traces back to (absent when no single flag is responsible, e.g. a duplicate key across two branches) |
| `--spec-schema` | dump the versioned node-tree grammar JSON (the schema the GUI + presets consume) and exit; ignores all other inputs. Since v0.51.0 it also carries an `archetypes` section: per-preset parameter field-specs (`flag`, `kind`, `required`, `repeatable`, `min`), generated from the registry |
| `--archetype <ARCHETYPE>` | build from a curated preset instead of a `--spec` node-tree (conflicts with `--spec`): `decaying-multisig`, `hashlock-gated`, `kofn-recovery`, `simple-timelocked-inheritance`, `tiered-recovery`. Parameters via the flags below; the lowered tree flows through the SAME validation gate |
| `--key <KEY>` | primary-path cosigner key (`[fp/path]xpub…`); repeat per cosigner. **Argv order is preserved into the quorum** (even `sortedmulti`'s descriptor string keeps authored order; sorting is script-time) |
| `--threshold <THRESHOLD>` | primary quorum k |
| `--recovery-key <RECOVERY_KEY>` | recovery-path cosigner key; repeat per cosigner (argv order preserved) |
| `--recovery-threshold <RECOVERY_THRESHOLD>` | recovery quorum k |
| `--final-key <FINAL_KEY>` | last-resort key (`decaying-multisig` tier 3) |
| `--older <OLDER>` | relative timelock (blocks) gating the recovery path (`decaying-multisig`: the tier-1 timelock) |
| `--recovery-older <RECOVERY_OLDER>` | `decaying-multisig` tier-2 relative timelock; must be **greater than** `--older` (tiers must unlock progressively later) |
| `--after <AFTER>` | `decaying-multisig` tier-3 absolute locktime (block height, or unix time past the BIP-65 threshold) |
| `--hash <HASH>` | SHA-256 digest (64 hex chars) for `hashlock-gated` |
| `--emit-spec` | print the lowered + gate-validated node-tree spec JSON instead of building — review it, edit it, feed it back via `--spec`. Conflicts with `--format` / `--json`; `--network` is accepted and ignored. The gate still runs: an invalid preset emits diagnostics, never a spec |
| `--allow <ALLOW>` | reviewed opt-out of ONE funds-safety sanity rule per occurrence (repeatable): `malleable`, `mixed-timelock`, `repeated-keys`, `resource-limit`, `sigless-branch`. Never silent — see "Reviewed sanity opt-out" below |
| `--no-auto-repair` | (global) no-op for this subcommand (there is no card decode to repair); accepted for global-flag uniformity |
| `--help` | print help |

### The validation gate

Emit is gated, in order; the first failure short-circuits to a
**node-addressed** diagnostic (`node_path` = a path like
`root.or_d[1].and_v[0]`) and exit 2:

1. **schema field-validate** — `1 ≤ k ≤ n`; hashlock hex length/validity;
   `older` must be a valid BIP-68 relative timelock — only the low 16 bits
   carry the value and bit 22 (`0x400000`) selects 512-second units, so the
   accepted domain is `1..=65535` (blocks) or `0x400000|(1..=65535)` (512-second
   units); any other bit set (incl. the bit-31 disable flag) or a zero 16-bit
   value is rejected, because consensus would silently mask it to a weakened or
   no-op timelock. `after` `1 ≤ N ≤ 0x7fffffff`. **watch-only screen** (an `xprv`
   / extended-private key is refused here, never echoed).
2. **type-check** — the rendered `wsh(M)` must parse (a missing `v:`
   wrapper → a `type_error` diagnostic at the offending subtree).
3. **`sanity_check`** — the BIP-built-in funds-footgun rules:
   `sigless_branch` (an anyone-can-spend path), `malleable`,
   `resource_limit`, `repeated_keys`, and `mixed_timelock` (an unspendable
   mixed height/time path — the "wrong timelock loses money" guard). Each
   is localized to the offending subtree.
4. **build-time complexity cap** — refuse a tree whose
   `2^(keys+hashes) × timelock-states` exceeds the always-previewable
   envelope (so the cost preview always renders); past the envelope, use a
   raw `--descriptor` with `compare-cost` / `export-wallet` instead.

### Reviewed sanity opt-out (`--allow`, v0.52.0)

Each `--allow <RULE>` waives exactly one step-3 sanity rule for THIS
invocation — a deliberate, reviewed act, never silent:

- Every rule that **actually fired** is named in an unmissable stderr
  warning (all output modes, `--json` included), and `--json` adds
  `"allowed_rules_fired": ["repeated_keys", …]` to the success envelope.
- An allowance that was requested but **did not fire** gets a
  `note: … did not fire` nudge (drop the stale flag).
- A refusal for an allowable rule names the exact token:
  `…; rerun with --allow mixed-timelock after review`.
- Allowing one rule never waives another: the gate refuses on the next
  failing rule.
- **The cost preview is unavailable on a sanity-overridden descriptor**
  (its taproot comparison would re-run the waived rules): the human view
  prints `cost preview unavailable for a sanity-overridden descriptor`,
  and `--json` emits `"cost": null`.
- `--emit-spec` records NO allowance in the spec document — replaying an
  emitted spec without `--allow` correctly refuses. The banner still
  prints on the emitting run.
- An allowed `repeated-keys` policy emits duplicate `keys_info` entries
  in its BIP-388 output (no dedup); hardware-signer registration
  behavior on duplicate keys is signer-defined.
- miniscript's 6th opt-out, `raw_pkh`, is not exposed — it is
  unreachable from this builder's node grammar.

Example — the same-key "degrading threshold" the presets deliberately
refuse (`RepeatedPubkeys`), built anyway after review:

```sh
mnemonic build-descriptor --spec degrading.json --allow repeated-keys --format descriptor
# stderr: WARNING: sanity rules OVERRIDDEN by --allow and FIRED: repeated-keys. …
```

### Worked example

A 2-of-3 multisig that degrades to a single recovery key after ~1 year
(`52560` blocks):

```sh
cat > policy.json <<'JSON'
{
  "schema_version": 1, "wrapper": "wsh",
  "root": { "or_d": [
    { "multi": { "k": 2, "keys": [
      "[11111111/48h/0h/0h/2h]xpubA…", "[22222222/48h/0h/0h/2h]xpubB…",
      "[33333333/48h/0h/0h/2h]xpubC…" ] } },
    { "and_v": [
      { "wrap": { "w": "v", "sub": { "pk": "[44444444/48h/0h/0h/2h]xpubD…" } } },
      { "older": 52560 } ] }
  ] }
}
JSON
mnemonic build-descriptor --spec policy.json --format descriptor
# → wsh(or_d(multi(2,…/<0;1>/*,…),and_v(v:pk(…),older(52560))))#<checksum>
```

The emitted descriptor round-trips: `export-wallet --descriptor <D> --format
bip388` reproduces `build-descriptor --format bip388`.

### Exit codes

| Condition | Exit |
|---|---|
| success | `0` |
| spec parse / IO error (bad JSON, unknown field, unsupported `schema_version`) | `2` |
| validation-gate failure (field / type / sanity / over-envelope / secret-key) | `2` |

## `mnemonic gen-man` (v0.73.0) {#mnemonic-gen-man}

Emit roff man pages for the whole `mnemonic` CLI tree into a directory. The
pages are generated directly from the compiled clap `Command` tree
(`clap_mangen`), so they are **binary-faithful by construction** — the man page
cannot drift from the binary's actual flag surface, and there is no
hand-authored man content to maintain.

One page is written per (nested) subcommand, named hyphen-joined parent→child:
`mnemonic.1` (root), `mnemonic-bundle.1`, `mnemonic-seed-xor-split.1`,
`mnemonic-xpub-search-path-of-xpub.1`, and so on. `scripts/install.sh` invokes
this after `cargo install` to drop the pages into the user manpath (no sudo, no
system files); see the install chapter.

### Synopsis

```sh
mnemonic gen-man --out <DIR>
```

### Flags

| Flag | Meaning |
|---|---|
| `--out <DIR>` (required) | Directory to write the `*.1` man pages into. Created if absent (`mkdir -p` semantics). |
| `--help` | Print help and exit. |

### Worked example

```sh
mnemonic gen-man --out ~/.local/share/man/man1
# → writes mnemonic.1 + one page per subcommand into that directory
man mnemonic            # if man does not find it: man -M ~/.local/share/man mnemonic
```

### Notes

- The global `--no-auto-repair` flag **does** render in every generated page's
  `OPTIONS` section (clap_mangen surfaces the root command's global args on each
  page); it is also discoverable via `mnemonic --help`.
- The output carries **no** `*-help*.1` pages (the generator uses the bare
  `generate_to` call with no pre-`build()`).

### Exit codes

| Condition | Exit |
|---|---|
| success | `0` |
| output-dir create / write I/O error | `1` |

---

## `mnemonic word-card` (v0.74.0) {#mnemonic-word-card}

Re-encode a **public** `mk1` xpub card or `md1` descriptor card as an engravable
BIP-39 **Word Card** — a list of BIP-39 words that carries the same payload as
the source `m*1` card, but rendered in the wordlist alphabet for hand-stamping
onto a steel plate. The encoding layers progressive Reed–Solomon error-correction
(`--parity-words` / `--parity-pct`), a non-linear integrity tag
(`--integrity-bits`) that catches an RS miscorrection, and an optional
cross-plate **RAID** layer (`--raid`) so a lost plate in an `mk1` xpub array can
be reconstructed from the survivors. `--decode` (and `--decode-plate` for a RAID
array) recovers the original `m*1` / xpub / descriptor.

The source `mk1` / `md1` cards are **PUBLIC, watch-only** material (an xpub or a
descriptor) — **not** spending secrets. So `--from` is not secret-classified, and
the secret `ms1` entropy card is intentionally **not** word-card-able. The value
engine lives in the in-workspace `wc-codec` crate; the toolkit bridges the
sibling codecs to it via a canonical-payload adapter.

### Synopsis

```sh
# encode (default): one solo Word Card per --from card
mnemonic word-card --from <MK1|MD1> [--from <MK1|MD1>]... [OPTIONS]

# encode an mk1 xpub array with RAID recovery plates
mnemonic word-card --from <MK1> --from <MK1> [--from <MK1>]... --raid <1|2> [OPTIONS]

# decode a single solo Word Card
mnemonic word-card --decode <WORD>... [OPTIONS]

# decode (RAID-reconstruct) an array from its surviving plates
mnemonic word-card --decode --decode-plate "<WORDS>" --decode-plate "<WORDS>"... [OPTIONS]
```

### Flags

| Flag | Purpose |
|---|---|
| `--from <MK1\|MD1>` | source `m*1` card to encode: an `mk1` xpub card or an `md1` descriptor card. Repeating flag (one per `mk1`/`md1`; multi-chunk cards may be passed joined or repeated — chunks auto-group by HRP). `-` reads one card per line from stdin. With `--raid 1\|2`, supply the `n` `mk1` data cards via repeated `--from`. PUBLIC material — not a secret |
| `--decode` | decode mode: recover the payload from an engraved Word Card. Words come from the positional `<WORD>...` list or `-`/stdin (whitespace-separated). For a RAID array, repeat `--decode-plate` once per surviving plate |
| `--decode-plate <WORDS>` | one RAID plate's whitespace-separated word list for `--decode` reconstruction (repeating flag). Supply the surviving `≥ n` plates of an `n + r` array to reconstruct a lost data plate. Mutually exclusive with the positional `<WORD>...` single-card form |
| `--parity-words <N>` | Reed–Solomon parity words `m` to append (the repair budget; corrects `⌊m/2⌋` substitutions / fills `m` erasures). Default `0` (detection only). Mutually exclusive with `--parity-pct` |
| `--parity-pct <PCT>` | Reed–Solomon parity as a PERCENTAGE of the data-symbol count `K` (`m = ceil(K * pct / 100)`), an alternative to `--parity-words`. E.g. `--parity-pct 25` ≈ a 25% redundancy budget. Mutually exclusive with `--parity-words` |
| `--raid <0\|1\|2>` | RAID recovery tier: `0` = no RAID (a single solo card; default), `1` = one XOR recovery plate (RAID-5, survives any 1 lost plate), `2` = two recovery plates (RAID-6, survives any 2). RAID requires `≥ 2` `mk1` data cards via repeated `--from`. Default `0` |
| `--integrity-bits <BITS>` | integrity-tag bit width `t` (the non-linear SHA-256 truncation that catches an RS miscorrection at `≤ 2⁻ᵗ`). Default `44` (4 words); minimum `33` |
| `--json` | emit a single JSON envelope on stdout instead of the text-form report (`schema_version: "2"`) |
| `--no-auto-repair` | global flag; skip auto-fire repair on decode failures (see [`verify-bundle` auto-fire](#mnemonic-verify-bundle)) |
| `--help` | print help and exit |

### Worked example

Encode a 2-chunk `mk1` xpub card as a solo Word Card (detection-only, no parity):

```sh
mnemonic word-card --from "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4 mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh"
# →
# # solo plate [0] (mk1, 88 words)
# abandon actor airport gas forest thing license abandon abandon abandon … logic
```

Add an 8-word Reed–Solomon repair budget (corrects up to 4 mis-stamped words),
then decode it back to the original xpub:

```sh
WORDS=$(mnemonic word-card --from "$MK1" --parity-words 8 | sed -n '2p')
echo "$WORDS" | mnemonic word-card --decode -
# →
# source_kind: mk1
# truncated: false
# erasures_filled: 0
# xpub: xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V
# policy_id_stub_count: 1
# mk1: mk1qp02pcpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qdz9lrx0gj4nwfhg
# mk1: …
```

Encode a 3-cosigner `mk1` xpub array with one XOR recovery plate (RAID-5), so any
single lost plate can be reconstructed from the other three:

```sh
mnemonic word-card --from "$MK1_A" --from "$MK1_B" --from "$MK1_C" --raid 1
# → 3 data plates + 1 recovery plate (each a labeled `# … plate [i]` header + word list)
```

Later, reconstruct a lost data plate from any three surviving plates:

```sh
mnemonic word-card --decode \
  --decode-plate "$PLATE0_WORDS" \
  --decode-plate "$PLATE2_WORDS" \
  --decode-plate "$RECOVERY_WORDS"
# → raid-reconstruct: n=3, reconstructed=[1]
#     [0] xpub: …
#     [1 *recovered] xpub: …
#       ! verify: reconstructed from RAID parity — independently verify this xpub against your other records before trusting it (it carries no integrity tag of its own)
#     [2] xpub: …
# (stderr) word-card: WARNING — plate(s) [1] were reconstructed from RAID parity; independently verify each *recovered xpub against your other records before trusting it.
```

A `*recovered` plate is one whose value came from the RAID parity solve, not its
own engraving — so it carries no per-plate integrity tag of its own. The always-on
`! verify:` advisory (text + `stderr`, and a `verify_advisory` field on the plate
in `--json`) fires ONLY for such MDS-solved plates; an all-present decode carries
no advisory. **When decoding a card SET, include a parity plate** — a mixed-vintage
chimera of plates from two different wallets that share a cosigner set is then
turned into a loud `RaidArrayMismatch` refusal by the spare-parity consistency
check, instead of silently returning a wrong xpub (constellation-eval F2).

### Notes

- **Public-only.** Only `mk1` (xpub) and `md1` (descriptor) cards can be
  word-carded; the secret `ms1` entropy card is refused by construction (it
  carries spending material). `--from` is intentionally absent from the
  argv-secret taxonomy.
- **`--parity-words` vs `--parity-pct`.** The two are mutually exclusive.
  `--parity-words N` is an absolute symbol budget; `--parity-pct P` scales the
  budget to the card size (`ceil(K · P / 100)`). With neither, the card is
  detection-only (`0` parity) — a corrupted word is flagged but not auto-fixed.
- **Integrity tag.** `--integrity-bits` sizes the non-linear SHA-256 truncation
  appended to the payload; it bounds the probability that an RS *miscorrection*
  (a confident-but-wrong repair) is silently accepted to `≤ 2⁻ᵗ`. The default
  `44` is 4 words; the floor is `33`.
- **RAID** (`--raid 1|2`) applies only to an array of `mk1` xpub cards (an
  `md1` is a single descriptor — use `--raid 0`). `--raid 1` is a single XOR
  parity plate (RAID-5); `--raid 2` adds an MDS recovery plate (RAID-6). Decode a
  RAID array with one `--decode-plate` per surviving plate. **Include a parity
  plate when decoding a card set** so the spare-parity consistency check can turn
  a mixed-vintage chimera (plates from two different wallets that share a cosigner
  set) into a loud `RaidArrayMismatch` refusal (constellation-eval F2).
- **Verify a `*recovered` xpub.** A plate reconstructed from RAID parity carries
  no integrity tag of its own, so decode emits an always-on `! verify:` advisory
  (text + `stderr`; a `verify_advisory` field in `--json`) for every MDS-solved
  plate. Independently confirm the reconstructed xpub against your other records
  before trusting it. The advisory never fires on an all-present decode.
- The Word Card is a re-rendering of **public** card material; it does not change
  the underlying xpub / descriptor identity. The re-emitted `mk1` chunks on
  decode may differ byte-for-byte from the input chunks (the `policy_id_stub` is
  re-derived) while preserving the same xpub.
- The `--json` wire-shape (`schema_version: "2"`) is **not** covered by the GUI
  `schema_mirror` gate (which enforces clap flag-name parity only); GUI consumers
  self-update via the paired-PR rule.

### Exit codes

| Condition | Exit |
|---|---|
| success (encode or decode) | `0` |
| bad input (no `--from`, non-`mk1` card under `--raid`, `--integrity-bits` below floor, I/O error) | `1` |
| decode / reconstruct refusal (parse failure beyond the repair budget, integrity-tag or header-CRC mismatch, unknown BIP-39 word, `ms1` / unknown HRP, RAID array-id or parity mismatch, RAID underdetermined — more than `r` plates missing) | `2` |
