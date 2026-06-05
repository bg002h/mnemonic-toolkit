# R0 Architect Review (Round 1) — `SPEC_verify_bundle_entropy_slot.md` (v0.43.1)

**Reviewer:** opus `feature-dev:code-reviewer` (R0 mandatory pre-impl gate)
**Date:** 2026-06-04
**Source SHA:** `0f404ae` (branch `verify-bundle-entropy-slot`, only SPEC added)
**Verdict:** 0 Critical / 1 Important / 1 Minor — **GATE: RED**

> Persisted verbatim per CLAUDE.md ("persist the full review-agent output BEFORE applying folds"). Fold log appended at bottom.

---

**Scope reviewed:** the proposed new `else if subkeys.contains(&SlotSubkey::Entropy)` arm in the `verify-bundle` descriptor binding loop, against source ground truth at `0f404ae`. Every SPEC claim was verified by reading the cited source, not trusted.

### Verified-accurate claims (no findings)

- **Binding-loop structure** (`crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`): the 5-tuple binding `let (xpub, fingerprint, path, ent_opt, emit_lang): (BipXpub, Fingerprint, DerivationPath, Option<Vec<u8>>, Option<bip39::Language>)` is at `:782-787`; Phrase/Seedqr arm `if` at `:788`; Xpub arm at `:830`; Ms1 arm at `:855`; catch-all `DescriptorReparseFailed` at `:885-892`. There is currently NO `Entropy` arm. All SPEC line citations are accurate. (Note the file lives at `src/cmd/verify_bundle.rs`, not `src/verify_bundle.rs` — basename in SPEC is fine.)
- **Helper signature** (`crates/mnemonic-toolkit/src/derive_slot.rs:65`): `derive_bip32_from_entropy_at_path(entropy: &[u8], passphrase: &str, language: Bip39Language, network: CliNetwork, path: &DerivationPath)`. `&entropy_bytes` (a `Zeroizing<Vec<u8>>`) deref-coerces to `&[u8]` — proven by the adjacent Ms1 arm compiling with the identical `&res.entropy` pattern (`:870-871`).
- **`into_parts()` tuple order** (`crates/mnemonic-toolkit/src/derive.rs:47-56`): `(Vec<u8>, Fingerprint, Xpub, Xpriv, DerivationPath)` = `(entropy, master_fingerprint, account_xpub, account_xpriv, account_path)`. The SPEC's destructure `(_acc_entropy, master_fp, xpub, _xpriv, _path)` is **correct** and byte-matches the Ms1 arm's destructure at `verify_bundle.rs:877`.
- **`language.into()`**: `args.language: Option<CliLanguage>` → `.unwrap_or_default().into()` → `bip39::Language`, identical to the bundle Entropy arm (`bundle.rs:1453,1456`).
- **5-tuple symmetry**: `Some((*entropy_bytes).clone())` is the right `Option<Vec<u8>>`; `emit_lang = None` matches the bundle Entropy arm's 5th element (`bundle.rs:1470`). Raw entropy carries no wire language — symmetric.
- **Scope**: `idx`, `anno_path`, `slot_inputs`, `subkeys`, `secp`, `args.network`, `args.passphrase`, `args.language` are all in scope at the insertion point (confirmed via `verify_bundle.rs:744-776` and the Ms1 arm's identical usage).
- **`hex::decode`**: `hex = "0.4"` at `Cargo.toml:50`. `hex::decode` is NOT currently `use`d or called in `verify_bundle.rs`, but the SPEC calls it fully-qualified (`hex::decode(...)`), which is reachable from anywhere in the crate without a `use`. Correct.
- **Zeroization/pinning**: the arm correctly omits in-arm `pin_pages_for` — the loop re-pins the returned `ent_opt` at `verify_bundle.rs:894-895`, consistent with both sibling arms. The helper's internal pin (`derive_slot.rs:103`) is dropped at `into_parts()`, but the loop-level re-pin covers it. No hazard.
- **Lockstep claims (§6)**: `is_legal_set` already contains `[Entropy]` (`slot_input.rs:348`); `entropy` is already `is_secret_bearing` (`slot_input.rs:85`) and documented in the manual (`docs/manual/src/40-cli-reference/41-mnemonic.md:67`). No positive per-mode allow-list exists — only the two negative catch-alls (`verify_bundle.rs:885`, `bundle.rs:1501`), both of which correctly stay (still-valid for `xprv`/`wif`). No GUI schema_mirror trigger (no flag change), no manual mirror trigger. All accurate.
- **FOLLOWUP** `verify-bundle-descriptor-entropy-slot-gap` exists (`design/FOLLOWUPS.md:59-66`, `Status: open`) and prescribes exactly this fix.

---

## Critical

None.

## Important

**I1 — Test #4 (§5.4) rests on an unverified canonicity assumption; its fallback does not escape the failure mode it names.** (`design/SPEC_verify_bundle_entropy_slot.md` §5 item 4)

Test #4 bundles `@0.entropy=<hex>` + `@1.xpub=<xpub>` against `CANONICAL_DESC = "wsh(sortedmulti(2,@0,@1))"` (`cli_ms1_slot.rs:61`). The verify step will reach the new arm (the descriptor binding loop at `verify_bundle.rs:744+` runs unconditionally for canonical and non-canonical alike; the only canonical secret-rejection is the secret+watch-only-in-same-slot conflict at `bundle.rs:1189-1214`, which a bare `[Entropy]` slot does not trip). **But the risk is in the bundle-setup step, and there is zero precedent for a secret cosigner bundling on a canonical `--descriptor wsh(sortedmulti)` string:**
- Every secret-cosigner `.success()` on a `--descriptor` *string* is **non-canonical** (`cli_non_canonical_descriptor.rs:22`, `wsh(andor(...))` → `synthesize_descriptor`).
- The only mixed secret+xpub multisig `.success()` (`cli_bundle_slip0132_info.rs:208-228`) uses **`--template wsh-sortedmulti`** (`synthesize_unified`), a different code path that does not transfer across the boundary.
- In `cli_ms1_slot.rs`, **every** use of `CANONICAL_DESC` is a refusal test (`:83`, `:113`, both `.code(2)`); no test bundles a bare secret against it and succeeds.

This is not proven-broken — but it is unproven, and the SPEC's §5.4 fallback is inadequate: it stays on `CANONICAL_DESC` (`@1.entropy=<different hex>`), which only escapes the distinct-key guard (`cli_unified_slot.rs:264`), **not** a canonicity-mode setup failure. A dev following §5.4 literally hits a dead end and may "fix" it by weakening the assertion.

**Fix:** retarget test #4 to a *proven* non-canonical multi-`@N` fixture. Reuse `cli_non_canonical_descriptor.rs:22`'s 3-cosigner `wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))` — already shown to bundle secret slots successfully — and place `@1.entropy=` (with phrase/xpub on the other slots) on it. That delivers the exact stated goal ("entropy arm at a non-`@0` slot in a multi-`@N` descriptor") with no unproven canonical dependency. Note `NONCANONICAL_DESC` stays single-`@0`, so it cannot supply the multi-`@N` cell — pull in the andor fixture explicitly.

## Minor

**M1 — §3's "arm order is load-bearing, not cosmetic" rationale is false.** (`design/SPEC_verify_bundle_entropy_slot.md` §3, paragraph 1)

The SPEC claims arm placement "sets the precedence when a slot carries multiple value-subkeys." But `is_legal_set` (`slot_input.rs:342-359`) permits `[Entropy]` *only* — there is no `[Entropy, X]` legal combination — so `subkeys.contains(&Entropy)` is never simultaneously true with the Xpub/Ms1 `contains` checks. Placement mirroring the bundle loop (Xpub→Entropy→Ms1) is correct and worth keeping for consistency, but the stated rationale is wrong and will mislead a future editor. Reword to "placement mirrors the bundle loop for consistency; precedence is moot because `is_legal_set` forbids `[Entropy, *]` co-occurrence."

---

**VERDICT: 0 Critical / 1 Important**
**GATE: RED**

Fold I1 (retarget test #4 to the proven non-canonical multi-`@N` `andor` fixture; correct the §5.4 fallback) and M1 (fix the §3 arm-order rationale), persist this review verbatim to `design/agent-reports/verify-bundle-entropy-slot-r0-r1-review.md`, then re-dispatch R1 per the per-fold reviewer-loop discipline. The arm's compile-correctness is solid — the only blocker is test-matrix soundness for the one canonical-descriptor cell.

---

## Fold log (applied after persisting)

- **I1 — FOLDED.** Independently grep-verified: `cli_non_canonical_descriptor.rs:21-43` (and `:232`) bundles `@0/@1/@2.phrase` on the 3-cosigner `wsh(andor(...))` with `.success()` — proven non-canonical multi-`@N`. Retargeted §5 test #4 to this fixture with `@0.phrase` / `@1.entropy=<hex>` / `@2.phrase`; removed the inadequate `CANONICAL_DESC` fallback; declared the andor fixture as a third local constant.
- **M1 — FOLDED.** Independently grep-verified: `slot_input.rs:343-359` `is_legal_set` lists `[Entropy]` standalone only; no `[Entropy, *]` arm. Reworded §3 to "placement mirrors the bundle loop for consistency; precedence is moot — `is_legal_set` forbids `[Entropy, *]` co-occurrence."
- Re-dispatched R0 round 2 after fold.
