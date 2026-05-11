# tech-manual v0.2.0 — Phase 2.1 reviewer report (r1)

| Field | Value |
|---|---|
| Cut | `tech-manual-v0.2.0` |
| Phase | 2.1 (Part III §III.1 — Descriptor → miniscript → address) |
| Commit under review | `1a8f8cc` |
| Date | 2026-05-11 |
| Reviewer | `feature-dev:code-reviewer` |
| Files in scope | `docs/technical-manual/src/30-address-derivation/31-descriptor-to-miniscript.md` + supporting (`62-index-table.md` rows, `.cspell.json` regex/word, two transcripts, two rendered figures) |

## Findings: 0 Critical / 2 Important / 1 Low / 1 Nit

---

## Important

**I-1. Fabricated BIP section titles — BIP-388 and BIP-389**

`31-descriptor-to-miniscript.md:41` cites "BIP-388 §'Wallet policies'" and `:129` cites "BIP-389 §'Multipath descriptors'". Neither section heading exists in the BIPs.

BIP-388's actual heading structure has no "Wallet policies" section; the document's title is "Wallet Policies for Descriptor Wallets" but the normative content lives under "Specification" → "Formal definition". BIP-389 has no "Multipath descriptors" section; the specification content is under "Specification" without that subsection name.

BIP-32 §"Public parent key → public child key" at `:131` is correct — that subsection exists verbatim in BIP-32.

Fix: Change `BIP-388 §"Wallet policies"` to `BIP-388 §"Specification"` (or drop the subsection cite and reference the BIP by number only, as is done for BIP-380 at `:130`). Change `BIP-389 §"Multipath descriptors"` to `BIP-389 §"Specification"`. The Source-pointers block at `:128-129` needs the same correction.

**I-2. §II.1 cross-reference heading strings do not match actual headings**

`31-descriptor-to-miniscript.md:84` references "§II.1 §'History note on retired dictionaries'" and `:112` references "§II.1 §'Worked encode'".

Actual headings in `docs/technical-manual/src/20-wire-formats/21-md1-wire-format.md`:

- Line 267: `## History note: retired wire-layer dictionaries`
- Line 233: `## Worked encode: \`wpkh(@0/<0;1>/*)\` (corpus vector \`wpkh_basic\`)`

If the manual's cross-reference renderer (Pandoc/mdBook) generates anchors from heading text, both links will silently 404. "on retired dictionaries" ≠ "retired wire-layer dictionaries"; "Worked encode" ≠ "Worked encode: …".

Fix at `:84`: Change `§"History note on retired dictionaries"` to `§"History note: retired wire-layer dictionaries"`. Fix at `:112`: shorten to `§II.1`'s worked-encode passage without the subsection anchor.

---

## Low

**L-1. Fourth pre-flight case omitted from pre-flight description**

`31-descriptor-to-miniscript.md:88` states "`derive_address` rejects **three** impossible-by-construction inputs". The implementation at `descriptor-mnemonic/crates/md-codec/src/derive.rs:113-118` has a fourth branch: when `use_site_path.multipath` is `None` (bare `/*`, no multipath group) and `chain != 0`, it returns `Error::ChainIndexOutOfRange { chain, alt_count: 0 }`.

The "chain index out of range" bullet at `:91` says "for a `<0;1>/*` use-site, `chain ∈ {0, 1}`" but never mentions the no-multipath case where only `chain = 0` is valid. A reader implementing a wrapper or writing tests against the documented contract would not know that passing `chain = 1` to a bare-wildcard descriptor is also a pre-flight rejection.

Fix: Change "rejects three impossible-by-construction inputs" to "four" and add a bullet for the no-multipath case.

---

## Nit

**N-1. `derive.rs:81-82` cite points to doc-comment, not generation site**

`31-descriptor-to-miniscript.md:43` cites "`Error::MissingPubkey { idx }` (`derive.rs:81-82`)". Lines 81-82 of derive.rs are a doc-comment bullet inside `derive_address`'s `# Errors` block — they list the error as a possible return, but the actual `Err(Error::MissingPubkey { idx: e.idx })` is generated in `to_miniscript.rs:73` (inside `build_descriptor_public_key`, called transitively from `derive_address`). A reader following the cite to investigate the error path would find prose, not code.

Fix: Change the cite to `(to_miniscript.rs:73)`.

---

## Verified-correct items (no action needed)

- All other line-number citations spot-checked and confirmed accurate: `derive.rs:14-19` (origin-path comment), `derive.rs:92-132` (function body), `to_miniscript.rs:54-64` (`to_miniscript_descriptor`), `to_miniscript.rs:130-168` (`node_to_descriptor`), `to_miniscript.rs:474-476` (`failed`), `origin_path.rs:82-96` (`PathDecl`/`PathDeclPaths`), `origin_path.rs:110-146` (read/write), `origin_path.rs:1-12` (wire layout), `use_site_path.rs:47-96` (`UseSitePath`).
- BIP-84 worked-example address `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu` confirmed against the integration test at `tests/address_derivation.rs:87-90` which derives the same xpub in-process and asserts the same address. Transcript output matches.
- Return type `Address<NetworkUnchecked>` confirmed at `derive.rs:97`.
- Origin-path-not-consulted claim confirmed: `derive.rs:14-19` says exactly that; the implementation calls `to_miniscript_descriptor(self, chain)` and `build_descriptor_public_key` uses only `use_site_path` for the derivation path — origin path is consumed only for the `DescriptorXKey.origin` field (PSBT metadata), not for key derivation arithmetic.
- Header bit 4 / `Tag::OriginPaths = 0x36` claim verified correct: `OriginPaths` and `0x36` are absent from current source; `PathDeclPaths::{Shared,Divergent}` controlled by `divergent_mode` arg (header bit 4) confirmed in `origin_path.rs:134`.
- BIP-32 §"Public parent key → public child key" section title confirmed to exist verbatim in BIP-32.
- The three-tier model, shared/divergent mode diagram, and TLV tag numbers (`0x00`, `0x01`, `0x02`, `0x03`) all accurate against `21-md1-wire-format.md` and the SPEC.
