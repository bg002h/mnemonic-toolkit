# codex32 and BCH

All three card formats use Bose–Chaudhuri–Hocquenghem (BCH)\index{BCH code} error-correction codes over GF(32)\index{GF(32)}. This chapter covers the cryptographic mechanism at engineering depth — enough to reproduce the checksum computation by hand for a short payload, understand the error-detection / error-correction guarantees, and grasp why md1 and mk1 *fork* the polynomial while ms1 adopts BIP-93\index{BIP-93} codex32\index{codex32} directly.

For the formal codex32 spec see [BIP-93](https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki); for the original codex32 design analysis see the Pearlwort / Poelstra paper cited in §66.

## What a BCH code does

A BCH code is a class of cyclic block codes parameterised by (a) the field size `q`, (b) the codeword length `n`, (c) the dimension `k` (number of data symbols), and (d) the *designed distance* `δ` (minimum Hamming distance between any two codewords). The codec works in three steps:

1. **Encode.** The encoder appends `n − k` *check symbols* to the `k` data symbols. The check symbols are the unique values that make the full `n`-symbol string a multiple of a fixed *generator polynomial*\index{generator polynomial} `g(x)`. Geometrically: the codewords form a subspace of all length-`n` strings; the check symbols project the data onto that subspace.

2. **Detect.** The decoder computes the *syndrome* — the remainder of the received string modulo `g(x)`. Syndrome = 0 iff the received string is a codeword (no errors, or undetectable error pattern). Syndrome ≠ 0 signals one or more errors.

3. **Correct.** When the syndrome is nonzero, the decoder uses the BCH decoding algorithm (Berlekamp–Massey for the error-locator polynomial, then Chien search for the roots) to recover the error positions. The number of correctable errors is bounded by `⌊(δ − 1) / 2⌋`.

The `q = 32` choice in codex32 is what makes the alphabet a 32-character bech32 alphabet — each symbol holds 5 bits. Each card character is one GF(32) symbol; a card with payload length `k` and check length `n − k` is exactly `n` characters long.

## The codex32 alphabet

BIP-93 inherits the bech32\index{bech32} alphabet (introduced for BIP-173\index{BIP-173} SegWit addresses) with deliberate visual-disambiguation properties:

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

md1 and mk1 use BCH codes over GF(32) with the *same algebraic machinery* as codex32 — same alphabet, same field, same encoding/decoding algorithms, **same generator polynomial coefficients**. The fork is not in the polynomial; it is in the **target residue** and the **HRP-mixing convention** layered on top of the shared generator.

md1's reference implementation pins one BCH code (the v0.11+ regular code\index{regular code}; the historical long code\index{long code} was retired together with the v0.x wire format). The generator coefficients (`GEN_REGULAR` in `crates/md-codec/src/bch.rs`) are the standard BCH(93,80,8) polynomial used across the bech32/codex32 family. The per-format differentiation appears in `MD_REGULAR_CONST`\index{MD\_REGULAR\_CONST} — a non-zero 65-bit constant derived from the top 65 bits of `SHA-256("shibbolethnums")` — that the verifier compares the polymod\index{polymod} output against. The format-specific constant is the **target residue**\index{target residue}. mk1's `bch.rs` follows the same shape with its own format-specific constant.

The BIP-93 design distance\index{BIP-93 design distance} properties — guaranteed correction of up to 4 unknown-position substitutions, or up to 8 known-position erasures, or up to 13 consecutive erasures — carry through because the *generator* is the same. The target-residue swap doesn't change the distance, only which polymod output counts as "valid" for the format.

### HRP mixing

The HRP is mixed in via the standard **BIP-173 HRP expansion**\index{HRP-mixing}: each character of the HRP contributes two values to the polymod input, namely `c >> 5` (the high-3-bit half) and `c & 31` (the low-5-bit half), with a zero separator between the two halves. The expanded HRP is *prepended* to the data + checksum stream before the polymod runs; the generator itself is unchanged. The reference implementation is at `crates/md-codec/src/bch.rs::hrp_expand`.

The verify path is therefore:

1. Take `hrp_expand(hrp) || data_symbols || check_symbols` as a flat GF(32)-symbol stream.
2. Compute the polymod from a fixed initial state (`POLYMOD_INIT`).
3. Compare the result against the format-specific target residue (`MD_REGULAR_CONST` for md1; a different constant for mk1; `0` for ms1 via BIP-93 codex32).

Consequences:

1. **No cross-format confusion.** A decoder applying mk1's target-residue check to an md1 string fails the comparison, regardless of payload content. The forked encoding prevents an md1 card from being mistakenly decoded as an mk1 card or vice versa, even when the character set overlaps.
2. **Format identification by prefix alone.** The HRP (`ms` / `mk` / `md`) is structurally inseparable from the polymod input. Stripping the HRP before BCH verification produces a different polymod path that matches no format's target residue. Format identification is therefore: read the HRP, dispatch to the correct codec, run its polymod with HRP-expanded prefix, compare against the format's target residue.
3. **md1 and mk1 are distinct in the residue, not the polynomial.** Same generator polynomial, different target constants. There is no possibility of an mk1 codeword's polymod output accidentally matching `MD_REGULAR_CONST` (or vice versa) for any non-trivial payload.

### Why fork at all?

The decision tree:

- **ms1's payload (BIP-39 entropy / BIP-32 master seed)** is exactly what BIP-93 codex32 was designed for. The K-of-N share encoding (planned for ms-codec v0.2) is also already specified in BIP-93. ms1 adopting codex32 *directly* is value-aligned: no extra spec surface, no risk of divergence from BIP-93.
- **md1's payload (BIP-388 wallet-policy templates)** is not what codex32 was designed for. md1's wire format is a bit-aligned bytecode (§II.1) — a different shape than codex32's "secret blob with optional sharing." Forking the BCH plumbing lets md1 carry an arbitrary bit-aligned payload in a codex32-style envelope, with its own HRP-mixed BCH for cross-format safety.
- **mk1's payload (xpub + origin metadata)** is similarly outside codex32's scope. The same forked-BCH-plus-HRP-mixing pattern applies.

A previously-planned `mc-codex32` shared-crate extraction (sharing the forked-BCH plumbing between md1 and mk1 as a single dependency) was retired on 2026-05-03 — the HRP-mixed BCH is not generic enough to be useful outside the m-format pair. md1 and mk1 maintain their own `bch.rs` modules; the *pattern* will be documented in a future cross-repo `PATTERNS.md`.

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

md1 and mk1 ship with one BCH code each (the regular code; the historical long code was retired in md-codec v0.12 along with the rest of the v0.x wire format). ms1 inherits both regular and long codes from BIP-93 codex32. The guaranteed-correction counts follow from the BIP-93 BCH design distance:

| Error pattern | md1 / mk1 (regular only) | ms1 regular (BIP-93) | ms1 long (BIP-93) |
|---|---|---|---|
| Corrected (random-position substitutions) | Up to 4 | Up to 4 | Up to 4 |
| Corrected (known-position erasures) | Up to 8 | Up to 8 | Up to 8 |
| Corrected (consecutive erasures — single contiguous burn mark) | Up to 13 | Up to 13 | Up to 15 |

Any error pattern outside the corrected range either produces a different valid codeword (extremely rare; designed-against) or fails the check. For real-world engraving damage (a scratch obscuring 1–3 characters, light pitting on a small contiguous run), the codes are over-engineered.

What the BCH does *not* protect against:

- **Wrong HRP transcribed.** If the user engraves the HRP wrong (e.g., types `mk1...` for what should be `md1...`), the polynomial check fails. No correction is attempted across the wrong-HRP boundary; the codec returns an error and the user re-checks the HRP. This is the right behavior: a wrong HRP is almost certainly a typo, not a damage event, and silently auto-correcting it would risk decoding the wrong payload.
- **Cross-card binding.** The BCH guarantees a *single card* is intact. Cross-card invariants (the `policy_id_stub` on mk1, the multiset xpub-match rule, etc.) are computed at the toolkit layer after each card decodes individually. The BCH cannot detect a bundle assembled from cards belonging to different wallets — that's what `mnemonic verify-bundle` is for.

## Note on the retired long code

md1 carried a long code in v0.x — used for payloads exceeding the regular envelope. v0.12 introduced bit-aligned chunking (multiple regular-code cards carrying a chunked payload), which subsumed the long code's role; the long-code path was dropped at the same time. The `md` CLI retains `--force-long-code` as a no-op flag for backward-compat in pipelines that pass it; the flag has no effect at v0.30+.

ms1 inherits BIP-93's regular + long codes unchanged. mk1's situation mirrors md1's (regular only).

## Summary

- All three formats use BCH codes over GF(32) with the BIP-93 design distance.
- ms1 uses BIP-93 codex32 directly via `rust-codex32`.
- md1 and mk1 share the BIP-93 generator polynomial and the standard BIP-173 HRP-expansion scheme; they differ from codex32 (and from each other) only in the **target residue** the verifier compares the polymod output against.
- The forked-vs-direct split is deliberate; the `mc-codex32` shared-crate extraction was retired on 2026-05-03.
- Cross-card binding (the `policy_id_stub` invariant set) is enforced at the toolkit layer, not the BCH layer.

The wire-format chapters (§II.1 / §II.2 / §II.3) describe what the data symbols *encode*, which is the next layer above the BCH plumbing.
