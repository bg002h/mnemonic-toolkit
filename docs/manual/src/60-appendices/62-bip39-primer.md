# Appendix B — BIP-39 entropy primer

BIP-39 is the standard that turns a randomly-generated chunk of
entropy into a sequence of *words*. Almost every Bitcoin wallet
software in use today understands BIP-39; the m-format star's ms1
card is a checksum-protected encoding of the same entropy.

This appendix is a one-page orientation, not a formal specification.
For the spec itself, see
[BIP-39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki).

## What BIP-39 does

1. Pick a number of bits of entropy: `128`, `160`, `192`, `224`, or
   `256`. (The corresponding word counts are `12`, `15`, `18`, `21`,
   and `24`.)
2. Compute a checksum: the first `bits/32` bits of `SHA-256(entropy)`.
3. Concatenate `entropy || checksum`; the result is a multiple of 11
   bits.
4. Slice the bitstream into 11-bit chunks, each indexing into a
   fixed 2048-word list (`english.txt`).
5. The mnemonic is the resulting word sequence.

To recover the entropy, reverse the slicing and verify the checksum.

## What BIP-39 doesn't do

- **Derive any keys.** That's BIP-32 (see Appendix C). The phrase →
  64-byte seed conversion is `PBKDF2(phrase, "mnemonic" || passphrase, 2048, HMAC-SHA-512)`.
- **Specify a wallet template.** That's BIP-44 / 49 / 84 / 86 / 388.
- **Protect against transcription errors.** A 12-word phrase has a
  4-bit checksum; a single transcription error has a small chance
  of producing a *different valid* phrase decoding to a different
  wallet. The m-format ms1 wraps this with BCH error correction so
  small stamping errors are detected and located.
- **Per-wallet uniqueness.** A phrase + an empty passphrase is one
  wallet; the same phrase + a passphrase is a *different* wallet.
  This is the BIP-39 passphrase / "13th word" convention.

## Wordlist sanity

The 2048-word wordlist is *sorted* and curated to:

- Have unique 4-letter prefixes — "abandon" and "ability" differ in
  the 4th character, never in the first three.
- Avoid pairs that are easily confused (no `won` / `one`, no
  `their` / `there`).
- Be ASCII-only (with non-English wordlists similarly curated for
  their respective alphabets).

This makes recovery from partial readability practical: typically
the first four letters of each word disambiguate.

## Multi-language wordlists

BIP-39 standardises ten wordlists: English (default), Japanese,
Korean, Spanish, Simplified Chinese, Traditional Chinese, French,
Italian, Czech, Portuguese. The toolkit accepts all ten via
`--language <LANGUAGE>`.

Two wallets created from the same entropy in different wordlists
recover to the same wallet — the entropy bits are language-agnostic,
the wordlist is just a presentation layer.

## Why ms1 instead of just engraving the BIP-39 phrase?

The m-format ms1 card carries the *entropy* (not the phrase) under
a BCH error-correction layer:

| Property | BIP-39 phrase on steel | ms1 on steel |
|---|---|---|
| Per-character checksum | no | yes (BCH) |
| Locatable error position | no | yes |
| Engraving alphabet | English (or other) words; full Roman | 32-character codex32 alphabet |
| Recoverability from N stamping errors | depends on prefix readability | up to a fixed number of bit errors |

The phrase remains valuable as a *mnemonic* — humans can read it,
hardware wallets accept it directly. The ms1 card's role is the
engraved-form-of-record, not a replacement for the phrase.
