# P1 per-phase R0 review — ms1-repair-demote-to-candidate — round 1

**Verdict: NOT GREEN (0 Critical / 1 Important / 2 Minor)**
**Reviewer:** Fable (per-phase R0 over the code + binary battery), per user directive. `mnemonic-secret` worktree @ `9552700` (base `c2fd4eb`).
**Dispatched:** 2026-07-09 (Cycle F, per-phase P1 R0, FULL ms-cli suite). Persisted verbatim per CLAUDE.md (cross-repo audit trail in toolkit).

The demotion is correct, complete, funds-safe — every attack cell run against the real binary behaves per SPEC §4. The one Important is a user-facing exit-code-doc contradiction no other phase owns.

## IMPORTANT-1 — `ms repair --help` / man page still documents "exit 5 = REPAIR_APPLIED" (twice)
`crates/ms-cli/src/main.rs:129` + `:134-136` — the `Repair` subcommand clap doc-comment reads "(exit 5 = REPAIR_APPLIED)" + "Exit 5 on correction-applied (D26)". Confirmed live: `ms --help` + `ms repair --help` show exit-5 while the binary exits 4; this feeds `ms gen-man` → `ms-repair.1` (shipped as a release asset at `ms-cli-v0.14.0`). This is the funds-MESSAGING surface (a corrected ms1 is an unverified candidate, not "repair applied"), and it falls through the plan's cracks: P1 cited only `cmd/repair.rs:16-22` (updated); P2 is toolkit `docs/manual/*` only; no phase after P1 touches ms-cli source (release is version-bump-only). **Fix (2 lines, same phase):** `(exit 5 = REPAIR_APPLIED)` → `(exit 4 = VERIFY-ME candidate)`; `Exit 5 on correction-applied (D26)` → the Cycle-F demotion wording. No test asserts this string (`main.rs:129` is the only live `REPAIR_APPLIED`).

## Minors
- **M-1** — `cli_repair.rs::repair_json_envelope_shape` pins `kind<verdict<corrected_chunks` but not `schema_version` first; the D27 invariant is the full 5-field order. Add a `raw.find("\"schema_version\"")` assertion (coverage, not behavior).
- **M-2** — `cmd/repair.rs:143-144` comment says the advisory "Mirrors … byte-for-byte"; the reason BODY is byte-identical to toolkit `src/repair.rs:1166-1168` but ms-cli prepends `repair: `. Reword to "reason text (prefixed `repair: `)". Cosmetic.

## Funds-attack results (run against built `target/debug/ms`) — ALL SAFE
1-subst text → exit 4 + UNVERIFIED/BIP-93 advisory + corrected on stdout; 1-subst `--json` → exit 4 `verdict:"candidate"`; clean → exit 0 NO advisory; clean `--json` → exit 0 `verdict:"blessed"` `repairs:[]`; uncorrectable (8 flips) → exit 2; stdin `-` → exit 4. **Exit 5 unreachable** (single success return `Ok(if any_correction {4} else {0})` @:153; errors via `?`→D26). Advisory fixed-text (`eprintln!` @:146-150, zero interpolation).

## D27 byte-match — VERIFIED
ms-cli `RepairJson` (`cmd/repair.rs:234-244`) = `schema_version, kind, verdict, corrected_chunks, repairs` — identical to toolkit P0 (`mnemonic-toolkit-cycleF/…/cmd/repair.rs:284-297`); live JSON confirms wire order. `RepairJsonDetail`/`Position` match; `verdict` `"candidate"`/`"blessed"` match. NO `IndelJson` in ms-cli. Advisory reason body byte-identical to toolkit engine.

## Secret-hygiene — PASS
`original`/`corrected_chunk`/`corrected_chunks`/serialized JSON all `Zeroizing` (cycle-15 Lane M, unchanged). New surface = only `&'static str` verdict + static advisory; no seed interpolation, no new Debug, no new leak. Corrected-on-stdout is pre-existing deliberate D9.

## Flipped/new tests — correct; collateral/NO-BUMP — PASS
3 flips encode SPEC §4 `ms repair` row (0/4+advisory/2), reproduced independently; new clean cells genuine (assert advisory ABSENT — G1). `cargo test -p ms-cli` → **225 passed, 0 failed, 5 ignored**; clippy clean. Changed files = 2; no `ms_codec` change; no version/Cargo delta; **mlock.rs byte-identical to base** (g6).

**Gate: fold IMPORTANT-1 (2-line main.rs help fix) + Minors, re-run full suite, re-dispatch round 2.**
