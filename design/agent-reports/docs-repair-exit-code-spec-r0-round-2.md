# R0 convergence (round 2) — `SPEC_docs_repair_exit_code_gui_manual.md` — Fable, adversarial

**Persisted per CLAUDE.md.** All 7 round-1 findings (C1 4i, I1 pinned-tier, I2 indel tier, I3 5A:11-12, M1/M2/M3) verified CLOSED against live source (4i:96-97/:123-124 confirmed; `cli_ms1_repair_demote.rs:306`=.code(5) unique-indel; 5A:11-13 second uniformity claim; pins confirmed). One fold-introduced Important:

## Findings
**Important:**
- **I-NEW-1:** the fold used toolkit version tags (v0.80/0.81, pinned v0.75.0) for ALL chapters, but each chapter documents a SPECIFIC binary with its OWN demote version + pin: `6a`/`ms repair` demotes at **ms-cli v0.14.0**, book pins **ms-cli-v0.13.0** (`pinned-upstream.toml:87`) — NOT toolkit v0.75.0; `79`/`mk repair` advisory ships **mk-cli v0.12.0**, pins **mk-cli-v0.11.0**. Implemented verbatim, 6a would carry a FALSE toolkit-version citation for the standalone `ms` binary. **[FOLDED — per-binary version mapping added under the table; header, 6a/79 loci, acceptance #2/#4 corrected; the "Pinned v0.75.0" column re-labeled "this manual's pinned tier".]**

**Minor:**
- **M-NEW-1:** `blessed`/`candidate` `--json` verdict pair is at `43-ms.md:417`, outside the cited :354-383. **[FOLDED — :417 added.]**
- **M-NEW-2:** the standalone-mk advisory wording differs ("reassemble via `mk decode`", mk-cli `cmd/repair.rs:141-143`) from the ms/toolkit shared line. **[FOLDED — 79 locus guardrail.]**

## VERDICT round 2: OPEN (0C / 1I) → folds applied, re-dispatch round 3.

---
**FOLD STATUS (opus, 2026-07-11):** I-NEW-1 (per-binary version mapping: 6a→ms-cli v0.14/pinned v0.13.0, 79→mk-cli v0.12/pinned v0.11.0, 4i→toolkit v0.80-81/pinned v0.75.0, 5A→always 5), M-NEW-1 (43-ms.md:417), M-NEW-2 (mk advisory guardrail) folded. Round-3 convergence R0 re-dispatched.
---
**ROUND 3 → 4 (opus, 2026-07-11):** R3 found I-R3-1 (acceptance #1 retained toolkit-only framing, contradicting the folded #2/#4/SPEC:22) — one-clause fold (per-binary demote-tier scoping + explicit no-toolkit-version-in-6a). **R4 VERDICT: GREEN (0C/0I)** — every version citation swept + confirmed per-binary-scoped; SPEC internally consistent, ready for impl. Docs implementer DEFERRED until v0.85.0 ships (same toolkit tree as the M2/M3/M4 implementer → serialize, avoid interleaved uncommitted diffs).
