# POST-IMPL WHOLE-DIFF REVIEW — Cycle E (mk1-repair-set-level-reverify) — round 1

**Verdict: GREEN (0 Critical / 0 Important)** — 1 Minor (test-fixture only, FOLDED post-review) + a release-sequencing reminder.
**Reviewer:** FRESH independent opus (cold read across BOTH repos). Toolkit `011eeb5d..57b5783e`; mk-cli `85bca69..71c2b31`.
**Dispatched:** 2026-07-07 (Cycle E, mandatory post-impl whole-diff). Persisted verbatim per CLAUDE.md.

The sequenced release (mk-cli tag → crates.io → toolkit v0.80.0) can proceed.

## Independent counts (both repos)
Toolkit `cargo test -p mnemonic-toolkit`: run 2 = all 204 result-lines "0 failed" (run 1 had the documented g4_a mlock page-straddle FLAKE — `mlock.rs:429`, NOT in this diff, passes in isolation + on re-run). New reverify suite 16 passed/2 ignored; ignored slow harness passes. Clippy `-D warnings` clean. mk-cli `cargo test -p mk-cli`: all pass incl. `cli_mk1_repair_reverify` 10/0. Clippy clean.

## Funds attacks — each SAFE
1. Toolkit `mnemonic repair` full-set miscorrection → `verify_mk1_set`→`Err(SetReassemblyMismatch)`→`cmd/repair.rs:225` `Err(e)=>Err(e.into())`→`ToolkitError::Repair→2` (error.rs:624), "NOT trustworthy" message (cell_4_1b).
2. Toolkit auto-repair → `Err`→`try_repair_and_short_circuit` (repair.rs:1682-1685) `Ok(())` no short-circuit; original decode error surfaces (exit 2), no report/kind (cell_4_1d convert, cell_4_5 inspect).
3. mk `mk repair` full-set miscorrection → `classify_mk1_set`→`Err(CliError::SetReassemblyMismatch)` via `?` BEFORE emit → exit 2, corrected string never printed (incl `--json`).
4. Candidate mis-blessed → `Unverified` never short-circuits (gate `!Blessed=>Ok(())` repair.rs:1701); `mnemonic repair`→exit 4+advisory; `mk repair`→exit 5+advisory. Never confident recovery.
5. decode-Err variant not caught → invariant is BLESS iff `decode==Ok`, reject on ANY Err (+ touched-chunk header-parse Err folds to Reject). Not an allowlist.
6. Batch/multi-group fold → `fold_verdict` reject>candidate>bless; dominant Reject returns `Err` before emit, suppresses ALL output (cell_4_5b + mk-cli batch both orders).
7. Caller bypass → `verify_mk1_set` unconditional in Mk1 arm via `?`; the only bless-path requires `set_verify==Blessed` (≡ `decode==Ok`). No bypass.

## Cross-repo parity: HOLDS
`classify_mk1_set` (mk-cli) and `verify_mk1_set` (toolkit) share byte-identical `group_is_complete_and_consistent`/`fold_verdict`/`describe_group_key`/`GroupKey`/`GroupVerdict`. Both group by corrected csid, skip untouched, gate complete-and-consistent, decode the exact corrected group, fold reject>candidate>bless. Same mk-codec source (toolkit `mk-codec="0.4.1"` crates.io == in-repo 0.4.1 @ main@85bca69). Neither blesses where the other rejects.

## MINOR (test-fixture only) — FOLDED post-review
Pinned `CORRUPTED_MK1_CHUNK1` differed toolkit vs mk-cli at position 50 (`6` vs `y`) while the mk-cli comment claimed "COPIED VERBATIM … aliases IDENTICALLY" — a false provenance claim (transcription slip). NOT a funds hole / behavioral divergence: the mk-cli string genuinely reaches the reject path (asserts `does not reassemble` — produced ONLY by `SetReassemblyMismatch`, i.e. BCH-corrected to a valid codeword, complete-and-consistent group, failed cross-chunk reassembly = the exact miscorrection class). Both repos non-vacuously prove the funds property. **FOLD (mnemonic-key `f561542`):** re-synced mk-cli to the toolkit's canonical bytes; reverify suite 10/10 green (same mk_codec → aliases identically). Single canonical seed now.

## P2 docs lockstep: ACCURATE
7.2e-5 rate confirmed reproducible — reviewer ran the ignored slow harness (N=1e6, seed 0x4634_5F31): 58 hits → 95% Clopper-Pearson upper bound 7.218e-5, exact match to the manual's cited bound. Exit codes 2/4/5, tri-state, single-plate UNVERIFIED advisory, batch reject-dominance, BIP-93, md1/ms1 notes in `44-mk-cli.md:230-289` + `41-mnemonic.md:3040-3099` all accurately describe the code. Regenerated `.err` transcripts carry the advisory line byte-matching the code string, correct order; `.out`/`.cmd` unchanged. No doc claim the code doesn't back.

## Release-readiness: CONFIRMED
Codecs genuinely NO-BUMP (no mk/md/ms-codec source; discriminator uses existing public API). `rand` dev-dep-only (single Cargo.lock line, already-resolved `rand 0.8.6`, zero prod/vendor impact). No clap-surface change → no schema_mirror/GUI owed. Only breaking behavior = exit-code contract (full-set wrong-fit 5→2 both repos) → MINOR both. ms1/md1 arms unaffected (`Blessed` hardcoded; `is_indel_trigger` excludes `SetReassemblyMismatch`, unit-test-locked). Clean/≤4/uncorrectable unchanged.

**Release-sequencing reminder (SPEC §6, not a defect):** this branch regenerated the `44-mk-*` transcripts to the NEW mk output but does NOT yet advance the 5 mk-cli sibling-pin refs. The toolkit release commit MUST advance them in lockstep — else `verify-examples` runs the OLD pinned `mk` (no advisory) and the examples gate fails. Sequence: mk-cli tag → crates.io → toolkit pin-advance + self-bump + tag.

**GREEN — proceed with the sequenced release.**
