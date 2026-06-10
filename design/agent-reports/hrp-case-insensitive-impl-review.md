# Implementation review — toolkit v0.53.3 HRP case-insensitive probes (2026-06-10)

Reviewer: Fable 5 implementation-review agent (post-impl, pre-commit; re-dispatched once after a timeout). Plan @ design/PLAN_hrp_case_insensitive_probes.md (R0 GREEN r4). Verdict: GREEN (0 Critical / 0 Important / 3 Minor — ALL folded post-review: manual mixed-case qualification, sibling companion invert-the-cells wording, the first-vs-last separator comment). Review verbatim below.

---

## Critical

None.

## Important

None.

## Minor

1. **Manual note overclaims uniform mixed-case rejection — contradicted by this cycle's own characterization test.** `41-mnemonic.md:3114-3117` "still reject mixed-case input" is false for md-codec (the `inspect_mixed_case_md1_accepted_characterization` cell proves acceptance). Lenient-direction only; qualify to "mk1/ms1 reject mixed-case; md1 is lenient".
2. **Sibling companion wording could misdirect the future pin-bump cycle:** "flips the uppercase-ms1 leg green (test cells already staged)" — the staged cells pin the ERRORS; a pin bump turns them RED until inverted. One clause makes the hand-off exact.
3. **Carried-over comment says "first `1` separator", code uses `rfind` (last).** Behavior correct per BIP-173; the comment block was edited this cycle so the wrong word was re-affirmed.

## Verdict

**GREEN** (0 Critical / 0 Important).

- **Probe sites (7) + relaxation + rider + doc-comments:** all probe-only (original string to codecs): repair.rs:110 classify_hrp_prefix (covers positionals + verify_bundle.rs:1242 + seed_intake.rs:129 + the M3 advisory :309 transitively), restore.rs:1028, target_intake.rs:26, address_of_xpub.rs:178, descriptor_intake.rs:156, cmd/silent_payment.rs:138, seed_intake.rs:207-213 cosmetic. validate_flag_hrp relaxation keeps true-HRP-mismatch rejection (D34 cells still pass). UnknownHrp truncation char-safe, >12-only; the 3 existing 10-char cells stay green. Doc-comments rewritten; residual-probe sweep clean.
- **Tests:** 15 new + 1 inverted, every plan bullet mapped incl. both pinned markers, the full-length VALID_MS1 fixture, the 51-char rider cell, the inversion's independent advisory assert. **TDD integrity:** scratch-reverting only the classify_hrp_prefix lowercase sent exactly the 8 classifier-dependent cells RED (7 other-site cells green — discriminating); restore sha256-identical (`30392b89…`).
- **Ritual:** CHANGELOG claims verified; version at Cargo.toml/Cargo.lock/both READMEs/scripts/install.sh:32; FOLLOWUPS index flip + promoted entry + spawned friendly-echo entry (cite exact); sibling companions accurate (ms-codec envelope.rs:100/:112 grep-confirmed; md observation filed).
- **Gates:** workspace 157 suites / 0 failures; clippy clean; FULL manual lint vs rebuilt release `mnemonic 0.53.3` + pinned siblings → all 6 stages OK.
