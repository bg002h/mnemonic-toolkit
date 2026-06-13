# R0 round 3 — architect review (verbatim) — SPEC_restore_non_nums_taproot_internal_key.md

> Persisted verbatim per CLAUDE.md. Dispatched via Agent tool (feature-dev:code-architect, inherited
> session default model Opus 4.8). **Verdict: GREEN (0 Critical, 0 Important, 1 Minor).** Source SHA
> `29613f3`, spec commit `e0f5a35` (post-r2-fold). The mandatory R0 gate is MET. The 1 Minor (m1,
> `tlv.pubkeys` population for the @-in-both RED-proof) was folded into §7 before proceeding.

---

## CRITICAL

None.

---

## IMPORTANT

None.

All R0-r2 I1 and I2 folds have landed correctly and are internally consistent:

**R0-r2 I1 verification (§6 bip388 guard placement):** Confirmed correct. The spec now places the explicit bip388 refusal exclusively inside the `None` branch of `build_multisig_import_payload` (`restore.rs:832-844` vicinity), gated on `matches!(script_type, WalletScriptType::P2tr | WalletScriptType::P2trMulti)`. The routing trace is watertight: a `Template(t, Cosigner(idx))` arm at the call site (`restore.rs:1207-1208` post-impl) produces `template = Some(t)` → takes the `Some(t)` branch at `restore.rs:828` → never reaches the `None` branch → never hits the refusal → reaches `Bip388Emitter::emit` with `template = Some(t)` → `format_bip388_wallet_policy` → `Cosigner(idx)` arm at `bip388.rs:115-127` → emits `tr(@idx/**,multi_a(k,...))` faithfully. `GeneralFaithful(Cosigner(key_index))` → `template = None` → `None` branch → explicit guard fires → `BadInput` exit 1. No contradiction remains.

**R0-r2 I2 verification (§7 test split):** Confirmed correct. The spec now splits format-output tests into: (a) non-NUMS general-tr bip388 → refused (exit 1, explicit guard), (b) non-NUMS distinct-trunk multisig bip388 → SUCCEEDS (golden `tr(@idx/**,multi_a(2,...))` via Template path). The existing `general_tr_format_bip388_refused` test at `cli_restore_taproot.rs:290` correctly identified for message-assertion update (exit 1 unchanged; `/<0;1>/*` message changes to the new explicit guard's message). No contradiction remains.

**§4 @-in-both structural guard:** Still intact and unweakened by the r2 folds. The index-membership check (`key_index ∈ indices`) remains the sole protection against the Template/Cosigner "leaf = all-others" shortcut silently constructing a different multisig. The Display-fidelity guard provably cannot catch it (pipeline.rs:28-31 `from_str().to_string()` is self-consistent). The GeneralFaithful arm is immune (reads the actual tree). The r2 folds touched only §6 and §7 text; §4 is untouched.

**NUMS regression:** Existing v0.49.1/v0.55.1 goldens stay byte-identical. The proposed change threads `Cosigner(key_index)` only for `is_nums:false`; `is_nums:true` continues threading `Nums` (call site `restore.rs:1207` post-impl). The `general_tr_format_bip388_refused` test's exit code stays 1; only the message text changes.

**All citation verifications:** `restore.rs:661-668` (TaprootRestore enum, pre-impl, no internal key field yet — correct), `:692` classify entry, `:700` is_nums:false blanket refusal (confirmed), `:719-720` MultiA/SortedMultiA, `:730` general arm, `:1207-1208` call site (both arms hard-code Nums — confirmed pre-impl state), `:796-798` comment (text confirmed), `:832-844` None branch with existing green guard (confirmed). `bip388.rs:109-127` TrMultiA/TrSortedMultiA arm with Nums (`:109-114`) and Cosigner (`:115-127`) — both confirmed. `pipeline.rs:113-156` build_tr_multi_a_descriptor Cosigner arm — confirmed. `mod.rs:87` TaprootInternalKey — confirmed. `cli_restore_taproot.rs:290` general_tr_format_bip388_refused — confirmed, asserts `code(1)` + `contains("/<0;1>/*")`. Manual `:771`, `:794`, `:1027` — all confirmed with current non-NUMS refusal text.

`green.rs:36` refuses via `is_multisig()` which includes `P2trMulti`. The spec's claim is accurate.

---

## MINOR

**m1 (§7 — `@-in-both` test construction omits `tlv.pubkeys`).** The spec instructs the implementer to construct a `Descriptor` directly with `Body::Tr { is_nums:false, key_index: i, .. }` and a `Body::MultiKeys { indices }` leaf, then call `chunk::split`. It does not mention populating `tlv.pubkeys`. At `restore.rs:1155`, `is_wallet_policy()` is checked before `classify_taproot_restore` is ever reached: a `Descriptor` with `tlv.pubkeys = None` or `tlv.pubkeys = Some(vec![])` hits "template-only" `ModeViolation` — the wrong gate, wrong slug, wrong test behavior. For the RED-proof to exercise the `@-in-both` structural guard's necessity (the spec's stated goal), `tlv.pubkeys` must be populated with at least one entry per key slot. The spec should note: "populate `tlv.pubkeys` with dummy 65-byte entries for each key slot so `is_wallet_policy()` passes the step-2 gate and the card reaches `classify_taproot_restore`."

---

VERDICT: GREEN
