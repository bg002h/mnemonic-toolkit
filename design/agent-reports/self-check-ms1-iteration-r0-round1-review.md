# R0 Architect Review — self-check-ms1-iteration — Round 1

> Persisted verbatim from the opus `feature-dev:code-architect` agent
> (`agentId: ac10f891303d7f474`). Had Read/Glob/Grep + advisor; verified source.

---

## VERDICT: 1 Critical / 1 Important / 2 Minor — NOT GREEN

## Critical (C1): Parity oracle is wrong — `args.slot` produces false positives on two valid bundle shapes

The SPEC's oracle (derive expected-phrase-bearing set from `args.slot` `is_secret_bearing()` subkeys) false-rejects two currently-valid `--self-check` shapes:

- **C1a — import-json envelope-sourced entropy** (`bundle.rs:1748-1784`): `bundle --import-json <envelope> --self-check` where the envelope carries `ms1[i] != ""` (from a prior `import-wallet --ms1`). The user passes NO `--slot` → `args.slot` empty → parity expects `ms1[i]==""` but it's non-empty → rejects a correct bundle. (Shape exercised by `tests/cli_bundle_import_json.rs`.)
- **C1b — wif slots** (`bundle.rs:708-750`): `SlotSubkey::Wif` IS `is_secret_bearing()` (`slot_input.rs:82-87`) but yields `entropy: None` → `ms1[i]==""`. Parity expects non-empty → false reject.

**Root cause:** `args.slot` records what the user SUPPLIED, not what drove emission. **Correct oracle:** `resolved_slots[i].entropy.is_some()` — the EXACT predicate driving `synthesize.rs:296` (`match &c.entropy { Some => emit ms1, None => "" }`). Thread `resolved_slots` (as `entropy_bearing: &[bool]`) into `self_check_bundle` at all FOUR call sites (`bundle.rs:411-414`, `:1607/1614-1615`, `:1641/1655/1657-1658`, `:1882/1907/1909-1910`). **Signature change:** `pub fn self_check_bundle(bundle, args, entropy_bearing: &[bool])` where `entropy_bearing[i] = resolved_slots[i].entropy.is_some()`.

**Bonus:** with `resolved[i].entropy` in scope, Option-2 entropy round-trip is free (no secret re-read): `ms_codec::decode(ms1[i])` → `Payload::Entr(bytes)`/`Mnem{entropy}` compared against `resolved[i].entropy.as_deref()`. The SPEC's Option-2 objection (stdin/@env re-read) is MOOT. Use it (strictly stronger).

## Important (I1): RED/guard cells are non-discriminating

The SPEC's multisig all-phrase RED cell agrees under BOTH oracles → doesn't prove C1 fixed. Required discriminating GREEN guards (Ok before and after, but Err under the WRONG impl):
- **G-A:** `bundle --import-json <seeded-envelope ms1[0]!=""> --self-check` → Ok (corrected: `resolved[0].entropy.is_some()` via line 1782).
- **G-B:** `bundle --template bip44 --slot @0.wif=<WIF> --self-check` → Ok (corrected: `resolved[0].entropy.is_none()` → expect "").

## Minor (M1): RED cell needs a SYNTHESIZED bundle, not a hand-built struct
`self_check_bundle` runs `md_codec::chunk::reassemble(bundle.md1)` FIRST (`:2031`); a dummy hand-built `Bundle` dies at md1_decode for the wrong reason → vacuous RED. Start from a synthesized valid bundle (`synthesize_descriptor`/`synthesize_full` fixtures, e.g. the ones in `synthesize.rs:1383`), then mutate `bundle.ms1[i] = String::new()`. (`Bundle`/`MsField=Vec<String>`/`MkField` ARE constructible in bundle.rs's `#[cfg(test)]` BIN unit, but md1 must be real.)

## Minor (M2): §3 Option 1/2 framing outdated
After C1, neither option as written is correct (Option 1 oracle wrong; Option 2 objection moot). Delete the framing; state the single corrected design.

## Verified Clean
- **Item 2 (parity correctness, corrected oracle):** watch-only (all entropy None → all "" expected → pass), full single-sig (pass), regressed multisig `[m0,"",""]` all-seeded (resolved[1,2].entropy.is_some → expect non-empty → "" → FAILS correctly), concrete-descriptor watch-only (all None → pass), hybrid import-json (per-slot resolved → correct). No false positive.
- **Item 3 (decode entrypoint):** `ms_codec::decode(&str) -> Result<(Tag,Payload)>` self-describing (`envelope::discriminate` picks Entr/Mnem); NO wire-language needed. Precedent `verify_bundle.rs:1316`.
- **Item 5 (SemVer):** PATCH+tag `v0.47.4` correct — `self_check_bundle` is BIN-crate pub (not the `mnemonic_toolkit::` lib API); signature add is internal; behavior strengthening.
- **Item 6 (existing tests):** verified NONE break under the corrected design — `cli_self_check.rs`, `cli_non_canonical_descriptor.rs` (×2), `cli_verify_bundle_multi_cosigner_mk1.rs`, `cli_ms1_slot.rs::ms1_mnem_self_check_round_trips`, `cli_bundle_import_json.rs::bundle_import_json_self_check_round_trip_passes` (envelope all-empty ms1 → all None → pass), and the `mnemonic-bundle-bip84-abandon.cmd` transcript (no stdout change). All pass unchanged.

## Required SPEC folds
- **§3:** delete Option 1/2; single design = `entropy_bearing[i]=resolved[i].entropy.is_some()`; per `i`: assert `ms1[i].is_empty() == !entropy_bearing[i]`; if bearing, `ms_codec::decode(&ms1[i])` succeeds AND extracted entropy == `resolved[i].entropy.as_deref().unwrap()`.
- **§4:** add G-A + G-B guards; RED cell uses a synthesized bundle then mutates ms1.
- **§5:** 5.1 corrected-oracle (not opt 1/2); 5.2 no false positive; 5.3 PATCH+tag ratified; 5.4 synthesized-bundle RED.
- **Signature:** `self_check_bundle(bundle, args, entropy_bearing: &[bool])` + 4 call sites.

After fold + re-dispatch → expected 0C/0I in one round.
