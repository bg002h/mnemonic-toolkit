# SPEC — BIP-39 final-word completer (v0.11.0)

**Cycle:** mnemonic-toolkit v0.11.0 minor bump. New feature; toolkit-only (no cross-repo work).
**Status:** Phase 0 in flight. Reviewer-loop continues until 0 critical / 0 important on the R0 Opus pass.
**Predecessor:** v0.10.1 patch shipped 2026-05-13 (tag at `ed5a1d9`; v0.9.0 Cycles A + B + the v0.10.1 carve-out completion closed v1.0-roadmap Bucket 1 fully).
**Authoring session:** 2026-05-13. Brainstorm + plan at `~/.claude/plans/radiant-seeking-teacup.md`; user decisions locked via AskUserQuestion (CLI shape: standalone subcommand; output: plain stdout + `--json-out <path>` side-effect; target word count: implicit from input; phase structure: 4 phases + PE).
**Driving FOLLOWUP:** `bip39-final-word-completer` (`design/FOLLOWUPS.md`; closes at PE).

---

## §1. Purpose

Add a deterministic, language-aware BIP-39 final-word completer to the mnemonic-toolkit CLI as a standalone subcommand `mnemonic final-word`. The subcommand consumes an N-1-word partial phrase plus an optional language hint, and emits the complete set of valid Nth-word completions for the partial.

### Use cases (paper backup recovery + paper backup generation)

1. **Recovery**: user has a partial paper backup (e.g., last word smudged); enumerate the small candidate set to find the original.
2. **Setup**: user manually generates N-1 words from a trusted entropy source (dice, coin-flip), then uses the completer to pick a valid Nth word that fixes the checksum.
3. **Verification**: user types their full phrase but suspects a copy error; can compute `final-word(first N-1 words)` and compare against their last word — if the last word is in the candidate set, the phrase passes BIP-39 checksum (mirror of `Mnemonic::parse_in` checksum check, but with the ability to surface the correct alternative if it's wrong).

### Output set size (deterministic)

| N (target word count) | Entropy bits | Checksum bits (CS) | Set size = 2^(11 − CS) |
|---|---|---|---|
| 12 | 128 | 4 | 128 |
| 15 | 160 | 5 | 64 |
| 18 | 192 | 6 | 32 |
| 21 | 224 | 7 | 16 |
| 24 | 256 | 8 | 8 |

### Algorithm: naïve enumeration over the 2048-entry wordlist

For each of the 2048 wordlist entries in the chosen language, append it to the N-1 partial, attempt `bip39::Mnemonic::parse_in(language, &candidate_phrase)`, collect the `Ok(_)` results. Cost: 2048 SHA-256 ops per query (~milliseconds total). Math correctness is delegated to the well-tested `bip39` crate; no hand-rolled checksum logic.

The bit-math direct-derivation alternative (compute entropy bits, iterate (11 − CS) high bits, derive checksum bits) was considered and rejected. It is 2× to 256× faster (microseconds vs milliseconds) but introduces a hand-rolled correctness surface duplicating the bip39 crate's. The naïve form is simpler, harder to get wrong, and fast enough.

### Threat model

**Input secret-bearing**: yes. The N-1 partial words ARE the seed-phrase material; recovering or testing the Nth word reconstitutes the full seed. Treat the partial exactly like `bundle --slot @N.phrase=` per Cycle A discipline:
- `--from phrase=-` value carve-out (the sole stdin path; mirrors `convert --from phrase=-` exactly per the existing precedent at `src/cmd/convert.rs`)
- Inline `--from phrase=<value>` emits a Cycle A argv-leakage advisory to stderr via `secret_advisory::secret_in_argv_warning(stderr, "--from phrase=", "--from phrase=-")` (existing helper at `src/secret_advisory.rs:25-30`)
- `Zeroizing<String>` for the parsed partial
- Cycle B Phase 3a Site 1-style mlock pin on the parsed partial bytes
- Lint anchor in `tests/lint_argv_secret_flags.rs` for the new flag-row

**Output secret-bearing**: yes. ANY single candidate word combined with the partial = a valid seed phrase. The full candidate set leak is equivalent to revealing up to 128 valid mnemonics (for N=12). Treatment:
- The plain stdout emit is acceptable for user CLI use but warrants a stdout-on-tty advisory. **This is a NEW advisory class for v0.11.0; no existing toolkit precedent.** Implementation uses `std::io::IsTerminal` (stable since Rust 1.70; toolkit MSRV is Rust 1.85 per `Cargo.toml` workspace `rust-version`). If `std::io::stdout().is_terminal()` returns `true` AND the candidate set is being emitted, write a stderr warning ("the candidate list is secret material; do not paste into untrusted tools").
- The `--json-out <path>` side-effect file inherits the user's umask; §2.6 mandates a best-effort permission-mode check warning under `#[cfg(unix)]` (Unix-only API; current toolkit is Unix-only via libc + mlock).

**Threat model NOT addressed**: the candidate set is computable from any process that observes the partial via swap, ptrace, etc. — same Cycle B "Threat model NOT addressed" classes apply. No new additions.

---

## §2. Functional surface

### §2.1 Library entry point

New module `crates/mnemonic-toolkit/src/final_word.rs`. The toolkit became a hybrid lib + bin in Cycle B Phase 2; this module gets exposed via `pub mod final_word;` in `src/lib.rs`.

```rust
pub fn final_word_candidates(
    partial_phrase: &str,
    language: crate::language::CliLanguage,
) -> Result<Vec<&'static str>, crate::error::ToolkitError>;
```

- `partial_phrase`: whitespace-separated word list of length N-1, where N ∈ {12, 15, 18, 21, 24}.
- `language`: `CliLanguage` enum (existing toolkit type; `.into()` to `bip39::Language`).
- Returns lexicographically sorted `Vec<&'static str>` (entries borrowed from `Language::wordlist()`).
- Errors map to existing `ToolkitError` variants:
  - `BadInput` for input-count violations (input not in {11, 14, 17, 20, 23} words)
  - `Bip39` for unknown-word / parse-in failures (reusing the existing `bip39_friendly` error mapping at `src/friendly.rs:10-31`)
  - No new `ToolkitError` variants needed for v0.11.0.

Algorithm (the working buffer is bare `String` not `Zeroizing<String>` — the partial is secret but transit through a function-local that drops at every loop iteration is short-lived; caller-side wrap of the partial is the canonical scrub site):

```rust
let wordlist: &'static [&'static str; 2048] = bip39::Language::from(language).word_list();
let mut candidates = Vec::with_capacity(256);
for &candidate in wordlist {
    let mut full = String::with_capacity(partial_phrase.len() + 1 + candidate.len());
    full.push_str(partial_phrase);
    full.push(' ');
    full.push_str(candidate);
    if bip39::Mnemonic::parse_in(language.into(), &full).is_ok() {
        candidates.push(candidate);
    }
}
candidates.sort_unstable();
Ok(candidates)
```

Note: the bip39 v2 API is `Language::word_list()` (underscore + space, not `wordlist()`); existing toolkit usage at `src/wordlists/mod.rs:86` confirms the spelling.

### §2.2 CLI subcommand

New file `crates/mnemonic-toolkit/src/cmd/final_word.rs`. Argument struct:

```rust
use crate::cmd::convert::{parse_from_input, read_stdin_to_string, FromInput, NodeType};

#[derive(clap::Args)]
pub struct FinalWordArgs {
    /// Partial phrase (N-1 words) as `phrase=<value>` or `phrase=-` to read from stdin.
    #[arg(long = "from", value_name = "phrase=<value-or-->", value_parser = parse_from_input, required = true)]
    pub from: FromInput,

    /// BIP-39 language (default: english). Required if the partial is ambiguous across languages.
    #[arg(long = "language", default_value = "english")]
    pub language: crate::language::CliLanguage,

    /// Side-effect: write a stable JSON envelope to this path.
    #[arg(long = "json-out", value_name = "PATH")]
    pub json_out: Option<std::path::PathBuf>,
}
```

The `--from` flag reuses the existing `FromInput` struct from `src/cmd/convert.rs:121` for the `phrase=<value>` / `phrase=-` shape. `FromInput` is a `struct { node: NodeType, value: String }` (not an enum); the `parse_from_input` value-parser at `convert.rs:66-83` handles the `<node>=<value>` tokenization. Refusal of non-phrase variants happens at `run()` entry: `if args.from.node != NodeType::Phrase { return Err(ToolkitError::BadInput(...)); }`.

**Single stdin path:** `--from phrase=-` is the SOLE stdin route (mirrors `convert --from phrase=-` exactly). No separate `--phrase-stdin` paired flag — this is a deliberate departure from `bundle --passphrase-stdin` / `derive-child --passphrase-stdin`, justified by the fact that `final-word` has only ONE secret input (the partial), so the paired-flag pattern adds complexity without surfacing a different input. R0 round 1 also surfaced that a `required = true` `--from` cannot coexist with a `conflicts_with = "from"` `--phrase-stdin` (clap-unsatisfiable); collapsing to a single stdin route avoids the issue entirely.

`run()` signature mirrors existing commands (e.g., `convert.rs:597` shape):

```rust
pub fn run<R: std::io::Read, W: std::io::Write, E: std::io::Write>(
    args: FinalWordArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError>;
```

The `stdin: &mut R` shape matches `read_stdin_to_string<R: Read>(stdin: &mut R)` at `convert.rs:566`. R0 round 2 caught the earlier `Option<&str>` signature as incompatible with the existing helper.

Behavior:
1. Validate `args.from.node == NodeType::Phrase`; otherwise refuse with `BadInput` per §2.5.
2. Resolve partial-phrase source: if `args.from.value == "-"`, read from stdin via `read_stdin_to_string(stdin)` (existing helper at `src/cmd/convert.rs:566`); else use `args.from.value` directly AND emit Cycle A argv-leakage stderr advisory via `secret_advisory::secret_in_argv_warning(stderr, "--from phrase=", "--from phrase=-")` (existing helper at `src/secret_advisory.rs:25-30`).
3. Wrap parsed partial in `Zeroizing<String>` immediately.
4. mlock the partial bytes via `mnemonic_toolkit::mlock::pin_pages_for(parsed_partial.as_bytes())` immediately after stdin/argv resolution; bind to function-scope guard variable.
5. Call `final_word_candidates(&parsed_partial, args.language)`.
6. Emit candidates to stdout: one word per line, sorted (the library already sorts).
7. If `args.json_out` set: write the JSON envelope to that path; under `#[cfg(unix)]` surface a permission-mode advisory if the resulting file is world-readable (umask 022 default — see §2.6).
8. If `std::io::stdout().is_terminal()` AND `!candidates.is_empty()`: write the secret-on-stdout stderr advisory. This is a **NEW advisory class for v0.11.0** (no existing toolkit precedent — `convert.rs:796` is the Electrum SeedVersion note, not a stdout-on-TTY check).

### §2.3 JSON envelope schema

`v1` schema (starts at `"1"` for consistency with existing toolkit envelopes — `bundle --json` is currently at `"4"`, `convert --json` at `"1"`; per R0 round 1 I4, starting a new envelope at `"0"` would reverse the project pattern). Stable; pinned via `tests/cli_final_word_json.rs` SHA-of-canonical-output.

```json
{
  "schema_version": "1",
  "language": "english",
  "partial_word_count": 11,
  "target_word_count": 12,
  "candidate_count": 128,
  "candidates": ["abandon", "ability", "..."]
}
```

Fields:
- `schema_version`: string `"1"` (mirrors `bundle --json` envelope convention; minor bumps for non-breaking additions).
- `language`: kebab-case string rendered via `CliLanguage::human_name(&self) -> &'static str` (defined at `src/language.rs:26-39`; e.g., `"english"`, `"simplified-chinese"`). NOTE: `CliLanguage` has no `Display` impl — `human_name()` is the canonical kebab-case renderer.
- `partial_word_count`: integer (input length).
- `target_word_count`: integer (`partial_word_count + 1`).
- `candidate_count`: integer (= `candidates.len()`; redundant but useful for envelope-only consumers).
- `candidates`: array of strings, lexicographically sorted.

Note on `feature` namespace tag: R0 round 1 I4 considered adding a `feature: "final_word"` discriminator field. Decision: omit — the existing envelopes (`bundle --json`, `convert --json`) discriminate by binary + `mode`/`template`, not by a `feature` tag. Adding the tag silently in v0.11.0 would create an inconsistent convention. If a future cycle wants a cross-envelope namespace, it can be added uniformly across all subcommands.

### §2.4 Exit codes

Reuse `ToolkitError::exit_code()`:
- `0` on success (candidates emitted)
- `64` (`BadInput`) for wrong word count, unknown word, wrong language, ambiguous-language refusal
- Other existing codes inherited from `ToolkitError` propagation (e.g., `74` for I/O when writing `--json-out`)

### §2.5 Refusals (per-input-class)

| Input class | Refusal exit code | Stderr message format |
|---|---|---|
| 0 words / empty partial | 64 | `final-word: empty partial phrase; need 11/14/17/20/23 words for a target of 12/15/18/21/24` |
| Word count not in {11,14,17,20,23} | 64 | `final-word: got K words; expected one of [11, 14, 17, 20, 23] (target = K+1 must be in {12,15,18,21,24})` |
| Unknown word | 64 | reuse existing `bip39::Error::UnknownWord` mapping via `friendly.rs`: `BIP-39 unknown-word at index I; not in selected wordlist (selected language: <L>; did you pick the right --language?)` |
| `--from` variant other than `phrase=` | 64 | `final-word --from only accepts phrase=<value> or phrase=-` |

### §2.6 Advisories (stderr, non-fatal)

| Trigger | Message |
|---|---|
| Inline secret on argv (`--from phrase=<inline-value>`) | Rendered by `secret_in_argv_warning(stderr, "--from phrase=", "--from phrase=-")`: `warning: secret material on argv (--from phrase=) — pipe via --from phrase=- to avoid /proc/$PID/cmdline exposure` |
| Candidate emit AND stdout is TTY | `warning: candidate list is secret material — pairing the partial phrase with any candidate yields a valid seed phrase; do not paste this output into untrusted tools` |
| `--json-out` write to a world-readable path (umask 022 default) | `warning: --json-out <path> inherits umask (file may be world-readable); consider --json-out /dev/stdout` (best-effort; under `#[cfg(unix)]` only, fire if `std::os::unix::fs::PermissionsExt::mode(metadata.permissions()) & 0o077 != 0`. Non-Unix targets skip this check; the toolkit is currently Unix-only via libc + mlock so cfg-gating is a future-proofing concession.) |

---

## §3. Out-of-scope (filed for explicit closure)

| OOS class | Rationale | Where it goes |
|---|---|---|
| `OOS-arbitrary-position-completion` | The v0.11.0 scope is N-1 specifically (last word only). Generalized "missing K of N words" is exponentially larger (2048^K candidates filtered by checksum) and a different UX. | Future cycle if demanded |
| `OOS-fuzzy-typo-correction` | "I think word #7 might be slightly wrong" — Levenshtein-distance-aware repair against the wordlist. Distinct algorithm. | Future cycle / FOLLOWUP if user demand |
| `OOS-cross-language-ambiguity-resolution` | If the partial validates in 2+ languages, we refuse with the `--language` requirement. We do NOT attempt to detect / disambiguate. Mirrors bip39 crate's behavior. | bip39 crate upstream (existing FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream` is the closest; no new FOLLOWUP needed) |
| `OOS-passphrase-aware-validation` | The Nth word completion is checksum-only; it does NOT depend on the BIP-39 passphrase (passphrase enters at the seed-derivation step, after the mnemonic is fully formed). The user's `bip39` passphrase is irrelevant. | N/A (intentional final shape) |
| `OOS-batch-mode` | "Read 10 partials, emit 10 candidate sets." Single-shot for v1; can pipe via xargs. | Future cycle if demanded |

---

## §4. Acceptance gates

| Gate | Criterion |
|---|---|
| G1 — Correctness on BIP-39 official vectors | For each of 5 N values, a BIP-39 official test vector from `tests/bip39_trezor_vectors.json` yields a candidate set of the expected size (per the §1 table — 128/64/32/16/8) including the canonical Nth word. (Per-N canonical zero-entropy ends: N=12→`about`, N=18→`agent`, N=24→`art`; N=15 and N=21 use non-zero-entropy entries.) PLUS two user-locked named anchor vectors must pass: (a) `abandon × 11 about` 12-word target — partial `abandon × 11` → 128 candidates including `"about"`; (b) `beef × 12` target — partial `beef × 11` → 128 candidates with SHA-pinned sorted output as the regression backstop (membership of `"beef"` in the candidate set is computed and pinned by the algorithm, not asserted a priori). Both fixtures plus their pinned SHAs ship in `tests/lib_final_word.rs` and are reused by the CLI integration tests in P2. |
| G2 — Plain stdout output | `mnemonic final-word --from phrase=<23-words> --language english` emits exactly 8 lexicographically-sorted words, one per line, with a trailing newline; no other stdout content. |
| G3 — JSON envelope schema stability | `--json-out <path>` writes the envelope; field set + order matches §2.3 byte-for-byte over the pinned vector corpus. Schema regression caught by `tests/cli_final_word_json.rs` SHA-pin. |
| G4 — Refusal coverage | All 4 refusal classes in §2.5 surface their pinned stderr message with exit code 64. |
| G5 — Cycle A discipline | (a) inline secret-on-argv emits the advisory via `secret_in_argv_warning`; (b) `Zeroizing<String>` wraps the parsed partial; (c) `--from phrase=-` works end-to-end as the sole stdin path; (d) lint rows added to `lint_argv_secret_flags.rs` and `lint_zeroize_discipline.rs`. |
| G6 — Cycle B discipline | mlock pin on parsed-partial bytes; lint anchor in `lint_safety_first_party_mlock.rs` if any new `unsafe` block (expect zero new unsafe). |
| G7 — Manual mirror | New `docs/manual/src/40-cli-reference/47-mnemonic-final-word.md` chapter (or extension to `41-mnemonic.md`); `docs/manual/tests/lint.sh` passes for the new subcommand's flag set; `docs/manual/tests/cli-subcommands.list` includes `final-word`. |
| G8 — No wire-format regression | v0.1 + v0.2 fixture-corpus SHA pins continue to hold (final-word does not touch wallet artifacts; this is a transparency check). |

---

## §5. Cross-refs

- **Plan + brainstorm:** `~/.claude/plans/radiant-seeking-teacup.md` (the operational artifact this SPEC ships from; per `feedback_plan_artifact_mirror_project_convention`, plan-mode content IS the SPEC + phased plan).
- **Existing CLI patterns to mirror:** `src/cmd/convert.rs` (`--from phrase=` value-or-stdin via `parse_from_input` + `FromInput` struct; `read_stdin_to_string` helper at `convert.rs:566`); `src/cmd/derive_child.rs` (`Zeroizing<String>` discipline). NOTE: `final-word` does NOT use a paired `--phrase-stdin` flag — `--from phrase=-` is the sole stdin path (single-secret-input justifies single-route).
- **Existing BIP-39 error mapping:** `src/friendly.rs:10-31` — reuse the 5-variant mapping verbatim
- **Existing wordlist access:** `src/wordlists/mod.rs:86` (calls `bip39::Language::English.word_list()`); the new module follows the same pattern but for the selected `CliLanguage`. Method name is `word_list()` (with underscore), not `wordlist()`.
- **Existing lint enumeration:** `tests/lint_argv_secret_flags.rs` (`FlagRow`), `tests/lint_zeroize_discipline.rs` (`ZeroizeRow`)
- **Cycle A SPEC (Zeroize discipline):** `design/SPEC_secret_memory_hygiene_v0_9_0.md`
- **Cycle B SPEC (mlock discipline):** `design/SPEC_secret_memory_hygiene_v0_9_B.md`
- **Manual flag-coverage lint:** `docs/manual/tests/lint.sh:62-98`
- **JSON envelope precedent:** `tests/cli_json_envelopes.rs:9-38` (`bundle --json` shape)
- **Predecessor cycle:** v0.10.1 patch (tag `mnemonic-toolkit-v0.10.1` at `ed5a1d9`; closed v1.0-roadmap Bucket 1 through the Cycle B Path B-lite carve-out)
