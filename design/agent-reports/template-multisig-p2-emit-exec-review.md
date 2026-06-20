# P2 — EMIT (Slice 1) execution review — `bundle --md1-form=template` multisig/general (#28 phase 2)

**Verdict: GREEN — 0 Critical, 0 Important.**

Reviewer: opus architect (mandatory per-phase R0 execution review over the implemented diff). Clean re-run after a prior dispatch died on a transient server rate-limit.
Scope: `git diff 29bbfb53..HEAD` (P2 `8967294d` off P1 `29bbfb53`), branch `feature/bundle-md1-template-multisig`. Files: `src/synthesize.rs` (+608), `src/cmd/bundle.rs` (+100), `tests/cli_bundle_md1_template_multisig.rs` (new, +438). Exactly 3 files; **mlock.rs untouched** (0 diff lines). md-codec **0.37.0** (registry, Cargo.lock checksum `fec7cad…`).

---

## Verified correct

**Test + clippy gate (empirical):**
- `tests/cli_bundle_md1_template_multisig.rs`: **13 passed / 0 failed**.
- `synthesize.rs` P2 unit pins run in the **--bins** test target (this crate compiles `synthesize::tests` into `main.rs`, NOT `--lib` — `--lib` shows 0 `synthesize::` tests; a structural quirk, NOT a missing-test defect). All 5 named pins green: `c1_general_policy_template_carries_origins_empty_fails_decode`, `canonical_multisig_template_elides_origins`, `template_admissible_gate`, `order_independent_shape_classifier`, `wallet_policy_id_for_template_is_order_sensitive`. `--bins` total: **1005 passed / 0 failed / 1 ignored**.
- Full `cargo test -p mnemonic-toolkit`: every target `ok`, **0 failed** across the whole suite (no `error[`, no `FAILED`). Phase-1 refusal pins (`cli_bundle_md1_template_form.rs`) green.
- `cargo clippy -p mnemonic-toolkit --tests --bins --lib -- -D warnings`: **exit 0, clean** (the scoped invocation correctly excludes the untracked `examples/idsearch_bench.rs` scratch — out of scope, as flagged).

**Focus 1 — arity-split admission `template_admissible()` (synthesize.rs:~1137):**
- (a) n≥2 → render-based `to_miniscript_descriptor(d, 0).is_ok() && !has_hardened_use_site`. Verified vs md-codec 0.37.0 `to_miniscript.rs`: `tr(sortedmulti_a)` hits the `(Tag::SortedMultiA, MultiKeys)` arm (`:584-589`) → `Err` → REFUSED; `sortedmulti`-in-combinator hits `(Tag::SortedMulti, MultiKeys)` (`:578-583`) → `Err` → REFUSED. `tr(NUMS, multi_a)` + non-taproot multi/sortedmulti/thresh/timelocks render → ADMITTED. `has_hardened_use_site` (`:89`) scans `use_site_path` + `use_site_path_overrides` for hardened wildcard/multipath → hardened use-site REFUSED. Integration pins confirm (`template_form_refuses_tr_sortedmulti_a`, `template_form_refuses_hardened_use_site` → exit 2; `template_form_admits_tr_nums_multi_a` → decodes keyless).
- (b) Gate runs on the **KEYED input** — `template_admissible(descriptor)` is called at the top of `synthesize_template_descriptor` BEFORE `let mut template = descriptor.clone()` and the keyless mutations. `to_miniscript_descriptor` needs `tlv.pubkeys`; calling it on the post-mutation keyless template would fail. Correct ordering.
- (c) n==1 keeps the phase-1 strict `cli_template_from_tree(&tree).is_some()` gate. **Right call.** A render-based gate WOULD wrongly admit bip49 `sh(wpkh)` (renders) and the nested-multi/sortedmulti 1-of-1 (renders) — reversing the deliberate phase-1 R0-I1 refusals. The conservative split preserves all 3 phase-1 refusal tests (`template_form_refuses_bip49_nested_segwit`, `…_nested_sortedmulti_1of1_descriptor_mode`, `…_nested_multi_1of1_descriptor_mode`), which pass in the full sweep. No gap.

**Focus 2 — C1-conditional origin (synthesize.rs:~1161):**
- `canonical_origin(&tree).is_some()` → `path_decl.paths = Shared(OriginPath{empty})`; `is_none()` → KEEP cloned source `path_decl` (Divergent/Shared as built). Verified vs md-codec 0.37.0 `validate.rs:221` `validate_explicit_origin_required`: no-op when `canonical_origin().is_some()`; otherwise requires a non-empty origin per `@N` (override or path_decl) → `Error::MissingExplicitOrigin{idx}`. The conditional exactly matches: canonical elides safely; general policy carries real origins so decode accepts.
- C1 negative pin `c1_general_policy_template_carries_origins_empty_fails_decode` is **non-vacuous**: it (i) asserts the fixture is non-canonical, (ii) emits + decodes the real template (carried origins pass `validate_explicit_origin_required`), then (iii) clones the decoded desc, forces `Shared(empty)`, and asserts `validate_explicit_origin_required` returns `Err(MissingExplicitOrigin)`. The positive and negative both fire against the SAME tree. The integration sibling `general_policy_template_md1_decodes_with_carried_origins` (degrade2 `wsh(or_i(...))`) re-pins through the CLI. Both green.
- Template-id origin-invariance: binding uses `compute_wallet_descriptor_template_id` (`bundle.rs:1221`) which the SPEC/identity pins show hashes tree + use-site + use-site-overrides only (not origin), so carry-vs-elide does not perturb the stub. Unchanged either way.

**Focus 3 — N-slot card back-half + binding (synthesize.rs:~1265, bundle.rs):**
- Single-slot back-half generalized: `n == 1` → `MkField::Single` (csi `derive_mk1_chunk_set_id_for_slot(&stub, 0)`, byte-identical to phase-1); `n ≥ 2` → `MkField::Multi`, one card per cosigner, all sharing the SAME `stub` (`vec![stub; n]`), csi `derive_mk1_chunk_set_id_for_slot(&stub, i)`. ms1 generalized to one entry per slot (watch-only → `""`). `cosigners.len() == n` enforced upstream in `synthesize_descriptor` (`:425`, `cosigners.len() != n → DescriptorParse`) before the template dispatch.
- csi slot-uniqueness: `derive_mk1_chunk_set_id_for_slot(stub, slot) = derive_mk1_chunk_set_id(stub) ^ slot` (`synthesize.rs:90`). slot < n ≤ 16 touches only the low 4 bits → preserves the leading-16-bit bundle-binding prefix while distinguishing same-xpub slots (audit I10). slot=0 reduces to the single-sig form → no regression.
- Binding stub is **form-generic**: `bundle_binding_stub` (`bundle.rs:1217`) → `WalletPolicyId` iff `is_wallet_policy()`, else `compute_wallet_descriptor_template_id`. Keyless template (`!is_wallet_policy()`) → template-id for ALL N cosigners; mk1 + md1 share it. No single-slot assumption remains. `multisig_template_bundle_self_check_passes` confirms the card↔template-id binding holds across all N mk1 cards.

**Focus 4 — D7 print + warning (bundle.rs:~1090/1148):**
- The order-sensitive `WalletPolicyId` print: single-sig path uses `wallet_policy_id_for_singlesig`; multisig/general uses `wallet_policy_id_for_template(&template_desc, resolved)` which re-injects resolved keys/fingerprints into the keyless template tree and rebuilds `path_decl` from the slot origins (`Shared` all-equal / `Divergent`). The funds-safety anchor `multisig_template_prints_wallet_policy_id` pins the printed id `== md_codec::compute_wallet_policy_id(policy_desc)` where `policy_desc` is decoded from a SEPARATE `bundle --md1-form=policy` subprocess — a **real INDEPENDENT cross-path anchor**, NOT self-referential (a wrong key-injection/origin/use-site rebuild would diverge from the independently-emitted policy id and fail). Stdout-purity asserted (`!stdout.contains("wallet-id")`). Green.
- Order-sensitivity verified vs md-codec `identity.rs:172` `compute_wallet_policy_id` + `canonicalize.rs:168` `canonicalize_placeholder_indices`: canonicalize only RE-NUMBERS placeholder indices to document order (permuting tree/path_decl/TLV in lockstep) — it does NOT sort the `@N`→key binding. Per-`@N` records concatenate in slot order. Swapping two distinct slot keys ⇒ different hash. `wallet_policy_id_for_template_is_order_sensitive` (`assert_ne!`) is non-vacuous.
- Warning fires order-dependent: `emit_template_order_warning` gated on `template_desc.n >= 2`; `is_order_independent_shape` true for `sortedmulti`/`sortedmulti_a` (walks wsh/sh/tr wrappers) → softened note (no "only one assignment"). Order-dependent → loud `N!` count + "only one assignment", plus the asymmetric-spending-role caveat when `canonical_origin().is_none()` (general policy). `n` = distinct `@N` slot count (md-codec `canonicalize.rs:969` "n is the count of distinct placeholders") = SPEC §3.4's N. Pins green (`order_dependent_multisig_template_emits_loud_warning`, `sortedmulti_template_softens_order_warning`, `general_policy_template_warns_about_spending_role`).

**Focus 5 — non-regression:** `single_sig_template_still_emits_after_multisig_admission` (n==1 still decodes keyless); the phase-1 `cli_bundle_md1_template_form.rs` byte-identity + `--md1-form=policy` pins remain green in the full sweep (the n==1 arm is verbatim phase-1).

**Focus 6 — regression sweep + safety:**
- Full `cargo test -p mnemonic-toolkit` GREEN; clippy `-D warnings` clean (exit 0). No `mlock::g4_a` flake observed. mlock.rs untouched.
- No `.unwrap()`/`.expect()`/`panic!` in new **production** code: `bundle.rs` new code has zero; `synthesize.rs` new production code's only index sites are guarded — `origin_paths[0]` reached only after the `slots.len() != n` early-return with n≥1 (and under `all_same || n == 1`, so `origin_paths` is non-empty); `cosigners[0]` inside the `n == 1` branch; `children[0]` under `children.len() == 1`. All remaining unwrap/panic matches are inside `mod tests`.
- N! overflow guard: `(1..=n).try_fold(1u64, |acc,k| acc.checked_mul(k))` → `None` arm prints "astronomically many"; no panic on a pathological placeholder count.
- Wrong-descriptor mutation: mutations apply to `template = descriptor.clone()`; the admission gate + `wallet_policy_id_for_template` read the original; `compute_wallet_policy_id` clones internally before canonicalizing. No mutation of the caller's descriptor.

**Fixture tracking:** `tests/.../degrade2.desc` via `include_str!("../../../.examples-build/degrade2.desc")` is **tracked** (committed `bc3681fb`; `.examples-build/.gitignore` explicitly tracks the `*.desc` inputs; `git check-ignore` empty). CI will compile the test.

---

## CRITICAL
None.

## IMPORTANT
None.

## MINOR
- **(observational, no action)** `synthesize::tests` (incl. the 5 P2 unit pins) compile into the **--bins** target, not `--lib` — running `cargo test -p mnemonic-toolkit --lib <name>` reports "0 tests / filtered out" and could mislead a future reviewer into thinking a pin is missing. They DO run under `--bins` and the default `cargo test -p mnemonic-toolkit`. Pre-existing crate structure; not introduced by P2. No change required.

## To turn GREEN
Already GREEN. Proceed to P3 (RESTORE completion) under the per-phase R0 gate. P3 is the load-bearing funds-safety R0 (the per-slot origin BUILD / I-A, carried-origin-never-loaded C1 invariant, distinct-keys + strong-prefix floors, swapped-`@N` reject) per the plan §8.
