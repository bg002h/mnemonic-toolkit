# Concept signposts

The rest of this manual assumes a working knowledge of four Bitcoin
concepts. This chapter is a one-paragraph signpost for each, with a
link to the full primer in [Part VI](#appendix-b-bip-39-entropy-primer).

## BIP-39\index{BIP-39} — wallet entropy as words

BIP-39 turns a randomly-generated chunk of entropy (typically 128 or
256 bits) into a sequence of 12 or 24 words drawn from a fixed 2048-word
list. The words encode the entropy losslessly; the same words
always recover the same wallet. The `ms1` card carries this entropy
in a checksum-protected, steel-engravable form. *Deep dive:*
[Appendix B](#appendix-b-bip-39-entropy-primer).

## BIP-32\index{BIP-32} — a tree of keys from one seed

A BIP-39 phrase deterministically yields a single master extended
private key. From that root, BIP-32 derives a tree of child keys
addressable by *paths* like `m/84'/0'/0'/0/0`. Wallets pick a
*purpose* path (BIP-44, BIP-49, BIP-84, BIP-86) and an *account*
index, then derive change and address keys from there. The `mk1`
card carries the *origin* — master fingerprint and the path used —
plus the resulting xpub. *Deep dive:*
[Appendix C](#appendix-c-bip-32-derivation-primer).

## Descriptors\index{descriptor} — the wallet's spending rule

A descriptor is a small string that tells Bitcoin Core (and other
descriptor-aware wallets) *exactly* how to construct a wallet's
addresses, including the script type (legacy / SegWit / taproot),
the keys involved, the multisig threshold, and the derivation. A
single-sig P2WPKH wallet's descriptor is simple:
`wpkh(xpub.../0/*)`. A 2-of-3 multisig descriptor is
`wsh(sortedmulti(2, xpub_a/0/*, xpub_b/0/*, xpub_c/0/*))`. The
**`md1`** card carries the descriptor as a wallet *policy*
(BIP-388 template + bound keys). *Deep dive:*
[Appendix D](#appendix-d-descriptors-and-bip-388-primer).

## Multisig\index{multisig} — K of N must sign

A multisig wallet requires *at least K* out of N cosigners to produce
a valid signature. K=N (every cosigner signs) is the toolkit's v0.1
default; K<N (threshold) is planned for v0.2. Multisig changes
neither the seed (each cosigner has their own) nor the address
derivation — it composes them through a script. The toolkit takes
each cosigner via `--slot @N.<subkey>=<value>`\index{slot}; the
descriptor card encodes the resulting multisig policy. *Deep dive
for newcomers:* [Appendix D](#appendix-d-descriptors-and-bip-388-primer)
covers descriptors broadly; multisig-specific worked examples are in
[the multisig workflow](#multi-source-2-of-3-multisig).

## Codex32 / BCH error correction\index{codex32}

Each card's last few characters are a checksum derived from a
Bose–Chaudhuri–Hocquenghem (BCH) polynomial. The `ms1` card uses
BIP-93 codex32 directly; the `mk1` and `md1` cards use a BCH variant
that mixes the human-readable prefix into the polynomial — same
algebra, different polynomial. A handful of bit errors in the
engraving are detected and *located*, so a partially-damaged card
can usually be corrected. *Deep dive:*
[Appendix E](#appendix-e-codex32-bch-m-codec-error-correction).

---

You can now read forward into [Part II — Quick start](#installing-the-toolkit)
and produce your first bundle. Or jump to whatever's relevant to
your situation.
