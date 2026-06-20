# SPEC R0 round 1 — bundle-md1-template-only-option (opus architect, verbatim)

> Reviewer: opus architect (read+bash; descriptor-mnemonic `c85cd49` == registry md-codec 0.36.0 (toolkit pin), mnemonic-key `913febc`). **Verdict: RED — 2 Critical, 1 Important, 5 Minor.** Folded → re-dispatch. UNCOMMITTED (freeze).

---

## R0 Review — SPEC `bundle_md1_template_only_2026-06-19` (phase 1, single-sig shareable template)

**Verdict: RED — 2 Critical, 1 Important, 5 Minor.** Two findings are silent-wrong-wallet / unimplementable-as-written holes an implementer following the SPEC verbatim would ship. All mechanically simple to fix; the architecture is sound and every id/wire/binding claim checked is correct.

### CRITICAL

**C1 — The §4.2 canonical gate as written ADMITS canonical multisig, re-opening the deferred C1 hole.** `canonical_origin` (`canonical_origin.rs:57-71`) returns `Some(...)` for `wsh(multi/sortedmulti) → m/48'/0'/0'/2'` and `sh(wsh(multi/sortedmulti)) → .../1'`. So `canonical_origin(&d.tree).is_some()` is TRUE for canonical 2-of-3 multisig. An implementer coding §4.2 verbatim would let `wsh(sortedmulti)` through template form → keyless multisig template → the C1 keyless-multisig inversion (deferred to phase 2). **Fix:** gate = `descriptor.n == 1 && canonical_origin(&d.tree).is_some()`. With `n==1` the only `Some` shapes are pkh/wpkh/tr-keypath; every multi/sortedmulti is `n≥2`. (`bare_wsh_at_n_returns_none:234` confirms bare single-key wsh returns None.) Refusal + `TemplateFormUnsupportedShape` fire on BOTH `n>1` and `is_none()`.

**C2 — §4.5 ingestion path is blocked: every `--md1` dispatches to `run_multisig`, which hard-refuses a keyless template today; the single-sig completion path is unreachable.** `restore.rs:177-179`: `if !args.md1.is_empty() { return run_multisig(...) }` — ANY `--md1` returns into `run_multisig` and never reaches the single-sig path. `run_multisig` at `restore.rs:1232-1236` hard-refuses keyless: `if !d.is_wallet_policy() { return Err(ModeViolation{ "--md1 is template-only ... needs a wallet-policy md1" }) }`. So `restore --md1 <keyless-template> --from <seed>` hits a `ModeViolation` today — answers the SPEC's own Q8: **No, it hits `restore.rs:1232`.** §4.5 unimplementable as written. **Fix:** specify the routing carve-out at `:177-179` + `:1232-1236` — if the reassembled md1 is a keyless single-sig template (`!is_wallet_policy() && n==1 && canonical_origin().is_some()`), route to the NEW single-sig completion; else keep today's behavior (the `:1232` refusal then correctly catches a keyless *multisig* template).

### IMPORTANT

**I1 — §4.3's binding re-root list is INCOMPLETE: omits two production hard-refusal gates in the bundle self-check that reject every template bundle.** `verify_self_consistency` has non-debug production `return Err` gates a template bundle trips before stub comparison: (1) `bundle.rs:2151-2156` `if !desc.is_wallet_policy() { return Err(BundleMismatch{"descriptor is not in wallet-policy mode"}) }` (separate from the `synthesize.rs:346` debug-assert §4.2 mentions); (2) `bundle.rs:2171-2177` `let Some(pubkeys) = desc.tlv.pubkeys.as_deref() else { return Err("tlv.pubkeys is absent — cannot bind mk1 xpubs") }` → `check_mk1_xpub_binding` `:2186/2220` (keyless template has no pubkeys to bind). §8 lists "a template bundle passes its own self-check" as a property — currently it can't. **Fix:** when form==template the self-check must (a) skip/branch the `:2151` is_wallet_policy gate, (b) skip the `:2171` pubkeys-absent refusal + `check_mk1_xpub_binding` (the mk1↔template binding is the stub-on-`WalletDescriptorTemplateId` check at `:2187/2236`, which still applies), (c) compute `expected_stub` at `:2158/2162` from `compute_wallet_descriptor_template_id`.

### MINOR
- **M1** §4.3 cites dead test-only sites `synthesize.rs:180/216` (inside `#[allow(dead_code)]` `synthesize_full`/`synthesize_watch_only`). The sole production string-level stub site is `synthesize_descriptor` (`:272-274`). Plan-doc target `:272`; note `:180/216` dead.
- **M2** §4.3 csi sites `:192/228` are dead; production csi sites are `synthesize.rs:290` (n==1) and `:310` (n≥2). Single-sig template → `:290` live.
- **M3** §4.4 "via `emit_output_class_advisory`" is a pattern reference; `bundle.rs:1035` emits an `OutputClass`, not an arbitrary id line. D7 is a NEW writeln modeled on it — don't literally reuse the fn.
- **M4** §4.7 should also flag the stale `mk-cli mod.rs:55-62` doc-comment (not only `key_card.rs:25-30`). Confirm the branch keys on `!descriptor.is_wallet_policy()` — `compute_wallet_policy_id` on a keyless template does NOT error (hashes empty-origin/template-only presence), so it silently mis-stubs without the branch; `is_wallet_policy()` is the correct discriminator.
- **M5** §6: `install.sh` is `scripts/install.sh` (not repo root). Toolkit `0.58.1 → 0.59.0` MINOR correct; all version sites exist. `TemplateFormUnsupportedShape` sorts between `SlotInputViolation` and `UnknownHrp`.

### CONFIRMED correct (load-bearing)
- §4.2 four mutations + production site `synthesize_descriptor:258`; encoder writes `d.path_decl` verbatim (`encode.rs:85`), no canonical-fill → empty path_decl reaches the wire, byte-identity real. `split`/`encode_payload` have no is_wallet_policy encode-guard → keyless template encodes + chunk-counts fine (`chunk.rs:235`).
- **Account normalization HOLDS:** elided path_decl on a canonical wrapper → `expand_per_at_n` (`canonicalize.rs:440-455`) resolves to EMPTY (does NOT fill m/84'/0'/0'; `MissingExplicitOrigin` doesn't fire because canonical). `WalletDescriptorTemplateId` excludes origin/account/keys/fp → account-0 vs account-5 byte-identical md1 + identical stub. Restore reconstructs explicit `m/84'/0'/account'` from `--account`/`--origin` (`derive_slot.rs:65`).
- §4.4 D7/I4 correct: `compute_wallet_policy_id` origin-significant, INVARIANT (`:161-171`) warns eliding needs canonical_origin re-intro — SPEC sidesteps by hashing the FULLY-KEYED explicit-origin presence-0b11 descriptor BOTH sides. `to_phrase()` on `[u8;16]` (`phrase.rs:17`) renders 12 words.
- §4.3 ms1 exclusion correct (`ms_codec::encode` carries no id). §4.6 target correct (`verify_bundle.rs:2152-2167` is the vacuous skip). `md decode` of a keyless canonical-elided template does NOT refuse (`decode.rs:68` short-circuits for canonical; `:69` no-op when pubkeys:None). No privacy/cross-binding concern (generic template-id stub is the intended feature; D7 is the disambiguator).

### To turn GREEN
1. **C1** → §4.2 gate = `n == 1 && canonical_origin().is_some()`.
2. **C2** → §4.5 routing carve-out at `restore.rs:177-179` + `:1232-1236`.
3. **I1** → §4.3 add the two self-check gates (`bundle.rs:2151`, `:2171/2186/2220`) to the template-form branch set.
