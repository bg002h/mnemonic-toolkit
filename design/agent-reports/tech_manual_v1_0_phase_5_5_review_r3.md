# tech-manual-v1.0 Phase 5.5 review — r3

Date: 2026-05-12
Reviewer: feature-dev:code-reviewer (r3)

## Summary

0C / 0I / 0L / 0N. Both r2 fixes confirmed; multi-author BIP sweep clean.

## r2 fix verification

**C-3:** CONFIRMED. `12-the-m-format-star.md:45` now uses `https://docs.rs/codex32` as the URL target; `rust-codex32` retained as the GitHub repo name in prose only.

**I-1:** CONFIRMED. `66-bibliography.md:17` now reads `BIP-85. Ethan Kosakovsky, Aneesh Karve.` Matches live BIP-85 header.

## Final sweep

### `docs.rs/rust-codex32` occurrences

`grep -rn 'docs\.rs/rust-codex32' docs/technical-manual/src/` — **0 occurrences**.

### `.cspell.json` additions

`Aneesh` + `Karve` confirmed at lines 34-35.

### Multi-author BIP sweep (all 20 BIP entries)

Every entry verified against live `bitcoin/bips` mediawiki headers. No omitted co-authors.

| BIP | Manual / Live | Match |
|---|---|---|
| BIP-32 | Pieter Wuille | pass |
| BIP-38 | Mike Caldwell, Aaron Voisine | pass |
| BIP-39 | Marek Palatinus, Pavol Rusnak, Aaron Voisine, Sean Bowe | pass |
| BIP-44 | Marek Palatinus, Pavol Rusnak | pass |
| BIP-45 | Manuel Araoz, Ryan X. Charles, Matias Alejo Garcia | pass |
| BIP-48 | Fontaine | pass |
| BIP-49 | Daniel Weigl | pass |
| BIP-84 | Pavol Rusnak | pass |
| BIP-85 | Ethan Kosakovsky, Aneesh Karve | pass (post-I-1) |
| BIP-86 | Ava Chow | pass (post-C-1) |
| BIP-87 | Robert Spigler | pass |
| BIP-93 | Leon Olsson Curr / Pearlwort Sneed (pseudonyms), Andrew Poelstra | pass |
| BIP-173 | Pieter Wuille, Greg Maxwell | pass |
| BIP-340 | Pieter Wuille, Jonas Nick, Tim Ruffing | pass |
| BIP-341 | Pieter Wuille, Jonas Nick, Anthony Towns | pass |
| BIP-342 | Pieter Wuille, Jonas Nick, Anthony Towns | pass |
| BIP-379 | Pieter Wuille, Andrew Poelstra, Sanket Kanjalkar, Antoine Poinsot, Ava Chow | pass |
| BIP-380 | Pieter Wuille, Ava Chow | pass |
| BIP-388 | Salvatore Ingala | pass |
| BIP-389 | Ava Chow | pass |

All 20 pass.

## Verdict

- [x] 0 C / 0 I — Phase 5.5 ready to close
- [ ] Findings present — iterate r4
