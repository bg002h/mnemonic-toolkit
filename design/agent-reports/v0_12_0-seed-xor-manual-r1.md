# v0.12.0 Phase 3 — Seed XOR manual mirror R1 reviewer report

**Phase:** P3 — manual chapter + cli-subcommands.list mirror
**Round:** R1 round 1 (clean LOCK)
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14
**Commit under review:** `375d50d` (P3)
**Predecessor:** `455b79d` (P2 R1 LOCK)

## Verdict

**0 Critical / 0 Important / 2 Nice-to-have — R1 LOCK round 1.**

Phase E (release rollup) cleared to start.

## Scope reviewed

All 14 mandatory reviewer checks: flag-table completeness for both split
+ combine; flag-name precision; refusal-table accuracy; advisory-table
accuracy; JSON output schema accuracy; worked example reproducibility;
anchor link integrity; cli-subcommands.list invariant; no premature
version-tag attribution; glossary subcommand-count fix; style consistency;
cross-feature flow note; glossary count math.

Files reviewed:
- `docs/manual/src/40-cli-reference/41-mnemonic.md` — intro update (7→8 subcommands) + new `## mnemonic seed-xor` section
- `docs/manual/tests/cli-subcommands.list` — `mnemonic seed-xor split` + `mnemonic seed-xor combine`
- `docs/manual/src/60-appendices/61-glossary.md` — `mnemonic` entry "Seven → Eight subcommands"
- Cross-referenced against `src/cmd/seed_xor.rs`, `src/main.rs` (Command enum), SPEC §2.2/§2.3/§2.5/§2.6

## Key validations

1. **Flag tables complete + byte-faithful.** Source-of-truth enumeration:
   - `split`: `--from`, `--shares`, `--language`, `--deterministic-from-master`, `--json-out`, `--help`. Manual table covers all 6.
   - `combine`: `--share`, `--shares`, `--language`, `--json-out`, `--help`. Manual table covers all 5.
   - `value_name = "phrase=<value-or-->"` matches source byte-exact for both `--from` and `--share`.

2. **Refusals byte-faithful.** All 8 refusal entries cross-checked against
   source emit sites in `src/cmd/seed_xor.rs`. Manual uses `...` placeholders
   for dynamic-detail trailing text — acceptable tabular form.

3. **Advisories byte-faithful.** All 5 advisory texts match SPEC §2.6 and
   the source emit sites (lines 184-189, 193-199, 318-321, 432-437).

4. **JSON schema accurate.** Both `SplitJson` and `CombineJson` field orders
   in the manual sample envelopes match the serde struct declaration in
   source byte-for-byte. Sample JSON is syntactically valid.

5. **Worked example reproducible.** The `abandon × 23 art` → split N=3 →
   combine round-trip claim is exercised by
   `cli_seed_xor_happy_paths.rs::split_24_word_round_trip`.

6. **Anchor integrity.** `## `mnemonic seed-xor`` slugifies to
   `mnemonic-seed-xor`; single occurrence; no conflict in `50-comparing/`.

7. **cli-subcommands.list invariant.** Both `mnemonic seed-xor split` and
   `mnemonic seed-xor combine` byte-identical to what `lint.sh` expects;
   the script splits on first space (`bin="${line%% *}"`, `sub="${line#* }"`)
   so `sub = "seed-xor split"` correctly invokes the nested subcommand.

8. **Subcommand count math.** `src/main.rs` `Command` enum = 8 variants
   (Bundle, VerifyBundle, Convert, ExportWallet, DeriveChild, FinalWord,
   SeedXor, GuiSchema). Glossary "Eight subcommands" + intro "Eight
   subcommands" correct (+1 from v0.11.0's seven).

9. **No premature tag attribution.** "This chapter mirrors v0.12.0" is a
   version-being-documented claim, not a tag-existence claim. Compliant.

10. **Style consistency.** Section structure (Synopsis → Flags split +
    Flags combine → Worked example → JSON output → Refusals → Advisories)
    matches the v0.11.0 `final-word` precedent with the natural extension
    for the dual-subcommand grammar.

## Nice-to-have findings (non-blocking)

**N1.** Cross-feature flow note absent: plan §A mentions that `seed-xor
combine` output flows naturally into `bundle --slot @0.phrase=-` /
`convert --from phrase=-` / `derive-child --from phrase=-` /
`final-word --from phrase=-`. The manual chapter doesn't cross-reference
this. Editorial; not blocking. Could be folded at PE alongside any other
narrative polish.

**N2.** Glossary count math: this cycle's +1 (Seven → Eight) is clean.
The v0.11.0 P3 R1 noted a pre-existing drift (the original "Five
subcommands" was off-by-two against the actual surface); that drift was
fixed at v0.11.0 PE (Seven), so the v0.12.0 +1 carries forward cleanly.

## Pre-existing (out of scope)

A pre-existing broken anchor was noted at `41-mnemonic.md:131` —
`#migrating-from-bip-39-only-to-the-m-format constellation` has a space
in the slug. NOT introduced by this commit; flagged for future cleanup.

## R1 LOCK

v0.12.0 P3 R1 LOCK round 1. Phase E (release rollup: version bump 0.11.0
→ 0.12.0, Cargo.lock refresh, CHANGELOG, FOLLOWUPS resolve, Opus PE
reviewer, tag `mnemonic-toolkit-v0.12.0`, post-tag SHA refresh, 6-job CI
matrix verification) cleared to start.
