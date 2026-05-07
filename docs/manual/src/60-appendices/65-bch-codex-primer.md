# Appendix E — codex32 / BCH / m-codec error correction

The three card formats (ms1, mk1, md1) all use Bose–Chaudhuri–
Hocquenghem (BCH) error-correction codes. ms1 uses BIP-93 codex32
directly; md1 and mk1 fork a Bitcoin-tuned BCH polynomial. This
appendix sketches *why* and *how* the codes catch and locate
engraving errors.

For the formal specs, see
[BIP-93 (codex32)](https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki)
and the descriptor-mnemonic / mnemonic-key BIP drafts.

## What a BCH code does

Given a stream of data symbols (each from a 32-character alphabet),
a BCH encoder appends a few extra symbols computed by polynomial
arithmetic. The decoder, on receiving the stream + checksum,
recomputes the polynomial and:

- If no errors: the recomputed checksum matches; data is intact.
- If a few errors: the polynomial mismatch *locates* the error
  positions (typically up to 4 incorrect characters in the standard
  codex32 setting).
- If many errors: the polynomial mismatch is uncorrectable; decode
  fails.

The "few" vs "many" threshold is determined by the polynomial's
distance — a property fixed by the polynomial choice, not
configurable per-card.

## codex32: BIP-93's contribution

Codex32 specifies:

- An alphabet of 32 characters chosen to avoid visual ambiguity
  (no `0` / `O`, no `1` / `l`, no `b` / `6` / `8`).
- A BCH polynomial tuned for the codex32 alphabet over GF(32).
- A human-readable prefix mechanism.
- Optional K-of-N share-splitting: one secret splits into N shares,
  any K reconstruct.

For ms1 (the secret card), codex32's single-string mode is used
verbatim via the upstream `rust-codex32` crate. The K-of-N share
mode is planned for ms-codec v0.2.

## md / mk: forked BCH plumbing

The md1 and mk1 cards use a *forked* BCH polynomial: same algebraic
machinery as codex32, different polynomial constants (tuned for
md1's and mk1's payload sizes and HRP-mixing requirements). The
fork is an explicit design choice in the descriptor-mnemonic and
mnemonic-key BIP drafts; a future `mc-codex32` extraction was
considered and *retired* in 2026-05-03 (see the project's CLAUDE.md
for the cross-repo coordination record).

## HRP mixing

The md / mk BCH variant mixes the human-readable prefix (`md` or
`mk`) into the polynomial computation. Two consequences:

- A card decoded with the wrong HRP fails the BCH check (no risk
  of mistaking an md1 string for an mk1 string, or vice versa).
- The polynomial constants for md1 and mk1 are distinct, even
  though the algebra is identical.

Codex32 has its own HRP mechanism in BIP-93; ms1 inherits that.

## Long vs regular code

Each format has two BCH codes:

- **Regular code** — shorter strings, fewer codewords needed,
  smaller checksum overhead. Used when the payload fits in the
  regular layout.
- **Long code** — longer strings, larger checksum overhead, used
  when the payload requires the extended layout (e.g., a
  multi-cosigner mk1 with many path components).

The toolkit picks the right code automatically; users never see
this dimension unless they pass `--force-long-code` (a debug flag)
or the codec emits a long-code string due to payload size.

## Error-correction limits in practice

A handful of stamping errors per card are correctable; an entire
character-row mis-stamping is not. The codec reports error
positions, so the operator can manually correct against the
original digital bundle. Beyond the correction radius, the
fall-back is re-deriving from the seed (single-sig) or from
cosigner cooperation (multisig).
