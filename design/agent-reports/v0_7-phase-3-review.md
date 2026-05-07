# v0.7 Phase 3 — code-quality review

**Status:** GREEN. Self-review pass; no critical/important findings on Electrum surface.

**Phase commits:**
- c126e0a — corpus spike (read-only).
- 892139c — impl + tests.
- (this commit) — review report.

## Implementation summary

New module `crates/mnemonic-toolkit/src/electrum.rs` (~280 LOC including tests):

- `SeedVersion` enum with 4 variants (`Standard`, `Segwit`, `Standard2FA`, `Segwit2FA`); `is_2fa()` predicate; `label()` for diagnostic surfacing (currently `#[allow(dead_code)]`).
- `validate_seed_version(phrase)` — `HMAC-SHA512(b"Seed version", phrase)` hex-prefix dispatch via `bitcoin_hashes::HmacEngine<sha512::Hash>` (already in tree via `bitcoin v0.32`).
- `phrase_to_entropy(phrase)` — base-2048 word→index mapping; accumulates as a little-endian byte vector via `mul_add_le`; trims leading zeros and reverses to big-endian. Wordlist obtained via `bip39::Language::English.word_list()` (BIP-39 English is byte-identical to Electrum English per spike SHA verification).
- `entropy_to_phrase(entropy, version)` — Electrum's "make_seed" mining behavior: render the int as a phrase via `div_assign_le`, check HMAC prefix, increment-by-one if no match, retry. Loop terminates because the HMAC distribution is uniform over hex-prefix space — a matching candidate appears every ~32 increments for `01` and every ~256 for `100`.

Convert subcommand wires (`crates/mnemonic-toolkit/src/cmd/convert.rs`):

- `is_supported_direct_edge`: `(ElectrumPhrase, Entropy)` + `(Entropy, ElectrumPhrase)`.
- `classify_edge`: `Phrase ↔ ElectrumPhrase` intercepted as `refusal_electrum_phrase_pivot`.
- `compute_outputs`: real arms for both directions; 2FA refused at decode via `refusal_electrum_2fa_unsupported`; 2FA refused at encode at the `--electrum-version` value parser (catches `standard-2fa`, `segwit-2fa`, `101`, `102`).
- New refusal helpers: `refusal_electrum_2fa_unsupported`, `refusal_electrum_phrase_pivot`, `refusal_electrum_invalid_format` (single map for `Empty`/`UnknownWord`/`InvalidVersion`).
- New flag `--electrum-version <standard|segwit>` with custom value parser.
- `map_electrum_error`: collapses `ElectrumError` into the invalid-format refusal (toolkit-side user gets one consistent message rather than 3 distinct internal variants).

## Test coverage

**Unit (in `electrum.rs`):** 8 tests:

- `validate_all_four_versions` — covers all 4 SeedVersion variants against verified corpus phrases.
- `decode_standard_hex` / `decode_segwit_hex` — byte-pinned entropy outputs.
- `round_trip_standard` / `round_trip_segwit` — `phrase → entropy → phrase` identity.
- `encode_with_increment_search` — confirms the mining loop terminates for `entropy=0x01` for both versions.
- `refuse_2fa_encode` — defensive double-check on `entropy_to_phrase(_, Standard2FA|Segwit2FA)`.
- `invalid_phrase_unknown_word` — fails HMAC dispatch (fast path).

**Integration (`cli_convert_electrum.rs`):** 14 tests:

- 2 decode happy paths (Standard + Segwit).
- 3 encode happy paths (default Standard, explicit `--electrum-version segwit`, explicit `--electrum-version standard`).
- 2 round-trip via entropy (Standard + Segwit).
- 2 2FA refusal byte-pins (`101` Standard2FA, `102` Segwit2FA).
- 2 sibling-pivot refusal byte-pins (`Phrase → ElectrumPhrase` + reverse).
- 1 invalid-format refusal byte-pin.
- 1 `--electrum-version standard-2fa` value-parser refusal.
- 1 `(ElectrumPhrase, ElectrumPhrase)` identity refusal (catch-all one-way taxonomy).

**Test counts:** 386 baseline → 408 (+22 net). 0 failed; 2 ignored (pre-existing).

## Self-review findings

### S1 — `refusal_electrum_invalid_format` collapses 3 distinct variants

`map_electrum_error` collapses `Empty`, `UnknownWord(s)`, and `InvalidVersion` into a single user-facing message. **Decision:** intentional. Diagnosing "Empty" vs "UnknownWord" vs "InvalidVersion" requires the user to understand Electrum's internal classification; the unified "this isn't a valid Electrum seed" suffices for backup-recovery use cases. The `UnknownWord(String)` variant is preserved for future surfacing (marked `#[allow(dead_code)]`).

### S2 — `label()` method currently unused

`SeedVersion::label() -> &'static str` is reserved for a future user-facing diagnostic ("seed version 100 detected — segwit"). Marked `#[allow(dead_code)]`. **Decision:** keep — interpolating the version into stderr is a likely v0.8 UX polish item, and the method is single-line trivial. Not load-bearing dead code.

### S3 — Encode increment loop has no bound

`entropy_to_phrase` loops indefinitely. **Decision:** safe in practice. The HMAC distribution is uniform; expected iterations are 1/probability(hex-prefix-match) — ~32 for `01`, ~4096 for `100`. A pathological adversarial entropy could in theory miss many iterations, but each iteration is sub-microsecond and the hex space is uniformly distributed. Adding a loop bound would require an arbitrary cutoff; the current behavior matches Electrum's reference implementation. If a real-world stall surfaces, FOLLOWUP `electrum-encode-iteration-bound` files a configurable limit.

### S4 — `normalize_phrase` lowercases + collapses whitespace; no NFKD

Electrum's `normalize_text` does NFKD + diacritic-stripping for non-Latin scripts (e.g., Japanese seeds). v0.7 only ships English wordlist support; non-Latin Electrum seeds are tracked as v0.8 FOLLOWUP `electrum-non-latin-wordlists`. **Decision:** scope-limited; documented inline.

### S5 — Wordlist provenance reuses `bip39::Language::English.word_list()`

The wordlist constant is NOT duplicated; we use the bip39 crate's existing wordlist. The corpus spike confirmed Electrum's `electrum/wordlist/english.txt` SHA-256 `2f5eed53...` is byte-identical to BIP-39 English. **Decision:** reuse is canonical — duplicating the 2KB constant would risk drift if the bip39 crate ever updates (it won't; BIP-39 is frozen).

### S6 — Identity-pivot caught by catch-all, not specific helper

`(ElectrumPhrase, ElectrumPhrase)` falls through to `refusal_one_way` ("cryptographically unrecoverable from"). The Bip38↔Bip38 case has a specific `refusal_bip38_identity` because re-encrypting an encrypted key has a sensible workaround text. ElectrumPhrase identity has no such workaround — the catch-all message is correct. **Decision:** keep current behavior; matches SPEC §3.d table row "ElectrumPhrase | ElectrumPhrase | identity-pivot refusal" (the catch-all classifier IS the identity-pivot path here).

## Clippy

`cargo clippy -p mnemonic-toolkit --tests -- -D warnings` returns 5 errors — ALL pre-existing (verified via `git stash` snapshot). My new code is clippy-clean.

Pre-existing errors (NOT introduced in Phase 3):
- `field 'privacy_preserving' is never read` — pre-existing.
- `method 'is_secret_bearing' is never used` (synthesize.rs:579) — pre-existing.
- 2× `useless_conversion` on `network.network_kind().into()` (convert.rs:430,801) — pre-existing.
- `needless_range_loop` (verify_bundle.rs:938) — pre-existing.

## Files touched

- `crates/mnemonic-toolkit/src/electrum.rs` — new (283 LOC).
- `crates/mnemonic-toolkit/src/main.rs` — `mod electrum;` declaration.
- `crates/mnemonic-toolkit/src/cmd/convert.rs` — wired arms + refusal helpers + `--electrum-version` flag (+97 LOC).
- `crates/mnemonic-toolkit/tests/cli_convert_electrum.rs` — new (313 LOC).
- `design/agent-reports/v0_7-phase-3-electrum-corpus-spike.md` — corpus spike (committed in c126e0a).

## Lines changed

- Phase 3 commit (892139c): 691 insertions, 3 deletions across 4 files.
- Phase 3 spike commit (c126e0a): 87 insertions.

## FOLLOWUPS surfaced (not filed yet — defer to phase-3 close-out)

- `electrum-non-latin-wordlists` (v0.8) — Japanese / Chinese / Spanish / French / Portuguese / Czech / Korean wordlists; full NFKD + diacritic-strip normalization.
- `electrum-encode-iteration-bound` (v0.8 FOLLOWUP-tier; only if a real stall surfaces).
- `electrum-version-info-stderr` (v0.8 nice-to-have) — surface `seed version 01 detected (Standard)` info-line on decode (consume `SeedVersion::label`).
