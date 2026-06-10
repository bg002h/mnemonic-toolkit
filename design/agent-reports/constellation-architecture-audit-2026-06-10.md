<!-- Persisted from workflow constellation-architecture-audit (run wf_d48c71fa-ccc), 2026-06-10.
23 agents, ~2.79M subagent tokens; 11 read-only auditors -> per-unit adversarial verification -> synthesis.
Baselines: toolkit 59c5254, md-codec 8f5a15f, ms-codec bb077bb, mk-codec 95d2f65, gui 97ee471.
48 verified findings (0 critical / 10 important / 15 minor / 23 observation); 4 refuted at verification.
Raw findings JSON alongside: constellation-audit-2026-06-10-findings.json -->

# m-format Constellation — First Independent Fable Audit Report

## 1. Executive Summary

This is the first uncorrelated review pass over a five-repo, funds-handling backup toolkit (seeds, keys, descriptors) that was built and reviewed almost entirely under an older model. The headline: **the cryptographic core is sound** — no finding shows funds loss, a wrong card emitted to steel, or a live plaintext-secret leak. The recurring weaknesses are exactly the failure modes the orchestrator flagged: **drift gates that verify a different artifact than the one shipped or compare a value against itself**, **secret-classification gaps in the GUI that are latent only because persistence is not yet wired**, and **cross-repo linkage formulas that two repos compute differently**. The codec layer defends itself well at the wire level; most real bugs live in the *toolkit/GUI seams* and in the *test/gate scaffolding* that is supposed to catch regressions but structurally cannot.

**The 5 highest-leverage fixes (funds-safety order):**

1. **Fix the cross-repo stub-formula divergence (mk-codec ↔ toolkit).** `mk encode/verify --from-md1` computes the 4-byte `policy_id_stub` from an *encoding-sensitive* hash while the toolkit uses the *encoding-stable* `WalletPolicyId` — the two never match, so `mk verify --from-md1` against a real toolkit card reports a spurious mismatch and `mk encode --from-md1` produces cards `bundle --self-check` rejects. And the test that should catch this is a tautology. (`stub-formula-divergence` + `from-md1-test-tautology`)
2. **Plug the secret-classification class before persistence is wired (GUI).** Four distinct holes — `--phrase`/`--ms1` mis-classified `secret:false`, positionals never redacted, WIF/raw-hex private keys in tree key fields not caught by the xprv-prefix heuristic — converge into one theme: master-secret material rendered in cleartext and/or droppable to `state.json` the instant Phase 8 wires `save()`. None can leak today (persistence has zero callers), so this is a **hard release-blocker to resolve before that wiring**, not an emergency.
3. **Close the vacuous secret-flag completeness gate (toolkit).** The gate presented as the toolkit half of the secret-flag drift defense compares `flag_is_secret` against itself — it cannot fail and provides zero protection against a future mis-classified secret flag. This is the leading-indicator gate for hole #2's whole class.
4. **Tag-gate the GUI release pipeline + bump its actions to @v5 (GUI).** GUI release tags run *no tests at all* — every drift/pin-coherence gate is bypassable on the release commit; and the release-publishing actions are still on Node-20 `@v4`, force-removed **2026-06-16** (days away). Both are mechanical fixes to the exact surface that publishes binaries.
5. **Fix the `combine_shares` panic + the multisig csi grouping bug (ms-codec / toolkit).** `ms combine --to phrase` panics (abort, exit 101) on a valid-checksum non-standard-length share set because the Entr arm skips the length validation the single-string path enforces; and multisig mk1 cards collide their `chunk_set_id` for same-xpub-different-path cosigners, producing a spurious `verify-bundle` failure on two individually-valid cards.

---

## 2. Cross-Cutting Patterns (dedup + cluster)

Four root-cause clusters span multiple units/repos:

- **CLUSTER A — "the gate verifies the wrong thing / itself."** Five findings: the vacuous secret-flag self-comparison (`vacuous-secret-flag-gate`), the `--from-md1` test tautology (`from-md1-test-tautology`), the g6 invariant checking sibling-`master` not the pinned tag (`g6-invariant-sibling-master-not-pin`), the conditional-drift doc lie + PATH-binary fallback (`conditional-drift-gate-stale-binary-doc-lie`), and the secret-drift gate's silent version-skip (`secret-drift-gate-version-skip-silent`). Common shape: a gate that *looks* protective but compares against a self-derived or wrong-version artifact. The completeness-via-macro technique that *does* work already exists in-repo (`declare_node_type_variants!`) and is the prescribed remedy for several.

- **CLUSTER B — "secret never classified secret (GUI), latent behind unwired persistence."** Five findings: `secret-false-flags-render-cleartext-no-confirm`, `positionals-never-redacted`, `tree-wif-hex-privkey-in-key-fields-unredacted`, `slot-secret-values-rendered-unmasked`, plus the umbrella `persistence-unwired-redaction-never-runs`. The redaction layer's drop-set is keyed on names/node-types/subkeys and provably misses positionals and non-xprv private keys; two flags are mis-classified at the schema level. The mitigating fact (no `save()` caller) is what holds severity at *important* not *critical* — but it converts the whole cluster into a **Phase-8 release gate**.

- **CLUSTER C — "mk1 chunk_set_id is derived inconsistently and collision-prone."** Four findings (`mk1-csi-collision-multisig-grouping`, `n1-vs-nge2-csi-derivation-inconsistency`, `anti-collision-16bit-invariant-false`, `mk1-chunk-set-id-fingerprint-grouping-assumes-distinct-fps`). One root cause: n=1 seeds the csi from the policy stub, n≥2 from the xpub fingerprint, the displayed card-id always from the stub. The only *important* consequence is the same-xpub-different-path grouping failure; the rest is display/doc drift the codec's integrity hashes defend against.

- **CLUSTER D — "validation skipped on one path that another path enforces."** Three findings (`combine-no-length-validation-panic`, `import-json-schema-version-unchecked`, `localize-broad-error-collapse`): a length/version/error check present on the canonical path is absent on a sibling path. Only the `combine_shares` one is reachable with real input today.

---

## 3. Prioritized Findings

### CRITICAL
None. No finding demonstrates funds loss, a wrong steel-engraved card, or a live plaintext-secret leak.

---

### IMPORTANT

**I1 — Cross-repo stub-formula divergence breaks `mk … --from-md1` linkage** *(CLUSTER A)*
- **Repo/loc:** mk-codec `crates/mk-cli/src/cmd/mod.rs:57-63` (used `encode.rs:68-70`, `verify.rs:106-117`); toolkit `crates/mnemonic-toolkit/src/synthesize.rs:157-159`, `cmd/bundle.rs:2078-2096`.
- **Defect:** `derive_stub_from_md1` = `SHA-256(encode_payload(desc))[..4]` (= encoding-sensitive `Md1EncodingId[..4]`); toolkit stamps/validates with `compute_wallet_policy_id(desc)[..4]` (encoding-stable). The two differ for ~every descriptor → `mk verify --from-md1` reports `ContentMismatch`; `mk encode --from-md1` cards fail `bundle --self-check`. mk SPEC §3.3 itself says impls SHOULD use the `WalletPolicyId` helper.
- **Fix:** change `derive_stub_from_md1` to `compute_wallet_policy_id(&descriptor).as_bytes()[..4]`. **Caveat the fix must clear:** mk-cli pins md-codec `0.34.0` vs toolkit's `0.35.0` — confirm `0.34.0` exposes `compute_wallet_policy_id` and is byte-identical, else bump the pin in lockstep. Update mk SPEC §3.3/§5.
- **Disposition:** fix-now. FOLLOWUP (companion in both repos): `mk-cli-from-md1-stub-must-use-walletpolicyid`.

**I2 — `--from-md1` test is a tautology masking I1** *(CLUSTER A)*
- **Repo/loc:** mk-codec `crates/mk-cli/tests/round_trip.rs:44-78` (oracle :46-49, assert :77).
- **Defect:** the oracle recomputes the implementation's own `encode_payload+sha256+[..4]` chain, so it passes regardless of whether the stub matches the toolkit. Textbook "test synthesizes the state the live path produces."
- **Fix:** add a cross-check assertion against `compute_wallet_policy_id(&descriptor).as_bytes()[..4]` — it fails now (surfacing I1), becomes the real cross-constellation guard after.
- **Disposition:** fix-now, bundled with I1.

**I3 — Vacuous secret-flag completeness gate** *(CLUSTER A / leading indicator for CLUSTER B)*
- **Repo/loc:** toolkit `crates/mnemonic-toolkit/tests/cli_gui_schema_v5_extensions.rs:283-307`; `src/cmd/gui_schema.rs:1196`; `src/secrets.rs:49-64`.
- **Defect:** both sides of the equality derive from `flag_is_secret(name)` (the schema's `secret` bit IS that predicate), so `assert_eq` is a tautology. No gate over `flag_is_secret`'s own completeness; the would-be backstop `lint_argv_secret_flags` is transitive on the same predicate. A new secret flag named outside the 11-entry `matches!` emits `secret:false` → GUI mask/zeroize disabled, all gates green.
- **Fix:** add a real completeness cell walking the live clap/gui-schema surface with a name heuristic (`passphrase|secret|password|share|seed|mnemonic|wif|xprv|entropy|digits`, excluding path/file/bool flags), failing on a heuristic-positive flag that `flag_is_secret` returns false for; or drive `expected_secret` from an independent literal allow-list.
- **Disposition:** fix-now. FOLLOWUP: `secret-flag-completeness-gate-non-circular`.

**I4 — GUI mis-classifies master-secret flags `secret:false` → cleartext render, no run-confirm** *(CLUSTER B)*
- **Repo/loc:** gui `src/schema/mnemonic.rs:2286` (+2448/2718), `src/schema/ms.rs:321`.
- **Defect:** `xpub-search --phrase` and `ms repair --ms1` are `secret:false` → render via plain `text_edit_singleline`, `should_warn_on_paste` false, `should_confirm_run` false. Master BIP-39 phrase / recoverable ms1 typed and displayed in cleartext, run without confirmation. (Persistence is incidentally closed via the name-net, so this is a live *display/confirm* exposure, not a persist leak.)
- **Fix:** flip both to `secret:true` — likely needs a toolkit `gui-schema` classification fix first (GUI mirrors the toolkit `secret` field), which loops back to I3. Restores masked widget + run-confirm + paste-warn.
- **Disposition:** fix-now. FOLLOWUPs already filed and OPEN (`FOLLOWUPS.md:52,:60`).

**I5 — Positional secret-equivalents cloned to `state.json` with zero redaction (latent)** *(CLUSTER B)*
- **Repo/loc:** gui `src/persistence.rs:115`; `src/form/mod.rs:53-63` (no `secret` field); `src/secrets.rs:200-225`.
- **Defect:** `redact_for_persistence` copies `positionals` verbatim; none of the four drop-classes match a positional; `PositionalArgSchema` has no `secret` field; `should_confirm_run` never inspects positionals; they render via bare `text_edit_singleline`. `ms combine <shares>` / bare ms1 are "Secret-equivalent" per their own help.
- **Fix:** add `secret: bool` to `PositionalArgSchema`, a redaction arm dropping secret positionals, route through `SecretLineEdit`, extend `should_confirm_run`. **Persistence MUST NOT be wired until this lands.**
- **Disposition:** fix-before-Phase-8. FOLLOWUP filed OPEN (`positional-secrets-not-redacted-at-persist`, `FOLLOWUPS.md:68`).

**I6 — WIF / raw-hex private key in tree key/keys field survives redaction (latent)** *(CLUSTER B)*
- **Repo/loc:** gui `src/form/tree_model.rs:650-669` (`is_xprv_like`/`blank_xprv_keys`); runs unconditionally at `:176-187`.
- **Defect:** `is_xprv_like` matches only the `prv`-at-byte-1..4 extended-private shape. The toolkit's own gate (`gate.rs:275-276`) states verbatim WIF/raw-hex secrets are *not prefix-detectable*. So a WIF (`K`/`L`/`c`-prefix) or raw-hex priv key pasted into a key/keys row passes the redaction walk untouched and would write to `state.json`. Distinct from the RESOLVED `tree-xprv-heuristic-only-covers-key-fields` (which was hex/w fields).
- **Fix:** don't rely on a prefix heuristic the toolkit calls incomplete — either allowlist (blank key/keys content unless it positively matches an xpub/descriptor-pubkey shape) or refuse to persist a tree that hasn't passed the watch-only gate.
- **Disposition:** fix-before-Phase-8. FOLLOWUP: `tree-wif-hex-privkey-in-key-fields-unredacted`.

**I7 — GUI release tags run NO tests** *(CLUSTER A)*
- **Repo/loc:** gui `.github/workflows/build.yml:3-8`, `schema-mirror.yml:3-7`.
- **Defect:** on a `mnemonic-gui-v*` tag only `build.yml` fires (clippy + build + release); the whole suite (schema_mirror, pin_coherence, secret-drift, …) lives in `schema-mirror.yml`, which has no `tags:` trigger. Re-tag / hotfix-branch tag / force-push / tag-before-master-run-finishes → release published from a commit whose gates never ran.
- **Fix:** add `tags: ['mnemonic-gui-v*']` to `schema-mirror.yml`'s push trigger (or a `cargo test --workspace` step to `build.yml`'s tag path), matching the toolkit's tag-gated `install-pin-check.yml`/`changelog-check.yml`.
- **Disposition:** fix-now. FOLLOWUP: `gui-release-tag-runs-no-gates`.

**I8 — GUI workflows still on actions @v4 (Node-20), force-removed 2026-06-16** *(CLUSTER A)*
- **Repo/loc:** gui `.github/workflows/build.yml:19,56,112,125`; `schema-mirror.yml:15`.
- **Defect:** all 5 JS-action sites (checkout ×3, upload-artifact, download-artifact) still `@v4` while the toolkit deliberately bumped to `@v5`. The release-publishing job (`download-artifact@v4` at `build.yml:125`) is exactly what breaks when Node-20 is removed — **days away**. No CI gate covers `actions/*` pins.
- **Fix:** bump all 5 to `@v5` before 2026-06-16. Add a cross-repo "actions runtime major" checklist line.
- **Disposition:** fix-now (time-critical). FOLLOWUP: `gui-actions-v4-to-v5-node20-deprecation`.

**I9 — `ms combine --to phrase` panics on valid-checksum non-standard-length share set** *(CLUSTER D)*
- **Repo/loc:** ms-codec `crates/ms-codec/src/envelope.rs:167-188` (Entr arm, no validate); `shares.rs:236-242`; `crates/ms-cli/src/cmd/combine.rs:96-97` (`.expect` panic).
- **Defect:** the Entr arm of `dispatch_payload` returns `Payload::Entr(data[1..])` without `.validate()` (the Mnem arm validates; the single-string `decode()` path validates via rules 9+10). codex32 `from_string` accepts any `[48,94)` valid-checksum string and `interpolate_at` only enforces equal intra-set length, so a non-ms-length set recovers an entropy length ∉ {16,20,24,28,32}. `from_entropy_in(...).expect("cannot fail")` then returns `Err(BadEntropyBitCount)` → **panic, exit 101** on the default `--to phrase`. `--to entropy` silently hex-dumps wrong-length bytes; only `--to ms1` errors cleanly.
- **Fix:** add `payload.validate()?` to the Entr arm of `dispatch_payload` so the gap closes for all callers; add a regression test feeding a non-standard-length valid-checksum set and asserting `Err`, not panic.
- **Disposition:** fix-now. FOLLOWUP: `combine-entr-arm-missing-payload-length-validation`.

**I10 — Multisig mk1 `chunk_set_id` collides for same-xpub-different-path cosigners** *(CLUSTER C)*
- **Repo/loc:** toolkit `crates/mnemonic-toolkit/src/synthesize.rs:277-278`; `cmd/verify_bundle.rs:1568-1602,1625-1682`; distinctness `cmd/bundle.rs:446-460`.
- **Defect:** n≥2 csi = `derive_mk1_chunk_set_id(xpub.fingerprint())` (path-independent). BIP-388 distinctness rejects only when BOTH `xpub.to_string()` AND path are equal, so same-xpub-different-path passes. `verify_bundle` groups supplied mk1 by csi and decodes per group; two same-fingerprint cards merge into one group → `ChunkedHeaderMalformed` → spurious verify failure (slot[0]→`DecodeFailed`, slot[1]→`NotSupplied`). Both cards are individually correct/decodable, so not funds loss / not wrong bytes; trigger config is uncommon.
- **Fix:** make the multisig csi slot-unique and reproducible (policy stub + slot index, or fold the origin path into the seed alongside the fingerprint). Regression test: 2-of-2 reusing one xpub at two paths, round-tripped through verify-bundle, must map both. Folds naturally with C-cluster cleanup (`n1-vs-nge2-csi-derivation-inconsistency`).
- **Disposition:** fix-now. FOLLOWUP: `mk1-csi-multisig-same-xpub-collision`.

---

### MINOR

**M1 — `self_check_bundle` doesn't verify mk1 xpub ↔ descriptor pubkey binding** — toolkit `cmd/bundle.rs:2060-2139`. Self-check validates stub/fingerprint/ms1 but never compares `card.xpub` to `desc.tlv.pubkeys` (verify-bundle DOES, `:1654`). Wrong-xpub-on-card class uncaught in self-check. *Fix:* add the `xpub_to_65` membership assert. *Disposition:* fix-now (cheap defense-in-depth). FOLLOWUP: `self-check-mk1-xpub-binding`.

**M2 — `localize()` collapses all `from_str` errors to `None`** — toolkit `descriptor_builder/gate.rs:438-453` vs the narrowed `localize_parse_failure:405-418`. Fail-closed (policy still refused); only a diagnostic node-path label could mis-point if step-2 is ever relaxed. *Fix:* narrow to `Err(NonTopLevel) => None`, else root fallback. *Disposition:* fix-now or FOLLOWUP `gate-localize-narrow-error-catch`.

**M3 — inspect/repair accept inline secret ms1 on argv without the argv advisory** — toolkit `cmd/inspect.rs:88-156`, `cmd/repair.rs:101-218`. ms1 is `PrivateKeyMaterial`; 14+ other commands emit `secret_in_argv_warning`; both support `--ms1 -`. Mitigated by `set_non_dumpable`. *Fix:* fire the advisory for inline ms1. *Disposition:* fix-now. FOLLOWUP: `inspect-repair-ms1-argv-advisory`.

**M4 — Emitted priv-key hex/WIF strings not `Zeroizing`** — toolkit `cmd/silent_payment.rs:256-257,271-272,97-98`; `cmd/nostr.rs:212,221,231,249` (extra unscrubbed copy at `:221`). Intended stdout output, consistent with upstream zeroize posture. *Fix:* wrap derived hex/WIF in `Zeroizing`. *Disposition:* FOLLOWUP `emitted-privkey-strings-not-zeroizing`.

**M5 — Slot secret values render unmasked; row removal doesn't zeroize** — gui `src/form/slot_editor.rs:219-236`. `should_confirm_run` and persistence-drop DO cover slots; gap is on-screen mask + per-removal residue. *Fix:* masked widget gated on `is_secret_bearing`, zeroize on remove. *Disposition:* FOLLOWUP `slot-editor-mask-and-zeroize-on-remove` (fold with Phase-8 work).

**M6 — Paste-warn modal is dead code presented as a shipped mitigation** — gui `src/secrets.rs:164-196`. `should_warn_on_paste`/`PASTE_WARN_MODAL_TEXT` have zero src callers; module prose claims it's active. *Fix:* wire it into `SecretLineEdit` paste handling, or remove and downgrade the prose. *Disposition:* FOLLOWUP `paste-warn-modal-wire-or-remove` (overlaps `paste-warn-live-wiring-untested`).

**M7 — `conditional_drift` gate runs against any `mnemonic` on PATH; doc falsely claims it skips when MNEMONIC_BIN unset** — gui `tests/gui_schema_conditional_drift.rs:13,28-33,194-210`. CI is correct (sets MNEMONIC_BIN to the pinned install); impact is dev-laptop false-green + doc lie. *Fix:* correct the doc; require MNEMONIC_BIN explicitly or log the resolved binary version. *Disposition:* FOLLOWUP `conditional-drift-doc-and-bin-resolution`.

**M8 — Secret-drift gate silently skips on gui-schema version < 5; CI smoke only asserts version ≥ 1** — gui `tests/schema_mirror_secret_drift.rs:61-91`. Latent (toolkit emits v5). *Fix:* CI smoke assert version ≥ 5, and/or panic-instead-of-skip when MNEMONIC_BIN is set. *Disposition:* FOLLOWUP `secret-drift-gate-version-floor-assert`.

**M9 — g6 mlock byte-equality checks sibling at moving `master`, not the pinned ms-cli tag** — toolkit `.github/workflows/rust.yml:215-220` (`ref: master`) vs `scripts/install.sh:38` (`ms-cli-v0.7.0`). `rust.yml` is not tag-triggered, so it's a dev-HEAD coherence check by construction. *Fix:* check out the sibling at the install.sh tag, or document g6 as a dev-HEAD check. *Disposition:* FOLLOWUP `g6-pin-sibling-at-released-tag`.

**M10 — import-json envelope `schema_version` declared but never validated** — toolkit `wallet_import/json_envelope.rs:62-90,149-170`; consumers `cmd/export_wallet.rs:619`, `cmd/bundle.rs:1716`. Constellation-internal format; only an older binary fed a newer envelope mis-maps (serde drops unknowns). In-repo precedent exists (`build_descriptor.rs:282 SUPPORTED_SCHEMA_VERSION`). *Fix:* reject unsupported versions on the consume path. *Disposition:* FOLLOWUP `import-json-schema-version-gate`.

**M11 — HRP classifiers case-sensitive; reject valid all-uppercase codec cards** — toolkit `repair.rs:106-119`, `cmd/restore.rs:1026`. Codecs accept uppercase (BIP-173); toolkit misroutes to a confusing parse error. Every path lands on a hard error (no silent wrong output). *Fix:* lowercase before the HRP probe. *Disposition:* FOLLOWUP `hrp-classifier-case-insensitive`.

**M12 — PolicyNode grammar-coverage guard is hand-list-vs-hand-list (vacuous on joint omission)** — toolkit `descriptor_builder/ir.rs:289,345-354`; the compile-forcing `declare_node_type_variants!` macro precedent exists at `cmd/convert.rs:1767`. *Fix:* apply the macro to `PolicyNode`. *Disposition:* FOLLOWUP `policynode-coverage-via-variant-macro` (CLUSTER A).

**M13 — 25 of 26 `vectors/v0_2` multisig goldens read by no test** — toolkit `tests/vectors/v0_2/*`; only `bip84-…` is read (`cli_self_check.rs:13`). Dead goldens; wire regression for those shapes caught by nothing. Already tracked OPEN (`orphaned-v0_2-md1-vectors-no-harness`, `FOLLOWUPS.md:89-95`). *Fix:* delete them or wire a `read_dir` decode-assert harness. *Disposition:* resolve the existing FOLLOWUP.

**M14 — md-codec tap-leaf validator only checks the leaf-root tag, never recurses into compound leaves** — md-codec `validate.rs:145-169`; doc-comment overstates enforcement. Safety net is real: `node_to_miniscript<Tap>` (`to_miniscript.rs:448-453`) rejects nested forbidden wrappers with a typed error at derive time. *Fix:* recurse into child bodies, or narrow the doc-comment. *Disposition:* FOLLOWUP `taptree-leaf-validator-recurse-or-doc`.

---

### OBSERVATION (no action forced — recorded for the audit trail)

- **n=1 vs n≥2 csi derivation inconsistency + display/wire mismatch** (toolkit `synthesize.rs:260 vs 278`, `cmd/bundle.rs:1087-1109`); false in-code comment at `:1087-1088`. Root of I10/CLUSTER C — fold the fix together. *FOLLOWUP:* `mk1-csi-unify-derivation`.
- **Documented "leading 16 bits of chunk_set_id agree across md1/mk1/ms1" invariant is false/unenforced** (CHANGELOG:1899 / technical-manual §IV.2; ms1 has no csi at all). *Fix:* correct the manual to describe the shared `policy_id_stub`. *FOLLOWUP:* `anti-collision-16bit-invariant-doc-correction`.
- **BIP-388 policy-name lossy round-trip** (toolkit `wallet_export/pipeline.rs:207` hardcodes `"imported-descriptor"`; `wallet_import/pipeline.rs:161-207` reads name into `_name`, never threads it). Descriptor/keys byte-faithful. *FOLLOWUP:* `bip388-policy-name-roundtrip`.
- **gate step-1 timelock check is partial** (toolkit `gate.rs:244-256`); step-2 `from_str` is the authoritative range gate (fail-closed). Doc note only.
- **silent-payment `--secret phrase=` hardcodes English** (toolkit `silent_payment.rs:155`); fail-safe (non-English fails checksum). No `--language` flag. *Doc or add flag.*
- **addresses `--from` resolves `@env:` for non-secret xpub source** (toolkit `addresses.rs:159-163`), contrary to the env_sentinel secret-surface-only doc. No leak (xpub public, can't begin `@env:`). *Doc or gate on `is_secret_bearing`.*
- **Preview line + run-confirm modal display secret argv tokens in cleartext** (gui `main.rs:804,842-844`). Arguably intended CLI preview; warrants a deliberate mask-secret-tokens decision. *FOLLOWUP:* `gui-preview-mask-secret-tokens`.
- **`--json` envelope wire-shape ungated; only frozen v0.27.0 fixtures (~25 versions stale)** (gui `tests/cli_envelope_smoke.rs`). Documented-accepted; low blast radius (no typed deserialization). New `archetype_schema_mirror`/`spec_nodes_mirror` DO gate the spec-schema surface. *At minimum refresh fixtures to current pin.*
- **`flag_is_secret` is a hand allowlist with no completeness gate** — the design-accepted boundary behind I3; would surface via the more-visible runtime advisory/mask/zeroize. *Optional review-checklist heuristic.*
- **Two-miniscripts patch is load-bearing; md-codec's SortedMultiA refusal message is stale-in-context** (toolkit `Cargo.toml:9-16`; md-codec `to_miniscript.rs:406-408`). Behavior safe (refuse, not mis-render). *Drop patch + fix message when miniscript #910/#915 publishes.*
- **verify-bundle fingerprint grouping assumes distinct fingerprints** — caught (not mis-decoded) by mk-codec's count + cross-chunk-hash guards; 2⁻³² accidental. Optional: assert distinct-group-count == cosigner-count for a clearer diagnostic.
- **mk1 wire non-canonical at the path layer** (mk-codec `path.rs:101-132`); decode many-to-one, normalized on re-encode, integrity hash over canonical bytes only — latent. *Doc or reject explicit-form table collisions.*
- **multi_a/sortedmulti_a key count capped at 32 by the 5-bit field** (md-codec `tree.rs:106-121,226-237`); tapscript ceiling is 999. Oversized cleanly refused. *Document the 32-key limit.*
- **SortedMultiA derive gap is correctly fenced** (md-codec `to_miniscript.rs:406-411`, typed error, no panic). Keep the FOLLOWUP open.
- **TLV rollback ≤7-bit tolerance wider than actual ≤4-bit padding** (md-codec `tlv.rs:286-302`). Only a hand-crafted 5-7-bit-junk tail slips; decoded descriptor still structurally valid. *Tighten to ≤4 or document.*
- **`total_chunks − 1` underflow unguarded** (mk-codec `header.rs:88`); unreachable today (`chunk.rs:73 .max(1)`). *Add `debug_assert!(total_chunks >= 1)`.*
- **`combine_shares` recovered-secret `Codex32String` not zeroized** (ms-codec `shares.rs:236`); upstream-blocked dormant dep, lifetime-minimized. *Widen the existing `rust-codex32-zeroize-upstream` FOLLOWUP to name the combine/recovery side.*
- **Several in-crate `synthesize.rs` tests assert presence not value** (`synthesize.rs:959-1022`); contained because `cross_binding_holds_round_trip` (`:1024-1050`) + integration goldens DO pin value-correctness. Note-to-editors only.
- **Hand-frozen lint canons (`lint_zeroize_discipline`) have no set-equality completeness** (`tests/lint_zeroize_discipline.rs:251-294`); count-range + file-level substring only, vs the argv lint's real closures. No demonstrated un-zeroized secret.
- **`cli_self_check.rs` module doc claims it reads a wsh-sortedmulti fixture it never opens** (`:5 vs :13`). Doc overstatement.
- **`conditional_visibility.rs` cells assert the conditional fn's map, not the live render loop**; emit-time suppression independently proven by `argv_assembler_visibility.rs`. No live bug.
- **POSITIVE: codex32 PR#2 padding-bug exposure verified absent against source** (ms-codec `shares.rs:180-243`) — `combine_shares` uses `from_string`+`interpolate_at`+`parts().data()`, never `from_seed`; the named regression test pins PR#2's exact secret across all 2-of-3 pairs. The FOLLOWUP's "NOT exposed" status matches the code. No action.

---

## 4. What the Audit Did NOT Find / What Stands Up Well

- **No funds-loss / wrong-card / live-secret-leak path** surfaced anywhere across all five repos. The most severe defects are a spurious-failure (I10), a panic-not-corruption (I9), and a cross-CLI linkage mismatch on a documented *indexing aid* (I1) — none alters emitted card bytes or derives a wrong address.
- **The codec wire layers defend themselves robustly.** mk-codec's reassembly enforces chunk-count + total-agreement + index-bounds + a cross-chunk SHA-256 hash (`chunk.rs:131-195`), so even a fingerprint collision is *caught*, not mis-decoded. md-codec's SortedMultiA and nested-forbidden-tap-leaf gaps are fenced with typed errors at derive time, never silent mis-renders. The `from_entropy_in`/`from_string` range checks are sound; the only gap is one *path* that skips them (I9).
- **The single-string `decode()` path and the user-invoked `verify-bundle` are the well-built reference paths** — they DO enforce payload-length validation (rules 9+10), the xpub↔pubkey binding (`:1654`), and the full drift checks. The bugs are consistently in the *sibling* paths (combine, self-check) and *gate scaffolding*, not the canonical flow.
- **The codex32 PR#2 padding bug is genuinely not reachable** by the combine path — verified against source, not prose, with real live-path regression coverage.
- **The GUI's persistence redaction is correct for the channels it does cover** (named secret flags, secret node types, secret slot subkeys) and the exit-sweep zeroes slot values; the gaps are the *uncovered channels* (positionals, non-xprv private keys) and the fact the whole layer is dormant. Crucially, **nothing leaks today** — every GUI persistence finding is latent behind an unwired `save()`.
- **The new spec-schema surface IS wire-shape-gated** (`archetype_schema_mirror`, `spec_nodes_mirror`) — a real improvement over the flag-name-only `schema_mirror`; the ungated surface is now only the older `--json` envelopes.
- **The completeness-via-macro technique that several gates lack already exists in-repo** (`declare_node_type_variants!`) — the fixes are about *applying a known-good pattern consistently*, not inventing new machinery.