# BRAINSTORM — bundle-md1-template-only-option (#27)

**Date:** 2026-06-19 · **Status:** brainstorm COMPLETE — D1–D6 LOCKED; awaiting go to SPEC. **UNCOMMITTED working draft** — frozen until the SeedHammer first-pass integration completes + an explicit go. Cross-repo, SeedHammer-driven.

## Feature
`bundle --md1-form=policy|template` (default `policy` = today). When `template`, emit a **keyless TEMPLATE md1** (script + origin + use-site, `pubkeys:null`) instead of the full wallet policy. **Motivation: fewer engraving plates** — single-sig template ≈ 1 chunk vs full policy ~2-3 (md1 single-string cap `SINGLE_STRING_PAYLOAD_BIT_LIMIT` = 320 bits; a 65-byte xpub + 4-byte fp TLV pushes a single-sig policy to multi-chunk).

## Headline (from cycle-prep): the hard part already exists
- md-codec `WalletDescriptorTemplateId` / `compute_wallet_descriptor_template_id` (`identity.rs:71-104`) is a ready-made **key-stable** binding id.
- A **keyless template md1 already encodes + round-trips** (`is_wallet_policy` keys off Pubkeys-TLV presence).
- So this is NOT "invent a template format" — it's (1) swap the binding to the template id when form==template, and (2) build the net-new **template-completion** in `restore`/`verify-bundle`.

---

## ID taxonomy (folded in — anchors D1/D4)

All three are **128-bit (16-byte) truncated SHA-256** ids in `md-codec/src/identity.rs`; they differ in *what they hash* → an abstraction spectrum (each forgets more):

| ID | Preimage | Changes when… | Invariant to… | Role |
|---|---|---|---|---|
| **`Md1EncodingId`** (`:11`) | canonical bit-packed **md1 payload** bytes (`encode_payload`) | *any* payload byte | nothing — it *is* the encoding | "this exact engraving." `.fingerprint()`=first 4B. (= SH Go `computeEncodingID`.) |
| **`WalletPolicyId`** (`:106`) | canonical-expanded **policy**: tree + per-`@N` origin + use-site + fp + xpub (+ presence-byte) | keys/xpubs, fingerprints, origin-path *values* (account), use-site, tree | encoding choices (path elision vs explicit) | "this *specific* wallet." **Card-binding root TODAY.** `.to_phrase()`→12-word id. |
| **`WalletDescriptorTemplateId`** (`:47`) | **template only**: use-site-path-decl + tree + `UseSitePathOverrides` TLV | the script tree / use-site / overrides | **keys, fingerprints, origin-path/account, header, HRP, checksum** | "this *kind* of wallet." **Key-stable** — what #27 binds on. |

Mental model: **encoding → policy → template** = *this engraving → this wallet → this wallet-shape*.

**Derived engraved binding stubs (slices, not new hashes — today rooted on `WalletPolicyId`):**
- md1 card prefix = `WalletPolicyId[0..2]` → 16 bits / 4 hex (`bundle.rs:1082`).
- mk1/ms1 per-cosigner `chunk_set_id` = `derive_mk1_chunk_set_id_for_slot(WalletPolicyId[0..4], slot)` = `derive_mk1_chunk_set_id(stub) ^ slot` → **20-bit** field (mk-codec `MAX_CHUNK_SET_ID=(1<<20)-1`). Top 16 bits = the md1 prefix (shared bundle glue); XOR-slot disambiguates cosigners; ms1 reuses its slot's mk1 csi.

**Why this drives #27:** binding roots on the **key-significant** `WalletPolicyId`, so a keyless template hashes differently (test `walletpolicyid_template_only_differs_from_full_cell_7`) and silently fails to bind → D1 re-roots template bundles on the key-stable `WalletDescriptorTemplateId`. Fingerprints are excluded from that id, so D4 (no fingerprints in the template) is independent of binding.

---

## Decisions (user-confirmed unless noted)

- **D1 ✓ — whole-bundle template-id binding.** When `--md1-form=template`, re-root ALL three cards' stubs on `WalletDescriptorTemplateId` (not just the md1). A keyless md1 can't carry the key-significant `WalletPolicyId`, so mk1/ms1 must switch too or they won't bind. "Carry both" rejected (keyless md1 can't compute the key-dependent id). Side benefit: template- vs policy-form bundles are distinguishable + can't cross-mix.
- **D2 ✓ — single-sig first, then multisig.** Single-sig completion = derive `@0` from seed at the template origin (clean extension of today's single-sig restore). Multisig completion = graft N cosigner keys (funds-safety-heavy) → phase 2 / its own R0-heavy work.
- **D3 ✓ — explicit `--cosigner @N=<mk1>` (REQUIRED).** Architect-confirmed: the mk1 carries **no `@N`** (`KeyCard = {policy_id_stubs, origin_fingerprint, origin_path, xpub}`, `key_card.rs:24-54`); the `chunk_set_id` only yields slot *relative to* a `base` you can't get from the mk1 alone; and for a keyless template every auto-map signal collapses (no pubkeys to xpub-match, template stub ≠ the mint base since policy_id is key-significant). Operator-asserted mapping is the only safe option here.
- **D4 ✓ — template carries NO fingerprints + cross-check guard CONFIRMED.** Fingerprints don't aid mapping (identical in self-multisig, omitted in privacy mode, excluded from the id). **Guard (CONFIRMED 2026-06-19):** even with explicit `@N=`, restore cross-checks each grafted mk1's `origin_path` (+ `policy_id_stubs` vs the bundle's template id) against the template's `@N` origin and **REFUSES LOUDLY on mismatch** — so a mistyped `@N=` can't silently mis-graft. Possible without fingerprints (mk1 carries origin_path + stubs); meaningful for distinct-cosigner multisig (harmless in self-multisig — all keys identical).
- **D5 ✓ — verify-bundle COMPLETES the template** (recompose approved): verify the cards bind via the template id AND recompose the watch-only wallet.
- **D6 ✓ — proceed in the constellation/toolkit track; inform the SeedHammer instance after D1 lands; do NOT bundle the Go `WalletPolicyId`/template-id port** (separate gated fork cycle). SH T6 stays full-policy-only, waiting on this.

---

## Revision A (2026-06-19) — GOAL clarification + R0-round-1 (C1) resolution

**GOAL (user):** a template md1 is a backup of a wallet **TYPE** (e.g. "BIP-84 single-sig"), **shareable** so ONE engraving serves thousands of users of that type. This is *why* `WalletDescriptorTemplateId` excludes origin/account/keys — it's account-agnostic **by design**, so the BIP-84 template is identical for everyone.

**Phasing (LOCKED):**
- **Phase 1 — single-sig shareable template.** Account-agnostic. The custom **account/origin is supplied at RESTORE** (`--account N` / `--origin <path>`), not baked into the shareable template. (A user wanting a self-contained *personal* backup can still emit a template that carries their specific origin on the wire — `path_decl` is always written — but that's a personal md1, not the one-for-thousands artifact.) **Custom origins/accounts ARE supported** — the account just lives at restore, not on the shared plate.
- **Phase 2 — multisig template.** DEFERRED (its own R0-heavy cycle) and **depends on #25 shipping first** (override-bearing templates need #25's per-`@N` faithful reconstruction; single-sig templates carry no overrides, so phase 1 is independent of #25).

**C1 resolution.** The generic template binding (D1) is **intentional, not a collision bug** — being identical across all users of a type is the feature. Single-sig phase 1 is **safe by construction**: the seed deterministically derives the wallet, so a same-shape template is interchangeable and there's no wrong-wallet risk. The **specific-wallet disambiguation** is moved out-of-band to the recorded `WalletPolicyId` (D7). For multisig phase 2 (where keys come from external cosigner cards and the assembly must be validated), `--expect-wallet-id` (D7) is the C1 safety check.

- **D7 ✓ (NEW) — `WalletPolicyId` disambiguator.** At bundle time we have the full keyed descriptor, so `bundle --md1-form=template` prints the **`WalletPolicyId` on STDERR** as both hex and its 12-word BIP-39 phrase (`identity.rs:129 to_phrase()`) — "the shareable template doesn't identify YOUR wallet; record this id with the engraving." Advisory-only (mirrors `timelock_advisory`/`unrestorable_advisory`); does NOT change the engraved stdout. **Restore side:** `restore --expect-wallet-id <phrase|hex>` recomputes the `WalletPolicyId` from the completed wallet (template + seed/keys + account) and **refuses loudly on mismatch** — confirming the assembled wallet is the one backed up. Optional in phase 1 (seed is already ground truth), the primary multisig safety check in phase 2.

## Revision B (2026-06-19) — R0-round-2 folds (C2, I4) + D7 short-id

**C2 (NEW, from R0-r2) — the shareable template must EXPLICITLY elide the origin.** "One engraving for thousands" requires the phase-1 single-sig template md1 STRING to be byte-identical across users. That does NOT happen for free: `synthesize_descriptor`/`build_descriptor` write an explicit `m/84'/0'/account'` `path_decl`. The byte-shareable artifact is specifically a **canonical-origin, keyless, fingerprint-stripped md1 with an ELIDED (empty) `path_decl`** (empty origin is a valid wire form for canonical wrappers — `validate.rs:182-185`, `canonical_origin.rs:45-79`). FOUR descriptor mutations for `--md1-form=template`: `pubkeys:None`, `fingerprints:None`, **origin elided (empty `path_decl`)**, drop the `is_wallet_policy` assert. A non-canonical wrapper or user-baked custom origin → a **personal, non-shareable** md1 (out of the one-for-thousands scope; or refused). After these, no per-user residue remains → byte-identity round-trips. **This is the crux decision the spec must state.**

**I4 (from R0-r2) — pin D7's recompute invariant.** `compute_wallet_policy_id` is origin-significant and does NOT consult `canonical_origin` (`identity.rs:161-171`). So the D7 id MUST be computed from the **fully-keyed, EXPLICIT-origin, presence-`0b11`** completed descriptor — on **both** the `bundle` side (the keyed descriptor, not the elided template) and the `restore --expect-wallet-id` side (rebuild with explicit `m/84'/0'/account'` + fp + xpub). Add a `bundle`-id == `restore`-recomputed-id round-trip differential test, else a canonicalization mismatch → false refusal.

**D7 id form — FLEXIBLE-LENGTH, user-chosen (user, 2026-06-19).** `WalletPolicyId` is **16 bytes (128 bits)**; the 12-word form is just BIP-39's 128-bit rendering. D7 does NOT impose a fixed length: `bundle --md1-form=template` prints the **full** `WalletPolicyId` (hex + 12-word phrase + a short 4-byte prefix shown for convenience); **the user records as few or as many bytes as they want** (convenience ↔ collision-resistance — a few bytes already makes accidental collision among a user's wallets negligible; more = stronger verification). `restore --expect-wallet-id <prefix>` accepts **any-length prefix** and matches the recomputed id's leading bytes against it (length = the user's chosen assurance). 
- **Which id:** record the **`WalletPolicyId`** (key/account-specific = the per-wallet disambiguator; NOT reproducible from the keyless template → must be captured at bundle time). The **`WalletDescriptorTemplateId`** is the *type* id (same for everyone) and is **recomputable from the template md1** the user already holds → no need to record it (printing it is an optional convenience label only).

**R0-r2 Minors:** restore `--account` EXISTS today (`restore.rs:99-101`, derives at `:340-346`); `--origin` is net-new (thin wrapper over `derive_bip32_from_entropy_at_path`); "restore consumes a keyless template md1" is net-new (single-sig restore reads no md1 today). Advisory idiom = `secret_advisory::emit_output_class_advisory` (not `timelock/unrestorable_advisory`). `build_descriptor` (`synthesize.rs:131`) is dead test-only — mutate `synthesize_descriptor`, not it. D5 `--recompose` scope still to pin.

**Architect folds (I1–I3):**
- **I1:** D4's origin cross-check is NET-NEW (today it's a soft `warning:`, overlap-prefix only, `verify_bundle.rs:2454-2473`) → spec it as a hard refusal with EXACT-origin equality; it's a *partial* mitigation (origin-distinct cosigners only), with D7's `--expect-wallet-id` the primary boundary. (Single-sig phase 1: origin is supplied/derived, so this is a sanity check, not the boundary.)
- **I2:** D1's re-root must switch ALL `compute_wallet_policy_id` sites coherently for template form: md1 prefix (`bundle.rs:1082`), the csi base (`bundle.rs:1103-1106` → `synthesize.rs:61`, computed twice — simplify), the `stub_linkage` self-check (`bundle.rs:2187/2236`), and mk-cli `derive_stub_from_md1` (`mod.rs:63-65`) + fix the stale `mk-codec/key_card.rs:25-30` doc. Bind template form on the template id; D7's `WalletPolicyId` is the separate disambiguator (NOT the bundle stub).
- **I3:** multisig phase-2 depends on #25 (md-codec ≥0.37.0 + narrowed override guard); single-sig phase-1 is independent (no overrides). Toolkit MINOR; the 0.36→0.37 re-pin is a phase-2 release-ritual touch.

---

## Key architectural facts (recon + architect)
- mk1 payload has **no slot field** (`key_card.rs:24-54`; bytecode rejects trailing bytes `decode.rs:46-48`). Cosigners differ only by xpub/fp/origin (byte-identical in self-multisig) + the csi XOR.
- `csi = derive_mk1_chunk_set_id(stub) ^ slot` (`synthesize.rs:61`); 20-bit; slot recoverable only with `base` = the policy/template stub.
- Multisig `restore` requires a full-policy md1 today (`restore.rs:1232-1238`); single-sig restore derives from seed + `--template` and **never reads an md1** → template-completion is net-new for both, simpler for single-sig.
- `verify_bundle.rs:1778-1811` already xpub-matches mk1→slot for FULL md1 (has pubkeys); that avenue is unavailable for a keyless template.

## Scope / SemVer (from cycle-prep)
- LARGE cross-repo. **toolkit MINOR** (flag + restore/verify completion), **mk-cli MINOR** (`derive_stub_from_md1` template-id branch + fix stale `mk-codec key_card.rs:27` doc), **md-codec ~NO-BUMP** (id already public), **mnemonic-gui MINOR paired**.
- Mandatory locksteps: GUI `schema_mirror` flag-NAME + GUI dropdown-VALUE paired-PR + manual `41-mnemonic.md` flag-row (`docs/manual/tests/lint.sh`). Release ritual: BOTH READMEs + install.sh sibling pins.
- Ordering: md-codec (confirm) → mk-cli → toolkit → GUI → manual; SH Go last.
- **Funds-safety gate (template-completion):** wrong key graft = wrong wallet → address-equivalence differential + INDEPENDENT golden; refuse loudly on any `@N`/key mismatch.

## Status: BRAINSTORM R0-GREEN (round 3, 2026-06-19)
R0: round 1 RED (C1 + 3I) → Revision A → round 2 RED (C2 + I4) → Revision B + flexible-D7 → **round 3 GREEN (0C/0I)**. Reviews persisted: `design/agent-reports/template-only-brainstorm-r0-round{1,2,3}-review.md`. D1–D7 + Revisions A/B LOCKED. Spec-ready.

**M-flex (round-3 Minor, advisory only):** `restore --expect-wallet-id` accepts any-length prefix but the spec should ADVISE a minimum (~≥4 bytes / the printed convenience prefix) — an over-short prefix is a collision footgun (1 byte ≈ 1/256). Not enforced (user chose flexible). Phrase form is strictly the full 128-bit rendering (`phrase.rs` takes `[u8;16]`); sub-128-bit assurance is hex/bytes only.

## Next
1. → SPEC (gated on **freeze lift + go**). Phase-1 scope per the round-3 review's SPEC scope statement. Mandatory opus R0 → 0C/0I before any code. New `ToolkitError` variants alphabetical; re-grep citations at write time; mutate `synthesize_descriptor` NOT the dead `build_descriptor`.
2. SH coordination: hand the SH instance the D1 binding decision (re-root on `WalletDescriptorTemplateId`) when we start the spec.
3. Tracked as task #28 (gated).
