# R0 round-3 architect review — PLAN_hrp_case_insensitive_probes (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 3, post-fold verification). Source 38db912. Verdict: RED (0 Critical / 1 NEW Important I1-r3 — the decision-2 rider had no test cell / 2 Minor; all round-2 folds verified faithful, the inverted-test oracle empirically airtight both directions). Review verbatim below.

---

## Critical

None.

## Important

**I1-r3 — The decision-2 rider (UnknownHrp echo truncation, error.rs:794-800) has NO red-first test cell; it would ship untested.** The plan's only echo assertion ("uppercase MS1 positional → … NO full-string echo") rides the *classification* fix: post-fix an uppercase ms1 is classified Ms1 and never reaches `UnknownHrp` at all — the rider's truncation is reachable only via a *genuinely unknown* HRP, which no listed cell supplies. The three existing `UnknownHrp` tests (`tests/cli_positional_hrp_autodetect.rs:128/:232/:447`) all use the 10-char `abc1xxxxxx` and assert only the "does not begin with a recognized HRP prefix" substring — under a "≤12 chars + `…`" truncation they stay green without ever exercising it. Fix, one clause: add a rider cell — long unknown-HRP positional (e.g. a 51-char `xs1…` secret-shaped string) → stderr does NOT contain the full string, DOES contain the truncated head + `…`, still contains "does not begin with a recognized HRP prefix", exit 2. Red today (error.rs:796 formats `'{got}'` in full); green only via the rider.

## Minor

**M1-r3 — `silent_payment.rs:134` is a colliding basename; disambiguate to `cmd/silent_payment.rs:134`** (src/silent_payment.rs:134 is BIP-352 test-vector code — the technical-manual G2 colliding-basename class).

**M2-r3 — The two ms-codec-attribution cells should pin full-length fixtures.** (a) the positional-MS1 inspect cell: a short `MS1XXX` dies inside codex32 on LENGTH before the envelope HRP check → friendly renders the Codex32 arm, never the WrongHrp marker; (b) the `--ms1 MS1… reaches ms-codec` relaxation cell needs a full-length value to pass the :785 pre-gate; its marker is the toolkit translation `repair: chunk 0 HRP mismatch — expected 'ms', found 'MS'` (mapping repair.rs:859-863, Display :524-530), NOT the friendly string. Fix: name `uppercased VALID_MS1` (repair.rs:1483) as the fixture for both + pin the repair-path marker.

## Fold-verification

All four round-2 folds verified faithful, complete, unambiguous at 38db912:
- **I1-r2 (option a) — CORRECTLY folded + empirically re-verified BOTH directions:** today `repair --ms1 MS1XXX` → exit 2 + `case mismatch` (inverted assert RED today); the lowercase twin (the exact post-relaxation path) → exit 2 + `repair: chunk 0 parse failed before correction could run: data-part length 3 is outside BIP-93's valid range [14, 93] ∪ [96, 108]`, no `case mismatch` (GREEN post-fix without either forbidden shortcut). Marker string exists at repair.rs:545.
- **M1-r2 — folded** (vacuity re-confirmed: post-relaxation stderr carries `--ms1` via the argv advisory line).
- **M2-r2 — folded** (inspect routing verified: inspect.rs:171 → friendly.rs:81 renders exactly `ms1 wrong HRP: got "MS", expected "ms"`). Residual fixture nit → M2-r3.
- **M3-r2 — folded** (:403-427 exact).

Whole-plan re-scan: every cited anchor live and exact (full list in the round-3 transcript): probe sites, doc-comments, advisory :310-315, error.rs:794-800 (full-`{got}` echo re-confirmed), verify_bundle.rs:1242, friendly.rs:79/:81, VALID_MS1 :1483, scripts/install.sh:32, FOLLOWUPS.md:17, the recon doc. SemVer PATCH stands. No fold-drift.

## Verdict

**NOT GREEN — 0 Critical / 1 Important / 2 Minor.** Fold the rider cell + the two minors; round 4 should be a fast GREEN.
