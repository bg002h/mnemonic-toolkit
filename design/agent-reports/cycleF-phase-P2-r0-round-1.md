# P2 per-phase R0 review — ms1-repair-demote-to-candidate — round 1

**Verdict: NOT GREEN (0 Critical / 2 Important / 3 Minor)**
**Reviewer:** Fable (per-phase R0, docs correctness + lockstep), per user directive. P2 commit `88fa3845` (base `e108a8a8`).
**Dispatched:** 2026-07-09 (Cycle F, per-phase P2 R0). Persisted verbatim per CLAUDE.md.

## Gate results (reviewer-run, this worktree's binaries)
`make verify-examples` → OK 62/0. `make lint` → OK (markdownlint 0 / cspell 0 / lychee 261 OK 0 err / flag-coverage + glossary + index pass). `cargo test -p mnemonic-toolkit` → exit 0, 205 blocks, 0 fail. 4 changed transcripts byte-match live P0/P1 (6 stream comparisons). Worked-example exit codes verified live (`mnemonic repair --ms1` text/json=4; `ms repair` corrupt=4/clean=0).

## Exit-code-model accuracy — ACCURATE (verified vs shipped code + live binaries)
Demotion (Ms1 arm touched⇒Unverified `repair.rs:1152-1168`→candidate_seen→indel_exit_code→4); ms-cli 0/4/2; verify-bundle direct `repair_card` compare (match⇒rows pass, mismatch⇒`ms1_entropy_match` fail redacted `diff_byte_offset:None` full-table exit 4; watch-only skip + debug_assert); standalone fall-through advisory ms1-gated; xpub-search exit 1 (`Codex32`⇒1); D20 envelope requires Blessed→ms1-unreachable; indel carve-out kind-generic unique→5/ambiguous→4; verdict blessed/candidate. Principled distinction phrased "verified now / verifiable-by-reassembly later" (NOT "oracle-verified") in all 4 chapters; 44 documents the mk-repair(5+adv) vs mnemonic-repair-mk1(4) asymmetry. Comment-sweep COMMENT-ONLY (line-by-line diff; suite green). GUI/schema no-op (zero `#[arg]`/`#[command]` change across `ecce14a7..88fa3845`).

## IMPORTANT-1 — D27 "byte-exact RepairJson parity" claims for md/mk are now FALSE
Cycle F added `verdict` to the toolkit's SHARED `RepairJson` for ALL kinds (`cmd/repair.rs:299`, unconditional) + ms-cli (P1). md-cli/mk-cli were NO-BUMP → their envelopes still LACK verdict. Empirical: `mnemonic repair --mk1 --json` keys `[schema_version,kind,verdict,corrected_chunks,repairs]` vs `mk repair --json` `[schema_version,kind,corrected_chunks,repairs]`. Stale sites (all P2-edited): `42-md.md:308-309`("byte-exact … D27") + `:322`; `44-mk-cli.md:201-202` + `:227` + `:334`("byte-matches toolkit's RepairJson"). 43-ms.md handled it (`:416-419` documents verdict-after-kind) — 42/44 missed. **Fix:** rewrite the 5 sites → "toolkit envelope is a strict SUPERSET since Cycle F (`verdict` inserted after `kind`); md/mk standalone envelopes are unchanged — shared-field parsers still work; byte-exact parity is retained only by ms-cli." (SPEC §6 "note wire-shape" obligation; same claim-class as the exit-5 sentence this phase rewrote. Design is correct — verdict is meaningful for all toolkit kinds; the toolkit is legitimately the superset CLI. Doc-only fix, no code change.)

## IMPORTANT-2 — glossary asserts pre-Cycle-F blanket "exit code 5" auto-fire
`docs/manual/src/60-appendices/61-glossary.md:243-246` (repair entry): "Also fires automatically (auto-fire short-circuit, exit code 5) on decode failures in convert and inspect." False for ms1 post-Cycle-F (fall-through + advisory, never 5). Outside SPEC §6's 4-chapter list but INSIDE the book the commit claims to lockstep; user-facing exit-code claim the code doesn't back. **Fix:** one sentence, qualify per kind (mirror the corrected 41 auto-fire table).

## Minors
- **M1** — `41-mnemonic.md:3289` (`--max-indel`): "a unique recovery prints the corrected string (exit 5, like any repair)" — the comparative is stale (ms1 subst + mk1 partial now exit 4). The indel behavior itself correct.
- **M2** — `41` principled-distinction paragraph (`:757-767`) lists "mk1 single-plate" as an exit-5 meaning without noting only standalone `mk repair` exhibits it (this chapter's `mnemonic repair` demotes it to 4); 43 "(mk1/md1 cross-chunk structure)" compresses (md1 verified-now not later). 44 disambiguates; a one-parenthetical cross-ref in 41 removes the tension.
- **M3 (process)** — file the `docs/manual-gui/` stale-sentence FOLLOWUP before ship (per followup-status discipline).

**Gate: fold I1 (5 sites) + I2 (glossary) + Minors, re-run doc gates, re-dispatch scoped convergence round before the whole-diff. ~6 sentences across 3 files; no code change.**
