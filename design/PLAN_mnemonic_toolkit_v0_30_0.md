# mnemonic-toolkit-v0.30.0 Implementation Plan (Cycle 5 — SeedQR encode/decode subcommand)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `mnemonic-toolkit-v0.30.0` (SemVer-MINOR; new top-level `mnemonic seedqr` subcommand for SeedQR encode/decode) + paired `mnemonic-gui-v0.15.0` (`SubcommandSchema` entries for `seedqr-decode` + `seedqr-encode`). Closes FOLLOWUP `wallet-import-jade-seedqr` (resolved-superseded by new slug `seedqr-encode-decode-subcommand`).

**Architecture:** New top-level `mnemonic seedqr` subcommand paralleling `seed-xor` / `slip39` / `final-word`. Library-local `SeedqrError` enum in `seedqr.rs` with hand-rolled `impl Display` (per `seed_xor.rs:31-67` precedent — NOT `thiserror::Error` derive). Library bip39 errors wrap into `SeedqrError::ChecksumFailure(String)`. CLI boundary maps `SeedqrError → ToolkitError::BadInput(format!("seedqr: {action}: {e}"))` via `map_seedqr_error`. NO new `ToolkitError` variants. NO new Cargo.toml deps (bip39 crate already available; thiserror NOT used). Standard SeedQR only (12 + 24 word; English-locked). Secret-memory hygiene applied via `Zeroizing` + `mlock::pin_pages_for` + `secret_in_argv_warning` per Cycle B (v0.10.0) discipline.

**Tech Stack:** Rust + clap-derive + serde + `bip39` crate v2 (already a dep). `make audit` for regression checks. `mnemonic gui-schema` for downstream JSON consumer.

**Brainstorm spec:** `design/BRAINSTORM_v0_30_0_seedqr.md` (committed 4d82a3c). P0 recon dossier: `design/cycle-5-p0-recon.md`. R0 reviews: `design/agent-reports/v0_30_0-brainstorm-{r0,r1}-review.md` + `design/agent-reports/v0_30_0-plan-doc-r0-review.md`.

**Source SHA at plan-write time:** `4d82a3c` (brainstorm commit).

**P0 STRICT-GATE locks + R0 folds applied to this plan-doc:**
- Subcommand placement: top-level `mnemonic seedqr {decode, encode}`.
- Variant scope: Standard SeedQR only. Word counts: 12 + 24 only. English-locked (no `--language` flag).
- Error pattern: library-local `SeedqrError` with hand-rolled `impl Display` (R0 C1 fold). Mapped via `map_seedqr_error` to `ToolkitError::BadInput`. NO `error.rs` changes. NO `thiserror` dep.
- **Exit code: 1** (`ToolkitError::BadInput(_) => 1` per `error.rs:429`; R0 C2 fold). NOT exit 2.
- JSON envelope: `{schema_version: "1", operation: "decode"|"encode", variant: "standard", word_count, phrase, digits}`.
- CLI shape: `mnemonic seedqr decode --digits <VALUE|->` + `mnemonic seedqr encode --from phrase=<VALUE|->`; both support `--json-out <PATH>`.
- BIP-39 machinery: `bip39::Mnemonic::parse_in(Language::English, ...)` for checksum-validating parse; `Language::English.word_list()` for raw wordlist. **"art" is at BIP-39 index 102 (NOT 99)** — verified against the English wordlist file (R0 C4 fold).
- Encode-side `--from` MUST validate `args.from.node == NodeType::Phrase` (R0 I1 fold) — mirrors `cmd/seed_xor.rs:163-167`.
- Encode-side `--from` clap-derive arg MUST include `value_parser = parse_from_input` (R0 C3 fold) — `FromInput` has no `FromStr` impl.
- Secret-memory hygiene MUST be applied (R0 I2 fold): `Zeroizing<String>` on resolved phrase/digits + `mlock::pin_pages_for(..)` page pins + `secret_in_argv_warning` advisories for inline-form input.
- `secrets.rs::flag_is_secret` MUST be updated to include `"--digits"` (R0 I3 fold). `--from` is NOT added (value-dependent secrecy handled via `secret_taxonomy::SECRET_NODE_TYPES`).
- Test cell count target: **≥30 cells** (R0 I6 fold; brainstorm-locked range 30-60).
- GUI lockstep: MANDATORY. Two new `SubcommandSchema` entries (`seedqr-encode` + `seedqr-decode`) placed between `seed-xor-combine` (L2367) and `slip39-split` (L2375; R0 I4 fold). **Encode before decode** within the seedqr group per the seed-xor / slip39 verb-ordering precedent (create-side before recover-side).

**SemVer policy:** MINOR per v0.11/v0.12/v0.13/v0.22 new-top-level-subcommand precedent.

---

## File structure

### Source files created (toolkit)

- `crates/mnemonic-toolkit/src/seedqr.rs` — library: `decode()` / `encode()` + library-local `SeedqrError` enum with hand-rolled `impl Display`. ~250 LOC.
- `crates/mnemonic-toolkit/src/cmd/seedqr.rs` — CLI: clap-derive args + `map_seedqr_error` + run dispatchers + JSON envelope serde. ~300 LOC.
- `crates/mnemonic-toolkit/tests/cli_seedqr.rs` — CLI integration suite. ≥30 cells. ~500 LOC.

### Source files modified (toolkit)

- `crates/mnemonic-toolkit/src/lib.rs` — add `pub mod seedqr;` at L63 (alphabetical: between `seed_xor` and `slip39`); append `seedqr` to the lib-local-error doc-comment list at L14-28.
- `crates/mnemonic-toolkit/src/cmd/mod.rs` — add `pub mod seedqr;` at L14 (alphabetical: between `seed_xor` and `slip39`).
- `crates/mnemonic-toolkit/src/main.rs` — add `Command::Seedqr(cmd::seedqr::SeedqrArgs)` variant in alphabetical position (between `SeedXor` and `Slip39`) + dispatch arm (NO `.map(|_| 0)` per R0 I5 fold).
- `crates/mnemonic-toolkit/src/secrets.rs:49-59` — add `"--digits"` to `flag_is_secret` match arm + add `--digits classifies as secret` test cell.

### Documentation modified (toolkit)

- `docs/manual/src/40-cli-reference/41-mnemonic.md` — new `## \`mnemonic seedqr\`` section between `slip39` (L1144-1586) and `gui-schema` (L1587).
- `docs/manual/src/45-foreign-formats.md` — rewrite `### Deferral — SeedQR` at L620-626 + update L786 bullet.

### Source files modified (mnemonic-gui)

- `mnemonic-gui/pinned-upstream.toml` — `[mnemonic].tag` v0.29.0 → v0.30.0.
- `mnemonic-gui/Cargo.toml` — workspace dep tag v0.29.0 → v0.30.0 + workspace version → v0.15.0.
- `mnemonic-gui/src/schema/mnemonic.rs` — add `SubcommandSchema` entries for `seedqr-encode` + `seedqr-decode` between L2367 and L2375.
- `mnemonic-gui/CHANGELOG.md` — new v0.15.0 entry.

### Release tooling

- `crates/mnemonic-toolkit/Cargo.toml:3` — version `0.29.0` → `0.30.0`.
- `CHANGELOG.md` — new `## [0.30.0] — 2026-MM-DD` section.
- `scripts/install.sh:32` — `mnemonic-toolkit-v0.29.0` → `mnemonic-toolkit-v0.30.0`.
- `design/FOLLOWUPS.md` — close 1 slug + file 4 new slugs.

---

## Tasks

### Task 1: Phase 0 (continued) — A4 SeedSigner reference symbol recon

**Files:** modify `design/cycle-5-p0-recon.md` (append §A4).

Deferred Phase 0 task per brainstorm R0 I7 fold. Lock the Python reference symbol path NOW so Phase 4 manual writing can cite it.

- [ ] **Step 1: WebFetch SeedSigner repo + locate SeedQR encoder symbol**

```
WebFetch url=https://github.com/SeedSigner/seedsigner prompt="Locate the Python file/function that implements SeedQR encoding (numeric BIP-39-indices format)"
```

Then follow up with a direct file fetch (e.g., `WebFetch url=https://raw.githubusercontent.com/SeedSigner/seedsigner/main/src/seedsigner/helpers/qr.py prompt="..."`) to confirm the exact module path and function/method name.

- [ ] **Step 2: Append §A4 to recon dossier**

In `design/cycle-5-p0-recon.md`, after §A3, add a §A4 block citing the verified Python symbol path + a literal recipe block users can run.

- [ ] **Step 3: Commit recon update**

```bash
git add design/cycle-5-p0-recon.md
git commit -m "design(cycle-5): P0 recon §A4 — SeedSigner Python ref symbol path verified"
```

---

### Task 2: Phase 1 — Author `seedqr.rs` library with TDD

**Files:**
- Create: `crates/mnemonic-toolkit/src/seedqr.rs`
- Modify: `crates/mnemonic-toolkit/src/lib.rs`

- [ ] **Step 1: Add module declaration to `lib.rs`**

In `crates/mnemonic-toolkit/src/lib.rs`, between L62 (`pub mod seed_xor;`) and L63 (`pub mod slip39;`), insert:

```rust
pub mod seedqr;
```

Verify alphabetical: `seed_xor` (`_` = 0x5F at pos 4) < `seedqr` (`q` = 0x71 at pos 4) < `slip39` (`l` at pos 1).

- [ ] **Step 2: Update lib.rs doc-comment block**

In `lib.rs:14-28`, the existing block documents the lib-local-error pattern for `final_word` / `seed_xor` / `slip39`. Append a fourth bullet:

```rust
//! - `seedqr` — SeedQR encode/decode subcommand (v0.30.0). Defines a
//!   small, self-contained `SeedqrError` so the library surface does
//!   not pull in the binary-private `ToolkitError`. The CLI handler in
//!   `src/cmd/seedqr.rs` (P2) converts `SeedqrError` into
//!   `ToolkitError::BadInput` at the boundary via
//!   `map_seedqr_error(e, action)`.
```

- [ ] **Step 3: Write the failing tests (TDD)**

Create `crates/mnemonic-toolkit/src/seedqr.rs`. Top of file (header + types + impls + bodies as `todo!()`):

```rust
//! SeedQR encode/decode primitives (v0.30.0 / Cycle 5).
//!
//! SeedQR is an open spec originated by SeedSigner: BIP-39 mnemonic
//! encoded as a numeric-string QR payload where each English-wordlist
//! index is rendered as a 4-digit zero-padded decimal.
//!
//! ## Scope (v0.30.0)
//!
//! - Variants: Standard SeedQR only.
//! - Word counts: 12 + 24 only.
//! - Language: English only.
//!
//! ## Error pattern
//!
//! Library-local `SeedqrError` enum with hand-rolled `impl Display`
//! (mirrors `seed_xor.rs:31-67` precedent). CLI boundary in
//! `cmd/seedqr.rs` converts via `map_seedqr_error(e, action)`.

use bip39::{Language, Mnemonic};

/// Library-local error. Mapped to `ToolkitError::BadInput` at the CLI
/// boundary via `cmd::seedqr::map_seedqr_error`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedqrError {
    InvalidDigits { got: usize },
    InvalidDigitChar { pos: usize, ch: char },
    InvalidWordIndex { pos: usize, idx: u16 },
    InvalidWordCount { got: usize },
    ChecksumFailure(String),
}

impl std::fmt::Display for SeedqrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeedqrError::InvalidDigits { got } => write!(
                f,
                "invalid digit count (expected 48 or 96; got {got})",
            ),
            SeedqrError::InvalidDigitChar { pos, ch } => write!(
                f,
                "invalid character at position {pos}: {ch:?}",
            ),
            SeedqrError::InvalidWordIndex { pos, idx } => write!(
                f,
                "invalid word index {idx} at position {pos} (must be 0..=2047)",
            ),
            SeedqrError::InvalidWordCount { got } => write!(
                f,
                "invalid word count: {got} (only 12 or 24 supported)",
            ),
            SeedqrError::ChecksumFailure(msg) => write!(
                f,
                "BIP-39 checksum failure: {msg}",
            ),
        }
    }
}

impl std::error::Error for SeedqrError {}

/// Decode a SeedQR numeric string into a BIP-39 phrase.
pub fn decode(input: &str) -> Result<String, SeedqrError> {
    todo!()
}

/// Encode a BIP-39 phrase into a SeedQR numeric string.
pub fn encode(phrase: &str) -> Result<String, SeedqrError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    // Canonical BIP-39 12-word test vector (Trezor): all-abandon-about.
    // "about" BIP-39 index 3 (zero-based; verified against English wordlist
    // file: line 4 = "about").
    const PHRASE_12: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const DIGITS_12: &str = "000000000000000000000000000000000000000000000003";

    // Canonical BIP-39 24-word test vector (Trezor): all-abandon-art.
    // "art" BIP-39 index 102 (zero-based; verified against English wordlist
    // file: line 103 = "art"). 92 zeros + "0102" = 96 digits.
    const PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
    const DIGITS_24: &str = "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000102";

    #[test]
    fn decode_12_word_canonical() {
        assert_eq!(decode(DIGITS_12).unwrap(), PHRASE_12);
    }

    #[test]
    fn decode_24_word_canonical() {
        assert_eq!(decode(DIGITS_24).unwrap(), PHRASE_24);
    }

    #[test]
    fn encode_12_word_canonical() {
        assert_eq!(encode(PHRASE_12).unwrap(), DIGITS_12);
    }

    #[test]
    fn encode_24_word_canonical() {
        assert_eq!(encode(PHRASE_24).unwrap(), DIGITS_24);
    }

    #[test]
    fn round_trip_12_word() {
        let encoded = encode(PHRASE_12).unwrap();
        assert_eq!(decode(&encoded).unwrap(), PHRASE_12);
    }

    #[test]
    fn round_trip_24_word() {
        let encoded = encode(PHRASE_24).unwrap();
        assert_eq!(decode(&encoded).unwrap(), PHRASE_24);
    }

    #[test]
    fn decode_strips_whitespace() {
        let padded = format!(" {DIGITS_12} \n\t");
        assert_eq!(decode(&padded).unwrap(), PHRASE_12);
    }

    #[test]
    fn decode_rejects_wrong_length_47() {
        let bad = &DIGITS_12[..47];
        assert!(matches!(decode(bad), Err(SeedqrError::InvalidDigits { got: 47 })));
    }

    #[test]
    fn decode_rejects_wrong_length_49() {
        let bad = format!("{DIGITS_12}0");
        assert!(matches!(decode(&bad), Err(SeedqrError::InvalidDigits { got: 49 })));
    }

    #[test]
    fn decode_rejects_wrong_length_95() {
        let bad = &DIGITS_24[..95];
        assert!(matches!(decode(bad), Err(SeedqrError::InvalidDigits { got: 95 })));
    }

    #[test]
    fn decode_rejects_wrong_length_97() {
        let bad = format!("{DIGITS_24}0");
        assert!(matches!(decode(&bad), Err(SeedqrError::InvalidDigits { got: 97 })));
    }

    #[test]
    fn decode_rejects_non_digit_char() {
        let bad = "00000000000000000000000000000000000000000000000A";
        assert!(matches!(decode(bad), Err(SeedqrError::InvalidDigitChar { pos: 47, ch: 'A' })));
    }

    #[test]
    fn decode_rejects_word_index_out_of_range() {
        let bad = format!("9999{}", &DIGITS_12[4..]);
        assert!(matches!(decode(&bad), Err(SeedqrError::InvalidWordIndex { pos: 0, idx: 9999 })));
    }

    #[test]
    fn decode_rejects_checksum_failure() {
        // 12 valid word indices but indices that don't checksum.
        let bad = "000100010001000100010001000100010001000100010001";
        assert!(matches!(decode(bad), Err(SeedqrError::ChecksumFailure(_))));
    }

    #[test]
    fn encode_rejects_13_word_count() {
        let bad = format!("{PHRASE_12} abandon");
        assert!(matches!(encode(&bad), Err(SeedqrError::InvalidWordCount { got: 13 })));
    }

    #[test]
    fn encode_rejects_18_word_count() {
        let bad = "abandon ".repeat(17) + "about";
        assert!(matches!(encode(&bad), Err(SeedqrError::InvalidWordCount { got: 18 })));
    }

    #[test]
    fn encode_rejects_invalid_word() {
        let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon notaword";
        // bip39::Mnemonic::parse_in's invalid-word error collapses into
        // SeedqrError::ChecksumFailure (with the underlying diagnostic
        // preserved).
        assert!(matches!(encode(bad), Err(SeedqrError::ChecksumFailure(_))));
    }

    #[test]
    fn encode_rejects_checksum_failure() {
        let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
        assert!(matches!(encode(bad), Err(SeedqrError::ChecksumFailure(_))));
    }
}
```

- [ ] **Step 4: Run tests to verify they fail**

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
cargo test --package mnemonic-toolkit --lib seedqr 2>&1 | tail -20
```

Expected: all tests FAIL with `todo!()` panic.

- [ ] **Step 5: Implement `decode`**

Replace the `decode` body with:

```rust
pub fn decode(input: &str) -> Result<String, SeedqrError> {
    // Strip all ASCII whitespace.
    let stripped: String = input.chars().filter(|c| !c.is_ascii_whitespace()).collect();

    // Validate length.
    let len = stripped.len();
    if len != 48 && len != 96 {
        return Err(SeedqrError::InvalidDigits { got: len });
    }

    // Validate all ASCII digits.
    for (pos, ch) in stripped.chars().enumerate() {
        if !ch.is_ascii_digit() {
            return Err(SeedqrError::InvalidDigitChar { pos, ch });
        }
    }

    // Chunk into 4-digit groups → word indices → words.
    let wordlist = Language::English.word_list();
    let mut words: Vec<&'static str> = Vec::with_capacity(len / 4);
    for (group, chunk) in stripped.as_bytes().chunks(4).enumerate() {
        // SAFETY: chunk is 4 ASCII bytes per prior digit-validation loop.
        let s = std::str::from_utf8(chunk).expect("ASCII digits");
        let idx: u16 = s.parse().expect("4 ASCII digits parse to u16");
        if idx as usize >= wordlist.len() {
            return Err(SeedqrError::InvalidWordIndex { pos: group * 4, idx });
        }
        words.push(wordlist[idx as usize]);
    }

    let phrase = words.join(" ");

    // Checksum-validate via bip39 crate.
    Mnemonic::parse_in(Language::English, &phrase)
        .map_err(|e| SeedqrError::ChecksumFailure(e.to_string()))?;

    Ok(phrase)
}
```

- [ ] **Step 6: Implement `encode`**

Replace the `encode` body with:

```rust
pub fn encode(phrase: &str) -> Result<String, SeedqrError> {
    // Tokenize on whitespace, lowercase.
    let words: Vec<String> = phrase
        .split_whitespace()
        .map(|w| w.to_lowercase())
        .collect();

    // Validate word count.
    if words.len() != 12 && words.len() != 24 {
        return Err(SeedqrError::InvalidWordCount { got: words.len() });
    }

    // Parse + checksum-validate via bip39 (also rejects invalid words).
    let normalized = words.join(" ");
    Mnemonic::parse_in(Language::English, &normalized)
        .map_err(|e| SeedqrError::ChecksumFailure(e.to_string()))?;

    // Map each word to its index via linear search.
    let wordlist = Language::English.word_list();
    let mut digits = String::with_capacity(words.len() * 4);
    for word in &words {
        let idx = wordlist
            .iter()
            .position(|w| *w == word.as_str())
            .expect("bip39::Mnemonic::parse_in already validated word membership") as u16;
        digits.push_str(&format!("{idx:04}"));
    }

    Ok(digits)
}
```

- [ ] **Step 7: Run tests to verify they pass**

```bash
cargo test --package mnemonic-toolkit --lib seedqr 2>&1 | tail -20
```

Expected: all 17 tests PASS.

- [ ] **Step 8: Run full lib suite for regressions**

```bash
cargo test --package mnemonic-toolkit --lib 2>&1 | tail -5
```

Expected: existing lib tests still pass.

- [ ] **Step 9: Commit**

```bash
git add crates/mnemonic-toolkit/src/seedqr.rs crates/mnemonic-toolkit/src/lib.rs
git commit -m "feat(seedqr): Phase 1 — seedqr.rs library + SeedqrError enum

Pure encode/decode primitives in src/seedqr.rs with library-local
SeedqrError enum + hand-rolled impl Display (per seed_xor.rs:31-67
precedent — no thiserror dep). 17 unit cells covering 12/24-word
canonical round-trips + 5 refusal classes. art is at BIP-39 index 102
(verified against English wordlist).

Phase 1 of design/PLAN_mnemonic_toolkit_v0_30_0.md."
```

---

### Task 3: Phase 2 — `cmd/seedqr.rs` CLI wiring + `map_seedqr_error` boundary + secrets.rs

**Files:**
- Create: `crates/mnemonic-toolkit/src/cmd/seedqr.rs`
- Modify: `crates/mnemonic-toolkit/src/cmd/mod.rs:14`
- Modify: `crates/mnemonic-toolkit/src/main.rs`
- Modify: `crates/mnemonic-toolkit/src/secrets.rs`

- [ ] **Step 1: Add module declaration to `cmd/mod.rs`**

In `crates/mnemonic-toolkit/src/cmd/mod.rs`, between L13 (`pub mod seed_xor;`) and L14 (`pub mod slip39;`), insert:

```rust
pub mod seedqr;
```

- [ ] **Step 2: Create `cmd/seedqr.rs`**

Create `crates/mnemonic-toolkit/src/cmd/seedqr.rs`:

```rust
//! `mnemonic seedqr` subcommand (v0.30.0 / Cycle 5).
//!
//! Wraps the `seedqr` library module's `decode` / `encode` primitives
//! in a clap-derive CLI surface. Library-local `SeedqrError` is mapped
//! to `ToolkitError::BadInput` at the boundary via `map_seedqr_error`,
//! mirroring `cmd/seed_xor.rs` / `cmd/slip39.rs` / `cmd/final_word.rs`
//! per `lib.rs:14-28` documented pattern.

use crate::cmd::convert::{parse_from_input, read_stdin_to_string, FromInput, NodeType};
use crate::error::ToolkitError;
use crate::secret_advisory::secret_in_argv_warning;
use crate::seedqr::{decode as seedqr_decode, encode as seedqr_encode, SeedqrError};
use clap::{Args, Subcommand};
use std::io::{Read, Write};

#[derive(Args, Debug)]
pub struct SeedqrArgs {
    #[command(subcommand)]
    pub action: SeedqrAction,
}

#[derive(Subcommand, Debug)]
pub enum SeedqrAction {
    /// decode a SeedQR numeric string into a BIP-39 phrase
    Decode(SeedqrDecodeArgs),
    /// encode a BIP-39 phrase into a SeedQR numeric string
    Encode(SeedqrEncodeArgs),
}

#[derive(Args, Debug, Clone)]
pub struct SeedqrDecodeArgs {
    /// SeedQR numeric digit string (48 or 96 ASCII digits). `-` reads from stdin.
    #[arg(long = "digits", value_name = "VALUE|-")]
    pub digits: String,

    /// Write JSON envelope to PATH (stdout empty when set).
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct SeedqrEncodeArgs {
    /// Phrase input as `phrase=<value>` (inline) or `phrase=-` (stdin).
    #[arg(
        long = "from",
        value_name = "phrase=VALUE|-",
        value_parser = parse_from_input,
        required = true,
    )]
    pub from: FromInput,

    /// Write JSON envelope to PATH (stdout empty when set).
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,
}

/// Maps a library-local `SeedqrError` to a CLI-boundary `ToolkitError`.
fn map_seedqr_error(e: SeedqrError, action: &str) -> ToolkitError {
    ToolkitError::BadInput(format!("seedqr: {action}: {e}"))
}

/// JSON envelope (mirrors XpubSearchEnvelope / InspectEnvelope /
/// RepairJson precedent: schema_version first; operation discriminator second).
#[derive(serde::Serialize)]
struct SeedqrEnvelope<'a> {
    schema_version: &'a str,
    operation: &'a str,
    variant: &'a str,
    word_count: usize,
    phrase: &'a str,
    digits: &'a str,
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &SeedqrArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    match &args.action {
        SeedqrAction::Decode(a) => run_decode(a, stdin, stdout, stderr),
        SeedqrAction::Encode(a) => run_encode(a, stdin, stdout, stderr),
    }
}

fn run_decode<R: Read, W: Write, E: Write>(
    args: &SeedqrDecodeArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // Argv-leakage advisory for inline form.
    if args.digits != "-" {
        secret_in_argv_warning(stderr, "--digits ", "--digits -");
    }

    // Resolve --digits value (inline or stdin); wrap in Zeroizing.
    let digits: zeroize::Zeroizing<String> = if args.digits == "-" {
        zeroize::Zeroizing::new(read_stdin_to_string(stdin)?)
    } else {
        zeroize::Zeroizing::new(args.digits.clone())
    };
    let _pin_digits = mnemonic_toolkit::mlock::pin_pages_for(digits.as_bytes());

    // Decode via library primitive.
    let phrase_plain = seedqr_decode(digits.as_str()).map_err(|e| map_seedqr_error(e, "decode"))?;
    let phrase: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(phrase_plain);
    let _pin_phrase = mnemonic_toolkit::mlock::pin_pages_for(phrase.as_bytes());

    // Canonical 48/96-digit form for JSON envelope echo.
    let canonical_digits: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(
        digits.chars().filter(|c| !c.is_ascii_whitespace()).collect(),
    );
    let word_count = phrase.split_whitespace().count();

    emit_decode_output(args, phrase.as_str(), canonical_digits.as_str(), word_count, stdout)
}

fn run_encode<R: Read, W: Write, E: Write>(
    args: &SeedqrEncodeArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // Validate that --from carries a phrase= node (NOT xpub=, ms1=, etc.).
    // Mirrors cmd/seed_xor.rs:163-167.
    if args.from.node != NodeType::Phrase {
        return Err(ToolkitError::BadInput(
            "seedqr encode only accepts phrase=<value> or phrase=-".into(),
        ));
    }

    // Argv-leakage advisory for inline form.
    if args.from.value != "-" {
        secret_in_argv_warning(stderr, "--from phrase=", "--from phrase=-");
    }

    // Resolve phrase input (inline or stdin); wrap in Zeroizing.
    let phrase: zeroize::Zeroizing<String> = if args.from.value == "-" {
        zeroize::Zeroizing::new(read_stdin_to_string(stdin)?)
    } else {
        zeroize::Zeroizing::new(args.from.value.clone())
    };
    let _pin_phrase = mnemonic_toolkit::mlock::pin_pages_for(phrase.as_bytes());

    // Encode via library primitive.
    let digits_plain = seedqr_encode(phrase.as_str()).map_err(|e| map_seedqr_error(e, "encode"))?;
    let digits: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(digits_plain);
    let _pin_digits = mnemonic_toolkit::mlock::pin_pages_for(digits.as_bytes());

    let canonical_phrase: zeroize::Zeroizing<String> = zeroize::Zeroizing::new(
        phrase
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join(" "),
    );
    let word_count = canonical_phrase.split_whitespace().count();

    emit_encode_output(args, canonical_phrase.as_str(), digits.as_str(), word_count, stdout)
}

fn emit_decode_output<W: Write>(
    args: &SeedqrDecodeArgs,
    phrase: &str,
    digits: &str,
    word_count: usize,
    stdout: &mut W,
) -> Result<u8, ToolkitError> {
    if let Some(path) = &args.json_out {
        let envelope = SeedqrEnvelope {
            schema_version: "1",
            operation: "decode",
            variant: "standard",
            word_count,
            phrase,
            digits,
        };
        let json = serde_json::to_string_pretty(&envelope).map_err(|e| {
            ToolkitError::BadInput(format!("seedqr: decode: json serialize: {e}"))
        })?;
        std::fs::write(path, json).map_err(|e| {
            ToolkitError::BadInput(format!("seedqr: decode: json-out write to {path:?}: {e}"))
        })?;
    } else {
        writeln!(stdout, "{phrase}").map_err(|e| {
            ToolkitError::BadInput(format!("seedqr: decode: stdout write: {e}"))
        })?;
    }
    Ok(0)
}

fn emit_encode_output<W: Write>(
    args: &SeedqrEncodeArgs,
    phrase: &str,
    digits: &str,
    word_count: usize,
    stdout: &mut W,
) -> Result<u8, ToolkitError> {
    if let Some(path) = &args.json_out {
        let envelope = SeedqrEnvelope {
            schema_version: "1",
            operation: "encode",
            variant: "standard",
            word_count,
            phrase,
            digits,
        };
        let json = serde_json::to_string_pretty(&envelope).map_err(|e| {
            ToolkitError::BadInput(format!("seedqr: encode: json serialize: {e}"))
        })?;
        std::fs::write(path, json).map_err(|e| {
            ToolkitError::BadInput(format!("seedqr: encode: json-out write to {path:?}: {e}"))
        })?;
    } else {
        writeln!(stdout, "{digits}").map_err(|e| {
            ToolkitError::BadInput(format!("seedqr: encode: stdout write: {e}"))
        })?;
    }
    Ok(0)
}
```

- [ ] **Step 3: Wire `Command::Seedqr` into `main.rs`**

In `crates/mnemonic-toolkit/src/main.rs`, locate the `Command` enum around L58-88. Insert between the `SeedXor` and `Slip39` variants:

```rust
    /// encode/decode SeedQR (BIP-39 mnemonic ↔ numeric digit-string QR payload)
    Seedqr(cmd::seedqr::SeedqrArgs),
```

In the dispatch `match` block around L104-133, insert between the `SeedXor` and `Slip39` arms (NO `.map(|_| 0)` per R0 I5 fold — `run` already returns `Result<u8, ToolkitError>`):

```rust
        Command::Seedqr(args) => cmd::seedqr::run(args, stdin, stdout, stderr),
```

- [ ] **Step 4: Update `secrets.rs::flag_is_secret`**

In `crates/mnemonic-toolkit/src/secrets.rs:49-59`, add `"--digits"` to the match arm:

```rust
pub fn flag_is_secret(flag_name: &str) -> bool {
    matches!(
        flag_name,
        "--passphrase"
            | "--passphrase-stdin"
            | "--bip38-passphrase"
            | "--bip38-passphrase-stdin"
            | "--digits"
            | "--ms1"
            | "--share"
    )
}
```

Note alphabetical order in the new match arm placement: `--bip38-passphrase-stdin` < `--digits` < `--ms1`.

In the `#[cfg(test)] mod tests` block, add `"--digits"` to the `known_secret_flags_classify_as_secret` test list:

```rust
        for name in [
            "--passphrase",
            "--passphrase-stdin",
            "--bip38-passphrase",
            "--bip38-passphrase-stdin",
            "--digits",
            "--ms1",
            "--share",
        ] {
            assert!(flag_is_secret(name), "{name} must classify as secret");
        }
```

- [ ] **Step 5: Build to verify**

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
cargo build --package mnemonic-toolkit 2>&1 | tail -10
```

Expected: clean build.

- [ ] **Step 6: Smoke-test the CLI surface**

```bash
./target/debug/mnemonic seedqr --help
./target/debug/mnemonic seedqr decode --help
./target/debug/mnemonic seedqr encode --help

# 12-word decode
./target/debug/mnemonic seedqr decode --digits 000000000000000000000000000000000000000000000003

# 12-word encode
./target/debug/mnemonic seedqr encode --from phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"

# 24-word decode (note "art" at index 102 → "...0102")
./target/debug/mnemonic seedqr decode --digits 000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000102

# 24-word encode (output: ends in "0102")
./target/debug/mnemonic seedqr encode --from phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
```

Expected outputs:
- 12-word decode → `abandon abandon ... abandon about`
- 12-word encode → `000000000000000000000000000000000000000000000003`
- 24-word decode → `abandon ... abandon art`
- 24-word encode → `...0102` (96 chars)

Argv-leakage advisory on stderr for inline forms.

- [ ] **Step 7: Smoke-test JSON envelope**

```bash
./target/debug/mnemonic seedqr decode --digits 000000000000000000000000000000000000000000000003 --json-out /tmp/seedqr-decode.json && cat /tmp/seedqr-decode.json
```

Expected JSON:
```json
{
  "schema_version": "1",
  "operation": "decode",
  "variant": "standard",
  "word_count": 12,
  "phrase": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
  "digits": "000000000000000000000000000000000000000000000003"
}
```

- [ ] **Step 8: Smoke-test --from non-phrase rejection**

```bash
./target/debug/mnemonic seedqr encode --from xpub=xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz 2>&1 | tail -3
echo "Exit code: $?"
```

Expected: stderr `seedqr encode only accepts phrase=<value> or phrase=-` + exit code 1.

- [ ] **Step 9: Smoke-test secrets.rs gui-schema emission**

```bash
./target/debug/mnemonic gui-schema | jq '.subcommands[] | select(.name=="seedqr") | .flags[]? // empty'
./target/debug/mnemonic gui-schema | grep -E "digits.*secret|--digits"
```

Expected: `--digits` carries the `secret: true` annotation in the gui-schema JSON (per the `flag_is_secret` update).

- [ ] **Step 10: Run library + secrets test suites**

```bash
cargo test --package mnemonic-toolkit --lib seedqr 2>&1 | tail -5
cargo test --package mnemonic-toolkit --lib secrets 2>&1 | tail -5
```

Expected: PASS.

- [ ] **Step 11: Commit**

```bash
git add crates/mnemonic-toolkit/src/cmd/seedqr.rs crates/mnemonic-toolkit/src/cmd/mod.rs crates/mnemonic-toolkit/src/main.rs crates/mnemonic-toolkit/src/secrets.rs
git commit -m "feat(seedqr): Phase 2 — cmd/seedqr.rs CLI wiring + secret hygiene

Adds Command::Seedqr variant + decode/encode subsubcommands. Library-local
SeedqrError mapped to ToolkitError::BadInput via map_seedqr_error at the
CLI boundary. JSON envelope: schema_version=1, operation=decode|encode,
variant=standard, word_count, phrase, digits.

Secret-memory hygiene applied per cmd/seed_xor.rs:163-178 precedent:
Zeroizing<String> on phrase/digits buffers, mlock::pin_pages_for page
pins, secret_in_argv_warning advisories for inline-form input.
secrets.rs::flag_is_secret extended to include --digits.

Phase 2 of design/PLAN_mnemonic_toolkit_v0_30_0.md."
```

---

### Task 4: Phase 3 — CLI integration tests

**Files:**
- Create: `crates/mnemonic-toolkit/tests/cli_seedqr.rs`

Target: **≥30 cells** per R0 I6 fold + brainstorm-locked range 30-60.

- [ ] **Step 1: Author integration test file**

Create `crates/mnemonic-toolkit/tests/cli_seedqr.rs`:

```rust
//! CLI integration tests for `mnemonic seedqr` (v0.30.0). Target ≥30 cells.

use assert_cmd::Command;
use serde_json::Value;

const PHRASE_12: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const DIGITS_12: &str = "000000000000000000000000000000000000000000000003";

const PHRASE_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const DIGITS_24: &str = "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000102";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

// ──────────────────────────────────────────────────────────────────────
// Decode — happy paths
// ──────────────────────────────────────────────────────────────────────

#[test]
fn decode_12_word_text_mode() {
    mnemonic().args(["seedqr", "decode", "--digits", DIGITS_12])
        .assert().success().stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn decode_24_word_text_mode() {
    mnemonic().args(["seedqr", "decode", "--digits", DIGITS_24])
        .assert().success().stdout(format!("{PHRASE_24}\n"));
}

#[test]
fn decode_stdin_space_form() {
    mnemonic().args(["seedqr", "decode", "--digits", "-"])
        .write_stdin(DIGITS_12)
        .assert().success().stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn decode_stdin_equals_form() {
    mnemonic().args(["seedqr", "decode", "--digits=-"])
        .write_stdin(DIGITS_12)
        .assert().success().stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn decode_json_mode_12_word() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic().args(["seedqr", "decode", "--digits", DIGITS_12, "--json-out", path.to_str().unwrap()])
        .assert().success().stdout("");
    let json: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(json["schema_version"], "1");
    assert_eq!(json["operation"], "decode");
    assert_eq!(json["variant"], "standard");
    assert_eq!(json["word_count"], 12);
    assert_eq!(json["phrase"], PHRASE_12);
    assert_eq!(json["digits"], DIGITS_12);
}

#[test]
fn decode_json_mode_24_word() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic().args(["seedqr", "decode", "--digits", DIGITS_24, "--json-out", path.to_str().unwrap()])
        .assert().success().stdout("");
    let json: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(json["word_count"], 24);
    assert_eq!(json["phrase"], PHRASE_24);
    assert_eq!(json["digits"], DIGITS_24);
}

// ──────────────────────────────────────────────────────────────────────
// Decode — refusals
// ──────────────────────────────────────────────────────────────────────

#[test]
fn decode_rejects_length_47() {
    let bad = &DIGITS_12[..47];
    mnemonic().args(["seedqr", "decode", "--digits", bad])
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: decode: invalid digit count"));
}

#[test]
fn decode_rejects_length_49() {
    let bad = format!("{DIGITS_12}0");
    mnemonic().args(["seedqr", "decode", "--digits", &bad])
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: decode: invalid digit count"));
}

#[test]
fn decode_rejects_length_95() {
    let bad = &DIGITS_24[..95];
    mnemonic().args(["seedqr", "decode", "--digits", bad])
        .assert().failure().code(1);
}

#[test]
fn decode_rejects_length_97() {
    let bad = format!("{DIGITS_24}0");
    mnemonic().args(["seedqr", "decode", "--digits", &bad])
        .assert().failure().code(1);
}

#[test]
fn decode_rejects_non_digit_char() {
    let bad = "00000000000000000000000000000000000000000000000A";
    mnemonic().args(["seedqr", "decode", "--digits", bad])
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: decode: invalid character"));
}

#[test]
fn decode_rejects_word_index_out_of_range() {
    let bad = format!("9999{}", &DIGITS_12[4..]);
    mnemonic().args(["seedqr", "decode", "--digits", &bad])
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: decode: invalid word index"));
}

#[test]
fn decode_rejects_checksum_failure() {
    let bad = "000100010001000100010001000100010001000100010001";
    mnemonic().args(["seedqr", "decode", "--digits", bad])
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: decode: BIP-39 checksum failure"));
}

// ──────────────────────────────────────────────────────────────────────
// Encode — happy paths
// ──────────────────────────────────────────────────────────────────────

#[test]
fn encode_12_word_text_mode() {
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .assert().success().stdout(format!("{DIGITS_12}\n"));
}

#[test]
fn encode_24_word_text_mode() {
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_24}"))
        .assert().success().stdout(format!("{DIGITS_24}\n"));
}

#[test]
fn encode_stdin_space_form() {
    mnemonic().args(["seedqr", "encode", "--from", "phrase=-"])
        .write_stdin(PHRASE_12)
        .assert().success().stdout(format!("{DIGITS_12}\n"));
}

#[test]
fn encode_json_mode_12_word() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .args(["--json-out", path.to_str().unwrap()])
        .assert().success().stdout("");
    let json: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(json["schema_version"], "1");
    assert_eq!(json["operation"], "encode");
    assert_eq!(json["variant"], "standard");
    assert_eq!(json["word_count"], 12);
    assert_eq!(json["phrase"], PHRASE_12);
    assert_eq!(json["digits"], DIGITS_12);
}

#[test]
fn encode_json_mode_24_word() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_24}"))
        .args(["--json-out", path.to_str().unwrap()])
        .assert().success().stdout("");
    let json: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    assert_eq!(json["word_count"], 24);
    assert_eq!(json["phrase"], PHRASE_24);
    assert_eq!(json["digits"], DIGITS_24);
}

// ──────────────────────────────────────────────────────────────────────
// Encode — refusals
// ──────────────────────────────────────────────────────────────────────

#[test]
fn encode_rejects_non_phrase_node_xpub() {
    mnemonic().args(["seedqr", "encode", "--from", "xpub=xpub6CUGRUonZSQ4TWtTMmzXdrXDtypWKiKrhko4egpiMZbpiaQL2jkwSB1icqYh2cfDfVxdx4df189oLKnC5fSwqPfgyP3hooxujYzAu3fDVmz"])
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr encode only accepts phrase="));
}

#[test]
fn encode_rejects_13_word_count() {
    let bad = format!("{PHRASE_12} abandon");
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: encode: invalid word count"));
}

#[test]
fn encode_rejects_15_word_count() {
    let bad = "abandon ".repeat(14) + "about";
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert().failure().code(1);
}

#[test]
fn encode_rejects_18_word_count() {
    let bad = "abandon ".repeat(17) + "about";
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert().failure().code(1);
}

#[test]
fn encode_rejects_21_word_count() {
    let bad = "abandon ".repeat(20) + "about";
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert().failure().code(1);
}

#[test]
fn encode_rejects_25_word_count() {
    let bad = format!("{PHRASE_24} abandon");
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert().failure().code(1);
}

#[test]
fn encode_rejects_invalid_word() {
    let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon notaword";
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: encode: BIP-39 checksum failure"));
}

#[test]
fn encode_rejects_checksum_failure() {
    let bad = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={bad}"))
        .assert().failure().code(1)
        .stderr(predicates::str::contains("seedqr: encode: BIP-39 checksum failure"));
}

// ──────────────────────────────────────────────────────────────────────
// Round-trip
// ──────────────────────────────────────────────────────────────────────

#[test]
fn round_trip_12_word_text() {
    let encode_out = mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .assert().success();
    let digits = String::from_utf8(encode_out.get_output().stdout.clone()).unwrap();
    let digits = digits.trim_end();

    mnemonic().args(["seedqr", "decode", "--digits", digits])
        .assert().success().stdout(format!("{PHRASE_12}\n"));
}

#[test]
fn round_trip_24_word_text() {
    let encode_out = mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_24}"))
        .assert().success();
    let digits = String::from_utf8(encode_out.get_output().stdout.clone()).unwrap();
    let digits = digits.trim_end();

    mnemonic().args(["seedqr", "decode", "--digits", digits])
        .assert().success().stdout(format!("{PHRASE_24}\n"));
}

#[test]
fn round_trip_12_word_through_json_envelope() {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    let path = tmp.path();
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .args(["--json-out", path.to_str().unwrap()])
        .assert().success();
    let json: Value = serde_json::from_reader(std::fs::File::open(path).unwrap()).unwrap();
    let digits = json["digits"].as_str().unwrap();

    mnemonic().args(["seedqr", "decode", "--digits", digits])
        .assert().success().stdout(format!("{PHRASE_12}\n"));
}

// ──────────────────────────────────────────────────────────────────────
// Argv-leakage advisory
// ──────────────────────────────────────────────────────────────────────

#[test]
fn decode_emits_argv_advisory_on_inline_form() {
    // Assert on the load-bearing template substring per
    // secret_advisory.rs:36-38: `"warning: secret material on argv (...)"`.
    // Asserting on `--digits` alone would be too loose; on `supplied in argv`
    // would be vacuous (substring doesn't appear in the template).
    mnemonic().args(["seedqr", "decode", "--digits", DIGITS_12])
        .assert().success()
        .stderr(predicates::str::contains("secret material on argv"))
        .stderr(predicates::str::contains("--digits"));
}

#[test]
fn encode_emits_argv_advisory_on_inline_form() {
    mnemonic().args(["seedqr", "encode", "--from"])
        .arg(format!("phrase={PHRASE_12}"))
        .assert().success()
        .stderr(predicates::str::contains("secret material on argv"))
        .stderr(predicates::str::contains("--from phrase="));
}

#[test]
fn decode_no_argv_advisory_on_stdin_form() {
    // Negate on the load-bearing template substring (per
    // secret_advisory.rs:36-38); negating on `supplied in argv` would be
    // vacuous since that substring is never emitted by any code path.
    let stderr = mnemonic().args(["seedqr", "decode", "--digits", "-"])
        .write_stdin(DIGITS_12)
        .assert().success();
    let stderr_bytes = stderr.get_output().stderr.clone();
    let stderr_str = String::from_utf8(stderr_bytes).unwrap();
    assert!(
        !stderr_str.contains("secret material on argv"),
        "stdin form must not emit argv-leakage advisory; got stderr: {stderr_str}"
    );
}
```

Cell count: 31 cells (covers ≥30 target per R0 I6 fold).

- [ ] **Step 2: Verify dev-deps**

```bash
grep -A3 "\[dev-dependencies\]" /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/Cargo.toml | head -10
```

Expected: `assert_cmd`, `predicates`, `tempfile` all already present (used by other `cli_*.rs` test files). If any missing, add.

- [ ] **Step 3: Run integration tests**

```bash
cargo test --package mnemonic-toolkit --test cli_seedqr 2>&1 | tail -25
```

Expected: all 31 cells PASS.

- [ ] **Step 4: Run full toolkit suite for regressions**

```bash
cargo test --package mnemonic-toolkit 2>&1 | tail -10
```

Expected: no regressions; new seedqr cells included.

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/tests/cli_seedqr.rs
git commit -m "test(seedqr): Phase 3 — CLI integration suite (31 cells)

Covers decode + encode in text mode + JSON-out mode + stdin (both
--digits=- and --digits - forms) + 6 refusal classes (decode wrong-length,
non-digit, word-index OOR, checksum; encode non-phrase-node-rejection,
4 wrong-word-counts, invalid-word, checksum) + round-trips (text +
through-JSON-envelope) + argv-leakage advisory presence/absence.

Phase 3 of design/PLAN_mnemonic_toolkit_v0_30_0.md."
```

---

### Task 5: Phase 4 — Manual chapter (chapter-41 add + chapter-45 update)

**Files:**
- Modify: `docs/manual/src/40-cli-reference/41-mnemonic.md`
- Modify: `docs/manual/src/45-foreign-formats.md`

**Phase 4 prelude:** ensure Task 1 §A4 is complete (SeedSigner Python ref symbol path locked in `design/cycle-5-p0-recon.md`). If not, run Task 1 first.

- [ ] **Step 1: Add `## \`mnemonic seedqr\`` section to chapter-41**

Insert immediately BEFORE the `## \`mnemonic gui-schema\`` heading at L1587 in `docs/manual/src/40-cli-reference/41-mnemonic.md`:

```markdown
## `mnemonic seedqr`

SeedQR is an open spec originated by [SeedSigner](https://seedsigner.com/seedqr-instructions/):
a BIP-39 mnemonic encoded as a numeric-string QR payload where each
English-wordlist index is rendered as a 4-digit zero-padded decimal.
12-word phrases produce 48 digits; 24-word phrases produce 96.

`mnemonic seedqr` has two subsubcommands:
- `decode` — read a SeedQR numeric string, emit the BIP-39 phrase.
- `encode` — read a BIP-39 phrase, emit the SeedQR numeric string.

### Synopsis

```
mnemonic seedqr decode --digits <VALUE|-> [--json-out <PATH>]
mnemonic seedqr encode --from phrase=<VALUE|-> [--json-out <PATH>]
```

### Flags

`decode`:
- `--digits <VALUE|->`: SeedQR numeric digit string (48 or 96 ASCII digits). `-` reads from stdin.
- `--json-out <PATH>`: emit a JSON envelope at PATH instead of plain text on stdout.

`encode`:
- `--from phrase=<VALUE|->`: BIP-39 phrase (12 or 24 English words). `phrase=-` reads from stdin. The toolkit refuses non-phrase node types (`xpub=`, `ms1=`, etc.).
- `--json-out <PATH>`: emit a JSON envelope at PATH instead of plain text on stdout.

Both subsubcommands emit an argv-leakage advisory on stderr when the
secret is supplied inline (e.g., `--digits <value>` or `--from phrase=<value>`).
Use the stdin form (`-`) to avoid the advisory.

### Scope (v0.30.0)

- **Variants:** Standard SeedQR only. CompactSeedQR (raw entropy bytes encoded in QR binary mode) is deferred.
- **Word counts:** 12 + 24 only. 15 / 18 / 21 deferred.
- **Language:** English only. SeedQR's open spec defines the encoding against the BIP-39 English wordlist.

### Worked example — decode

```
$ mnemonic seedqr decode --digits 000000000000000000000000000000000000000000000003
abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
```

JSON envelope form:

```
$ mnemonic seedqr decode --digits 000000000000000000000000000000000000000000000003 --json-out /tmp/decode.json
$ cat /tmp/decode.json
{
  "schema_version": "1",
  "operation": "decode",
  "variant": "standard",
  "word_count": 12,
  "phrase": "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
  "digits": "000000000000000000000000000000000000000000000003"
}
```

### Worked example — encode

```
$ mnemonic seedqr encode --from phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
000000000000000000000000000000000000000000000003
```

Pipe to a QR generator:

```
$ mnemonic seedqr encode --from phrase="abandon ... about" | qrencode -o out.png -
```

### Cross-impl smoke recipe

Verify byte-identical output against the SeedSigner Python reference at
`<CITE_SYMBOL_PATH_FROM_§A4>` (recipe finalized at commit time using the symbol
path recorded in `design/cycle-5-p0-recon.md` §A4).

### Exit codes

- `0` — success.
- `1` — `BadInput` (any `SeedqrError` variant: invalid digit count/character, word index out of range, wrong word count, BIP-39 checksum failure; OR non-phrase node passed to `encode --from`).

### Stderr templates

- `seedqr: decode: invalid digit count (expected 48 or 96; got N)`
- `seedqr: decode: invalid character at position N: <char>`
- `seedqr: decode: invalid word index N at position M (must be 0..=2047)`
- `seedqr: decode: BIP-39 checksum failure: <bip39-crate-diagnostic>`
- `seedqr: encode: invalid word count: N (only 12 or 24 supported)`
- `seedqr: encode: BIP-39 checksum failure: <bip39-crate-diagnostic>`
- `seedqr encode only accepts phrase=<value> or phrase=-`
```

Substitute `<CITE_SYMBOL_PATH_FROM_§A4>` with the verified symbol path from Task 1.

- [ ] **Step 2: Rewrite chapter-45 §Deferral — SeedQR (L620-626)**

```bash
sed -n '618,630p' /scratch/code/shibboleth/mnemonic-toolkit/docs/manual/src/45-foreign-formats.md
```

Replace L620-626 with:

```markdown
### SeedQR (Jade + SeedSigner + others)

SeedQR is an open spec originated by SeedSigner; Blockstream Jade and
several other wallets (Coldcard, Cobo, Krux) adopted it. Because SeedQR
encodes a BIP-39 seed (not a wallet policy), it does NOT round-trip
through `mnemonic import-wallet` — instead, decode the SeedQR payload
to a phrase via `mnemonic seedqr decode`, then feed the phrase into
`mnemonic bundle` or any other downstream subcommand.

See [`mnemonic seedqr`](40-cli-reference/41-mnemonic.md#mnemonic-seedqr)
for the encode/decode subsurface (v0.30.0+).
```

(Keep L607-608 `jade_specific_fields` reservation sentence as-is per brainstorm R0 I1 fold note.)

- [ ] **Step 3: Update chapter-45 "What's NOT supported" bullet at L786**

Replace:
```markdown
- **Jade SeedQR variant** (`wallet-import-jade-seedqr`) — see
```
With:
```markdown
- ~~**Jade SeedQR variant**~~ — shipped in v0.30.0 as a vendor-neutral subsurface. See [`mnemonic seedqr`](40-cli-reference/41-mnemonic.md#mnemonic-seedqr).
```

If mdbook fails to render `~~text~~`, use `<del>text</del>`.

- [ ] **Step 4: Run manual lint**

```bash
make -C /scratch/code/shibboleth/mnemonic-toolkit/docs/manual lint MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic 2>&1 | tail -10
```

Set MD_BIN / MS_BIN / MK_BIN if Makefile requires; check Makefile for the expected env var set.

Expected: lint PASS.

- [ ] **Step 5: Run the prose's commands locally per architect-must-run-prose-commands**

Execute each command block in the new `## mnemonic seedqr` section against the built binary. Confirm output matches the manual prose byte-for-byte.

```bash
./target/debug/mnemonic seedqr decode --digits 000000000000000000000000000000000000000000000003
./target/debug/mnemonic seedqr decode --digits 000000000000000000000000000000000000000000000003 --json-out /tmp/decode.json && cat /tmp/decode.json
./target/debug/mnemonic seedqr encode --from phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
```

If any output differs, fix the prose (not the implementation) BEFORE commit.

- [ ] **Step 6: Commit**

```bash
git add docs/manual/src/40-cli-reference/41-mnemonic.md docs/manual/src/45-foreign-formats.md
git commit -m "docs(seedqr): Phase 4 — manual chapter-41 + chapter-45 update

New section ## mnemonic seedqr in chapter-41 covering synopsis, flags,
worked examples (decode + encode + JSON envelope), cross-impl smoke
recipe vs SeedSigner Python ref (per §A4 recon), exit codes (1, NOT 2),
stderr templates.

Chapter-45 §Deferral — SeedQR rewritten to redirect users at
mnemonic seedqr; chapter-45 bullet updated with strike-through.

Phase 4 of design/PLAN_mnemonic_toolkit_v0_30_0.md."
```

---

### Task 6: Phase 5 — Toolkit cycle close (version bump + CHANGELOG + tag + push)

**Files:**
- Modify: `crates/mnemonic-toolkit/Cargo.toml:3`
- Modify: `CHANGELOG.md`
- Modify: `scripts/install.sh:32`

- [ ] **Step 1: Bump version**

`crates/mnemonic-toolkit/Cargo.toml:3`:
- OLD: `version = "0.29.0"`
- NEW: `version = "0.30.0"`

- [ ] **Step 2: Bump install.sh self-pin**

`scripts/install.sh:32`:
- OLD: `mnemonic-toolkit|...|mnemonic-toolkit-v0.29.0|no|`
- NEW: `mnemonic-toolkit|...|mnemonic-toolkit-v0.30.0|no|`

- [ ] **Step 3: Add CHANGELOG entry**

In `CHANGELOG.md`, add above the v0.29.0 section:

```markdown
## [0.30.0] — 2026-MM-DD

**SemVer-MINOR** — new top-level `mnemonic seedqr` subcommand.

### Added

- **`mnemonic seedqr decode|encode`** — SeedQR encode/decode top-level subcommand.
  - `seedqr decode --digits <VALUE|->` reads a 48 or 96 ASCII-digit SeedQR string, validates BIP-39 checksum, emits the BIP-39 phrase.
  - `seedqr encode --from phrase=<VALUE|->` reads a 12- or 24-word English BIP-39 phrase, emits the SeedQR numeric string.
  - Both subsubcommands support `--json-out <PATH>` (envelope: `schema_version: "1"`, `operation: "decode"|"encode"`, `variant: "standard"`, `word_count`, `phrase`, `digits`).
  - Standard variant only; English-locked. CompactSeedQR + 15/18/21-word counts + bundle-slot integration filed as FOLLOWUPs.
- New library module `mnemonic_toolkit::seedqr` with `decode()` / `encode()` primitives + library-local `SeedqrError` enum (no new `ToolkitError` variants; mapped via `cmd::seedqr::map_seedqr_error` at CLI boundary per `lib.rs:14-28` documented pattern).
- `secrets.rs::flag_is_secret` extended to include `"--digits"`.

### Documentation

- New `## mnemonic seedqr` section in manual chapter-41.
- Chapter-45 `### Deferral — SeedQR` rewritten to redirect to the new subcommand.

### FOLLOWUP closure

- **Closed (resolved-superseded):** `wallet-import-jade-seedqr` (superseded by new vendor-neutral slug `seedqr-encode-decode-subcommand`).

### Newly filed FOLLOWUPs

- `seedqr-compact-variant`, `seedqr-15-18-21-word-counts`, `seedqr-bundle-slot-integration`, `seedqr-digits-from-input-unification`.
```

Replace `2026-MM-DD` with the actual ship date.

- [ ] **Step 4: Full audit before commit**

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit && make audit 2>&1 | tail -20
```

If `make audit` doesn't exist: `cargo test --workspace && cargo clippy --all-targets --workspace -- -D warnings && cargo fmt --check`.

Expected: GREEN.

- [ ] **Step 5: Commit + tag + push**

```bash
git add crates/mnemonic-toolkit/Cargo.toml scripts/install.sh CHANGELOG.md Cargo.lock
git commit -m "release(toolkit): mnemonic-toolkit v0.30.0 — SeedQR encode/decode subcommand

New top-level mnemonic seedqr subcommand (SemVer-MINOR per v0.11/v0.12/
v0.13/v0.22 new-top-level-subcommand precedent). Closes 1 v0.28+
FOLLOWUP (wallet-import-jade-seedqr, resolved-superseded by new slug
seedqr-encode-decode-subcommand).

Architectural pivot from predecessor brainstorm's wallet-import framing:
SeedQR carries a BIP-39 seed (not a wallet policy), so top-level
seedqr decode/encode is the right surface (paralleling seed-xor/slip39/
final-word). Library-local SeedqrError enum mapped at CLI boundary via
ToolkitError::BadInput (exit code 1) per lib.rs:14-28 documented pattern
(no ToolkitError variants; no thiserror dep). Secret-memory hygiene
applied via Zeroizing + mlock + secret_in_argv_warning. secrets.rs
flag_is_secret extended to classify --digits.

See design/PLAN_mnemonic_toolkit_v0_30_0.md for per-phase detail."

git tag mnemonic-toolkit-v0.30.0
git push origin master
git push origin mnemonic-toolkit-v0.30.0
```

- [ ] **Step 6: Verify install-pin-check CI**

```bash
gh run list --limit 5 --json status,conclusion,name,headBranch | jq '.[] | select(.name|test("install-pin"))'
```

Wait for `conclusion: success`.

- [ ] **Step 7: Create GH Release**

```bash
gh release create mnemonic-toolkit-v0.30.0 \
  --title "mnemonic-toolkit-v0.30.0 — SeedQR encode/decode subcommand" \
  --notes "$(awk '/^## \[0\.30\.0\]/,/^## \[0\.29\.0\]/' CHANGELOG.md | head -n -1)"
```

---

### Task 7: Phase 6 — GUI lockstep

**Files:**
- Modify: `mnemonic-gui/pinned-upstream.toml`
- Modify: `mnemonic-gui/Cargo.toml`
- Modify: `mnemonic-gui/src/schema/mnemonic.rs`
- Modify: `mnemonic-gui/CHANGELOG.md`

- [ ] **Step 1: Pin bump**

```bash
cd /scratch/code/shibboleth/mnemonic-gui && git pull --ff-only origin master
```

`pinned-upstream.toml` `[mnemonic]` tag: `mnemonic-toolkit-v0.29.0` → `mnemonic-toolkit-v0.30.0`.

`Cargo.toml` workspace dep tag: `mnemonic-toolkit-v0.29.0` → `mnemonic-toolkit-v0.30.0`. Also bump workspace version `0.14.0` → `0.15.0`.

- [ ] **Step 2: cargo update**

```bash
cd /scratch/code/shibboleth/mnemonic-gui && cargo update --workspace 2>&1 | tail -10
```

Expected: `mnemonic-toolkit` resolves to v0.30.0.

- [ ] **Step 3: Capture gui-schema output for new entries**

```bash
/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic gui-schema | jq '[.subcommands[] | select(.name | startswith("seedqr"))]'
```

Save the exact JSON shape — Step 4 mirrors it into the schema struct.

- [ ] **Step 4: Add `SubcommandSchema` entries**

In `mnemonic-gui/src/schema/mnemonic.rs`, the `SUBCOMMANDS` const at L2310-2478 currently has the ordering:
- L2359: `seed-xor-split`
- L2367: `seed-xor-combine`
- L2375: `slip39-split`
- L2383: `slip39-combine`

Insert TWO new entries between `seed-xor-combine` (L2367) and `slip39-split` (L2375). Per the existing verb-ordering convention (split/create-side first, combine/recover-side second), order them: `seedqr-encode` first (create-side), `seedqr-decode` second (recover-side):

```rust
    SubcommandSchema {
        name: "seedqr-encode",
        // [mirror structural pattern of seed-xor-split with the
        // exact field shape captured from Step 3 gui-schema JSON]
    },
    SubcommandSchema {
        name: "seedqr-decode",
        // [mirror structural pattern of seed-xor-combine with the
        // exact field shape from Step 3]
    },
```

Re-read existing `SubcommandSchema` usages (L1018, L1106, L1174, L1232 are the seed-xor / slip39 leaves) to confirm the struct's current field shape before authoring.

- [ ] **Step 5: Run schema_mirror locally with explicit MNEMONIC_BIN**

```bash
cd /scratch/code/shibboleth/mnemonic-gui
MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic cargo test --test schema_mirror 2>&1 | tail -25
```

Expected: PASS. If FAIL, diff toolkit's `gui-schema` JSON against the schema-mirror entry to identify field-name parity issues.

(MUST use explicit MNEMONIC_BIN per Cycle 4 feedback — `$PATH` resolves stale binaries.)

- [ ] **Step 6: Update GUI CHANGELOG**

In `mnemonic-gui/CHANGELOG.md`, add at the top:

```markdown
## [0.15.0] — 2026-MM-DD

**SemVer-MINOR** — toolkit pin bump to v0.30.0 (new `mnemonic seedqr` subcommand).

### Added

- New `SubcommandSchema` entries for `seedqr-encode` + `seedqr-decode` mirroring the toolkit's new top-level `mnemonic seedqr` subcommand. Placed between `seed-xor-combine` and `slip39-split` (create-side `encode` before recover-side `decode` per seed-xor / slip39 verb-ordering precedent).

### Changed

- Toolkit pin: `mnemonic-toolkit-v0.29.0` → `mnemonic-toolkit-v0.30.0`.
```

- [ ] **Step 7: Run full GUI suite**

```bash
cd /scratch/code/shibboleth/mnemonic-gui
MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic cargo test --workspace 2>&1 | tail -10
```

Expected: PASS.

- [ ] **Step 8: Commit + tag + push GUI**

```bash
cd /scratch/code/shibboleth/mnemonic-gui

git add pinned-upstream.toml Cargo.toml Cargo.lock src/schema/mnemonic.rs CHANGELOG.md
git commit -m "release(gui): mnemonic-gui v0.15.0 — toolkit v0.30.0 pin (mnemonic seedqr)

Schema-mirror lockstep with mnemonic-toolkit-v0.30.0: adds
SubcommandSchema entries for seedqr-encode + seedqr-decode reflecting
the new mnemonic seedqr top-level subcommand. Placed between
seed-xor-combine and slip39-split per alphabetical + verb-ordering
precedent (encode/create-side before decode/recover-side).

Toolkit pin: v0.29.0 → v0.30.0."

git tag mnemonic-gui-v0.15.0
git push origin master
git push origin mnemonic-gui-v0.15.0
```

- [ ] **Step 9: Verify GUI CI schema_mirror gate**

```bash
gh run list --limit 5 --repo bg002h/mnemonic-gui --json status,conclusion,name,headBranch | jq '.[] | select(.name=="schema_mirror" or .name=="CI")'
```

Expected: `conclusion: success`.

- [ ] **Step 10: GH Release**

```bash
cd /scratch/code/shibboleth/mnemonic-gui
gh release create mnemonic-gui-v0.15.0 \
  --title "mnemonic-gui-v0.15.0 — toolkit v0.30.0 pin (mnemonic seedqr)" \
  --notes "$(awk '/^## \[0\.15\.0\]/,/^## \[0\.14\.0\]/' CHANGELOG.md | head -n -1)"
```

---

### Task 8: Phase 7 — End-of-cycle opus review + FOLLOWUP closure

**Files:**
- Create: `design/agent-reports/v0_30_0-end-of-cycle-review.md`
- Modify: `design/FOLLOWUPS.md`

- [ ] **Step 1: Dispatch opus end-of-cycle review**

Dispatch `feature-dev:code-reviewer` (opus) on the full working tree across BOTH repos. Persist verbatim to `design/agent-reports/v0_30_0-end-of-cycle-review.md`.

The reviewer's prompt:
```
Read CLAUDE.md, design/BRAINSTORM_v0_30_0_seedqr.md, design/PLAN_mnemonic_toolkit_v0_30_0.md, and all commits since 4d82a3c (toolkit) + initial GUI HEAD (8f9e83b). Verify:
- All 8 phase deliverables landed.
- No new ToolkitError variants.
- SemVer MINOR class preserved (toolkit v0.30.0; GUI v0.15.0).
- JSON envelope shape matches brainstorm lock.
- schema_mirror test green against new pin.
- Secret-memory hygiene (Zeroizing/mlock/argv-warning) actually present.
- secrets.rs::flag_is_secret includes --digits.
- FOLLOWUP closures + new filings landed.
Surface C/I/M with file:line citations.
```

- [ ] **Step 2: Fold C/I findings**

Apply inline. Re-dispatch sonnet R1 if non-trivial.

- [ ] **Step 3: Close `wallet-import-jade-seedqr` FOLLOWUP**

In `design/FOLLOWUPS.md:2559`, update:
- `**Status:** open` → `**Status:** resolved (superseded by `seedqr-encode-decode-subcommand` per Cycle 5; v0.30.0)`
- Add `**Resolved by:** Cycle 5 toolkit commit <SHA> + GUI commit <SHA>`.

- [ ] **Step 4: File 4 new FOLLOWUPs**

Append to `design/FOLLOWUPS.md` (use the existing entry template — `Surfaced/Where/What/Why deferred/Status/Tier/Tags/Companion`):

1. `seedqr-compact-variant` — CompactSeedQR ingest (raw entropy; 16/32 bytes; needs explicit `--variant compact --word-count` flag). Tier v0.30+.
2. `seedqr-15-18-21-word-counts` — 15/18/21-word phrases (60/72/84 digits). Tier v0.30+.
3. `seedqr-bundle-slot-integration` — `mnemonic bundle --slot @N.seedqr=<file>` auto-decode at slot-emit. Tier v0.30+.
4. `seedqr-digits-from-input-unification` — long-term surface unification: extend `FromInput` with `seedqr=<value>` node type; deprecate `--digits`. Tier v0.30+.

- [ ] **Step 5: Commit + push**

```bash
git add design/FOLLOWUPS.md design/agent-reports/v0_30_0-end-of-cycle-review.md
git commit -m "design(cycle-5-close): FOLLOWUP closure + end-of-cycle opus review

Closes wallet-import-jade-seedqr (resolved-superseded by Cycle 5 /
v0.30.0). Files 4 new FOLLOWUPs (compact-variant /
15-18-21-word-counts / bundle-slot-integration /
digits-from-input-unification).

Persists opus end-of-cycle review verbatim."

git push origin master
```

- [ ] **Step 6: Update memory**

Add `project_v0_30_0_cycle_shipped` memory entry summarizing outcome + feedback surfaced during execution. (Memory updates happen in conversation context.)

---

## Cross-phase invariants

- **Per-phase TDD:** Phases 1 + 3 author tests before implementation.
- **Per-phase reviewer-loop:** sonnet for trivial; opus for non-trivial.
- **No clippy bypass:** `cargo clippy --all-targets --workspace -- -D warnings` MUST pass before each commit. `cargo fmt --check` MUST pass.
- **Worktree-isolation invariant:** subagent prompts MUST include `pwd && git rev-parse --show-toplevel` before any write (per `feedback_no_parallelism_for_code_generation`).
- **Bisect-hygiene:** each phase commit independently buildable + testable.
- **Cross-repo lockstep ordering:** toolkit tag MUST land before GUI pin bump (toolkit Phase 5 → GUI Phase 6).
- **Stale `$PATH` binary gotcha:** GUI `schema_mirror` runs MUST use explicit `MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic`.

## Phase ordering rationale

- Task 1 (A4 recon) BEFORE Task 5 (Phase 4 manual): manual cross-impl recipe needs A4's symbol path.
- Tasks 2 → 3 → 4 strictly sequential (library → CLI → integration tests).
- Task 5 (manual) needs Phase 2's binary built (for manual lint via `MNEMONIC_BIN`).
- Task 6 (toolkit cycle close) MUST land before Task 7 (GUI lockstep) — GUI cannot resolve a not-yet-existent toolkit tag.
- Task 8 (end-of-cycle review) lands LAST so reviewer sees complete cross-repo working tree.

## Risk register

- **`bip39::Mnemonic::parse_in` invalid-word vs checksum errors:** library design collapses both into `SeedqrError::ChecksumFailure`. Brainstorm acknowledges.
- **Manual lint extra env vars:** Phase 4 Step 4 reads Makefile if lint errors. `MD_BIN`/`MS_BIN`/`MK_BIN` may be required.
- **GUI `SubcommandSchema` struct shape evolution:** Phase 6 Step 4 reads existing usages before authoring. If shape evolved since Cycle 4, brief re-recon.
- **Strike-through markdown rendering:** Phase 4 Step 3 may need `<del>` if mdbook drops `~~`.
- **`thiserror` dep escape:** if any executing agent attempts to add `thiserror` as a workaround for the hand-rolled `impl Display`, REJECT — the lib-local-error precedent is intentional (`lib.rs:14-28`) and the brainstorm + R0 fold both explicitly forbid the dep addition.

---

## Self-review (pre-R1 dispatch)

- ✓ All brainstorm decisions covered.
- ✓ All R0 plan-doc findings (4C / 6I) folded:
  - C1: hand-rolled `impl Display` instead of `thiserror`.
  - C2: exit code 1 (not 2) in manual + CHANGELOG + test assertions.
  - C3: `value_parser = parse_from_input` + import added.
  - C4: `DIGITS_24` = `...0102` (not 0099).
  - I1: `NodeType::Phrase` validation in `run_encode`.
  - I2: `Zeroizing` + `mlock::pin_pages_for` + `secret_in_argv_warning` throughout.
  - I3: `secrets.rs::flag_is_secret` extended + test.
  - I4: GUI placement between `seed-xor-combine` (L2367) and `slip39-split` (L2375); `encode` before `decode` per verb-ordering.
  - I5: dispatch arm `cmd::seedqr::run(...)` directly (no `.map(|_| 0)`).
  - I6: 31 cells in `tests/cli_seedqr.rs` (≥30 target).
- ✓ No "TBD" / "implement later" placeholders.
- ✓ Function signatures consistent (`SeedqrError`, `map_seedqr_error`, `decode`, `encode`, `SeedqrEnvelope`, `SeedqrArgs/SeedqrAction/SeedqrDecodeArgs/SeedqrEncodeArgs`).
- ✓ File paths consistent.
- ✓ Exit codes consistent (1; from `error.rs:429` mapping).
- ✓ Cross-impl smoke recipe placeholder `<CITE_SYMBOL_PATH_FROM_§A4>` resolved via Task 1.
