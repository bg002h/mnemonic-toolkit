# v0.31.6 plan-doc R0 review

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** R0
**Plan under review:** `design/PLAN_mnemonic_toolkit_v0_31_6.md`
**Date:** 2026-05-21
**Source SHA:** `0693479` (master HEAD)

## Verdict

**YELLOW.** 0 Critical / 3 Important / 1 Minor.

## Important (I)

**I1 — Substitution-point cascade explicit-ordering.** Plan correctly picks L808+ but must explicitly note that the substitution occurs BEFORE the §3/§4/§5/§7 refusal pre-checks at L811 (`targets`-parse), L837-845, L857, L868, L894, L923-939 (auto-fire), L994, L1008. After Seedqr→Phrase substitution all downstream `primary.node` checks see the substituted value. The auto-fire branch (L923-939) that drives Ms1/Mk1 repair is irrelevant for Phrase node — Seedqr decode failures returned earlier with a BadInput. Cascade is correct under L808+ placement; just needs enumeration.

**I2 — `flag_is_secret("--digits")` lockstep.** `secrets.rs:49` `SECRET_FLAG_NAMES` includes `"--digits"`. The deprecation cycle does NOT remove the secret-classification — deprecated values still leak. Plan should explicitly state: `flag_is_secret("--digits")` remains `true`. One-line acknowledgment.

**I3 — clap-level `conflicts_with` instead of runtime BadInput.** Q10 — prefer `#[arg(conflicts_with = "from")]` on `digits` (exit 2 at parse, mirrors `--passphrase` / `--passphrase-stdin` pattern). Plan currently models the mutex as runtime BadInput (exit 1). Switch to clap-level; test cell `decode_both_digits_and_from_refused` asserts exit 2 + clap error text.

## Minor (M)

**M1 — Add `--from seedqr=-` stdin-end-to-end cell on convert side.** 4 convert + 5 seedqr-decode cells total.

## Verifications passed

- Q4: `is_argv_secret_bearing` composition at L107-109 auto-flows from `is_secret_bearing`.
- Q5: `run_decode` consumers at L97/L102/L105 trivially refactor to `Option<String>`.
- Q6: `emit_secret_in_argv_advisories` at L1542-1559 iterates `args.from` filtering on `is_argv_secret_bearing` — new Seedqr auto-emits advisory.
- Q7: `secret_taxonomy_parity_with_is_secret_bearing` parity test exists at `convert.rs:1670` and WILL catch missing `"seedqr"` entry → drift gate confirmed.

## Recommendation

Fold I1 + I2 + I3 + M1 then proceed to Phase 2.
