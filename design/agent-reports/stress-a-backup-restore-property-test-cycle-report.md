# Stress Cycle A — backup→restore property test — CYCLE REPORT

**Shipped:** 2026-06-11, NO-BUMP (test + `proptest` dev-dep only). SPEC + R0 ×2 GREEN.
**Deliverable:** `crates/mnemonic-toolkit/tests/prop_backup_restore_roundtrip.rs`.

## What it does
A `proptest` property over a typed-template generator of valid, reconstructable wallet policies (10 schemas, all 13 fragment keywords covered, fresh-key allocator, one timelock class/tree): `build-descriptor --spec -` → `bundle --descriptor` (concrete watch-only) → `restore --md1 --json`, asserting three independent oracles — O1 structural AST-modulo-keys, O2 md1 fixed-point, O3 address differential from the ORIGINAL descriptor (rust-miniscript). The worst outcome (green-but-vacuous) is foreclosed by: `max_global_rejects: 8` + `generator_covers_all_fragments`, the 5 permanent oracle self-test cells, and O3's script-hash equality. Default 64 cases (`PROP_CASES` env override).

## Bring-up proof (R0-r1 I3 — the harness catches the class it exists for)
Temporarily reverted the v0.54.0 faithful-arm routing (`cmd/restore.rs::run_multisig` `None => faithful_multisig_descriptor` → the old `template_from_descriptor` + `build_descriptor_string` collapse). The property FAILED and SHRANK to:
```
minimal failing input: schema = 1, seed = 0
O1 structural mismatch:
  recon: "wsh(multi(1,[11111111/48'/0'/0'/2']KEY/<0;1>/*,[22222222/…]KEY/<0;1>/*,[33333333/…]KEY/<0;1>/*))"
```
— the simplest timelocked-recovery policy, collapsed to plain multi (the `older()` dropped) — exactly the C1 silent-collapse class, auto-minimized. Fix restored; property GREEN.

## Bug found on run #1 (the harness working)
`bundle-accepts-sortedmulti-in-combinator-restore-cannot` (filed in `design/FOLLOWUPS.md`): `build-descriptor`/`bundle`/`export-wallet` accept `wsh(or_d(sortedmulti(…),…))` (rust-miniscript parses it) but `restore --md1` refuses it (md-codec `Tag::SortedMulti` must be the sole wsh/sh child). A card you can engrave but not mechanically restore — loud refuse, not silent funds-loss. The generator was scoped to `sortedmulti` top-level-only (the reconstructable set); the asymmetry is the FOLLOWUP.

## Tests (all green, clippy clean)
`backup_restore_roundtrip` (the property), `generator_covers_all_fragments`, 5 oracle self-test cells (`oracle1_rejects_{dropped_timelock,multi_sortedmulti_swap,masked_timelock_value}`, `oracle1_accepts_keyless_equivalent_redepth`, `oracle3_rejects_wrong_descriptor_address`), `smoke_handpicked_policies` (10 schemas end-to-end), `negative_property_unreconstructable_shapes_refuse_loudly` (per-key-override + hardened-wildcard via the `@N` slot pipeline → loud refuse).
