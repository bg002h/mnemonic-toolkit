# R0 Architect Gate — Round 2 — SPEC_friendly_tests_and_chunk_mk1.md

> Round 1 = 0C/2I/5M; all folded. Reviewer had Read/Grep/WebFetch; parent persists.

**Verdict: 0 Critical / 1 Important / 0 Minor — NOT GREEN.**

The I2 fold corrected bip39 (four→three) but stopped one step short: it conflated *enum-level* `#[non_exhaustive]` with *mapper-level* fallthrough risk and left `friendly_bitcoin` mis-classified. The bare-`_`-wildcard mapper set is **two** (ms_codec, mk_codec), not three. Grep-confirmed against source.

## Critical
None.

## Important

**I-new-1 — `friendly_bitcoin` mis-classified as a `_`-wildcard / non_exhaustive mapper; the count is two, not three.**
`friendly_bitcoin` (`friendly.rs:34-40`) is a 3-arm exhaustive match on the **toolkit-local** `BitcoinErrorKind` (`Bip32`, `XpubParse`, `FingerprintParse`) with **NO bare `_` arm**. The `#[non_exhaustive]` `bitcoin::bip32::Error` is never matched variant-by-variant — it is the payload of the `Bip32(b)` arm, Display-forwarded (`format!("BIP-32 error: {}", b)`). At the mapper level bitcoin behaves like the closed mappers: all 3 arms testable, no fallthrough net, a future bip32 variant flows through `Bip32`→Display (testably, never "unhandled").
Grep-confirmed: only `friendly_ms_codec` (`:129`) and `friendly_mk_codec` (`:181`) carry a bare `_ => "unhandled … {:?}"` arm; bip39, bitcoin, md_codec have none.
The SPEC draws mapper-level consequences from enum-level non_exhaustiveness and they're false for bitcoin at four sites: `:30` (lists bitcoin in the `_`-wildcard set of three), `:44` (out-of-scope: "three" mappers' `_` arms untestable — bitcoin has none), `:46`/M5 (`!contains("unhandled")` load-bearing for bitcoin — vacuous: no `_` arm), and it contradicts the SPEC's own `:42` ("all 3 bitcoin arms, no untestable `_`").
**Fix:** bare-`_`-wildcard set = **two** (ms_codec, mk_codec). Closed/no-`_` set = md_codec, bip39, **and bitcoin** (three). Update `:30`, `:44`, `:46`, and broaden the module-doc fix (`:4-6` must drop **both** bip39 and bitcoin from the `_`-set, not bip39 alone). I2 should have been four→**two**, demoting bitcoin as well.

## Minor
None new.

## Fold confirmation (R1 findings)
- **I1 — clean.** §3.1 now `cargo test -p mnemonic-toolkit --bin mnemonic friendly`; source-confirmed `main.rs:16 mod friendly;` (not lib); no `--lib` remains.
- **I2 — PARTIAL / re-opened (I-new-1).** bip39 correctly reclassified closed (5 arms, no `_`, source-confirmed); module-doc fix prescribed. But still wrongly counts three non_exhaustive mappers incl. bitcoin; correct is two.
- **M1 — clean.** `AmbiguousLanguages` dropped; bip39 = 3 constructible arms. Inner `struct AmbiguousLanguages([bool; MAX_NB_LANGUAGES])` has a private field + no public ctor → unbuildable; mapper's `AmbiguousLanguages(_)` pattern unaffected.
- **M2 — clean.** `emit`→`emit_unified` in SPEC body + ship-step; source-confirmed `fn emit_unified` at `bundle.rs:778`.
- **M3/M4 — clean.** §1 adds remove-`#[allow]`-format.rs:32 + reword-format.rs:28-31-comment; both source-confirmed.
- **M5 — clean as principle, membership inherits I-new-1** (bitcoin must move to the vacuous group).

## Substance (intact)
Chunk sites (ms1 :951 stays; mk1 :962/:974 swap; import :7), byte-identical (`format.rs:33` = `{chunk_5char(s)}`), arm counts (md_codec ~44/0-tested biggest gap), constructibility resolution, no-bump/no-lockstep disposition — all confirmed.

**Re-dispatch after folding I-new-1** (narrow: move bitcoin to closed/vacuous at :30/:44/:46, module-doc :4-6 drop bitcoin too, "three"→"two"). One more round → GREEN.
