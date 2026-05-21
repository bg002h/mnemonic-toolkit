# BRAINSTORM — `mnemonic-toolkit-v0.30.0` (SeedQR encode/decode subcommand)

**Date:** 2026-05-21 (post-`mnemonic-toolkit-v0.29.0` SemVer-minor cliff).
**Source SHA at brainstorm time:** `eebf798` (post-amend Cycle 4 commit, master).
**Sync state:** local master ≡ origin/master.
**Predecessor brainstorm:** `design/BRAINSTORM_v0_28_plus_residual_followups.md` §"Cycle 5 — `mnemonic-toolkit-v0.29.1` (jade-seedqr)" (L136). Original framing's `v0.29.1` PATCH classification supersedes here — see §SemVer reclassification.
**Kickoff:** `design/CYCLE_5_KICKOFF.md`.
**R0 reviewer report:** `design/agent-reports/v0_30_0-brainstorm-r0-review.md` (Opus, YELLOW: 2C/8I/5M; all C+I folded in this revision).
**P0 recon dossier:** `design/cycle-5-p0-recon.md`.

## Architectural pivot — cycle slug rename + SemVer reclassification

The predecessor brainstorm filed this cycle under FOLLOWUP slug `wallet-import-jade-seedqr` with the assumption that SeedQR ingest would extend the `wallet-import` surface (new `--format jade-seedqr` value OR auto-detect under `--format jade`). Cycle-start brainstorm pivots on two scope-correction findings:

1. **SeedQR carries a SEED (BIP-39 entropy), not a wallet policy.** The `wallet-import` envelope is shaped around foreign wallet policies (descriptor + cosigners + threshold). Forcing SeedQR through it would require a synthetic empty-policy `ParsedImport`, which misrepresents the data and pollutes the envelope shape.
2. **SeedQR is an OPEN spec originated by SeedSigner.** Jade is one of several adopters (Coldcard, Cobo, Krux, plus SeedSigner itself). Naming the surface `jade-*` ties an open standard to one vendor.

The parent FOLLOWUP `wallet-import-jade` body (`design/FOLLOWUPS.md:2360`) already hedged: *"SeedQR formats — distinct surface — may be folded later as an inline mode rather than a wallet-import format if user-direction warrants."* This brainstorm formalizes that direction.

**New cycle slug:** `seedqr-encode-decode-subcommand` (vendor-neutral).

**Old slug disposition:** `wallet-import-jade-seedqr` marked **resolved (superseded)** at cycle-close with cross-cite to the new slug + Cycle 5 SHA.

### SemVer reclassification (R0 C2 fold)

Predecessor brainstorm targeted **PATCH `v0.29.1`** under the wallet-import additive-enum-value assumption. The architectural pivot to a top-level subcommand reclassifies per project precedent:

| Prior new-top-level-subcommand cycle | Toolkit version | SemVer class |
|---|---|---|
| `final-word` | v0.11.0 | MINOR |
| `seed-xor` | v0.12.0 | MINOR |
| `slip39` | v0.13.0 | MINOR |
| `repair` + `inspect` (paired adds) | v0.22.0 | MINOR |

**Cycle 5 target:** **MINOR `v0.30.0`** (toolkit) + paired **MINOR `v0.15.0`** (GUI). Mirrors the four-cycle precedent. Downstream GUI consumers pinning the toolkit at PATCH (`^0.29`) do NOT silently receive the new subcommand — they opt in explicitly by bumping to `^0.30`.

## Decisions locked (with the user, in this brainstorm session + R0 folds)

1. **Surface placement:** new top-level `mnemonic seedqr` subcommand. Parallels `seed-xor` / `slip39` / `final-word`. NOT under `wallet-import`. NOT under `convert`. NOT inlined into `bundle`.
2. **Variant scope:** **Standard SeedQR only**. CompactSeedQR deferred to a future FOLLOWUP (`seedqr-compact-variant`) due to sniff-ambiguity (16/32 raw bytes carry no distinguishing signature).
3. **Symmetry:** decode + encode both ship in v0.30.0.
4. **Encode output shape:** numeric string on stdout (text-only). No QR rendering, no image output. Consistent with the toolkit's uniformly text-in/text-out surface. Users pipe to any QR generator (`qrencode`, online tools, hardware-wallet UI) downstream.
5. **Word-count scope:** 12 + 24 only. SeedSigner's SeedQR spec is explicit about 12/24; 15/18/21 are filed forward as `seedqr-15-18-21-word-counts` FOLLOWUP at cycle close.
6. **Error-handling pattern (R0 C1 fold):** library-local `SeedqrError` enum in `seedqr.rs`, mapped to `ToolkitError::BadInput` at the CLI boundary via `map_seedqr_error(e: SeedqrError) -> ToolkitError`. **NO new `ToolkitError` variants.** Mirrors `final_word` / `seed_xor` / `slip39` per `lib.rs:14-28` documented pattern.
7. **Language flag (R0 I2 fold):** **OMIT `--language` flag.** SeedSigner's SeedQR spec defines encoding against the BIP-39 English wordlist only; no other language has a canonical SeedQR encoding. Decision (b) per R0 I2.
8. **JSON envelope (R0 I3 + I5 folds):** include `schema_version: "1"` as first field per `XpubSearchEnvelope` / `InspectEnvelope` / `RepairJson` precedent. Rename `kind` → `operation: "decode" | "encode"` per `seed_xor` / `slip39` `operation` field precedent.
9. **Module wiring (R0 M3 fold):** `pub mod seedqr;` in `lib.rs`, matching `final_word` / `seed_xor` / `slip39` sibling pattern.
10. **Stdin convention (R0 M5 fold):** both `--digits=-` and `--digits -` accepted (clap parses both forms equivalently).

## CLI surface (locked)

```
mnemonic seedqr decode --digits <VALUE|->                         # stdout: BIP-39 phrase + newline
mnemonic seedqr decode --digits <VALUE|-> --json-out <PATH>       # JSON envelope at PATH; stdout empty

mnemonic seedqr encode --from phrase=<VALUE|->                    # stdout: 48 or 96 digits + newline
mnemonic seedqr encode --from phrase=<VALUE|-> --json-out <PATH>  # JSON envelope at PATH; stdout empty
```

**Convention adherence:**
- `--from phrase=<VALUE|->` mirrors `seed-xor split` (`cmd/seed_xor.rs` `#[arg]` block at L46-51) + `slip39 split` (`cmd/slip39.rs` `#[arg]` block at L86-90). Stdin signaled by `phrase=-`.
- `--digits <VALUE|->` is a new dedicated flag for the decode-side input (SeedQR digits text). Stdin signaled by `--digits=-` or `--digits -` (clap parses both). NOT routed through `FromInput` because SeedQR digits are a distinct surface from the existing `phrase/xpub/xprv/ms1/...` types and adding it to `FromInput` would be a global change beyond this cycle's scope.
- `--json-out <PATH>` mirrors `seed-xor split` (`cmd/seed_xor.rs:68-69`) + `slip39 split` (`cmd/slip39.rs:137-138`). When set, stdout is empty and JSON envelope writes to PATH.
- **No `--language` flag** — SeedQR is English-locked per spec.

**JSON envelope shape (locked, R0 I3 + I5 folds):**

```json
{
  "schema_version": "1",
  "operation": "decode",
  "variant": "standard",
  "word_count": 24,
  "phrase": "abandon abandon … abandon art",
  "digits": "001000010001…"
}
```

- `decode` populates all 6 fields with `operation: "decode"`.
- `encode` populates all 6 fields with `operation: "encode"`.
- `schema_version: "1"` mirrors `XpubSearchEnvelope` / `InspectEnvelope` / `RepairJson` precedent (per CHANGELOG L318 `inspect-json-schema-version-backfill` closure).
- `operation` mirrors `seed_xor` / `slip39` `operation: "split"|"combine"` precedent.
- `variant` reserves the slot for CompactSeedQR (`"compact"`) when the future FOLLOWUP ships.

## Architecture (locked)

- **New library module:** `crates/mnemonic-toolkit/src/seedqr.rs` (top-level under `src/`, NOT under `wallet_import/`). Pure encode/decode primitives + library-local `SeedqrError` enum. ~200 LOC including the error enum + tests.
- **New CLI module:** `crates/mnemonic-toolkit/src/cmd/seedqr.rs`. Houses `SeedqrArgs` (Args + Subcommand wiring), `SeedqrDecodeArgs`, `SeedqrEncodeArgs`, `map_seedqr_error(e) -> ToolkitError::BadInput(...)`, run dispatchers, JSON envelope serde structs. ~250 LOC.
- **No changes to `wallet_import/jade.rs`** — that parser keeps owning the `multisig_file` JSON wrapper path verbatim.
- **`main.rs` plumbing:** new `Command::Seedqr(cmd::seedqr::SeedqrArgs)` variant + dispatch arm.
- **Module wiring:** `pub mod seedqr;` in `lib.rs`, slotted in alphabetical order between existing modules (currently `pub mod slip39;` is at the seed-encoding cluster; place `seedqr` immediately before it per alphabetical order).
- **No new `ToolkitError` variants.** Library-local `SeedqrError` enum is mapped to `ToolkitError::BadInput(format!("seedqr: <action>: <diagnostic>"))` at the CLI boundary in `cmd/seedqr.rs`.

## Data flow (locked)

### Decode

1. Read `--digits` input → strip whitespace/newlines/tabs.
2. Validate: exactly 48 OR 96 characters, all ASCII digits `0..=9`. Else → `SeedqrError::InvalidDigits`.
3. Chunk into 4-digit groups → parse each as `u16`.
4. Validate each index ∈ `0..=2047`. Else → `SeedqrError::InvalidWordIndex`.
5. Map index → BIP-39 English wordlist word (use `bip39::Language::English.word_list()[idx]`; per A3 recon).
6. Assemble phrase → re-parse via `bip39::Mnemonic::parse_in(Language::English, &phrase)` to validate checksum. Else → `SeedqrError::ChecksumFailure`.
7. Emit phrase on stdout (text mode) OR JSON envelope at `--json-out` (JSON mode).

### Encode

1. Read `--from phrase=` input → tokenize on whitespace → lowercase each word → re-join.
2. Validate word count ∈ `{12, 24}`. Else → `SeedqrError::InvalidWordCount`.
3. Parse + checksum-validate via `bip39::Mnemonic::parse_in(Language::English, &phrase)`. Else → `SeedqrError::ChecksumFailure` (invalid-word case folds into this too — `bip39::Error` discriminates; we collapse all `parse_in` errors to ChecksumFailure with the underlying diagnostic preserved in the formatted message).
4. Map each word → index via linear search over `Language::English.word_list()` (O(N=24·2048) ≈ 49k comparisons per phrase; <1ms; acceptable per A3 recon).
5. Format each index as `format!("{:04}", idx)`.
6. Concatenate → 48 or 96 digits → emit on stdout (text mode) OR JSON envelope at `--json-out` (JSON mode).

## Error handling (locked, R0 C1 fold)

**Library-local `SeedqrError` enum** (defined in `crates/mnemonic-toolkit/src/seedqr.rs`):

```rust
#[derive(Debug, thiserror::Error)]
pub enum SeedqrError {
    #[error("invalid digit count (expected 48 or 96; got {got})")]
    InvalidDigits { got: usize },
    #[error("invalid character at position {pos}: {ch:?}")]
    InvalidDigitChar { pos: usize, ch: char },
    #[error("invalid word index {idx} at position {pos} (must be 0..=2047)")]
    InvalidWordIndex { pos: usize, idx: u16 },
    #[error("invalid word count: {got} (only 12 or 24 supported)")]
    InvalidWordCount { got: usize },
    #[error("BIP-39 checksum failure: {0}")]
    ChecksumFailure(String),
}
```

**CLI-boundary mapper** (defined in `crates/mnemonic-toolkit/src/cmd/seedqr.rs`):

```rust
fn map_seedqr_error(e: SeedqrError, action: &str) -> ToolkitError {
    ToolkitError::BadInput(format!("seedqr: {action}: {e}"))
}
```

**Exit code:** all `SeedqrError` variants → exit **1** (via `ToolkitError::BadInput` mapping; verified `error.rs:429`). Sibling `seed-xor` / `slip39` / `final-word` all use the same exit-1 class for lib-local error surfaces.

**Stderr templates:** `seedqr: decode: <diagnostic>` / `seedqr: encode: <diagnostic>` (e.g., `seedqr: decode: invalid digit count (expected 48 or 96; got 50)`).

**No `error.rs` changes.** `ToolkitError` enum + cascade match blocks remain untouched. (R0 C1 fold removes the original Phase 3 entirely.)

### Secret-memory hygiene (added per plan-doc R0 I2 fold)

Both `--from phrase=...` (encode) and `--digits ...` (decode) carry BIP-39 secret material. Cycle B (v0.10.0) established the toolkit-wide page-pin discipline. Phase 2 implementation MUST mirror `cmd/seed_xor.rs:163-178` precedent:

- **argv-leakage advisory:** call `secret_in_argv_warning(stderr, "--from phrase=", "--from phrase=-")` for inline `--from phrase=<value>`; call `secret_in_argv_warning(stderr, "--digits ", "--digits -")` for inline `--digits <value>`.
- **`Zeroizing<String>` wrapping:** the resolved phrase + the resolved digits + the decoded phrase + the computed digits all wrap in `Zeroizing<String>`.
- **`mlock::pin_pages_for(..)` page pins:** apply to the resolved phrase and computed digits byte buffers.
- **Residual stdout leak:** the toolkit's convention accepts the stdout-side phrase/digits emission as crossing the secret boundary at the user-explicit-output point.

### Secret-classification (added per plan-doc R0 I3 fold)

`crates/mnemonic-toolkit/src/secrets.rs::flag_is_secret` enumerates secret-bearing CLI flags for GUI consumers. Phase 2 implementation MUST add `"--digits"` to the match arm (unconditionally-secret flag-level inclusion) and an accompanying `--digits classifies as secret` test cell.

**Why `--from` is NOT added:** `--from` secrecy is value-dependent (`phrase=<value>` is secret; `xpub=<value>` is not). The `secret_taxonomy::SECRET_NODE_TYPES` mechanism covers this. Flag-level inclusion is appropriate ONLY for unconditionally-secret flags; `--digits` qualifies (SeedQR-encoded BIP-39 seed → always secret).

## SemVer + lockstep (locked, R0 C2 + I8 folds)

- **Toolkit SemVer:** MINOR — `v0.29.0 → v0.30.0`. New top-level subcommand per `final-word`/`seed-xor`/`slip39`/`repair`+`inspect` precedent.
- **GUI lockstep:** **MANDATORY**. New `SubcommandSchema` entries for `seedqr-decode` + `seedqr-encode` (mirrors `seed-xor-split`/`seed-xor-combine` two-leaf precedent). `schema_mirror` integration test enforces parity.
- **GUI version:** `mnemonic-gui-v0.15.0` (MINOR; new subcommand schema entries).

## FOLLOWUP closure semantics

**Slugs closed at Cycle 5 SHIP:**
1. `wallet-import-jade-seedqr` → **Status: resolved (superseded)** with `**Resolved by:** Cycle 5 (seedqr-encode-decode-subcommand) at <Cycle-5-toolkit-SHA>` line + cross-cite to new slug.

**Slugs newly filed at Cycle 5 SHIP (cycle close, Phase 7):**
1. `seedqr-compact-variant` — CompactSeedQR (raw entropy bytes; 16/32 bytes; ambiguity-handling needed). Tier `v0.30+`.
2. `seedqr-15-18-21-word-counts` — 15/18/21-word BIP-39 phrases (60/72/84 digits). Tier `v0.30+`.
3. `seedqr-bundle-slot-integration` — option to auto-decode SeedQR file at slot-emit time via `--slot @N.seedqr=<file>` (defer-to-future-decision; not committed). Tier `v0.30+`.
4. `seedqr-digits-from-input-unification` (R0 I4 fold) — long-term surface asymmetry: `convert` uses `--from <node>=`, `seed-xor`/`slip39`/`seedqr-encode` use `--from phrase=`, but `seedqr-decode` uses `--digits <value>`. Future v0.30+ candidate: extend `FromInput` with a `seedqr=<value>` node type and deprecate `--digits` in favor of `--from seedqr=...`.

## Manual placement (locked, R0 I1 fold)

- **Primary chapter:** new `## \`mnemonic seedqr\`` section in `docs/manual/src/40-cli-reference/41-mnemonic.md`, placed AFTER the `slip39` section (currently at L1144–1586) and BEFORE the `gui-schema` section (L1587). New section spans ~150–200 lines: synopsis, flags, decode worked example, encode worked example, cross-impl smoke recipe vs SeedSigner Python ref, exit codes, stderr templates.
- **Cross-references updated:** `docs/manual/src/45-foreign-formats.md:620-626` (existing `### Deferral — SeedQR` subsection) is rewritten to redirect users to `mnemonic seedqr decode` and drop the deferral language; `docs/manual/src/45-foreign-formats.md:786` ("Jade SeedQR variant — see") is updated to point at the new `41-mnemonic.md` section. **The L607-608 `jade_specific_fields` reservation sentence stays as-is** — the field becomes truly reserved with no anticipated near-term consumer in v0.30+ (kept for potential CompactSeedQR FOLLOWUP).
- **Mirror invariant:** the manual chapter-45 changes are NON-CLI changes (foreign-formats docs); the chapter-41 addition mirrors clap-derive's `--help` output for the new `seedqr` subcommand. CI `make -C docs/manual lint` exercises the bidirectional flag-coverage check; verify locally before commit per `feedback-architect-must-run-prose-commands`.

## Fixtures (locked)

- **Synthesis policy:** SeedQR's algorithm is deterministic (`format(index, '04d')` per BIP-39 word index). No device capture needed; fixtures synthesize directly from canonical BIP-39 test vectors.
- **Canonical sources:**
  - Trezor's BIP-39 12-word vector: `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about` → entropy `00…00`, indices all-0 except last word `0003` → digits `000000000000000000000000000000000000000000000003` (44 zeros + `0003`).
  - Trezor's BIP-39 24-word vector: `abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art` → indices all-0 except last word `art` (BIP-39 0-indexed position `102`; verified against the BIP-39 English wordlist file) → 92 zeros + `0102`.
- **Cross-impl smoke recipe** (documented in manual chapter):
  - Reference impl: SeedSigner's `seedsigner` Python tools at `https://github.com/SeedSigner/seedsigner` (exact symbol path locked at Phase 0 A4 per R0 I7).
  - Recipe: feed both impls the same BIP-39 phrase; assert byte-identical digit strings.
  - Manual chapter cites the cross-impl smoke as a verification recipe users can run themselves.

## Test plan (locked, R0 M1 fold)

### Unit tests (`crates/mnemonic-toolkit/src/seedqr.rs#[cfg(test)] mod tests`)
- 12-word `all-abandon-about` round-trip (encode then decode → byte-equal phrase).
- 24-word `all-abandon-art` round-trip.
- Decode: wrong-length input rejection (47, 49, 95, 97).
- Decode: non-digit-char rejection (`004A0001…`).
- Decode: word-index `>2047` rejection (group `9999`).
- Decode: checksum-failure rejection (valid digits but indices that don't checksum).
- Encode: word-count rejection (13-word, 18-word, 25-word).
- Encode: invalid-word rejection (typo).
- Encode: checksum-failure rejection (12 valid words that don't checksum).

### CLI integration tests (`crates/mnemonic-toolkit/tests/cli_seedqr.rs`)
- Decode text mode: `--digits=<vector>` → stdout phrase.
- Decode JSON mode: `--digits=<vector> --json-out=<tempfile>` → JSON envelope at tempfile; stdout empty.
- Decode stdin mode: `--digits=-` reads from stdin.
- Decode stdin mode alt-form: `--digits -` (space-separated) reads from stdin.
- Encode text mode: `--from phrase=<vector>` → stdout digits.
- Encode JSON mode: `--from phrase=<vector> --json-out=<tempfile>` → JSON envelope at tempfile.
- Encode stdin mode: `--from phrase=-` reads from stdin.
- Round-trip: encode → decode → byte-equal phrase (CLI-level).
- Exit codes: all `SeedqrError` variants exercised end-to-end with stderr-template assertions.

**Approximate cell count:** **30-60 cells** (final count locked at plan-doc R0 per R0 M1 fold). Range accounts for: (a) 5 refusal classes × decode/encode permutations, (b) stdin/inline/JSON-out × text/json matrix, (c) round-trip CLI cells, (d) negative cells for the deferred variants.

### GUI integration tests (`mnemonic-gui/tests/schema_mirror.rs`)
- `schema_mirror` integration test verifies the hand-maintained `SubcommandSchema` entries for `seedqr-decode` + `seedqr-encode` match the `gui-schema` JSON output from the toolkit binary.
- Local verification: explicit `MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic` env var (per `feedback_a0_recon_check_gui_schema_json` + the schema-mirror-test `$PATH`-fallback gotcha verified at A2).

## Cross-cutting (locked)

- **`scripts/install.sh` self-pin:** L32 bump `v0.29.0 → v0.30.0`.
- **`Cargo.toml`:** version `0.29.0 → 0.30.0`.
- **CHANGELOG entry:** new `[Unreleased]` → `## [0.30.0] — 2026-MM-DD` block with added/changed/fixed sections.
- **No `ToolkitError` variants added** (R0 C1 fold; library-local `SeedqrError` mapped at CLI boundary). No `error.rs` changes.
- **`gui-schema` JSON emission:** verify post-implementation that `mnemonic gui-schema` emits two `seedqr-decode` + `seedqr-encode` subcommand entries; cross-check vs `schema_mirror` per `feedback_a0_recon_check_gui_schema_json` (Cycle 4 surprise).

## Phase decomposition

### Phase 0 — P0 STRICT-GATE recon (~1-2 h) — DONE (this dossier)

Dispatch parallel Explore (read-only) agents:
- **A1:** Re-verify cited line numbers in this brainstorm against current `master`. ✓ done (see `design/cycle-5-p0-recon.md` §A1).
- **A2:** Inspect `mnemonic-gui` repo state. ✓ done (§A2).
- **A3:** Verify `bip39` crate availability + identify the exact symbol path. ✓ done (§A3).
- **A4 (added per R0 I7):** Verify SeedSigner Python reference symbol path. **DEFERRED to Phase 4 prelude** (manual writing); time-boxed at <1h. Output appended to recon dossier as §A4 at that time.

### Phase 1 — TDD `seedqr.rs` core library + `SeedqrError` enum

Test-first per CLAUDE.md per-phase TDD. Author unit tests + library primitives + library-local `SeedqrError` enum (R0 C1 fold). Reviewer-loop until 0C/0I.

### Phase 2 — `cmd/seedqr.rs` CLI wiring + `map_seedqr_error` boundary

Add subcommand + flag plumbing + `main.rs` dispatch arm + `map_seedqr_error(e, action) -> ToolkitError::BadInput` mapper. Reviewer-loop until 0C/0I.

### Phase 3 (formerly "ToolkitError variant additions" — REMOVED per R0 C1 fold)

NO Phase 3. The library-local `SeedqrError` design eliminates the need for `error.rs` cascade work. Phases renumber: old Phase 4 → new Phase 3, etc.

### Phase 3 (was 4) — CLI integration tests + JSON envelope schema

Author `tests/cli_seedqr.rs`. JSON envelope serde struct (with `schema_version: &'static str` + `operation: &'static str` + `variant: &'static str` per R0 I3+I5 folds) + assertions. Reviewer-loop until 0C/0I.

### Phase 4 (was 5) — Manual chapter (chapter-41 add + chapter-45 update)

Write `## \`mnemonic seedqr\`` section in `41-mnemonic.md` + update cross-refs in `45-foreign-formats.md` (corrected to L620-626 per R0 I1 fold). Phase 4 prelude: complete A4 SeedSigner symbol-path recon. Run `make -C docs/manual lint` locally per `feedback-architect-must-run-prose-commands`. Reviewer-loop until 0C/0I.

### Phase 5 (was 6) — Cycle close (commit + tag + push + GH Release)

Split-commit hygiene check per Cycle 4 R0-I3: no mechanical-only work piggybacks the cycle for v0.30.0; single commit anticipated. SHA self-reference under amend pattern accepted (Cycle 2/3/4 recurrence). Toolkit tag `mnemonic-toolkit-v0.30.0`. Push. Verify install-pin-check CI green. GH Release.

### Phase 6 (was 7) — GUI lockstep

After toolkit tag is live:
1. GUI pin bump (`mnemonic-gui/pinned-upstream.toml` + `Cargo.toml`) + `cargo update`.
2. New `SubcommandSchema` entries for `seedqr-decode` + `seedqr-encode` in `mnemonic-gui/src/schema/mnemonic.rs`.
3. Verify `schema_mirror` test locally with explicit `MNEMONIC_BIN=...`.
4. GUI CHANGELOG + Cargo.toml version bump → `v0.15.0`.
5. Commit + tag + push.
6. Closure-verification: GUI CI `schema_mirror` gate green on tag.

### Phase 7 (was 8) — End-of-cycle opus review + FOLLOWUP closure

Per Cycle 4 precedent + CLAUDE.md mandate: dispatch opus on full uncommitted working tree (toolkit + GUI). Persist verbatim at `design/agent-reports/v0_30_0-end-of-cycle-review.md`. Fold any C/I findings before final tag. Close FOLLOWUP slug `wallet-import-jade-seedqr` (resolved-superseded) + file 4 new FOLLOWUPs per §FOLLOWUP closure semantics above.

## Effort estimate

| Phase | Effort |
|---|---|
| 0 (recon) | 1-2 h ✓ done (A1-A3); A4 deferred (<1h) |
| 1 (`seedqr.rs` lib + TDD + `SeedqrError`) | 3-4 h |
| 2 (CLI wiring + `map_seedqr_error` boundary) | 2 h |
| 3 (CLI tests + JSON envelope) | 3-4 h |
| 4 (manual + A4 prelude) | 2-3 h |
| 5 (cycle close + toolkit tag) | 1 h |
| 6 (GUI lockstep) | 1-2 h |
| 7 (end-of-cycle review + FOLLOWUP closure) | 1 h |
| **Total** | **13-17 h** (single working session OR 2-3 sessions) |

Cycle 5 is **NOT multi-week** as the predecessor brainstorm framed it. The "multi-week" framing assumed wallet-import-envelope plumbing + cross-format-mismatch-matrix wiring, none of which apply to a top-level subcommand. Effort is comparable to v0.11.0 (`final-word`) or v0.12.0 (`seed-xor`) cycles.

## Disciplines preserved from Cycles 1-4

- **P0 STRICT-GATE recon** before plan-doc body. ✓ done.
- **Plan-doc reviewer-loop** (Opus R0 → persist verbatim → fold → R1 verify → until 0C/0I) applies to brainstorm-spec too per CLAUDE.md. R0 done; R1 pending after this fold.
- **Per-phase TDD** (tests before impl).
- **Per-phase reviewer-loop** until 0C/0I (sonnet for trivial fold-verify, opus for non-trivial).
- **End-of-cycle opus review** on full working tree.
- **Bisect-hygiene split commits** if mechanical work bundled (anticipated: NONE for v0.30.0).
- **SHA self-reference under amend** accepted (Cycle 2/3/4 recurrence).
- **Cross-repo lockstep ordering** — toolkit tag first, GUI pin bump second.
- **Stale `$PATH` binary gotcha** — explicit `MNEMONIC_BIN=...` for local `schema_mirror`.
- **`schema_mirror` gate scope** — clap flag-name parity only; JSON wire-shape NOT gated by the mirror.

## Memory entries consulted

- `project_v0_29_0_cycle_shipped` — Cycle 4 full context.
- `project_v0_28_7_cycle_shipped` — Cycle 3.
- `feedback_a0_recon_check_gui_schema_json` — toolkit `--help` vs `gui-schema` JSON divergence.
- `feedback_no_parallelism_for_code_generation` — agent-N+1 parallelism unsafe; worktree-isolation invariant.
- `feedback_opus_primary_review_agent` — opus is the primary review agent for substantive cycles.
- `feedback_architect_must_run_prose_commands` — for manual chapters: run the commands the prose claims.
- `feedback_r0_must_read_source_off_by_n` — every R0 should grep against source ground truth.
- `feedback_verify_bundle_round_trip_per_phase_r0_scope` — Phase 4 R0 must include verify-bundle round-trip if bundle-side state mutates (N/A this cycle: no bundle state changes).

## R0 fold summary

This brainstorm revision folds the following R0 findings from `design/agent-reports/v0_30_0-brainstorm-r0-review.md`:

| R0 finding | Fold action |
|---|---|
| C1 — direct `ToolkitError` variants violate lib-local pattern | Library-local `SeedqrError` enum + `map_seedqr_error` CLI boundary. Phase 3 removed. |
| C2 — PATCH inverts new-subcommand MINOR precedent | Reclassified v0.29.1 → v0.30.0; GUI v0.14.1 → v0.15.0. Filename renamed. |
| I1 — chapter-45 citation off (L608 → L620) | Citation corrected. |
| I2 — encode-side `--language` ambiguity | Locked: no `--language` flag; English implicit per spec. |
| I3 — JSON envelope missing `schema_version` | Added as first field. |
| I4 — `--digits` long-term shape unclear | New FOLLOWUP `seedqr-digits-from-input-unification` to be filed at cycle close. |
| I5 — `kind` discriminator namespace clash | Renamed `kind` → `operation` matching sibling pattern. |
| I6 — Phase 0 dossier naming | Verified `cycle-N-p0-recon.md` matches Cycle 3/4 precedent. |
| I7 — Phase 0 SeedSigner symbol recon | Added A4 task to Phase 0 (deferred to Phase 4 prelude). |
| I8 — GUI version cascade from C2 | Cascaded: GUI v0.15.0. |
| M1 — cell count understated | Widened to 30-60 (final lock at plan-doc R0). |
| M2 — predecessor anchor | Verified; no change. |
| M3 — module-wiring vague | Locked: `pub mod seedqr;` in `lib.rs`. |
| M4 — phase-numbering vs Cycle 3/4 | Intentional simplification; no change (single-slug cycle). |
| M5 — stdin convention | Loosened: both `--digits=-` and `--digits -` accepted. |

**Resulting verdict expectation for R1:** GREEN (0C/0I).

## Open questions for R1 reviewer

None at fold-write time. All R0 findings folded. R1 verifies the folds did not introduce new C/I + that nothing R0 caught remains.
