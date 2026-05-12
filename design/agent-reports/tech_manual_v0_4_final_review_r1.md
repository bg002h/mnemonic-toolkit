# tech-manual-v0.4 final review — r1

Date: 2026-05-11
Reviewer: feature-dev:code-reviewer (final r1)

## Summary

- Cross-chapter consistency: 1C / 0I / 0L / 0N
- Back-matter completeness: 0C / 0I / 0L / 0N
- SPEC §7 acceptance criteria: 0C / 0I / 0L / 0N
- Mirror invariants: 0C / 0I / 0L / 0N
- Other: 0C / 0I / 0L / 0N

Total: 1C / 0I / 0L / 0N

## Findings

### Critical

#### C-1 — §V.4.8 + glossary `non_exhaustive` entry falsely claim `md_codec::Error` is `#[non_exhaustive]`

**Locations:**
- `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md` (the §V.4.8 bullet at line 431).
- `docs/technical-manual/src/60-back-matter/61-glossary.md` (the `non_exhaustive` entry, lines 219-221).

**Evidence (three independent sources):**

1. `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/error.rs:19-20` — `#[derive(Debug, Error, PartialEq, Eq)] pub enum Error { ... }`. No `#[non_exhaustive]` attribute.
2. §V.1.3.9 of the manual (line 206) correctly documents the derives as `Debug, Error, PartialEq, Eq` only — no `#[non_exhaustive]`.
3. `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/error.rs:172` carries the comment "md_codec::Error is NOT `#[non_exhaustive]`; match is exhaustive" — directly contradicting the §V.4.8 + glossary claims.

**Impact:** High. The claim propagates into readers' match arms — they will add unnecessary `_ => …` catch-all arms when writing exhaustive matches over `md_codec::Error`, obscuring unhandled variants. The glossary entry's authoritative framing makes the claim load-bearing for any reader trusting the manual.

**Fix:**
- §V.4.8 line 431: replace "each likewise `#[non_exhaustive]`" — note `ms_codec::Error` and `mk_codec::Error` are `#[non_exhaustive]`, but `md_codec::Error` is the exception.
- Glossary `non_exhaustive` body: remove `md_codec::Error` from the "uniformly applied" list and add a parenthetical noting it as the exception.

## SPEC §7 acceptance criteria status

- **A2** (every public function referenced by name in Part V chapters): 92/92 coverage gate passed per `tests/api-surface-coverage.sh`. Independently spot-checked 10+ items across the four crates; all present. PASS.
- **A3** (every error variant in each crate's `Error` enum has a row): md-codec 43 + mk-codec 22 + ms-codec 10 + toolkit 26 = 101 variants, all in tables. PASS.
- **A8** (worked examples verified): all 4 Part V chapters cite their `cargo run` invocation; `make verify-examples` 15/15. PASS.
- **A11** (build reproducible): 242pp, 841,528 bytes, SHA256 `e1b6dd5e1b75810a6a955943546deb6ad9703eeaf5a7e30517941c114eabc6e9`, byte-identical across two clean `SOURCE_DATE_EPOCH=1746921600` builds. PASS.

## Mirror invariants status

- **`tech-manual-api-surface-mirror`**: 5 `pub` items per crate spot-checked at HEAD against chapter content. All present (incl. `BitWriter`, `decode_md1_string`, `encode_md1_string`, `KeyCard::new`, `decode_bytecode`, `decode`, `Tag::ENTR`, `synthesize_unified`, `BundleJson`, `VerifyCheck`). PASS.
- **`tech-manual-wire-format-mirror`**: no chapter dependency this cycle; skipped.

## FOLLOWUPS status

- `cross-repo md1-wsh-multi-unsorted-integration-test`: still open. CONFIRMED.
- `cross-repo md1-bip49-integration-test`: still open. CONFIRMED.
- No new FOLLOWUPS added during the cycle. CONFIRMED.

## CHANGELOG status

v0.4.0 entry absent (expected — tag commit adds it). Most recent entry `tech-manual [0.3.0]` properly formatted.

## Release-history row

`tech-manual-v0.4.0` row at line 10: "glossary 96 entries, index 530 rows, BIP cross-reference 20 BIPs, 242pp PDF". All four counts verified correct.

## Cycle-exit verification confirmed

- `make lint` 6/6 green.
- `make verify-examples` 15/15 transcripts.
- `cargo test --workspace --all-features` all green.
- PDF reproducible (242pp, byte-identical across two clean builds).

## Verdict

- [ ] 0 C / 0 I / 0 L / 0 N — tag-ready
- [x] Findings present — fold C-1 inline and re-run final
