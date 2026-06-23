## R0 Review — P0a MECHANISM spec (`SPEC_cycleC_P0a_mechanism_2026-06-23.md`)

**Verdict: GREEN — 0 Critical / 0 Important / 5 Minor.** Gate may proceed to implementation. All Minors are citation/contract-hardening polish, none block coding; fold them into the spec before/with the sample-proof commit.

Verified live at `737ff5ffb582745df607a3db2c9cc6051fadece8`. Beyond reading the cited source I **prototyped the M1 filter and reproduced every mechanism leg end-to-end** (whole-include, contiguous excerpt, fail-closed ×3, deliberate-drift RED, include→wrap-long-code composition in the LaTeX writer) against the real binaries and real filters. The mechanism is sound by construction.

### (a) Does M1 make rendered-prose == golden-.out BY CONSTRUCTION, in both `make html` and `make pdf`? — YES
- The filter walks `CodeBlock`, reads `el.attributes["include"]`/`["lines"]` from the AST, reads the `.out` from `TRANSCRIPTS_DIR`, and **replaces the placeholder body wholesale** → the `.md` placeholder is discarded, so prose==.out is structural, not hand-maintained. Prototype confirmed: whole-include renders the full body; `lines="2-21"` / `lines="2-4"` render exactly the slice, in BOTH the html and latex writers.
- Wiring runs in both pipelines: `make html` uses `MD_FILTER_ARGS` (Makefile:146); `make pdf`→tex uses `PDF_FILTER_ARGS` (Makefile:182). The two are independent assignments (Makefile:80-81) — the spec correctly prepends to BOTH. `primer-box.lua` only handles `Div` (verified — line 14), so zero collision with the CodeBlock include path; the implementer's "confirm no sentinel collision" residual is provably a no-op.
- Excerpt mechanism is unambiguous and SUPERIOR to the PLAN's substring-gate fallback: `lines="N-M"` keeps excerpts structural too. `TRANSCRIPTS_DIR` propagation mirrors the proven `MERMAID_CACHE_DIR` `export` pattern (Makefile:90) — sound.

### (b) Is prose==binary transitivity real, or is there a drift gap? — REAL, with one narrow gate-invisible edge
- prose==.out (M1, structural) ∧ .out==binary (verify-examples replay) ⟹ prose==binary. The `.out`==binary leg is currently GREEN: replaying all 20 manual `.cmd` against the absolute current-tier bins yields `[verify-examples] OK (20 transcripts pass)`.
- **Gap (Minor #2):** verify-examples compares via `$(...)` (strips ALL trailing newlines); M1 strips exactly ONE. A `.out` with 2 trailing newlines passes the gate yet renders a spurious blank line. Moot for the 3 sample blocks (each ends in exactly one LF — xxd-verified; the binary also emits one LF) but under-specified as an all-books contract.

### (c) Does the gate change make verify-examples RUN in CI for all 4 books with the right binaries? — YES for manual (P0a's only surface); other 3 correctly deferred
- `manual.yml` already `cargo install`s mk-cli-v0.10.2 (79) / md-cli-v0.9.2 `--features cli-compiler` (86) / ms-cli-v0.11.0 (90), builds `mnemonic` debug (96), runs `make audit` (104 — = lint + verify-examples + anchor-check) with absolute `MNEMONIC_BIN` (105) + `FIXTURES_DIR` (109) and bare `md`/`ms`/`mk` (106-108, which resolve to the cargo-installed `~/.cargo/bin`). `make pdf` runs as a separate step (115). So once M1 lands, manual's prose is gated transitively. P0a correctly scopes quickstart/tech-manual/manual-gui CI to P2/P1a/P4. Off-by-one citation noted (Minor #1).

### (d) Determinism — clean
No version/date/abspath/epoch in the sample `.out` (grep-verified); fixed public vectors (fp `73c5da0a`); the argv/spend advisories are literals; the C fixture is hand-authored static.

### (e) Migrating the existing goldens — no risk
**0** existing fences use `include=`/`lines=` (grep-verified) → adding the filter cannot retroactively process any current prose. P0a converts only 2 sample chapters to excerpts, both **provably correct**: `22-first-bundle.md:52-71` == `.out` lines 2-21; `24-recover.md:66-68` == `24-recover-mk1.out` lines 2-4; the md1 fence == `24-recover-md1.out` line 1. **Finding 1's re-classification is correct and load-bearing:** both 24-recover blocks ARE excerpts (drop leading SLIP-0132 / trailing watch-only notes); a bare whole-include would silently add those notes. The purpose-built C fixture for the whole-include leg is the right call.

### (f) Sample-proof sufficiency (incl. deliberate-drift RED) — sufficient; one cosmetic asymmetry
Reproduced independently: whole-include + excerpt render (html + latex), fail-closed on missing-target / unset `TRANSCRIPTS_DIR` / out-of-range `lines=` (all exit 1 with FATAL), include→wrap-long-code composition (the 105-char xpub run split after 64 chars in LaTeX), and the deliberate-drift RED (`73c5da0a`→`DEADBEEF` ⟹ `[verify-examples] FAIL ... 17c17`). The proof validates BOTH M1 code paths + both pipelines + the gate. Two polish gaps: the smoke's MD leg renders the shared C fence without the include filter loaded (Minor #3), and §5's local-replay recipe omits the load-bearing `FIXTURES_DIR` (Minor #4).

### Folds confirmed sound
All four R0-folds verified against live source: §5 excerpt re-classification (Finding 1) is correct and the most important fold; filter-smoke PDF leg loads only primer-box (Finding 2, line 77 — confirmed); 20 not 35 manual goldens (Finding 3 — confirmed 20 `.cmd`); bare-md-is-CI-correct (Finding 4 — claim correct, line numbers off-by-one).

**Recommendation:** GREEN. Fold the 5 Minors (3 are one-line citation/recipe fixes; 2 are mechanism-contract hardening for P0b) and proceed to the sample-proof TDD. Per project convention, re-dispatch a scoped convergence review after the fold (folds can introduce drift) before declaring the spec converged.