# SPEC — `xpub-search passphrase-of-xpub --passphrase-candidates-file` (candidate-list passphrase scan)

**Status:** R0 gate (pre-implementation). MUST converge to 0 Critical / 0 Important before any code.
**Resolves:** FOLLOWUP `xpub-search-passphrase-bruteforce` (candidate-FILE scope only).
**Source SHA:** branch `xpub-search-passphrase-candidates-file` off master `45e83fe` (citations verified live through R0-r3).
**SemVer:** MINOR — additive capability (candidate-list scan) + a refinement of the top-level passphrase-recovery boundary messaging; a `Match` wire-shape addition.

---

## 1. Summary

`xpub-search passphrase-of-xpub` verifies whether ONE passphrase produces a target xpub (re-derive master via `derive_master_seed`, then `match_xpub_against_paths` over BIP-44/49/84/86 + `--add-path`). This cycle adds **`--passphrase-candidates-file <PATH>`**: a text file with **one candidate passphrase per line**; the command loops the existing verify oracle over the candidates, **aborts on the first match**, and reports which line matched (else exits `XpubSearchPassphraseCandidatesExhausted`/4). Candidates never touch argv (user requirement: "a text file to be imported, rather than a long list on command line").

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

**(R0-r1 I3) Secret classification = `secret: false` (it holds a PATH, not a secret value).** Mirror the established convention for path flags: `--decrypt-password-file` (`mnemonic-gui schema mnemonic.rs:2133-2134` "holds a PATH (non-secret)") and `--secret-file` (`:3042` "a plain path"). Concretely: do NOT extend `secrets.rs::flag_is_secret`; the GUI mirror entry (§6) is `FlagKind::Path { stdio_sentinel: false }, secret: false`; and **NO `lint_argv_secret_flags.rs` Route is added** (the secret never enters argv — the file IS the channel — so there's no `*-stdin`/`=-`/`@env:` evidence anchor a Route would require; a `secret: true` classification would FAIL the `flag_axis_set_equals_gui_schema` closure with nothing to anchor). The file's sensitivity is conveyed in the help text + a one-line runtime stderr advisory ("note: <path> holds candidate passphrases — treat as sensitive"), NOT via the secret-flag taxonomy.

**Mutex (exactly-one passphrase source). (R0-r2 M-2 — decided: use a clap `ArgGroup`.)** Today `--passphrase` / `--passphrase-stdin` are a mandatory one-of (pairwise `conflicts_with` + `required_unless_present`, `:78-92`). **Replace** that pairwise pair with a clap **`ArgGroup`** `passphrase_source` (`required = true`, `multiple = false`) over `{passphrase, passphrase_stdin, passphrase_candidates_file}`, and **REMOVE** the now-redundant per-field `conflicts_with`/`required_unless_present` on `--passphrase`/`--passphrase-stdin` (`:80-81`, `:89-90`) to avoid double-validation. **No stdin contention:** `--passphrase-candidates-file` reads a FILE, so the seed may still arrive via `--phrase-stdin`/`--ms1-stdin` (unlike a hypothetical candidate-stdin). Mutex/required violations are clap errors → **exit 64** (`main.rs:147`).

## 3. Scan engine — new `cmd/xpub_search/passphrase_search.rs`

**(R0-r1 I1) Dispatch FIRST, before the existing single-passphrase resolve.** `passphrase_of_xpub.rs::run` resolves the passphrase inline at `:260-289` and falls into an `else` → `BadInput("requires --passphrase or --passphrase-stdin")` at `:282-289` when neither `--passphrase` nor `--passphrase-stdin` is set — which is EXACTLY candidates-file mode. So `run` must branch at the top of its handler: **`if args.passphrase_candidates_file.is_some() { return run_candidate_scan(...) }`** — routing to the scan engine and SKIPPING the inline resolve+derive+match (`:260-318`) entirely (the scan owns its own per-candidate derive/match loop). Update the now-stale `:283-285` "unreachable" comment (the 3-way group means the `else` is reachable only as a clap-guaranteed-impossible defensive arm).

Mirrors the existing `*_search.rs` primitives (`path_search.rs` etc.). Streams the file line-by-line (no full-file buffering):

```
for (line_no_1based, line) in file.lines().enumerate():
    strip trailing '\r'? (BufRead::lines already drops '\n'; strip a trailing '\r' for CRLF)
    if line.is_empty(): continue            // blank-line skip
    candidates_tried += 1
    let candidate: Zeroizing<String> = Zeroizing::new(line)   // (I-r3) owned secret
    let seed = derive_master_seed(&mnemonic, &candidate)       // seed is Zeroizing<[u8;64]>
    if let Some(hit) = match_xpub_against_paths(seed, &paths, &target65):
        return Match { …hit…, matched_candidate_line: line_no_1based }   // ABORT on first
return Exhausted { candidates_tried }
```

- The `mnemonic` is parsed ONCE (from `--phrase`/`--ms1`/positional via `resolve_seed`); per candidate, `derive_master_seed(mnemonic, passphrase)` (PBKDF2) + `Xpriv::new_master` + a full `match_xpub_against_paths` walk re-run.
- **(R0-r3 I-r3) Memory hygiene.** Each candidate line is an OWNED secret → wrap in `Zeroizing<String>` before `derive_master_seed`, mirroring the single-passphrase path (`passphrase_of_xpub.rs:260`); it scrubs on drop each iteration. The derived seed is already `Zeroizing<[u8;64]>` (`derive_slot.rs:31`). **Add a `ZEROIZE_ROWS` entry** to `tests/lint_zeroize_discipline.rs` for the new `passphrase_search.rs` candidate-line site (evidence anchor `Zeroizing::new(...)`), per the lint's documented "add a row AND wrap" process (`:46-47`) — recording the new site in the curated discipline list (NB the lint is curated-row/lagging like `schema_mirror`, so the row-add is a Phase-2 deliverable, §8, not an auto-firing leading gate). (mlock-pinning per candidate is optional churn; the `Zeroizing` wrap is the load-bearing invariant.)
- **(R0-r1 I2) Do NOT overload `XpubSearchNoMatch`.** Its `Display` (`error.rs:785`) is hardcoded *"…paths searched={searched}; widen the range with --max-account / --number-of-accounts…"* — wrong for a candidate scan (and "paths searched=0; widen --max-account" is nonsense for an empty file). Add a **NEW** variant `ToolkitError::XpubSearchPassphraseCandidatesExhausted { candidates_tried: usize }` (alphabetical placement per CLAUDE.md; `exit_code` → **4** like `XpubSearchNoMatch`; `kind` arm added), `Display` = *"no candidate in --passphrase-candidates-file produced the target xpub (N candidate(s) tried); verify the seed and --target-xpub, or add more candidates."* The empty-file case (`candidates_tried == 0`) gets a tailored note ("--passphrase-candidates-file had no candidates (all lines blank)"). Non-`--json` exhaustion → this error (stderr + exit 4). `--json` exhaustion → emit a `NoMatch` envelope carrying `candidates_tried` then exit 4 — mirroring the existing run() `--json` no-match path, which prints the `NoMatch` envelope FIRST and THEN returns the error (`passphrase_of_xpub.rs:365-384`).
- **`STDERR_ADVISORY` (`:234`) emits ONCE**, not per candidate.
- **Perf (R0-r1 M2):** per-candidate cost is PBKDF2-2048 (dominant) + master-key + `searched_count` child derivations (default 80 = 4 templates × 20 accounts, more with `--add-path`). Whole-file runtime scales `candidates × searched_count`, so a wide `--number-of-accounts` multiplies it — note in `--help`/manual. Stream the file; omit progress reporting for v1 (finite user-supplied list; no rate-limit needed).
- File-open failure → IO/`BadInput` error; empty file → the `Exhausted{candidates_tried:0}` variant above (exit 4).

## 4. Output — report WHICH line matched (don't echo the secret to stdout by default)

The matching passphrase is already in the user's file; echoing it to stdout/scrollback adds exposure. Default **text** output reports the **1-indexed file line number** of the match (+ the derived path/xpub), so the user can locate it (`sed -n '<n>p' file`):
```
✓ match: candidate on line 42 derives <xpub> at m/84'/0'/0' (bip84)
```
**(R0-r2 M-1)** All new optional result fields carry `#[serde(skip_serializing_if = "Option::is_none")]` so the single-`--passphrase` envelope is byte-unchanged (no `…: null` keys leak onto the existing path). `PassphraseOfXpubResult::Match` (`:172`) gains two **optional** fields (None in single-`--passphrase` mode; Some in scan mode):
- `matched_candidate_line: Option<usize>` (1-indexed file line).
- `matched_passphrase: Option<String>` — included in **`--json` only** (machine consumption; the operator explicitly opted into structured output). NOT in the default text form. **(R0-r3 I-r3)** held as `Zeroizing<String>` in-memory from the winning candidate until serialized (it is bound for `--json` stdout by opt-in, so this is partial — the load-bearing wrap is the per-candidate line above).

This is a **`--json` wire-shape change** (added optional fields) — NOT gated by GUI `schema_mirror` (flag-NAME gate only; per `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification`); GUI/consumers self-update. **(R0-r1 I2)** The scan's `NoMatch` JSON envelope ADDS `candidates_tried: Option<usize>` (= #non-blank lines tried) and keeps `searched_count` = the per-passphrase path-search count (UNCHANGED meaning — paths-per-candidate; clearly distinct from `candidates_tried`, avoiding the `searched`-over-report bug class of sibling slug `xpub-search-address-of-xpub-searched-count-semantic`). The **stderr/exit** path for scan exhaustion uses the NEW `XpubSearchPassphraseCandidatesExhausted{candidates_tried}` variant (§3) — NOT `XpubSearchNoMatch` (whose hardcoded "paths searched=…; widen --max-account" Display is wrong for a candidate scan).

## 5. Boundary refinement (after_help + manual + guard) — the btcrecover lockstep

The flat "`mnemonic` cannot brute-force" now has a bounded exception. Refine all three coupled sites, **keeping** the btcrecover pointer + URL + `2026-05-25` currency stamp (the `cli_help_fixtures.rs:34-38` guard asserts only those three substrings — stays green):
- `main.rs:51` `PASSPHRASE_RECOVERY_HELP` const (decl `:51`; the "cannot brute-force" text is `:54`, body `:51-62`, M3): add a sentence — *"If you have a LIST of likely passphrases, `mnemonic xpub-search passphrase-of-xpub --passphrase-candidates-file <file> --target-xpub <known-xpub>` tests each against a value you know. To GENERATE/mutate a keyspace (wordlists, masks, typo models), use btcrecover:"* then the existing pointer.
- `docs/manual/src/40-cli-reference/41-mnemonic.md:23` mirror — same refinement.
- `cli_help_fixtures.rs` — no change needed (asserts btcrecover/URL/date, all retained); add a NEW assertion that `--help` for `passphrase-of-xpub` lists `--passphrase-candidates-file` if a per-subcommand help fixture exists (else covered by schema_mirror/flag-coverage).

## 6. Lockstep / FOLLOWUPs

- **GUI `schema_mirror`:** `xpub-search-passphrase-of-xpub` IS GUI-schema'd (`mnemonic-gui/src/schema/mnemonic.rs:2214`). The new `--passphrase-candidates-file` flag trips the flag-NAME gate → pin-blocked (GUI can't lead its toolkit pin). Add the flag this cycle + **file FOLLOWUP `gui-xpub-search-passphrase-candidates-file-flag-pending-pin-bump`** (mirrors `gui-restore-multisig-flags-pending-pin-bump`). The eventual GUI mirror entry is **`FlagKind::Path { stdio_sentinel: false }, secret: false`** (per I3, copy the `--decrypt-password-file` entry shape at `mnemonic.rs:2148-2156`). Confirm at R0 via `gui-schema` diff (only this one flag should appear; `secret:false`).
- **Manual:** `41-mnemonic.md` passphrase-of-xpub section — new flag row + a candidate-file worked example + the §5 boundary refinement. Run **`make audit`** (anchor-check + verify-examples + flag-coverage), not just `make lint`. The flag-coverage lint REQUIRES the manual to list `--passphrase-candidates-file` (mirror the clap surface).
- **`--json` wire-shape:** ungated (note for GUI consumers; no schema_mirror change).
- **Mode (c) generated wordlists:** add a one-line `external`/btcrecover note on the slug at resolution (NOT a new FOLLOWUP — btcrecover owns keyspace generation).

## 7. Tests — `tests/cli_xpub_search_passphrase_candidates.rs` (new)

Fixture: a seed (`abandon…about`) with a KNOWN passphrase `P` producing a target xpub `X` at bip84 (capture `X` at test time via `mnemonic xpub-search passphrase-of-xpub --passphrase P --target-xpub <self>` or `convert`).
- **hit:** candidate file with `P` among decoys → exit 0, text reports the correct 1-indexed line; `--json` `result:"match"` + `matched_candidate_line` + `matched_passphrase == P`.
- **miss:** file without `P` → exit 4 `XpubSearchPassphraseCandidatesExhausted`, `candidates_tried == #non-blank lines` (the `--json` no-match envelope carries `candidates_tried`).
- **abort-on-first:** `P` appears twice; reports the FIRST line; `candidates_tried` ≤ that line's index.
- **blank-line skip:** blanks between candidates don't count toward `candidates_tried`; the line number still maps to the file (not the candidate ordinal).
- **exact-bytes:** a candidate with a trailing space (`"pw "`) is tested literally (no trim) — a file line `pw ` matches a passphrase `pw ` and NOT `pw`.
- **mutex:** `--passphrase X --passphrase-candidates-file f` → clap error **exit 64** (M1 — `main.rs:147` overrides clap 2→64; existing tests pin `code(64)` at `cli_xpub_search_passphrase_of_xpub.rs:105,129`); none of the 3 sources → clap "required" error (exit 64).
- **seed-via-stdin coexist:** `--phrase-stdin` (seed) + `--passphrase-candidates-file` (candidates) works (no stdin contention).
- **empty/missing file:** missing → IO error; empty → exit 4 + "no candidates" note.
- **secret hygiene:** the **default (non-`--json`)** stdout does NOT contain `P` (only the line number) — this cell must NOT also pass `--json` (where `P` legitimately appears by opt-in); a separate `--json` cell asserts `matched_passphrase == P`. The runtime advisory notes the file is sensitive.
- Full workspace `cargo test --no-fail-fast` + clippy GREEN per phase.

## 8. Phased plan
- **Phase 1 (RED):** `tests/cli_xpub_search_passphrase_candidates.rs` — fails (flag unknown today). Verify RED-for-the-right-reason.
- **Phase 2 (GREEN):** §2 flag + ArgGroup mutex; §3 `passphrase_search.rs` (candidate line `Zeroizing<String>`); §4 result fields (`skip_serializing_if`) + text/json output; the new `XpubSearchPassphraseCandidatesExhausted` error variant (alphabetical; `exit_code`/`kind`/Display arms); **add the `passphrase_search.rs` `ZEROIZE_ROWS` row** (M-r4a). Workspace test + clippy GREEN. Per-phase opus review → persist.
- **Phase 3 (docs + release):** §5 after_help/manual refinement + `make audit`; CHANGELOG; version v0.45.0 → **v0.46.0**; README markers; install.sh self-pin; FOLLOWUP `xpub-search-passphrase-bruteforce` → resolved (file-scope) + file `gui-xpub-search-passphrase-candidates-file-flag-pending-pin-bump`. Per-phase review.
- **Phase 4 (ship):** clean tree → ff-merge → tag `mnemonic-toolkit-v0.46.0` → push → watch CI (rust, install/sibling-pin-check, manual).

## 9. Risk
Low-moderate. Engine is a thin loop over a proven oracle. The genuine decisions (all R0-checkable): the 3-way passphrase-source mutex (clap ArgGroup vs pairwise), the exact-bytes line handling (strip only the line terminator; blank-skip), and the secret-output posture (line-number to stdout, passphrase only in `--json`). The `Match`/`NoMatch` wire-shape additions are optional fields (backward-compatible for the single-passphrase path).
