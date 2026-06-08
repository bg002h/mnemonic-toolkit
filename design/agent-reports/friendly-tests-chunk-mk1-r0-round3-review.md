# R0 Architect Gate — Round 3 — SPEC_friendly_tests_and_chunk_mk1.md

> Round 2 = 0C/1I (I-new-1); folded. Reviewer had Read/Grep; parent persists.

**Verdict: GREEN (0C / 0I) — 1 non-blocking Minor (folded post-review).**

## Critical / Important (new)
None.

## Minor (new)
- **M-new-1 (`:38`) — residual enum-vs-mapper phrasing.** The Approach assertion #2 rationale read "the trap that the `#[non_exhaustive]` mappers risk" — enum-level framing I-new-1 killed elsewhere. Non-blocking (the M5 note at `:50` authoritatively resolves the 2/3 split; lines 45-46 name exact arms). **Folded after this review** → now reads "the trap that the 2 wildcard mappers `friendly_ms_codec`/`friendly_mk_codec` risk — vacuous for the 3 closed mappers". Does not require a Round 4 gate.

## Fold confirmation — I-new-1 (CONFIRMED)
Taxonomy now mapper-level + correct: TWO wildcard mappers with bare `_ => "unhandled…"` (`friendly_ms_codec` `_`@`:129`, `friendly_mk_codec` `_`@`:181`); THREE closed/no-`_` (`friendly_md_codec`, `friendly_bip39`, `friendly_bitcoin`). Source-verified: `friendly_bitcoin` (`:34-40`) exhaustively matches toolkit-local closed `BitcoinErrorKind`, Display-forwarding `#[non_exhaustive]` `bitcoin::bip32::Error` only inside the `Bip32(b)` arm — no wildcard. Taxonomy paragraph (`:30-34`), out-of-scope (`:48`), M5 note (`:50`) all agree 2/3 + drop bitcoin from wildcard set. Module-doc fix (`:34`) correctly drops BOTH bip39 + bitcoin (source `friendly.rs:4-6` lists both stale). No "three non_exhaustive mappers" / bitcoin-in-wildcard phrase survives.

## Rest intact
- Item 1 chunk sites: `bundle.rs:951` ms1 (stays), `:962`/`:974` mk1 (swap), import `:7`; `format.rs:28-31` stale doc, `:32` `#[allow(dead_code)]`, `:33` `chunk_5char(s)` body. All source-confirmed at `8665d91`. Byte-identical holds.
- R1 folds intact: I1 (bin-target `:57`), M1 (drop AmbiguousLanguages `:45`), M2 (emit_unified `:14`/`:66`), M3/M4 (allow-removal + doc reword `:20-22`).
- Disposition: no-bump/no-tag, test-only + byte-identical, no lockstep.

**GREEN — cleared to implement.**
