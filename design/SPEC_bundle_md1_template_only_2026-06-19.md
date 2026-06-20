# SPEC — bundle-md1-template-only-option (Phase 1: single-sig shareable template)

**Date:** 2026-06-19 · **UNCOMMITTED working draft** (SeedHammer freeze; commit when the freeze lifts). SeedHammer-driven; constellation/toolkit track owns the binding decision.
**Brainstorm (R0-GREEN, 3 rounds):** `design/BRAINSTORM_bundle_md1_template_only_2026-06-19.md` + `design/agent-reports/template-only-brainstorm-r0-round{1,2,3}-review.md`.
**Source SHAs (grep-verified):** mnemonic-toolkit `4e21d94`, descriptor-mnemonic `c85cd49`, mnemonic-key `913febc`, mnemonic-gui (pin per toolkit).
**SemVer:** toolkit **MINOR** (`0.58.x → 0.59.0`), mk-cli **MINOR** (template-id stub branch + doc), md-codec **NO-BUMP** (id + wire form already public), mnemonic-gui **MINOR paired** (flag + dropdown value).
**R0: round 1 RED (C1/C2/I1) → folded → round 2 GREEN (0C/0I).** Reviews: `design/agent-reports/template-only-spec-r0-round{1,2}-review.md`. The implementation plan-doc then takes its own per-phase R0 before any code (gated on the freeze lift + go).

## 0. Gate + funds-safety
Implementation is GATED on the SeedHammer first-pass freeze lifting + an explicit go. Funds-safety bar: template-completion must never silently produce a wrong wallet — the gates are (a) byte-identical shareable templates, (b) the D7 `WalletPolicyId` recompute-and-match invariant, (c) single-sig only (the seed *derives* the wallet — no external key graft, so the multisig "inversion" hole C1 cannot arise this phase).

## 1. Motivation / goal
A **template** md1 is a backup of a wallet **TYPE** (e.g. BIP-84 single-sig), **shareable** so one engraving serves thousands of users of that type. This cuts engraving plates — a keyless single-sig template ≈ 1 chunk vs a full policy's ~2-3 (`SINGLE_STRING_PAYLOAD_BIT_LIMIT = 320` bits, `md-codec chunk.rs:219`; a 65-byte xpub + 4-byte fp TLV overflows it). The user's specific wallet is identified separately by the recorded `WalletPolicyId` (D7).

## 2. Scope
### IN (phase 1)
- `bundle --md1-form=policy|template` (default `policy` = today). `template` emits a **keyless, fingerprint-stripped, canonical-origin (elided) single-sig** md1.
- The bundle's three cards (ms1/mk1/md1) re-root their binding stub on the key-stable `WalletDescriptorTemplateId` when form==template.
- D7: `bundle --md1-form=template` prints the key/account-specific `WalletPolicyId` on STDERR (flexible-length disambiguator to record with the engraving).
- `restore` single-sig template-completion: ingest a keyless template md1 + seed + `--account`/`--origin` → the watch-only wallet; optional `--expect-wallet-id <prefix>`.
- `verify-bundle` completes + recomposes a single-sig template bundle.
- GUI flag/dropdown + manual locksteps.

### OUT (deferred → phase 2, its own cycle)
- **Multisig** template-completion (C1: keyless multisig inverts the cross-check to a silent-wrong-wallet hole; needs a key/origin-discriminating template-bundle stub or `--expect-wallet-id` as the boundary) — AND depends on the in-flight **#25** per-`@N` reconstruction (override-bearing templates).
- **Override-bearing templates** (overrides are multisig-only).
- **Non-canonical wrappers / genuinely custom derivation paths** as a *personal* (origin-carrying) template — phase 1 REFUSES these for `template` form (use `--md1-form=policy`).

## 3. ID taxonomy + binding model (grounding)
Three 16-byte truncated-SHA-256 ids (`md-codec/identity.rs`): `Md1EncodingId` (`:11`, the exact encoding), `WalletPolicyId` (`:106`, key/account-specific — "this wallet"), `WalletDescriptorTemplateId` (`:47`, key/account-invariant — "this wallet TYPE"). Card binding today derives from `WalletPolicyId` (md1 prefix `[0..2]`; mk1/ms1 csi from `[0..4]`). **For template form, binding re-roots on `WalletDescriptorTemplateId`** so the keyless template and the keyed ms1/mk1 cards derive the SAME stub (a keyless md1 cannot carry the key-significant `WalletPolicyId`; pinned `walletpolicyid_template_only_differs_from_full_cell_7` `identity.rs:610`).

## 4. Design

### 4.1 The flag
`bundle --md1-form <policy|template>` (clap value-enum, default `policy`). New `Md1Form` enum. No behavior change when `policy`. This is a NEW clap flag + NEW dropdown value → triggers all three locksteps (§4.8).

### 4.2 Template emission (the byte-shareable artifact)
When `--md1-form=template`, the md1 is built from a **normalized-to-canonical, keyless** descriptor. Concretely, four mutations on the descriptor that `synthesize_descriptor` (`synthesize.rs:258`; NOT the dead test-only `build_descriptor:131`) feeds to `encode_md1_string`:
1. `tlv.pubkeys = None`,
2. `tlv.fingerprints = None`,
3. **`path_decl` origin ELIDED to empty** (`OriginPath { components: vec![] }`) — valid+decodable iff `canonical_origin(&d.tree).is_some()` (`validate.rs:183-185`, `canonical_origin.rs:45-79`; empty origin round-trips `origin_path.rs:54-76`),
4. drop the `debug_assert!(is_wallet_policy())` (`synthesize.rs:346`) for the template path.
After these, no per-user byte remains (header, empty path_decl, standard use-site, template tree, empty TLV are all user-independent) → **two users of the same type get a byte-identical template md1**, regardless of their account (the account is normalized away — it lives in D7 + restore `--account`).

**Canonical gate (C1).** `--md1-form=template` REQUIRES **`descriptor.n == 1 && canonical_origin(&d.tree).is_some()`**. The `n == 1` conjunct is LOAD-BEARING: `canonical_origin` ALONE returns `Some(...)` for canonical MULTISIG too (`wsh(multi/sortedmulti) → m/48'/0'/0'/2'`, `sh(wsh(...)) → …/1'`, `canonical_origin.rs:57-71`), so `is_some()` by itself would admit a 2-of-3 and re-open the deferred C1 keyless-multisig hole. With `n==1` the only `Some` shapes are pkh/wpkh/tr-keypath single-sig (every multi/sortedmulti is `n≥2`; bare single-key wsh returns `None`, `canonical_origin.rs:234`). Refuse on EITHER `n>1` OR `is_none()` with a clear message ("template form supports standard single-sig wallet types only; use `--md1-form=policy`"). New `ToolkitError::TemplateFormUnsupportedShape` (alphabetical, between `SlotInputViolation` and `UnknownHrp`). Edge note: a degenerate `wsh(multi(1,@0))` (1-of-1 via the multi tag) is `n==1`+canonical → slips the `n==1` gate, but it carries exactly one key the seed derives — NOT the C1 inversion; the plan-doc may add a `multi`-tag guard or a test pin, non-blocking.

### 4.3 Binding-stub re-root (I2) — what's in the STRINGS vs the labels
**Which strings reflect the template id (answers "will ms1/mk1 reflect it"):**
- **mk1 string — YES.** `synthesize.rs:180/216/273` compute `stub = policy_id.as_bytes()[..4]` and use it for BOTH the mk1 `KeyCard` payload (`:186/222/280`, the `policy_id_stubs`) AND its encoded `chunk_set_id` (`:192/228`, `derive_mk1_chunk_set_id_for_slot` `:61`). For template form these MUST use `WalletDescriptorTemplateId.as_bytes()[..4]`. So the mk1 STRING reflects the template id (chunk_set_id header + payload stub); its only user-specific content is the xpub.
- **ms1 string — NO.** `ms_codec::encode(...)` (`synthesize.rs:172`) emits plain codex32 entropy + checksum with NO id field — unchanged by form. Only the ms1 *card-id LABEL* (`bundle.rs:1117`) reflects the template-id value (display metadata, not in the string).
- **md1 string — YES** (it IS the template; the engraved md1 card-prefix label is `bundle.rs:1082`).

So template form rewrites the **md1 + mk1 strings** (+ display labels), NOT the ms1 string.

**Switch ALL stub-derivation from `compute_wallet_policy_id` → `compute_wallet_descriptor_template_id` coherently** at the PRODUCTION sites (M1/M2 — `synthesize_descriptor:272-274` is the live string-level stub; `:290` the live n==1 csi; `synthesize.rs:180/216/192/228` are `#[allow(dead_code)]` test helpers, not phase-1 production): the string-level stub (`synthesize.rs:272`→KeyCard + csi `:290`), the display labels (`bundle.rs:1082`, `:1103-1106`), and mk-cli `derive_stub_from_md1` (§4.7). Compute the id ONCE (fold the current double-recompute). The D7 `WalletPolicyId` (§4.4) is the SEPARATE disambiguator, NOT the bundle stub.

**Bundle self-check must BRANCH for template form (I1).** `self_check_bundle` (`bundle.rs:2139`) has production gates a keyless template trips before the stub check: (1) `bundle.rs:2151-2156` `if !desc.is_wallet_policy() → Err(BundleMismatch)` (a separate, NON-debug gate from the `synthesize.rs:346` assert §4.2 drops); (2) `bundle.rs:2171-2177` `pubkeys.is_none() → Err("cannot bind mk1 xpubs")` → `check_mk1_xpub_binding` (`:2186/2220`) — structurally inapplicable to a keyless template (no md1 xpubs to bind against). For template form: (a) skip/branch the `:2151` `is_wallet_policy` gate, (b) skip the `:2171` pubkeys-absent refusal + `check_mk1_xpub_binding`, (c) compute `expected_stub` (`:2158/2162/2187/2236`) from `compute_wallet_descriptor_template_id`. The template self-check verifies stub-coherence + mk1 origin/fingerprint — NOT md1-xpub binding. (Line numbers re-grepped at plan-doc time per §9.)

### 4.4 D7 — `WalletPolicyId` stderr disambiguator (I4)
At bundle time we hold the fully-keyed descriptor. `bundle --md1-form=template` prints on STDERR (via the `secret_advisory::emit_output_class_advisory` idiom, `bundle.rs:1035`) the **`WalletPolicyId` computed from the FULLY-KEYED, EXPLICIT-origin (`m/84'/0'/account'`), presence-`0b11` descriptor** — NOT the elided template (I4: `compute_wallet_policy_id` is origin-significant and ignores `canonical_origin`, `identity.rs:161-185`). Render: full hex + 12-word phrase (`identity.rs:129 to_phrase()`) + a short convenience 4-byte prefix. Advisory-only; does NOT change the engraved stdout. Message: "the shareable template does not identify YOUR wallet — record this id with the engraving (as many bytes as you wish)." **Flexible-length:** the user records any prefix.

### 4.5 restore — single-sig template-completion
- **Ingestion routing (net-new, C2).** Today `restore.rs:177-179` is `if !args.md1.is_empty() { return run_multisig(...) }`, and `run_multisig` hard-refuses any keyless md1 at `restore.rs:1232-1236` (`if !d.is_wallet_policy() { ModeViolation "needs a wallet-policy md1" }`). So a `--md1 <template>` does NOT reach any single-sig path today — it hits that refusal. **Carve-out:** at the `:177-179` dispatch, reassemble the md1 and branch — if it is a **keyless single-sig template** (`!d.is_wallet_policy() && d.n == 1 && canonical_origin(&d.tree).is_some()`), route to the NEW single-sig completion (below); otherwise keep today's behavior (`run_multisig`; its `:1232` refusal then correctly catches a keyless *multisig* template). The template provides the script type + use-site; the seed provides the key; `--account`/`--origin` provide the origin.
- **`--account`** (exists, `restore.rs:99-101`, default 0) selects the account. **`--origin <path>`** (NEW, thin wrapper over `derive_bip32_from_entropy_at_path` `derive_slot.rs:65`) selects an arbitrary origin. The template's canonical script type + the supplied account/origin → the explicit `m/84'/0'/account'` descriptor → derive the key → the concrete watch-only descriptor.
- **`--expect-wallet-id <prefix>`** (NEW, optional): restore recomputes the `WalletPolicyId` from the **fully-keyed, explicit-origin, presence-`0b11`** completed descriptor (the same preimage as §4.4 — invariant pinned) and compares its leading bytes against the supplied prefix; **refuses loudly on mismatch** (`ModeViolation`/exit 4). Any-length prefix; **advisory minimum ≥4 bytes** (M-flex — warn, don't enforce, on a shorter prefix).

### 4.6 verify-bundle
For a template bundle, replace the current vacuous template-only skip (`verify_bundle.rs:2152-2167`) with: verify the three cards bind via the template-id stub, then **complete + recompose** the watch-only wallet (single-sig: seed + `--account`/`--origin`) and assert internal consistency. `--expect-wallet-id` supported here too (same recompute-and-match).

### 4.7 mk-cli (consistency)
`mk-cli derive_stub_from_md1` (`mod.rs:63-65`) gains a template-id branch keyed on **`!descriptor.is_wallet_policy()`** (the correct discriminator — M4: `compute_wallet_policy_id` on a keyless template does NOT error, it silently hashes the template-only/empty-origin preimage, so without the explicit branch it would mis-stub): for a keyless template md1, derive the stub from `compute_wallet_descriptor_template_id`, so `mk encode`/`mk verify` agree with the toolkit on template-bundle stubs. Fix the stale docs at BOTH `mk-cli mod.rs:55-62` ("top 4 bytes of the policy's WalletPolicyId") and `mk-codec/key_card.rs:25-30` ("SHA-256(canonical_bytecode)") — neither is accurate for the template branch.

### 4.8 Locksteps (mandatory)
- **GUI `schema_mirror`:** add the `--md1-form` flag-NAME to the GUI `bundle` SubcommandSchema (`mnemonic-gui/src/schema/mnemonic.rs` `BUNDLE_FLAGS`, ~:3776) + the `policy|template` dropdown VALUES (paired-PR — schema_mirror gates flag-NAMES not values) + GUI pin bump.
- **Manual:** add the `--md1-form` row + a `### Template-only md1` section to `docs/manual/src/40-cli-reference/41-mnemonic.md`; `docs/manual/tests/lint.sh` flag-coverage; run `make -C docs/manual audit`.

## 5. Test / oracle strategy (RED-first; funds-safety gate)
1. **Byte-identity (the goal):** two DIFFERENT seeds, same wallet type + same account-shape → `bundle --md1-form=template` produces **byte-identical** md1 strings. And across different accounts (account 0 vs 5) of the same type → still byte-identical (account normalized away).
2. **D7 round-trip differential:** `bundle --md1-form=template` printed `WalletPolicyId` == `restore`-recomputed `WalletPolicyId` for the same seed+account (proves the I4 same-preimage invariant). Mismatch → false-refusal regression guard.
3. **Completion address-equivalence:** `bundle --md1-form=template` → `restore --md1 <template> --from <seed> --account N` → derived addresses == the original full-policy wallet's addresses (independent golden / Core `deriveaddresses`).
4. **`--expect-wallet-id`:** correct prefix → pass; wrong prefix → loud refuse; over-short prefix → advisory warning (M-flex).
5. **Refusals (BOTH ends):** `--md1-form=template` on a non-canonical wrapper / multisig / `n>1` / custom non-standard path → clear refusal at **bundle-emit** (the §4.2 `n==1 && canonical` gate); AND a keyless **multisig** template `--md1` → refusal at **restore-ingest** (the §4.5 carve-out falls through to `run_multisig`'s `restore.rs:1232` `ModeViolation`). Pin both arms.
6. **Binding + self-check:** a template bundle's mk1/md1 share the template-id stub (ms1 unchanged); a template md1 does NOT bind to a *policy*-form mk1 (different stub) and vice-versa; AND `self_check_bundle` (`bundle.rs:2139`) PASSES for a single-sig template bundle (the I1 branch — must not trip the keyless `is_wallet_policy`/pubkeys-absent gates).
7. **Non-regression:** `--md1-form=policy` (default) is byte-identical to today for all existing corpus bundles.

## 6. SemVer / version sites / ordering
- **toolkit MINOR** (`0.58.1 → 0.59.0`): new flag + restore/verify completion. Version sites: Cargo.toml, BOTH READMEs, `scripts/install.sh`, fuzz/Cargo.lock, Cargo.lock, CHANGELOG.
- **mk-cli MINOR** (`0.9.x → 0.10.0` or `0.9.1`-MINOR per its convention): stub template-id branch + doc. **md-codec NO-BUMP** (no source change — id + form pre-exist; the toolkit/mk consume existing public API).
- **GUI MINOR paired:** schema flag/value + pin bump.
- **Ordering:** md-codec (confirm, no change) → mk-cli (publish if its stub is consumed standalone) → toolkit (flag + completion) → GUI paired-PR → manual `make audit`. No #25 dependency (single-sig carries no overrides).
- **Locksteps:** §4.8. New `ToolkitError` variant(s) alphabetical. Re-grep all citations at plan-doc time.

## 7. Deferred (phase 2 — separate cycle)
Multisig template-completion (the discriminating template-bundle stub `H(WalletDescriptorTemplateId ‖ sorted cosigner-fingerprints)` OR `--expect-wallet-id` as the boundary; D4 origin cross-check as a partial mitigation; explicit `--cosigner @N=<mk1>` graft with origin cross-check refusal) + override-bearing templates + the #25 dependency + the personal/custom-path template. Tracked in the brainstorm Revision A/B.

## 8. Risks / R0 focus
- **Funds-safety:** the completion must round-trip to the SAME addresses (§5.3) and the D7 recompute must use the EXACT same preimage on both sides (§5.2). R0/impl-review weight here.
- **Account normalization:** confirm elide-to-canonical truly drops the account for ALL canonical types (44/49/84/86 × network) and that restore's `--account`/`--origin` reconstruct the explicit origin that D7 hashes.
- **Stub coherence:** the four re-root sites (§4.3) + mk-cli (§4.7) must all switch, or a template bundle fails its own self-check / cross-tool verify.
- Per-phase R0 gates the plan-doc and each implementation phase before code.

## 9. Citation-decay
Line numbers are snapshots at the header SHAs; re-grep against current `origin/master`/`origin/main` at plan-doc time (CLAUDE.md). Note local is 1 commit ahead (the parallel SeedHammer `bundle-md1-template-only-option` FOLLOWUP, unpushed `4e21d94`); re-baseline at execution.
