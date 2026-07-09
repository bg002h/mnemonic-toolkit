# P2 per-phase R0 review — ms1-repair-demote-to-candidate — round 2

**Verdict: GREEN (0 Critical / 0 Important)**
**Reviewer:** Fable (scoped convergence, doc-only fold), per user directive. Fold commit `02ca52ea` on `88fa3845`.
**Dispatched:** 2026-07-09 (Cycle F, per-phase P2 R0 round 2). Persisted verbatim per CLAUDE.md.

All round-1 findings folded correctly; both doc gates green; no code/transcript drift.

## Gates (reviewer-run)
`make verify-examples` → OK 62/62. `make lint` → OK (markdownlint 0 / cspell 0 / lychee 261 OK 0 err / flag-coverage + glossary + index pass). `cargo test` not re-run — fold is doc-only (`git diff 88fa3845 02ca52ea --name-only` = 4 `.md` + FOLLOWUPS + the round-1 report; zero src/transcript); round-1's 205/0 stands.

## Findings — all RESOLVED
- **I1 (5 sites) — accurate + internally consistent.** Empirically re-confirmed: `mnemonic repair --mk1 --json` = `[schema_version,kind,verdict,corrected_chunks,repairs]` vs `mk repair --json` = `[…,kind,corrected_chunks,repairs]` → toolkit strict superset (verdict after kind); `ms repair --ms1 --json` BYTE-IDENTICAL to `mnemonic repair --ms1 --json` (0-byte diff) → the retained "byte-exact only by ms-cli" clause TRUE; untouched `43-ms.md:330/:348` remain accurate (ms↔toolkit pair). 3 chapters now mutually consistent: toolkit=superset / md,mk=NO-BUMP subset (shared-field-parser compatible) / ms=exact.
- **I2 — glossary `:243-248`** qualified per kind (mk1/md1 verified=5; ms1=candidate not auto-applied). Backed by code (ms1 never short-circuits; helper requires Blessed).
- **M1 — `41:3289-3294`** unique indel re-validates full checksum → self-verifies, unlike demoted substitution.
- **M2 — `41:756-763`** principled-distinction notes mk1-single-plate exit-5 = standalone `mk repair`; `mnemonic repair` demotes incomplete mk1 → exit-4.
- **M3 — FILED** `gui-manual-repair-exit-code-lockstep` (out of Cycle F scope per SPEC §6; LOW; tier manual-gui).

## Residual-drift sweep (whole book) — clean
No residual false "exit 5 on ms1" claim (grep, 0 hits after excluding correct negations). No residual blanket "consistent across all four CLIs" parity claim — the only `exit == 5` occurrence is inside 44's new "Asymmetry vs. the toolkit" corrective heading. Round-1 review persisted verbatim.

**Gate: P2 GREEN (0C/0I). Cleared to advance to the mandatory post-impl whole-diff Fable review.**
