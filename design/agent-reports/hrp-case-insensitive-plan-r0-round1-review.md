# R0 round-1 architect review — PLAN_hrp_case_insensitive_probes (2026-06-10)

Reviewer: Fable 5 architect agent (R0, pre-implementation). Plan @ design/PLAN_hrp_case_insensitive_probes.md (+ ratified recon cycle-prep-recon-hrp-case-insensitive.md), source 38db912. Verdict: RED (0 Critical / 2 Important / 5 Minor). Review verbatim below.

---

## Critical

None.

## Important

**I1 — Decision 1 deletes behavior that an existing test pins by name; neither the plan nor the recon inventories the inversion.** `tests/cli_positional_hrp_autodetect.rs:404-428` `validate_flag_hrp_case_mismatch_distinguishable` runs `repair --ms1 MS1XXX` and asserts `stderr.contains("case mismatch")` (positive marker at :418-421) — exactly the repair.rs:178-187 branch Decision 1 relaxes. Post-relaxation this test REDs, but the plan's test list only enumerates NEW cells. The TDD contract must explicitly flip/replace this test (expected post-fix: `validate_flag_hrp` passes; the value reaches the repair engine → `repair_via_ms_codec` → ms-codec-attributed error; exit still 2) and rewrite its I5-history doc-comment at :396-402. Concrete fix: add one line to the plan's Tests bullet: "INVERT existing `validate_flag_hrp_case_mismatch_distinguishable` (cli_positional_hrp_autodetect.rs:404) — the I5 pin — to assert acceptance-then-codec-attribution." (Blast radius checked: `cli_env_var_sentinel.rs:565-599` exercises the TRUE-mismatch branch and is unaffected; no other test asserts "case mismatch"/"lowercase canonical".)

**I2 — silent_payment.rs:134 is a ratified fix site with zero test coverage in the plan.** No cell for `silent-payment --secret MS1…` (today: clean-but-mis-attributed refusal at :171-176, verified; post-fix: ms-codec `WrongHrp{got:"MS"}`). Per-site red-first is the house standard; add one cell asserting the ms-codec-attributed error replaces the generic refusal.

## Minor

**M1 — Adjacent full-secret echo the rider does NOT close (note, don't expand — file a FOLLOWUP).** `friendly.rs:79` `ms_codec::Error::Codex32(c) => format!("ms1 codex32: {:?}", c)` Debug-prints `codex32::Error::InvalidChecksum { string }` (constructed with the FULL input) → a known-HRP lowercase ms1 with an uncorrectable checksum echoes the full near-secret on stderr today. None of the planned cells trigger it (uppercase → WrongHrp, no echo; mixed → InvalidCase(Case, char), no echo — both verified), so the plan's assertions hold, but Decision-2's rationale is not fully delivered by the rider. Recommend a one-line FOLLOWUP (`friendly.rs` one-arm InvalidChecksum redaction, or codex32-upstream observation) recorded in the same promoted entry.

**M2 — Doc-comment inventory misses the case-enumeration block.** Decision 1 also obsoletes repair.rs:143-150 ("Three cases for non-sentinel values: … 2. Case-mismatch …") and the :121-124 "strict per-flag HRP validation" header framing. Add both to the rewrite list.

**M3 — Version-site path imprecision:** "install.sh:32" → `scripts/install.sh:32`.

**M4 — SemVer note:** PATCH defensible (BIP-173 conformance + consistency restoration), but the v0.49.0 "newly accepted input form = MINOR" precedent cuts close. Keep the advisor-confirm step non-skippable; no change required.

**M5 — verify-bundle and xpub-search positional uppercase ride solely on the shared-fn fix** (verify_bundle.rs:1242 + seed_intake.rs:129 both call classify_hrp_prefix — verified). One cheap verify-bundle positional cell would cover the surface. Optional.

## Verified claims (spot-checks, all confirmed)

- All 7 probe anchors exact at 38db912; repair.rs:312 advisory probe is the same fn (one change fixes it).
- **The ms-codec overturn is REAL:** vendored ms-codec-0.4.0 envelope.rs `discriminate` :95, `fields.hrp != HRP` :100, `share_index_byte != SHARE_INDEX_V01` :112; Codex32String stores the ORIGINAL string un-normalized, engine lowercases per-char for checksum, `set_check_case` allows Upper/rejects Mixed → consistent-uppercase passes codex32 then fails the envelope WrongHrp{got:"MS"}. Companion split correct.
- mk-codec 0.4.0 bch.rs:646-651 Mixed-only reject + to_lowercase; md-codec 0.35.0 accepts uppercase AND mixed (per-char lowercase) — consequence table accurate.
- Decision 1 safety: the true-HRP-mismatch path (:188-199) intact (`--ms1 MK1xxx` still rejects); sentinels precede; indel relax bypasses the fn. No other code keys on the branch.
- Decision 2 safety: UnknownHrp constructed only in classify_hrp_prefix; rendered only via Display; no JSON envelope serializes `got`; no test pins the full echo; truncated form stays actionable.
- Auto-fire interaction benign: uppercase ms1 → decode_card auto-fire → repair_via_ms_codec → WrongHrp → falls through on Err/empty-repairs (repair.rs:1310-1318 anticipates it). No spurious exit-5.
- GUI/friendly: no GUI consumption of these strings; friendly.rs has no UnknownHrp/HrpMismatch arms; mk MixedCase friendly arm exists (:140).
- FOLLOWUPS.md:17 index line + version sites confirmed.

## Verdict

**NOT GREEN — 0 Critical / 2 Important.** Fold I1 + I2 (+ minors), re-dispatch. The recon's load-bearing claims — including the ms-codec envelope overturn — all survived adversarial verification; the gaps are test-plan completeness, not design.
