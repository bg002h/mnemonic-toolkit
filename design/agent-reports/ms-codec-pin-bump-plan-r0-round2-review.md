# R0 round-2 architect review — PLAN_ms_codec_pin_bump_0_4_2 (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 2, post-fold). master @ 6101fe0 (lock at ms-codec 0.4.2). Verdict: GREEN (0 Critical / 0 Important / 3 Minor, all non-blocking; M-a folded). Review verbatim below.

---

## Critical
None.
## Important
None. All three round-1 findings folded correctly + empirically reproduced against the bumped binary.

## Minor

**M-a (I2 under-enumeration)** — the I2 fold lists header lines `:5-6` and `:12-13`, but the module header's `:17` bullet (uppercase ms1 → ms-codec-attributed error, via silent-payment) is ALSO now false (silent-payment now derives an `sp1q…` address). The "rewrite the `//!` header" instruction is scope-correct, but add `:17` to the explicit list so a surgical implementer doesn't leave it stale.

**M-b (no defect, confirmation)** — the repair cell's distinct `RepairError::HrpMismatch` string (`repair: chunk 0 HRP mismatch — expected 'ms', found 'MS'`, :369) is GONE post-bump (repair echoes the uppercase card through, exit 0) → cell 2 genuinely flips RED, correctly in the inversion list.

**M-c (non-goal confirmation, no action)** — `docs/technical-manual/.../23-ms1-wire-format.md:203` + `53-ms-codec-api.md:176` document the `WrongHrp` rule for genuinely-wrong HRPs (`xs`≠`ms`), NOT an uppercase-ms1-rejects claim, in a separately-pinned namespace. Outside the end-user-manual mirror scope. Leave alone.

## Fold-verification

- **C1 — VERIFIED present + behaviorally correct.** Spot-check: `ms-shares combine --share MS10ENTRSQ…34V7F --to entropy` → exit 2, `error: ms1 the secret share (index 's') must not be combined …`, NO secret bytes; byte-matches the lowercase refusal. `VALID_MS1` uppercased IS a secret-at-S card (refusal names `index 's'`). Delegation `cmd/ms_shares.rs:385` → `ms_codec::combine_shares` confirmed.
- **I1 — VERIFIED.** The 4th cell stays GREEN-but-false (`ms1_decode: ok`; exit-4 from absent mk1/md1 only); name/doc(:324-326)/assert-comment(:347) now false. In the inversion list.
- **I2 — VERIFIED present;** under-enumerates by `:17` → M-a.
- **M1/M2/M3 — VERIFIED** (2 tag-gated workflows + rust/manual/technical-manual on master push; HrpMismatch string distinct + gone post-bump; 43-ms.md ms-cli ref is the sibling version, leave it).
- **Empirical recon INDEPENDENTLY REPRODUCED:** `cargo test --test cli_hrp_case_insensitive` on the bumped lock → EXACTLY 3 fail (the inversion list), 12 pass; old WrongHrp/HrpMismatch strings gone; silent-payment uppercase ≡ lowercase BYTE-IDENTICAL (`sp1qqfqnnv8cz…`); inspect exit 0 + kind:ms1/tag:entr + advisory + card not echoed.
- **SemVer PATCH well-supported** (v0.53.3 anchor; ms-codec 0.4.1/0.4.2 PATCH; advisor-at-tag retained). Version sites + FOLLOWUPS targets + manual note (41-mnemonic.md:3113-3116, preserve the still-true mixed-case-rejection clause) all verified.

## Verdict

**GREEN — 0 Critical / 0 Important.** Implementation may proceed (fold M-a's `:17` into the doc-rewrite list).
