# Brainstorm R0 round 2 — bundle-md1-template-only-option (#27) (opus architect, verbatim)

> Reviewer: opus architect (Plan agent; read+bash). **Verdict: RED — 1 Critical (C2), 1 Important (I4), 5 Minor.** Folded via Revision B. UNCOMMITTED (freeze).

---

## R0 Round 2 Review — Revision A + D7

**Verdict: RED — 1 Critical, 1 Important, 5 Minor.** The C1 fold is genuinely sound and closes the silent-wrong-wallet hole for what ships. I1/I2/I3 folds are correct. But the re-scope's load-bearing new premise — "one engraving for thousands" requires the phase-1 template md1 to be **byte-identical** — is not yet established by the brainstorm, and the code shows it does not happen for free.

### Round-1 findings closed?
- **C1 — CLOSED for phase-1 single-sig.** Single-sig restore derives the wallet from the seed and never reads an md1 for keys (`derive_bip32_from_entropy` `restore.rs:340-346` → `derive_slot.rs:42-57`). No external key graft → the C1 inversion cannot arise. A same-shape template is interchangeable; deferring multisig + #25 removes the hole from what ships.
- **I1 — CLOSED.** The existing origin cross-check is soft `warning:` + `break`, overlap-prefix only (`verify_bundle.rs:2454-2473`). Revision A correctly downgrades D4 to a phase-1 sanity-check + partial phase-2 mitigation.
- **I2 — CLOSED, with a compounding stale-doc note.** Four switch-sites real (`bundle.rs:1082`, `:1103-1106`→`synthesize.rs:290/310`, `:2187/2236`, mk-cli `mod.rs:63`). For phase-1 single-sig these are STILL in scope (the keyless template binds mk1/ms1 via the stub, which must re-root on `WalletDescriptorTemplateId` — template-only `WalletPolicyId` differs, `identity.rs:610-617`). `derive_stub_from_md1` already uses `compute_wallet_policy_id`, so the template-id branch is net-new. `key_card.rs:25-30` doc stale in two layers.
- **I3 — CLOSED.** Single-sig templates carry no overrides → phase-1 independent of #25. Multisig phase-2 → #25 stands.

### CRITICAL
**C2 (NEW) — "byte-identical shareable template" is asserted, not yet designed; the elision it depends on is encoder-controlled and the current bundle path does the opposite.**
- (a) Origin elision exists but only for canonical shapes, not automatic. `OriginPath{components:vec![]}` encodes depth-0 (`origin_path.rs:54-66`); for a canonical wrapper an empty `path_decl` is valid+decodable (`validate.rs:182-185`, test `:474-481`). Round-1's "origin mandatory on the wire" was the wrong inference.
- (b) The truly-shareable artifact is specifically the canonical-origin, account-0-shape template. A custom account → a different (personal) md1 (`parse_descriptor.rs:204-238` populates path_decl from the annotation; `compute_wallet_policy_id`/`encode_payload` hash/emit it verbatim).
- (c) The blocker: the current production path does NOT elide and is keyed. `synthesize_descriptor` (`synthesize.rs:258-348`) builds with `pubkeys:Some`, `fingerprints:Some`, explicit `path_decl` from `md_origin_path(network,account)` (`build_descriptor:140,153-154`), `debug_assert!(is_wallet_policy())` `:346`. The template needs FOUR net-new mutations: `pubkeys:None`, `fingerprints:None`, **origin elided (empty path_decl) — NOT stated in Revision A**, drop the assert. Without step 3 each user's template carries their account → the goal silently fails. After 1-4, no residual per-user byte → byte-identity round-trips.

### IMPORTANT
**I4 (NEW) — D7's recompute-and-match has an unstated canonicalization precondition that can silently fail.** `compute_wallet_policy_id` is origin-significant and does NOT consult `canonical_origin` (`identity.rs:161-171`); the elided-vs-explicit convergence (`walletpolicyid_stable_across_origin_elision` `:571-588`) holds only because the test supplies the canonical path via `OriginPathOverrides`. So: the D7 id must be computed from the user's keyed, EXPLICIT-origin descriptor (NOT the elided template) at bundle; and `restore --expect-wallet-id` must rebuild with explicit `m/84'/0'/account'` + fp + xpub (presence `0b11`) to match. Pin "D7 id = `compute_wallet_policy_id` of the fully-keyed, explicit-origin, presence-`0b11` completed descriptor, both sides" + a bundle-id == restore-recomputed-id round-trip differential.

### Answers to specific Qs
- **Q4:** `--account` EXISTS today (`restore.rs:99-101`, derives at `:340-346`); `--origin` is net-new (thin wrapper over `derive_bip32_from_entropy_at_path` `derive_slot.rs:65`). "Restore consumes a keyless template md1" is genuinely net-new (single-sig restore reads no md1; `--template` is the wallet-TYPE enum, not an md1).
- **Q5:** D7 feasible+sound with the I4 precondition.
- **Q6:** the D1 re-root IS still phase-1 scope (mk1/ms1 bind via the stub; can't be the keyed `WalletPolicyId`). D7 does NOT make the stub question moot — D7 is the out-of-band disambiguator (NOT the bundle stub).

### Minors
- M1 citation drift `restore.rs:177-179`→`:340-346`. M2 advisory idiom is `secret_advisory::emit_output_class_advisory` not `timelock/unrestorable_advisory`. M3 `key_card.rs:25-30` stale two layers. M4 `build_descriptor` (`synthesize.rs:131`) is dead test-only — mutate `synthesize_descriptor`. M5 D5 `--recompose` of a keyless template verifies almost nothing without a key source — pin scope.

### What turns this GREEN
1. **C2:** explicit Revision-A decision — phase-1 template = canonical-origin, keyless, fingerprint-stripped md1 with elided (empty) `path_decl`; name it the byte-shareable artifact; non-canonical/custom-origin = personal/out-of-scope; enumerate the four `synthesize.rs` mutations + lifting the assert.
2. **I4:** pin the D7 invariant (fully-keyed, explicit-origin, presence-`0b11`, both sides) + round-trip differential.
Re-run R0 round 3.
