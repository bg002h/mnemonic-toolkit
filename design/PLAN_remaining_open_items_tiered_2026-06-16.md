# PLAN — remaining open items, tiered/phased (architect recommendation, 2026-06-16)

> Opus architect (feature-dev:code-architect) output, persisted verbatim. Input recon:
> `cycle-prep-recon-remaining-open-items-2026-06-16.md` @ toolkit master `15d43c9`. Every
> load-bearing premise re-verified against live source (ledger below). This is a
> RECOMMENDATION, not yet an R0-gated plan-doc — each item it recommends is still its own
> R0-gated mini-cycle before code.

---

## Premise verification ledger

| Claim | Verdict | Evidence |
|---|---|---|
| STRESS-A still wsh-only | ✅ | `prop_backup_restore_roundtrip.rs:251` hardcodes `"wrapper":"wsh"` |
| `arm_dup_if` still `#[ignore]` + empty body | ✅ | FOLLOWUPS:4191; ignore-reason disproven (`wsh(or_i(...))` parses on 13.0.0) |
| `import-wallet` enum lacks `green` | ✅ | `import_wallet.rs:142` enum has no `green` (export-only) |
| `addresses --address-type` required | ✅ | `addresses.rs:35-36` typed, no default, not Option |
| ms-codec lacks `indel_reject_contract.rs` | ✅ | only md-codec + mk-codec have it |
| `unrestorable` advisory never shipped | ✅ | grep in toolkit src/ = 0 hits |
| `bundle --descriptor` needs `@N` (C4 door missing) | ✅ | `parse_descriptor.rs:136` |
| `nostr --all-script-types` precedent | ✅ | `cmd/nostr.rs:91` |

Scope notes: **C4** is thinner than "new ingest door" (import-wallet parsers already do `[fp/path]xpub→@N`; gap = bare-descriptor-STRING entry point only → likely a thin `import-wallet --format descriptor`). **C2** overstated — the BIP-388 expander is already shared (`wallet_import::pipeline::expand_bip388_policy`) → lowest-risk MINOR in tier C.

## Tiers

- **Tier 1 — fast test-hardening (NO-BUMP, independent, parallelizable): B1 B2 B3 B4.** Three different repos → zero collision. B1 highest value (only harness exercise of toolkit-unique `tr(NUMS,sortedmulti_a)`; may FIND a bug). B2/B4 trivial.
- **Tier 2 — small features (versioned, lockstep-aware): C2 C1 C5 C3.** Ordered by value/risk. C2 (cheap MINOR, shared expander) + C1 (PATCH advisory) = clear wins; C5 thin MINOR; C3 convenience.
- **Tier 3 — decision-gated (no code until answered): C4 C6 D2.**
- **Tier 4 — rehearsal-gated infra: D1** (CI v6/7/8 — throwaway-tag rehearsal of `download-artifact@v8`/`checkout@v6` first).
- **Tier 5 — parked / no-action: A1–A3 (upstream-blocked on miniscript >13.1.0), D3 (rides a pin bump), D4 (not due ~2026-09), E (wontfix tracker).**

## Recommended phased order

- **Phase 0 (DO FIRST): run B1–B4 in parallel.** No-regret: NO-BUMP, cross-repo, B1 is the highest expected value of anything open. File B1's missing FOLLOWUP slug.
- **Phase 1: C2 → C1, then C5.** Lockstep: **C3 & C5 are CLI-flag changes → paired GUI `schema_mirror` + `docs/manual` flag-coverage in the SAME cycle** (lagging gate — else silent drift, cf. v0.27.2's 8-flag backfill). C2/C1 add no flags (C2 = new input shape on existing `--descriptor`; C1 = stderr) → no schema_mirror surface; C2 still MINOR + manual note (new input format = capability, v0.49.0 precedent). No publish for any Tier-2 item (toolkit git-tag only).
- **Phase 2: decision-gated** — only after the user answers (below). D2 stay status-quo unless a concrete lib consumer appears.
- **Phase 3: D1** infra (rehearse tag, then bump). Slots anywhere after Phase 0.
- **Defer/drop:** C3 (FOLLOWUP itself says "confirm demand"; full lockstep for pure convenience → unscheduled). Park A/D2/D3/D4/E. C6 likely WONTFIX-with-note.

## Per-item sketches + overhead

**B1 STRESS-A tr leg** (toolkit, ~120-180 LOC, ⭐): tr leg in `prop_backup_restore_roundtrip.rs` generating `tr(NUMS,{multi_a|sortedmulti_a}(k,…))` strings → `bundle --descriptor`; O1/O2/O3 reuse; + non-NUMS/@-in-both refusal cell. R0+impl review, NO-BUMP, file the missing slug.
**B2 arm_dup_if de-stub** (toolkit, 1 test): de-ignore + write body (`wsh(or_i(pk(@0/<0;1>/*),dv:older(144)))` → assert `Tag::DupIf`), `--bin mnemonic`. Trivial. Slug open.
**B3 ms-codec themes 1/2/3** (mnemonic-secret, test-only): add indel reject-contract + correction property to ms-codec proptest, mirror siblings. R0+review, NO-BUMP, no publish.
**B4 md-codec bitcoind breadth** (descriptor-mnemonic, test-only): +4-6 Shape rows (plain multi, hashlock, after, or_d/andor; NO SortedMultiA). Slug `bitcoind-differential-corpus-breadth`. NO-BUMP.
**C2 verify-bundle BIP-388 intake** (toolkit, MINOR, lowest-risk): `is_bip388_policy_shape` probe + `expand_bip388_policy` at verify-bundle (ORDERING: policy probe before @N/key-regex). No new flag → no schema_mirror; manual note. Slug `verify-bundle-bip388-policy-intake`.
**C1 unrestorable-shape advisory** (toolkit, PATCH): stderr advisory at bundle/export-wallet engrave for shapes restore refuses (sortedmulti-in-combinator / use-site overrides / hardened wildcard). No lockstep. Slug `bundle-unrestorable-shape-advisory`.
**C5 import --format green** (toolkit, MINOR+GUI+manual): add `Green` to import enum/dispatch — FIRST confirm whether `--format bitcoin-core` already reads a Green singlesig export (then doc-only). File a slug.
**C3 single-sig batch emit** (toolkit, MINOR+GUI+manual): `addresses --all-script-types`/`export-wallet --all-single-sig` over {p2pkh,p2sh-p2wpkh,p2wpkh,p2tr}, mirror `nostr --all-script-types`. **DEFER (confirm demand).** Slug `single-sig-multi-script-type-batch-emit-not-surfaced`.
**C4** (gated): thin `import-wallet --format descriptor` reusing `[fp/path]xpub→@N` extraction. **C6** (gated): mk-codec MINOR + vectors + publish, OR WONTFIX. **D2** (gated): ~80-file de-entangle of 3 `ToolkitError` variants; `pub use` shim recon-DISPROVEN; (c) status-quo recommended. **D1**: throwaway-tag rehearsal then bump; eval SHA-pin vs floating-major.

## Decisions the user must make (gate Tier 3)

1. **C4** — do you need to feed a BARE `wsh(sortedmulti(...))` string (no @N, no wallet-file)? standalone `md`-CLI only, or also the toolkit door? (If no real workflow → drop; import-wallet parsers cover hardware files.)
2. **C6** — should mk1 PRESERVE the cosigner's `ypub`/`zpub` on-card, or is normalize-in/re-emit-out sufficient? (No funds-safety diff; preservation = mk-codec wire field + vectors + publish. Lean WONTFIX-with-note.)
3. **D2** — keep status-quo (c), or commit to de-entangle SPEC (b)? (Only worth (b) if a concrete external lib consumer appears; shim disproven.)
4. **C3** — real demand to emit all 4 single-sig script types in one shot, or speculative? (Lean: unscheduled until asked.)
5. **C5** — confirm whether `import-wallet --format bitcoin-core` already reads a real Green singlesig export (→ doc-only) before coding.

## Bottom line

**Start Phase 0 now: B1–B4 in parallel** (NO-BUMP, cross-repo, B1 the highest-value bug-finder). **Then C2 + C1** (cheapest versioned wins, no flag-lockstep). **Hold C3/C4/C5/C6/D2/D1 behind the 5 decisions** — expect C3 + C6 → "don't build / WONTFIX", D2 → status-quo, leaving C5 (thin, maybe doc-only) + the D1 rehearsal as the only other genuinely-worth-doing items.
