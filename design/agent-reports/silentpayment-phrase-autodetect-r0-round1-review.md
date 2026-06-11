# R0 Review — silent-payment phrase language auto-detect — ROUND 1

**Source SHA:** `cdef7cd`. **Verdict: 🟡 YELLOW — 0 Critical / 2 Important / 3 Minor.** Design (English-first `or_else(|_| parse(s))`) is sound; API verified against bip39-2.2.2; citations accurate; PATCH + no-lockstep confirmed. All findings are SPEC fixes.

## Critical
None.

## Important

**I1 — English-first fallback silently changes the error MESSAGE for malformed English phrases with cross-wordlist words.** A bad-checksum English phrase whose words also exist in another wordlist: today → "invalid checksum"; after → `parse_in(English)` Errs (InvalidChecksum) → `.or_else` → `parse(s)` → `language_of` returns `AmbiguousLanguages` → message becomes "ambiguous word list: English, French". The phrase was ALREADY rejected (no funds issue), but the message flip is a testable behavior change T3 doesn't cover. **Fix (option a):** document this known edge in the Design section + forbid a regression test that pins the OLD message for it.

**I2 — T3 oracle must be a FROZEN LITERAL, not a runtime `parse_in(English,…)` compute (circular).** "or a captured golden" is an escape hatch; a golden computed via the same code path is tautological. Mandate a frozen literal xpriv string constant (same lesson as mnemonic-key v0.8.0 I1/I2: "de-tautologize w/ frozen literal; forbid runtime compute call"). Remove "or a captured golden".

## Minor
- **M1** — SPEC prose reverses causality: `#[cfg(feature="unicode-normalization")]` GATES `parse_in`/`parse`; the feature is active via bip39's default `std → alloc → unicode-normalization` chain (not "implied by parse_in use").
- **M2** — T4: name the FOLLOWUP to file if deferred — `silentpayment-japanese-bip39-seed-vector-cross-check`.
- **M3** — pin the resolved crate version in citations: `bip39-2.2.2 lib.rs:532` (parse), `:131` (AmbiguousLanguages), `:432` (language_of). (Confirmed `bip39 = 2.2.2` in Cargo.lock.)

## Confirmations
- API verified: `parse` (bip39-2.2.2 lib.rs:532, `#[cfg(unicode-normalization)]`) NFKD-normalizes + auto-detects via `language_of`; `parse_in` (:520) normalizes too → no normalization regression. Entropy-hex English default correctly left (raw entropy has no wire language).
- Seed-from-words semantics correct (Japanese phrase ≠ same-entropy English phrase). Funds-relevant.
- English-first is the right call (avoids the AmbiguousLanguages regression for valid English phrases that bare `parse(s)` would introduce).
- Citations accurate @ cdef7cd; 3 self-pins correct; PATCH; no schema_mirror/manual/GUI/sibling lockstep.
