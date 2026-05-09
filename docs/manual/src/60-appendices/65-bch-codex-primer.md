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
- If a few errors: the polynomial syndrome *locates* the error
  positions. BIP-93 guarantees correction of up to 4 unknown-position
  substitutions, or up to 8 errors at known positions ("erasures"),
  or up to 13 consecutive erasures. See *Error detection and
  correction guarantees per card* below for the precise table.
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

The toolkit picks the right code automatically based on payload
size; no user flag is needed. (`md encode --force-long-code` is
accepted on the binary as a forward-compat scaffold but is a
documented no-op since md-codec v0.12.0; long-code mode for md1
was dropped on the codec side.)

## Error detection and correction guarantees per card

The numbers below are taken **verbatim from BIP-93** §"Error
Correction" and §"Generating the Checksum". They apply to every
BCH variant in the constellation that shares codex32-pattern
parameters: the minimum distance is set by the generator
polynomial, which is identical across ms1, mk1, and md1; only the
target residue differs (HRP-mixed `"mk"` / `"md"` for the forked
codes, BIP-93 stock for ms1).

### Regular code (n=93 symbols, k=80, 13-symbol checksum)

Used by ms1 (BIP-93 directly), and by mk1 + md1 (forked, same
generator polynomial). Applies to data strings of ≤93 bech32
characters.

| Property | Guarantee | Source |
|---|---|---|
| Detection — characters affected | ≥ 1 and ≤ 8 errors guaranteed detected | BIP-93 §"Generating the Checksum" |
| Detection — random patterns beyond 8 errors | < 3 × 10⁻²⁰ probability of missed detection | BIP-93 §"Generating the Checksum" |
| Correction — substitutions (unknown positions) | up to **4** substitutions corrected | BIP-93 §"Error Correction" |
| Correction — erasures (known positions, e.g., `?`) | up to **8** erasures corrected | BIP-93 §"Error Correction" |
| Correction — consecutive erasures (a single scratch) | up to **13** consecutive erasures corrected | BIP-93 §"Error Correction" |

### Long code (n=108 symbols, k=93, 15-symbol checksum)

Used by mk1 only (mk1 chunked-mode strings of 96–108 chars). md1
dropped the long code in md-codec v0.11; ms1 v0.1 payloads always
fit in the regular bracket.

| Property | Guarantee | Source |
|---|---|---|
| Detection — characters affected | ≥ 1 and ≤ 8 errors guaranteed detected | BIP-93 §"Long codex32 Strings" |
| Detection — random patterns beyond 8 errors | < 3 × 10⁻²³ probability of missed detection | BIP-93 §"Long codex32 Strings" |
| Correction (substitutions / erasures / consecutive erasures) | same as regular code: 4 / 8 / 13 | BIP-93 §"Error Correction" applied to the longer code |

### How the code variant is chosen per card

| Card | Payload kind (v0.1) | Typical string length | Code variant |
|---|---|---|---|
| ms1 | BIP-39 entropy 16–32 B (`entr`) | ~70 chars | regular |
| mk1 | xpub + origin (single-string mode) | ~52–55 chars | regular |
| mk1 | xpub + origin (chunked mode, longer paths) | 96–108 chars | long |
| md1 | wallet policy | 75–93 chars | regular only (v0.11+) |

The toolkit picks the variant automatically based on data length;
no user flag is needed. (The "typical string length" column is a
best-read estimate pending an empirical sweep across all payload
kinds; tracked as `bch-string-length-empirical-sweep` in
`docs/manual/FOLLOWUPS.md`.)

### One subtlety: HRP mixing does not change the correction guarantees

The forked BCH for mk1 and md1 changes the **target residue** —
the constant the polymod is XOR'd against at encode time and
compared against at decode time. It does **not** change the
**generator polynomial** or the **field arithmetic**, both of
which are inherited byte-for-byte from BIP-93. Minimum distance,
and therefore the detection / correction guarantees in the tables
above, are properties of the generator polynomial alone — so the
BIP-93 numbers transfer to mk1 and md1 unchanged. What HRP mixing
buys is **format separation**: an mk1 string can never accidentally
validate as an md1 or ms1 string, because the target residues
disagree.

### Errors the BCH code does NOT handle: deletions and insertions

An "erasure" in the table above means *the character at a known
position is unreadable, but the position itself is preserved*
(typically rendered `?`). The string length is unchanged.

A **deletion** (a missing character that causes everything after
it to shift left) is a length-changing event and is **outside the
BCH code's correction model**. Same for **insertions** (extra
character shifts everything right).

In practice:

- **Detection of length-changing damage is overwhelming.** The
  decoder first checks that the string length matches one of the
  expected lengths (≤93 for regular, 96–108 for long). One deletion
  drops the length by 1; if the encoded length was a known fixed
  value for the payload kind, the decoder rejects on length
  mismatch *before* even running the polynomial. Even when a
  deletion happens to be paired with a compensating insertion (so
  the length comes out right), ~half the symbols end up in wrong
  positions and the polynomial syndrome is wildly non-zero. The
  probability of length-changing damage silently passing the
  checksum is comparable to the < 3 × 10⁻²⁰ general missed-detection
  bound.
- **Correction of length-changing damage is not supported.** The
  decoder reports "string length out of range" or "checksum
  invalid" without identifying the missing or extra character's
  position. The polynomial syndrome encodes substitution-error
  positions, not insertion / deletion positions.
- **First line of recovery: count the characters on the plate.**
  Each card has a known string length. If the count is off,
  recount the engraved plate before attempting to decode — the
  position of the missing or extra character is something only
  physical inspection can recover. This is one practical reason
  the toolkit emits both the contiguous form (handy for
  copy-paste) and the chunked-in-fives form (handy for engraving)
  of every string: chunking makes character counts trivially
  auditable on the plate.

## Hand-decodability of ms1 via the bundled codex32 paper-computer

ms1 (HRP `ms`) uses BIP-93 codex32 directly via `rust-codex32` —
identical generator polynomial, identical target residue
`MS32_CONST = 0x10ce0795c2fd1e62a`, identical alphabet. The codex32
hand-computation toolkit therefore decodes and verifies ms1 strings
unmodified. A copy of the upstream PDF is bundled in this repo at
[`docs/codex32/2023-03-07--color.pdf`](../../../codex32/2023-03-07--color.pdf)
(MIT-licensed; see `docs/codex32/README.md` for attribution,
provenance, and SHA-256 pin) so users can verify ms1 strings
offline without depending on `secretcodex32.com` staying online.

The codex32 hand-computation toolkit is richer than a single
rotating disc: it ships a Checksum Worksheet (a triangular grid
the user fills in two characters at a time), a 1024-entry Checksum
Table indexed by 2-character bech32 pairs (the actual BCH
polymod-step lookup), an Addition wheel for GF(32) XOR, a dice
de-biasing worksheet for generating secret material, and three
additional rotating discs for Shamir share arithmetic (Recovery,
Translation, Fusion). Hand-verification of an ms1 string follows
the bundled PDF's *Checksum Worksheet (Verification Instructions)*
page directly.

### mk1 and md1: hand-decodability is out of scope

The mk1 (HRP `mk`) and md1 (HRP `md`) cards use forked BCH
plumbing — different target residues (`MK_REGULAR_CONST =
0x1062435f91072fa5c`, `MK_LONG_CONST = 0x41890d7e441cbe97273`,
`MD_REGULAR_CONST = 0x0815c07747a3392e7`) for cryptographic format
separation, and (for mk1-long) a different generator polynomial.

A v0.1 cycle shipped per-format hand-computation discs for these
cards, but a post-ship audit against the codex32 toolkit found
those discs structurally insufficient: a 32×32 cell grid exposes
only the LOW-5-bits of one polymod step, discarding the upper
55–65 bits of state the user needs to carry forward to the next
step. Codex32's hand-computation works through the combined
Checksum Table, Worksheet, and Addition wheel; a single-step
polymod disc cannot substitute. A v0.2 derivative covering mk1
and md1 would have
required ~6 pages of dense per-format lookup tables plus
per-format worksheets — substantial work for cards that carry
public material (xpub + origin metadata for mk1; wallet policy
template for md1) where hand-decodability is not load-bearing.

The strategic decision was therefore to retire the v0.1
deliverables and not pursue v0.2 derivatives. Hand-verification
for mk1 and md1 is performed at decode time by the toolkit's CLIs
(`mnemonic verify-bundle`, `mk verify`, `md verify`); see Chapter 4
for the operational details. The audit findings and the cycle
that retired the v0.1 deliverables are recorded under the
manual-v0.1.8 unified closure note in the *Closed* section of
`docs/manual/FOLLOWUPS.md`.

### Why HRP mixing was accepted at this cost

Format separation. An mk1 string cannot accidentally validate as
an ms1 or md1 string at decode time, because a wallet decoding
mk1 compares the polymod against `MK_REGULAR_CONST` /
`MK_LONG_CONST` while a wallet decoding ms1 compares against
`MS32_CONST`. The probability that random bit-flipping turns a
valid ms1 string into a valid mk1 string is the probability of
the polymod landing on `MK_REGULAR_CONST` instead of
`MS32_CONST` — combinatorially implausible with NUMS-derived
constants.

The trade-off: stock-codex32 hand-decodability for mk1 / md1 was
given up in exchange for cryptographically strong format
separation. ms1 retains direct codex32 compatibility because it
uses BIP-93 unchanged. The practical recipe today is:

- For ms1 hand-verification, print the bundled
  [codex32 paper-computer PDF](../../../codex32/2023-03-07--color.pdf)
  and follow its Checksum Worksheet instructions verbatim.
- For mk1 and md1, hand-verification is not in scope; the BCH
  checksums are verified at decode time by the toolkit's CLIs
  (`mnemonic verify-bundle`, `mk verify`, `md verify`).

### Sources for further reading

- **[BIP-93](https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki)** — the canonical codex32 specification, including the §"Error Correction" claims quoted above and a Python reference implementation of the polynomial.
- **[Bundled codex32 paper-computer PDF](../../../codex32/2023-03-07--color.pdf)** (`docs/codex32/2023-03-07--color.pdf`) — Curr & Snead's original Shamir Secret Sharing paper-computer toolkit, MIT-licensed; see `docs/codex32/README.md` for attribution, provenance, and SHA-256 pin.
- **[apoelstra/rust-codex32](https://github.com/apoelstra/rust-codex32)** — Andrew Poelstra's CC0 Rust reference implementation. ms-codec depends on `codex32 = "=0.1.0"` directly via this crate.
- mk1's forked BCH plumbing: `crates/mk-codec/src/string_layer/bch.rs` in the [mnemonic-key](https://github.com/bg002h/mnemonic-key) repo.
- md1's forked BCH plumbing: `crates/md-codec/src/bch.rs` in the [descriptor-mnemonic](https://github.com/bg002h/descriptor-mnemonic) repo.
- The `mc-codex32` shared-crate extraction (which would have unified the mk1/md1 fork with rust-codex32) was considered and retired on 2026-05-03; see CLAUDE.md in any of the four repos for the coordination record.

## Operational fallbacks

When stamping errors exceed the per-card correction radius (4
substitutions or 8 erasures in known positions), the fall-backs are:

- **Single-sig:** re-derive the lost card from the seed phrase
  plus the wallet template — `mk1` and `md1` are public material
  and are reconstructable from `ms1` plus knowledge of the
  template (chapter 35 walks through every scenario).
- **Multisig:** for a 2-of-3 wallet, any one cosigner's `ms1` may
  be lost without losing spending capability (the threshold absorbs
  it); two cosigner `ms1` losses are fatal.

When length-changing damage (deletion / insertion) is suspected
but the character count looks right, the operator should manually
re-decode each five-character chunk against the original digital
bundle to spot the mis-aligned chunk.
