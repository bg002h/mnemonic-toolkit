# cycle-prep recon — 2026-06-11 — silentpayment-phrase-english-only + bip388-policy-name-lossy-roundtrip

**Origin/master SHA at recon time:** `cdef7cd` (v0.53.6, CI in flight)
**Local branch:** `master` (up-to-date)
Two genuine user-facing correctness `[obs]` bugs (user picked "real bugs"). Independent; likely TWO cycles (or one with R0-split).

---

## Per-slug verification

### `silentpayment-phrase-english-only` — non-English BIP-39 phrases mis-derive
- **WHAT:** `resolve_master_xpriv` parses a raw BIP-39 phrase with `bip39::Language::English` UNCONDITIONALLY, while the ms1 branch resolves per-card wire language.
- **Citations (verified @ cdef7cd):**
  - `silent_payment.rs:162` — **ACCURATE** (drifted from FOLLOWUP's `:155`): `bip39::Mnemonic::parse_in(bip39::Language::English, s)` in the raw-phrase branch (`:160-167`). A non-English phrase (Japanese/Spanish/…) fails checksum/wordlist → error.
  - ms1 branch `:141-158` correctly uses `crate::language::payload_bip39_language(&payload, English)` — **ACCURATE**.
  - entropy-hex branch `:168-174` is English-default — **ACCEPTABLE, NOT the bug** (raw entropy has no wire language; English is the canonical re-encode wordlist; comment says so). DO NOT change.
- **Feasibility:** `bip39 = { version = "2", features = ["all-languages"] }` (Cargo.toml:42) → ALL wordlists compiled in. `bip39::Mnemonic::parse_normalized(s)` (bip39-2 lib.rs:514) AUTO-DETECTS language (NFKD-normalizes; returns `Error::AmbiguousLanguages` (:131,:419) when a phrase is valid in multiple wordlists). 
- **Action for SPEC:** replace `:162` `parse_in(English, s)` with `parse_normalized(s)` (auto-detect), mapping `AmbiguousLanguages` + invalid errors to `ToolkitError::SilentPayment` with a clear message. **Design Qs for R0:** (1) `parse` vs `parse_normalized` (normalization correctness for non-ASCII); (2) how to surface `AmbiguousLanguages` (some short phrases are legitimately ambiguous — refuse with guidance, since silent-payment has NO `--language` flag to disambiguate); (3) confirm the language only affects the phrase→seed PBKDF2 (it does — the words' UTF-8 NFKD is the PBKDF2 input). Tests: a known non-English (e.g. Japanese) phrase derives the SAME address as the canonical reference; an ambiguous phrase refuses cleanly. No `--language` flag added → **no schema_mirror/manual lockstep.** SemVer PATCH (fixes a refusal that should succeed). Cite SHA `cdef7cd`.

### `bip388-policy-name-lossy-roundtrip` — `--format bip388` round-trip drops the policy name
- **WHAT:** emit hardcodes the policy name; expand deserializes-but-drops it.
- **Citations (verified @ cdef7cd):**
  - `wallet_export/pipeline.rs:207` — **ACCURATE**: `"name": "imported-descriptor"` HARDCODED in the emitted policy JSON (`descriptor_to_bip388_wallet_policy`).
  - `wallet_import/pipeline.rs:162` — **ACCURATE** (drifted from FOLLOWUP's `:161-207`): `#[serde(rename = "name")] _name: String` — the real `"name"` is deserialized into `_name` (load-bearing for `deny_unknown_fields`) but NEVER threaded through.
  - `expand_bip388_policy(json: &str) -> Result<String, ToolkitError>` (`wallet_import/pipeline.rs:187`) returns ONLY the expanded descriptor String — no name carrier.
- **Feasibility / surface:** MEDIUM. Threading the name expand→emit requires: (a) `expand_bip388_policy` to ALSO surface `_name` (return a struct `{descriptor, name}` OR a sibling extractor) — touches its ~10 in-file tests + the export_wallet/bundle `--descriptor` consumers; (b) carry the name to the emit step. NOTE: `EmitInputs.wallet_name` ALREADY exists (the `resolved_wallet_name` lift from import-json envelopes flows a name into it) — the bip388 policy name could ride the SAME `wallet_name` channel; (c) emit `:207` uses the carried name (fallback to "imported-descriptor" when absent, preserving current behavior for non-named inputs).
- **Action for SPEC:** thread the policy `name` from expand → `EmitInputs.wallet_name` → emit. **Design Qs for R0:** the exact carrier (extend `expand_bip388_policy` return vs a sibling `extract_bip388_policy_name`); whether the round-trip is single-invocation (`export-wallet --descriptor <policy> --format bip388`) so the name survives in-process; the empty-name fallback. Tests: byte-perfect name round-trip (input name "X" → emitted name "X"); unnamed input → "imported-descriptor" default. SemVer PATCH (metadata fidelity; no wire-schema field add/remove — the `name` field already exists). **Check:** does any GUI/manual reference the hardcoded name? (likely not — internal.) Cite SHA `cdef7cd`.

---

## Cross-cutting observations
1. Both are PATCH; both internal (no clap flag add) → no schema_mirror; verify the manual doesn't describe the hardcoded bip388 name.
2. Citation drift in BOTH FOLLOWUP entries (sp `:155`→`:162`; bip388 expand `:161`→`:162`/`:187`) — use the live lines.
3. silentpayment-phrase is the CLEANER/smaller win (one-call swap + error mapping); bip388-name is MEDIUM (signature + threading + ~10 tests touched).

## Recommended scope
**Two separate PATCH cycles** (or one with R0-split). Do **silentpayment-phrase first** (smaller, self-contained, clean auto-detect swap). Then **bip388-name** (threading via the existing `wallet_name` channel). Each: cycle-prep done → SPEC → R0 to 0C/0I → TDD (RED-proven) → impl review → ship. No lockstep for either.
