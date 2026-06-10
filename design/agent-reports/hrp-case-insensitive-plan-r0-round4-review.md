# R0 round-4 architect review — PLAN_hrp_case_insensitive_probes (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 4, post-fold verification). Source 38db912. Verdict: GREEN (0 Critical / 0 Important / 2 wording Minors — folded before implementation). Review verbatim below.

---

## Critical

None.

## Important

None.

## Minor

**M1-r4 — Decision-2's truncation rule is still dual-form; pin one normatively.** Line 9 says "at most the pre-`1` prefix or a short head (e.g. 12 chars + `…`)" while the rider cell asserts a truncated head + `…` at a 12-char width. Every red/green claim holds under either reading, so cosmetic — but pin "rule = first 12 chars + `…`" so the TDD-red-first test writer doesn't choose.

**M2-r4 — Wording-only: the short-fixture rationale names the wrong arm.** A 6-char input dies at ms-codec's OWN rule-9 pre-dispatch gate (decode.rs:43-48 → UnexpectedStringLength, BEFORE Codex32String::from_string), so friendly renders the UnexpectedStringLength arm, not Codex32(_). Conclusion unchanged (short fixture never reaches the envelope HRP check; full-length VALID_MS1 required).

## Fold-verification

All three round-3 folds verified faithful, complete, unambiguous:
- **I1-r3 rider cell — CORRECTLY folded, every clause verified** (error.rs:796 full-`{got}` red-today ✓; the 3 existing 10-char cells assert only truncation-immune substrings ✓; lowercased `xs1…` matches no known HRP → stays UnknownHrp post-fix ✓).
- **M1-r3 disambiguation — CORRECTLY folded**; repo-wide probe sweep re-confirms the site list is EXHAUSTIVE for non-test probes.
- **M2-r3 fixtures/markers — CORRECTLY folded and independently re-derived through pinned ms-codec 0.4.0 + codex32 0.1.0 source:** decode path → friendly `ms1 wrong HRP: got "MS", expected "ms"` (inspect.rs:171 → friendly.rs:81); repair path → `repair: chunk 0 HRP mismatch — expected 'ms', found 'MS'` (parse_chunk passes the lowercased 47-char data part; decode_with_correction residue==0 → decode(ORIGINAL) → envelope WrongHrp → repair.rs:859-863 → Display :524-530). The two markers cannot be confused.

Whole-plan final scan: every cited anchor live and exact (probe sites, doc-comments, inverted-test :396-402/:403-427/:423-426, version sites incl. scripts/install.sh:32, FOLLOWUPS.md:17, CHANGELOG head). SemVer PATCH stands. No fold-drift.

## Verdict

**GREEN — 0 Critical / 0 Important / 2 Minor (wording; fold without re-review).** Implementation may begin.
