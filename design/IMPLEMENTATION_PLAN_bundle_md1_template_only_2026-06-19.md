# IMPLEMENTATION PLAN — bundle-md1-template-only-option (Phase 1: single-sig shareable template)

**Date:** 2026-06-19 · **UNCOMMITTED working draft** (SeedHammer freeze; NO CODE until the freeze lifts + an explicit go). 
**SPEC (R0-GREEN, 2 rounds):** `design/SPEC_bundle_md1_template_only_2026-06-19.md` + `design/agent-reports/template-only-spec-r0-round{1,2}-review.md`.
**Source SHAs (grep-verified):** mnemonic-toolkit `4e21d94`, descriptor-mnemonic `c85cd49`, mnemonic-key `913febc`. Re-grep all citations at execution time (CLAUDE.md citation-decay; local is 1-ahead with the parallel SeedHammer FOLLOWUP — re-baseline).
**Versions:** toolkit `0.58.1 → 0.59.0` MINOR, mk-cli `0.9.x` MINOR, **md-codec / mk-codec NO-BUMP** (the template id + keyless wire form already exist in the pinned `md-codec 0.36`), mnemonic-gui MINOR paired.

## 0. Gate + discipline
Each phase is per-phase TDD (RED tests before impl) + a per-phase opus R0 to **0C/0I before advancing** (CLAUDE.md). Funds-safety class (a wrong completion = wrong wallet) → the address-equivalence + D7-same-preimage oracles are the make-or-break gate, not exit-0. **No code is written until the SeedHammer first-pass freeze lifts and the user gives an explicit go.**

## 1. Cross-repo shape — NO publish dependency
Unlike the #25 cycle, this has **no upstream-publish gate:** the toolkit consumes `md_codec::compute_wallet_descriptor_template_id` (already public in the pinned `md-codec 0.36`, `identity.rs:71-104`) and `mk_codec::encode_with_chunk_set_id`/`KeyCard` (existing); it computes its own stubs (`synthesize.rs`). So:
- **Phase 1 (toolkit)** is self-contained — no md-codec/mk-codec change, develop + ship against the current pins.
- **Phase 2 (mk-cli, in the `mnemonic-key` repo)** is a PARALLEL consistency fix (so `mk verify`/`encode` agree on template stubs) — not on the toolkit's critical path.
- **Phase 3 (mnemonic-gui)** is PAIRED with the toolkit flag (schema_mirror), built against the toolkit binary.

---

## 2. PHASE 1 — toolkit (`0.59.0`)

### P1.1 — flag + canonical gate + error (entry point)
- **Impl:** `bundle` gains `--md1-form <policy|template>` (clap value-enum `Md1Form`, default `policy`; `cmd/bundle.rs`). Canonical gate (C1): template form REQUIRES `descriptor.n == 1 && canonical_origin(&d.tree).is_some()` (`canonical_origin.rs`); refuse on `n>1` OR `is_none()` → new `ToolkitError::TemplateFormUnsupportedShape` (declare `error.rs:306`-area alphabetical between `SlotInputViolation` and `UnknownHrp`; **M6: add arms at ALL THREE exhaustive (no catch-all) match blocks — exit-code (`error.rs:550`), name/kind (`:615`), AND `message()` (`:628`, the round-2-found third block)**, else it won't compile. `details():847` has a `_ => None` catch-all so is exempt; `Display::fmt:875` delegates to `message()`). The gate applies to BOTH emission paths — `--template <bip44/49/84/86>` AND descriptor-mode `--descriptor 'wpkh(...)'` (`synthesize_descriptor:1616`); M5: `--md1-form=template` on a non-canonical/multisig descriptor refuses identically. (Edge: a `wsh(multi(1,@0))` 1-of-1 slips `n==1` — add a `multi`-tag guard OR a test pin; non-blocking.)
- **TDD (RED):** `--md1-form=policy` == today (default unchanged); `--md1-form=template` on multisig / `n>1` / non-canonical / custom-path → `TemplateFormUnsupportedShape`; `--md1-form=template` on a canonical single-sig (bip44/49/84/86) → proceeds.

### P1.2 — template emission (the 4 mutations) — with form-threading (I1)
- **Threading (I1 — required first):** the standard path `bundle --template bip84 --md1-form=template` enters `synthesize_unified` (`bundle.rs:421` → `synthesize.rs:776`), which BUILDS the keyed descriptor (`pubkeys=Some`, `fingerprints=Some`, populated `path_decl`, `synthesize.rs:835-850`) then delegates to `synthesize_descriptor:859`. `synthesize_descriptor` takes `descriptor: &Descriptor` by SHARED ref (`:259`) → cannot mutate in place, and neither fn takes the form. So: thread `Md1Form` (or `bool template_form`) through `synthesize_unified:776 → synthesize_descriptor:258`. **Signature-change blast radius (round-2 Minor):** changing those two signatures forces every caller to update — beyond the descriptor-mode trio (`bundle.rs:1616/1726/1969`), the OTHER production callers are `bundle.rs:421` (the `--template` path into `synthesize_unified`), `verify_bundle.rs:427/525/630` (`synthesize_unified`) + `:1104` (`synthesize_descriptor`), `import_wallet.rs:1455` (`synthesize_descriptor`), plus ~8 test callers in `synthesize.rs`/`parse_descriptor.rs`. All pass `Md1Form::Policy` (today's behavior) — mechanical + compiler-caught; **re-grep all callers at execution and pass the default** so the implementer doesn't stall.
- **Impl (the 4 mutations):** when `template_form`, build a `descriptor.clone()` (the production chokepoint is `synthesize_descriptor:258`, OR `synthesize_unified:835-850` pre-delegation) with: `tlv.pubkeys=None`, `tlv.fingerprints=None`, **`path_decl` origin elided to empty** — exact field write `descriptor.path_decl.paths = PathDeclPaths::Shared(OriginPath{components:vec![]})` (for `n==1` the path is ALWAYS `Shared`, `synthesize.rs:816`) — and drop the `debug_assert!(is_wallet_policy())` (`:346`) on the template path. Confirm the stub/csi (P1.3, `:272/:290`) + the dropped assert read the MUTATED clone, not the original. Encoder writes `path_decl` verbatim (`encode.rs:85`) → empty origin reaches the wire.
- **TDD (RED — the GOAL):** two DIFFERENT seeds, same type + account-shape → **byte-identical** template md1; account-0 vs account-5 of the same type → **byte-identical** (account normalized away); a keyless canonical template `md decode`s without refusal (`decode.rs:68` canonical short-circuit).

### P1.3 — binding-stub re-root (template id)
- **Impl:** for template form switch the PRODUCTION stub sites from `compute_wallet_policy_id` → `compute_wallet_descriptor_template_id`: the string-level stub (`synthesize.rs:272` → mk1 `KeyCard` + csi `:290`) and the display labels (`bundle.rs:1082`, `:1103-1106`). Compute the id ONCE (fold the current double-recompute). ms1 string is UNCHANGED (LIVE `ms_codec::encode` at `synthesize.rs:339`; `:172` is the dead `synthesize_full` helper — no id field either way).
- **TDD:** a template bundle's mk1/md1 share the template-id stub; a template md1 does NOT bind to a policy-form mk1 (different stub) and vice-versa.

### P1.4 — self-check branch (I1)
- **Impl:** `self_check_bundle` (`bundle.rs:2139`) branches for template form: skip the `:2151` `is_wallet_policy → BundleMismatch` gate; skip the `:2171` pubkeys-absent refusal + `check_mk1_xpub_binding` (`:2186/2220`) — inapplicable to a keyless template; compute `expected_stub` (`:2158/2187/2236`) from `compute_wallet_descriptor_template_id`. Surviving checks (stub-coherence + mk1 origin/fp `:2193-2205` + ms1 parity `:2253-2294`) still apply.
- **TDD:** `self_check_bundle` PASSES for a single-sig template bundle (must not trip the keyless gates).

### P1.5 — D7 `WalletPolicyId` stderr advisory (I4)
- **Impl:** `bundle --md1-form=template` prints on STDERR (a NEW `writeln!` modeled on the `secret_advisory::emit_output_class_advisory` idiom, `bundle.rs:1035` — not a literal reuse) the `WalletPolicyId` computed from the **FULLY-KEYED, EXPLICIT-origin (`m/84'/0'/account'`), presence-`0b11`** descriptor (NOT the elided template — I4: `compute_wallet_policy_id` is origin-significant, `identity.rs:161-185`). Render: full hex + 12-word `to_phrase()` (`identity.rs:129`) + a convenience 4-byte prefix. Advisory-only; stdout (the engraved cards) unchanged.
- **TDD:** the printed id is recomputable; goes to stderr not stdout; render forms match.

### P1.6 — restore single-sig template-completion (C2)
- **Impl:** routing carve-out at `restore.rs:177-179`: reassemble `args.md1` (hoist the `chunk::reassemble` call from `run_multisig:1229`) and branch — keyless single-sig template (`!d.is_wallet_policy() && d.n==1 && canonical_origin().is_some()`) → NEW single-sig completion; else `run_multisig` (its `:1232-1238` refusal then catches keyless multisig). Completion sub-steps (I2):
  - **(a) tree→type:** map `d.tree → CliTemplate` (a tag-match mirroring `canonical_origin`'s shape dispatch — NOT literally the inverse of `script_type_from_template`, which is `CliTemplate → Option<ScriptType>` at `convert.rs:402`) and derive ONLY that one type — do NOT iterate all four templates (`:328-331/339`).
  - **(b) `--from` REQUIRED (funds-safety):** today `--from` is `required_unless_present="md1"`, so `restore --md1 <template>` with NO `--from` is clap-valid and would mis-route to watch-only — the template-completion arm MUST explicitly REJECT a missing `--from` (the seed is mandatory; a no-seed template restore is a silent-wrong-route hole).
  - **(c) key + origin:** seed-derived key + `--account` (exists, `:99-101`/derive `:340-346`) + NEW `--origin <path>` (wrapper over `derive_bip32_from_entropy_at_path` `derive_slot.rs:65`) → concrete watch-only descriptor.
- **`--expect-wallet-id <prefix>`** (NEW, optional): build a **typed, fully-keyed, explicit-origin (`m/84'/0'/account'`), presence-`0b11` `md_codec::Descriptor`** (the `build_descriptor:131` shape — NOT the `build_descriptor_string:387` text path), recompute `WalletPolicyId`, match leading bytes, **refuse loudly on mismatch** (`ModeViolation`/exit 4); advisory min ≥4 bytes (warn, don't enforce). Same typed-Descriptor builder feeds the D7 round-trip oracle (§5.4).
- **TDD:** `bundle --md1-form=template` → `restore --md1 <t> --from <seed> --account N` → derived addresses == the original full-policy wallet (independent golden / Core); keyless **multisig** template at restore → refusal (the carve-out fall-through, `:1232`); `--expect-wallet-id` correct→pass, wrong→refuse, short→advisory.

### P1.7 — verify-bundle completion + recompose
- **Impl:** replace the vacuous template-only skip (`verify_bundle.rs:2152-2167`) with: verify the cards bind via the template-id stub, then complete+recompose the single-sig watch-only wallet (seed + `--account`/`--origin`); support `--expect-wallet-id`.
- **TDD:** verify-bundle on a template bundle recomposes + asserts consistency; `--expect-wallet-id` mismatch → refuse.

### P1.8 — GUI + manual locksteps
- GUI: add `--md1-form` flag-NAME to `mnemonic-gui/src/schema/mnemonic.rs` `BUNDLE_FLAGS` (~:190/:3778) + `policy|template` dropdown VALUES (paired-PR) + pin bump. Run `cargo test --test schema_mirror` with the new toolkit binary (flag-name PASS; values are discipline). **Do NOT `cargo fmt` the GUI.**
- Manual: add the `--md1-form` row + a `### Template-only md1` section to `docs/manual/src/40-cli-reference/41-mnemonic.md`; run `make -C docs/manual audit`.

### P1.9 — version + ship
- toolkit `0.59.0`: Cargo.toml, BOTH READMEs (`README.md:13` + `crates/mnemonic-toolkit/README.md:9`), `scripts/install.sh`, `fuzz/Cargo.lock`, `Cargo.lock`, CHANGELOG. fmt: `cargo +1.95.0 fmt -p mnemonic-toolkit` then `git checkout -- …/mlock.rs` (g6). **Per-phase R0 on the toolkit changes to 0C/0I.** Ship.

---

## 3. PHASE 2 — mk-cli (**mnemonic-key**, NOT descriptor-mnemonic — M1), parallel
- **P2.1:** `derive_stub_from_md1` (`mk-cli mod.rs:63-69`) gains a `!descriptor.is_wallet_policy()` branch → `compute_wallet_descriptor_template_id` (M4: `compute_wallet_policy_id` does NOT self-refuse on a keyless template → would silently mis-stub without the branch). Fix the stale docs at `mk-cli mod.rs:55-62` AND `mk-codec key_card.rs:25-30`.
- **TDD:** `mk encode`/`mk verify` on a keyless template md1 → the template-id stub (agrees with the toolkit).
- **P2.2:** mk-cli MINOR + CHANGELOG; publish only if a standalone consumer needs it (the toolkit does NOT depend on it). Per-phase R0.

## 4. PHASE 3 — mnemonic-gui (paired)
Folded into P1.8 (schema flag/value + pin). MINOR; GUI has no fmt CI gate (don't `cargo fmt` it).

## 5. Consolidated funds-safety test inventory (all RED-first)
1. Byte-identity: two seeds / two accounts, same type → identical template md1 (P1.2).
2. Binding: mk1/md1 share template-id stub; template↔policy cross-reject (P1.3).
3. Self-check PASSES for a template bundle (P1.4).
4. D7 round-trip: `bundle`-printed `WalletPolicyId` == `restore`-recomputed (same explicit-origin presence-`0b11` preimage) (P1.5/P1.6).
5. Completion address-equivalence: template+seed+account → same addresses as the full-policy wallet, INDEPENDENT golden (P1.6).
6. Refusals BOTH ends: multisig/`n>1`/non-canonical at bundle-emit; keyless multisig at restore-ingest fall-through (P1.1/P1.6).
7. `--expect-wallet-id`: correct→pass, wrong→loud refuse, short→advisory (P1.6).
8. Non-regression: `--md1-form=policy` byte-identical to today for all corpus bundles (P1.1).

## 6. SemVer / version sites / locksteps
- toolkit MINOR (§P1.9 sites). mk-cli MINOR (P2). **md-codec / mk-codec NO-BUMP.** GUI MINOR paired.
- Locksteps: GUI schema flag-NAME + dropdown-VALUE paired-PR + manual flag-row (`docs/manual/tests/lint.sh`, `make audit`). New `ToolkitError` variant(s) alphabetical.
- Ordering: toolkit (self-contained) is the critical path; mk-cli + GUI parallel/follow. No publish gate.
- SH coordination: hand the SH instance the D1 binding decision (re-root on `WalletDescriptorTemplateId`) at implementation start.

## 7. Risks / per-phase R0 focus
- **Account normalization** (the goal): confirm elide-to-empty truly drops the account for ALL canonical types × network, and restore's `--account`/`--origin` rebuild the explicit origin D7 hashes (the §5.4/5.5 oracles).
- **Self-check + restore branches** must not leave a residual keyless gate (I1/C2) — enumerate every `is_wallet_policy`/`pubkeys`-absent gate at execution (re-grep).
- **D7 same-preimage** (I4): the bundle and restore sides must hash the identical fully-keyed explicit-origin descriptor.
- Per-phase R0 gates each phase before code.

## 8. Citation-decay
Line numbers are snapshots at the header SHAs; re-grep at execution. The `self_check_bundle` (not `verify_self_consistency`) name + the production-vs-dead synth sites (`:258/272/290` live; `:131/164/207/180/216` dead) are the round-2-corrected anchors.
