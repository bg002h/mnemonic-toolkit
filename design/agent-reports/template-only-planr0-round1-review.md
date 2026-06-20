# Plan-doc R0 round 1 — bundle-md1-template-only-option (opus architect, verbatim)

> Reviewer: opus architect (read+bash; pins md-codec 0.36.0 == c85cd49, mnemonic-key 913febc). **Verdict: RED — 0 Critical, 2 Important, 6 Minor.** Folded → re-dispatch. UNCOMMITTED (freeze).

---

# R0 Review — IMPLEMENTATION PLAN `bundle_md1_template_only_2026-06-19` (Phase 1)

**Verdict: RED — 0 Critical, 2 Important, 6 Minor.** Architecture sound; the closed SPEC findings (C1/C2/I1) are correctly carried with accurate edit-site anchors; every codec/API/binding claim checked is TRUE. Two Important gaps would not produce a wrong wallet but an implementer coding verbatim would stall or under-build them — fold before GREEN.

## Verified correct (load-bearing)
- **§1 NO-publish:** `compute_wallet_descriptor_template_id` public in pinned md-codec 0.36.0 (`identity.rs:71-103`); `compute_wallet_policy_id:172`, `to_phrase:129`. `mk_codec::encode_with_chunk_set_id`/`KeyCard::new` live (`synthesize.rs:280-291`). Toolkit does NOT call mk-cli `derive_stub_from_md1` → P2 genuinely parallel. Confirmed.
- **Edit sites LIVE:** `synthesize_descriptor:258` (stub `:272`, csi `:290`, assert `:346`); dead helpers `build_descriptor:131`/`synthesize_full:164`/`synthesize_watch_only:207` correctly flagged. Display labels `bundle.rs:1082`/`:1103`. `self_check_bundle:2139` gates `:2151/:2158→:2162/:2171/:2186/:2220`, stub `:2187/2236`. Restore `:177-179` + `run_multisig:1198` reassemble `:1229` + refusal `:1232-1238`. `--account:99-101`, derive `:340-346`, `derive_bip32_from_entropy_at_path:65`, `verify_bundle.rs:2152-2167`. All present + correctly attributed.
- **C1 gate load-bearing:** `canonical_origin` Some for `wsh(multi)→m/48'/0'/0'/2'` (`:58-62`), `sh(wsh(multi))→…/1'` (`:65-74`); `Descriptor.n` = placeholder count (`encode.rs:19`) → `n==1` excludes k-of-m. Confirmed.
- **D7/I4 same-preimage:** `compute_wallet_policy_id` INVARIANT (`identity.rs:161-171`) confirms eliding needs canonical_origin re-intro → the fully-keyed explicit-origin presence-0b11 both-sides hash is correct; both sides can build that Descriptor. Confirmed.
- **Byte-identity oracle (P1.2):** `validate_explicit_origin_required` no-op when canonical (`validate.rs:184-186`); account normalized away → testable as written.
- **Version sites/SemVer:** `0.58.1` at `Cargo.toml:3`/`README.md:13`/`crates/.../README.md:9`/`scripts/install.sh:32`; lockfiles + CHANGELOG exist. `TemplateFormUnsupportedShape` between `SlotInputViolation` (`error.rs:306`) and `UnknownHrp` (`:313`). GUI `BUNDLE_FLAGS` `mnemonic.rs:190`/`:3778`. Correct.

## IMPORTANT
**I1 — P1.2/P1.3 omit the `synthesize_unified` front-half + form-threading + the by-ref mutation impossibility.** The standard path `bundle --template bip84 --md1-form=template` goes `run → synthesize_unified:421/776`, which BUILDS the descriptor with `pubkeys=Some`, `fingerprints=Some`, populated `path_decl` (`synthesize.rs:835-850`) then delegates to `synthesize_descriptor:859`. `synthesize_descriptor` takes `descriptor: &Descriptor` by SHARED REF (`:259`) → cannot mutate in place; and neither fn takes an `Md1Form` param. **Fix:** (a) thread `Md1Form`/`bool template_form` through `synthesize_unified:776 → synthesize_descriptor:258` AND the descriptor-mode callers (`bundle.rs:1616/1726/1969`); (b) apply the 4 mutations on a `descriptor.clone()` gated on form (in `synthesize_descriptor` or `synthesize_unified:835-850` pre-delegation); (c) confirm stub/csi (`:272/:290`) + dropped assert (`:346`) read the MUTATED clone.

**I2 — P1.6 restore carve-out under-specifies tree→type resolution, `--from`-required, and the restore-side typed Descriptor.** (1) single-sig restore iterates all 4 templates (`:328-331,339`) or `--template`; a template md1 encodes ONE type in `d.tree` → the carve-out must map `d.tree → CliTemplate` (inverse of `script_type_from_template:350`). (2) `--from` is `required_unless_present="md1"`, so `restore --md1 <template>` with NO `--from` is clap-valid → routes to watch-only `run_multisig`; the completion arm REQUIRES the seed → must explicitly REJECT missing `--from` (else a no-seed template restore silently mis-routes — the one residual funds-safety hole). (3) `--expect-wallet-id` recompute needs a fully-keyed explicit-origin presence-0b11 typed `md_codec::Descriptor`, but restore builds a STRING via `build_descriptor_string:387` → the plan must name the typed-Descriptor builder (`build_descriptor:131` shape) so the D7 preimage is byte-identical to the bundle side. **Fix:** add (a)/(b)/(c) to P1.6.

## MINOR
- **M1 (most consequential):** §3/§P2.1 mislabel mk-cli as "(descriptor-mnemonic)" — it's **mnemonic-key** `crates/mk-cli/src/cmd/mod.rs:63-69`; stale docs `mod.rs:55-62` + `mnemonic-key/crates/mk-codec/src/key_card.rs:25-30`. (Line numbers exact; repo label wrong.)
- **M2:** §1/§P1.3 cite ms1 `ms_codec::encode` at `synthesize.rs:172` (dead `synthesize_full`); LIVE ms1 encode `:339`. Claim correct.
- **M3:** `expected_stub` assignment is `:2162` (`:2158` is the call; `:2187/2236` comparisons).
- **M4:** refusal block `:1232-1238` (plan says `:1232-1236`); reassemble call `:1229` (plan `:1226` = comment header).
- **M5:** `--md1-form=template` in descriptor-mode (`--descriptor 'wpkh(...)'`) is a SECOND emission path (`synthesize_descriptor:1616`) — plan should allow (gate applies) or refuse outside `--template`. No `--bsms-form` flag exists (moot); `--threshold` already guarded (`:172/263`).
- **M6:** adding `TemplateFormUnsupportedShape` needs arms at the exit-code (`error.rs:550`) + name (`:615`) match blocks, not just the declaration — add a line to P1.1 so it compiles RED-first.

## Funds-safety completeness
§5 covers all four required properties (keyless-multisig refused both ends; self-check passes; completion address-equivalence vs INDEPENDENT golden; D7 same-preimage). No silent-wrong-wallet path untested ONCE I2's `--from`-required refusal is added (the no-seed mis-route) — that's why I2 is Important.

## To turn GREEN
1. **I1** → thread `Md1Form` through `synthesize_unified:776 → synthesize_descriptor:258` + descriptor-mode callers; 4 mutations on a clone; stub/csi/assert read the clone.
2. **I2** → P1.6 add tree→`CliTemplate`, `--from`-required refusal for the template arm, restore-side typed fully-keyed Descriptor for the D7 recompute.
Fold the 6 Minors (M1 repo label most consequential).
