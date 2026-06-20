# #28 Phase 1 — `bundle --md1-form=template` IMPLEMENTATION execution review (R0 round 1)

**Verdict: RED — 0 Critical, 2 Important.**

Reviewer: opus architect (mandatory per-phase post-implementation execution review).
Branch: `feature/bundle-md1-template-only` (3 commits `b0bad50`, `1fcad73`, `e59e4f7` off `master a1e26df`).
Diff: `git diff master..feature/bundle-md1-template-only` — 1827 insertions, 7 source files + 3 new test files.
md-codec: published `0.37.0` (no bump). Plan: `IMPLEMENTATION_PLAN_bundle_md1_template_only_2026-06-19.md`; SPEC `SPEC_bundle_md1_template_only_2026-06-19.md`.

## Test / clippy results (run by reviewer)
- `cargo test -p mnemonic-toolkit` → **3130 passed / 0 failed / 13 ignored** (exit 0).
- The 3 new test files alone: `cli_bundle_md1_template_form` 9/9, `cli_restore_md1_template` 6/6, `cli_verify_bundle_md1_template` 4/4 — all green.
- `cargo clippy -p mnemonic-toolkit --tests` → **clean, exit 0**, zero warnings.
- `mlock.rs` UNTOUCHED (`git diff --stat … -- '**/mlock.rs'` empty).

## Verified correct

- **D7 same-preimage round-trip is genuine, NOT vacuous (focus 2).** Empirically: `bundle --md1-form=template --account 3` printed `wallet-id (hex): 53e764eb3331ef26749555b4cf874a95`; `restore … --account 3 --expect-wallet-id 53e764eb…` → `✓ wallet-id verified`; the SAME md1+id with `--account 5` → `✗ WALLET-ID MISMATCH` (exit 4). Both sides build the identical preimage through the single `wallet_policy_id_for_singlesig` → `build_descriptor` (`synthesize.rs:206-215` / `:167-195`) helper — fully-keyed, explicit-origin (`PathDeclPaths::Shared(origin_path)`, `:183`), presence-`0b11` (`fingerprints=Some`, `pubkeys=Some`, `:189-190`). `compute_wallet_policy_id` reads `path_decl.paths` and does NOT consult `canonical_origin` (md-codec `identity.rs:161-185` INVARIANT note) — so the explicit origin is load-bearing and correctly present on both sides. The account-discrimination proves the id is key/account-significant.

- **P1.6(b) the `--from`-REQUIRED refusal closes the no-seed mis-route (focus 1a).** `restore --md1 <template>` with NO `--from` → `error: restore of a keyless single-sig TEMPLATE md1 requires --from <seed> …` exit 2, from `run_singlesig_template_completion` (`restore.rs:679-686` `ModeViolation`), NOT a fall-through to watch-only `run_multisig`. The clap `required_unless_present="md1"` would admit it; the runtime carve-out catches it explicitly. Test `template_restore_without_from_is_refused` pins it.

- **P1.6(a) `d.tree → CliTemplate` maps to exactly ONE type (focus 1b).** `cli_template_from_tree` (`synthesize.rs:228-237`) matches `(Pkh,KeyArg)→bip44`, `(Wpkh,KeyArg)→bip84`, `(Tr,Tr{tree:None})→bip86`; everything else `None`. `run_singlesig_template_completion` derives ONLY that one type (`restore.rs:670`, `script_type_from_template(template)`), not all four. Verified against md-codec `tree.rs` Body/Tag enums.

- **P1.6 completion address-equivalence (focus 1c).** Test `template_completion_equals_independent_full_restore` (bip44/84/86 × account {0,5}) asserts the completed descriptor AND first-receive addresses equal the INDEPENDENT golden `restore --from <seed> --template …` — a path that never touches `--md1` / the new code. The two reach the shared derivation engine (`derive_bip32_from_entropy` + `build_descriptor_string`) via different routing → a wrong-completion would diverge. The golden is not self-referential through the new code. Acceptable per the SPEC's "shared derivation engine = acceptable" standard.

- **P1.6(d) keyless multisig template at restore still refused (focus 1d).** A keyless `wsh(sortedmulti(1,@0))` and a keyless 2-of-3 both have `cli_template_from_tree → None` (top tag `Wsh`, body `Children`) → the carve-out (`restore.rs:206-213`) does NOT route to single-sig → `run_multisig` → its keyless `ModeViolation` fires. Test `keyless_multisig_md1_refused_at_restore` (n=3, built direct via md_codec) pins exit 2; reviewer also confirmed the n=1 `wsh(sortedmulti)` case is refused at restore (`error: --md1 is template-only … needs a wallet-policy md1`).

- **P1.5 D7 advisory placement + form (focus 2).** `emit_template_wallet_id_advisory` (`bundle.rs:1069-1119`) writes to stderr only; test `d7_wallet_id_on_stderr_not_stdout` confirms `wallet-id` on stderr, absent from stdout. Renders full hex + 4-byte prefix + 12-word phrase via `to_phrase()`. `--expect-wallet-id` mismatch refuses exit 4 (`restore.rs:790-799` `RestoreMismatch`); short-prefix advisory (`<4` bytes warns, does not enforce — test `expect_wallet_id_short_prefix_advises`); correctly SKIPPED under `--origin` with a `notice:` (reviewer confirmed: `--origin m/84'/0'/7' --expect-wallet-id deadbeef` → `notice: --expect-wallet-id is not checked when --origin overrides…`, no false refusal).

- **P1.2 the 4 mutations + form-threading (focus 3).** `synthesize_template_descriptor` (`synthesize.rs:923-993`) applies `tlv.pubkeys=None`, `tlv.fingerprints=None`, `path_decl.paths=Shared(OriginPath{components:vec![]})`, drops the `is_wallet_policy` assert. Stub/csi/labels read the MUTATED `template` clone (`template_id` from `compute_wallet_descriptor_template_id(&template)`, `:967`). Byte-identity holds across DIFFERENT seeds AND across accounts (tests `template_md1_byte_identical_across_different_seeds`, `…_across_accounts`). `Md1Form` threaded through `synthesize_unified`/`synthesize_descriptor` at ALL callers passing `Policy` (verified the descriptor-mode trio `bundle.rs:1725/1836/2080`, `:441`, `verify_bundle.rs:668/767/873/1346`, `import_wallet.rs:1455`, + test callers). Non-regression test `policy_form_byte_identical_to_default` (bip44/49/84/86) confirms `--md1-form=policy` is byte-identical to today.

- **P1.3 binding-stub re-root (focus 5).** Template md1's mk1 binds on `WalletDescriptorTemplateId` (test `template_md1_is_keyless_and_binds_on_template_id` recomputes the stub via the PUBLIC `md_codec` API). Template↔policy cross-reject confirmed both in unit test (`template_md1_does_not_cross_bind_to_policy_form`) and reviewer-run verify-bundle (template md1 + policy mk1 → `✗ mk1_template_stub_bind` exit 4). ms1 is byte-unchanged (live `ms_codec::encode`, `synthesize.rs:981-993`, no id field).

- **P1.4 self_check branch (focus 6).** `self_check_bundle` (`bundle.rs:2255+`) keys `template_form = !desc.is_wallet_policy()`, skips `is_wallet_policy`/pubkeys-absent/`check_mk1_xpub_binding` ONLY for template form; the stub-coherence check (`card.policy_id_stubs.iter().any(|s|*s==expected_stub)`) + mk1 origin/fp + ms1 parity still run. `expected_stub` via the shared `bundle_binding_stub` (`bundle.rs:1148-1157`, same `is_wallet_policy` discriminant `synthesize` emitted under). Not vacuous: reviewer confirmed a wrong-type / cross-mixed card → exit 4. Test `template_bundle_self_check_passes`.

- **P1.1 error variant placement.** `TemplateFormUnsupportedShape` is alphabetical between `SlotInputViolation` and `UnknownHrp` in the enum decl (`error.rs:306/318/323`) AND all three exhaustive blocks: `exit_code` (`:567-569`, exit 2), `kind`/name (`:633-635`), `message` (`:823`, the round-2 third block). `details()` has the `_=>None` catch-all (exempt). Compiles clean.

- **Gate refuses non-canonical descriptor-mode (focus 4, partial).** Reviewer-run: descriptor-mode `sh(wpkh(…))` (bip49) → refused (`canonical_origin` None, message names bip49-nested-segwit); descriptor-mode `wpkh(…)` canonical → emits. Template-mode `--template wsh-sortedmulti --threshold 1` → refused by the PRE-EXISTING `requires N > 1` guard before reaching the new gate. Template-mode `--template bip49` → refused (test `template_form_refuses_bip49_nested_segwit`).

## CRITICAL
None.

## IMPORTANT

### I1 — The multi-tag guard does NOT catch a nested-multi `wsh(sortedmulti(1,@0))` in descriptor-mode; a multisig keyless template IS EMITTED, contradicting the gate's own docstring and the SPEC §5-test-5 "refuse multisig at bundle-emit" requirement.

**Bug.** `synthesize_template_descriptor`'s edge guard (`synthesize.rs:954-963`) checks ONLY the TOP-LEVEL node:
```rust
if matches!(descriptor.tree.body, Body::MultiKeys { .. } | Body::Variable { .. })
   || matches!(descriptor.tree.tag, Tag::Multi | Tag::SortedMulti) { … refuse }
```
For `wsh(sortedmulti(1,@0))` the top node is `tag=Wsh, body=Children([sortedmulti…])` — the multi is NESTED, so neither arm fires. Next, `canonical_origin(&tree)` returns `Some(m/48'/0'/0'/2')` for `wsh(multi/sortedmulti)` (md-codec `canonical_origin.rs:57-62`), and `descriptor.n == 1` for a 1-of-1. All three gates pass → a keyless **multisig** template md1 is emitted.

**Evidence (reviewer-run):**
```
$ mnemonic bundle --network mainnet --md1-form template --group-size 0 \
    --descriptor "wsh(sortedmulti(1,[00000000/48'/0'/0'/2']xpub6Bg…/<0;1>/*))"
# md1 (wallet policy)
md1fnrrwqqpqqgqpsgwqqqdz3ysvdysdzcc          ← multisig template EMITTED (should be refused)
```
The docstring directly above the guard claims the opposite: *"A degenerate `wsh(multi(1,@0))` 1-of-1 carries the `Multi`/`SortedMulti` tag; it is refused explicitly"* (`synthesize.rs:935-938`) — but the descriptor's TOP tag is `Wsh`, not `Multi`, so the claim is false. The SPEC §4.2 edge note pre-classifies the *funds-safety* impact of this 1-of-1 case as "non-blocking" (the seed derives the one key; not the C1 inversion), and reviewer confirmed it is harmless downstream — `cli_template_from_tree` returns `None` for the nested-multi shape, so **restore refuses it** (`error: --md1 is template-only … needs a wallet-policy md1`) and **verify-bundle** likewise can't route it. So this is NOT a silent-wrong-wallet (hence not Critical). But it IS:
  1. a gate the SPEC's own §5-test-5 requires ("refuse … multisig … at bundle-emit") that DOES NOT fire on the descriptor-mode nested-multi path;
  2. a guard whose docstring + code make a false claim about what they reject;
  3. an emit that produces an un-completable dead artifact a user could mistake for a valid shareable template.

The existing test `template_form_refuses_multisig_template` passes for the WRONG reason — it uses `--template wsh-sortedmulti --threshold 2` (n=2, caught by the pre-existing template-mode `N>1` guard), never exercising the n==1 descriptor-mode path where the new gate is the only line of defense.

**Fix.** Make the multi-family guard recursive (or check the canonical-origin *result*): refuse when `canonical_origin` resolves to a multisig path (`m/48'/…/2'` or `…/1'`), OR walk the tree for any `Multi/SortedMulti/MultiA/SortedMultiA` tag, OR (simplest, matching the spec's intent) require the top-level shape to be exactly one of pkh/wpkh/tr-keypath — i.e. gate on `cli_template_from_tree(&descriptor.tree).is_some()` (the same classifier restore/verify already use). Add a descriptor-mode test: `bundle --md1-form=template --descriptor 'wsh(sortedmulti(1,…))'` → exit 2.

### I2 — verify-bundle (P1.7) silently lacks `--origin`; its own docstrings claim `--origin` support that does not exist (plan-fidelity miss + factually wrong comments).

**Gap.** `VerifyBundleArgs` has NO `--origin` field (only `restore` got one, `restore.rs:108`). `verify_singlesig_template` resolves the seed slot and recomposes using `args.account` ONLY (`verify_bundle.rs:497/549/583`). Yet two in-function comments assert `--origin` support: `:296` *"(`--slot @0.<secret>=` + `--account`/`--origin`)"* and the docstring `:456` *"resolve the seed slot … (+ `--account`/`--origin`)"*. SPEC §4.6 lists `--account`/`--origin` for the recompose, and plan P1.7 says "seed + `--account`/`--origin`". So a wallet restored at a non-canonical `--origin` cannot be verified/recomposed through `verify-bundle` (it would resolve at the wrong, canonical-account origin and report a spurious result if the user expected the override).

**Severity rationale:** NOT a funds-safety hole — `restore` (the recovery path) fully supports `--origin` (reviewer confirmed end-to-end: `restore … --origin m/84'/0'/7'` yields the correct `/84'/0'/7'` descriptor + address). `verify-bundle` is a diagnostic/consistency tool, and this is a NEW surface (no regression). But it is a real plan-fidelity miss AND ships misleading comments that future readers will trust.

**Fix.** Either (a) add `--origin` to `VerifyBundleArgs` + thread it through `verify_singlesig_template`'s slot resolution + descriptor build (mirroring `run_singlesig_template_completion`), and add a test; OR (b) if `--origin` on verify-bundle is intentionally deferred, correct the two comments (`:296`, `:456`) to say `--account` only, note the limitation in the SPEC §4.6 / plan P1.7, and file a FOLLOWUP. (a) is the spec-faithful choice.

## MINOR

- **M1 — `cli_template_from_tree` accepts `tr(is_nums=true, tree=None)` as bip86.** The `(Tag::Tr, Body::Tr { tree: None, .. })` arm (`synthesize.rs:234`) ignores `is_nums`, so a NUMS-keypath template classifies as bip86. Harmless: `bundle --template bip86` only ever emits `is_nums=false`, and at restore the completion REBUILDS a `tr(<seed key>)` from type+seed (the template's NUMS-ness never reaches output) — so a NUMS template would simply complete to the user's own bip86 single-sig. Mirrors md-codec's own `canonical_origin` (which also ignores `is_nums` for the `tree:None` arm), so it's consistent. Consider matching `is_nums: false` for precision; non-blocking.

- **M2 — `decode_wallet_id_prefix` rejects odd-length hex with exit 1, but a too-LONG prefix (>16 bytes) is handled by the `id_bytes.len() < prefix.len()` mismatch path → exit 4, not a clearer "prefix longer than the 16-byte id" message.** Cosmetic; the refusal is still loud and correct.

- **M3 — `emit_template_wallet_id_advisory` is best-effort silent on any failure** (`bundle.rs:1078/1082/1090` early-returns). Acceptable per its docstring (cards on stdout are authoritative; the canonical gate already guaranteed a completable single-sig reached here), but a hard internal error in id-recompute would silently drop the D7 disambiguator with no diagnostic. Consider a one-line stderr `warning:` on the `Err(_)` arm. Non-blocking.

## To turn GREEN
1. **(I1)** Fix the template-form multi-family gate so a nested-multi `wsh(sortedmulti(1,@0))` (and any multisig) is refused at bundle-emit in BOTH template-mode and descriptor-mode — recommend gating on `cli_template_from_tree(&descriptor.tree).is_some()` (or refusing a multisig `canonical_origin` result / any recursive multi-tag). Add a descriptor-mode `wsh(sortedmulti(1,…)) --md1-form=template` → exit-2 test. Correct the false docstring at `synthesize.rs:935-938`.
2. **(I2)** Either add `--origin` to `verify-bundle`'s template path (thread through slot-resolution + descriptor build, add a test) to match SPEC §4.6 / plan P1.7; or, if deferring, correct the two `--origin`-claiming comments (`verify_bundle.rs:296`, `:456`), document the `--account`-only limitation, and file a FOLLOWUP.
3. Re-dispatch this review after the fold (per-phase reviewer-loop continues until 0C/0I).
