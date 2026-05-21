# mnemonic-toolkit-v0.32.0 Implementation Plan (Cycle 14 — seedqr-compact-variant)

> **For agentic workers:** REQUIRED SUB-SKILL: `superpowers:subagent-driven-development` or direct execution.

**Goal:** Ship `mnemonic-toolkit-v0.32.0` (SemVer-MINOR; new `--variant` flag). Closes `seedqr-compact-variant` FOLLOWUP. Adds CompactSeedQR (binary-mode QR payload = raw BIP-39 entropy bytes) encode + decode to `mnemonic seedqr`, represented as lowercase hex on the CLI. SeedSigner-faithful: 12 + 24 words only.

**Architecture:** New `--variant <standard|compact>` flag (default `standard`) on both `seedqr encode` and `seedqr decode`. Standard variant = the existing decimal-digit path (unchanged). Compact variant: `encode` emits the raw BIP-39 entropy bytes as lowercase hex (16 bytes = 32 hex chars for 12-word; 32 bytes = 64 hex chars for 24-word); `decode` reads hex → entropy bytes → recompute BIP-39 checksum → phrase. Per SeedSigner's `CompactSeedQrEncoder` (primary source verified), the compact payload is exactly the BIP-39 entropy (checksum bits stripped) — so it equals `Mnemonic::to_entropy()`. SeedSigner's reference only handles 12/24 words; the toolkit matches that restriction (refuses 15/18/21 for compact with a clear message).

**Tech Stack:** Rust; reuses `bip39` + `hex` (existing deps); zero new deps; 3 new library-local `SeedqrError` variants (NOT `ToolkitError` — no alphabetical-ordering concern); zero lib.rs changes.

**P0 STRICT-GATE recon (verified at master HEAD `a7576d0`):**
- `crates/mnemonic-toolkit/src/seedqr.rs:24-30` — `SeedqrError` enum (add 3 variants).
- `crates/mnemonic-toolkit/src/seedqr.rs:62-138` — `decode` / `encode` (standard; unchanged) [R0 M3 citation fix].
- `crates/mnemonic-toolkit/src/cmd/seedqr.rs:30-63` — `SeedqrDecodeArgs` + `SeedqrEncodeArgs` (add `--variant`).
- `crates/mnemonic-toolkit/src/cmd/seedqr.rs:77-84` — `SeedqrEnvelope` has a `variant: &'a str` field; both `emit_decode_output` + `emit_encode_output` currently hardcode `variant: "standard"` (flip to dynamic).
- SeedSigner primary source `src/seedsigner/models/encode_qr.py::CompactSeedQrEncoder` (verified 2026-05-21): packs 11-bit indices, strips 4 (12-word) / 8 (24-word) checksum bits, byte-packs → raw entropy bytes. Only 12/24 handled.

**Design locks (user-confirmed 2026-05-21):**
- **Payload form:** hex text (CLI-consistent; no binary file I/O this cycle). User renders the binary QR via `| xxd -r -p | qrencode -8` (documented recipe).
- **Word counts:** 12 and 24 only (SeedSigner-faithful). Refuse 15/18/21 for compact.
- **`--word-count` flag NOT needed:** byte-count (16/32) disambiguates word-count on decode; phrase determines it on encode. The FOLLOWUP-body's `--word-count` mention was for binary-sniff-ambiguity, sidestepped by explicit `--variant` + hex.

**SemVer rationale (v0.31.6 → v0.32.0 MINOR):** new `--variant` flag NAME on both `seedqr encode` + `seedqr decode` = clap surface change → trips the GUI `schema_mirror` flag-NAME-parity gate → MANDATORY paired GUI release (v0.17.0, Cycle 14b). MINOR per project precedent for new-flag surface (v0.30.0 SeedQR introductory was MINOR).

## File structure

### Source files modified (toolkit library)
- `crates/mnemonic-toolkit/src/seedqr.rs`:
  - `SeedqrError` enum: add `CompactWordCountUnsupported { got: usize }`, `CompactInvalidHex(String)`, `CompactByteCountUnsupported { got: usize }`. Add `Display` arms.
  - Add `pub fn encode_compact(phrase: &str) -> Result<String, SeedqrError>`: tokenize + checksum-validate via `Mnemonic::parse_in`; refuse word-count ∉ {12, 24}; `to_entropy()` → `hex::encode`.
  - Add `pub fn decode_compact(input: &str) -> Result<String, SeedqrError>`: strip ASCII whitespace; `hex::decode` (map err → `CompactInvalidHex`); refuse byte-count ∉ {16, 32}; `Mnemonic::from_entropy_in` → phrase string.

### Source files modified (toolkit consumer)
- `crates/mnemonic-toolkit/src/cmd/seedqr.rs`:
  - Add a `SeedqrVariant` **derived `ValueEnum`** (`Standard` / `Compact`) per the strong project convention (`CliExportFormat`, `CliNetwork`, `BsmsForm`, `MultisigPathFamily`, slip39 enum all use `#[derive(ValueEnum)]` + `#[arg(value_enum)]`). R0 M1 fold — NOT a hand-rolled `PossibleValuesParser`.
  - `SeedqrDecodeArgs`: add `variant: SeedqrVariant` field (`--variant`, default `standard`).
  - `SeedqrEncodeArgs`: add `variant: SeedqrVariant` field (`--variant`, default `standard`).
  - `run_decode`: dispatch on `args.variant` — standard → `seedqr_decode`; compact → `decode_compact`. The `--from seedqr=<value>` payload is decimal (standard) or hex (compact).
  - `run_encode`: dispatch on `args.variant` — standard → `encode`; compact → `encode_compact`.
  - `SeedqrEnvelope.variant`: set from the actual variant ("standard" | "compact"). The `digits` field holds the payload (decimal for standard, hex for compact) — the `variant` field disambiguates.

### Test files modified (toolkit)
- `crates/mnemonic-toolkit/src/seedqr.rs` (in-file `tests` mod):
  - `encode_compact_12_word` — PHRASE_12 → 32-hex-char entropy `00000000000000000000000000000000`.
  - `encode_compact_24_word` — PHRASE_24 → 64-hex-char entropy (all-zeros).
  - `decode_compact_12_word` / `decode_compact_24_word` — hex → phrase.
  - `round_trip_compact_12_word` / `round_trip_compact_24_word`.
  - `encode_compact_rejects_15_word` (CompactWordCountUnsupported).
  - `decode_compact_rejects_invalid_hex` (CompactInvalidHex).
  - `decode_compact_rejects_20_byte_count` (CompactByteCountUnsupported — 15-word entropy size, valid BIP-39 but not compact).
- `crates/mnemonic-toolkit/tests/cli_seedqr.rs` (or the unification file):
  - `encode_compact_12_word_cli` — `seedqr encode --variant compact --from phrase=<12w>` → hex stdout.
  - `decode_compact_12_word_cli` — `seedqr decode --variant compact --from seedqr=<hex>` → phrase.
  - `encode_compact_json_envelope` — asserts `variant: "compact"` + hex in `digits`.
  - `compact_round_trip_via_cli` — encode compact → decode compact byte-equal phrase.
  - `encode_compact_rejects_15_word_cli`.
  - **`decode_compact_24_word_cli`** (R0 M2 — 24-word CLI happy path; 64-hex).
  - **`decode_compact_uppercase_and_whitespace_hex`** (R0 M2 — asserts case-insensitive + whitespace-strip).
  - **`standard_decode_of_64_char_hex_clean_error`** (R0 M2 — `--variant standard --from seedqr=<64-zero-hex>` gives a clean `InvalidDigits` error, not a panic; 64 ∉ {48,60,72,84,96}).

### Documentation modified (toolkit)
- `docs/manual/src/40-cli-reference/41-mnemonic.md`:
  - `mnemonic seedqr` synopsis + flags: add `--variant <standard|compact>`.
  - §"Scope" subsection: flip CompactSeedQR from "deferred" to "shipped v0.32.0 (12/24 only, hex form)".
  - Add a worked example: compact encode → `xxd -r -p | qrencode -8` recipe for the binary QR.

### Release tooling
- `crates/mnemonic-toolkit/Cargo.toml:3` — `0.31.6` → `0.32.0`.
- `CHANGELOG.md` — new `## [0.32.0]` section.
- `scripts/install.sh:32` — pin → `mnemonic-toolkit-v0.32.0`.
- `design/FOLLOWUPS.md` — close `seedqr-compact-variant`; file `gui-seedqr-variant-flag-mirror` (v0.17.0 GUI lockstep — `--variant` net-new flag on seedqr-encode + seedqr-decode).

## Tasks

### Task 1: Phase 2 — Library encode_compact / decode_compact + error variants
- [ ] Add 3 `SeedqrError` variants + Display arms.
- [ ] Add `encode_compact` + `decode_compact`.
- [ ] Add 9 in-file unit cells.
- [ ] Build + run lib tests.
- [ ] Commit Phase 2.

### Task 2: Phase 3 — CLI --variant flag + dispatch
- [ ] Add `SeedqrVariant` enum + `--variant` on both args structs.
- [ ] Dispatch in run_encode / run_decode; set envelope variant.
- [ ] Add 5 integration cells.
- [ ] Build + run.
- [ ] Commit Phase 3.

### Task 3: Phase 4 — Manual chapter
- [ ] Synopsis + flags + Scope + worked-example (xxd→qrencode recipe).
- [ ] Manual lint.
- [ ] Commit Phase 4.

### Task 4: Phase 5 — Cycle close
- [ ] Version bump + install.sh + CHANGELOG.
- [ ] Pre-tag audit (test + clippy + manual lint).
- [ ] Opus end-of-cycle review.
- [ ] Commit + tag mnemonic-toolkit-v0.32.0 + push + GH Release.
- [ ] Wait for install-pin-check CI green.
- [ ] Close FOLLOWUP + file GUI lockstep FOLLOWUP.

### Task 5: Phase 6 — GUI v0.17.0 lockstep (Cycle 14b)
- [ ] Add `--variant` Dropdown to seedqr-encode + seedqr-decode SubcommandSchema.
- [ ] Bump pin v0.31.6 → v0.32.0.
- [ ] Tag + release mnemonic-gui-v0.17.0.

## Cross-phase invariants

- Opus R0 review on plan-doc BEFORE Phase 2.
- Opus end-of-cycle review BEFORE tag.
- No `cargo fmt --all`.
- MANDATORY GUI lockstep (`--variant` net-new flag NAME).
- install-pin-check CI gate.

## Risk register

- **Compact byte-count vs BIP-39 entropy sizes** — `Mnemonic::from_entropy_in` accepts 16/20/24/28/32 (all valid BIP-39 sizes); compact must ADDITIONALLY reject 20/24/28 (15/18/21-word). The `CompactByteCountUnsupported` check must run BEFORE `from_entropy_in` to give a compact-specific error (not a generic BIP-39 error).
- **Hex case + whitespace** — `decode_compact` strips ASCII whitespace + accepts upper/lower hex (hex crate is case-insensitive). Lock lowercase emission on encode.
- **`variant` field in JSON envelope** — already present (currently hardcoded "standard"); flip to dynamic. No wire-shape break (additive value).
- **Standard decode of hex input** (R0 check-6 tightened) — `--variant standard --from seedqr=<hex>` is safe by TWO properties: (a) hex with letters a-f fails the `InvalidDigitChar` check; (b) all-decimal compact hex (e.g. all-zeros) has length 32 (12-word) or 64 (24-word), neither of which collides with the standard digit-length set {48,60,72,84,96} → clean `InvalidDigits` error. The actual safety property is the length-set non-collision (not just "a-f non-decimal"). Locked by the `standard_decode_of_64_char_hex_clean_error` cell.
- **GUI lockstep MANDATORY** — `--variant` is a net-new flag NAME on TWO subcommands (seedqr-encode + seedqr-decode); both schemas need the flag. v0.17.0.

## Self-review (pre-R0 dispatch)
- ✓ P0 recon + SeedSigner primary-source byte format verified.
- ✓ Design locks (hex + 12/24-only) folded.
- ✓ `--word-count` correctly omitted (byte-count disambiguates).
- ✓ SemVer MINOR + mandatory GUI lockstep classified.
- ✓ Error-ordering: byte-count check before from_entropy_in.
- ✓ Test surface: 9 lib + 5 CLI cells.
