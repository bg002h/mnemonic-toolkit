# codex32 and BCH

All three card formats use Bose–Chaudhuri–Hocquenghem (BCH) error-correction codes over GF(32). This chapter covers the cryptographic mechanism at engineering depth — enough to reproduce the checksum computation by hand for a short payload, understand the error-detection / error-correction guarantees, and grasp why md1 and mk1 *fork* the polynomial while ms1 adopts BIP-93 codex32 directly.

For the formal codex32 spec see [BIP-93](https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki); for the original codex32 design analysis see the Pearlwort / Poelstra paper cited in §66.

## What a BCH code does

A BCH code is a class of cyclic block codes parameterised by (a) the field size `q`, (b) the codeword length `n`, (c) the dimension `k` (number of data symbols), and (d) the *designed distance* `δ` (minimum Hamming distance between any two codewords). The codec works in three steps:

1. **Encode.** The encoder appends `n − k` *check symbols* to the `k` data symbols. The check symbols are the unique values that make the full `n`-symbol string a multiple of a fixed *generator polynomial* `g(x)`. Geometrically: the codewords form a subspace of all length-`n` strings; the check symbols project the data onto that subspace.

2. **Detect.** The decoder computes the *syndrome* — the remainder of the received string modulo `g(x)`. Syndrome = 0 iff the received string is a codeword (no errors, or undetectable error pattern). Syndrome ≠ 0 signals one or more errors.

3. **Correct.** When the syndrome is nonzero, the decoder uses the BCH decoding algorithm (Berlekamp–Massey for the error-locator polynomial, then Chien search for the roots) to recover the error positions. The number of correctable errors is bounded by `⌊(δ − 1) / 2⌋`.

The `q = 32` choice in codex32 is what makes the alphabet a 32-character bech32 alphabet — each symbol holds 5 bits. Each card character is one GF(32) symbol; a card with payload length `k` and check length `n − k` is exactly `n` characters long.

## The codex32 alphabet

BIP-93 inherits the bech32 alphabet (introduced for BIP-173 SegWit addresses) with deliberate visual-disambiguation properties:

```text
0123456789  → q p z r y 9 x 8 g f      (no '0' / 'O' confusion)
acdefghjklmnpqrstuvwxyz → 2 t v d w 0 s 3 4 5 h 7 e 6 m u a c l    (no '1' / 'l' confusion)
```

(The actual table is in BIP-93 and bech32. The non-trivial property: every pair of characters that could plausibly be confused on a worn engraving — `0`/`O`, `1`/`l`/`I`, `b`/`6`/`8`, `5`/`S`, `2`/`Z` — maps to characters with *different* GF(32) values, so a misread is detected by the BCH check.)

Each character carries 5 bits, so an `n`-character card holds `5n` bits of (data + check) capacity. The HRP and separator are not in the BCH-protected space; they are read literally and used as cofactors in the polynomial computation (see "HRP mixing" below).

## ms1: BIP-93 codex32 direct

ms1 uses the codex32 polynomial verbatim, as specified in BIP-93. The `ms-codec` crate calls into `rust-codex32` for both encode (compute check symbols, emit the card string) and decode (parse the card, verify the checksum, return the payload). There is no md1- or mk1-style fork.

The implication for cross-implementation work: an ms1 implementation in any other language can be built on any conforming BIP-93 codex32 library; ms1 *adds* nothing to the polynomial computation. The ms1 wire format (§II.3) describes only what payload bytes go into the codex32 envelope, not the BCH computation itself.

## md1 and mk1: forked BCH plumbing

md1 and mk1 use BCH codes over GF(32) with the *same algebraic machinery* as codex32 (same alphabet, same field, same encoding/decoding algorithms) but **different generator polynomial constants**.

Two BCH codes are defined per format:

- **Regular code.** Shorter check length; used when the payload fits in the regular envelope.
- **Long code.** Longer check length; used for larger payloads that exceed the regular envelope.

Each generator polynomial is selected to satisfy the same BIP-93 design distance properties — guaranteed correction of up to 4 unknown-position substitutions, or up to 8 known-position erasures, or up to 13 consecutive erasures — but the polynomial *coefficients* differ between codex32, md1, and mk1.

The polynomial constants are pinned in each crate's `bch.rs` module and documented in the corresponding BIP draft.

### HRP mixing

The crucial difference between codex32 and the md1/mk1 fork is **how the human-readable prefix is mixed in**. Codex32 mixes the HRP `ms` via a specific scheme in BIP-93. md1 and mk1 use a *different* HRP-mixing convention that produces a *per-format target residue* — when the BCH polynomial division is computed over `[HRP || separator || data || check]`, a conforming md1 card divides to leave residue `R_md1`; a conforming mk1 card leaves `R_mk1`; a conforming ms1 (codex32) card leaves `0` (BIP-93 convention).

Consequences:

1. **No cross-format confusion.** A decoder applying mk1's polynomial + target-residue to an md1 string fails the checksum check, regardless of payload content. The forked encoding prevents an md1 card from being mistakenly decoded as an mk1 card or vice versa, even when the character set overlaps.

2. **Format identification by prefix alone.** The HRP (`ms` / `mk` / `md`) is structurally inseparable from the BCH check. Stripping the HRP before BCH verification produces a string that does not pass any of the three formats' checks. Format identification is therefore: read the HRP, dispatch to the correct codec, run its BCH check.

3. **md1 and mk1 are distinct in algebra.** Even sharing the *same character* alphabet and the *same field* (GF(32)), md1 and mk1 cards have different generator polynomials. There is no possibility of an mk1 codeword accidentally validating as an md1 codeword.

### Why fork at all?

The decision tree:

- **ms1's payload (BIP-39 entropy / BIP-32 master seed)** is exactly what BIP-93 codex32 was designed for. The K-of-N share encoding (planned for ms-codec v0.2) is also already specified in BIP-93. ms1 adopting codex32 *directly* is value-aligned: no extra spec surface, no risk of divergence from BIP-93.
- **md1's payload (BIP-388 wallet-policy templates)** is not what codex32 was designed for. md1's wire format is a bit-aligned bytecode (§II.1) — a different shape than codex32's "secret blob with optional sharing." Forking the BCH plumbing lets md1 carry an arbitrary bit-aligned payload in a codex32-style envelope, with its own HRP-mixed BCH for cross-format safety.
- **mk1's payload (xpub + origin metadata)** is similarly outside codex32's scope. The same forked-BCH-plus-HRP-mixing pattern applies.

A previously-planned `mc-codex32` shared-crate extraction (sharing the forked-BCH plumbing between md1 and mk1 as a single dependency) was retired on 2026-05-03 — the HRP-mixed BCH isn't generic enough to be useful outside the m-format pair. md1 and mk1 maintain their own `bch.rs` modules; the *pattern* will be documented in a future cross-repo `PATTERNS.md`.

## Worked decode example (no errors)

The cleanest way to convince yourself the BCH math is concrete: walk through a short md1 card by hand. Take the corpus vector for `wpkh_basic`:

| Part | Value |
|---|---|
| HRP | `md` |
| Separator | `1` |
| Data symbols | `yqpqqxqq` |
| Check symbols | `8xtwhw4xwn4qh` |
| Full card | `md1yqpqqxqq8xtwhw4xwn4qh` |

Each character (data + check) is one GF(32) symbol. The decoder:

1. Strips `md1` (HRP + separator); retains `yqpqqxqq8xtwhw4xwn4qh` as the GF(32)-symbol stream.
2. Computes the BCH syndrome by polynomial division: feed the stream + HRP-mixed initial state into the polynomial computation; read off the residue.
3. For a conforming md1 card, the residue equals the target residue `R_md1` (the format-specific constant pinned in `md-codec`'s `bch.rs`).
4. Decoder proceeds to parse the data symbols into the v0.30 wire format payload (§II.1).

For an actual bit-by-bit trace of the polynomial computation, see the reference implementation at `crates/md-codec/src/bch.rs`. The hand-computation is mechanically straightforward but symbol-dense; the worked example here is the high-level shape only.

## Error-detection guarantees

The BCH codes in all three formats give the same design distance, inherited from BIP-93:

| Error pattern | Regular code | Long code |
|---|---|---|
| Detected (any pattern up to 8 random symbol errors) | Always | Always |
| Corrected (random-position substitutions) | Up to 4 | Up to 4 |
| Corrected (known-position erasures) | Up to 8 | Up to 8 |
| Corrected (consecutive erasures, e.g., one continuous burn mark) | Up to 13 | Up to 13 |

"Always detected" here means: any error pattern outside the corrected range either produces a different valid codeword (extremely rare — designed-against) or fails the check. For real-world engraving damage (a scratch obscuring 1–3 characters, light pitting on a small contiguous run), the codes are over-engineered.

What the BCH does *not* protect against:

- **Wrong HRP transcribed.** If the user engraves the HRP wrong (e.g., types `mk1...` for what should be `md1...`), the polynomial check fails. No correction is attempted across the wrong-HRP boundary; the codec returns an error and the user re-checks the HRP. This is the right behavior: a wrong HRP is almost certainly a typo, not a damage event, and silently auto-correcting it would risk decoding the wrong payload.
- **Cross-card binding.** The BCH guarantees a *single card* is intact. Cross-card invariants (the `policy_id_stub` on mk1, the multiset xpub-match rule, etc.) are computed at the toolkit layer after each card decodes individually. The BCH cannot detect a bundle assembled from cards belonging to different wallets — that's what `mnemonic verify-bundle` is for.

## Long vs regular code dispatch

Per format, the encoder selects regular or long code based on the payload size. The dispatch is deterministic and visible:

- The first symbol of the card (after `<HRP>1`) carries a *length indicator* that the decoder reads to determine which code applies. md1 documents this in §II.1.1 (header bit 0 carries the dispatch).
- An mismatched dispatch (e.g., a regular-code card whose length indicator says "long") fails BCH verification at the codeword-length step before any payload parsing occurs.

The `--force-long-code` CLI flag (on `md encode`) lets an operator force the long code even when the regular would suffice — useful for cross-implementation conformance testing.

## Summary

- All three formats use BCH codes over GF(32) with the BIP-93 design distance.
- ms1 uses BIP-93 codex32 directly via `rust-codex32`.
- md1 and mk1 use a forked BCH with HRP-mixed per-format target residues, sharing the algebra but not the polynomial constants.
- The forked-vs-direct split is deliberate; the `mc-codex32` shared-crate extraction was retired in 2026-05-03.
- Cross-card binding (the `policy_id_stub` invariant set) is enforced at the toolkit layer, not the BCH layer.

The wire-format chapters (§II.1 / §II.2 / §II.3) describe what the data symbols *encode*, which is the next layer above the BCH plumbing.
