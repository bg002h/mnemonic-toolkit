# R0 Architect Review (round 1) — SPEC_xpub_search_passphrase_candidates_file.md

**Reviewer:** opus `feature-dev:code-reviewer`. **Date:** 2026-06-05. **Branch:** `xpub-search-passphrase-candidates-file` (master `86a59bb`).
**Verdict:** **0 Critical / 3 Important — RED.** (3 Minors.) Re-dispatch after fold.

> Persisted verbatim per CLAUDE.md before folding. All 3 Importants + M1 independently re-verified at source by the orchestrator (`passphrase_of_xpub.rs:282-289`; `error.rs:785-789`; `mnemonic-gui/src/schema/mnemonic.rs:2133-2134`; `main.rs:147` + `cli_xpub_search_passphrase_of_xpub.rs:92,105,129`).

---

**Transparency on binary runs:** reviewer env had no shell — all source-derived. On exit codes + flag set, existing integration tests + clap-override source are stronger than a run.

## CRITICAL
None.

## IMPORTANT

### I1 — §3 omits guarding the existing inline passphrase-resolve block; candidates-file mode falls into the "unreachable" `else` and BadInputs on every invocation
`passphrase_of_xpub.rs:282-289`: when `passphrase_stdin == false && passphrase == None` (exactly candidates-file mode) control hits the `else` → `return Err(BadInput("requires --passphrase or --passphrase-stdin"))` BEFORE any scan runs. The block's safety comment (":283-285", "clap's required_unless_present pair makes this unreachable") is invalidated by §2 widening the mutex to 3-way. SPEC §3 adds a `passphrase_search.rs` but never says to branch the inline resolve at :260-289 first → an implementer ships a feature that BadInputs on every `--passphrase-candidates-file` call.
**Fix:** §3 must state the dispatch: when `passphrase_candidates_file.is_some()`, route to the scan engine and SKIP the single-passphrase resolve+derive+match at :260-318 entirely. Update the :283-285 comment.

### I2 — §4 reuses `XpubSearchNoMatch` → cross-channel count/message contradiction
`ToolkitError::XpubSearchNoMatch { mode, searched }` Display (`error.rs:785-789`) is hardcoded: *"no match in searched set: mode={mode}, paths searched={searched}; widen the range with --max-account / --number-of-accounts, or supply additional templates via --add-path"*. §4 keeps `searched_count`=path count (~80) in the JSON body but sets `XpubSearchNoMatch.searched = candidates_tried` for stderr → for one invocation JSON says `searched_count:80` while stderr says "paths searched=1000." Empty-file (`searched:0`) prints "paths searched=0; widen --max-account" — nonsense for "empty candidate file."
**Fix:** reconcile, don't just add a field. Either (a) distinct `mode` + a scan-specific Display arm with scan-appropriate advice, or (b) a NEW error variant for scan-exhaustion (don't overload `.searched`). Empty-file needs its own clean message.

### I3 — `--passphrase-candidates-file` secret-classification unspecified; SPEC's "SENSITIVE" wording steers toward the wrong `secret:true` (would trip `lint_argv_secret_flags.rs` with no anchor)
SPEC never addresses `secrets.rs::flag_is_secret` / the gui-schema `secret` bit / the `lint_argv_secret_flags.rs` axis-1 closure. `tests/lint_argv_secret_flags.rs` set-equals declared `FLAG_ROUTES` against gui-schema flags with `secret==true && kind!="boolean"`; marking the flag `secret:true` (per §2's "SENSITIVE") FAILS the closure until a Route is added — but a Route needs a non-argv evidence anchor (`*-stdin`/`=-`/`@env:`/refusal), and a FILE flag has none. Established convention is the OPPOSITE: PATH flags are `secret:false` — `mnemonic-gui/src/schema/mnemonic.rs:2133-2134` ("`--decrypt-password-file` holds a PATH (non-secret)"), `:3042` (`--secret-file` "a plain path"), both `FlagKind::Path { stdio_sentinel:false }, secret:false`.
**Fix:** §2/§6 state explicitly: `--passphrase-candidates-file` is `secret:false`, `FlagKind::Path { stdio_sentinel:false }`, mirroring `--decrypt-password-file`/`--secret-file`; `flag_is_secret` NOT extended; NO lint Route. Keep file-sensitivity in help text + a runtime advisory only.

## MINOR
- **M1 — §7 clap errors exit 64, not 2.** `main.rs:147` overrides clap default 2→64; existing tests pin it (`cli_xpub_search_passphrase_of_xpub.rs:92,105,129` `.code(64)`). New mutex/required tests must assert 64.
- **M2 — §3 perf claim understates.** Per candidate = PBKDF2 (`derive_master_seed`) + `Xpriv::new_master` + a full `match_xpub_against_paths` walk over `searched_count` paths (default 80). PBKDF2 dominates but cost scales `candidates × searched_count`; a wide `--number-of-accounts` multiplies whole-file runtime. Document.
- **M3 — §5 citation drift.** `main.rs:51` is the const decl; the "cannot brute-force" text is `:54` (body :51-62). Manual `41-mnemonic.md:23` ACCURATE. `cli_help_fixtures.rs:34-38` asserts `btcrecover`+URL+`2026-05-25` — ACCURATE; refinement stays green iff those retained.

## What verified clean
- Oracle symbols (`derive_master_seed -> Zeroizing<[u8;64]>` derive_slot.rs:31; `match_xpub_against_paths -> Option<MatchedPath{template_name,path,account}>` path_search.rs:16-49) — ACCURATE; the parse-mnemonic-once + re-derive-per-candidate loop is faithful.
- Mutex today (pairwise, :78-92) ACCURATE; ArgGroup(required, single) over the 3 is the lower-drift option; seed-source group is independent (no conflict-graph entanglement); FILE flag = no stdin contention (seed-via-`--phrase-stdin` coexists) — ACCURATE.
- Wire-shape: `Match`/`NoMatch` :172-198 real; optional fields back-compat; NOT schema_mirror-gated (flag-NAME gate only, per `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification`) — ACCURATE.
- GUI lockstep: `xpub-search-passphrase-of-xpub` schema'd (mnemonic.rs:2681,3642); `--passphrase-candidates-file` is the only addition; pin-blocked→FOLLOWUP mirrors resolved `gui-restore-multisig-flags-pending-pin-bump` — ACCURATE.
- Line-handling/NFKD: strip only trailing `\n`/`\r`, blank-skip, exact-bytes — correct; scan inherits the identical `derive_master_seed→to_seed` NFKD as the `--passphrase` path (no new footgun). Secret-output posture (line-number default, passphrase in `--json`) coherent IF the hygiene test checks ONLY the default-text path (not `--json`). SemVer MINOR defensible.

## VERDICT: 0 Critical / 3 Important — RED. Fold I1/I2/I3 + M1/M2/M3, re-dispatch.
