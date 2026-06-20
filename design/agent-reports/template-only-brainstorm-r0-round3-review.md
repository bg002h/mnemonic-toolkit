# Brainstorm R0 round 3 — bundle-md1-template-only-option (#27) (opus architect, verbatim)

> Reviewer: opus architect (Plan agent; read+bash). **Verdict: GREEN — 0 Critical, 0 Important, 1 non-blocking Minor.** Brainstorm spec-ready. UNCOMMITTED (freeze).

---

All confirmed. The I1 cross-check at `verify_bundle.rs:2454-2473` is a soft `writeln!(stderr, "warning: ...")` + `break` (continues, passes), overlap-prefix only (`.zip()` at :2454 stops at min len) — exactly as the brainstorm's I1 fold describes. `Phrase::from_id_bytes` takes a `[u8; 16]` (`phrase.rs:17`) and always renders 12 words — confirming the 12-word form is strictly the 128-bit rendering (a sub-128-bit prefix is only expressible as hex/bytes, not a shorter standard BIP-39 phrase).

## R0 round 3 — verification verdict: GREEN

### C2 (byte-identical shareable template) — CLOSED
(a) Empty `path_decl` valid+decodable for a canonical wrapper: `validate_explicit_origin_required` short-circuits `Ok(())` when `canonical_origin(&d.tree).is_some()` (`validate.rs:183-185`); `wpkh(@N)`→`Some(m/84'/0'/0')` (`canonical_origin.rs:50`); empty `OriginPath` encodes depth-0 and round-trips (`origin_path.rs:54-76`). (b) After the 4 mutations no residual per-user byte (header, empty path_decl, standard use-site, template tree, empty TLV — all user-independent; xpub/fp live only in the dropped TLVs) → byte-identical md1 across users of a type. (c) Explicit + complete for SPEC.

### I4 (D7 recompute precondition) — CLOSED, load-bearing
`compute_wallet_policy_id` is origin-significant, doesn't consult `canonical_origin` (`identity.rs:172-185`, INVARIANT `:161-171`). Traced via `expand_per_at_n` (`canonicalize.rs:437-455`): an elided template on a canonical wpkh wrapper resolves to the EMPTY origin and the `MissingExplicitOrigin` guard does NOT fire → it would hash an empty origin, differing from the explicit `m/84'/0'/0'` id. The fold's pin (compute D7 from the fully-keyed, explicit-origin, presence-`0b11` descriptor on BOTH sides + round-trip differential) is the correct, necessary closure. Bundle computes from the keyed descriptor (`synthesize.rs:272`/`bundle.rs:1082,1103`); restore can rebuild it (explicit origin via `--account`/`--origin` over `derive_slot.rs:65` + seed-derived xpub+fp). Presence-significance pinned `identity.rs:610-618`.

### D7 flexible-length — SOUND
Recording the POLICY id (not the template id, which is recomputable from the md1) is correct. Any-length leading-byte prefix matching against the recomputed 16-byte id is mechanically sound.
- **Minor M-flex (advisory only):** an arbitrarily short prefix is a footgun (1 byte → ~1/256 collide); the BIP-39 phrase form is strictly the full 128-bit rendering (`phrase.rs:13-19`). The user chose flexible, so the spec should carry an ADVISORY minimum (suggest ≥4 bytes / the convenience prefix already printed) rather than enforce one. Does not gate.

### Drift check — no new C/I
No fold contradicts D1–D7 or Revision A/B. D7's id is consistently the separate disambiguator, never the bundle stub. The empty-path_decl elision (C2) and the explicit-origin recompute (I4) are complementary (elided = engraved/shared artifact; explicit = ephemeral preimage for the advisory id — never confused, I4 mandates the explicit descriptor for the hash). Phase-1 single-sig scope internally consistent; multisig correctly deferred (C1 + #25); single-sig independent of #25 (no overrides).

### SPEC scope statement (phase 1)
Phase 1 ships a single-sig shareable template end-to-end: `bundle --md1-form=template` emits a canonical-origin, keyless, fingerprint-stripped md1 with an elided (empty) `path_decl` via exactly four mutations to `synthesize_descriptor` (`pubkeys:None`, `fingerprints:None`, empty `path_decl`, drop the `is_wallet_policy` debug-assert), re-roots all three cards' binding stubs on the key-stable `WalletDescriptorTemplateId` across every `compute_wallet_policy_id` site, and prints the full key/account-specific `WalletPolicyId` (hex + 12-word phrase + a convenience prefix) on STDERR as a D7 advisory; `restore` gains net-new keyless-template-md1 ingestion on the single-sig path with the existing `--account` plus a net-new `--origin` and an optional `--expect-wallet-id <any-length-prefix>` that recomputes the `WalletPolicyId` from the fully-keyed, explicit-origin, presence-`0b11` completed descriptor (never the elided template) and matches leading bytes; `verify-bundle` completes+recomposes the watch-only wallet; with GUI flag/dropdown and manual `41-mnemonic.md` locksteps. Multisig, override-bearing templates, and the #25-dependent reconstruction are deferred to phase 2.

**Verdict: GREEN** — both round-2 findings (C2, I4) closed, the D7 flexible-length fold sound, no new C/I. One non-blocking Minor (advisory minimum prefix length). Clear to proceed to SPEC.
