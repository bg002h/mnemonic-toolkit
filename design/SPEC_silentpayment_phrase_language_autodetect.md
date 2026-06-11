# SPEC — silent-payment: auto-detect the BIP-39 phrase language (not English-only)

**Cycle:** toolkit PATCH · **Source SHA:** `cdef7cd` (v0.53.6) · **Recon:** `cycle-prep-recon-silentpayment-phrase-lang+bip388-name-roundtrip.md`.
**Resolves:** `silentpayment-phrase-english-only`.

## Problem (verified @ `cdef7cd`)
`resolve_master_xpriv` (`cmd/silent_payment.rs:109`) parses a raw BIP-39 phrase with `bip39::Mnemonic::parse_in(bip39::Language::English, s)` (`:162`) — UNCONDITIONALLY English. A non-English phrase (Japanese, Spanish, French, …) fails the English wordlist/checksum → the command refuses a perfectly valid seed. The ms1 branch (`:141-158`) already resolves per-card wire language via `crate::language::payload_bip39_language`; only the RAW-phrase branch is English-locked. (The entropy-hex branch `:168-174` English default is CORRECT — raw entropy carries no wire language; the canonical re-encode wordlist is English. Leave it.)

The seed is derived from the phrase WORDS (`derive_master_seed` → PBKDF2 over the NFKD of the words), so a Japanese phrase's seed ≠ the same-entropy English phrase's seed — the language is funds-relevant, not cosmetic.

## Design — English-first, then auto-detect (regression-safe)
`bip39` is `features = ["all-languages"]` (Cargo.toml:42), resolved `bip39 = 2.2.2` (Cargo.lock). Both `parse` and `parse_in` REQUIRE `feature = "unicode-normalization"`, active via bip39's default `std → alloc → unicode-normalization` feature chain (the toolkit does not set `default-features = false`). `bip39::Mnemonic::parse(s)` (bip39-2.2.2 lib.rs:532, `#[cfg(feature = "unicode-normalization")]`) NFKD-normalizes then auto-detects via `language_of` (bip39-2.2.2 lib.rs:432), returning `Error::AmbiguousLanguages` (bip39-2.2.2 lib.rs:131) when a phrase is valid in multiple enabled wordlists.

Replace `:162`:
```rust
// English-first preserves the exact current behavior for English phrases
// (incl. any that are word-ambiguous across wordlists); auto-detect is the
// fallback for non-English. parse()/parse_in() both NFKD-normalize.
let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, s)
    .or_else(|_| bip39::Mnemonic::parse(s))
    .map_err(|e| ToolkitError::SilentPayment(format!("BIP-39 phrase: {e}")))?;
```
**Why English-first, not bare `parse(s)` (R0-r1 confirmed):** a bare `parse(s)` would auto-detect for ALL phrases, so a previously-working English phrase whose words happen to also be valid in another enabled wordlist would newly error with `AmbiguousLanguages` — a regression. English-first parses English exactly as today; only English-FAILURES fall through to auto-detect. The genuinely-ambiguous non-English case still surfaces `AmbiguousLanguages` (mapped to a `SilentPayment` error — silent-payment has no `--language` flag to disambiguate, so a clear refusal is correct).

**Known + accepted error-MESSAGE change (R0-r1 I1) — NOT a regression:** for an ALREADY-INVALID English phrase (e.g. bad checksum) whose words ALSO appear in another wordlist, the message changes from "invalid checksum" to `AmbiguousLanguages` ("ambiguous word list: English, …") — because `parse_in(English)` Errs, `.or_else` fires, and `parse(s)`'s `language_of` is ambiguous. The phrase was already rejected (no funds impact); only the diagnostic text differs. This is acceptable. **Do NOT add a test that pins the OLD message for this edge** (it would falsely RED the fix).

No new flag → **no `schema_mirror` / manual / GUI / sibling lockstep.** SemVer **PATCH** (turns a wrongful refusal into a correct derivation; English unchanged).

## Tests (TDD)
`resolve_master_xpriv` is a private fn in `cmd/silent_payment.rs` — add an in-file `#[cfg(test)] mod` (or extend one) so the tests hit it directly without the CLI harness. Construct phrases via the crate (no hardcoded wordlists).
- **T1 (non-English now resolves — the fix, RED-proven):** `let jp = bip39::Mnemonic::from_entropy_in(bip39::Language::Japanese, &[0u8;16]).unwrap().to_string();` → `resolve_master_xpriv(&jp, "", CliNetwork::Mainnet, &mut sink)` returns `Ok`. **RED:** with the bare English `parse_in` it returns `Err` (the Japanese words fail the English wordlist).
- **T2 (words-based, not entropy-reencode — correctness):** the Japanese-phrase xpriv ≠ the English-phrase-for-the-SAME-entropy xpriv: `from_entropy_in(English, &[0u8;16])` vs `from_entropy_in(Japanese, &[0u8;16])` → distinct master xprivs (different NFKD word strings → different PBKDF2 seed). Pins that the fix derives from the actual phrase words, not from re-encoding entropy to English.
- **T3 (English no-regression, FROZEN LITERAL — R0-r1 I2):** a known English phrase resolves to a specific xpriv pinned as a **frozen string-literal constant** (captured once, out-of-band / from a reference). Assert `resolve_master_xpriv(en_phrase, "", Mainnet, &mut sink).unwrap().to_string() == "<frozen xprv…>"`. Do NOT compute the oracle at runtime via `parse_in(English,…)`/`derive_master_seed` — that is circular (both sides go through the code under test). The literal is the non-tautological pin.
- **T4 (cross-check against a BIP-39 seed vector — strong):** for the Japanese all-zeros-entropy phrase, the derived master xpriv equals `Xpriv::new_master(Mainnet, <known BIP-39 Japanese seed for 0x00…00 + empty passphrase>)` (the seed pinned as a frozen literal). If the canonical seed cannot be sourced within the implementation window, rely on T1+T2 and **file FOLLOWUP `silentpayment-japanese-bip39-seed-vector-cross-check`**.
- Existing `tests/cli_silent_payment.rs` + full suite stay green.

## Ritual
CHANGELOG `[<next patch>]`; version bump (Cargo.toml + Cargo.lock); **self-pins: `README.md:13` + `crates/mnemonic-toolkit/README.md:9` (`<!-- toolkit-version: -->`) + `scripts/install.sh:32` tag.** FOLLOWUPS resolve `silentpayment-phrase-english-only`. No manual/schema_mirror/GUI/sibling lockstep (no CLI surface change). Mandatory R0 gate to 0C/0I before code; persist reviews to `design/agent-reports/`.

## Non-goals
Adding a `--language` flag (auto-detect suffices; a flag = schema_mirror+manual lockstep); the entropy-hex English default (correct); the bip388-name round-trip (the next cycle).
