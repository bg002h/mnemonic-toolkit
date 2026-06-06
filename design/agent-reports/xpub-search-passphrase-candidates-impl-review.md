# Per-phase implementation review — mnemonic-toolkit v0.46.0 (passphrase-candidates-file scan)

**Reviewer:** opus `feature-dev:code-reviewer` (gate before tag). **Date:** 2026-06-05.
**Branch:** `xpub-search-passphrase-candidates-file` (Phase 2 `628ce29` + Phase 3 `6596511`).
**Verdict:** **0 Critical / 1 Important — RED** → folded (I1 + 4 Minors) → GREEN.

> Persisted verbatim per CLAUDE.md. I1 independently re-verified at source by the orchestrator (SPEC §2:31 mandates the runtime advisory; `grep treat-as-sensitive src/` returned no match before the fold). Reviewer environment had no shell — runtime items verified via the test-file assertions + source trace; the orchestrator additionally ran the binary (advisory emits on stderr; missing-file exit 1).

---

## Important

**I1 — The SPEC-mandated runtime sensitivity advisory was not implemented.** SPEC §2:31 (R0-r1 I3): the file's sensitivity is conveyed "in the help text **+ a one-line runtime stderr advisory** (`note: <path> holds candidate passphrases — treat as sensitive`), NOT via the secret-flag taxonomy." This was the explicit compensating control the R0 architect accepted for the `secret: false` classification. The help-text half landed (`passphrase_of_xpub.rs:96-97`); the runtime stderr advisory did NOT (grep over `src/` + all four R0 reports found neither the impl nor a waiver). Disclosure/UX gap, not a leak (candidate lines `Zeroizing`; default stdout doesn't echo the passphrase, test-enforced; file never in argv). **Fix:** emit one `writeln!(stderr, …)` before the read loop in `run_candidate_scan`; thread a `stderr` writer into the fn (its `passphrase_of_xpub.rs:281` call site has `stderr` in scope).

## Minor
- **M1** — Manual JSON-output subsection (`41-mnemonic.md:3538-3572`) omits the new `matched_candidate_line`/`matched_passphrase`/`candidates_tried` fields. Add a candidate-scan JSON sample.
- **M2** — Manual lacks a candidate-file worked example (SPEC §6:82 asked for one).
- **M3** — Manual exit-codes table (`:3580`) lists only `XpubSearchNoMatch` for exit 4 (omits candidate exhaustion); Refusals table (`:3588-3589`) still frames the mutex as 2-way `--passphrase`/`--passphrase-stdin` (stale vs exactly-one-of-three).
- **M4** — No missing-file test cell (Cell 9 covers empty/all-blank only); `File::open` failure → `BadInput` (exit 1).

## What verified clean (source + test-assertion trace)
- **Dispatch (R0-r1 I1):** scan branch at `passphrase_of_xpub.rs:280-282` — after `resolve_seed` (:275), before the inline single-passphrase resolve (:290). `run_candidate_scan` re-resolves target + rebuilds paths itself; nothing from the skipped block lost; seed advisory (:272) precedes `resolve_seed` so always emits. ACCURATE.
- **Scan engine:** streams `BufReader::lines()` (no slurp); strips only trailing `\r` after `lines()` drops `\n`, no other trim; blank-skip; candidate `Zeroizing<String>` (:63); per-candidate `derive_master_seed` + `Xpriv::new_master` + `match_xpub_against_paths`; abort-on-first; 1-indexed FILE line = `idx+1` over all lines incl. blanks; `candidates_tried` counts non-blank only; miss → `XpubSearchPassphraseCandidatesExhausted{candidates_tried}` exit 4 + JSON NoMatch-first. ACCURATE.
- **Error variant:** alphabetical last; `exit_code→4`; `kind` arm; Display avoids "widen --max-account", empty-file tailored note; `details()` `_ => None` catch-all so no exhaustiveness break. ACCURATE.
- **Mutex:** 3-way `ArgGroup(required, !multiple)`; pairwise removed; FILE flag no `--phrase-stdin` contention; Cells 7/8 pin 64. ACCURATE.
- **Secret hygiene/classification:** candidate `Zeroizing`; `ZEROIZE_ROWS` row w/ anchor `Zeroizing::new(raw)`; `flag_is_secret` unchanged (not secret); no `lint_argv_secret_flags` Route. ACCURATE.
- **Wire-shape:** new fields `skip_serializing_if`; builders set None; single-`--passphrase` envelope byte-unchanged. ACCURATE.
- **Boundary/guard/version/docs:** after_help + manual keep btcrecover+URL+`2026-05-25` (`cli_help_fixtures` green); version 0.46.0 consistent across 5 sites; CHANGELOG matches; FOLLOWUP resolved + GUI pin-bump filed; flag-coverage satisfied. ACCURATE.

## VERDICT: 0 Critical / 1 Important — RED (I1). Fold → re-confirm.

---

## Fold note (applied after persisting)
- **I1 — FOLDED:** `run_candidate_scan` now takes `stderr: &mut E` and emits `note: {path} holds candidate passphrases — treat as sensitive` before the read loop (call site threads `stderr`). Runtime-verified: advisory on stderr, exit 0. New test Cell 12 locks it (`stderr contains "treat as sensitive"`).
- **M4 — FOLDED:** new Cell 11 (missing file → exit 1).
- **M1/M2/M3 — FOLDED:** manual gains the candidate-scan JSON sample, a candidate-file worked example, and the exit-codes (candidate-exhaustion exit 4) + refusals (3-way group) table updates. `make audit` GREEN.
- Affected tests (candidate 12, cli_help 4, passphrase-of-xpub 10, zeroize 2) + make audit + full toolkit suite + clippy GREEN.
- **Confirmation re-review (opus, fold `f6d38ef`): 0 Critical / 0 Important — GREEN.** I1 fold verified byte-exact to SPEC §2:31 (advisory first statement, stderr-only, path-only — no passphrase leak); no drift to match/miss/exit paths or Cells 1-10; M1/M2/M3 manual folds accurate; whole-cycle coherent. One pre-existing non-blocking Minor (exit-1 table row omits "file not found", which Cell 11 now pins). **Cleared for the v0.46.0 tag.**
