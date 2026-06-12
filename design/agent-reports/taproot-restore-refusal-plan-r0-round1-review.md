# R0 Review — taproot restore-refusal contracts (PLAN) — Round 1
Reviewer: Fable 5, 2026-06-12. Verified against mnemonic-toolkit origin/master 2f03eb0 (working tree clean; binary `target/debug/mnemonic` = 0.55.0 matches `crates/mnemonic-toolkit/Cargo.toml:3`).

## Verdict: RED (0 Critical / 2 Important / 3 Minor)

## Critical

None.

## Important

**I1 — The is_nums:false deferral premise is FALSE: that refusal IS bundle-reachable (probe-disproven).**
Plan §1 claims `bundle --descriptor 'tr(<xpub>,multi_a(…))'` "emits NOTHING (probe: empty)" and therefore defers the restore.rs:689 non-NUMS refusal to a heavier hand-built md_codec wire fixture. Re-probed against the 0.55.0 binary:

- `tr(X2, multi_a(2,X0,X1))` with a **distinct** cosigner internal key (X2 ∉ leaf set): `bundle` exit **0**, emits a 6-chunk md1, `.descriptor` round-trips EXACTLY; `restore --md1 …` exit **2**, stderr `taproot multisig md1 with a non-NUMS (cosigner) internal key is not supported by restore yet — re-engrave from seed with mnemonic >= v0.48.0 to get a NUMS tr md1 (FOLLOWUP restore-multisig-taproot-reconstruction)` — the restore.rs:689 arm, reached via the exact bundle→restore mechanism the plan already uses.
- The plan's "(probe: empty)" was almost certainly an artifact of probing with an IK that DUPLICATES a leaf key: `tr(X0, multi_a(2,X0,X1))` refuses at bundle time with `BIP-388 distinct-key violation: slot @0 and slot @1 resolve to identical (xpub, path)` (reproduced, exit 2, empty stdout). That is a bundle-side BIP-388 gate, not non-reachability of the restore arm.
- Bonus reachability: even keypath-only `tr(X0)` (origin-annotated xpub, no script tree) bundles (3-chunk md1) and `restore --md1` refuses via the SAME :689 arm — the `Body::Tr { is_nums: false, .. }` match arm (restore.rs:685) precedes the `tree: None` arm (:692), so any non-NUMS tr hits :689 first.

Fix: add a third refusal cell (`cosigner_internal_key_tr_bundles_but_restore_refuses_non_nums`) using the distinct-IK shape — it is the same cheap two-step as cells 1–2, needs no wire fixture — and rewrite §1's "NOT bundle-reachable" list. Pin the substring `non-NUMS (cosigner) internal key`. The only arm that genuinely needs a direct fixture is `tree: None` with `is_nums: true` (`tr(NUMS)` keypath-only refuses at bundle's origin-annotation gate: "descriptor has neither @N placeholders nor [fp/path]-annotated keys" — probe-confirmed); deferring THAT one to the FOLLOWUP's T3 is correct.

**I2 — Plan under-delivers its own FOLLOWUP's stated T1 scope (3 arms + multi-leaf wire round-trip).**
`design/FOLLOWUPS.md:4075` (`restore-general-and-multi-leaf-taproot-roundtrip`, scope split (a)) defines the T1 cycle as "pinning the **3 untested refusal arms** + a **multi-leaf wire/verify/address round-trip**". The plan pins 2 arms and its faithful-backup cell (cell 3) covers the GENERAL leaf only. With I1, the third bundle-reachable arm is cheap (no fixture machinery), and the multi-leaf wire-faithfulness assertion is equally cheap — probe-confirmed `tr(NUMS,{pk(X0),pk(X1)})` round-trips `.descriptor` EXACTLY too (extend cell 2 or cell 3 with the same equality assertion on the multi-leaf bundle). The verify/address legs are reasonably out of scope (verify-bundle needs cosigner mk1 cards — plan §5-Q3's lean is sound, and restore refuses before any address derivation), but the plan must either (a) fold in the third arm + the multi-leaf descriptor-equality assertion, or (b) explicitly amend the FOLLOWUP's scope-split wording in the same change so the registry and the shipped cycle agree. Lean (a) + a one-line FOLLOWUP touch-up noting `tree:None` is the only fixture-requiring arm.

## Minor

**M1 — Cell 3's "modulo NUMS substitution" caveat is factually wrong; assert EXACT equality.**
Probe: input with the H-point hex → emitted `.descriptor` is an EXACT string match (no checksum appended, no rewriting). Input with the literal `NUMS` token → emitted descriptor PRESERVES the literal `NUMS` token (no substitution to hex either). There is no substitution in either direction, so "equals the input, modulo NUMS substitution" describes a transformation that does not happen. As worded the cell would still pass (the caveat weakens, not breaks, the assertion), but it invites the implementer to write a dead normalizer. Fix: assert strict `input == emitted` and pick ONE input spelling (the hex H-point matches what restore's NUMS detection consumes; if the literal `NUMS` spelling is used, note the token is preserved verbatim on the wire).

**M2 — Fixture mis-cite: `XPUB4_0`/`XPUB4_1` and `[73c5da0a/48'/0'/0'/2']`-bracketed xpubs do not exist in tests/.**
grep over `crates/mnemonic-toolkit/tests/` finds zero `XPUB4` hits and zero `[…/48'/0'/0'/2']`-bracketed xpub literals — `48'/0'/0'/2'` appears only as `@N.path=` slot arguments (e.g. `cli_export_wallet_jade.rs:46`). Ready-made bracketed-xpub literals DO exist: the 3-cosigner `87'/0'/0'` trio at `cli_bundle_import_json.rs:312-314` (fps `73c5da0a` / `b8688df1` / `28645006`) — all probes in this review used exactly those and they exercise every cell (including the distinct-IK third arm, which needs 3 keys). Fix the plan's "Keys:" line to name a real source (lift those three as local consts) or to state the consts are minted fresh.

**M3 — Found quirk (note, not a blocker for this test-only cycle): keypath-only `tr(X0)` md1 refuses with a wrong-shaped message.**
The :689 message says "taproot **multisig** md1 … re-engrave from seed with mnemonic >= v0.48.0 to get a NUMS tr md1" — but a keypath-only `tr(xpub)` card is single-sig; re-engraving from seed would produce a different (single-sig) card, so the advice misleads. It is a clean refusal (exit 2, no mis-reconstruction), so it is a diagnostics-accuracy quirk, not a funds bug. Suggest a one-line note on the `restore-general-and-multi-leaf-taproot-roundtrip` FOLLOWUP (its T3 leaf-membership work naturally subsumes it); do NOT reword the message in this NO-BUMP cycle (that would invalidate the very contract being pinned).

## Notes

- **§1 probes verified live.** General leaf `tr(NUMS,and_v(v:pk(X0),after(12000000)))`: bundle exit 0, 3-chunk md1; restore exit **2**, stderr exactly `error: taproot md1 leaf is not a recognized multisig (multi_a / sortedmulti_a)`. Multi-leaf `tr(NUMS,{pk(X0),pk(X1)})`: same exit/message. Both match restore.rs:710.
- **Exit code confirmed:** `ToolkitError::ModeViolation { .. } => 2` at `crates/mnemonic-toolkit/src/error.rs:541`. The plan's exit-2 assertion is correct.
- **Untested confirmed:** zero `tests/` hits for "not a recognized multisig"; the `non-NUMS` hits in `tests/cli_compare_cost.rs` are the compare-cost keypath-advisory surface, not restore's refusal — both restore arms are genuinely unpinned.
- **Routing confirmed:** `--md1` non-empty always dispatches `run_multisig` (restore.rs:175-176); `Tag::Tr` always routes to `taproot_template_and_internal_key` (restore.rs:1081-1085) — the v0.54.0 faithful general-policy arm is explicitly non-taproot-only, so no taproot shape can leak into it.
- **No silent mis-reconstruction found.** Probed shapes: general leaf, multi-leaf, single-leaf `pk`, distinct-cosigner-IK multi_a, keypath-only xpub, keypath-only NUMS — every non-(multi_a/sortedmulti_a∧NUMS) shape refuses loudly (exit 2) or is bundle-gated; the only accept path is the v0.49.1-tested MultiA/SortedMultiA+NUMS reconstruction. The "wrong outcome" class the orchestrator asked about does not currently exist on this surface.
- **Substring pinning (§5-Q2): agree.** "not a recognized multisig" is present, distinctive, and survives minor rewording; pin substrings, not full strings, in all three refusal cells (`non-NUMS (cosigner) internal key` for the third).
- **Test invocation (§6): confirmed house pattern.** `cli_restore_multisig.rs:67` `restore_args` = `restore --network mainnet` + repeated `--md1 <chunk>`; `Command::cargo_bin("mnemonic")` (`:22` et al.) is the bin-spawn pattern. Plan matches.
- **Scope/NO-BUMP (§7): confirmed.** One new test file, no `src/` change, no clap surface → no manual/GUI/schema_mirror lockstep. Both FOLLOWUPs already filed: `upstream-miniscript-taptree-depth2-display-asymmetry` (FOLLOWUPS.md:4062) and `restore-general-and-multi-leaf-taproot-roundtrip` (FOLLOWUPS.md:4071) — nothing to file this cycle beyond the I2/M3 touch-ups.
- A single-leaf `tr(NUMS,pk(X0))` cell would be redundant with cell 1 (same :710 arm, probe-confirmed identical refusal) — not worth a fourth cell.
- Probe temp files deleted; no source edited.
