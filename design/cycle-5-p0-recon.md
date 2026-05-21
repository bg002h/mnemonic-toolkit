# Cycle 5 — P0 STRICT-GATE recon dossier

**Date:** 2026-05-21
**Cycle target:** `mnemonic-toolkit-v0.30.0` (was v0.29.1; reclassified MINOR per R0-C2 fold) + paired `mnemonic-gui-v0.15.0`.
**Source SHA:** `eebf798` (master HEAD).
**Brainstorm under review:** `design/BRAINSTORM_v0_30_0_seedqr.md`.

## A1 — Line-citation refresh

| Cited | Actual | Match? | Note |
|---|---|---|---|
| `cmd/seed_xor.rs:46-54` | 46-52 | partial | `#[arg(` block spans 46-51; `pub from: FromInput,` at 52. Update span. |
| `cmd/seed_xor.rs:68-69` | 68 (attr) + 69 (field) | yes | `#[arg(long = "json-out")]` at 68; field at 69. |
| `cmd/slip39.rs:86-100` | 86-91 | partial | `#[arg(` block spans 86-90; `pub from:` at 92. Cited span was too wide. |
| `cmd/slip39.rs:137-139` | 137-138 | yes | `#[arg(long = "json-out")]` at 137; field at 138. |
| `41-mnemonic.md:1144` | 1144 | yes | `## \`mnemonic slip39\`` exact match. |
| `41-mnemonic.md:1587` | 1587 | yes | `## \`mnemonic gui-schema\`` exact match. |
| `45-foreign-formats.md:608` (cited as "Deferral subsection") | 608 is `jade_specific_fields` prose; **L620** is the actual `### Deferral — SeedQR` header | NO | R0 I1: rewrite citation to L620-626. |
| `45-foreign-formats.md:786` ("Jade SeedQR variant") | 786 | yes | "Jade SeedQR variant" bullet item exact match. |
| `FOLLOWUPS.md:2559` (`wallet-import-jade-seedqr` slug start) | 2559 | yes | Slug header exact match. |
| `FOLLOWUPS.md:2360` (inline-mode hedge) | 2360 | yes | "inline mode rather than wallet-import format" phrase match. |

**Conclusion:** all markdown section-header citations match. Code-block ranges off by 2-3 lines (line-number drift since cycle brainstorm-write). I1 chapter-45 citation is the only load-bearing fix.

## A2 — `mnemonic-gui` pre-Cycle-5 state

- **Pin in `mnemonic-gui/pinned-upstream.toml` L22 (`[mnemonic]` table):** `mnemonic-toolkit-v0.29.0`.
- **Pin in `mnemonic-gui/Cargo.toml` L42:** git dep with tag `mnemonic-toolkit-v0.29.0`.
- **Latest GUI tag:** `mnemonic-gui-v0.14.0` (per Cycle 4 close).
- **GUI master HEAD:** `8f9e83bb864c474e8d20f1cd44ad1e9517776922`.
- **Branch status:** clean (no uncommitted changes).
- **Schema mirror file** `mnemonic-gui/src/schema/mnemonic.rs`:
  - 2,484 lines.
  - 18 entries in `SUBCOMMANDS` const (L2310-2478): `bundle`, `verify-bundle`, `convert`, `export-wallet`, `derive-child`, `final-word`, `seed-xor-split`, `seed-xor-combine`, `slip39-split`, `slip39-combine`, `repair`, `inspect`, `import-wallet`, `xpub-search-path-of-xpub`, `xpub-search-account-of-descriptor`, `xpub-search-address-of-xpub`, `xpub-search-passphrase-of-xpub`, `compare-cost`.
  - **No `seedqr-*` entry** — confirmed (Cycle 5 must add `seedqr-decode` + `seedqr-encode`).
- **Schema mirror integration test** `mnemonic-gui/tests/schema_mirror.rs`:
  - 726 lines.
  - `MNEMONIC_BIN` resolution at L47-50: env var `<CLI_uppercase>_BIN` preferred (`MNEMONIC_BIN`); falls back to literal binary name via `$PATH` lookup. Pattern: `std::env::var(&env_var).unwrap_or_else(|_| cli_name.to_string())`. **Confirms Cycle 4 feedback:** local runs MUST set explicit `MNEMONIC_BIN=...` to avoid stale `$PATH` binary pickup.

## A3 — BIP-39 machinery availability for `seedqr.rs`

**Cargo.toml dep:** `bip39 = { version = "2", features = ["all-languages"] }` at L31.

**Existing toolkit BIP-39 API surface in use:**
- Parse + checksum-validate: `bip39::Mnemonic::parse_in(Language::English, phrase) -> Result<Mnemonic, bip39::Error>` (used at `cmd/seed_xor.rs:181`).
- Entropy → phrase (checksum recomputation): `bip39::Mnemonic::from_entropy_in(lang, entropy) -> Result<Mnemonic, bip39::Error>` (used at `cmd/seed_xor.rs:208` + `derive_slot.rs:55`).
- Phrase → entropy: `Mnemonic::to_entropy(&self) -> Vec<u8>` (used at `cmd/seed_xor.rs:192`).
- Wordlist access: `bip39::Language::word_list() -> &'static [&'static str; 2048]` (used at `final_word.rs:129,135`).

**Word↔index mapping:** **NOT exposed publicly by `bip39` crate**. Two options:
1. **Inline linear search** for `word_to_index`: `lang.word_list().iter().position(|w| *w == word)`. O(2048) per call; acceptable for max-24-word phrases (24 lookups = 49k comparisons; <1ms).
2. **`index_to_word`** is `O(1)`: `lang.word_list()[idx as usize]`.

**SLIP-39 precedent:** `src/slip39/wordlist.rs:57,64` ships private `word_to_index`/`index_to_word` helpers. NOT reusable directly (SLIP-39 wordlist ≠ BIP-39 wordlist), but the pattern can be mirrored for BIP-39 inside `seedqr.rs`.

**Checksum validation:** delegated entirely to `bip39::Mnemonic::parse_in()` / `from_entropy_in()`. No hand-rolled checksum API exists or needs to.

**Single-sentence answer:** `seedqr.rs` imports `use bip39::{Mnemonic, Language};` — uses `Mnemonic::parse_in(Language::English, phrase)` for decode-side checksum-validating parse, `Mnemonic::from_entropy_in(Language::English, entropy_bytes)` for synthesized round-trip, and `Language::English.word_list()` for raw wordlist access (linear-search for word→index in encode; array-indexing for index→word in decode). **Zero new Cargo.toml deps required.**

## A4 — SeedSigner Python reference symbol path

**Deferred:** Phase 0 A4 (added per R0 I7) will WebFetch the SeedSigner repo + locate the exact Python symbol path before Phase 5 manual writing. Brainstorm cites the placeholder `seedsigner.helpers.qr.SeedSigner.encode_standard_seedqr` with caveat "verify exact symbol at write time"; A4's job is to lock the symbol or change it.

**Expected scope:** under 1h. Output added to this dossier as `## A4` section after the WebFetch completes.

## Recon verdict

**GREEN** with one citation fix (R0 I1 — L608 → L620 for the chapter-45 Deferral subsection citation). All other surface checks pass:
- bip39 crate is available; zero new Cargo.toml deps needed.
- GUI repo is at the expected pin + clean state.
- Schema mirror has no pre-existing `seedqr-*` entries (cycle's job to add them).
- Markdown section headers verified at cited line numbers.

Brainstorm folds proceed against this dossier.
