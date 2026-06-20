# cycle-prep recon — 2026-06-20 — bundle-md1-template-only-option (PHASE 2: multisig template)

**Toolkit (origin/master) SHA at recon time:** `cbdadbb7` (branch `master`, tag `mnemonic-toolkit-v0.59.1-1-gcbdadbb7`; Cargo `0.59.1`)
**Sync state:** up-to-date (0 ahead / 0 behind origin/master).
**Untracked:** none at recon start.
**Multi-repo recon** (constellation). Primary-source SHAs:

| Repo | Path | Branch | HEAD | Verdict |
|---|---|---|---|---|
| mnemonic-toolkit | `/scratch/code/shibboleth/mnemonic-toolkit` | `master` | `cbdadbb7` (v0.59.1) | **all the work is here** |
| descriptor-mnemonic (md-codec) | `/scratch/code/shibboleth/descriptor-mnemonic` | `main` | `54dd765` (v0.37.0) | **NO change needed** |
| mnemonic-key (mk-cli / mk-codec) | `/scratch/code/shibboleth/mnemonic-key` | `main` | `3258271` (mk-cli v0.10.0) | **NO change needed** |

Slug verified: `bundle-md1-template-only-option` (phase-2 / multisig portion). **Verdict: feasible and well-scoped; this is a PURE TOOLKIT cycle.** The codec layer (encode + key-independent `WalletDescriptorTemplateId` + decode) and the mk-cli form-aware stub already handle keyless multisig. Emit is a small guard-lift; the cycle's substance is the **recompose/verify completion path** and its **C1 funds-safety boundary**.

---

## Per-slug verification

### `bundle-md1-template-only-option` — phase 2 (multisig template)

**WHAT:** Extend the existing `bundle --md1-form=template` from single-sig to **multisig** wallet policies — emit a keyless multisig template md1 (`pubkeys:null`, N cosigner slots, threshold k, sortedmulti structure), bind cards via the template-stable id, and teach `restore`/`verify-bundle` to **recompose** a concrete watch-only multisig wallet from the template + externally-supplied cosigner keys + the own seed. Motivation (parent slug): cut engraving plates (template ≈ 1 chunk/plate vs full policy ≈ 2-3).

**Citations / claims (re-checked against PRIMARY SOURCE at the SHAs above):**

- **md-codec supports keyless multisig templates entirely** — **CONFIRMED, no codec change.** `md encode 'wsh(sortedmulti(2,@0/**,@1/**,@2/**))'` (no `--key`) emits a template (`pubkeys:None`) — `template.rs:1756-1762,1780`; tree encodes `Body::MultiKeys{k,indices}` key-independently (`tree.rs:115-139`). `compute_wallet_descriptor_template_id` (`identity.rs:71-104`) hashes only use-site + tree bits, recurses multisig nodes → stable, distinct ids (live: 2-of-3=`b02b4403`, 2-of-2=`aad0e0e0`, 3-of-3=`a227f95e`, 2-of-3-`multi`=`9229657a`). Round-trip decode confirmed. `to_miniscript_descriptor` correctly errors `MissingPubkey` (`to_miniscript.rs:122`) on a keyless template — the boundary the toolkit recompose must cross. **No md-codec gap.**

- **The toolkit EMIT refusal is two `n!=1` guards** — **CONFIRMED (`synthesize.rs`).** `synthesize_template_descriptor` (`:981`): guard A `descriptor.n != 1` → `TemplateFormUnsupportedShape` (`:987-994`); guard B `cli_template_from_tree(&tree).is_none()` (refuses every multi/sortedmulti at any depth incl. degenerate 1-of-1, commit `d8b6ecaa`, `:1005-1012`). The keyless mutation block (`:1023-1032`) nulls `pubkeys`+`fingerprints` (already N-agnostic) and elides origin (`PathDeclPaths::Shared(empty)` — the one mutation hard-coded to the single-slot case). **Delta to emit multisig: lift both guards + generalize the origin-elide to N slots (handle the `Divergent` path_decl arm at `synthesize_unified:907`, or gate to canonical-shared-origin multisig).** Threshold k + sortedmulti + N slots are preserved for free (unmutated `descriptor.tree`).

- **Card binding stub is already form-generic** — **CONFIRMED.** `bundle_binding_stub` (`cmd/bundle.rs:1151-1159`) discriminates on `is_wallet_policy()` (false for ANY keyless template → `WalletDescriptorTemplateId`); per-slot csi `derive_mk1_chunk_set_id_for_slot(stub, slot)` is already a slot loop (`bundle.rs:1212-1217`). **No multisig-specific stub change on emit.**

- **mk-cli form-aware stub handles multisig templates** — **CONFIRMED, no mk change.** `mk-cli .../cmd/mod.rs:72-82 derive_stub_from_md1` → `is_wallet_policy() ? WalletPolicyId : WalletDescriptorTemplateId`; the else-arm is N-agnostic. **No mk-cli/mk-codec change needed.**

- **A keyless multisig template is NOT reconstructible from the md1 alone** — **CONFIRMED (the recompose blocker).** `restore.rs:1655-1661` gates `if !d.is_wallet_policy()` → `ModeViolation` ("multisig restore needs a wallet-policy md1"); even past it, `expand_per_at_n` + `e.xpub.ok_or(...)` (`restore.rs:1752-1761`) fails for a template. RED test already pins it: `tests/cli_restore_md1_template.rs:206 keyless_multisig_md1_refused_at_restore`. **So N cosigner keys MUST be supplied externally.**

- **The recompose model is decided in the design record** — **CONFIRMED.** Keys come from explicit, REQUIRED `--cosigner @N=<mk1|xpub>` (brainstorm D3, `BRAINSTORM_bundle_md1_template_only_2026-06-19.md:39`); mk1 carries no `@N` slot field (`mk-codec key_card.rs:24-54`) so operator-asserted `@N=` is the only safe source. `restore` ALREADY parses `--cosigner @N=<mk1|xpub>` (repeatable, `restore.rs:78,1952-1986`, mk1 via `mk_codec::decode`) and `--from` (`required_unless_present="md1"`) — but TODAY these are **cross-check** inputs (compared against `c.key65` sourced FROM the md1's pubkeys, `restore.rs:1987`). **The template path must INVERT this: BUILD the `ResolvedSlot`s from the supplied `@N` keys + own-seed slot, instead of reading them off the md1.**

- **verify-bundle has a single-sig template path but no multisig one, and lacks the intake flags** — **CONFIRMED.** `verify_singlesig_template` (`verify_bundle.rs:478-608`) is the mirror to extend; the multisig path (`:883-976`) sources keys from `md1.tlv.pubkeys` and fails on a template (`:2474-2489` "descriptor is template-only"). **verify-bundle has NEITHER `--from` NOR `--cosigner`** (only `--slot`/`--mk1`/`--md1`/`--origin`/`--expect-wallet-id`) → a multisig-template verify needs new external cosigner-key intake + a `verify_multisig_template` + an early short-circuit.

- **Dependency #25 (override-bearing per-`@N` reconstruction)** — **CONFIRMED SHIPPED** (toolkit 0.58.2/0.59.1). The phase-2-on-#25 gate (brainstorm I3) is now CLEAR. Phase-2 release-ritual: confirm/bump the md-codec pin to **0.37.0** (`54dd765`).

- **Taproot `sortedmulti_a` leg is still OPEN** — **CONFIRMED carve-out.** `restore-md1-taproot-use-site-override-arm` shipped `tr(NUMS,multi_a)` (v0.59.1) but `tr(sortedmulti_a)` remains gated on rust-miniscript >13.1.0 (`Terminal::SortedMultiA`); `md-codec-sortedmulti-a-to-miniscript-rendering-gap` open. **Scope the multisig template cycle to NON-taproot (wsh + sh(wsh)),** inheriting the full-policy path's bounds.

**Action for brainstorm spec:** scope = **emit (guard-lift) + multisig template-completion in `restore` + `verify-bundle`**, wsh/sh(wsh) only, with the **C1 funds-safety boundary** as the central design decision (below). Cite source SHAs: toolkit `cbdadbb7`, md-codec `54dd765`, mk-cli `3258271`. Re-grep all `synthesize.rs`/`restore.rs`/`verify_bundle.rs` line numbers at write time (they decay).

---

## The C1 funds-safety boundary — the central brainstorm decision

This is **why** multisig template was deferred (SPEC §2 OUT / §4.2; brainstorm D2). Single-sig template completion is safe: the seed re-derives `@0` and cross-checks the template's own slot. **Multisig completion grafts keys the operator supplies externally** — a mistyped, misordered, or attacker-substituted `--cosigner @N=` **silently builds the wrong watch-only wallet** (the keyless md1 has no pubkeys to validate against). Layers the design record prescribes (SPEC §7 + D4/D7):

1. **D7 `--expect-wallet-id` (load-bearing):** after assembling, recompute the **full `WalletPolicyId`** of the completed descriptor and require it to equal an operator-supplied expected id. This is the primary boundary — without the keys-in-md1 cross-check, the assembled wallet must be pinned to a known-good id.
2. **D4 origin cross-check (partial):** mk1 `origin_path` + `policy_id_stubs` vs the template's `@N` origin/template-id — only discriminates origin-DISTINCT cosigners; insufficient alone.
3. **Discriminating template-bundle stub (design option):** `H(WalletDescriptorTemplateId ‖ sorted cosigner-fingerprints)` as a card-binding that captures the cosigner set without the keys — an alternative/complement to `--expect-wallet-id`. The brainstorm must choose: `--expect-wallet-id` boundary vs (or +) the discriminating stub.
4. **Address-equivalence differential (test-side):** verify the completed descriptor's first addresses against an independent golden (the full-policy bundle of the same wallet) so a wrong assembly cannot pass tests.

**This boundary is the riskiest piece and warrants the heaviest R0 scrutiny** (a wrong-wallet hole is funds-loss-adjacent). It is the reason this is "its own R0-heavy cycle," not a flag flip.

---

## Cross-cutting observations

1. **Scope is narrower than the parent FOLLOWUP implied — pure toolkit.** Both sibling codecs (md-codec, mk-cli) need ZERO changes (verified at primary source); the parent slug's "coordinated change across md-codec + mk-codec + toolkit" is, for multisig, **toolkit-only**. No companion FOLLOWUP edits in the sibling repos are required for the codec/stub (only the cross-repo umbrella entry updates).

2. **Emit ≪ completion in effort.** The emit delta is ~a guard-lift + one origin-elide generalization (S). The cycle's mass is `restore` flow-inversion + a new `verify_multisig_template` + the funds-safety boundary (M-L). Don't let the easy emit half mask the hard completion half during sizing.

3. **The restore flow INVERSION is subtle.** Today `--cosigner`/`--from` are cross-checks against md1-sourced keys; the template path must make them the *build* source. This touches the security-load-bearing assembly path — a partial-verification regression (e.g. marking unsupplied slots "verified") is exactly the class the v0.44.0 Phase-2-R1 Critical fold caught. TDD must assert: only operator-supplied+id-pinned assemblies are accepted; a swapped `@N` is rejected.

4. **verify-bundle gains intake flags → lockstep trips.** Adding multisig-template cosigner-key intake to `verify-bundle` (e.g. `--cosigner @N=<mk1>` and/or a seed slot) is a **new clap flag/value** → it DOES trip the **GUI `schema_mirror`** (flag-name parity, `mnemonic-gui/src/schema/mnemonic.rs`) AND the **manual mirror** (`docs/manual/src/40-cli-reference/`). Emit (`--md1-form=template`, existing flag) does NOT. Plan the paired GUI + manual update in-lockstep with the implementing PR (per CLAUDE.md). `restore` may reuse its existing `--cosigner`/`--from` (no new flag) — confirm at spec time.

5. **Version drift (minor):** HEAD is v0.59.1 (`cbdadbb7`), not v0.59.0 — v0.59.1 is the unrelated #26 taproot-override `multi_a` leg. The single-sig template shipped in v0.59.0 (`b0bad50e`/`d8b6ecaa`). Not material.

6. **Stale-FOLLOWUP housekeeping:** the parent `bundle-md1-template-only-option` is OPEN (umbrella); the single-sig phase shipped (v0.59.0) but the entry wasn't flipped (also noted in the sibling SeedHammer recon). On phase-2 completion, update the umbrella + the `restore-multisig-cosigner-scope` §11 I4 carve-out + the SeedHammer-side `constellation-template-only-engraving` (which can then extend to multisig).

---

## Recommended brainstorm-session scope

- **One toolkit cycle:** "multisig wallet-policy template (emit + complete + verify)" — phase 2 of `bundle-md1-template-only-option`. **wsh + sh(wsh) only** (taproot `sortedmulti_a` carved out, inherits the open render-gap refusal).
- **Sizing: M–L.** Three slices:
  - **Slice 1 — EMIT (S):** lift the two `n!=1` guards in `synthesize_template_descriptor`; generalize the origin-elide to N slots (Divergent arm or canonical-shared gate). Goldens: keyless multisig template md1 + its `WalletDescriptorTemplateId`; the form-aware bundle stub for N cosigners.
  - **Slice 2 — RESTORE completion (M):** new multisig-template gate (mirror `restore.rs:207-217`) → an INVERTED completion path that builds `ResolvedSlot`s from REQUIRED `--cosigner @N=<mk1|xpub>` (all N) + `--from <seed>` (own slot) → `build_descriptor_string`; with the **C1 boundary** (`--expect-wallet-id` + D4 origin cross-check). Heaviest R0 focus.
  - **Slice 3 — VERIFY-BUNDLE (M):** `verify_multisig_template` paralleling `verify_singlesig_template`, an early short-circuit, and the new external cosigner-key intake flags (→ GUI schema_mirror + manual lockstep).
- **SemVer: MINOR** (additive: multisig template emit + completion + verify; existing `policy`/single-sig `template` output unchanged).
- **Locksteps:** EMIT none; VERIFY-BUNDLE new flags → **GUI `schema_mirror` + `docs/manual/src/40-cli-reference/` paired updates** (mandatory, same PR). RESTORE likely reuses existing flags (confirm). Cross-repo: **no md-codec/mk-codec change** → no sibling companion edits beyond the umbrella FOLLOWUP.
- **Dependencies:** #25 SHIPPED (clear); re-pin md-codec → 0.37.0 (`54dd765`) as a release-ritual touch.
- **Ordering:** Slice 1 → Slice 2 → Slice 3 (verify depends on emit's template-stable cards + restore's completion semantics).
- **Gate reminder (CLAUDE.md):** the brainstorm SPEC and the IMPLEMENTATION_PLAN each MUST pass an opus architect R0 to 0C/0I before any code (folds persisted verbatim to `design/agent-reports/`), then single-implementer TDD in a worktree, then the mandatory whole-diff adversarial exec review. **Slice 2's funds-safety boundary is the load-bearing R0 target** (wrong-wallet hole = funds-adjacent); require an address-equivalence differential vs a full-policy golden.
- **Downstream:** this UNBLOCKS the SeedHammer fork-side multisig template engrave (the `constellation-template-only-engraving` recon recommended single-sig-only *because* the constellation was single-sig-only; completing this lets the fork cycle cover multisig too).
