# SPEC — `xpub-search passphrase-of-xpub --passphrase-candidates-file` (candidate-list passphrase scan)

**Status:** R0 gate (pre-implementation). MUST converge to 0 Critical / 0 Important before any code.
**Resolves:** FOLLOWUP `xpub-search-passphrase-bruteforce` (candidate-FILE scope only).
**Source SHA:** branch `xpub-search-passphrase-candidates-file` off master `86a59bb`.
**SemVer:** MINOR — additive capability (candidate-list scan) + a refinement of the top-level passphrase-recovery boundary messaging; a `Match` wire-shape addition.

---

## 1. Summary

`xpub-search passphrase-of-xpub` verifies whether ONE passphrase produces a target xpub (re-derive master via `derive_master_seed`, then `match_xpub_against_paths` over BIP-44/49/84/86 + `--add-path`). This cycle adds **`--passphrase-candidates-file <PATH>`**: a text file with **one candidate passphrase per line**; the command loops the existing verify oracle over the candidates, **aborts on the first match**, and reports which line matched (else exits `XpubSearchNoMatch`/4). Candidates never touch argv (user requirement: "a text file to be imported, rather than a long list on command line").

**Scope (user-set 2026-06-05):** candidate **FILE** only. NO stdin candidate variant (dropped — also avoids `--phrase-stdin` double-stdin contention). NO generated/mutated wordlists (mode c stays btcrecover's job — the top-level after_help continues to point there for keyspace *generation*).

## 2. CLI surface — `passphrase_of_xpub.rs`

Add to `PassphraseOfXpubArgs`:
```rust
/// File of candidate BIP-39 passphrases, ONE per line (no argv exposure).
/// Each line is a literal candidate (the trailing newline — and a CR before
/// it — is stripped; NO other whitespace trimming, since a passphrase is an
/// exact byte string). BLANK lines are skipped (test the no-passphrase case
/// with `--passphrase ""`). The command derives the master seed per candidate
/// and stops at the first one that produces --target-xpub. The file is
/// SENSITIVE (holds secret candidates).
#[arg(long, value_name = "PATH")]
pub passphrase_candidates_file: Option<PathBuf>,
```

**Mutex (exactly-one passphrase source).** Today `--passphrase` / `--passphrase-stdin` are a mandatory one-of (pairwise `conflicts_with` + `required_unless_present`, `:78-92`). Replace with a clap **`ArgGroup`** `passphrase_source` (`required = true`, `multiple = false`) over `{passphrase, passphrase_stdin, passphrase_candidates_file}` (or extend the pairwise form to `conflicts_with_all` + `required_unless_present_any` across all three — R0 picks the lower-drift option). **No stdin contention:** `--passphrase-candidates-file` reads a FILE, so the seed may still arrive via `--phrase-stdin`/`--ms1-stdin` (unlike a hypothetical candidate-stdin).

## 3. Scan engine — new `cmd/xpub_search/passphrase_search.rs`

Mirrors the existing `*_search.rs` primitives (`path_search.rs` etc.). Streams the file line-by-line (no full-file buffering):

```
for (line_no_1based, line) in file.lines().enumerate():
    strip trailing '\r'? (BufRead::lines already drops '\n'; strip a trailing '\r' for CRLF)
    if line.is_empty(): continue            // blank-line skip
    candidates_tried += 1
    let seed = derive_master_seed(&mnemonic, &line)   // language already resolved once
    if let Some(hit) = match_xpub_against_paths(seed, &paths, &target65):
        return Match { …hit…, matched_candidate_line: line_no_1based }   // ABORT on first
return NoMatch { candidates_tried }
```

- The `mnemonic` is parsed ONCE (from `--phrase`/`--ms1`/positional via `resolve_seed`); only `derive_master_seed(mnemonic, passphrase)` (PBKDF2) re-runs per candidate.
- **`STDERR_ADVISORY` (`:234`) emits ONCE**, not per candidate.
- **`derive_master_seed` is the per-candidate cost (PBKDF2-2048).** A large file is slow but bounded; stream + (optional) a periodic stderr progress line every N candidates (R0: include or omit — keep simple, omit for v1 unless trivial). No rate-limit needed (finite user-supplied list, not a generator).
- File-open failure → `BadInput`/IO error (exit per `error.rs`); empty file (0 candidates) → `XpubSearchNoMatch{searched:0}` (exit 4) with a stderr note "no candidates in file".

## 4. Output — report WHICH line matched (don't echo the secret to stdout by default)

The matching passphrase is already in the user's file; echoing it to stdout/scrollback adds exposure. Default **text** output reports the **1-indexed file line number** of the match (+ the derived path/xpub), so the user can locate it (`sed -n '<n>p' file`):
```
✓ match: candidate on line 42 derives <xpub> at m/84'/0'/0' (bip84)
```
`PassphraseOfXpubResult::Match` (`:172`) gains two **optional** fields (None in single-`--passphrase` mode; Some in scan mode):
- `matched_candidate_line: Option<usize>` (1-indexed file line).
- `matched_passphrase: Option<String>` — included in **`--json` only** (machine consumption; the operator explicitly opted into structured output). NOT in the default text form.

This is a **`--json` wire-shape change** (added optional fields) — NOT gated by GUI `schema_mirror` (flag-NAME gate only; per `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification`); GUI/consumers self-update. NoMatch keeps `searched_count`; ADD `candidates_tried: Option<usize>` for the scan (define `searched_count` = per-passphrase path-search count UNCHANGED; `candidates_tried` = #non-blank lines tried — avoids the `searched`-over-report bug class of sibling slug `xpub-search-address-of-xpub-searched-count-semantic`). The `XpubSearchNoMatch{mode,searched}` error: for the scan, `searched` = `candidates_tried`.

## 5. Boundary refinement (after_help + manual + guard) — the btcrecover lockstep

The flat "`mnemonic` cannot brute-force" now has a bounded exception. Refine all three coupled sites, **keeping** the btcrecover pointer + URL + `2026-05-25` currency stamp (the `cli_help_fixtures.rs:34-38` guard asserts only those three substrings — stays green):
- `main.rs:51` `PASSPHRASE_RECOVERY_HELP`: add a sentence — *"If you have a LIST of likely passphrases, `mnemonic xpub-search passphrase-of-xpub --passphrase-candidates-file <file> --target-xpub <known-xpub>` tests each against a value you know. To GENERATE/mutate a keyspace (wordlists, masks, typo models), use btcrecover:"* then the existing pointer.
- `docs/manual/src/40-cli-reference/41-mnemonic.md:23` mirror — same refinement.
- `cli_help_fixtures.rs` — no change needed (asserts btcrecover/URL/date, all retained); add a NEW assertion that `--help` for `passphrase-of-xpub` lists `--passphrase-candidates-file` if a per-subcommand help fixture exists (else covered by schema_mirror/flag-coverage).

## 6. Lockstep / FOLLOWUPs

- **GUI `schema_mirror`:** `xpub-search-passphrase-of-xpub` IS GUI-schema'd (`mnemonic-gui/src/schema/mnemonic.rs:2214`). The new `--passphrase-candidates-file` flag trips the flag-NAME gate → pin-blocked (GUI can't lead its toolkit pin). Add the flag this cycle + **file FOLLOWUP `gui-xpub-search-passphrase-candidates-file-flag-pending-pin-bump`** (mirrors `gui-restore-multisig-flags-pending-pin-bump`). Confirm at R0 via `gui-schema` diff (only this one flag should appear).
- **Manual:** `41-mnemonic.md` passphrase-of-xpub section — new flag row + a candidate-file worked example + the §5 boundary refinement. Run **`make audit`** (anchor-check + verify-examples + flag-coverage), not just `make lint`. The flag-coverage lint REQUIRES the manual to list `--passphrase-candidates-file` (mirror the clap surface).
- **`--json` wire-shape:** ungated (note for GUI consumers; no schema_mirror change).
- **Mode (c) generated wordlists:** add a one-line `external`/btcrecover note on the slug at resolution (NOT a new FOLLOWUP — btcrecover owns keyspace generation).

## 7. Tests — `tests/cli_xpub_search_passphrase_candidates.rs` (new)

Fixture: a seed (`abandon…about`) with a KNOWN passphrase `P` producing a target xpub `X` at bip84 (capture `X` at test time via `mnemonic xpub-search passphrase-of-xpub --passphrase P --target-xpub <self>` or `convert`).
- **hit:** candidate file with `P` among decoys → exit 0, text reports the correct 1-indexed line; `--json` `result:"match"` + `matched_candidate_line` + `matched_passphrase == P`.
- **miss:** file without `P` → exit 4 `XpubSearchNoMatch`, `candidates_tried == #non-blank lines`.
- **abort-on-first:** `P` appears twice; reports the FIRST line; `candidates_tried` ≤ that line's index.
- **blank-line skip:** blanks between candidates don't count toward `candidates_tried`; the line number still maps to the file (not the candidate ordinal).
- **exact-bytes:** a candidate with a trailing space (`"pw "`) is tested literally (no trim) — a file line `pw ` matches a passphrase `pw ` and NOT `pw`.
- **mutex:** `--passphrase X --passphrase-candidates-file f` → clap error (exit 2); none of the 3 sources → clap "required" error.
- **seed-via-stdin coexist:** `--phrase-stdin` (seed) + `--passphrase-candidates-file` (candidates) works (no stdin contention).
- **empty/missing file:** missing → IO error; empty → exit 4 + "no candidates" note.
- **secret hygiene:** the default (non-`--json`) stdout does NOT contain `P` (only the line number); the advisory notes the file is sensitive.
- Full workspace `cargo test --no-fail-fast` + clippy GREEN per phase.

## 8. Phased plan
- **Phase 1 (RED):** `tests/cli_xpub_search_passphrase_candidates.rs` — fails (flag unknown today). Verify RED-for-the-right-reason.
- **Phase 2 (GREEN):** §2 flag + ArgGroup mutex; §3 `passphrase_search.rs`; §4 result fields + text/json output. Workspace test + clippy GREEN. Per-phase opus review → persist.
- **Phase 3 (docs + release):** §5 after_help/manual refinement + `make audit`; CHANGELOG; version v0.45.0 → **v0.46.0**; README markers; install.sh self-pin; FOLLOWUP `xpub-search-passphrase-bruteforce` → resolved (file-scope) + file `gui-xpub-search-passphrase-candidates-file-flag-pending-pin-bump`. Per-phase review.
- **Phase 4 (ship):** clean tree → ff-merge → tag `mnemonic-toolkit-v0.46.0` → push → watch CI (rust, install/sibling-pin-check, manual).

## 9. Risk
Low-moderate. Engine is a thin loop over a proven oracle. The genuine decisions (all R0-checkable): the 3-way passphrase-source mutex (clap ArgGroup vs pairwise), the exact-bytes line handling (strip only the line terminator; blank-skip), and the secret-output posture (line-number to stdout, passphrase only in `--json`). The `Match`/`NoMatch` wire-shape additions are optional fields (backward-compatible for the single-passphrase path).
