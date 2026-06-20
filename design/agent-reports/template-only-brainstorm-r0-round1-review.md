# Brainstorm R0 round 1 — bundle-md1-template-only-option (#27) (opus architect, verbatim)

> Reviewer: opus architect (Plan agent; read+bash). **Verdict: RED — 1 Critical, 3 Important, 5 Minor.** UNCOMMITTED (freeze). Pre-SPEC gate; findings to be folded after user decisions on C1/phasing.

---

The GUI schema's bundle block exists (the recon's line 3776 is approximately right — the bundle SubcommandSchema is present with `BUNDLE_FLAGS`). The toolkit pins md-codec at `0.36`. mk-codec is `0.4.0`, mk-cli `0.9.0`.

# Pre-SPEC R0 Review — `bundle-md1-template-only-option` (#27)

**Verdict: RED.** 1 Critical, 3 Important. The brainstorm's headline ("the hard part already exists") is correct for the *binding-id* and *wire-form* legs, and D1's re-root is sound mechanically. But the brainstorm under-models the **funds-safety inversion** in multisig template-completion: when the md1 is keyless, the external mk1 keys become the *source of truth* rather than the *thing being verified*, and the existing cross-checks (which all assume the md1 carries ground-truth `tlv.pubkeys`) become circular. D4 as written ("refuses loudly") describes behavior that **does not exist today** (the existing origin check is a soft `warning:`, not a refusal) and **cannot fully close the hole** because the template deliberately excludes the discriminators (keys + origins) needed to distinguish two wallets of the same shape. These must be folded in before SPEC.

I verified every cited file:line against the source at the stated SHAs (descriptor-mnemonic `c85cd49`, mnemonic-key `913febc`, toolkit HEAD `4e21d94`). Citation accuracy is high; the id-taxonomy table is correct.

## CRITICAL

### C1 — Multisig template-completion has a silent-wrong-wallet hole: the cross-check inverts to a no-op (D2/D4/D5).

Today's multisig `restore` model: **the md1 carries the keys as ground truth** (`restore.rs:1307` pulls `c.key65` from `e.xpub` ← `desc.tlv.pubkeys`), and `--from`/`--cosigner @N=` are *verified against that ground truth*:
- `restore.rs:1465` — own seed: derive own key at the md1's origin, match the md1's pubkey.
- `restore.rs:1533` — `--cosigner @N=mk1`: `supplied65 != c.key65` → hard `RestoreMismatch` (exit 4, `:1550-1558`).

For a **keyless template** md1, `desc.tlv.pubkeys` is `None`, so there is no `c.key65` ground truth. Under D2-multisig the grafted mk1 keys *become* `c.key65`. The `:1533` check then compares the grafted key against itself — **vacuous**. The `:1465` own-seed check only proves the seed matches the mk1 the operator handed in, **not that this is the right wallet**. **In policy mode the bundle binds keys and external sources verify; in template mode external sources *provide* the keys and nothing validates the assembly.** A wrong/swapped/foreign mk1 graft produces a different but *internally consistent* watch-only wallet, silently.

D4's origin cross-check does NOT close this:
1. **The template id excludes BOTH keys AND origin** (`identity.rs:47-53`, pinned by `wdt_id_invariant_to_origin_path_change` `:325-338` + `wdt_id_invariant_to_fingerprint_addition` `:354-362`). Two different wallets — same K-of-N tree + use-site + overrides, different keys *and* origins — share the **same** `WalletDescriptorTemplateId`, hence the same D1 stub. The stub cannot discriminate them (the binding-collision is real; the discriminator is deliberately hashed out).
2. Per-`@N` origins survive on the wire (`encode.rs:85` always writes `path_decl`; mandatory) so D4 is *feasible* — but origins only discriminate when cosigners have **distinct** origins. The dominant shape is `PathDeclPaths::Shared` (`synthesize.rs:588-591`, `816-819`) — self-multisig or any wallet with all cosigners at `m/48'/0'/0'/2'`. There every `@N` origin is identical, the origin cross-check is a structural no-op, and C1's circular self-check is the only "validation" left.

**Design change required.** Either: **(a)** fold a key/origin-significant discriminator into the template *bundle* stub — e.g. `H(WalletDescriptorTemplateId ‖ sorted per-@N origin-fingerprints)` or `‖ xpub-fingerprints` — restoring per-wallet binding (D1's "bare template-id" is **insufficient**; the mk1/ms1 stub must be the template-*bundle* stub). OR **(b)** require an out-of-band `--expect-wallet-id` (the original `WalletPolicyId` 12-word phrase) that restore recomputes from the assembled keys and matches. OR **(c)** defer multisig template-completion to phase 2 citing C1; phase-1 single-sig is safe because the seed deterministically *derives* the one key (no external graft, no inversion).

## IMPORTANT

### I1 — D4 "REFUSES LOUDLY on mismatch" describes behavior that does not exist; nearest facility is a soft `warning:`.
The existing origin cross-check (`verify_bundle.rs:2454-2473`) only `writeln!(stderr, "warning: …")` and `break` — it continues and passes. The restore `--cosigner` hard-gate (`restore.rs:1550-1558`) is the *key* comparison (C1: vacuous for templates), not an origin comparison. No existing code refuses on mk1-origin-vs-template-origin mismatch. Also the existing check is **overlap-prefix only** (`:2441-2448`, zips to `min(len)`). **Fold:** D4 must specify (i) a new hard refusal (not warning), (ii) exact-equality on origins for the template-graft path, (iii) that it catches mistyped `@N=` only across *origin-distinct* cosigners → partial mitigation, not the primary boundary (C1 is).

### I2 — Stub-derivation hardwired to `compute_wallet_policy_id` at multiple sites; D1's re-root must switch all coherently.
- `bundle.rs:1082` — md1 card prefix `pid[0..2]`.
- `bundle.rs:1103-1106` — the csi base `pid[..4]` → `derive_mk1_chunk_set_id_for_slot` (`synthesize.rs:61`). (bundle.rs computes the id twice — simplification opportunity.)
- `bundle.rs:2187`/`:2236` — the self-check `stub_linkage` gate (refuses if mk1 `policy_id_stubs` ∌ `expected_stub`); `expected_stub` must recompute on the template/template-bundle stub when form==template, else every template bundle self-check fails.
- `mk-cli mod.rs:63-65` `derive_stub_from_md1` (used by `mk encode`/`verify`) needs the template-id branch. **Stale-doc target correction:** `mod.rs:55-62` is correct; the genuinely stale doc is `mk-codec/key_card.rs:25-30` ("top 4 bytes of the policy's SHA-256(canonical_bytecode)", line 28). **Fold:** enumerate all four switch-sites + the key_card.rs doc; define the template-bundle-stub formula once.

### I3 — md-codec NOT NO-BUMP given #25; ordering mis-stated.
A template can carry `use_site_path_overrides` (they ARE in the template id, `identity.rs:79-98`), so multisig template-completion needs the in-flight #25 cycle's per-`@N` faithful reconstruction (md-codec 0.37.0 + the narrowed `restore.rs:1247` guard). **#27 depends on #25 landing first.** If #27 ships first, an override-bearing template either hits the `:1247` refusal (can't complete) or grafts onto the baseline-collapsed path (the #25 silent-wrong-derivation bug). **Fold:** state the hard #25 → #27 dependency, the md-codec ≥0.37.0 re-pin, and decide whether phase-1 supports override-bearing templates or defers them (rides #25's non-taproot scope). [NOTE: single-sig templates cannot carry overrides — overrides are multisig-only — so phase-1 single-sig is independent of #25.]

## MINOR
- **M1** id-taxonomy table accurate (verified `identity.rs` `:16/:55/:115`, preimages + invariances + `.to_phrase()` `:129`). K-of-N enters template-id via `Multi{k,indices}` + `kiw` — "this kind of wallet" correctly includes K-of-N.
- **M2** plate motivation sound: `SINGLE_STRING_PAYLOAD_BIT_LIMIT=64*5=320` (`chunk.rs:219`), `chunks=bits.div_ceil(320)` (`:249`).
- **M3** D5 `--recompose`: the current template-only path (`verify_bundle.rs:2152-2167`) SKIPS `md1_xpub_match` as a vacuous pass; D5 must replace it with actual completion+recompose, and pin what a keyless `--recompose` with no key source actually verifies (≈ "the template decodes" — almost nothing). Specify whether it reuses restore's `@N=` grafting (inherits C1) or is keyless-recompose.
- **M4** D3 (`--cosigner @N=`) re-confirmed correct (`key_card.rs:24-54` no slot field; csi relative; xpub-match auto-map collapses `verify_bundle.rs:1824-1832`). But its safety now leans entirely on the C1 fix — without a discriminating stub, `@N=` is asserted not verified.
- **M5** GUI/manual locksteps confirmed (GUI `bundle` SubcommandSchema + `BUNDLE_FLAGS`; dropdown-VALUE paired-PR + manual `41-mnemonic.md` + `docs/manual/tests/lint.sh`). Toolkit/mk-cli MINOR correct; toolkit re-pin 0.36→0.37 is a release-ritual touch.

## What turns this GREEN
1. **C1:** specify a key/origin-significant template-*bundle* stub (a), OR `--expect-wallet-id` recompute-and-match (b), OR explicitly DEFER multisig template-completion to phase 2 citing C1 — and state phase-1 single-sig is safe because the seed *derives* the one key (no external graft; `restore.rs:177-179` single-sig path).
2. **I1:** D4 = net-new hard refusal, exact-origin equality, downgraded to "partial mitigation, origin-distinct cosigners only."
3. **I2:** enumerate the four `compute_wallet_policy_id` switch-sites + `key_card.rs:25-30` doc; define the template-bundle-stub formula once.
4. **I3:** declare #25 → #27 dependency + md-codec ≥0.37.0 re-pin; decide override-bearing-template handling for phase 1.
