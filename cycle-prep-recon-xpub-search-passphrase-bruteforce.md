# cycle-prep recon — 2026-06-05 — xpub-search-passphrase-bruteforce

**Origin/master SHA at recon time:** `86a59bb`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** only `cycle-prep-recon-*.md` / `feature-coverage-survey-*.md` scratch + `.claude/` + `CONTINUITY.md` (none load-bearing).

Slug verified: `xpub-search-passphrase-bruteforce`. **CLEAN** — the existing `passphrase_of_xpub.rs` doc-comment literally documents the exclusion + names this FOLLOWUP; all citations accurate. **User-scoped (2026-06-05) to candidate-FILE mode ONLY** (`--passphrase-candidates-file <path>`, one line per candidate): the user explicitly wants "a text file to be imported, rather than a long list on command line." **DROP** the stdin candidate variant (mode b) and **DEFER** generated wordlists (mode c) to btcrecover. File-only also dodges the `--passphrase-candidates-stdin`-vs-`--phrase-stdin` double-stdin contention (seed can still use `--phrase-stdin`).

---

## Per-slug verification
### xpub-search-passphrase-bruteforce
- **WHAT (from FOLLOWUPS.md):** extend `xpub-search passphrase-of-xpub` (single-passphrase verify) to scan many candidates. Modes (a) `--passphrases-file`, (b) `--passphrases-stdin`, (c) generated wordlists. **Scoped this cycle: (a)+(b) only.**
- **Citations:**
  - `crates/mnemonic-toolkit/src/cmd/xpub_search/passphrase_of_xpub.rs` — **ACCURATE**. Exists; doc-comment `:19-23` says verbatim "No `--passphrases-file <path>` brute-force / No generated passphrase wordlists / Filed as FOLLOWUP `xpub-search-passphrase-bruteforce` for v0.27+".
  - the oracle to loop: `derive_master_seed(mnemonic, passphrase)` (`:30`) → `match_xpub_against_paths` (`:26`, from `path_search.rs`) over standard BIP-44/49/84/86 paths — **ACCURATE**. Result builders `build_passphrase_match` (`:201`) / `build_passphrase_no_match` (`:220`).
  - new `passphrase_search.rs` for the iterator/streaming — **ACCURATE placement**: the module `cmd/xpub_search/` already pairs each `*_of_*.rs` mode with a `*_search.rs` primitive (`path_search.rs`, `address_search.rs`, `account_search.rs`); `passphrase_search.rs` fits.
  - **give-up exit code already exists:** `XpubSearchNoMatch { mode, searched }` → `exit_code` **4** (`error.rs:328,533`). No new error variant needed — exhaustion emits `XpubSearchNoMatch` (exit 4) with `searched` = #candidates tried; a hit → exit 0 + the matching passphrase.
  - existing passphrase-source flags `--passphrase` / `--passphrase-stdin` are a **mandatory one-of group** (`passphrase_of_xpub.rs:80-92`, `conflicts_with` + `required_unless_present`) — the new flags JOIN this group (see Action).
- **Action for brainstorm spec:** add `--passphrases-file <path>` + `--passphrases-stdin` to `PassphraseOfXpubArgs`; new `passphrase_search.rs` (read candidates from file/stdin, loop `derive_master_seed`+`match_xpub_against_paths`, abort-on-first-match → emit the winning passphrase + path, else `XpubSearchNoMatch{searched=#candidates}`). Cite source SHA `86a59bb`.

---

## Cross-cutting observations
1. **Flag-group + naming hazard (resolve in SPEC).** Today exactly one of `{--passphrase, --passphrase-stdin}` is required. The new flags make it one-of `{--passphrase, --passphrase-stdin, --passphrases-file, --passphrases-stdin}`. **`--passphrases-stdin` (plural, NEW) vs `--passphrase-stdin` (singular, existing) is a dangerous near-collision** + both read stdin (mutually exclusive by construction). SPEC should pick a clearer name (e.g. `--passphrase-candidates-file` / `--passphrase-candidates-stdin`, or `--passphrases-from-stdin`) and define the full conflicts_with matrix.
2. **`searched`-count semantic.** Open sibling slug `xpub-search-address-of-xpub-searched-count-semantic` (FOLLOWUPS `:382`) is about `XpubSearchNoMatch.searched` over-reporting in address mode. The new passphrase scan must define `searched` unambiguously (= #passphrase candidates tried, NOT candidates×paths) to avoid the same bug class. Note (not fix) the sibling slug.
3. **Boundary-refinement lockstep (the btcrecover stance).** The flat "`mnemonic` cannot brute-force" claim lives in THREE coupled places that must move together: `main.rs:51` `PASSPHRASE_RECOVERY_HELP` const; `tests/cli_help_fixtures.rs:22` guard test (asserts the btcrecover pointer — KEEP it, it still points there for keyspace *generation*); `docs/manual/src/40-cli-reference/41-mnemonic.md:23` mirror. Refine to: "mnemonic verifies a candidate list you supply (`xpub-search passphrase-of-xpub --passphrase-candidates-file`); for keyspace GENERATION/mutation/masks, use btcrecover." The guard test asserts `btcrecover` + the URL — refinement keeps both, but if it asserts the literal "cannot brute-force" phrase, update that assertion.
4. **Secret-handling (by design reveals the passphrase).** Candidates come from a FILE/stdin (NOT argv → no argv-leak advisory per candidate; emit the existing argv advisory only for an inline `--passphrase`). A hit prints the matching passphrase to stdout/`--json` — that IS the forensic deliverable (this tool's purpose is to FIND the passphrase), so watch-only-out does NOT apply here; note the candidate FILE is sensitive (advisory). Do NOT emit the per-invocation STDERR_ADVISORY (`:234`) once per candidate — emit once.

## Recommended brainstorm-session scope
**Single cycle.** `xpub-search-passphrase-bruteforce` (modes a+b). **SemVer: MINOR** (new user-facing capability + a messaging/boundary change to the after_help; though "additive flags on an existing subcommand" alone would be PATCH, the candidate-list scan + after_help refinement make MINOR the honest call). **Size ≈ 120-200 LOC:** `passphrase_search.rs` (file/stdin candidate iterator + abort-on-match loop) + flag wiring + the conflicts_with matrix + after_help/manual refinement. **Locksteps:**
- **GUI `schema_mirror`:** `xpub-search-passphrase-of-xpub` IS GUI-schema'd (`mnemonic-gui/src/schema/mnemonic.rs:2214`, C4) → the 2 new flags trip `schema_mirror`. Pin-blocked (GUI can't lead its toolkit pin) → add the flags this cycle + **file FOLLOWUP `gui-xpub-search-passphrase-candidates-flags-pending-pin-bump`** (mirrors the `gui-restore-multisig-flags-pending-pin-bump` precedent). Confirm at R0 via `gui-schema` diff.
- **Manual:** `41-mnemonic.md` passphrase-of-xpub section (new flags + candidate-list behavior) + the recovery-boundary refinement (obs #3). Run `make audit` (anchor-check + verify-examples), not just `make lint`.
- **after_help guard:** update `cli_help_fixtures.rs` if it pins the exact "cannot brute-force" wording.
- **DEFER mode (c)** generated wordlists → btcrecover (note in the SPEC; optionally a `wont-fix`/`external` sub-note on the slug rather than a new FOLLOWUP, since btcrecover already owns keyspace generation).
- No sibling-codec change. No inter-slug dependency (note the `searched`-count sibling, obs #2).
