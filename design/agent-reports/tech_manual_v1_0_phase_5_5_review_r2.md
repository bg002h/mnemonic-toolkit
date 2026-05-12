# tech-manual-v1.0 Phase 5.5 review — r2

Date: 2026-05-12
Reviewer: feature-dev:code-reviewer (r2)

## Summary

1C / 1I / 0L / 0N (new findings; r1 fixes both confirmed).

## r1 fix verification

**C-1 (BIP-86 author):** CONFIRMED. `66-bibliography.md` line 18 now reads `**BIP-86.** Ava Chow. *Key Derivation for Single Key P2TR Outputs.*` — verified against live BIP-86 mediawiki (single author).

**C-2 (rust-codex32 / codex32 dup entry):** CONFIRMED. Single merged entry at line 43; both names referenced; `docs.rs/codex32` URL; cited-in list §II.3, §IV.3, §V.3. No `docs.rs/rust-codex32` URL survives in `66-bibliography.md`.

## New findings

### Critical

**C-3 — Broken URL `docs.rs/rust-codex32` in `12-the-m-format-star.md:45`**

`docs/technical-manual/src/10-foundations/12-the-m-format-star.md:45` reads `` The `ms-codec` crate consumes [`rust-codex32`](https://docs.rs/rust-codex32) verbatim ``. Same broken-URL error as bibliography C-2, but in a different file — r1 caught it in the bibliography but missed this Part-I.2 occurrence. URL returns HTTP 404 on live fetch.

Fix: change link target from `https://docs.rs/rust-codex32` to `https://docs.rs/codex32` (the crates.io package name; `rust-codex32` is the GitHub repo name).

### Important

**I-1 — BIP-85 missing co-author Aneesh Karve**

`66-bibliography.md` line 17 attributes BIP-85 to "Ethan Kosakovsky" alone. Live BIP-85 header lists two authors: Ethan Kosakovsky AND Aneesh Karve. Consistent with multi-author treatment elsewhere in the bibliography (BIP-38, BIP-39, BIP-44, etc.).

Fix: `**BIP-85.** Ethan Kosakovsky, Aneesh Karve. *Deterministic Entropy From BIP-32 Keychains.*`.

## Verdict

- [ ] 0 C / 0 I — Phase 5.5 ready to close
- [x] Findings present — iterate r3
