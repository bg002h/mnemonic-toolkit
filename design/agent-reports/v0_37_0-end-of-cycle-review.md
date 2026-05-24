# END-OF-CYCLE REVIEW — v0.37.0 (final gate before tag)

**Date:** 2026-05-24
**Reviewer:** feature-dev:code-reviewer (opus)
**Scope:** full branch diff `master...HEAD` (6 commits) — code + tests + manual + release prep
**Verdict:** GREEN (0 Critical / 0 Important) — 1 non-blocking Minor

## Confirmed
- **Code correctness holds.** `template_from_descriptor` (`wallet_export/mod.rs:260`) — exhaustive over all 6 Descriptor variants (verified vs miniscript-13), `sortedmulti(`-before-`multi(` ordering handled, `Ms(_)→Err` message matches Cell 3, no panic. `format_requires_template` (`cmd/export_wallet.rs:51`) exhaustive no-`_`, true-set = exactly the `template.ok_or_else`-refusers. EmitInputs wiring: `derived_template` after taproot refusal `:643-653`; only `template`+`threshold_user_supplied` changed (14 fields untouched).
- **Release-prep aligned.** Cargo.toml 0.37.0; Cargo.lock 0.37.0; both README markers 0.37.0 (readme_version_current); install.sh self-pin v0.37.0; CHANGELOG [0.37.0] accurate; FOLLOWUP flipped resolved with accurate Resolution subsection.
- **Manual lockstep complete & runnable.** All 5 recipes stripped + runnable; specter retains `--wallet-name`; no residual `--from-import-json … --template` recipe; taproot note intact; cli-ref row updated with the correct "cannot pass --template explicitly" caveat.
- **SemVer/lockstep correct.** MINOR; Cell 5 mutex test confirms no clap flag-name change → no GUI schema_mirror lockstep needed.
- **Test integrity.** Rewritten C1 cells non-vacuous (34/6 truth table); p11d needles load-bearing; jade-singlesig literal matches `jade.rs:61`; grepped other from-import-json tests (green, taproot) — none assert now-wrong behavior.
- **CHANGELOG no overclaim** ("taproot stays walled off" matches code).
- **Conventions.** No new ToolkitError variants (alpha-order N/A); per-phase reviews persisted; commit trailers present; only `.claude/` untracked.

## Minor (non-blocking, conf 60)
- FOLLOWUPS.md `Where` line (`:3184`) cites the original-snapshot `mod.rs:211 script_type_from_descriptor`; the fix added `template_from_descriptor` at `:260`. Stale-by-design snapshot on a now-resolved entry whose Resolution subsection names the correct fn — not a misleading active citation. No action for ship.

## Disclosure
Reviewer had no shell; verified compilation soundness statically. Controller confirmed `cargo test -p mnemonic-toolkit` (795+114+… all 0 failed) + `cargo clippy --all-targets -- -D warnings` clean on this tree + `readme_version_current` green + manual lint OK.

## VERDICT
**GREEN (0C/0I).** Clear to tag/ship.
