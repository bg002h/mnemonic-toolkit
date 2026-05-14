# SPEC — `mnemonic seed-xor` (v0.12.0)

**Status:** Phase 0 — SPEC author + R0 reviewer-loop.
**Cycle:** v0.12.0 (toolkit-only minor bump).
**Predecessor:** v0.11.0 final-word completer shipped at `mnemonic-toolkit-v0.11.0` (tag at `f6c036a`, 2026-05-14).
**Driving FOLLOWUP:** `seed-xor-coldcard-compat` (filed at this P0; closes at PE).
**Brainstorm + plan:** consolidated in plan-mode artifact `~/.claude/plans/radiant-seeking-teacup.md` (BRAINSTORM + PLAN sections); this document is the standalone SPEC §A rendered from that plan.

External reference:
- [Coldcard `docs/seed-xor.md`](https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md)
- [Coldcard `shared/xor_seed.py`](https://github.com/Coldcard/firmware/blob/master/shared/xor_seed.py) (reference implementation)

---

## §1 Purpose

Add a Coldcard-compatible BIP-39 ↔ BIP-39 all-or-nothing XOR-based seed splitter as the `mnemonic seed-xor` subcommand (with `split` + `combine` sub-subcommands).

**Use cases:**
1. Paper-backup recovery with split-location storage (any single share is statistically independent of the master and looks like a valid BIP-39 phrase on its own — plausible-deniability concession Coldcard documents informally).
2. Manual seed-generation hardening: combine N separately-generated BIP-39 phrases to derive a master that no single party controls.
3. Coldcard interop: import/export shares that round-trip a Coldcard hardware wallet (12/18/24-word sizes).

**Toolkit-only minor bump.** No cross-repo work; no sibling-codec coordination.

## §2 Functional surface

### §2.1 Library entry point

New module `crates/mnemonic-toolkit/src/seed_xor.rs`. Library-local types (`SeedXorError`) per the v0.11.0 final-word precedent (avoids pulling in binary-private `ToolkitError` / `CliLanguage`; tracked under existing FOLLOWUP `library-error-and-language-surface-promotion`).

```rust
pub fn seed_xor_split(
    entropy: &[u8],
    n_shares: usize,
    rng: &mut (impl rand_core::CryptoRng + rand_core::RngCore),
) -> Result<Vec<zeroize::Zeroizing<Vec<u8>>>, SeedXorError>;

/// Deterministic variant matching Coldcard's `shared/xor_seed.py` —
/// SHA256d over master + "Batshitoshi" + per-share index per the
/// reference implementation. Algorithm: see Coldcard upstream.
pub fn seed_xor_split_deterministic(
    entropy: &[u8],
    n_shares: usize,
) -> Result<Vec<zeroize::Zeroizing<Vec<u8>>>, SeedXorError>;

pub fn seed_xor_combine(
    shares: &[&[u8]],
) -> Result<zeroize::Zeroizing<Vec<u8>>, SeedXorError>;

#[derive(Debug)]
pub enum SeedXorError {
    BadEntropyLength { got: usize, expected_one_of: &'static [usize] },
    TooFewShares { got: usize, min: usize },                // min = 2
    MismatchedShareLengths { lengths: Vec<usize> },
}
```

**Algorithm.**
- `split`: for each share except the last, populate `share[i] = RNG_bytes(entropy.len())` (or the Coldcard SHA256d-deterministic equivalent for `_deterministic`). The last share is `master XOR share[0] XOR ... XOR share[N-2]`. (Equivalent statement: pick N-1 random masks; the Nth share carries `master` XOR'd with all masks. Any single share is statistically independent of the master.) **Per-share BIP-39 checksum recomputation is the CLI-layer's responsibility, NOT the lib's** — the library emits raw entropy bytes.
- `combine`: bytewise XOR all share entropies. Returns `Zeroizing<Vec<u8>>`. Length validation ensures all inputs match.

**Accepted entropy lengths:** `{16, 20, 24, 28, 32}` bytes = 12/15/18/21/24-word BIP-39 phrases.
- Coldcard interop: 16/24/32 bytes = 12/18/24-word phrases (verified against `xor_seed.py:assert len(raw_secret) in (16, 24, 32)`).
- Toolkit-only extensions: 20/28 bytes = 15/21-word phrases (Coldcard hardware cannot round-trip those).

### §2.2 CLI subcommand grammar

New file `crates/mnemonic-toolkit/src/cmd/seed_xor.rs`. Two sub-subcommands via clap nested enum:

```rust
#[derive(clap::Subcommand)]
pub enum SeedXorCommand {
    /// Split a BIP-39 phrase into N XOR shares.
    Split(SeedXorSplitArgs),
    /// Combine N XOR shares back into a BIP-39 phrase.
    Combine(SeedXorCombineArgs),
}
```

#### `mnemonic seed-xor split`

| Flag | Required | Default | Purpose |
|---|---|---|---|
| `--from <phrase=<v-or->>` | yes | — | Master phrase (inline value or `phrase=-` stdin) |
| `--shares <N>` | yes | — | Number of shares to emit; `N >= 2` |
| `--language <LANG>` | no | `english` | BIP-39 language of input + output (10 BIP-39 languages) |
| `--deterministic-from-master` | no | false | Use SHA256d-based deterministic generation (Coldcard interop); default is OS CSPRNG |
| `--json-out <PATH>` | no | — | Side-effect: write JSON envelope to PATH |

**Stdout:** `N` lines, each a BIP-39 phrase of the same length as the input, in the same language. Trailing newline.

#### `mnemonic seed-xor combine`

| Flag | Required | Default | Purpose |
|---|---|---|---|
| `--share <phrase=<v-or->>` | yes (repeating; `ArgAction::Append`) | — | Share phrase; at most ONE may be `phrase=-` (stdin per single-stdin-per-invocation rule) |
| `--shares <N>` | yes | — | Asserted share count; MUST equal the number of `--share` flags |
| `--language <LANG>` | no | `english` | BIP-39 language of input + output |
| `--json-out <PATH>` | no | — | Side-effect JSON envelope |

**Stdout:** one line, the reconstructed BIP-39 phrase with valid BIP-39 checksum. Trailing newline.

### §2.3 JSON envelope schema

Schema `v1`. Single struct discriminated by `operation`:

```json
{
  "schema_version": "1",
  "operation": "split",
  "language": "english",
  "word_count": 12,
  "share_count": 3,
  "deterministic": false,
  "shares": ["phrase-1 ...", "phrase-2 ...", "phrase-3 ..."]
}
```

```json
{
  "schema_version": "1",
  "operation": "combine",
  "language": "english",
  "word_count": 12,
  "share_count": 3,
  "phrase": "reconstructed phrase ..."
}
```

Field order is part of the schema (SHA-pinned via `tests/cli_seed_xor_json.rs` at GREEN time).

### §2.4 Exit codes

- `0` on success (operation completed; output emitted).
- `1` for runtime refusals (`ToolkitError::BadInput` / `SeedXor` / `Bip39` per `src/error.rs:244` precedent).
- `64` reserved for clap parse errors.

### §2.5 Refusals (9 classes)

| # | Input class | Exit | Stderr message stem |
|---|---|---|---|
| 1 | `split --from` phrase word-count not in {12,15,18,21,24} | 1 | `seed-xor split: phrase must be 12/15/18/21/24 words; got K` |
| 2 | `split --shares` < 2 | 1 | `seed-xor split: --shares must be >= 2; got N` |
| 3 | `combine --share` count mismatch vs `--shares` | 1 | `seed-xor combine: --shares N requires exactly N --share arguments; got K --share values for --shares N` |
| 4 | `combine` mixed-length shares | 1 | `seed-xor combine: all shares must be the same word count; got mix of {K1, K2, ...}` |
| 5 | `combine` mixed-language shares (one parses as english, another as spanish) | 1 | `seed-xor combine: all shares must use the same BIP-39 language; got mix of {L1, L2, ...}` |
| 6 | `combine` share at position I has BIP-39 checksum failure | 1 | `seed-xor combine: share at position I has invalid BIP-39 checksum (not a parseable mnemonic in --language <L>)` |
| 7 | `combine` unknown word in share at position I | 1 | (reuse `friendly_bip39` for `bip39::Error::UnknownWord`; prefix with `seed-xor combine: share at position I:`) |
| 8 | `--from` or `--share` variant other than `phrase=` | 1 | `seed-xor only accepts phrase=<value> or phrase=-` |
| 9 | Two or more `--share phrase=-` slots (multi-stdin) | 1 | `seed-xor combine: at most one --share value may be `-` (single stdin per invocation)` |

### §2.6 Advisories (stderr, non-fatal)

| Trigger | Stderr advisory |
|---|---|
| Inline `--from phrase=<v>` OR any inline `--share phrase=<v>` | Rendered by `secret_in_argv_warning(stderr, flag, alternative)`: `warning: secret material on argv (--from phrase= OR --share phrase=) — pipe via phrase=- to avoid /proc/$PID/cmdline exposure` — fires per-occurrence (NOT deduped) |
| `split` AND stdout is TTY AND `share_count >= 1` | **NEW advisory class** (first multi-secret-on-stdout): `warning: Seed XOR shares on stdout — each of the N=<n> lines is independently a complete BIP-39 phrase; ALL N shares are required to reconstruct the master; distribute them to N separate locations; do not paste this output into a single untrusted tool. Substitution of a wrong-but-valid-BIP-39 share is undetectable by Seed XOR — verify the recovered wallet's derived address before trusting it.` |
| `combine` AND stdout is TTY | `warning: combined phrase is secret material — Seed XOR has no authentication tag; verify the recovered wallet's expected derived address before trusting; if a share was substituted with a wrong-but-valid one, the result will validate but derive the wrong wallet` |
| `--json-out` to a world-readable path (umask 022 default; `#[cfg(unix)]` only) | Reuse v0.11.0 permission-mode advisory pattern at `cmd/final_word.rs:175-200` |
| `split --deterministic-from-master` with input phrase length ∈ {15, 21} words | `warning: --deterministic-from-master with 15/21-word input is toolkit-only — Coldcard's xor_seed.py natively supports 12/18/24 only; resulting shares will NOT round-trip a Coldcard device. For Coldcard interop, use 12/18/24-word input.` |

## §3 Out-of-scope (filed for explicit closure)

| OOS class | Rationale | Where it goes |
|---|---|---|
| `OOS-seed-xor-15-21-coldcard-interop` | Coldcard's `xor_seed.py` supports 16/24/32-byte entropy (= 12/18/24-word seeds). Our 15/21 support is toolkit-only; Coldcard hardware cannot round-trip those two sizes. | Documented in manual + `seed-xor split --help` |
| `OOS-seed-xor-threshold-k-of-n` | Seed XOR is fundamentally all-or-nothing. K-of-N is SLIP-39's domain. | v0.13.0 |
| `OOS-seed-xor-cross-language-shares` | Mixed-language combine is refused (§2.5 row 5); we don't attempt translation. | Hard refusal |
| `OOS-seed-xor-substitution-detection` | No MAC + no per-share digest in the Seed XOR format; mathematically undetectable. | Manual warning + advisory text |

## §4 Acceptance gates

| Gate | Criterion |
|---|---|
| G1 — Coldcard round-trip on 12/18/24 sizes | Vendor a Coldcard reference vector (one each for 12/18/24-word). Split via `--deterministic-from-master`, then combine, byte-equal recovery. Pinned in `tests/lib_seed_xor.rs`. |
| G2 — Algorithmic round-trip on all 5 sizes | For each of 12/15/18/21/24 words: random entropy → `split N=2..=5` → `combine` → byte-equal master. Property test ≥ 100 random vectors per size. |
| G3 — Plain stdout shape | `seed-xor split --from phrase=<X> --shares 3` emits exactly 3 lines, each a parseable BIP-39 phrase in the input language; trailing newline; no extraneous stdout. |
| G4 — JSON envelope stability | SHA-pinned envelope over 2 anchor vectors (deterministic split with `--deterministic-from-master`). |
| G5 — Refusal + advisory coverage | All 9 refusal classes (§2.5) have CLI tests asserting exit code 1 + pinned stderr stem. The 5 advisory classes (§2.6) — including the new `--deterministic-from-master` + 15/21-word advisory — each have a positive + negative test. |
| G6 — Cycle A/B discipline | Cycle A: argv-leakage advisory + `Zeroizing<String>` wraps + new `lint_argv_secret_flags.rs` rows (`seed-xor split --from phrase=`, `seed-xor combine --share phrase=`) — count goes 21 → 23. Cycle B: mlock Site 1 pins on parsed inputs + per-share output buffers. New `lint_zeroize_discipline.rs` row. |
| G7 — Manual chapter | `docs/manual/src/40-cli-reference/41-mnemonic.md` adds `## mnemonic seed-xor` section (Synopsis + Flags for split + Flags for combine + Worked example + JSON output + Refusals + Advisories); `cli-subcommands.list` adds `mnemonic seed-xor split` + `mnemonic seed-xor combine`; chapter intro bumps from 7 to 8 subcommands (7 user-facing + introspection-only `gui-schema`). |
| G8 — Bundle interop | A worked-example test: `seed-xor combine` output piped into `bundle --slot @0.phrase=-` produces the expected bundle (the recovered phrase IS a valid BIP-39 phrase). |

## §5 Cross-refs

Existing utilities reused (paths verified at grep-against-source ground truth):

| Utility | Path |
|---|---|
| `secret_advisory::secret_in_argv_warning(stderr, flag, alternative)` | `src/secret_advisory.rs:25-30` |
| `mnemonic_toolkit::mlock::pin_pages_for(&[u8])` | `src/mlock.rs:90-127` |
| `FromInput` + `parse_from_input` + `NodeType` | `src/cmd/convert.rs:30-151` |
| `read_stdin_to_string<R: Read>(stdin: &mut R)` | `src/cmd/convert.rs:566-572` |
| `CliLanguage` + `From<CliLanguage> for bip39::Language` | `src/language.rs:6-57` |
| `ToolkitError::BadInput` / exit codes | `src/error.rs:11,242-280` |
| `friendly_bip39` (for `bip39::Error::UnknownWord` mapping) | `src/friendly.rs:10-31` |
| `bip39::Mnemonic::parse_in` + `from_entropy_in` | `bip39 = "2"` (existing dep) |
| `std::io::IsTerminal` | std (v0.11.0 first toolkit use) |
| `#[cfg(unix)]` permission-mode helper precedent | `src/cmd/final_word.rs:175-200` |
| JSON envelope `schema_version: "1"` + serde struct field-order convention | `src/cmd/final_word.rs:148-200` (v0.11.0 precedent) |
| Manual chapter pattern | v0.11.0 `## mnemonic final-word` (Synopsis → Flags → Worked example → JSON output → Refusals → Advisories) |
| Lint anchors | `tests/lint_argv_secret_flags.rs` (baseline 21 rows post-v0.11.0) + `tests/lint_zeroize_discipline.rs` (loose-bound 18..=35) |

New crate dep:
- `rand_core = "0.6"` (RustCrypto, MIT/Apache-2.0) — added as a crate dep in `crates/mnemonic-toolkit/Cargo.toml` (single-member workspace; no `[workspace.dependencies]` section).
