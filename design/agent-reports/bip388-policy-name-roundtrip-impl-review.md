# Implementation Review — BIP-388 policy-name round-trip (v0.53.8) — before commit

Reviewed the uncommitted working tree against the R0-GREEN spec.

**Verdict: 0 Critical / 0 Important / 2 Minor (both FIXED).**

## Minor (both folded)
- **M1** — duplicate `wallet_name_is_non_default` comment in `export_wallet.rs` (a scratch-revert/restore artifact). FIXED (removed the duplicate copy).
- **M2** — a PRE-EXISTING standalone FOLLOWUP `bip388-policy-roundtrip-wallet-name-not-honored` (filed v0.49.0, same bug as the audit-backlog `bip388-policy-name-lossy-roundtrip`) still had `Status: open`. FIXED (marked resolved v0.53.8, cross-referencing the slug + the carved-out template-path follow-up).

## Confirmations (adversarial checklist)
- **General-lift blast radius safe:** only `bitcoin-core` (never uses wallet_name), `bip388` (the fix), and `specter` (T5 — intentional unblock) accept descriptor passthrough (`template_opt==None`); sparrow/electrum/coldcard refuse passthrough via `ok_or_else(inputs.template)` so are unreachable. No silent incorrect change.
- **Scope/precedence:** `bip388_policy_name` declared `None` unconditionally before the block → safe on --template/non-policy paths; bip388 ⇒ --descriptor ⇒ template_opt None ⇒ lands in the None→None leaf, can't be shadowed; --descriptor/--template mutually exclusive (guard). Precedence flag > policy-name > default correct.
- **Two-parse flow:** `bip388_policy_name(desc)` before `expand_bip388_policy(desc)?` — if expand errors, `?` propagates and the captured name is never used; None-on-malformed lets expand surface the real error. No stale-name path.
- **Signature change:** both callers updated (`bip388.rs` → `inputs.wallet_name`, `build_descriptor.rs` → `DEFAULT_BIP388_POLICY_NAME`); no other caller; re-export from `mod.rs` correct; build_descriptor output byte-identical.
- **Tests:** T1 + T5 RED-proven (scratch-revert both halves: emit-name + non_default flip); T2/T3/T4 sound; two-step test comment corrected; `cli_export_wallet.rs:885` stays green (concrete descriptor → None → default).
- **No regression:** full suite green (158 ok, 0 failures); clippy clean.
