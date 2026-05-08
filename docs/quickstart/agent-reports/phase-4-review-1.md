# Phase 4 — spec-compliance review, round 1

**Date:** 2026-05-08
**Branch:** `quickstart/v0_1` at commit `12e344a`
**Reviewer:** feature-dev:code-reviewer (spec-compliance focus)
**Verdict:** SPEC_COMPLIANT

## Checks passed

**H1 titles.** All four match plan Body column exactly.

**Mermaid block (ch 42).** `42-multisig-watch-only.md` lines 12-42: `flowchart LR` with three cosigner subgraphs feeding xpubs (not phrases) to the coordinator bundle node. Output nodes are 3 mk1 + 1 md1. Correct.

**Ch 41 commands.** `mnemonic convert --from phrase=… --to xpub --template bip84 --network mainnet` matches `mnemonic-convert.txt` flag grammar. `mnemonic bundle --network mainnet --template bip84 --slot @0.xpub=…` matches `mnemonic-bundle.txt`. Both correct.

**Ch 42 watch-only multisig.** Uses `--template wsh-sortedmulti --threshold 2 --slot @N.xpub=…` exclusively; no `--slot @N.phrase=` anywhere; output described as "4 cards: 3 mk1 + 1 md1, no ms1 cards". Correct.

**Forward-pointer chain.** 41→42 (line 88), 42→51 (line 125), 51→52 (line 87), 52→manual ch 67 (lines 4-5 and 118-120). All present.

**Manual chapter slugs.** Verified all 21 linked targets against the filesystem:
- `30-workflows/31` through `38` — all 8 exist.
- `40-cli-reference/41` through `44` — all 4 exist.
- `50-comparing/51` through `57` — all 7 exist.
- `60-appendices/62`, `63`, `64`, `65`, `67` — all 5 exist.
Zero dead links.

**Relative paths.** All cross-chapter links use `../../../manual/src/…` from depth-4 quickstart files — resolves correctly to `docs/manual/src/…`.

**Ch 52 five items.** All five plan-listed troubleshooting items present. Forward-pointer to manual ch 67 present at lines 4-5 and 118-120.

**Ch 41 Reminder cross-reference.** Lines 20-23: `> **Reminder.** … See [Generating entropy safely](../20-singlesig/22-generate-entropy.md).` — satisfies plan Task 4.5 step 3 / spec §5:126.

**Spec D2 coverage.** Single-sig WO (ch 41) + multisig WO (ch 42) both present.

**Task 4.3 BIP primers.** Ch 51 covers chs 62-65 (4 primers).

## Notes (non-blocking)

- Ch 52's `## When in doubt` section (lines 108-120) adds a three-point general guidance block not prescribed by the plan. Additive and consistent with newcomer voice.
- Ch 42's `--multisig-path-family bip48` aside (lines 65-68) accurate per `mnemonic-bundle.txt`; improves newcomer coverage for Coldcard/SeedSigner users.

## Verdict

**SPEC_COMPLIANT.** All structural and CLI-flag requirements met.
