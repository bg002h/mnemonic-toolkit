# v0.32.0 plan-doc R0 review (Cycle 14 — seedqr-compact-variant)

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** R0
**Plan:** `design/PLAN_mnemonic_toolkit_v0_32_0.md`
**Date:** 2026-05-21
**Source SHA:** `a7576d0`

## Verdict

**GREEN.** 0 Critical / 0 Important / 3 Minor refinements (none block Phase 2).

## Verifications (all pass)

1. Citations hold (modulo line-drift — see M3).
2. `SeedqrEnvelope.variant` present; BOTH emit sites (`emit_decode_output`, `emit_encode_output`) hardcode `variant: "standard"`. Plan's flip-to-dynamic accurate.
3. `encode_compact`: `to_entropy()` == SeedSigner compact payload (11-bit pack minus checksum). 12w→16B→32 hex; 24w→32B→64 hex. Equivalence holds (bip39 v2.2.2).
4. `decode_compact` byte-count check BEFORE `from_entropy_in` is CORRECT + NECESSARY (`from_entropy_in` accepts 16/20/24/28/32; compact must reject 20/24/28 with a compact-specific error).
7. GUI lockstep MANDATORY — `--variant` net-new flag on seedqr-encode AND seedqr-decode.
8. 3 error variants kept (distinct failure modes; library-local).
9. SemVer MINOR correct.

## Minor (folded)

**M1 — Use derived `ValueEnum`.** Strong project convention (`CliExportFormat`, `CliNetwork`, `BsmsForm`, `MultisigPathFamily`, slip39 enum all use `#[derive(ValueEnum)]` + `#[arg(value_enum)]`). Use `SeedqrVariant { Standard, Compact }` derived ValueEnum, NOT hand-rolled `PossibleValuesParser`. GUI gui_schema treats both identically → mirror unaffected.

**M2 — Add 3 test cells:**
- 24-word CLI happy path (plan only had 12-word CLI; add 64-hex).
- Uppercase + embedded-whitespace hex decode (risk-register claims case-insensitive + whitespace-strip but no cell asserts it).
- Standard-decode-of-64-char-hex → clean error assertion (footgun check 6).

**M3 — Citation line-drift.** Plan cites `seedqr.rs:56-96` for encode/decode; actual `:62-138`. Update per CLAUDE.md citation-decay rule.

## Footgun analysis (check 6)

Standard-decode-of-hex: a compact hex payload whose length lands in {48,60,72,84,96} AND is all-decimal-digits would mis-parse as standard. But 32/64-char compact hex never collide with the standard length set {48,60,72,84,96}, so the all-zero compact case is safe (clean `InvalidDigits`). Letters a-f fail at `InvalidDigitChar`. NOT a correctness footgun for valid compact payloads; risk-register prose ("a-f are non-decimal") is slightly optimistic — tighten to note the length-set non-collision is the actual safety property. Add the test cell per M2.

## Recommendation

Fold M1+M2+M3 (lightweight), proceed to Phase 2.
