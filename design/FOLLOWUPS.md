# Follow-up tracker

Single source of truth for items that surfaced during a review or implementation pass but were not fixed in the same commit. Mirrors the conventions of the sibling `descriptor-mnemonic`, `mnemonic-key`, and `mnemonic-secret` repos.

## How to use this file

**Format for each entry:**

```markdown
### `<short-id>` — <one-line title>

- **Surfaced:** Phase X review of commit <SHA>, or "inline TODO at <file>:<line>"
- **Where:** `<file>:<line>` or "design — SPEC §X"
- **What:** 1–3 sentences describing the gap or improvement opportunity
- **Why deferred:** the reason it didn't ship in the original commit
- **Status:** `open` | `resolved <COMMIT>` | `wont-fix — <one-line reason>`
- **Tier:** `v0.1-blocker` | `v0.1-nice-to-have` | `v0.2` | `cross-repo` | `v1+` | `external`
- **Tags:** *(optional)* space-separated free-form tags for grouping FOLLOWUPS into themes that cut across Tiers. Used to reference a thematic batch as a group in user-driven planning. Convention: lowercase kebab-case; multiple tags allowed. Established tags: `wallet` (v0.28.0 cycle's 11-entry surface — 6 new wallet-import parsers + BSMS BIP-129 work + cross-format matrix + parser-side gaps surfaced during execution).
```

Reference the `<short-id>` from commit messages when closing: `closes FOLLOWUPS.md <short-id>`.

## Tiers (definitions)

- **`v0.1-blocker`**: must fix before tagging `mnemonic-toolkit-v0.1.0`. (Empty after release.)
- **`v0.1-nice-to-have`**: should fix before v0.1 if time permits, but won't block release. Documented in v0.1's CHANGELOG if shipped.
- **`v0.2`**: explicitly deferred to v0.2 (multisig templates, non-zero account, K-of-N share bundles).
- **`v0.2-nice-to-have`**: surfaced during v0.2 review; non-blocking. Documented in v0.2's CHANGELOG if shipped.
- **`v0.3`**: explicitly deferred to v0.3 (user-supplied descriptor passthrough; resolve during v0.3 cycle).
- **`v0.3-nice-to-have`**: surfaced during v0.3 review; non-blocking.
- **`v0.4-cross-repo`**: deferred to v0.4 AND requires coordination with sibling repos.
- **`v0.4-nice-to-have`**: surfaced during v0.4 review; non-blocking. Documented in v0.4's CHANGELOG if shipped.
- **`v0.4.1`**: explicitly deferred from v0.4.0 to a v0.4.1 follow-on patch (typically scope-safety deferrals).
- **`v0.4.2`**: explicitly deferred from v0.4.1 to a v0.4.2 follow-on patch.
- **`v0.4.2-nice-to-have`**: surfaced during v0.4.1 review; non-blocking. Documented in v0.4.2's CHANGELOG if shipped.
- **`v0.4.3`**: explicitly deferred to a v0.4.3 follow-on patch.
- **`v0.4.3-nice-to-have`**: surfaced during v0.4.2 review; non-blocking.
- **`v0.4.4`**: explicitly deferred to a v0.4.4 follow-on patch.
- **`v0.4.4-nice-to-have`**: surfaced during v0.4.3 review; non-blocking.
- **`v0.5`**: explicitly deferred to a v0.5 minor release (typically scope too large for a v0.4.x patch).
- **`cross-repo`**: depends on coordination with sibling repos (`descriptor-mnemonic`, `mnemonic-key`, `mnemonic-secret`). Mirrored by a companion entry in the affected sibling's tracker; both cite each other.
- **`v1+`**: deferred indefinitely.
- **`external`**: depends on upstream work (e.g., a sibling crate exposing a helper).

---

## Open items

### `restore-multisig-cosigner-scope` — `mnemonic restore` multisig-cosigner half: own seed + shared md1 + other cosigners' mk1/xpub → concrete multisig descriptor + cosigner cross-check

- **Surfaced:** 2026-06-04, `mnemonic restore` cycle (toolkit v0.43.0). SPEC-R0 round-0 C1 descoped multisig from the v0.43.0 single-sig ship; see `design/SPEC_mnemonic_restore.md §11` and `design/IMPLEMENTATION_PLAN_mnemonic_restore.md` (single-sig shipped at this cycle's tag).
- **Where:** `crates/mnemonic-toolkit/src/cmd/restore.rs` (a multisig branch); bridge candidates: `crates/mnemonic-toolkit/src/wallet_export/mod.rs:262` (`template_from_descriptor`, takes a miniscript `MsDescriptor` — NOT `md_codec::Descriptor`), `crates/mnemonic-toolkit/src/cmd/bundle.rs:1015` (`extract_multisig_threshold(node: &md_codec::tree::Node) -> Option<u8>` — currently **private**), `crates/mnemonic-toolkit/src/cmd/bundle.rs:1138` (`bundle_run_unified_descriptor` — the production md1→concrete `--descriptor` STRING lex/resolve/parse/bind path), `crates/mnemonic-toolkit/src/wallet_export/pipeline.rs:18` (`build_descriptor_string`), and `descriptor-mnemonic/crates/md-codec/src/to_miniscript.rs:53` (`to_miniscript_descriptor`, errors `MissingPubkey { idx }` at `:72` on a template-only md1).
- **What:** Extend `mnemonic restore` to the multisig-cosigner scenario the user originally asked for: own seed (`ms1`/`phrase`/`entropy`/`seedqr`) + the shared md1 wallet-policy template + the other cosigners' public keys (`mk1` decoded via `mk_codec::decode`, or raw `xpub`) → a concrete watch-only multisig descriptor, with the own-seed-derived cosigner xpub cross-checked against the md1's slot for the supplied position. Carries the same fingerprint hard-gate / `--allow-mismatch` / watch-only-out invariants as single-sig.
- **Why deferred:** SPEC-R0 C1 — the md1→concrete-descriptor route is NOT implementable from the originally-cited APIs: `template_from_descriptor` takes a miniscript `MsDescriptor` (not `md_codec::Descriptor`); `md_codec::Descriptor` has no `Display`; `to_miniscript_descriptor` errors `MissingPubkey` on a template-only md1; `extract_multisig_threshold` is private. The follow-on SPEC must choose + R0 one of the three SPEC §11 bridge options: **(a)** accept a `--descriptor '<@N-template-string>'` input only (reuse the verified `bundle_run_unified_descriptor` bind path) with `--md1` as cross-check-only; **(b)** derive `CliTemplate` from md1 **policy params** — script-type + threshold `k` via `extract_multisig_threshold(&d.tree)` (bump to `pub(crate)`) + `d.n` — then `build_descriptor_string`; or **(c)** `to_miniscript_descriptor` with its wallet-policy-mode constraint spelled out. Plus SPEC §11 I4 (wallet-policy-vs-template-only `tlv.pubkeys` auto-detect branch), the cosigner cross-check, and `--cosigner @N=mk1|xpub` (no new slot subkey). Needs its own SPEC + mandatory R0 to 0C/0I before any code. md-codec pinned `0.35.0`.
- **Status:** `resolved` mnemonic-toolkit-**v0.44.0**. Built via bridge option **(c)** — the decisive reframe (runtime-verified) is that a toolkit-emitted multisig md1 is a **wallet-policy** card (`d.is_wallet_policy()`; `tlv.pubkeys` populated for all N), so the concrete descriptor is reconstructible from the **md1 alone**: `chunk::reassemble` → taproot/template-only refusal (exit 2) → `to_miniscript_descriptor(&d,0)` → `template_from_descriptor` → cosigner `ResolvedSlot`s from `expand_per_at_n` (65-byte xpubs reconstructed with the `--network`-authoritative `NetworkKind`) → `build_descriptor_string` multipath `<0;1>/*`. Own seed (`--from`) + cosigner (`--cosigner @N=mk1|xpub`) are OPTIONAL per-position cross-checks (65-byte form, never `Xpub ==`); only supplied positions are marked verified (PARTIAL otherwise — a Phase-2-R1 Critical fold). `extract_multisig_threshold` bumped to `pub(crate)`. Original §11 framing (option (a) / supply-keys-to-build) inverted; (a) BLOCKED. Scope: wsh + sh(wsh); taproot + template-only refused. Full audit trail: `design/SPEC_restore_multisig_cosigner.md` + `design/agent-reports/restore-multisig-cosigner-{r0-r1,r0-r2,phase-2-r1,phase-2-r2}-review.md` (R0 GREEN after one fold round; Phase 2 GREEN after the C1 fold).
- **Tier:** `v0.5`
- **Tags:** `restore` `wallet` `multisig`
- **Spawned FOLLOWUPs:** `restore-multisig-taproot-reconstruction`, `restore-multisig-format-payloads`, `gui-restore-multisig-flags-pending-pin-bump`.

### `restore-multisig-taproot-reconstruction` — `mnemonic restore --md1` refuses a taproot multisig md1; BLOCKED on `toolkit-trmultia-nums-internal-key` (bundle emits a placeholder `is_nums:false` tr md1 that no reconstruction path can reproduce — see the 2026-06-05 R0-r1 correction below)

- **Surfaced:** 2026-06-05, multisig-cosigner restore cycle (toolkit v0.44.0). Scope-decision (user): defer taproot.
- **Where:** `crates/mnemonic-toolkit/src/cmd/restore.rs` `run_multisig` taproot pre-gate (`d.tree.tag == md_codec::Tag::Tr` → `ModeViolation` exit 2); root blocker in `descriptor-mnemonic/crates/md-codec/src/to_miniscript.rs` (`AddressDerivationFailed { "Tag::SortedMultiA must be a tap-leaf root child; rust-miniscript v13 has no Terminal::SortedMultiA fragment" }`).
- **What:** A wallet-policy md1 whose tree is `Tr{…, SortedMultiA{k, indices}}` is refused at the `Tag::Tr` pre-gate because restore classifies the descriptor via `md_codec::to_miniscript_descriptor(&d,0)`, which ERRORS descending into the `SortedMultiA` child (it hand-builds `Terminal` fragments and rust-miniscript v13 has no `Terminal::SortedMultiA`). **CORRECTION (2026-06-05 upstream re-check — supersedes the original "bespoke emitter" framing below):** NO bespoke tr-string emitter or hand-computed BIP-386 checksum is needed. The toolkit's OWN descriptor builder `build_descriptor_string(template, &slots, k, network, account, Some(taproot_internal_key))` (`wallet_export/pipeline.rs:18`) ALREADY constructs a valid `tr(sortedmulti_a(…))#<csum>` — runtime-proven: `export-wallet --template tr-sortedmulti-a --taproot-internal-key nums --format descriptor` emits `tr(sortedmulti_a(2,…))#caply36x`, and `build_descriptor_string` round-trips the string through `MsDescriptor::from_str` (`pipeline.rs:28`), so **rust-miniscript v13's STRING parser handles `tr(sortedmulti_a)` fine** — only md-codec's manual-`Terminal` path lacks the fragment. The md1 tree carries everything reconstruction needs: internal key from `Body::Tr { is_nums, key_index }` (`md-codec tree.rs:49` → `TaprootInternalKey::Nums`/`Cosigner(key_index)`), `k` from `extract_multisig_threshold(&d.tree)` (already recurses into `Body::Tr`, `bundle.rs:1021`), cosigner keys from `expand_per_at_n(&d)` (same call wsh uses), template (`TrSortedMultiA` vs `TrMultiA`) from the Tr child's tag. **Fix = route around md-codec for taproot:** read template + internal-key + k + slots off `d.tree` directly, then call the existing `build_descriptor_string` with `Some(internal_key)` — ~50-100 LOC of tree introspection + reuse, mirroring the v0.44.0 wsh/sh-wsh path. Single-sig restore ships bip86 (taproot single-sig), so this closes a parity asymmetry for multisig.
- **2026-06-05 R0-r1 CORRECTION (supersedes BOTH framings above — the "reuse-heavy" re-check was itself built on a FALSE premise).** A SPEC (`design/SPEC_restore_multisig_taproot.md`, branch `restore-multisig-taproot-reconstruction`) was written on the route-around + NUMS-reuse plan and **FAILED R0 (2 Critical)** — see `design/agent-reports/restore-multisig-taproot-r0-r1-review.md`. **Root error:** the re-check verified `export-wallet --taproot-internal-key nums` (which builds `tr(NUMS, sortedmulti_a)` — and emits a *descriptor/wallet-file*, never an md1), but `bundle` (the sole md1 emitter) emits a DIFFERENT shape: `wrapper_node` for `TrMultiA`/`TrSortedMultiA` hard-codes `Body::Tr { is_nums: false, key_index: 0 }` with leaf `indices:(0..n)` (`template.rs:209-215`) → the descriptor `tr(@0, sortedmulti_a(k, @0,@1,…,@n-1))` (cosigner @0 is the internal key AND in the leaf). This is a placeholder (the code comment `template.rs:203-208` says it SHOULD emit `is_nums: true`/NUMS) locked by `template.rs:446`. **Consequences:** (C1) a NUMS-only reconstruct refuses every real toolkit tr md1; (C2) `build_descriptor_string` cannot reproduce the `@0-in-both` shape under ANY `TaprootInternalKey` (`Nums`→wrong internal key; `Cosigner(0)`→drops @0 from the leaf, `pipeline.rs:135`). The slot/threshold/`is_wallet_policy`/`expand_per_at_n` reuse DOES hold (all tree-shape-agnostic); only the descriptor build is blocked.
- **Why deferred / BLOCKED-ON:** **dependency-order inversion.** The clean reuse only becomes correct AFTER the prerequisite FOLLOWUP `toolkit-trmultia-nums-internal-key` makes `bundle`/`wrapper_node` emit `is_nums: true` (NUMS) — a **change to the tr md1 wire content** (backward-compat consideration, though tr multisig is new/rare and arguably non-functional today since md-codec can't even render the current `@0-in-both` shape). Sequence: (1) `toolkit-trmultia-nums-internal-key` (fix bundle tr emit → NUMS); (2) THEN this slug = clean `build_descriptor_string(..., Some(Nums))` reuse + the `is_nums:true` classification branch. Alternatively (1') teach `build_tr_multi_a_descriptor` an "internal-key cosigner kept in leaf" mode to reconstruct the *current* shape — closer to the original bespoke fear; not recommended. md-codec pinned `0.35.0`.
- **Status:** `open` — **blocked on `toolkit-trmultia-nums-internal-key`** (R0-r1 RED; not implementable as scoped).
- **Tier:** `v0.5`
- **Tags:** `restore` `wallet` `multisig` `taproot` `blocked`

### `toolkit-trmultia-nums-internal-key` — `bundle` tr multisig (`tr-multi-a`/`tr-sortedmulti-a`) emits a placeholder cosigner-internal-key (`is_nums:false, key_index:0`) instead of NUMS

- **Surfaced:** referenced in code since v0.30 (`crates/mnemonic-toolkit/src/template.rs:206` comment) but **never formally filed in this registry until 2026-06-05** (surfaced as the blocker for `restore-multisig-taproot-reconstruction` R0-r1).
- **Where:** `crates/mnemonic-toolkit/src/template.rs:194-215` (`wrapper_node` `TrMultiA|TrSortedMultiA` arm: `Body::Tr { is_nums: false, key_index: 0 }`, leaf `indices:(0..n)`); locked by `template.rs:446` (`assert!(!is_nums, …)`); `synthesize.rs:399` (bundle's emit path consumes `wrapper_node` verbatim, no NUMS substitution); contrast `parse_descriptor.rs` `substitute_nums_sentinel` (the only `is_nums:true` setter, user-`--descriptor` intake only).
- **What:** A toolkit-bundled taproot multisig wallet uses cosigner @0 as the taproot key-path internal key (and @0 is also a script-path leaf key) — `tr(@0, sortedmulti_a(k, @0,…,@n-1))`. BIP-388 script-path-only multisig conventionally uses a provably-unspendable NUMS internal key (`tr(NUMS, sortedmulti_a(k, …))`, what `export-wallet --taproot-internal-key nums` emits). The current shape is (a) non-standard, (b) un-renderable by `md_codec::to_miniscript` (which can't build `SortedMultiA`) AND not reproducible by `build_descriptor_string` (no "internal-key cosigner kept in leaf" mode) — so toolkit tr multisig md1s are effectively non-round-trippable today. Decide + implement the NUMS internal key for `wrapper_node` tr templates (update the `template.rs:446` lock + any md1 wire fixtures); this unblocks `restore-multisig-taproot-reconstruction`.
- **Why deferred:** Surfaced as a prerequisite during the taproot-restore R0; its own cycle (wire-content change to tr md1 + fixture updates + the bundle/verify-bundle/export round-trip).
- **Status:** `open`
- **Tier:** `v0.5`
- **Tags:** `bundle` `taproot` `multisig` `wire-format` `blocks-restore-taproot`

### `restore-multisig-format-payloads` — `mnemonic restore --md1 --format <export-format>` (importable multisig wallet payloads) is refused; only the descriptor doc is emitted

- **Surfaced:** 2026-06-05, multisig-cosigner restore cycle (toolkit v0.44.0).
- **Where:** `crates/mnemonic-toolkit/src/cmd/restore.rs` `run_multisig` (`args.format.is_some()` → `ModeViolation` exit 2).
- **What:** Single-sig restore's `--format` emits an importable wallet-software payload via the `export-wallet` `WalletFormatEmitter` dispatch, but REQUIRES a single `--template` — which does not fit multisig. Multisig restore currently emits only the concrete descriptor (text / `--json` / `--output`) and refuses `--format`. A follow-on could wire multisig `--format` through the multisig-capable emitters (`coldcard-multisig`, `bsms`, `bitcoin-core`, `descriptor`, …) by building a multisig `EmitInputs` from the reconstructed cosigner slots + threshold (the data is all in hand post-reconstruction: `template`, `slots`, `k`, `taproot_internal_key`). Mirror the single-sig `build_import_payload` `collect_missing`-first contract.
- **Why deferred:** Out of v0.44.0 scope (the descriptor + cross-check is the core); additive surface with its own per-emitter multisig test matrix.
- **Status:** `resolved` mnemonic-toolkit-**v0.45.0**. `run_multisig` `--format` refusal gate removed; new `build_multisig_import_payload` builds a multisig `EmitInputs` (`threshold: Some(k)`, **`threshold_user_supplied: true`** — k from md1 is authoritative + Sparrow's `collect_missing` refuses a multisig template otherwise; `taproot_internal_key: None`; `<template>-<account>` wallet name == export-wallet's default) and runs the `collect_missing`-first → `emit` dispatch **byte-identical to `export_wallet.rs:506-560`** (incl. the coldcard-multisig 6-variant `CliTemplate` match). 9 emit (`bitcoin-core`/`bip388`/`coldcard`/`coldcard-multisig`/`jade`/`sparrow`/`electrum`/`bsms`/`descriptor`), 2 refuse (`specter` missing-wallet-name exit 2, `green` no-multisig exit 1), mirroring export-wallet exactly. Payload computed AFTER the mismatch hard-gate (exit 4 precedes any emit). Watch-only-out preserved. Test: `tests/cli_restore_multisig_format.rs` (10 cells; per-format threshold token + 3-fp containment, not byte-parity — md1 real-fp + template-mode is a provenance no export-wallet invocation reproduces). Audit trail: `design/SPEC_restore_multisig_format_payloads.md` + `design/agent-reports/restore-multisig-format-payloads-{r0-r1,r0-r2}-review.md` (R0 GREEN after one fold round).
- **Tier:** `v0.5`
- **Tags:** `restore` `wallet` `multisig` `export`
- **Spawned FOLLOWUPs:** `restore-emit-dispatch-3way-dedup`.

### `restore-emit-dispatch-3way-dedup` — the 11-arm `collect_missing`→`emit` `WalletFormatEmitter` dispatch now exists in 3 byte-identical copies

- **Surfaced:** 2026-06-05, multisig restore `--format` cycle (toolkit v0.45.0) — R0 + advisor noted the duplication.
- **Where:** `crates/mnemonic-toolkit/src/cmd/export_wallet.rs:506-560` (`run`), `crates/mnemonic-toolkit/src/cmd/restore.rs` single-sig `build_import_payload` (~`:587-660`), and the v0.45.0 multisig `build_multisig_import_payload` (~`:662-760`).
- **What:** Three byte-identical copies of the `match format { … collect_missing … }` + refuse + `match format { … emit … }` dispatch (incl. the coldcard-multisig 6-variant template branch). Consolidate into one `wallet_export::emit_payload(inputs: &EmitInputs, format: CliExportFormat) -> Result<String>` (collect_missing-first → emit, with the coldcard-multisig branch) consumed by all three sites. Same species as `descriptor-origin-extraction-dedup`. The copies were kept byte-identical deliberately so the consolidation is mechanical.
- **Why deferred:** The de-dup touches two shipped/tested paths (`export-wallet` `run` + single-sig restore) and would change single-sig restore's coldcard-multisig refusal message — refactoring unrelated to the v0.45.0 goal (the advisor explicitly recommended 3rd-copy-plus-FOLLOWUP over the refactor). Drift window is "copies are byte-identical today."
- **Status:** `resolved` toolkit-**v0.46.1** — extracted `pub(crate) fn emit_payload(&EmitInputs, CliExportFormat) -> Result<String, ToolkitError>` into `cmd/export_wallet.rs`, consumed by all dispatch sites. **Count correction:** this entry said "3-way / 3 byte-identical copies" but recon found **FOUR** copies — the uncited 4th was `export_wallet.rs::run_from_import_json` — and they were NOT all byte-identical: the single-sig restore `build_import_payload` coldcard-multisig arm diverged (old "requires a multisig wallet; restore is single-sig" string). Decision (a): single-sig restore now routes through the helper's unified 6-variant `_ =>` refusal ("requires a multisig --template …"), exit 1 unchanged (one user-visible wording change, pinned by a new `cli_restore.rs` cell). Net −124 LOC. No CLI-surface change → no GUI/manual/sibling lockstep. Audit trail: `design/SPEC_restore_emit_dispatch_dedup.md` + `design/agent-reports/restore-emit-dispatch-dedup-r0-round{1,2}-review.md` + `…-phase-2-review.md`.
- **Tier:** `v0.5`
- **Tags:** `restore` `export-wallet` `refactor` `dedup`

### `gui-restore-multisig-flags-pending-pin-bump` — mnemonic-gui `RESTORE_FLAGS` must add `--md1`/`--cosigner` + flip `--from required:false`, blocked on the GUI bumping its toolkit pin to ≥ v0.44.0

- **Surfaced:** 2026-06-05, multisig-cosigner restore cycle (toolkit v0.44.0) — the paired GUI schema-mirror half.
- **Where:** `mnemonic-gui/src/schema/mnemonic.rs` `RESTORE_FLAGS` (`:355`): add `FlagSchema` entries for `--md1` (repeating, `secret:false`) and `--cosigner` (repeating, `secret:false`), and flip the existing `--from` entry `required: true` → `required: false` (toolkit `RestoreArgs.from` is now `Option<String>` with `required_unless_present = "md1"`).
- **What:** The v0.44.0 cycle added `--md1`/`--cosigner` to the toolkit's `restore` clap surface. The paired GUI `schema_mirror` update is **intentionally unmergeable until the GUI bumps its toolkit pin to ≥ v0.44.0**: `schema_mirror` runs `mnemonic gui-schema` against the PINNED toolkit binary, so a `RESTORE_FLAGS` that lists `--md1`/`--cosigner` against an older pinned binary (without those flags) FAILS the gate (the "schema-ahead-of-pins" class). `schema_mirror` is flag-NAME-only, so the `--from required` flip is NOT gate-caught — the GUI would mis-render `--from` as mandatory until the prose flip lands; the leading discipline is this paired-PR record. When the GUI next bumps its toolkit pin (≥ v0.44.0), land the `RESTORE_FLAGS` delta in the same PR. Mirrors the resolved precedent `gui-ms1-slot-subkey-pending-pin-bump`.
- **Why deferred:** Cross-repo; the GUI toolkit-pin bump is its own GUI cycle (a `schema_mirror`-ahead-of-pin change cannot ship in isolation).
- **Status:** `resolved` mnemonic-gui-**v0.25.0** (`a9abac2`, tag `mnemonic-gui-v0.25.0`). `RESTORE_FLAGS` += `--md1`/`--cosigner` (both `Text`, `repeating`, `secret:false`); `--from` flipped `required:true → false`. The `required_unless_present="md1"` semantic is modeled as a GUI-authored at-least-one rule `conditional::restore` (`--from` Required while `--md1` empty) — NOT just flat-false — because the toolkit `gui-schema` `conditional_rules` projection is a hand-encoded allowlist (`gui_schema.rs:336-345`) with no `restore` arm, so restore emits `conditional_rules: []`; the rule is GUI-authored/ungated, same posture as `repair`/`inspect` (spawned FOLLOWUP `gui-schema-restore-required-unless-md1-projection`). Toolkit pin bumped v0.43.0 → v0.44.0 (`Cargo.toml` + `pinned-upstream.toml` + `Cargo.lock`, `pin_coherence` lockstep). `schema_mirror` + `conditional_visibility` + `gui_schema_conditional_drift` + full suite green (4 pinned bins). Audit trail: `mnemonic-gui/design/SPEC_gui_v0_25_0_restore_multisig_flags.md` + `design/agent-reports/gui-v0_25_0-restore-multisig-{R0,impl}-review.md` (R0 GREEN first round; impl review GREEN).
- **Tier:** `cross-repo`
- **Tags:** `restore` `multisig` `gui` `schema-mirror`
- **Spawned FOLLOWUPs:** `gui-schema-restore-required-unless-md1-projection`, `gui-readme-install-pin-coherence-guard`.

### `gui-schema-restore-required-unless-md1-projection` — toolkit `gui-schema` `conditional_rules` projection omits restore's `--from required_unless_present="md1"` (the GUI rule is GUI-authored/ungated)

- **Surfaced:** 2026-06-05, mnemonic-gui v0.25.0 cycle (R0 + impl review).
- **Where:** `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:336-345` (`build_subcommand_conditional_rules` — hand-encoded `match name` allowlist with arms only for bundle/verify-bundle/export-wallet/convert/derive-child/compare-cost; restore → `_ => Vec::new()`).
- **What:** Toolkit `restore` carries `#[arg(long, required_unless_present = "md1")]` on `--from` (`cmd/restore.rs:60`), but the `gui-schema` `conditional_rules` projection has no restore arm, so it emits `conditional_rules: []` for restore. The GUI therefore cannot mirror the at-least-one constraint via the normal drift-gated mechanism (`mnemonic-gui/tests/gui_schema_conditional_drift.rs` skips empty-rule subcommands); mnemonic-gui v0.25.0 modeled it as a **GUI-authored** rule `conditional::restore` (un-gated, same posture as the GUI's `repair`/`inspect` at-least-one rules). Promote: add a `restore` arm to `build_subcommand_conditional_rules` emitting `--from required_unless_present=[--md1]`, so the GUI rule becomes drift-gated. Requires a toolkit release + a GUI pin bump to consume it (then the GUI rule is enforced by `gui_schema_conditional_drift`).
- **Why deferred:** Out of scope for the GUI-only v0.25.0 catch-up (a toolkit `gui-schema` change cannot ship in a GUI cycle); the GUI-authored rule is correct + faithful in the meantime.
- **Status:** `resolved` — **toolkit half** toolkit-**v0.46.2** (`b74badd`): `restore_conditional_rules()` + `build_subcommand_conditional_rules` `"restore"` arm; `mnemonic gui-schema` projects `restore`'s `--from required-unless-md1` as `not(flag_present "--md1") → {--from, required}` (mirroring bundle's `--template` precedent; `EXPECTED_ARM_COUNT 6→7`). **GUI consumption half** mnemonic-gui-**v0.27.0** (`4b83a9f`): toolkit pin v0.46.0→v0.46.2 activates `gui_schema_conditional_drift` for restore (the `conditional::restore` fn already matched — drift GREEN); `("restore", 1)` added to `SUBCOMMAND_FLOORS` (total 34→35); 3 now-stale "no restore arm/emits []/not drift-gated" comments rewritten. The GUI rule is now toolkit-projected + drift-gated. Audit trail: toolkit `design/SPEC_gui_schema_restore_conditional_projection.md` + `design/agent-reports/gui-schema-restore-conditional-projection-*`; GUI `design/SPEC_gui_v0_27_0_restore_conditional_consume_readme_guard.md` + `design/agent-reports/gui-v0_27_0-restore-conditional-*` (R0 GREEN over 3 rounds).
- **Tier:** `cross-repo`
- **Tags:** `restore` `multisig` `gui` `gui-schema` `conditional-rules`

### `gui-readme-install-pin-coherence-guard` — mnemonic-gui `README.md` install-command pins drift silently (no version-marker guard)

- **Surfaced:** 2026-06-05, mnemonic-gui v0.25.0 cycle (impl review Minor).
- **Where:** `mnemonic-gui/README.md` install-command block (the `cargo install --tag …` lines) — the self-tag + the toolkit pin had drifted to `mnemonic-gui-v0.22.0` / `mnemonic-toolkit-v0.41.0` (stale since v0.23.0) while the README claims "pinned tags match `pinned-upstream.toml`". Backfilled to v0.25.0 / v0.44.0 in the v0.25.0 cycle.
- **What:** Unlike the toolkit repo (which has `tests/readme_version_current.rs`), mnemonic-gui has NO guard asserting the README's `--tag` lines match `pinned-upstream.toml` + the crate version, so they drift silently between releases (3 versions, here). Add a small `mnemonic-gui/tests/readme_pin_coherence.rs` (pure-logic) asserting each README `cargo install --tag <X>` line equals the corresponding `pinned-upstream.toml` tag (+ the GUI self-tag equals `Cargo.toml` version), mirroring the toolkit's `readme_version_current` + the GUI's existing `pin_coherence` guard.
- **Why deferred:** Out of scope for the v0.25.0 schema catch-up; the instance was backfilled, the class (the guard) is the follow-on.
- **Status:** `resolved` mnemonic-gui-**v0.27.0** (`4b83a9f`). New pure-logic `mnemonic-gui/tests/readme_pin_coherence.rs` asserts each README `cargo install … --tag <X>` line equals its source of truth — the GUI self-tag vs `Cargo.toml` `version`, the four sibling tags (`mnemonic-toolkit`/`md-cli`/`ms-cli`/`mk-cli`) vs `pinned-upstream.toml` `[mnemonic|md|ms|mk].tag`. Whitespace-tolerant (`split_whitespace`); proven non-vacuous (RED on a deliberate pin desync). Mirrors `pin_coherence.rs` + the toolkit's `readme_version_current.rs`. Shipped alongside the restore-projection consumption half in the same GUI cycle.
- **Tier:** `cross-repo`
- **Tags:** `gui` `readme` `pin-coherence` `test-guard`

### `verify-bundle-descriptor-entropy-slot-gap` — `verify_bundle` descriptor binding loop has no `@N.entropy=` arm; raw-entropy cosigners in descriptor verify-bundle mode fall to `DescriptorReparseFailed`

- **Surfaced:** 2026-06-03, ms1-slot cycle (toolkit v0.41.0) SPEC-R0-I1. Out of scope for the ms1-slot cycle; pre-existing.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` descriptor binding loop (the `if subkeys.contains(Phrase) || Seedqr { … } else if Xpub { … } else if Ms1 { … } else { return DescriptorReparseFailed }` chain, ~`:788-890`).
- **What:** The descriptor-mode binding loop in `verify_bundle` has arms for `Phrase`/`Seedqr`, `Xpub`, and (as of v0.41.0) `Ms1`, but NO `SlotSubkey::Entropy` arm. A `verify-bundle --descriptor <BIP-388 template> --slot @N.entropy=<hex>` invocation therefore falls through to the catch-all `else → DescriptorReparseFailed` (exit 2 with a "subkey set not supported in descriptor verify-bundle path" detail) rather than deriving the cosigner xpub from the raw entropy. By contrast, the `bundle` descriptor loop (`bundle_run_unified_descriptor`) DOES have an `Entropy` arm, and the `verify_bundle` TEMPLATE path resolves `@N.entropy=` via the shared `resolve_slots`. So the gap is specifically: raw-`entropy` cosigner + `verify-bundle` + `--descriptor` mode.
- **Why deferred:** The ms1-slot cycle SPEC explicitly scoped its descriptor verify-bundle work to the new `Ms1` arm (SPEC-R0-I1: "this loop has NO Entropy arm to mirror — do NOT add one"). Adding a parallel `Entropy` arm is a separable enhancement (mirror the bundle-loop `Entropy` arm into the verify-bundle descriptor loop, deriving via `derive_slot::derive_bip32_from_entropy_at_path` at `anno_path`). No SPEC, no test coverage, and no user request yet — filed for visibility.
- **Status:** `resolved` mnemonic-toolkit-**v0.43.1**. Added the `else if subkeys.contains(&SlotSubkey::Entropy)` arm to the `verify_bundle.rs` descriptor binding loop (between the `Xpub` and `Ms1` arms), mirroring the `bundle` Entropy arm's behavior: hex-decode → derive at `anno_path` (routing through the shared `derive_slot::derive_bip32_from_entropy_at_path`, the step-for-step equivalent of the bundle arm's inline derivation), `emit_lang=None`. New `tests/cli_verify_bundle_entropy_slot.rs` (5 round-trip/mismatch tests). Behavior-only fix (no clap surface change → no GUI/manual lockstep). NB: the catch-all error is **exit 4** (`DescriptorReparseFailed`), not exit 2 as this entry originally stated (stale citation — runtime-verified at fix time). Full audit trail: `design/SPEC_verify_bundle_entropy_slot.md` + `design/agent-reports/verify-bundle-entropy-slot-{r0-r1,r0-r2,phase-2-r1}-review.md` (R0 GREEN 0C/0I after one fold; Phase 2 GREEN 0C/0I).
- **Tier:** `v0.4.4-nice-to-have`

### `gui-ms1-slot-subkey-pending-pin-bump` — mnemonic-gui `Ms1` slot-editor picker + `SECRET_SLOT_SUBKEYS` snapshot are prepared but block on the GUI bumping its toolkit pin to ≥ v0.41.0

- **Surfaced:** 2026-06-03, ms1-slot cycle (toolkit v0.41.0) end-of-cycle R0.
- **Where:** `mnemonic-gui` branch `bundle-slot-ms1-gui` (local, commit `d04bad9`, NOT pushed/merged): `src/form/slot_editor.rs::SlotSubkey` (add `Ms1` picker variant + `ALL`/`as_str`/`is_secret_bearing`) + `src/secrets.rs` `v0_3_canonical_fallback::SECRET_SLOT_SUBKEYS` snapshot (`["phrase","seedqr","entropy","ms1","xprv","wif"]`).
- **What:** The toolkit v0.41.0 cycle added the `ms1` `--slot` subkey (secret-bearing). The paired GUI update (slot-editor picker + secret-redaction snapshot) is PREPARED on the branch above but is **intentionally unmergeable in isolation**: `mnemonic-gui/src/secrets.rs` re-exports `SECRET_SLOT_SUBKEYS` as a compile-time `const` from the toolkit crate, PINNED at `mnemonic-toolkit-v0.37.3` (`mnemonic-gui/Cargo.toml:42` — 5 entries, no `ms1`), and a `const _: () = assert!(secret_slice_eq(<re-export>, <snapshot>))` guard fires (E0080) when the 6-entry snapshot diverges from the 5-entry re-export. This guard correctly PREVENTS shipping a non-redacting `Ms1` picker (picking `ms1` in the GUI without the const containing `"ms1"` would leak the secret value past `persistence.rs:91` redaction). So the picker + snapshot MUST land together with a toolkit-pin bump to ≥ v0.41.0.
- **Why deferred:** The GUI toolkit pin is very stale (v0.37.3 vs current v0.41.0 — ~5 minor versions); bumping it pulls the full intervening surface (mnemonic addresses / silent-payment / nostr / K-of-N ms-shares / ms1-slot) and is its own GUI cycle (also needs the K-of-N `ms-shares` schema-mirror from the v0.40.0 cycle). The `schema_mirror` gate does NOT cover this (ms1 is a free-form `--slot` value, not a clap flag/value-enum), so there is no leading auto-gate — the leading discipline is this paired-PR record. When the GUI next bumps its toolkit pin, land the `bundle-slot-ms1-gui` draft (picker + snapshot) in the same PR.
- **Status:** `resolved` — mnemonic-gui **v0.22.0** (`0078505`, tag `mnemonic-gui-v0.22.0`). Bumped the GUI's Cargo toolkit lib pin v0.37.3 → v0.41.0 (+ all 4 `pinned-upstream.toml` tags to current) and landed the `bundle-slot-ms1-gui` draft (slot-editor `Ms1` picker + `SECRET_SLOT_SUBKEYS` snapshot += `ms1`); the const-assert now compiles (6-entry re-export == snapshot; `SECRET_NODE_TYPES` unchanged). Also added the `md repair` schema entry + a `tests/pin_coherence.rs` guard (Cargo pin == pinned-upstream `[mnemonic].tag`) closing the "schema-ahead-of-pins, masked by local-binary run" bug class, and RESTORED `schema_mirror` green (the pins had lagged the schemas since GUI v0.21.3). Full audit trail in `mnemonic-gui/design/{SPEC_gui_v0_22_0_pin_catchup_ms1.md,IMPLEMENTATION_PLAN_gui_v0_22_0.md,agent-reports/gui-ms1-*}`.
- **Tier:** `cross-repo`

### `ms-kofn-json-wire-shape-ungated` — `mnemonic ms-shares` (+ sibling `ms split`/`combine`/`inspect`-share) `--json` wire-shapes + the `--to` value-enum are NOT schema_mirror-gated

- **Surfaced:** 2026-06-03, ms K-of-N v0.2 cycle Phase 4 (Task 4.2c) — mnemonic-toolkit v0.40.0 / ms-codec 0.4.0 / ms-cli v0.7.0.
- **Where:** `crates/mnemonic-toolkit/src/cmd/ms_shares.rs` (`split`/`combine` `--json` emit). Sibling consumers: `mnemonic-secret/crates/ms-cli/src/cmd/{split.rs,combine.rs,inspect.rs}` (`--json` emit). GUI mirror `mnemonic-gui/src/schema/{mnemonic.rs,ms.rs}` (consumes ONLY the flag-name + per-flag `secret` projection).
- **What:** The K-of-N surface adds `--json` output objects GUI consumers may parse: `mnemonic ms-shares split --json` → `{ "shares": [...] }`; `mnemonic ms-shares combine --json` → the recovered-secret object; and the ms-cli siblings `ms split --json` → `{ shares, k, n, id, kind, language? }`, `ms combine --json`, `ms inspect --json` of a share → `{ kind: "share", threshold, id, index }`. The `schema_mirror` gate enforces ONLY clap **flag-NAME** parity (+ the per-flag `secret` projection); it does NOT gate the runtime `--json` **wire-shape** of any of these, nor the `combine --to` value-enum dropdown contents (`phrase|entropy|ms1`). A wire-shape key change or a new `--to` value trips NO automated gate — it accumulates silently until a GUI consumer mis-parses (the lagging-indicator class documented in `CLAUDE.md`).
- **Why deferred:** Standing posture for ALL toolkit `--json` wire-shapes (`CLAUDE.md`: "Scope of the gate — clap flag-NAME parity, NOT JSON wire-shape"; the generalization is the `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` option (b), v0.30+, named in `CLAUDE.md`). Downstream self-updates via the **paired-PR rule**: any `--json` wire-shape / `--to` value-enum change to this K-of-N surface MUST land a same-cycle (or paired sibling) `mnemonic-gui` PR. This entry records the K-of-N un-gated consumers for a future wire-shape editor.
- **Companion:** `bg002h/mnemonic-secret` `design/FOLLOWUPS.md` entry `ms-kofn-json-wire-shape-ungated`.
- **Status:** `open` (standing-posture / paired-PR tracking — fires no automated gate by design).
- **Tier:** `cross-repo`

### `manual-repair-flag-mutex-inaccuracy` — repair/inspect flag-table says `--ms1/--mk1/--md1` "mutually exclusive" but source allows combining per D35

- **Surfaced:** 2026-05-24, indel-v2 (v0.37.3) end-of-cycle review (M2). PRE-EXISTING inaccuracy, not introduced this cycle. The manual's `mnemonic repair` flag table describes `--ms1`, `--mk1`, `--md1` as "mutually exclusive with" one another, but the actual CLI permits combining them (multi-group repair, one HRP per card): source doc-comments say "May be combined with … per D35" and the test `multi_group_both_emit_exit_5` proves a combined `--ms1 … --mk1 …` invocation emits both repairs at exit 5.
- **Where:** `docs/manual/src/40-cli-reference/41-mnemonic.md:2277-2279` (repair table — three rows each say "mutually exclusive with" the other two HRP flags). **Check the class:** the `inspect` table at `:2546` (and adjacent `--mk1`/`--md1` inspect rows) likely repeats the same "mutually exclusive" wording — verify whether `inspect` actually rejects multi-group before deciding to fix or leave it (inspect may genuinely be single-group). Source ground truth: `crates/mnemonic-toolkit/src/cmd/repair.rs:39,46,52` ("May be combined … per D35").
- **What:** Replace the "mutually exclusive with `--mk1` / `--md1`" clause in each repair-table row with the accurate "may be combined with `--mk1` / `--md1` (one HRP per card; per D35)" wording mirroring the source doc-comments. Audit the inspect table for the same class and fix iff inspect also allows combining; otherwise leave inspect as-is and add a one-line note that repair (not inspect) supports multi-group. No code/flag change — manual prose only; manual lint (flag-NAMES) does not gate this prose.
- **Why deferred:** Out of scope for the v0.37.3 indel cycle (the cycle touched repair behavior, not the mutex documentation); pre-existing since the multi-group repair feature shipped. Natural fold target: any future docs-refresh PATCH or a repair-surface cycle.
- **Status:** `open`
- **Tier:** `docs`

### `cross-start-convergence-remaining-cells` — finish the cross-start convergence + standalone-bijection test matrix

- **Surfaced:** 2026-05-26, v0.37.4 cross-start-convergence cycle (architect review I2). The SPEC `design/SPEC_cross_start_convergence_and_bijection_tests.md` specified 14 cells across 2 files; v0.37.4 shipped 7 (Property A: A1, A2, A1-neg, A4, A5, A6, A7). The remainder were deferred so the F3 bug fix would not be coupled to longer test-authoring.
- **Where:** `crates/mnemonic-toolkit/tests/cli_cross_start_convergence.rs` (add A8) + a new `crates/mnemonic-toolkit/tests/cli_standalone_bijections.rs` (B1–B6).
- **What:** ~~B1/B2 — `xpub → mk1 → xpub` (+ fingerprint + path reverse edges); B3 — multisig per-cosigner `xpub↔mk1`; B4/B5/B6 — `descriptor → md1 → descriptor` (canonical single-sig / non-canonical / multisig) via `md_codec::chunk::reassemble`~~ — **DONE** (post-v0.37.4: `tests/cli_standalone_bijections.rs`, B1–B6 green; the md1↔descriptor bijection is asserted in the API-supported direction `reassemble(&[&str]) → Descriptor → split(&Descriptor)` byte-identity since md_codec has no string render/parser). **REMAINING: A8** — non-canonical `wsh(andor)` descriptor ≡ BSMS wallet-file convergence.
- **A8 finding (NOT papered over):** an initial attempt found the **descriptor-mode** path and the **BSMS-import→bundle** path emit *different* `md1` (descriptor policy) AND `mk1` for the SAME concrete non-canonical `wsh(andor)` descriptor — i.e. convergence does NOT hold as constructed. BSMS *accepted* the non-canonical descriptor (no parse error); the divergence is in the canonical form. Root cause UNVERIFIED — either (a) a use-site / origin canonicalization mismatch in how the two paths represent the same concrete descriptor (a test-construction artifact, cf. the A4 `/0/*`-vs-`/<0;1>/*` lesson), or (b) a genuine non-convergence of descriptor-mode vs BSMS-import for non-canonical descriptors (a real product finding). Needs a dedicated investigation: reassemble both `md1` sets, diff the `Descriptor` trees (use-site path? origin? key order?), determine which path is canonical.
- **Why deferred:** Architect flagged A8 as the highest-risk cell; the root-cause investigation is out of scope for the test-authoring follow-on. The convergence premise is proven for canonical descriptors (A1–A7 green); A8 covers only the non-canonical-via-BSMS edge.
- **A8 resolution (v0.37.5):** the A8 divergence root cause was finding **F4** — the elided-origin default-path inference emitted `PathDecl::Divergent([p,p,p])` for identical paths while the explicit-origin/import path emitted `Shared`. F4 was fixed (collapse identical inferred paths to `Shared` in `bundle.rs` + the symmetric `verify_bundle.rs` inference); A8 now converges. Full 14-cell matrix green.
- **Status:** `resolved` mnemonic-toolkit-v0.37.5 (B1–B6 + A8 all shipped; F4 fixed)
- **Tier:** `v0.37+-test-coverage`

### `multisig-tr-bip48-script-type-3-policy` — decide whether `tr-*` + `--multisig-path-family bip48` (→ `m/48'/.../3'`) is refused or remains honored

- **Surfaced:** 2026-05-26, v0.37.4 F3 fix architect review (edge case). With the F3 fix, `tr-multi-a` / `tr-sortedmulti-a` + `--multisig-path-family bip48` now derive at `m/48'/<coin>'/<account>'/3'` (`template.bip48_script_type()` returns `Some(3)` for taproot). BIP-48 officially defines only script-types 1 (sh-wsh) and 2 (wsh); `3` for taproot is a convention, not standardized.
- **Where:** `crates/mnemonic-toolkit/src/template.rs` (`bip48_script_type`), `crates/mnemonic-toolkit/src/parse.rs` (`default_origin_path`). The `3` convention pre-exists at `synthesize.rs` (`synthesize_multisig_full`) and `cmd/xpub_search/candidate_paths.rs`.
- **What:** Decide + document whether honoring `bip48` for taproot multisig (deriving at `/3'`) is intentional (current behavior — honors an explicit flag, consistent with the pre-existing convention) or should be refused as out-of-spec. Review recommendation: document as intentional rather than reopen.
- **Why deferred:** Policy question, not a defect; current behavior is internally consistent.
- **Resolution (v0.37.6): bless + warn.** Decided to KEEP honoring the explicit flag (deriving at `m/48'/.../3'`) and emit a stderr advisory at every command that derives the `3'` path (`bundle`, `export-wallet`, `verify-bundle`), pointing to `--multisig-path-family bip87` for a standardized path. Logic centralized in the pure helper `CliTemplate::bip48_nonstandard_script_type_warning(family)` (unit-tested); integration tests in `tests/cli_tr_bip48_advisory.rs`. Stderr-only → no GUI/manual lockstep.
- **Status:** `resolved` mnemonic-toolkit-v0.37.6
- **Tier:** `v1+`

### `sparrow-from-import-json-wallet-name-preservation` — `--from-import-json --format sparrow` resets `name`/`label` to the export default instead of preserving the source name

- **Surfaced:** 2026-05-27, manual-prose-execution-gate cycle Phase-2 capture of `roundtrip-sparrow-singlesig`. The chapter-45 sparrow round-trip recipe (`45-foreign-formats.md:305-320`) implied a clean semantic round-trip but the captured `diff` is non-empty: `name`/`label` change from the fixture's `"bip84-0"` to export-wallet's default `"imported-descriptor"`. Tracked in the cycle's commit + chapter prose addendum.
- **Where:** `src/cmd/export_wallet.rs:679` (`wallet_name_resolved = args.wallet_name.clone().unwrap_or_else(|| "imported-descriptor".to_string())`) — applies to all `--from-import-json` paths. The envelope's source-provenance carries the original wallet name in `provenance.source_metadata` (sparrow parser populates from `name` field), but `run_from_import_json` doesn't lift it into `wallet_name_resolved` when `--wallet-name` is absent.
- **What:** When `--wallet-name` is absent on `--from-import-json`, prefer the envelope's source-provenance wallet-name (if any) over the static `imported-descriptor` default. Apply to sparrow (round-trip impact); consider whether specter/jade/coldcard should follow (specter's recipe at `:404-411` passes `--wallet-name` explicitly to demonstrate this very thing — could lift name there too). Behavior-change MINOR (round-trip output shifts); own R0; no flag change → no GUI lockstep.
- **Why deferred:** out of scope for the manual-prose-gate cycle (which is gate-establishment + transcript coverage); fix surfaces as a separate behavior change.
- **Status:** `resolved` (v0.37.8 — 2026-05-28)
- **Tier:** `v0.37+-cli-fix`
- **Resolution (2026-05-28):** Shipped as v0.37.8 universal source-name lift. Brainstorm broadened scope from sparrow-only to all 6 name-carrying formats (sparrow / specter / jade / electrum / bitcoin-core / coldcard-multisig — coldcard singlesig has no `name` field) per "fix the class, not the instance" feedback. Extended `ImportJsonEnvelope` with 6 optional per-format `*_source_metadata: Option<serde_json::Value>` carry-fields + a `resolved_wallet_name()` accessor using a `walk_str(&Value, &[&str])` helper for the jade nested `coldcard_compat.name` case. Added `ImportProvenance::coldcard_multisig_source_metadata()` + corresponding emit block in `cmd/import_wallet.rs:1779-1807` (the only previously-unemitted name-carrying format). Renamed `EmitInputs.wallet_name_was_user_supplied` → `wallet_name_is_non_default` to cover both explicit `--wallet-name` and envelope-lifted cases (Specter's `MissingField::WalletName` refusal now dissolves on lifted names too). Use-site at `cmd/export_wallet.rs:693-696` flows the lifted name through. Test matrix: 8 unit cells + 8 integration cells (`tests/cli_export_wallet_universal_name_lift.rs`); pre-existing `p11c_refusal_matrix_specter_no_wallet_name` narrowed from `[bsms, coldcard-multisig]` to `[bsms]` (BSMS BIP-129 wire shape has no wallet-name field; the other 5 now lift). Sparrow + coldcard-multisig chapter-45 transcripts re-captured; prose addendum updated. Plan-doc R0→R1 GREEN (3C/4I/4M → 0C/0I); persisted at `design/agent-reports/sparrow-name-universal-lift-R{0,1}-review.md`.

### `manual-anchor-dangler-backlog-cleanup` — manual has ~174 real intra-doc `#anchor` danglers; enable lychee `--include-fragments` on build output once cleaned

- **Surfaced:** 2026-05-27, manual-prose-execution-gate cycle Phase-1 pre-flight (per R0 I4 fold of `SPEC_manual_prose_execution_gate.md`).
- **Where:** `docs/manual/build/m-format-manual.md` (post-`make md` concatenated single document); empirical run: `lychee --offline --include-fragments --no-progress build/m-format-manual.md` → **174 errors / 603 unique anchor references**. Per-source-file run (`lychee … src/`) gives 97 errors but mostly cross-file false positives — the build-output approach is the right architectural locus.
- **What:** Three failure classes identified in the pre-flight enumeration:
  1. **URL-encoded spaces** (e.g. `#welcome-to-the-m-format%20constellation` ×6) — markdown writer encoded a space in the link, but pandoc's slug uses a hyphen. Mechanical fix: replace `%20` with `-` in offending links.
  2. **Missing `worked-example-*` definitions** (~10 referenced, no matching `{#worked-example-*}` anchor in any source) — references to anchors that were never created.
  3. **Slug-rule mismatches** (e.g. `#when-to-use-bip-85-vs.-multisig`, `#xprv-xpub`) — link author's slug-guess differs from pandoc's actual slugification (handling of `.`, `/`, multiple hyphens).
- **Approach (pre-designed):** Run lychee against the **build output** (`build/m-format-manual.md`), not the per-file `src/` (lychee per-file can't see cross-file pandoc concat). Add a new lint stage that depends on `make md`, OR add a separate `make anchor-check` target wired into `make audit`. Once the 174 danglers are fixed (or an honest baseline-snapshot mechanism is built), enable lychee `--include-fragments` (anchor-only enum default at v0.24.2; the toolchain version is already pinned).
- **Why deferred:** 174 cross-cutting prose edits is well beyond this cycle's scope; an `--exclude` list of 174 items would silence the check; the build-output integration adds Makefile dependency complexity. Own cycle with the empirical taxonomy already in hand.
- **Status:** `resolved` (2026-05-28; CI/docs-only, no version bump)
- **Tier:** `v0.37+-docs-hygiene`
- **Resolution (2026-05-28):** Shipped 3 pieces in one cycle. **Piece 1 (architectural — single biggest dissolver):** the pre-cycle `make md` target emits pandoc-GFM which STRIPS explicit `{#id}` heading anchors (verified empirically: `pandoc --to gfm` discards `## Heading {#my-id}` anchors; `pandoc --to html` preserves them as `<h2 id="my-id">`). New `make html` target emits `build/m-format-manual.html` using `$(MD_FILTER_ARGS)` (strip-latex + primer-box, mirrors `make md`); new `anchor-check` Makefile target wires lychee against the HTML output; `audit` umbrella extended to include `anchor-check`. This single fix dissolved **162 of the 174 original errors** (174 → 12 lychee errors against HTML), because lychee on HTML reads `id="..."` directly — side-stepping all pandoc-vs-lychee slug-rule disagreement AND recovering the 15 src/ explicit `{#id}` anchors AND dissolving the ~9 worked-example-* TOC slug-rule mismatches (re-classified at R0 C3 fold). **Piece 2 (mechanical — authoring slug-guess errors):** 8 references across 3 src/ files (`60-appendices/69-index-table.md:15,26,28,29,30,32` + `10-foundations/11-welcome.md:94` + `50-comparing/51-format-decision.md:59`) had literal-space-inside-link-target that pandoc URL-encoded to `%20`; corrected via targeted sed. Slug-2 also dropped "constellation" interpolation (real heading at `50-comparing/54-mformat-vs-others.md:1`). **Piece 3 (baseline-snapshot — residual 7):** post-Pieces-1+2 the residual is **7 unique slugs** (`anchor-dangler-baseline.txt`), all author-side (version-suffix `.` mangling `v0.25.0`→`v0250`, heading-rename-without-link-update). New `tests/anchor-check.sh` script enforces baseline EXACTLY: new dangler → `::error::` exit 1; baseline shrunk without same-PR ratchet → `::error::` exit 1 (R0 I4 fold: hardened from warning to enforced error). Trial-runs verified: clean = exit 0 ("OK anchor-check: 7 danglers match baseline"); synthetic-NEW (inject `#fake-test-slug`) = exit 1 with named annotation; synthetic-SHRUNK (add phantom slug to baseline) = exit 1 with ratchet-instruction annotation. **Per-tier counts:** 174 pre-cycle → 162 dissolved by Piece 1 + 8 fixed by Piece 2 (with overlap from same root architectural issue) → 7 baseline. Full `make audit` green (lint + verify-examples 20/20 + anchor-check). Spec: `design/SPEC_manual_anchor_dangler_cleanup.md`. Reviews: R0 RED 3C/4I/3M → R1 RED 0C/2I/2M → R2 GREEN 0C/0I (`design/agent-reports/manual-anchor-dangler-R{0,1,2}-review.md`).

### `manual-yml-sibling-pin-vs-install-sh-drift-gate` — add static gate that `manual.yml` sibling-binary install pins match `scripts/install.sh`

- **Surfaced:** 2026-05-27, manual-prose-execution-gate cycle Piece 2 (defense-in-depth complement to the closed parent FOLLOWUP `manual-yml-and-install-sh-sibling-gui-pin-staleness`).
- **Where:** `.github/workflows/manual.yml:72-88` (cargo-install steps for `mk-cli@<tag>`, `md-cli@<tag>`, `ms-cli@<tag>`) ↔ `scripts/install.sh:35,38,41` (canonical sibling pins).
- **What:** The parent FOLLOWUP fixed the symptom (sibling pins were stale) v0.36.4; today they match. But no static gate enforces this — they can drift again. Mirror the pattern of `install-pin-check.yml` (which gates the toolkit self-pin against the tag): add a CI step that asserts `manual.yml`'s `mk-cli@`/`md-cli@`/`ms-cli@` tags equal `install.sh`'s corresponding pins. Per `feedback_fix_the_class_hunt_for_second_instance.md`: prefer a gate over hand-fixed instances.
- **Why deferred:** out of scope for this cycle (which is prose-gate + transcripts + harness hardening). Trivial follow-on once tackled.
- **Status:** `resolved` (2026-05-28; CI-only, no version bump)
- **Tier:** `v0.37+-ci-hygiene`
- **Resolution (2026-05-28):** Shipped `.github/workflows/sibling-pin-check.yml` — fires on every push + PR + manual dispatch. Single bash step parses `scripts/install.sh`'s `component_info` arms into a dynamic `pkg→tag` table (excludes toolkit self-pin, already gated by `install-pin-check.yml`), then scans every `.github/workflows/*.yml` for `cargo install --git ... --tag <tag> <pkg>` lines and asserts each `<tag>` matches the canonical pin keyed on exact `<pkg>` match. Drift produces `::error::sibling-pin-check: <file>:<line>: <pkg> pin '<actual>' does not match scripts/install.sh canonical '<canonical>'` and exits 1; unknown sibling (forward-compat) produces `::warning::` and continues. Scope explicitly excludes GHA tool-dep pins (`actions/*`, `dtolnay/rust-toolchain`, `lychee-v*`, `markdownlint-cli2@*`) and the structural mk-cli-only quickstart mock (`MD_BIN=true MS_BIN=true`). Trial-runs verified: clean tree exits 0 with 4 OK lines (manual.yml mk-cli/md-cli/ms-cli + quickstart.yml mk-cli); synthetic drift (sed-edit manual.yml mk-cli pin) produces the expected `::error::` + exit 1. Actionlint clean. Spec R0 RED 0C/0I/4M → folded M1+M2+M3 inline; M4 (CHANGELOG `[Unreleased]` disposition) resolved at impl time: project doesn't maintain `[Unreleased]` block so this lands without a CHANGELOG entry (mirrors manual-prose-execution-gate precedent — CI-only, no bump). Spec: `design/SPEC_sibling_pin_drift_gate.md`; R0: `design/agent-reports/sibling-pin-drift-gate-R0-review.md`.

### `path-raw-bracketed-vs-bare-convention-unification` — `ResolvedSlot.path_raw` is overloaded (bracketed `[fp/path]` from envelope path vs bare from `resolve_slots`); unify

- **Surfaced:** 2026-05-27, v0.37.7 F5 fix R0 review (M1). Root cause of F5 (export-wallet --from-import-json corrupting Coldcard/Jade/Electrum derivations) is `mk1_card_to_resolved_slot` (`wallet_import/json_envelope.rs:282`) populating `ResolvedSlot.path_raw` as a bracketed `[fp/path]` origin-annotation, while `resolve_slots` (`cmd/bundle.rs:547,628`) populates it as a bare derivation path. F5 was fixed at the export-wallet from-import-json BOUNDARY (normalize to `format!("m/{}", s.path)`) to avoid rippling into `bundle --import-json`, but the underlying overload remains.
- **Latent cosmetic bug uncovered by R0:** `bundle --import-json --json` emits a polluted `bundle.multisig.cosigners[].origin_path` field (`bundle.rs:767,1000-1004`) of shape `"m/[fp/path]"` (bracket inside the path string). Not asserted by any test; not blocking.
- **What:** Unify the convention: either (a) source-fix `mk1_card_to_resolved_slot` to produce bare `m/path` and update bracket-consumers (`import_wallet.rs::origin_path_from_bracket` callers, `coldcard_multisig.rs:658`) accordingly, OR (b) introduce a typed wrapper that makes the convention explicit. Approach (a) is simpler and additionally fixes the bundle cosmetic pollution.
- **Why deferred:** F5 boundary fix is correct + verified ripple-free; source-fix is broader scope. Track for a future cleanup cycle.
- **Status:** `resolved` (v0.37.9 — 2026-05-29)
- **Tier:** `v0.37+-refactor`
- **Resolution (2026-05-29):** Shipped as v0.37.9 via **option (b)-strong — DELETE the field** (not option (a)). Two opus architect consults + grep-verification established that `ResolvedSlot.path_raw` is pure denormalization of the typed `fingerprint` + `path` (binary-private → no SemVer cost; the "round-trip fidelity" intent was stale — distinctness reversed to typed `.path` in v0.5 and no consumer reads source bytes). Deleted the field; added `impl ResolvedSlot` methods `origin_path_bare()` (→ `m/...` or `""` for the default/WIF slot) + `bracketed_origin()` (→ `[fp/comps]` from the typed fields, byte-identical to the former producers modulo fp-lowercasing). **Fix-the-class:** the bracketed convention had ~8 producers (every foreign-format parser + the mk1 re-decode), not the one the entry cites — all dropped the field; the 6 single-sig `build_slot_fields` tuples collapsed 4→3, the bundle/verify_bundle descriptor-mode 5-tuples → 4. Cosmetic bug fixed (`bundle --import-json --json` cosigner `origin_path` now bare; T1/T2). F5 band-aid (`export_wallet.rs`) removed structurally. `coldcard_multisig`'s LOCAL `ResolvedCosigner.path_raw` (descriptor-key builder) intentionally kept — different type, legitimate bracketed use. Amendments folded at R0/R1: A1 (error-text bracketed→bare, display-only), A2 (`check_resolved_slots_distinctness` → typed `.path`, converging with the descriptor-mode twin; new collision test T6), A3 (`bundle --json` canonicalizes non-canonical `--slot @N.path=`; T9), A4 (same on `export-wallet`; T10). SPEC `design/SPEC_path_raw_bracketed_bare_unification.md` (R0 RED 2C/4I/4M → R1 0C/1I/2M → R2 GREEN); end-of-cycle R0 GREEN 0C/0I; reviews at `design/agent-reports/path-raw-unification-*`. Full suite 2482/0; clippy clean; `make audit` green (2 manual transcripts re-captured for the bare-origin rendering). No GUI/manual flag lockstep (binary-private, no clap change). Companion latent cosmetic bug in the entry body is the resolved item.

### `pr-26-roundtrip-warning-suppression` — surface canonicalize / UTF-8 errors instead of swallowing them in `emit_roundtrip_stderr_warning` + JSON envelope

- **Surfaced:** 2026-05-19, post-merge comprehensive review of PR #26 (see `design/agent-reports/pr-26-post-merge-comprehensive-review.md` — C1 + I7). The SPEC §7.4 stderr warning is the only non-JSON-mode feedback that a Bitcoin Core blob isn't round-tripping byte-exactly; if `canonicalize_bitcoin_core` errors (parser / canonicalizer disagreement, non-UTF-8 input, internal serde mismatch) the function returns `Ok(())` with no diagnostic. JSON mode drops the error reason via `.ok()` → envelope shows surface `"canonicalize_failed"` only.
- **Where:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:471-478` (two `Err(_) => return Ok(())` arms); `cmd/import_wallet.rs:334-338, 396-402` (`canonicalize_*.ok()` in JSON-mode roundtrip emit).
- **What:** (a) Replace `Err(_) => return Ok(())` with `Err(e) => { writeln!(stderr, "warning: import-wallet: roundtrip check skipped: canonicalize_bitcoin_core failed: {e}")?; Ok(()) }`. For the UTF-8 case, fold to `String::from_utf8_lossy` + explicit "blob is not UTF-8" notice. (b) In JSON mode, capture the `Err` and emit `{ "status": "canonicalize_failed", "error": "<message>" }` instead of `null` diff.
- **Why deferred:** PR #26 already merged at squash `66c8a56`; v0.26.x patch line is the natural fold target.
- **Status:** resolved — v0.27.1 Phase 1 (`84a5c70` SPEC amendment-first + `c856c8c` impl + `d3d828b` R0 M1+M2 fold). Stderr arm emits `warning: import-wallet: roundtrip check skipped: canonicalize_bitcoin_core failed: <ToolkitError>` and `notice: import-wallet: blob is not UTF-8; roundtrip check uses lossy decode`. JSON envelope's `roundtrip.canonicalize_failed` branch carries additive `error: String` field. 4 unit cells regression-guard the fold.
- **Tier:** `v0.27`

### `pr-26-shape-mismatch-silent-defaults` — distinguish "absent" from "shape-wrong" for `active`/`internal`, threshold, and origin_fingerprint

- **Surfaced:** 2026-05-19, post-merge comprehensive review of PR #26 (I4 + I5 + I6). Three sites silently substitute defaults on malformed input where they should error: (1) `active`/`internal` use `.and_then(.as_bool).unwrap_or(false)` — string `"true"`, integer `1`, etc. silently flip to `false` and downstream `--select-descriptor active-*` emits a misleading "no active-* descriptor found" error. (2) `mk1_card_to_resolved_slot` substitutes `xpub.fingerprint()` for missing `origin_fingerprint` — master-fp vs current-xpub-fp are semantically distinct; descriptor reconstruction silently produces mismatched origin annotations. (3) `extract_threshold` maps `thresh(256, …)` u8-overflow to `None` → `"threshold": null` in envelope, user sees "no-threshold" descriptor.
- **Where:** `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs:273-280` (active/internal); `wallet_import/json_envelope.rs:258-260` (fingerprint substitution); `wallet_import/bsms.rs:354-362` + `bitcoin_core.rs:455-462` (threshold).
- **What:** (1) Active/internal: distinguish absent (`None → default false`) from shape-wrong (`Some(non-bool) → ImportWalletParse`); mirror `parse_range_field`'s pattern. (2) Fingerprint substitution: emit a stderr NOTICE on fallback that names the slot index (close the `let _ = slot_idx; // reserved for future error-context attribution` self-confessed gap). (3) Threshold: return `Result<Option<u8>, ToolkitError>` distinguishing "no `thresh()` found" from "thresh argument failed u8 parse".
- **Why deferred:** PR #26 already merged; the three sites are independent of each other but share the "silent default substituting for shape-mismatch" pattern.
- **Status:** resolved — v0.27.1 Phase 2 (`62b5237` impl + `ab49d20` R0 M1+M3 fold). (I4) new `parse_bool_field` helper distinguishes absent vs shape-wrong for `active`/`internal`. (I5) `mk1_card_to_resolved_slot` emits stderr NOTICE on origin_fingerprint substitution naming slot index + downstream-mismatched-origins warning. (I6) `extract_threshold` returns `Result<Option<u8>, ToolkitError>`; u8 overflow becomes typed error. 9 new cells; 1546 → 1555 tests.
- **Tier:** `v0.27`

### `pr-26-comment-rot-fold` — citation accuracy + cycle-phase vocabulary sweep on v0.26.0 surface

- **Surfaced:** 2026-05-19, post-merge comprehensive review of PR #26 (C2 + I8 + I9 + I10 + I11). Five categories of comment-rot identified by the comment-analyzer agent: (1) module doc lists non-existent `--slot @N.ms1=` surface (SPEC §3.1 row 6 is actually `--from <node>=`); (2) unfiled FOLLOWUP slug `compare-cost-single-leaf-tr-input` cited in user-visible error + 2 comments — grep returns zero; (3) SPEC citation `§7.0.a..d` doesn't resolve (no `§7.0` header in `SPEC_wallet_import_v0_26_0.md` — leaked brainstorm shorthand); (4) `error.rs` doc comments tag variants "Phase 2/3/5 emits" — internal cycle vocabulary meaningless post-cycle; (5) user-visible error string contains `"supported in Phase 2"`.
- **Where:** `crates/mnemonic-toolkit/src/env_sentinel.rs:1-13`; `cost/strip.rs:5,51`; `cost/mod.rs:75`; `wallet_import/bsms.rs:10`; `wallet_import/bitcoin_core.rs:34`; `error.rs:181-222`.
- **What:** (1) Rewrite `env_sentinel.rs` module doc to mirror SPEC §3.1 table verbatim. (2) Either file the `compare-cost-single-leaf-tr-input` slug as a separate FOLLOWUP (preferred — preserves user-visible text) or drop the slug from both code sites. (3) Either define `## §7.0 Locks` (or similar) as a real SPEC §-anchored section, or rewrite to reference the in-prose locks directly. (4) Replace "Phase N emits" with function-anchored citations (e.g., `Emitted by import_wallet::dispatch_auto_detect_format`). (5) Drop "in Phase 2" from the user-visible `cost/mod.rs:75` error string.
- **Why deferred:** PR #26 already merged; comment hygiene is a sweep best done in one pass, not piecemeal.
- **Status:** resolved — v0.27.1 Phase 3 (`c6ce500`). C2 (env_sentinel.rs row 6 → `--from <node>=` per SPEC §3.1); I8 (slug `compare-cost-single-leaf-tr-input` filed in v0.27.0 cycle close at `53a1bf6`, cite text refreshed); **I9 wontfix** per Q3 + Q3a verification (§7.0.a..d source citations correctly reference `IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md §7.0` anchor — agent-report finding was wrong; ≥10 grep hits confirmed at Phase 0 recon); I10 (5 ImportWallet* variants now function-anchored); I11 (cost/mod.rs:75 user-visible string drops "in Phase 2"); M2 carry-over (json_envelope.rs module doc syncs with Phase 2 signature changes).
- **Tier:** `v0.27`

### `pr-26-test-coverage-gap-fold` — overlay-conflict + select-descriptor matrix + sniff/BSMS-line-count rejection cells + multisig round-trip byte-exact assertion

- **Surfaced:** 2026-05-19, post-merge comprehensive review of PR #26 (I12-I19). 8 Important test-coverage gaps identified by pr-test-analyzer: (I12) `--ms1` + `--slot @i.phrase=` conflict path untested → silent precedence-change regression risk; (I13) phrase-overlay-mismatch (`Source::Phrase` + wrong phrase) untested — only ms1 path covered; (I14) `apply_seed_overlay` non-entropy ms1 branch (e.g. `Payload::Pphr` instead of `Entr`) untested; (I15) `--select-descriptor` invalid-index / no-active-match / malformed-selector cells missing; (I16) sniff `Ambiguous` arm has no live integration test; (I17) BSMS unrecognized line-count (3/4/5/7+) only weakly covered; (I18) BSMS sniff false-positive (lowercase / leading whitespace) not pinned; (I19) multisig round-trip never asserts `roundtrip.byte_exact == true` / `semantic_match == true` — only fingerprint+count substrings.
- **Where:** `crates/mnemonic-toolkit/tests/cli_import_wallet_seed_overlay.rs` (I12-I14); `tests/cli_import_wallet_bitcoin_core.rs:107-131` (I15); `tests/cli_import_wallet_sniff.rs` (I16); `tests/cli_import_wallet_bsms.rs` (I17-I18); `tests/cli_import_wallet_roundtrip.rs:371-452` (I19).
- **What:** Add the 8 missing cells (~30-50 LOC each). pr-test-analyzer flags **I12 (overlay conflict) + I19 (multisig roundtrip byte_exact)** as the highest-ROI subset (~30 LOC total) — fold first; the remaining 6 can stage across patch releases.
- **Why deferred:** PR #26 already merged; coverage gaps are not regressions (existing behavior is correct), just absent regression guards.
- **Status:** resolved — v0.27.1 Phase 4 (`5835f92`). 14 new cells across 5 test files: I12 (overlay conflict, 1 cell); I13 (phrase-mismatch via --slot, 1 cell); I14 (ms_codec decode-Err arm — the strictly-non-entropy branch at overlay.rs:128-132 is structurally unreachable in ms-codec v0.2.0's single-Entr-variant Payload enum; cell exercises the adjacent decode-Err arm; 1 cell); I15 (--select-descriptor matrix: OOB, no-match, malformed; 3 cells); I16 (sniff Ambiguous arm dispatch shape via truth-table unit test; 1 cell — no user-constructable blob triggers this arm today); I17 (BSMS line-count rejection on 3/4/5/7-line blobs; 4 cells); I18 (BSMS sniff strictness: lowercase / leading-whitespace; 2 cells via subprocess-dispatch helper); I19 (multisig roundtrip envelope semantic_match assertion; 1 cell). 1555 → 1569 tests.
- **Tier:** `v0.27`

### `pr-26-type-design-anti-pattern-sweep` — unify SearchOutcome / ImportProvenance / BsmsVerification enums

- **Surfaced:** 2026-05-19, post-merge comprehensive review of PR #26 (I20 + I21 + 6 recurring anti-patterns identified by type-design-analyzer). The top-3 highest-ROI refactors: (1) **Unify 4 xpub-search result structs** (`PathOfXpubResult`, `PassphraseOfXpubResult`, `AccountOfDescriptorResult`, `AddressResultJson`) into `enum SearchOutcome<T> { Match(T), NoMatch }` — eliminates 8 representable-invalid states across 4 types in one edit. (2) **`ParsedImport` provenance**: replace `(Option<BsmsAuditFields>, Option<CoreSourceMetadata>)` with `provenance: ImportProvenance` enum — eliminates the both-set / both-none impossible states. (3) **`BsmsAuditFields.signature_verified: bool` → `BsmsVerification` enum** (`NotAttempted | Failed(reason) | Verified`) — parallels v0.27.0 Phase 6.5 I7's `Round1VerificationStatus` refactor; closes the future-trap before the BIP-322 inline-verifier lands.
- **Where:** `crates/mnemonic-toolkit/src/cmd/xpub_search/{path,passphrase,account,address}_of_*.rs` (refactor 1); `wallet_import/mod.rs:60` (refactor 2); `wallet_import/mod.rs:188` (refactor 3).
- **What:** Three independent refactor sub-tasks; can stage independently. The wire-shape preservation discipline: serde-flatten the new enums to preserve the existing JSON envelope shape — this becomes a Serialize-only refactor (no consumer break) if done with `#[serde(untagged)]` or `flatten`. Mirror the Phase 6.5 I7 pattern from `cmd/import_wallet.rs:835-850` for the `BsmsVerification` enum shape.
- **Why deferred:** PR #26 already merged; representable-invalid states are latent bugs not active ones (no current code path exercises the impossible combinations), so this is a type-design-cleanup pass rather than a regression fix.
- **Status:** resolved (partial) — v0.27.1 Phase 5a + 5c (`28d2203`). **Phase 5a shipped:** API-discipline scaffolding for 3 xpub-search result types (`path_of_xpub` + `passphrase_of_xpub` + `account_of_descriptor`) via private builder functions enforcing match-arm correlation at call sites; fields remain `pub` because the type-level fix (tagged enum + `#[serde(skip_serializing_if)]`) requires a wire-shape change deferred to v0.28+ → see new FOLLOWUP `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution`. **Phase 5c shipped:** `BsmsAuditFields.signature_verified: bool` → `BsmsVerification` enum (`NotAttempted | Verified | Failed { reason }`); wire shape preserved via derived getter `signature_verified()`. **Phase 5b DEFERRED:** ImportProvenance enum for `ParsedImport`'s `(Option<BsmsAuditFields>, Option<CoreSourceMetadata>)` representable-invalid pair touches 14+ sites including `apply_select_descriptor` — deferred for scope discipline → see new FOLLOWUP `pr-26-import-provenance-enum-internal-refactor`. 7 drift cells against `tests/fixtures/v0_27_0_envelopes/`. 1569 → 1576 tests.
- **Tier:** `v0.27`

### `xpub-search-result-type-level-invariant-blocked-on-wire-shape-evolution` — convert xpub-search result structs to tagged enums (v0.28+ SemVer-minor wire-shape change)

- **Surfaced:** 2026-05-19, v0.27.1 Phase 5a R1 cargo-build smoke test. The plan-doc-locked private-constructor approach (`build_path_match` etc.) is API-discipline scaffolding — it enforces the `result:"match"` ↔ `Some(payload)` correlation at internal call sites BUT direct struct-literal construction remains legal (fields are `pub`). The type-design-analyzer's I20/I21 framing was "make illegal states unrepresentable at the type level"; that requires the wire-shape change (tagged enum variant + `#[serde(skip_serializing_if = "Option::is_none")]` on the formerly-always-emitted-as-null Option fields) which v0.27.1 PATCH bump disallows.
- **Where:** `crates/mnemonic-toolkit/src/cmd/xpub_search/{path_of_xpub,passphrase_of_xpub,account_of_descriptor}.rs` (3 structs); update `tests/fixtures/v0_27_0_envelopes/` companion fixture dir per Q5c discipline (capture v0.28.0 fixtures, convert v0.27.0 cells to `#[ignore]` with SemVer rationale).
- **What:** Refactor each result struct to `enum FooResult { Match { ...required fields... }, NoMatch }` with serde-tagged enum + skip-on-no-match for the match-only fields. The wire shape changes: no-match no longer emits `"path": null, "template": null, "account": null` — those keys are omitted entirely. Consumers parsing via `Option<String>` are unaffected; raw JSON inspectors checking `.path is null` would break. Document in CHANGELOG `### Changed (wire shape)` per v0.26.0 → v0.27.0 precedent. Phase 5a's private builders become natural construction sites for the new enum variants.
- **Why deferred:** PATCH bump (v0.27.x) doesn't allow wire-shape replacement. mnemonic-gui at the consumed pin grep'd 0 external direct-literal hits (Phase 0 recon §4) so future option (c) — move the struct definitions into a private module + re-export only typed builders — is unblocked at the moment a SemVer minor bump opens the wire-shape change window.
- **Status:** `resolved 49cb211f230eb4becd773e2053bb63eb15fe07cc` — mnemonic-toolkit-v0.29.0 cycle. All 3 result types (`PathOfXpubResult`, `PassphraseOfXpubResult`, `AccountOfDescriptorResult`) converted to `#[serde(tag = "result", rename_all = "snake_case")]` tagged enums with `Match { ... }` + `NoMatch { ... }` variants. JSON wire-shape break: no-match no longer emits `"path": null, "template": null, "account": null` — keys are absent on `no_match`. Discriminator field name preserved as `"result"` (`"match"` / `"no_match"`). 3 v0.27.0 envelope drift cells (`tests/cli_xpub_search_drift_v0_27_0.rs:80, 142, 189`) marked `#[ignore]` with SemVer rationale referencing this slug. This is the SemVer-minor cliff driver for v0.29.0.
- **Tier:** `v0.28+`

### `pr-26-import-provenance-enum-internal-refactor` — replace ParsedImport's `(Option<BsmsAuditFields>, Option<CoreSourceMetadata>)` with `ImportProvenance` enum

- **Surfaced:** 2026-05-19, v0.27.1 Phase 5b scope-discipline deferral. The plan-doc R3 committed to Phase 5b as part of the v0.27.1 cycle's type-design sweep, but practical implementation touches 14+ sites including `apply_select_descriptor`'s filter machinery. Phase 5a + 5c shipped clean; Phase 5b deferred to keep the cycle's scope tight.
- **Where:** `crates/mnemonic-toolkit/src/wallet_import/mod.rs:60-80` (ParsedImport struct + apply_select_descriptor uses `source_metadata` directly at 5+ sites); `wallet_import/bsms.rs:272-273` + `wallet_import/bitcoin_core.rs:291-306` (parser construct sites); `cmd/import_wallet.rs:587,599,806,818,825,846` (envelope emit consumer sites — both text and JSON paths).
- **What:** Introduce `ImportProvenance { Bsms(BsmsAuditFields), BitcoinCore(CoreSourceMetadata) }` enum. Replace ParsedImport's pair with a single `provenance: ImportProvenance` field. Internal-only refactor — wire shape unchanged (envelope-side `bsms_audit` / `source_metadata` fields stay flat siblings; emit code matches on the new enum to populate them). Two practical options: (a) thread the enum through all 14 sites in one commit; (b) add back-compat `bsms_audit() -> Option<&BsmsAuditFields>` / `source_metadata() -> Option<&CoreSourceMetadata>` accessor methods, leave existing field-access sites unchanged. Option (b) is the lower-risk path.
- **Why deferred:** Phase 5b's 14-site footprint exceeded the v0.27.1 cycle's scope window after Phase 5a + 5c absorbed the type-design budget. The representable-invalid pair (both-set, both-none) is purely internal — no wire-shape or user-visible surface — so deferral has zero impact on shipped behavior.
- **Status:** resolved (cc15cf0; v0.27.2 Phase 2)
- **Tier:** `v0.28+` → `v0.27.2` (resolved at v0.27.2 per Shape A approval)

### `compare-cost-single-leaf-tr-input` — single-leaf `tr()` input support for `compare-cost`

- **Surfaced:** 2026-05-19, post-merge comprehensive review of PR #26 (I8). The slug is already cited in 2 source-code comments + 1 user-visible error message at `cost/strip.rs:5,51` and `cost/mod.rs:75`, but no FOLLOWUP was filed in the v0.26.0 cycle — this filing closes the citation-without-target loop.
- **Where:** `crates/mnemonic-toolkit/src/cost/strip.rs:51-54` (the `translate_descriptor` function's `Descriptor::Tr(_) => Err(UnsupportedWrapper)` arm); `cost/mod.rs:75` (user-visible Display impl for `UnsupportedWrapper`); `cost/mod.rs:131-138` (`run_compare_cost`'s `InputForm::{Miniscript, Descriptor}` dispatch — the `InputForm::Descriptor` arm calls `strip::translate_descriptor`). Citations verified against origin/master SHA `1abd9d1` 2026-05-19.
- **What:** Extend `translate_descriptor` (at `cost/strip.rs`) to accept `tr(<internal-key>, <single-leaf-script>)` (single-leaf only — multi-leaf TapTree is a separate scope). Map to a cost-domain that compares fairly against `wsh(...)` outputs. Specify the SPEC §-anchor before implementing — `tr()` cost comparison vs `wsh()` is non-trivial (different witness-stack shapes, different fee surfaces).
- **Why deferred:** v0.26.0 ship scope didn't include taproot input parsing for compare-cost; user-visible error directs users at this slug.
- **Status:** resolved (v0.28.0; 78936ab). Phase P12 shipped single-leaf `tr(IK, M)` input support per SPEC compare-cost v0.28.0 §11. `translate_descriptor` extended to accept `Descriptor::Tr(_)` where the TapTree contains a single leaf-script; multi-leaf TapTree continues to refuse with `UnsupportedWrapper`. Internal key surface re-used the existing cost-domain mapping against `wsh(...)`.
- **Tier:** `v0.27`

### `inspect-json-schema-version-backfill` — backfill `schema_version: "1"` field on `InspectJson` envelope to match `XpubSearchJson`

- **Surfaced:** 2026-05-18, v0.26.0 C1 (path-of-xpub) — `XpubSearchEnvelope` introduces a top-level `schema_version: "1"` field for forward-compat versioning of the per-mode tagged-union body. `InspectJson` (and `RepairJson`) carry no equivalent field; consumers that learn to read `schema_version` from `xpub-search` JSON have no parallel signal on inspect/repair envelopes.
- **Where:** `crates/mnemonic-toolkit/src/cmd/inspect.rs` `InspectJson` struct; `crates/mnemonic-toolkit/src/repair.rs` `RepairJson` struct.
- **What:** Add a top-level `schema_version: "1"` (or fresh integer initializer) to both `InspectJson` and `RepairJson` envelopes; document the SemVer compatibility policy (`Major.Minor.Patch` shape; additive fields require a Minor bump). Coordinate with mnemonic-gui consumer paths.
- **Why deferred:** scope discipline at C1; the existing consumers parse the existing envelope shape and would break on the additive field unless their parsers tolerate unknown top-level keys. v0.27+ touch.
- **Status:** resolved — v0.27.0 Phase 1 (`InspectEnvelope { schema_version, body: InspectJson }` wrapper mirroring `XpubSearchEnvelope` precedent at `cmd/xpub_search/mod.rs:111-116`). FOLLOWUP-body wording cited "both `InspectJson` and `RepairJson`"; source-verification (R3) found `RepairJson` ALREADY carries `schema_version: "1"` inline at `cmd/repair.rs:155` + construct site `cmd/repair.rs:178` (latent FOLLOWUP-body inaccuracy). Closes as no-op for Repair side; ships InspectEnvelope only.
- **Tier:** `v0.27`

### `xpub-search-passphrase-bruteforce` — brute-force passphrase scanning over a candidates file / wordlist for `xpub-search passphrase-of-xpub`

- **Surfaced:** 2026-05-18, v0.26.0 C4 (passphrase-of-xpub MVP scope). The C4 mode verifies a SINGLE passphrase; brute-force scanning is a deliberate MVP exclusion.
- **Where:** `crates/mnemonic-toolkit/src/cmd/xpub_search/passphrase_of_xpub.rs` + new `passphrase_search.rs` for the iterator/streaming.
- **What:** Three modes to consider: (a) `--passphrases-file <path>` newline-delimited candidates; (b) `--passphrases-stdin` streamed candidates; (c) generated wordlists (e.g. EFF Long, common passphrases). Per-candidate stderr-advisory budget; rate-limit / progress reporting; deterministic iteration order; abort-on-first-match. Engineering surface includes resource-bounding (memory + time) and a clear "give-up" exit code.
- **Why deferred:** scope discipline at C4; the single-passphrase verification covers the common-case forensic question ("does X passphrase produce this xpub?"). Brute-force needs a careful UX + rate-limit story to avoid foot-guns.
- **Status:** `resolved` mnemonic-toolkit-**v0.46.0** — mode **(a) file only** (`--passphrase-candidates-file <PATH>`, one candidate per line, no argv). New `passphrase_search.rs` dispatches BEFORE the inline single-passphrase resolve, streams the file (each candidate `Zeroizing<String>`), loops `derive_master_seed`→`match_xpub_against_paths`, aborts on first match. Reports the matching FILE LINE to stdout (passphrase only in `--json`); exit 4 `XpubSearchPassphraseCandidatesExhausted` with `candidates_tried` on miss. 3-way passphrase-source `ArgGroup`. **Modes (b) stdin-candidates and (c) generated wordlists were INTENTIONALLY NOT BUILT** (user decision 2026-06-05): (b) dropped to avoid `--passphrase-candidates-stdin`-vs-`--phrase-stdin` contention; (c) is keyspace GENERATION = btcrecover's job (the `--help` footer was refined to keep pointing there for generation while owning candidate-list verification). The `--rate-limit`/`--progress` surface is unneeded for a finite user-supplied list. Audit trail: `design/SPEC_xpub_search_passphrase_candidates_file.md` + `design/agent-reports/xpub-search-passphrase-candidates-r0-r{1,2,3,4}-review.md` (R0 GREEN after 4 rounds).
- **Tier:** `v0.27`
- **Spawned FOLLOWUPs:** `gui-xpub-search-passphrase-candidates-file-flag-pending-pin-bump`.

### `gui-xpub-search-passphrase-candidates-file-flag-pending-pin-bump` — mnemonic-gui `XPUB_SEARCH_PASSPHRASE_OF_XPUB_FLAGS` must add `--passphrase-candidates-file`, blocked on the GUI bumping its toolkit pin to ≥ v0.46.0

- **Surfaced:** 2026-06-05, candidate-file passphrase-scan cycle (toolkit v0.46.0) — the paired GUI schema-mirror half.
- **Where:** `mnemonic-gui/src/schema/mnemonic.rs` `XPUB_SEARCH_PASSPHRASE_OF_XPUB_FLAGS` (`~:2681`): add a `FlagSchema` for `--passphrase-candidates-file` (`FlagKind::Path { stdio_sentinel: false }`, `secret: false` — a PATH, not a secret value; copy the `--decrypt-password-file` entry shape).
- **What:** v0.46.0 added `--passphrase-candidates-file` to the toolkit's `passphrase-of-xpub` clap surface. The paired GUI `schema_mirror` update is unmergeable until the GUI bumps its toolkit pin to ≥ v0.46.0 (`schema_mirror` runs `gui-schema` against the PINNED binary; listing the flag against an older pinned binary FAILS the "schema-ahead-of-pins" gate). When the GUI next bumps its toolkit pin, land the `XPUB_SEARCH_PASSPHRASE_OF_XPUB_FLAGS` delta in the same PR. Mirrors the resolved precedents `gui-restore-multisig-flags-pending-pin-bump` / `gui-ms1-slot-subkey-pending-pin-bump`.
- **Why deferred:** Cross-repo; the GUI toolkit-pin bump is its own GUI cycle (a `schema_mirror`-ahead-of-pin change cannot ship in isolation).
- **Status:** `resolved` mnemonic-gui-**v0.26.0** (`f6caa20`, tag `mnemonic-gui-v0.26.0`). Added the `--passphrase-candidates-file` `FlagSchema` to `XPUB_SEARCH_PASSPHRASE_OF_XPUB_FLAGS` (`FlagKind::Path { stdio_sentinel: false }`, `secret: false` — mirrors `--decrypt-password-file`); bumped the GUI toolkit pin v0.44.0 → v0.46.0 (Cargo.toml + pinned-upstream.toml + Cargo.lock, `pin_coherence` lockstep) + version v0.25.0 → v0.26.0 + README install-pins. No conditional (toolkit emits `conditional_rules: []` for passphrase-of-xpub) and no secret-projection delta (`secret:false`). `schema_mirror` + `xpub_search_schema_mirror` + `pin_coherence` + full suite green (4 pinned bins); CI (build + schema-mirror) green. Audit trail: `mnemonic-gui/design/SPEC_gui_v0_26_0_passphrase_candidates_flag.md` + `design/agent-reports/gui-v0_26_0-passphrase-candidates-R0-review.md` (R0 GREEN first round).
- **Tier:** `cross-repo`
- **Tags:** `xpub-search` `passphrase` `gui` `schema-mirror`

### `xpub-search-manual-gui-chapters` — `mnemonic-gui` user manual chapters for the 4 new `xpub-search` modes

- **Surfaced:** 2026-05-18, v0.26.0 C5 cycle-close — manual-gui chapters were deferred per user direction during the C5→C6 boundary check.
- **Where:** `docs/manual-gui/src/40-mnemonic/4c-xpub-search-path-of-xpub.md` (and 4d/4e/4f for the other 3 modes); `docs/manual-gui/tests/expected_gui_schema_inventory.json` (regenerate from extract_gui_schema.py); `docs/manual-gui/pinned-upstream.toml` (bump mnemonic-gui pin to v0.11.0 once tagged).
- **What:** Four new chapters mirroring the existing `4b-slip39-combine.md` pattern (~200-500 LOC each): synopsis, per-flag anchor sections with `id="mnemonic-<sub>-<flag>"`, NodeValueComposite node enumerations (where applicable), dropdown variant anchors, exit-codes table, warnings + secret-class material disclaimers. Drives the `gui-schema-coverage` lint gate (bidirectional anchor parity vs the GUI's `SubcommandSchema` source at the pinned tag).
- **Why deferred:** user direction during C5→C6 boundary — 4 chapters × ~200-500 LOC each (~800-2000 LOC total prose). Out-of-band cycle.
- **Status:** open
- **Tier:** `v0.27`

### `mlock-g1-1-test-page-alignment-luck` — `mlock_unit::g1_1_single_page_pin_has_page_count_one` flakes under parallel test execution

- **Surfaced:** 2026-05-18, v0.26.0 C1 cycle observation. After C1's binary-layout shift the test fails under `cargo test`'s default parallel test runner because the heap allocator's bump pointer for a fresh `Box<[u8; 64]>` happens to straddle a page boundary; passes single-threaded (`cargo test -- --test-threads=1`). Pre-existing brittleness pattern (heap-allocator-luck), not a regression introduced by xpub-search.
- **Where:** `crates/mnemonic-toolkit/tests/mlock_unit.rs:28` (assertion site); `crates/mnemonic-toolkit/src/mlock.rs::pin_pages_for` (page-count derivation).
- **What:** Pin the test buffer at a known page-aligned address (e.g., `std::alloc::alloc` with a Layout that forces alignment to `*PAGE_SIZE*`) so the assertion is invariant across parallel-execution heap states. Alternative: relax the assertion to `>= 1 && <= 2` and add a paired test that uses an aligned allocator to pin the exact-page-count guarantee.
- **Why deferred:** non-regression (single-threaded passes; the v0.10.0 mlock cycle landed under this pre-existing flake too — see `feedback-default-cargo-test-runs-sibling-dependent-tests` memory). v0.27+ touch.
- **Status:** resolved (c9ead62; v0.27.2 Phase 1.3)
- **Tier:** `v0.27`

### `xpub-search-gui-bespoke-hub-pane` — discoverable umbrella hub UI for `xpub-search` modes

- **Surfaced:** 2026-05-18, v0.26.0 C5 plan-vs-codebase recon. Plan §7.2 enumerated a "hub" navigation pane with nav cards linking to the 4 mode panes. The GUI has no pane abstraction — every subcommand is a flat row in the subcommand-name ComboBox.
- **Where:** `mnemonic-gui/src/main.rs:346-602` (central panel renderer; net-new per-pane dispatch branch); `mnemonic-gui/src/schema/mnemonic.rs` (a new `SubcommandSchema` entry for the hub itself, or a sibling navigation manifest).
- **What:** Introduce a "hub" pseudo-pane visible when the user picks the umbrella `xpub-search` from a dropdown above the subcommand selector. Hub renders 4 cards (one per mode) with mode-name + 1-line description + click-through. v0.12.0 UI polish; not a v0.11.0 blocker.
- **Why deferred:** C5 plan-vs-codebase recon revealed plan §7.2's "pane" architecture was overspecified; v0.11.0 ships the 4 modes via the generic flag-renderer + the existing subcommand-name ComboBox.
- **Status:** open
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `xpub-search-gui-bespoke-hub-pane`.

### `xpub-search-gui-bespoke-widgets` — per-mode composite widgets (TargetXpubField / DescriptorIntakeField / TargetAddressField / etc.)

- **Surfaced:** 2026-05-18, v0.26.0 C5 plan-vs-codebase recon. Plan §7.3 enumerated `SeedIntakeWidget`, `TargetXpubField`, `DescriptorIntakeField`, `TargetAddressField`, `AddPathRepeater`, `XpubSearchResultRenderer` as net-new widgets. GUI codebase has NO `PhraseField` / `PassphraseField` / `Ms1Field` named types; the plan's "widget reuse" framing was wrong.
- **Where:** `mnemonic-gui/src/form/` (new modules).
- **What:** Per-mode composite widgets with affordances beyond the generic `widget::render` dispatch: TargetXpubField with prefix-detect badge; AddressTypeField that auto-suggests from xpub prefix; DescriptorIntakeField with multi-line textarea + shape-detect badge; AddPathRepeater with +/− buttons. v0.12.0 polish.
- **Why deferred:** v0.11.0 ships via the generic FlagKind dispatcher; the bespoke widgets are UX polish, not functional blockers.
- **Status:** open
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `xpub-search-gui-bespoke-widgets`.

### `xpub-search-gui-positional-intake` — positional ms1 (HRP-autodetect) routing in mnemonic-gui

- **Surfaced:** 2026-05-18, v0.26.0 C5. The toolkit accepts a positional ms1 (HRP-autodetect) on P1/P2/P4; the GUI's argv assembler does not surface this affordance — the GUI forces users into `--ms1` explicitly.
- **Where:** `mnemonic-gui/src/form/invocation.rs::assemble_argv`; `mnemonic-gui/src/schema/mnemonic.rs` `positional_args: NO_POSITIONALS` on the 4 xpub-search entries.
- **What:** Add a "drop any card" textarea/file-drop affordance that auto-routes via HRP detection (`ms1` → positional, `mk1`/`md1` → would be future modes' surfaces). v0.12.0 polish.
- **Why deferred:** v0.11.0 keeps GUI argv assembly simple; positional intake is a polish item.
- **Status:** open
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `xpub-search-gui-positional-intake`.

### `xpub-search-descriptor-md1-detection-bech32-validate` — md1 tie-break uses `starts_with("md1")` not full bech32 validation

- **Surfaced:** 2026-05-18, v0.26.0 holistic architect review (C2 R0 m2 carry-forward). `crates/mnemonic-toolkit/src/cmd/xpub_search/descriptor_intake.rs:167` routes any tokens-list whose entries all start with the literal `"md1"` prefix into the md1 chunk-assembly funnel. A garbage token like `md1xxx` routes there and fails at `md_codec::chunk::reassemble` with a clear typed error rather than falling through to literal-xpub.
- **Where:** `crates/mnemonic-toolkit/src/cmd/xpub_search/descriptor_intake.rs:167`.
- **What:** Tighten the tie-break to a real bech32 syntactic validation (e.g. `bech32::decode(t).is_ok() && t.starts_with("md1")` — even a shape-only check would be tighter). Defensible to leave as-is (the md1 HRP is unambiguous and false-positives surface as typed errors); fold or defer based on appetite for tightening.
- **Why deferred:** v0.26.0 behavior is correct (false-positives produce clean error messages, not silent misroutes); not a v0.26.0 blocker.
- **Status:** open
- **Tier:** `v0.27-nice-to-have`

### `xpub-search-address-of-xpub-searched-count-semantic` — P3 `XpubSearchNoMatch.searched` over-reports candidates

- **Surfaced:** 2026-05-18, v0.26.0 holistic architect review (C3 R0 m2 carry-forward). `crates/mnemonic-toolkit/src/cmd/xpub_search/address_of_xpub.rs:290-293` computes the `searched` count for `ToolkitError::XpubSearchNoMatch` as `num_targets × gap_limit × chains`. The actual candidate-set is `gap_limit × chains` (one shared rendered-address Vec; see `address_search.rs:75-90` for the shared-build). For 3 targets / gap_limit=20 / 2 chains: reports 120 truth-is-40.
- **Where:** `crates/mnemonic-toolkit/src/cmd/xpub_search/address_of_xpub.rs:290-293`.
- **What:** Decide on the canonical semantic — "total candidate-comparisons performed" (current; correct under that read) vs "unique child-addresses derived" (truth: `gap_limit * chains`). Either fix the count or document the chosen semantic inline. The per-target JSON envelope's `scanned_external` + `scanned_internal` fields are already correct for the per-target perspective.
- **Why deferred:** semantic-debate, not a correctness regression; the per-target JSON envelope fields are correct. v0.26.0 ships the slightly-over-reported aggregate.
- **Status:** resolved (`8304f5b`; v0.27.2 Phase 1.5) — chose "candidate-comparisons performed" semantic; docstring + inline comment clarified.
- **Tier:** `v0.27-nice-to-have`

### `xpub-search-gui-flag-mutex-visibility` — cross-flag conditional visibility for `xpub-search` mutex groups

- **Surfaced:** 2026-05-18, v0.26.0 C5. The 4 xpub-search SubcommandSchema entries set `conditional: None` for v0.11.0; cross-flag mutex visibility (e.g., greying `--ms1` when `--phrase` is filled in) is open.
- **Where:** `mnemonic-gui/src/form/conditional.rs` (new per-subcommand functions following the existing pattern at `slip39_split` / `slip39_combine` / `repair` / `inspect` / `derive_child` / etc.).
- **What:** Per-subcommand `fn(&FormState) -> FlagVisibility` functions that grey/hide flags based on cross-flag state. For xpub-search modes: enforce the seed-intake mutex visually (only one of `--phrase` / `--phrase-stdin` / `--ms1` / `--ms1-stdin` interactive at a time); surface the P4 mandatory-passphrase requirement before run-confirm; flag the multi-`--target-address` repeating affordance in P3. v0.12.0 polish.
- **Why deferred:** v0.11.0 ships with `conditional: None`; the user sees all flags simultaneously and clap-side handles the mutex at exec. The GUI's run-confirm modal will surface clap errors verbatim — functional but not ideal UX.
- **Status:** open
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `xpub-search-gui-flag-mutex-visibility`.

### `verify-bundle-watch-only-xpub-path-internal-consistency` — watch-only verify-bundle does not cross-check mk1 xpub byte-level fields against md1's claimed OriginPath

- **Surfaced:** 2026-05-17, Q&A session in `descriptor-mnemonic` repo while explaining xpub↔keypath semantics. User asked whether `verify-bundle` cross-checks md1 and mk1 internal claims; code-read confirmed it does not.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` — `run_watch_only` at `:306`, watch-only multisig branch in `run_multisig` at `:386–:413`, and the comparison sites in `emit_verify_checks` (`mk1_path_match` at `:1087–:1108`, `mk1_fingerprint_match` at `:1058–:1085`) and `emit_md1_checks` (`md1_xpub_match` at `:1654–:1691`). All checks currently compare supplied cards against a synthesized `expected` Bundle built from `--slot @N.xpub=` + template + canonical BIP path; none read mk1's `KeyCard.xpub` byte-level fields (depth, child_number, parent_fingerprint) for direct comparison against md1's `Descriptor.tlv.origin_paths` length/last-element/parent-fingerprint.
- **What:** Add a no-seed-required internal-consistency cross-check between the supplied mk1 xpub and the supplied md1 `OriginPath`. Three byte-comparison assertions (all derivable from already-decoded structs, no key derivation):
  1. `xpub.depth == origin_path.len()` (path length matches BIP-32 depth field).
  2. `xpub.child_number == origin_path.last()` (final index matches, with hardened-bit comparison preserved).
  3. For multisig: cross-cosigner `parent_fingerprint` consistency where the same parent is claimed.
  Emit as new checks (e.g., `mk1_md1_path_internal_consistency[i]`) in the SPEC §5.4 / §5.7 schema; SPEC bump per the schema-version policy. Closes the daylight between the existing stderr warning ("watch-only … does not verify --slot @0.xpub= is actually at the claimed BIP path") and the broader claim that watch-only mode can catch *internally* inconsistent bundles even without seed access.
- **Why deferred (historical):** not a correctness regression — current behavior matches the disclaimed warning at `:317–:331`. No concrete bundle-mismatch report driving urgency. Touching the SPEC §5.4 / §5.7 check schema (additive but version-bumping) is non-trivial scope for an unscheduled enhancement; better folded into a future verify-bundle hardening cycle than bolted on as a patch.
- **Resolution:** RESOLVED in v0.24.0 cycle (Tranche A.1 D30 tier upgrade — rationale: "consolidating verify-bundle defense-in-depth work in v0.24.x cycle; cross-check is cheap and the related code is in active flight"). Shipped a stderr-WARNING (not hard-error / not VerifyCheck-schema-additive) cross-check in `emit_watch_only_xpub_path_cross_check` called from `run_watch_only` and the watch-only branch of `run_multisig`. Three checks per cosigner:
  1. `xpub.depth == md1 OriginPath length` (mk-codec reconstructs depth from origin_path at decode, so this effectively asserts mk1.origin_path.len() == md1 path-decl length).
  2. `xpub.child_number == md1 OriginPath last component` (value + hardened bit).
  3. Parent-fingerprint structural sanity: at md_depth 0 the BIP-32 master invariant requires `parent_fingerprint == [0;4]`; at md_depth 1 the parent IS the master, so `parent_fingerprint` must equal the claimed master fingerprint (via md1 TLV `fingerprints` or mk1's `origin_fingerprint`). Deeper paths skip the check (would require parent xpub derivation, infeasible without seed).
  Failure mode: stderr WARNING per cosigner. Verify-bundle exit code + `result: ok / mismatch` verdict UNCHANGED — the SPEC §5.4 / §5.7 check schema is intentionally NOT extended (kept the design simple; the existing VerifyCheck rows already cover the load-bearing checks). 5 new integration cells in `crates/mnemonic-toolkit/tests/cli_verify_bundle_watch_only.rs` covering happy-path silence + 3 single-cosigner failure modes + 1 multi-cosigner failure mode. Resolution commit: TODO (this cycle's release tag).
- **Status:** resolved v0.24.0 cycle
- **Tier:** `v0.24.0`

### `gui-schema-global-flag-emission` — `mnemonic gui-schema` JSON omits global flags from per-subcommand schemas

- **Surfaced:** 2026-05-17, v0.22.x follow-ups cycle Phase A.1 execution (mnemonic-gui v0.9.0 catchup). Realized R7 risk from `/home/bcg/.claude/plans/nifty-wiggling-gosling.md` §5. mnemonic-gui v0.9.0 attempted to mirror `--no-auto-repair` into its 10 per-subcommand `*_FLAGS` arrays and the schema-mirror drift gate hard-failed: the toolkit's `cmd::gui_schema` v4 JSON emitter does not include global flags in any subcommand's `flags` array; only clap's per-subcommand `--help` TEXT propagates them.
- **Where:** `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` (the v4 emitter); downstream consumers across `mnemonic-gui` (currently the only known consumer). GUI workaround at `mnemonic-gui/src/runner.rs::prepend_no_auto_repair` + `MnemonicGuiApp.no_auto_repair` field + action-bar checkbox in `mnemonic-gui/src/main.rs` (load-bearing fallback shipped at `mnemonic-gui-v0.9.0`; ~30 LOC).
- **What:** Extend `cmd::gui_schema`'s emitter to include global flags (e.g. `--no-auto-repair`, `--debug`) per-subcommand so downstream consumers can mirror them in their per-subcommand schemas without inventing fallbacks. Either (a) duplicate the global-flag entries into every subcommand's `flags` array, or (b) add a sibling top-level `global_flags` array consumed alongside per-subcommand flags. Bump schema version per SPEC §6.10.6 additive-bump policy if needed.
- **Why deferred (historical):** the GUI's R7 fallback (action-bar checkbox prepending the flag to argv) was functionally complete at v0.9.0; the toolkit-side emitter fix is a future cycle's mechanical extension. UX improvement (per-subcommand native vs top-level affordance) but not a correctness gap.
- **Resolution:** RESOLVED in v0.24.0 cycle (Tranche B). `cmd/gui_schema.rs` v5 envelope adds three additive fields to every `flags[]` entry: `default_value: Option<String>`, `global: bool`, `secret: bool`. Schema integer version bumped `4 → 5`. `--no-auto-repair` propagates to every subcommand's flags array with `global: true`. mnemonic-gui v0.10.0 consumes the v5 fields, mirrors `--no-auto-repair` natively per-subcommand, and retires the R7 action-bar fallback. Companion FOLLOWUP closure in `bg002h/mnemonic-gui` `FOLLOWUPS.md` lockstep at v0.24.0 release tag.
- **Status:** resolved v0.24.0 cycle
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-schema-global-flag-emission`.

### `toolkit-mnemonic-force-tty-promote-from-test-only` — promote `MNEMONIC_FORCE_TTY` env-var from test-only to first-class public contract

- **Surfaced:** 2026-05-17, v0.22.x follow-ups cycle D23 lock execution (mnemonic-gui v0.9.0 catchup). Realized R1 risk from `/home/bcg/.claude/plans/nifty-wiggling-gosling.md` §5.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run` doc-comment (currently classifies the env-var as test-only); `crates/mnemonic-toolkit/src/cmd/convert.rs` + `crates/mnemonic-toolkit/src/cmd/inspect.rs` (same env-var consumed via the `is_terminal()` gate); downstream consumer at `mnemonic-gui/src/runner.rs::run` (sets `MNEMONIC_FORCE_TTY=1` on subprocess spawn at `mnemonic-gui-v0.9.0`).
- **What:** mnemonic-gui v0.9.0 sets `MNEMONIC_FORCE_TTY=1` in the toolkit subprocess env so that the toolkit's `std::io::stdout().is_terminal() && !no_auto_repair` auto-fire gate fires for GUI-spawned invocations (GUI subprocesses are piped, not TTY — without the env override the GUI would never see auto-fire repair reports from `convert` / `inspect` / `verify-bundle`). The env-var is currently documented test-only in `verify_bundle::run` doc-comment. GUI consumption creates a load-bearing dependency on the env-var's behavior; promotion to a first-class public contract (with explicit semver guarantee on its semantics) would harden the GUI side against silent toolkit-internal refactors. Update doc-comment in lockstep to reflect public-contract status; consider adding to the manual's "environment variables" section.
- **Why deferred (historical):** functional risk is documentary, not behavioral; the env-var works correctly at v0.22.1. Toolkit-side promotion is a future cycle's documentation + semver-contract addition.
- **Resolution:** RESOLVED in v0.24.0 cycle (Tranche A.1 sub-item 2). Doc-comment in `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run` rewritten to reflect first-class public-contract status with semver-stable semantics; cites mnemonic-gui v0.9.0+ as a known consumer. New "Environment variable `MNEMONIC_FORCE_TTY`" subsection added to the user manual at `docs/manual/src/40-cli-reference/41-mnemonic.md` under the verify-bundle auto-fire section. Companion FOLLOWUP closure in `bg002h/mnemonic-gui` `FOLLOWUPS.md` lockstep at v0.24.x release tag.
- **Status:** resolved v0.24.0 cycle
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `toolkit-mnemonic-force-tty-promote-from-test-only`.

### `verify-bundle-auto-fire-helper-refactor` — wire BCH auto-fire short-circuit into `verify-bundle` decode failures (v0.22.1 patch)

- **Surfaced:** 2026-05-17, v0.22.0 cycle Phase 5 scope-reduction.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` — 8 sites: `:887` `:937` `:1096` `:1101` `:1122` `:1127` `:1219` `:1532`. Helpers `emit_verify_checks` (`:856`), `emit_multisig_checks` (`:1083`), `emit_md1_checks` (`:1526`) need signature change `Vec<VerifyCheck>` → `Result<Vec<VerifyCheck>, ToolkitError>` so `?` propagation can short-circuit. 4 production callers (`:283`, `:338`, `:420`, `:682`) + 6 in-file test callers (`:1671`, `:1718`, `:1748`, `:1822`, `:1924`, `:1999`) need updates.
- **What:** v0.22.0 shipped auto-fire on `convert` (sites #9, #10) and `inspect` (site #11) but DEFERRED the `verify-bundle` helper refactor because the signature cascade through 10 callers (including v0.20/v0.21 round-trip regression cells) was high-risk for a single-shot tag window.
- **Why deferred:** scope discipline; the 3 sites already shipped cover the highest-frequency user-visible decode paths (`mnemonic convert --from ms1=… --to phrase` and `mnemonic inspect`). `verify-bundle` keeps its current UX (decode failures still surface as `VerifyCheck { passed: false, decode_error: Some(...) }` rows) until v0.22.1.
- **Resolution:** SHIPPED in v0.22.1 (2026-05-17). 3 helpers refactored to `Result<_, ToolkitError>`; 8 originally-planned sites resolved to 6 actual supplied-side wire-ups (Phase 0 R0 caught the plan's double-count of 2 expected-side decodes at original `:1096`/`:1101`). TTY-conditional default per D18 — auto-fire fires under `is_terminal() && !no_auto_repair`; pipe-context preserves the legacy VerifyCheck row behavior.
- **Status:** resolved v0.22.1 cycle
- **Tier:** `v0.22.1`

### `ms-codec-decode-with-correction-public-api` — promote `ms_codec::decode_with_correction` for downstream BCH consumers

- **Surfaced:** 2026-05-17, v0.22.0 brainstorm + R0.
- **Where:** `mnemonic-secret/crates/ms-codec/src/decode.rs` (cross-repo).
- **What:** Add `pub fn decode_with_correction(s: &str) -> Result<(Tag, Payload, Vec<RepairDetail>)>` that internally runs BCH correction within t=4 capacity before the existing decode pipeline. This lets the toolkit's `repair.rs` consume the sibling-codec native API instead of replicating BCH primitives.
- **Why deferred:** v0.22.0 toolkit-side launch first; consume sibling-codec native APIs once promoted.
- **Status:** `resolved 2026-05-17` — v0.22.x follow-ups cycle Phase B.3+B.4: ms-codec v0.2.0 shipped at `bg002h/mnemonic-secret` `f3fa531` (Phase B.4) with new `bch` module (B.3 `676097d`) + `bch_decode` module + `decode_with_correction(&str) -> Result<(Tag, Payload, Vec<CorrectionDetail>), Error>` + new `Error::TooManyErrors { bound: 8 }` variant. Toolkit-side consumer migration tracked at the companion `toolkit-repair-consume-native-codec-api` entry below (resolved at toolkit `b8ca6df`).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-secret`

### `md-codec-decode-with-correction-public-api` — promote BCH primitives + `decode_with_correction` for md HRP

- **Surfaced:** 2026-05-17, v0.22.0 R0.
- **Where:** `descriptor-mnemonic/crates/md-codec/src/bch.rs` (currently `pub(crate)`; cross-repo).
- **What:** Promote `bch::polymod_run` / `bch::hrp_expand` / `bch::MD_REGULAR_CONST` to `pub`, OR add a `pub fn decode_with_correction(strings: &[&str]) -> Result<Descriptor>` wrapper. Either path lets the toolkit's `repair.rs` consume md-codec primitives instead of vendoring `MD_NUMS_TARGET`.
- **Why deferred:** v0.22.0 vendored the constant + drift-gates it via `#[cfg(test)]` against an md1 stability suite.
- **Status:** `resolved 2026-05-17` — v0.22.x follow-ups cycle Phase B.1+B.2: md-codec v0.34.0 shipped at `bg002h/descriptor-mnemonic` `56dc300` (Phase B.2) with visibility promotions (B.1 `94069ea`: `pub const GEN_REGULAR` / `MD_REGULAR_CONST` + `pub fn polymod_run` / `hrp_expand` / `bch_create_checksum_regular` / `bch_verify_regular`) + new `bch_decode` module (~450 LOC BM+Chien port) + `decode_with_correction(&[&str]) -> Result<(Descriptor, Vec<CorrectionDetail>), Error>` + new `Error::TooManyErrors { chunk_index, bound: 8 }` variant. Toolkit-side consumer migration tracked at the companion `toolkit-repair-consume-native-codec-api` entry below (resolved at toolkit `b8ca6df`). Follow-up scope tracked at new `md-codec-decode-with-correction-supports-non-chunked-md1` (this file).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/descriptor-mnemonic`

### `ms-cli-repair-flag` — `ms repair` subcommand mirroring toolkit's `mnemonic repair`

- **Surfaced:** 2026-05-17, v0.22.0 brainstorm.
- **Where:** `mnemonic-secret/crates/ms-cli/src/cmd/` (NEW subcommand; cross-repo).
- **What:** Add `ms repair <ms1>` for ms1 BCH error-correction. Mirrors `mnemonic repair --ms1`. Blocked on `ms-codec-decode-with-correction-public-api`.
- **Status:** `resolved 2026-05-17` — v0.22.x follow-ups cycle Phase B.5: ms-cli v0.4.0 shipped at `bg002h/mnemonic-secret` `18f558a`. New `ms-cli/src/cmd/repair.rs` with `--ms1 <MS1>` required option + `--json` + exit-code parity (`0`/`5`/`2`) + cross-CLI `RepairJson` parser-reuse (D27) + D9 secret-on-stdout advisory preserved. Wraps `ms_codec::decode_with_correction` (B.4). D25 handler-signature unification cascade (5 pre-existing handlers `Result<()>` → `Result<u8>` with `Ok(0)` terminators). 5 integration cells.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-secret`

### `mk-cli-repair-flag` — `mk repair` subcommand mirroring toolkit's `mnemonic repair`

- **Surfaced:** 2026-05-17, v0.22.0 brainstorm.
- **Where:** `mnemonic-key/crates/mk-cli/src/cmd/` (NEW subcommand; cross-repo).
- **What:** Add `mk repair <mk1>...` for mk1 BCH error-correction (regular + long codes). `mk-codec` already does internal correction within `decode`, so this is mostly a UX-parity feature.
- **Status:** `resolved 2026-05-17` — `mk-cli-v0.4.0` shipped at `bg002h/mnemonic-key` `0ecbf1a` (mnemonic-toolkit v0.22.x follow-ups cycle Phase A.3'; plan `/home/bcg/.claude/plans/nifty-wiggling-gosling.md` §2.A.2).
- **Resolution:** new `mk-cli/src/cmd/repair.rs` consumes `mk_codec::string_layer::decode_string` (already-public BCH primitive per `mk-codec-v0.3.1`); surfaces full `DecodedString` (`code`/`corrections_applied`/`corrected_positions`/`corrected_char_at`); exit 5 = REPAIR_APPLIED per D26; JSON envelope byte-matches toolkit's `RepairJson` schema per D27. 7 new integration cells. D25 handler-signature cascade (6 handlers `Result<()> → Result<u8>`) shipped in the same release commit. Companion FOLLOWUP closure at `bg002h/mnemonic-key` `design/FOLLOWUPS.md` `mk-cli-repair-flag` lockstep.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-key`

### `md-cli-repair-flag` — `md repair` subcommand mirroring toolkit's `mnemonic repair`

- **Surfaced:** 2026-05-17, v0.22.0 brainstorm.
- **Where:** `descriptor-mnemonic/crates/md-cli/src/cmd/` (NEW subcommand; cross-repo).
- **What:** Add `md repair <md1>...` for md1 BCH error-correction. Blocked on `md-codec-decode-with-correction-public-api`.
- **Status:** `resolved 2026-05-17` — v0.22.x follow-ups cycle Phase B.6: md-cli v0.6.0 shipped at `bg002h/descriptor-mnemonic` `d4fbe48`. New `md-cli/src/cmd/repair.rs` with variadic `<MD1_STRINGS>...` positional + `--json` + atomic per-chunk semantics per plan §1 D28 (any chunk failing BCH capacity aborts the whole call; no partial output) + exit-code parity (`0`/`5`/`2`) + cross-CLI `RepairJson` parser-reuse (D27). Wraps `md_codec::decode_with_correction` (B.2). D25 handler-signature unification cascade (9 pre-existing handlers `Result<(), CliError>` → `Result<u8, CliError>` with `Ok(0)` terminators). 5 integration cells. Surfaced new chunked-form-only constraint tracked at companion `md-codec-decode-with-correction-supports-non-chunked-md1`.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/descriptor-mnemonic`

### `toolkit-repair-consume-native-codec-api` — replace toolkit-side BCH replication with sibling-codec native APIs

- **Surfaced:** 2026-05-17, v0.22.0 R1.
- **Where:** `crates/mnemonic-toolkit/src/repair.rs`.
- **What:** Once `ms-codec` + `md-codec` expose native `decode_with_correction` (siblings to mk-codec's existing internal correction), refactor `repair.rs` to delegate per-HRP to each codec's native correction primitive instead of parameterizing the toolkit's own polymod calls. Cleaner layering; one BCH implementation per codec instead of toolkit-side parameter-juggling.
- **Why deferred:** cross-repo dep chain; awaits items #2 and #3.
- **Status:** `resolved b8ca6df` — v0.22.x follow-ups cycle Phase B.7 (mnemonic-toolkit v0.23.0). Deleted `MS_NUMS_TARGET` (was `repair.rs:42`) + `MD_NUMS_TARGET` (was `repair.rs:45`) vendored constants. Deleted `(Self::Ms1, BchCode::Regular)` and `(Self::Md1, BchCode::Regular)` arms from `CardKind::target_residue()` (mk1 arms unchanged — mk-codec primitives still consumed natively). New `repair_via_ms_codec` + `repair_via_md_codec` private helpers delegate to `ms_codec::decode_with_correction` (B.4) and `md_codec::decode_with_correction` (B.2) respectively, with sibling-codec error → `RepairError` translation per plan §2.B.4 D29 error-mapping table. New `RepairError::PostCorrectionDecodeFailed { chunk_index: Option<usize>, detail: String }` catch-all variant absorbs orphan §4-rule decoder errors that the per-variant translation table did not enumerate. Public `repair_card` contract unchanged. All 32 pre-existing repair cells stay green via substring-match (not byte-exact equality).
- **Tier:** `cross-repo`

### `md-codec-decode-with-correction-supports-non-chunked-md1` — toolkit consumer perspective of the chunked-form-only constraint

- **Surfaced:** 2026-05-17, v0.22.x follow-ups cycle Phase B.6 implementation surfaced the gap; filed at Phase B.8 (release-boundary docs). Toolkit-side companion to the descriptor-mnemonic primary entry.
- **Where:** `crates/mnemonic-toolkit/src/repair.rs::repair_via_md_codec` (Phase B.7 delegation point) → `md_codec::decode_with_correction` (B.2 surface) → `chunk::split` + `chunk::reassemble` (the chunked-form-only integration points). After Phase B.7 migration, the toolkit's `mnemonic repair --md1` inherits the chunked-form-only constraint: users attempting to repair a non-chunked single-string md1 (the form emitted by plain `md encode` for small payloads) see a wire-format-mismatch error from the sibling codec, wrapped through `repair_via_md_codec` as `RepairError::UnparseableInput` or `RepairError::PostCorrectionDecodeFailed`.
- **What:** Toolkit-side consumer tracker for the md-codec primary. When the primary lands its non-chunked-form coverage, the toolkit migration (no code change required — the delegation already routes through the updated md-codec API) needs a CHANGELOG note + manual chapter update + smoke test confirming non-chunked-form repair works end-to-end through the toolkit's `mnemonic repair --md1`. No toolkit code change in v0.23.0; consumption tracked for the future md-codec patch release.
- **Why deferred (historical):** Primary work lived in md-codec (B.2's `chunk::split`/`chunk::reassemble` integration); toolkit consumed whatever md-codec exposed. The chunked-form path covers the most common multi-chunk error-recovery use case; the non-chunked-form tail-case was a UX nice-to-have until shipped.
- **Resolution:** RESOLVED in the output-class-advisory Phase-2 / Tier-0 fold (branch `output-class-advisory-phase2`). Prior `resolved v0.24.0 cycle` annotation was premature: the md-codec fix did ship in v0.35.0 during v0.24.0, but the toolkit's `Cargo.toml` pin was never bumped (remained `"0.34.0"`) so the fix was silently inaccessible. On the 0.34 pin, `mnemonic repair --md1 md1yqppqxqq8xtwhw4xwn4qh` (one-error non-chunked) exited **2** with `error: repair: post-correction decode failed: wire-format version mismatch: got 2, expected 4` — wholesale broken. The Tier-0 fix: bump `Cargo.toml` line 22 from `md-codec = "0.34.0"` → `md-codec = "0.35"`, relock (`md-codec v0.34.0 -> v0.35.0` in `Cargo.lock`). On 0.35 the same invocation exits **5** and recovers the original. Regression guard: `crates/mnemonic-toolkit/tests/cli_repair_md1_non_chunked.rs` (fixture `md1yqpqqxqq8xtwhw4xwn4qh`, corrupt position 3). No toolkit code change in `repair.rs` was required — the `repair_via_md_codec` delegation already routes through the updated md-codec API.
- **Status:** resolved output-class-advisory-phase2 (Tier-0 pin bump; prior `resolved v0.24.0 cycle` was premature — the pin lag was never corrected)
- **Tier:** `cross-repo`
- **Companion:** `bg002h/descriptor-mnemonic` `design/FOLLOWUPS.md` `md-codec-decode-with-correction-supports-non-chunked-md1` (primary; resolved at md-codec v0.35.0); `bg002h/mnemonic-secret` `design/FOLLOWUPS.md` `md-codec-decode-with-correction-supports-non-chunked-md1` (sibling-codec mirror); `bg002h/mnemonic-gui` `FOLLOWUPS.md` `md-codec-decode-with-correction-supports-non-chunked-md1` (GUI-side consumer).

### `hrp-correction-heuristics` — Levenshtein-1 HRP-typo auto-suggestion

- **Surfaced:** 2026-05-17, v0.22.0 brainstorm §9 open question.
- **Where:** `crates/mnemonic-toolkit/src/repair.rs::RepairError::HrpMismatch` site.
- **What:** When the user supplies `ns1…` / `mz1…` / `mb1…` etc., compute Levenshtein-1 over `{"ms", "mk", "md"}` and suggest the closest valid HRP in the error message. Today the user gets `expected 'ms', found 'ns'` but no "did you mean 'ms'?" prompt.
- **Resolution:** SHIPPED in v0.22.1 (2026-05-17) per D19. Vendored 10-line `hrp_lev1` + `suggest_hrp` in `repair.rs`; extended `RepairError::HrpMismatch` Display arm. Ambiguous inputs (`mb` is 1-sub from all three known HRPs) silently omit the suffix.
- **Status:** resolved v0.22.1 cycle
- **Tier:** `v0.22.1`

### `repair-json-short-circuit-output` — JSON envelope for auto-fire short-circuit when `--json` was requested

- **Surfaced:** 2026-05-17, v0.22.0 D14 decision.
- **Where:** `crates/mnemonic-toolkit/src/repair.rs::emit_repair_report`.
- **What:** When auto-fire fires under a `convert --json` / `inspect --json` invocation, today the repair report is emitted as TEXT-form even though the calling context expected JSON. v0.23 should detect the JSON context and emit a structured JSON envelope wrapping the repair report.
- **Resolution:** SHIPPED in v0.22.1 (2026-05-17) per D20. `try_repair_and_short_circuit` extended with `json_context: bool` param; new `emit_repair_report_json` body emits the AutoFireRepairJson schema with `auto_repair_short_circuit: true` + `exit_code: 5` discriminator fields. Applies to all 3 auto-fire surfaces (convert / inspect / verify-bundle).
- **Status:** resolved v0.22.1 cycle
- **Tier:** `v0.22.1`

### `bech32-correction-api-version-pin` — track upstream `bech32` crate correction API stability

- **Surfaced:** 2026-05-17, v0.22.0 Phase 0 (rust-bech32 v0.11.1 had no public `Corrector` API; vendored via mk-codec primitives instead).
- **Where:** monitor `bech32` crate releases.
- **What:** If/when `bech32 v0.12+` publishes a stable `primitives::correction::Corrector` API, evaluate migrating `repair.rs` off mk-codec primitives onto upstream. Probably blocked indefinitely; mk-codec primitives are stable and serve all 3 HRPs.
- **Upstream status (last checked 2026-05-17):** **still incomplete — no migration unblock signal.** Latest published version on crates.io is **`bech32 v0.11.1`** (docs.rs confirms: no items mentioning `correction`, `Corrector`, `repair`, BCH, or polymod in the public API). Available modules per docs.rs are `hrp`, `primitives`, `segwit` — none expose a correction primitive. The release-tracking PR #189 for `v0.12.0` ("Release tracking PR: `v0.12.0`") was opened 2026-01-16 and remains open as of last check (last update 2026-01-16); CHANGELOG.md on master shows no entries past `0.11.0 - 2024-02-23`. Issue #95 ("Support identifying potentially errorneous characters", open since 2024) is the only feature-tracking signal for correction work and remains unassigned. Recent commit activity on master is limited to CI / rustc-toolchain maintenance (most recent commits 2026-05-12). No open PR introduces a `Corrector` type or `decode_with_correction` API. **Future-cycle action if a `v0.12+` upstream lands a public correction primitive:** evaluate migrating `repair.rs`'s mk1 branch off vendored mk-codec primitives to upstream `bech32`; this would let the toolkit drop its mk-codec dependency on `string_layer::decode_string`'s correction internals and consolidate on a single upstream BCH implementation. Until then the toolkit stays on mk-codec primitives (which work; see `toolkit-repair-consume-native-codec-api` resolution).
- **Status:** open
- **Tier:** `v1+`

### `verify-bundle-auto-fire-feature-flag-survey` — survey users on default-on vs default-off for verify-bundle auto-fire

- **Surfaced:** 2026-05-17, v0.22.0 R6 risk.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (once helper refactor ships per `verify-bundle-auto-fire-helper-refactor`).
- **What:** Auto-fire on `verify-bundle` decode failures changes UX from "VerifyCheck row" to "exit-5 short-circuit + repair report." Survey users whether default-on (helpful) or default-off (preserves existing automation expectations) is preferable. Inform v0.22.1 default.
- **Resolution:** RESOLVED in v0.22.1 (2026-05-17) via D18 TTY-conditional default: auto-fire under TTY, legacy behavior when piped. This middle path obviates the user survey — interactive users get the helpful UX while scripts/CI keep the automation contract. No survey was conducted; the TTY split is the answer.
- **Status:** resolved v0.22.1 cycle
- **Tier:** `v0.22.1`

### `gui-schema-conditional-rules-v1` — project SPEC §6.6/§6.9 mutex/conditional rules into gui-schema JSON (drift-gated)

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle (`design/IMPLEMENTATION_PLAN_gui_conditional_applicability_v1.md`). Motivating bug: GUI bundle form default state (template = `bip84`, single-sig) emits `--threshold 1 --multisig-path-family bip48` which CLI rejects with SPEC §6.6 byte-exact errors (`crates/mnemonic-toolkit/src/cmd/bundle.rs:120`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` (P1 target — emit `conditional_rules` array, version bump 1→2); `design/SPEC_mnemonic_toolkit_v0_5.md` §6.10 (P0 canonical home — Predicate AST + Effect + drift invariant); `mnemonic-gui/src/form/conditional.rs` (P2 — ~14 new rules); `mnemonic-gui/src/form/invocation.rs` (P3 — visibility gate); `mnemonic-gui/tests/gui_schema_conditional_drift.rs` (P4 — NEW drift gate); `mnemonic-gui/src/main.rs:197-211` (P5 — remove bad default seed).
- **What:** Cross-repo mechanism + comprehensive rule coverage. Adds machine-readable `conditional_rules` to `mnemonic gui-schema` JSON; GUI's `assemble_argv` gains a visibility gate (Hidden + Disabled suppress emission, Required does not); drift gate test enforces parity between toolkit JSON and GUI hand-coded `conditional.rs`. v1 encodes ~17 enforceable visibility rules across `bundle`, `verify-bundle`, `export-wallet`, `convert`, `derive-child`. Runtime/slot-count-dependent rules deferred; see companion `gui-schema-runtime-conditional-projection`.
- **Why deferred:** in-progress this cycle (toolkit v0.16.0 + mnemonic-gui v0.5.0 lockstep; `mnemonic-gui v0.4.3` cut first as scope-isolated v0.15.0 wire-format catchup per plan §4 prerequisite).
- **Status:** `resolved 519bcfc` — shipped at `mnemonic-toolkit v0.16.0` (2026-05-16). SPEC §6.10 added; gui_schema.rs JSON projection emitted (schema v2); 1001/1001 workspace tests green. Lockstep GUI release at `mnemonic-gui v0.5.0` (commit `7b7e07d`) ships the consumer side.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-conditional-applicability-drift-fix` (resolved at GUI v0.5.0 commit `7b7e07d`).

### `gui-schema-runtime-conditional-projection` — project SPEC §6.6 slot-count-dependent + runtime rules into gui-schema JSON

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle. Filed at cycle open per plan §1.4 + §7 item 1.
- **Where:** `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` (toolkit side — extend `Predicate` AST + `conditional_rules` emitter with slot-count-aware predicates); `mnemonic-gui/src/form/conditional.rs` (gui side — slot-count signal from `FormState` to conditional engine); SPEC §6.6 rows 8, 9, 10, 11, 13, 14 (cross-cite into §6.10.7's "Runtime-deferred rules" block).
- **What:** v1 cycle deferred slot-count-dependent and post-binding rules because the GUI's conditional engine consumes `FormState` snapshots that don't natively expose `len(slots)` or `max(@N)+1`. A future cycle will plumb a slot-count signal through `FormState` + extend the `Predicate` AST with `slot_count_op` / `slot_count_min` / etc. variants. Concrete rules to add: §6.6 row 9 (T-in-range vs N), row 10 (single-sig with N > 1), row 11 (multisig with N == 1), row 13 (BIP-388 distinct-key), row 14 (per-`@N` annotation inconsistency).
- **Why deferred:** Out of v1 scope per plan §1.4 — these rules need a dynamic slot count signal not knowable until the form is filled, and surface naturally at Run time via the CLI's typed error. v1 ships argv-level submission + lets the CLI emit the error.
- **Status:** `resolved 0329800` — fully closed 2026-05-16 across three sub-cycles. v2-cycle (`mnemonic-toolkit-v0.17.0`, `4758168`) shipped the **predicate-machinery** half: schema v3 + `SlotCountEq`/`SlotCountGte`/`SlotCountLte` Predicate variants + SPEC §6.10.2 grammar docs (`a26c809`); GUI consumer + drift gate at `mnemonic-gui-v0.6.0` (`9d447d0`). Row 12 closed via the separate `pin_value` Effect (v2 cycle). The remaining row partition closed via two child FOLLOWUPs (both now resolved): `gui-schema-effect-on-dropdown-options-vocab` → resolved `c7ac604` (Batch B-1: `mnemonic-toolkit-v0.18.0` + `mnemonic-gui-v0.7.0` — `disable_options` Effect grammar for rows 10/11 + GUI-internal `NumberMax::FromSlotCount` for row 9); `gui-schema-cross-slot-predicate-projection` → resolved `38ad066` (Batch B-2: `mnemonic-gui-v0.7.1` — row 8 GUI-internal `detect_slot_index_gaps`; rows 13/14 wontfix with CLI-rejection-sufficient rationale). SPEC §6.10.7 mapping table partition reflects the final disposition: rows 8/9 `ENCODED v3 (GUI-internal)`, rows 10/11 `ENCODED v3` (toolkit-emitted), rows 12 `ENCODED v2`, rows 13/14 CLI-rejection-sufficient (wontfix).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-schema-runtime-conditional-projection` (resolved in lockstep).

### `gui-number-widget-unset-sentinel` — Number/Range/Timestamp/TaggedOrIndexed widgets lack a "no value" sentinel

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, plan §7 item 2 + plan §1.4 third bullet. Cross-referenced here for cycle bookkeeping; primary tracking lives in `bg002h/mnemonic-gui` `FOLLOWUPS.md`.
- **Where:** `mnemonic-gui/src/schema/mod.rs:263-268` (`flag_value_is_present` always returns true for Number/Range/Timestamp/TaggedOrIndexed); `mnemonic-gui/src/form/widget.rs:101-126` (`default_flag_value_for` seeds Number widgets to `min` regardless of user interaction).
- **What:** Number/Range/Timestamp/TaggedOrIndexed widgets currently have no "no value" sentinel — once seeded by `default_flag_value_for`, the value is always-present per `flag_value_is_present`. v1 sidesteps this via the §6.10 visibility gate (Hidden + Disabled flags don't emit regardless of widget value). A future cycle may add an explicit unset state for UX clarity (e.g., a "clear" affordance next to numeric widgets so users can explicitly opt out of supplying a numeric flag).
- **Why deferred:** Out of v1 scope per plan §1.4 — the visibility gate makes this unnecessary for the motivating bug. UX-quality improvement, not a correctness gap.
- **Status:** `resolved 84a69b8` — `mnemonic-gui-v0.6.0` P3 (2026-05-16; GUI-only, no toolkit code touched). +`FlagValue::Unset` variant with `#[serde(other)]` for forward-compat; argv assembler treats Unset as absent uniformly. See companion entry in `bg002h/mnemonic-gui` for full closure notes + caveat about serde-other on externally-tagged enums.
- **Tier:** `cross-repo` (gui-impact-only; cross-referenced for cycle completeness)
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-number-widget-unset-sentinel` (primary tracking, to be filed at cycle close).

### `gui-default-form-state-template-aware-seed` — replace static default-state seed with template-aware seed

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, plan §7 item 3. Natural successor to P5 (the v1 cycle's static seed cleanup at `mnemonic-gui/src/main.rs:203`). Cross-referenced here for cycle bookkeeping; primary tracking lives in `bg002h/mnemonic-gui` `FOLLOWUPS.md`.
- **Where:** `mnemonic-gui/src/main.rs:197-211` (default form-state seed; v1's P5 removes the `--multisig-path-family bip87` line but leaves the static structure intact).
- **What:** Replace the static screenshot-mode default seed with a template-aware default. When the user picks a multisig template (e.g., `wsh-sortedmulti`), the form auto-seeds multisig defaults (e.g., `--multisig-path-family bip87`, `--threshold` to a reasonable default); when the user picks single-sig, the form omits those flags entirely.
- **Why deferred:** Out of v1 scope per plan §7 — optional follow-on. The v1 P5 cleanup removes the unconditionally-wrong seed; the template-aware version is a UX enhancement.
- **Status:** `resolved 538dc70` — `mnemonic-gui-v0.6.0` P4 (2026-05-16; GUI-only, no toolkit code touched). `form::conditional::template_defaults_for(template)` returns canonical multisig defaults (`--threshold = 2`, `--multisig-path-family = bip48`) for non-single-sig templates; per-frame egui hook applies them via seed-on-empty discipline. See companion entry in `bg002h/mnemonic-gui` for full closure notes.
- **Tier:** `cross-repo` (gui-impact-only; cross-referenced for cycle completeness)
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-default-form-state-template-aware-seed` (primary tracking, to be filed at cycle close).

### `gui-schema-numeric-flag-value-pin-effect` — add `pin_value` Effect variant for §6.6 row 12 ("--account != 0 when --descriptor present") projection

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, R1 I3 reviewer fold. Plan §2.1 row 12 + §3 manifest rule 7 + §6.10.7 mapping table all marked DEFERRED for this rule.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.3 (Effect vocabulary); `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` (Effect enum + serializer); `crates/mnemonic-toolkit/src/cmd/bundle.rs:200-205` (the rule the projection encodes — `DESCRIPTOR_WITH_NONZERO_ACCOUNT`); `mnemonic-gui/src/form/conditional.rs` (consumer — Number widget value-coerce-to-zero handler).
- **What:** Add a `pin_value: { flag, value }` Effect variant to the §6.10.3 vocabulary so the GUI can coerce `--account` to 0 (or any pinned numeric value) when `--descriptor` is present, mirroring SPEC §6.6 row 12's CLI rejection at `bundle.rs:200-205`. Without this, the GUI's Number widget for `--account` defaults to `0` (per `default_flag_value_for`), which is the safe value; the rule only fires when the user actively types a nonzero value, in which case the CLI's byte-exact error suffices for v1.
- **Why deferred:** Out of v1 scope per R1 I3 reviewer fold — the GUI default of 0 makes this rare misuse; the CLI error is informative. Adding a `pin_value` Effect requires SPEC §6.10.3 expansion + GUI Number-widget coercion semantics not warranted by user evidence.
- **Status:** `resolved 4758168` — `mnemonic-toolkit-v0.17.0` P0+P1 (2026-05-16). SPEC §6.10.3 v3 grammar extension: `pin_value` Visibility variant with wire shape `{"pin_value": {"value": V}}` (`a26c809`); §6.10.4 NEW emission table enumerates PinValue's REPLACE-user-value semantic (distinct from Hidden/Disabled which suppress); §6.10.7 row 12 flipped DEFERRED → ENCODED v2; `gui_schema.rs::VisibilityProjection +PinValue` + manual `Serialize` impl preserving v2 bare-string back-compat (Copy dropped); `bundle_conditional_rules` emits the row 12 rule (`76db841`). GUI consumer + custom Deserialize accepting both v2 + v3 wire shapes + `assemble_argv` PinValue emission path at `mnemonic-gui-v0.6.0` (`9d447d0`).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-schema-numeric-flag-value-pin-effect` (to be filed at cycle close).

### `gui-schema-template-groups-meta-field` — emit per-subcommand `meta.template_groups` to retire `SINGLE_SIG_TEMPLATES` const

- **Surfaced:** 2026-05-16, GUI conditional-applicability v1 cycle, R1 I4 reviewer fold. Plan §7 item 5.
- **Where:** `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` (toolkit side — emit `meta.template_groups: { single_sig: [..], multisig: [..] }` block sourced from `Template::is_multisig()`); `mnemonic-gui/src/form/conditional.rs` (gui side — replace module-level `SINGLE_SIG_TEMPLATES: &[&str] = &["bip44", "bip49", "bip84", "bip86"]` with parse from JSON `meta.template_groups`); `crates/mnemonic-toolkit/src/template.rs:46-56` (`is_multisig()` source-of-truth — unchanged).
- **What:** v1 cycle replicates the single-sig template set client-side as a module-level `SINGLE_SIG_TEMPLATES` const in `conditional.rs`. The drift gate test detects divergence, but a future cleanup cycle can collapse the const by having the toolkit emit `meta.template_groups` in the gui-schema JSON.
- **Why deferred:** Out of v1 scope — the drift gate suffices for parity enforcement. Cleanup-class change.
- **Status:** `resolved 4758168` — `mnemonic-toolkit-v0.17.0` P0+P1 (2026-05-16). SPEC §6.10.8 NEW per-subcommand `meta` block documentation (`a26c809`); `gui_schema.rs::build_subcommand_meta` emits `meta.template_groups: { single_sig, multisig }` sourced from `CliTemplate::is_multisig()` (`76db841`). GUI-side `SINGLE_SIG_TEMPLATES` const promoted `pub(crate) → pub` + new parity test `tests/schema_mirror.rs::single_sig_templates_const_matches_meta_template_groups` (MNEMONIC_BIN-gated) at `mnemonic-gui-v0.6.0` `9d447d0`. Pair-of-checks posture (drift gate for per-rule projection + const-vs-meta for the bulk list) closes without coupling conditional-fn purity to a runtime subprocess fetch. **Defect carried forward**: `build_subcommand_meta` emits the meta block for `derive-child` but derive-child has no `--template` flag (toolkit-side bug surfaced by opus reviewer at cycle close) — tracked at new FOLLOWUP `gui-schema-derive-child-meta-template-groups-spurious`.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-schema-template-groups-meta-field` (to be filed at cycle close).

### `spec-v0_5-missing-v0_3-descriptor-mode-rows` — SPEC §6.6 table missing the v0.3-NEW descriptor-mode rows

- **Surfaced:** 2026-05-16, during the GUI conditional-applicability v1 cycle (P0 SPEC read, pre-write phase). Discovered while reading `design/SPEC_mnemonic_toolkit_v0_5.md` §6.6 (lines 189-217) to draft the §6.10 GUI-projection subsection.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_5.md` §6.6 table (lines 199-213); `crates/mnemonic-toolkit/src/cmd/bundle.rs:124-129` (the comment at line 124 reads "v0.3 NEW rows (SPEC §6.9). Byte-exact.")
- **What:** Four byte-exact error consts at `bundle.rs:124-129` — `DESCRIPTOR_AND_DESCRIPTOR_FILE`, `DESCRIPTOR_WITH_THRESHOLD`, `DESCRIPTOR_WITH_PATH_FAMILY`, `DESCRIPTOR_WITH_NONZERO_ACCOUNT` — were tagged "v0.3 NEW rows" in the source, but the v0.5 SPEC §6.6 table does NOT enumerate them. They are runtime-enforced at `bundle.rs:179-205` and pinned byte-exactly by `tests/cli_mode_violations_v0_5.rs`, so runtime behavior is correct; the gap is purely SPEC documentation drift. §6.9's text "Rows 1-14 in §6.6 plus rows 15-17 in §6.7" undercounts: actual enforced rows include the four missing descriptor-mode rules above. A future SPEC-cleanup cycle should add them to the §6.6 table (e.g., as rows 12.1/12.2/12.3/12.4 between the existing row 12 descriptor-threshold conflict and row 13 BIP-388 distinct-key) with cross-citations to `bundle.rs::mode_text::*`.
- **Why deferred:** Surfaced mid-cycle on an unrelated patch (GUI conditional-applicability v1 added a NEW §6.10 next to §6.6 and deliberately did NOT modify §6.6 itself to keep this drift fix decoupled from the cycle's scope). Independent SPEC-only fix.
- **Status:** `open`
- **Tier:** `v1+`

### `gui-run-confirm-modal-secret-redaction-manual-companion` — manual-prose lockstep companion to GUI run-confirm-modal redaction fix

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle M-P2.4 batch 4 R0 source-grep. The Defense-2 prose in `docs/manual-gui/src/10-foundations/14-secret-handling.md` (LOCKed in M-P2.4 batch 2) and the feature-2 description in `11-what-is-mnemonic-gui.md` both claim the run-confirm modal "shows the assembled argv with secret values replaced by `***`". `mnemonic-gui/src/main.rs:512-535` shows no such redaction; the modal renders each argv token verbatim. The manual prose was patched in the M-P2.4 batch-4 commit to honestly describe the actual (undesired) behavior + recommend cold-node-only operation as an operational mitigation.
- **Where:** `docs/manual-gui/src/10-foundations/14-secret-handling.md` Defense-2 section; `docs/manual-gui/src/10-foundations/11-what-is-mnemonic-gui.md` feature-2 description; `docs/manual-gui/pinned-upstream.toml` (currently pinned to `mnemonic-gui-v0.3.0`, must bump to whatever GUI tag ships the redaction fix).
- **What:** When the GUI ships the redaction fix (tracked at sibling `bg002h/mnemonic-gui` `FOLLOWUPS.md` `gui-run-confirm-modal-secret-redaction`), this manual must (i) revert the v1.0 honest-broken framing in chapters 11 + 14, (ii) restore the `***` redaction claim, (iii) drop the cold-node-only operational warning to a hover-tooltip-grade general-hygiene remark (still useful but no longer load-bearing for the security model), and (iv) bump `pinned-upstream.toml` to the GUI tag that ships the fix.
- **Why deferred:** Surfaced AFTER M-P2.4 batches 1-2 LOCKed; the manual cannot fix the GUI behavior, only describe it. v1.0 manual ships with honest-broken framing + cold-node operational mitigation; v1.1 will close the loop in lockstep with the GUI fix.
- **Status:** `open`
- **Tier:** `v1.1+`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-run-confirm-modal-secret-redaction`.

### `gui-manual-cross-refs-to-cli-manual` — bidirectional links between docs/manual-gui/ and docs/manual/ chapters where concepts overlap

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle planning. Filed per the v1.0 cycle plan (in-flight; to be archived at PE close to `design/PLAN_manual_gui_v1.md`) §2.7.
- **Where:** docs/manual-gui/src/ chapters that document concepts also covered in docs/manual/ (foundations, glossary terms, secret-bearing advisories, BIP-39 / BIP-32 primers). Currently no cross-links exist; the manuals are independent build units.
- **What:** v1.1 enhancement (Option C in the v1 planning §1.1). Add bidirectional `[CLI manual: §X.Y](https://...manual.pdf#section)` links between the GUI manual's foundations chapter and the CLI manual's equivalent chapters. Both PDFs hosted; deep-links use stable kebab-case anchors. Concept-overlap zones: bundle/card/slot terminology, BIP-39 entropy primer, BIP-32 derivation primer, codex32 / BCH primers, m-format constellation glossary.
- **Why deferred:** v1.0 cycle scope-locked to Option B (~165 pages, GUI-shaped only). Cross-references introduce bidirectional drift hazard because the CLI manual's chapters are NOT structured around the GUI's IA. Wait until both manuals are in steady state.
- **Status:** `open`
- **Tier:** `v1.1+`

### `cli-manual-html-target` — add HTML target + gh-pages deploy for the CLI manual

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle planning. Filed per the v1.0 cycle plan (in-flight; to be archived at PE close to `design/PLAN_manual_gui_v1.md`) §2.7.
- **Where:** docs/manual/Makefile (currently has `md` and `pdf` targets only; no `html`). docs/manual/Dockerfile.build (would need no changes — pandoc + xelatex already installed). .github/workflows/manual.yml (would need a gh-pages deploy job mirroring docs/manual-gui/.github/workflows/manual-gui.yml's pattern).
- **What:** v1.1 enhancement. Add `make html` target producing `build/m-format-manual.html`, plus a gh-pages deploy step in `manual.yml` triggered on `manual-v*` tags. Output lands at `https://bg002h.github.io/mnemonic-toolkit/manual/` (parallel to the GUI manual at `/manual-gui/`). Enables in-app help-icon deep-linking for any future CLI-output-driven tooling (e.g., a `mnemonic --help <subcommand>` that emits a manual URL alongside the help text).
- **Why deferred:** CLI manual is hand-curated and stable at v0.1; no current user-facing demand for HTML hosting. The GUI manual v1.0 cycle is the first time gh-pages infrastructure lands in this repo; CLI-manual HTML can piggyback once that's proven.
- **Status:** `open`
- **Tier:** `v1.1+`

### `gui-manual-localization` — non-English content support for the GUI manual

- **Surfaced:** 2026-05-15, manual-gui v1.0 cycle planning. Filed per the v1.0 cycle plan (in-flight; to be archived at PE close to `design/PLAN_manual_gui_v1.md`) §2.7.
- **Where:** docs/manual-gui/src/. Pandoc supports multi-language documents via the `lang` metadata field + Babel/Polyglossia LaTeX packages.
- **What:** Translate the GUI manual into at least one additional language (likely Spanish, given the existing BIP-39 wordlist support for `spanish`). Add a language-selector to the GUI's help-icon URL helper to deep-link to localized anchors. Requires a translation infrastructure: per-language `src/` trees + a build matrix in CI + native-speaker review.
- **Why deferred:** v1.0 ships English-only. Localization is a substantial undertaking (translation cost + per-language QA cost + ongoing drift maintenance). Defer until the manual stabilizes AND there's specific demand from a user community.
- **Status:** `open`
- **Tier:** `v2+`

### `library-error-and-language-surface-promotion` — move `error` + `language` + `friendly` modules from main.rs to lib.rs

- **Surfaced:** 2026-05-13, v0.11.0 Phase 1 R1 reviewer-loop. The P1 GREEN impl pivoted to a self-contained library surface (`FinalWordLanguage` + `FinalWordError` library-local enums) because exposing `error`/`language`/`friendly` from lib.rs today would require moving them out of `src/main.rs`'s private-module set — a cross-module refactor touching every binary file that imports `ToolkitError`. R1 reviewer endorsed the P1 pivot but recommended filing this FOLLOWUP for the future cleaner refactor.
- **Where:** Move `crates/mnemonic-toolkit/src/{error,language,friendly}.rs` from main.rs-private to lib.rs-public. Audit every `crate::error::*` / `crate::language::*` / `crate::friendly::*` import in the binary tree and re-route to `mnemonic_toolkit::error::*` / `mnemonic_toolkit::language::*` / `mnemonic_toolkit::friendly::*`. Delete `FinalWordLanguage` + `FinalWordError` library-local types and route `final_word_candidates` through `CliLanguage` + `ToolkitError` directly.
- **What:** Future cleaner crate-shape. Avoids the per-feature pattern of "library-local mirror enums" that v0.11.0 final-word and any future feature would need. Lowers boilerplate on every CLI-boundary wrapper.
- **Why deferred:** Out-of-scope for v0.11.0 — this is a crate-shape refactor that affects every binary module, not a feature-localized change. The duplication cost in v0.11.0 (10 BIP-39 language variants + 2 error variants) is bounded and trivially stable; the refactor cost is the inverse. Defer to a focused crate-shape cycle.
- **Status:** `open`
- **Tier:** `v1+`-refactor (no user-facing impact; pure crate-hygiene)

### `bip39-final-word-completer` — `mnemonic final-word` subcommand (v0.11.0)

- **Surfaced:** 2026-05-13, post-v0.10.1 user feature-request. New feature, not a deferral from a prior cycle. Plan + brainstorm at `~/.claude/plans/radiant-seeking-teacup.md`; SPEC at `design/SPEC_final_word_v0_11_0.md`.
- **Where:** New module `crates/mnemonic-toolkit/src/final_word.rs` (lib surface) + new `crates/mnemonic-toolkit/src/cmd/final_word.rs` (CLI surface) + new `Command` variant in `src/main.rs`. Library entry: `pub fn final_word_candidates(partial_phrase: &str, language: FinalWordLanguage) -> Result<Vec<&'static str>, FinalWordError>` (P1 pivoted to library-local types per FOLLOWUP `library-error-and-language-surface-promotion`). CLI: `mnemonic final-word --from phrase=<N-1-words-or-> [--language <L>] [--json-out <path>]` (single stdin route via `phrase=-` per R0 round 1 C1; no paired `--phrase-stdin` flag).
- **What:** Given an incomplete BIP-39 mnemonic of length N-1 (N ∈ {12, 15, 18, 21, 24}) and a language, emit the complete set of wordlist entries that, when appended as the Nth word, yield a phrase with a valid BIP-39 checksum. Set size is deterministic: 2^(11 − CS) ∈ {128, 64, 32, 16, 8}. Use cases: paper-backup recovery (smudged last word), manual entropy generation (dice/coin → N-1 words → checksum-fixing Nth word), verification (does my last word match what the checksum implies?). Algorithm: naïve enumeration over the 2048-entry wordlist with `bip39::Mnemonic::parse_in` as the correctness oracle.
- **Status:** `resolved f6c036a` — `mnemonic-toolkit-v0.11.0` tag pushed 2026-05-14. As-shipped: P0 SPEC `design/SPEC_final_word_v0_11_0.md` (R0 LOCK across 3 rounds at `design/agent-reports/v0_11_0-final-word-spec-r0.md`); P1 library `mnemonic_toolkit::final_word` with self-contained `FinalWordLanguage` + `FinalWordError` (R1 LOCK clean round 1, `design/agent-reports/v0_11_0-final-word-lib-r1.md`); P2 CLI handler `cmd/final_word.rs` with Cycle A `Zeroizing<String>` + `secret_in_argv_warning` + Cycle B `pin_pages_for` (R1 LOCK clean round 1, `design/agent-reports/v0_11_0-final-word-cli-r1.md`); 37 CLI tests + 17 lib tests (all green); 2 SHA-pinned JSON envelope anchors; P3 manual chapter + cli-subcommands.list mirror (R1 LOCK clean round 1, `design/agent-reports/v0_11_0-final-word-manual-r1.md`). Filed companion FOLLOWUP `library-error-and-language-surface-promotion` for the future crate-shape cleanup that would unify the library-local types with `CliLanguage` + `ToolkitError`.
- **Tier:** `v0.11.0-feature`.
- **Companion:** N/A (toolkit-only; no cross-repo work — ms-cli has no candidate insertion point per Phase 1 exploration in the plan).

### `seed-xor-coldcard-compat` — `mnemonic seed-xor` Coldcard-compatible BIP-39 ↔ BIP-39 all-or-nothing splitter (v0.12.0)

- **Surfaced:** 2026-05-14, post-v0.11.0 user feature-request. Filed at P0 alongside [[slip39-shamir-secret-sharing]] as the two-cycle pair planned in `~/.claude/plans/radiant-seeking-teacup.md`.
- **Where:** New module `crates/mnemonic-toolkit/src/seed_xor.rs` (lib surface; library-local `SeedXorError`) + new `crates/mnemonic-toolkit/src/cmd/seed_xor.rs` (CLI surface; `Split` + `Combine` sub-subcommands) + new `Command::SeedXor` variant in `src/main.rs`. Library entry: `pub fn seed_xor_split(entropy: &[u8], n_shares: usize, rng: &mut (impl rand_core::CryptoRng + rand_core::RngCore)) -> Result<Vec<Zeroizing<Vec<u8>>>, SeedXorError>` + paired deterministic + combine functions. CLI: `mnemonic seed-xor split --from phrase=<v-or-> --shares N [--language LANG] [--deterministic-from-master] [--json-out PATH]` and `mnemonic seed-xor combine --share phrase=<v-or-> ... --shares N [--language LANG] [--json-out PATH]`.
- **What:** Given a single BIP-39 entropy (12/15/18/21/24 words), split into N BIP-39 phrases such that bytewise XOR of all N entropies reconstitutes the master. Per-share BIP-39 checksum is recomputed so each share is itself a parseable, structurally-valid BIP-39 phrase. ALL-OR-NOTHING (not a threshold scheme — for K-of-N use SLIP-39 via [[slip39-shamir-secret-sharing]]). Coldcard-compatible at 12/18/24-word sizes (per `shared/xor_seed.py:assert len(raw_secret) in (16, 24, 32)`); 15/21 are toolkit-only extensions that Coldcard hardware cannot round-trip. No MAC; substitution of a wrong-but-valid-BIP-39 share is mathematically undetectable (per §A.2.6 advisory text). NEW advisory class introduced: multi-secret-on-stdout (K-of-N share emit pattern, first toolkit use).
- **Status:** `resolved 63b4503` — `mnemonic-toolkit-v0.12.0` tag pushed 2026-05-14. As-shipped: P0 SPEC `design/SPEC_seed_xor_v0_12_0.md` (R0 LOCK clean round 1, `design/agent-reports/v0_12_0-seed-xor-spec-r0.md`); P1 library `mnemonic_toolkit::seed_xor` with library-local `SeedXorError` + `seed_xor_split` / `seed_xor_split_deterministic` / `seed_xor_combine` (R1 LOCK clean round 1, `design/agent-reports/v0_12_0-seed-xor-lib-r1.md`; 17 tests + 2000 round-trip property-test pairs + Coldcard byte-pin anchor); P2 CLI handler `cmd/seed_xor.rs` with `split` + `combine` sub-subcommands wired through Cycle A/B discipline (R1 LOCK clean round 1, `design/agent-reports/v0_12_0-seed-xor-cli-r1.md`; 44 CLI tests; 2 SHA-pinned JSON envelope anchors); P3 manual chapter + cli-subcommands list (R1 LOCK clean round 1, `design/agent-reports/v0_12_0-seed-xor-manual-r1.md`). New advisory class introduced (multi-secret-on-stdout for K-of-N share emit) ready for SLIP-39 v0.13.0 to parameterize.
- **Tier:** `v0.12.0-feature`.
- **Companion:** [[slip39-shamir-secret-sharing]] (the v0.13.0 cycle's larger K-of-N counterpart; two-cycle plan ships v0.12.0 first to validate the new advisory class then v0.13.0 to extend it parameterized).

### `seed-xor-coldcard-doc-test-vectors` — vendor Coldcard `docs/seed-xor.md` test vectors

- **Surfaced:** 2026-05-14, post-v0.12.0 user request. v0.12.0 P1 already pins one Coldcard byte-pin anchor (`abandon × 12` deterministic share[0] in `tests/lib_seed_xor.rs::deterministic_split_abandon_12_share_0_byte_pin`) and the algorithm is byte-correct against `shared/xor_seed.py` (verified at P1 R1 LOCK). The Coldcard *documentation* additionally publishes two worked examples that demonstrate the algorithm by hand at the BIP-39-phrase level — these are valuable as end-to-end regression anchors at the **CLI surface** layer (not just the lib byte-XOR layer), since they exercise the full "BIP-39 phrase → entropy bytes → per-share recompute → BIP-39 phrase" round-trip with non-trivial master entropy (not the all-zeros `abandon × N` degenerate).
- **Where:** `crates/mnemonic-toolkit/tests/cli_seed_xor_happy_paths.rs` — add `coldcard_doc_24_word_vector` + `coldcard_doc_12_word_vector` tests that invoke `mnemonic seed-xor combine --share <3-phrases> --shares 3` and assert byte-equality with the documented master. NO `split` half — the doc doesn't claim Coldcard's RNG-generated shares are derivable from the master (they're random) so we can only round-trip the combine direction.
- **What:** Two worked-example vectors from <https://github.com/Coldcard/firmware/blob/master/docs/seed-xor.md>:

  **Vector 1 (24-word, N=3):**
  - Master: `silent toe meat possible chair blossom wait occur this worth option bag nurse find fish scene bench asthma bike wage world quit primary indoor`
  - Share A: `romance wink lottery autumn shop bring dawn tongue range crater truth ability miss spice fitness easy legal release recall obey exchange recycle dragon room`
  - Share B: `lion misery divide hurry latin fluid camp advance illegal lab pyramid unaware eager fringe sick camera series noodle toy crowd jeans select depth lounge`
  - Share C: `vault nominee cradle silk own frown throw leg cactus recall talent worry gadget surface shy planet purpose coffee drip few seven term squeeze educate`

  **Vector 2 (12-word, N=3):**
  - Master: `cannon opinion leader nephew found yard metal galaxy crouch between real trade`
  - Share A: `romance wink lottery autumn shop bring dawn tongue range crater truth ability`
  - Share B: `boat unfair shell violin tree robust open ride visual forest vintage approve`
  - Share C: `lion misery divide hurry latin fluid camp advance illegal lab pyramid unhappy`

  Coldcard's docs caveat that the examples are "illustrative" rather than formally normative test vectors, but the BIP-39 phrases + the XOR algorithm + per-share checksum recompute uniquely determine the round-trip — they're verifiable.
- **Why deferred:** v0.12.0 already ships a Coldcard byte-pin anchor on the deterministic-split share[0] computation (covers `Batshitoshi` prefix + SHA256d + slice-width regression). The doc-vectors are additive: end-to-end CLI round-trip evidence with non-trivial entropy. Skipped in v0.12.0 to keep P1/P2 scope tight; the patch cycle is free of any other open seed-xor work so it's a clean isolated bump.
- **Status:** `open` (filed at v0.12.0 PE+1; closes at v0.12.1 PE).
- **Tier:** `v0.12.1-patch`.
- **Companion:** N/A (toolkit-only; doc-only upstream — no sibling-repo coordination).

### `resolved-slot-entropy-zeroizing-field` — change `ResolvedSlot.entropy` to `Option<Zeroizing<Vec<u8>>>`

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 2 GREEN (deferred from in-cycle landing due to 19-site cascade).
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs:582` — `pub entropy: Option<Vec<u8>>` field on the `ResolvedSlot` (private) struct. 19 read/write sites cascade through bundle.rs, verify_bundle.rs, parse_descriptor.rs (incl. test mods).
- **What:** Per plan §"Phase 2 — Impl" step 4, ResolvedSlot.entropy was scheduled to become `Option<Zeroizing<Vec<u8>>>` so the field-resident entropy scrubs on drop. Phase 2 GREEN landed local-wrap discipline at every producer + consumer site (entropy entering the field is `Zeroizing` at construction; reads clone to a local `Zeroizing`) but left the field type as `Option<Vec<u8>>` — so the field-resident copy itself is unwrapped during its lifetime.
- **Why deferred:** 19-site cascade across 3 files + test mods is mechanically large and not representative of the per-row wrap discipline the Phase 2 zeroize-lint is enforcing. The local-wrap discipline at producer + consumer sites covers the value's transit; only the brief field-resident lifetime is unwrapped. A separate small commit can complete the field type change in one shot.
- **Status:** `superseded by resolved-slot-derived-account-zeroizing-field` (2026-05-13, Phase 3a R0 v3-fold RESCOPE per `~/.claude/plans/2026-05-13-cycle-b-phase-3a-rescope-proposal.md`). The R0 v2 LOCK that bundled this migration with the Cycle B Phase 3a `_entropy_pin` apply work was reviewed and re-scoped to Path B-lite, which carves out the field-type migration to a focused v0.10.1 patch. The new entry [[resolved-slot-derived-account-zeroizing-field]] takes broader scope (covers both `ResolvedSlot.entropy` AND `DerivedAccount.entropy: Vec<u8>` → `Zeroizing<Vec<u8>>`, plus `impl Drop for DerivedAccount` deletion, plus `into_parts` body change, plus the lint anchor relabel + new row, plus CHANGELOG entry).
- **Tier:** `v0.10.1-patch` (escalated from `v0.9.2-nice-to-have`; broader scope under the superseding entry)

### `resolved-slot-derived-account-zeroizing-field` — migrate Cycle-A `Vec<u8>` entropy fields to `Zeroizing<Vec<u8>>` (supersedes [[resolved-slot-entropy-zeroizing-field]])

- **Surfaced:** 2026-05-13, Cycle B Phase 3a R0 v3-fold RESCOPE (Path B-lite). Carved out from the R0 v2 LOCK (commit `9be0f0f`) which had bundled this migration with the `_entropy_pin` apply work; the rescope keeps the pin work in Phase 3a (toolkit `v0.10.0`) and defers the field-type migration to a focused v0.10.1 patch.
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs:585` (`pub entropy: Option<Vec<u8>>`); `crates/mnemonic-toolkit/src/derive.rs:21,49` (`pub entropy: Vec<u8>` + `impl Drop for DerivedAccount`); `crates/mnemonic-toolkit/src/derive.rs:37` (`into_parts()` body); `crates/mnemonic-toolkit/tests/lint_zeroize_discipline.rs` (row labels at lines 14-16, 109-113); `CHANGELOG.md`.
- **What** (7 deliverables, all in one v0.10.1 patch):
  1. `ResolvedSlot.entropy: Option<Vec<u8>>` → `Option<Zeroizing<Vec<u8>>>` at `synthesize.rs:585`. Cascade: 6 ctor sites (`cmd/bundle.rs:{348,449,1065}` real-pin sites + `:417,491` watch-only None sites + `synthesize.rs:1184` test) wrap construction in `Zeroizing::new(...)`. ~6 read-site edits become Deref-through-Zeroizing.
  2. `DerivedAccount.entropy: Vec<u8>` → `Zeroizing<Vec<u8>>` at `derive.rs:21`. 1 ctor site (`derive_slot.rs:77`) wraps in `Zeroizing::new(...)`.
  3. DELETE `impl Drop for DerivedAccount` at `derive.rs:49-58` (Zeroizing's Drop carries the scrub responsibility; `pub-struct-drop-semver-risk-monitor` FOLLOWUP exit also closes here).
  4. `into_parts()` body change at `derive.rs:37`: `mem::take(&mut self.entropy)` → `mem::take(&mut *self.entropy)` (Deref through Zeroizing). Outward signature returning `Vec<u8>` is preserved.
  5. `tests/lint_zeroize_discipline.rs` row "DerivedAccount impl Drop scrubs entropy on drop" relabeled to "DerivedAccount entropy field is `Zeroizing<Vec<u8>>`" with new evidence; lint lines 109-113 deferred-FOLLOWUP comment block DELETED; new row "ResolvedSlot entropy field is `Option<Zeroizing<Vec<u8>>>`" with evidence `pub entropy: Option<Zeroizing<Vec<u8>>>` against `src/synthesize.rs`.
  6. `CHANGELOG.md` v0.10.1 entry: "Field-type migration: `ResolvedSlot.entropy` and `DerivedAccount.entropy` to `Zeroizing<Vec<u8>>`; deletes `impl Drop for DerivedAccount` (Zeroizing carries scrub). Closes deferred FOLLOWUP `resolved-slot-entropy-zeroizing-field`. Closes monitoring FOLLOWUP `pub-struct-drop-semver-risk-monitor`."
  7. R1 Opus review per `feedback_opus_primary_review_agent`; report at `design/agent-reports/v0_10_1-zeroizing-field-migration-r1.md`.
- **Why deferred (separated from Cycle B Phase 3a):** Carving the field-type migration out of Phase 3a removes audit-trail entanglement (Cycle A's `lint_zeroize_discipline.rs` stays untouched in Cycle B), eliminates the Arc-wrap design from being conflated with the Zeroizing migration in reviewer eyes, and lets v0.10.1 ship as a clean focused patch with no concurrent mlock work. The cascade cost is roughly equal whether bundled or split (each ctor site takes 1 edit per landing; combined = 14 edits in one PR; split = 7 + 7 edits across two PRs). The structural-discipline gap (human-maintained `impl Drop` scrub vs. structurally-guaranteed Zeroizing field-type) stays at Cycle-A levels through Cycle B; v0.10.1 closes it.
- **Status:** `resolved ed5a1d9` — `mnemonic-toolkit-v0.10.1` tag pushed 2026-05-13. As-shipped scope: 12 ctor sites wrapped (6 direct `ResolvedSlot {` + 6 via `pub type CosignerKeyInfo = ResolvedSlot;` alias trap, caught by R0 round 1; the inline body enumeration above is the pre-R0-grep snapshot and intentionally not retroactively refreshed). 7 explicit read-site fixes (`Option::as_deref` single-step Deref mismatch, `e.clone()` over Zeroizing-ref, double-wrap break, PartialEq mismatch, =-assignment re-wrap; plus 2 sites covered by ctor-local rebind). `impl Drop for DerivedAccount` deleted (Zeroizing-drives-scrub structural guarantee replaces it). Companion FOLLOWUP `pub-struct-drop-semver-risk-monitor` also resolved (DerivedAccount-specific watch — closure follows from the Drop deletion). Plan at `~/.claude/plans/v0_10_1-zeroizing-field-migration.md` (R0 LOCK across 3 rounds); R1 impl-review CLEAR at `design/agent-reports/v0_10_1-zeroizing-field-migration-r1.md`. 620 tests green; clippy clean; miri clean.
- **Tier:** `v0.10.1-patch`.
- **Companion:** N/A (toolkit-only patch — no cross-repo work; ms-cli `v0.3.0` ships in Cycle B PE without coordination).

### `rust-secp256k1-secretkey-zeroize-upstream` — `secp256k1::SecretKey` has no Drop+Zeroize

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 2 R1 (Opus, finding I-2). The lint at `lint_safety_third_party_blocked.rs` now scans for `SecretKey::from_slice` patterns in addition to the original Mnemonic/Xpriv anchors.
- **Where:** Upstream crate `bitcoin = "0.32"` (transitive `secp256k1`). Affects every `SecretKey::from_slice` construction in `crates/mnemonic-toolkit/src/{bip85,parse_descriptor,cmd/convert}.rs` — 5 production call sites. Each carries a `SAFETY: third-party-blocked` doc-comment pointing at this FOLLOWUP.
- **What:** `secp256k1::SecretKey` is stack-bound, provides `non_secure_erase()` (which is best-effort and compiler-defeatable, per the upstream's own doc) but does NOT implement Drop with Zeroize. The toolkit's mitigation is lifetime minimization + SAFETY-anchored doc-comments at the construction sites; the residual gap is that the 32-byte scalar lives in stack memory until function exit unscrubbed. Closes when upstream `rust-secp256k1` ships a Drop+Zeroize impl for SecretKey (or when the toolkit migrates to a different curve library that does).
- **Status:** `open` (upstream-blocked)
- **Tier:** `external`

### `rust-bip39-mnemonic-zeroize-upstream` — `bip39::Mnemonic` has no Drop+Zeroize

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 2 GREEN — surfaced while landing the `SAFETY: third-party-blocked` doc-comment discipline at every `Mnemonic::parse_in` / `Mnemonic::from_entropy_in` call site in this repo.
- **Where:** Upstream crate `bip39 = "2"`. Affects every `Mnemonic` construction in `crates/mnemonic-toolkit/src/{bip85,derive,derive_slot,synthesize,parse_descriptor,cmd/{bundle,convert,derive_child}}.rs` — 25 production call sites enumerated by `lint_safety_third_party_blocked.rs::SCAN_FILES`. Each site carries a `SAFETY: third-party-blocked` doc-comment pointing at this FOLLOWUP.
- **What:** `bip39::Mnemonic` holds the phrase + internal entropy buffer but does not implement `Drop` with `Zeroize::zeroize`. The toolkit's mitigation is lifetime minimization (construct → `to_entropy()` / `to_seed()` into `Zeroizing` → immediate drop), but a residual gap remains: the secret bytes inside `Mnemonic` are not actively scrubbed before deallocation. Closes when upstream `bip39` adds `impl Drop` + `zeroize` dep.
- **Status:** `open` (upstream-blocked)
- **Tier:** `external`

### `rust-bitcoin-xpriv-zeroize-upstream` — `bitcoin::bip32::Xpriv` is Copy + no Drop + no Zeroize

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 2 GREEN — surfaced while landing the `SAFETY: third-party-blocked` doc-comment discipline at every `Xpriv::new_master` / `Xpriv::derive_priv` call site in this repo.
- **Where:** Upstream crate `bitcoin = "0.32"`. Affects every `Xpriv` construction in `crates/mnemonic-toolkit/src/{bip85,derive_slot,synthesize,parse_descriptor,cmd/{bundle,convert,derive_child}}.rs` — also enumerated by `lint_safety_third_party_blocked.rs`.
- **What:** `bitcoin::bip32::Xpriv` is `Copy` and has no Drop hook upstream. Phase 0 R3 C-R3-3 verified that `drop(xpriv)` on a `Copy` type is a no-op for memory cleanup (the value bitwise-copies into `drop()` and the original binding remains untouched; every `derive_priv` call leaves a fresh stack copy). Closes when upstream `bitcoin` removes `Copy` from `Xpriv` + adds `impl Drop` + `zeroize` dep — a coordinated breaking change requiring downstream migration at every `Xpriv` call site.
- **Status:** `open` (upstream-blocked; non-trivial breaking change for upstream)
- **Tier:** `external`

### `convert-minikey-stdout-redaction` — widen `NodeType::is_secret_bearing` to cover Casascius MiniKey on the stdout-redaction pathway

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 1 R1 review (Opus 4.7, finding N-2 partial — surfaced while folding the wider-tag method lift onto `NodeType`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` — `NodeType::is_secret_bearing` (around `convert.rs:85-96`) excludes `MiniKey` from the existing redaction + secret-on-stdout pathways at `convert.rs:769` (`from_value` redaction in `--from <secret>=` echo) and `convert.rs:796` (`secret-on-stdout` warning). Phase 1 added a wider `NodeType::is_argv_secret_bearing` method (around `convert.rs:98-110`) that DOES include MiniKey for argv-leakage advisory purposes; the narrower predicate is preserved to avoid expanding Phase 1's scope.
- **What:** MiniKey (Casascius mini-key — a private-key encoding) is a private-key carrier per survey §5 row "convert --from minikey=" but is currently NOT redacted in the `from_value` echo path and does NOT fire the `secret-on-stdout` warning on convert edges that emit a MiniKey value to stdout. Tightening: either widen `is_secret_bearing` to include MiniKey, or change the two call sites to use the wider `is_argv_secret_bearing` predicate. Either approach is small and additive.
- **Why deferred:** Phase 1 scope (argv-leakage closure) ships in lockstep with SPEC v0.9.0 §1 item 1; widening the existing secret-on-stdout warning is a separate user-facing behavior change that would entrain additional fixture updates in `tests/cli_convert_minikey.rs` (currently no advisory is expected) and warrants its own SPEC/disposition pass.
- **Status:** resolved — v0.34.5. Switched the two `convert` stdout-redaction call sites (`convert.rs:1042` from_value JSON echo + `:1069` secret-on-stdout warning) from `is_secret_bearing()` to the wider `is_argv_secret_bearing()`. The actual fix is `:1042` — `--from minikey= --to wif --json` no longer echoes the minikey private key in `from_value` (regression cell `minikey_input_redacted_in_json_from_value`). `:1069` is a no-op for MiniKey today (one-way `MiniKey→Wif`; the WIF output already trips it) but keeps both pathways on one predicate. Closed via cycle-prep recon (SHA `b17444b`).
- **Tier:** `v0.9.1-nice-to-have` (small mechanical fix; can ship in a Phase E cycle-close patch or in Cycle B planning).

### `argv-overwrite-after-parse` — rewrite `argv[]` post-clap to clear secret bytes from `/proc/$PID/cmdline`

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-2` (/proc/self/cmdline post-parse overwrite class).
- **Where:** Hypothetical new module `crates/mnemonic-toolkit/src/argv_overwrite.rs` (does not yet exist). Touches every binary entry-point (`mnemonic`, `md`, `mk`, `ms`) to invoke the overwrite shim immediately after `clap::Parser::parse()` returns. The kernel-owned mirror lives in `/proc/$PID/cmdline`; on Linux a raw FFI write into the original `argv[][i]` byte ranges (via `libc::__progname`-adjacent pointer arithmetic, or the `set_proctitle`-style trick) is the only path that actually mutates the in-kernel copy.
- **What:** Phase 1 added a stderr advisory whenever a secret is detected on argv but did NOT mutate argv. The residual gap: an attacker reading `/proc/$PID/cmdline` (same-UID; or any UID without `PR_SET_DUMPABLE=0`) sees the secret bytes for the lifetime of the process. Real fix is to (a) zero-overwrite the in-place argv slots immediately after clap consumes them, OR (b) call `prctl(PR_SET_DUMPABLE, 0)` to deny `/proc/$PID/cmdline` reads to other UIDs (narrower mitigation — does not protect same-UID reads or core dumps). Both are FFI-heavy and platform-specific.
- **Why deferred:** Phase 1's `--*-stdin` paired-flag + `=-` route closes argv-leakage for documented usage; the residual covers users who ignore the warning. SPEC §3 explicitly defers this to a future cycle pending the raw-FFI route.
- **Status:** resolved — v0.34.7. Implemented the `PR_SET_DUMPABLE(0)` mitigation (SPEC §3 OOS-2 option (b)) across all 4 m-format CLIs: a `process_hardening::set_non_dumpable()` call at the top of each `main()` denies other-UID `/proc/$PID/cmdline` reads + core dumps (Linux; no-op elsewhere). The in-place argv-overwrite (option (a)) was deliberately DECLINED — Rust's std does not expose the original `argv`, and `setproctitle`-style in-place mutation is glibc/musl/static-linking-fragile + racy + a corruption risk for marginal same-UID value (same-UID already implies ptrace/`/proc/mem` access). Residual same-UID `/proc/cmdline` window documented + accepted. Shipped: mnemonic-toolkit v0.34.7 + md-cli v0.6.1 + ms-cli v0.4.1 + mk-cli v0.4.2 + paired GUI v0.19.3 pin bump. Closed via cycle-prep recon (SHA `6e718ad`).
- **Tier:** `v1+`

### `clap-argv-pre-parse-residue` — libc `OsString` heap copies of `argv[]` live un-scrubbed before clap parses

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-libc-osstring` class.
- **Where:** `std::env::args_os() -> Vec<OsString>` materialization, called by Rust startup before user code runs. The toolkit cannot intercept earlier than `main()` without replacing libc rt0 (or per-invocation raw-FFI argv intercept). Mirrors the `mnemonic-gui/secret_widget.rs` doc-comment caveat for cross-repo parity.
- **What:** Phase 2's `std::mem::take(&mut args.phrase)` + `Zeroizing::new(...)` scrubs the clap-created `String` allocation but cannot reach the prior `OsString` heap allocation that libc materialized BEFORE clap parsed. Those `OsString` buffers drop un-scrubbed. Addressable only by libc replacement (e.g., musl + custom rt0) or per-invocation raw-FFI argv intercept before clap.
- **Why deferred:** Outside the toolkit's reach (kernel/libc layer). Mirrors mnemonic-gui caveat for parity.
- **Status:** `open`
- **Tier:** `v1+`

### `allocator-pool-residue` — `Zeroizing<Vec<u8>>` drop-time scrub may be defeated by custom-allocator page retention

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-allocator-residue` class.
- **Where:** Allocator-layer concern — applies to every `Zeroizing<Vec<u8>>` / `Zeroizing<String>` / `Zeroizing<Box<[u8]>>` allocation in the workspace when the binary is built against a non-default allocator (e.g., `jemallocator` with cache retention, `mimalloc` with retention, `tcmalloc`). The Cycle A test environment uses the system allocator (glibc malloc); custom allocators are NOT in scope.
- **What:** When a `Zeroizing<Vec<u8>>` drops, the bytes are zeroed in-place by `zeroize::Zeroize` BEFORE the deallocation call returns the page to the allocator. With the system allocator this is sound — the zeroed pages are returned to the OS or to a free-list with the zeros intact. With a retention-class custom allocator (jemalloc with `lg_dirty_mult=-1` or `dirty_decay_ms=-1`, mimalloc with retain pools, etc.), the allocator may *re-zero* or *re-use* the page for an unrelated allocation in ways that briefly expose the secret bytes to in-process readers. Mitigation requires a secret-class-aware page management discipline (custom allocator hook) or a dedicated mmap'd secret arena ([[dedicated-secret-arena]]).
- **Why deferred:** Defense-in-depth class; system allocator (default for `mnemonic`, `md`, `mk`, `ms` binaries) is sound. Custom allocators are an opt-in build configuration not in Cycle A scope.
- **Status:** `open`
- **Tier:** `v1+`

### `pub-struct-drop-semver-risk-monitor` — `impl Drop` on `DerivedAccount` breaks move-out destructure for external library users

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-pub-struct-drop` class.
- **Where:** `crates/mnemonic-toolkit/src/derive.rs:14-66` — `pub struct DerivedAccount` with `impl Drop` (Phase 2 prereq landing at commit `16f64d7`). Rust E0509 forbids move-out destructuring (`let DerivedAccount { entropy, .. } = derived;` and `let entropy = derived.entropy;`) when the enclosing struct has `impl Drop`. Field-borrow access (`&derived.entropy`) and Deref-via-method-call (`derived.entropy.as_slice()`) remain compatible.
- **What:** Cycle A chose `impl Drop` over changing the field type to `Zeroizing<Vec<u8>>` because the latter would force a v0.10.0 minor bump per the pre-1.0 SemVer convention. The trade-off is that any external library user with a move-out destructure pattern on `DerivedAccount` will get an E0509 compile error post-fold. We treat this as patch-tag-compatible (move-out is uncommon in external use; field-borrow is the typical access pattern). Monitor: if downstream library users surface complaints, the cycle re-tags retroactively as a minor bump (v0.10.0).
- **Why deferred:** The fold itself shipped in Phase 2; this entry is the monitoring artifact for the residual semver risk.
- **Status:** `resolved ed5a1d9` — `mnemonic-toolkit-v0.10.1` tag pushed 2026-05-13. `impl Drop for DerivedAccount` deleted in v0.10.1 as part of the Cycle B Path B-lite carve-out completion ([[resolved-slot-derived-account-zeroizing-field]]). Move-out destructuring (`let DerivedAccount { entropy, .. } = derived;` and `let entropy = derived.entropy;`) is once again E0509-free. The watched semver-risk concern is removed; no retroactive minor-bump retag needed.
- **Tier:** `v1+`

### `dedicated-secret-arena` — mmap-allocated page-aligned secret arena for secret-class allocations

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-secret-arena` class.
- **Where:** Hypothetical new module pattern — would replace the default heap allocator for `Zeroizing`-wrapped allocations with a dedicated mmap'd region (à la rust `secrecy` / `secure_string` crates). Touches the GlobalAlloc surface or requires a typed allocator parameter on `Zeroizing<T, A>`-style wrappers.
- **What:** `mlock(2)` pins pages, not bytes; future maintainers may need a dedicated mmap-allocated secret arena for page-aligned heap placement of secret-class allocations. This avoids both the page-vs-byte granularity trap (Cycle B mlock will hit this) and the allocator-pool residue ([[allocator-pool-residue]] becomes addressable via a dedicated allocator path). Out of scope for Cycles A and B both — this is the third-pass design class.
- **Why deferred:** Design class — Cycle A is "first-pass at OWNED-buffer hygiene"; Cycle B is mlock; arena is the natural third cycle.
- **Status:** `open`
- **Tier:** `v1+`

### `sha3-shake256-zeroize-upstream` — `sha3::Shake256` XOF reader state has no `Zeroize`

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 carry-over from survey §3.
- **Where:** Upstream crate `sha3 = "0.10"`. Affects `crates/mnemonic-toolkit/src/bip85.rs` callees that use `Shake256::default()` + `.update()` + `.finalize_xof()` to derive BIP-85 child entropy. The XOF reader holds Keccak sponge state with BIP-85 child entropy mixed in until the reader drops.
- **What:** `sha3::digest::ExtendableOutput` returns a `Shake256Reader` that reads the XOF stream lazily. The reader's internal Keccak state carries the absorbed entropy (the child secret) until the reader drops. Upstream `sha3` does not implement `Zeroize` on the Keccak state or the XOF reader. Toolkit mitigation: minimize XOF reader lifetime — call `.read(...)` into a Zeroizing<[u8; N]> output immediately and drop the reader. Residual gap: the Keccak state inside the reader is not actively scrubbed before deallocation. Closes when upstream `sha3` adds `impl Zeroize` on its core state types.
- **Status:** `open` (upstream-blocked)
- **Tier:** `external`

### `bip38-crate-internal-zeroize-upstream` — `bip38` crate's scrypt intermediate state is not zeroize-aware

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 carry-over from survey §3.
- **Where:** Upstream crate `bip38 = "1"`. Affects `crates/mnemonic-toolkit/src/cmd/convert.rs::Bip38` decrypt arm at `convert.rs:1135` (and the V3 NULL-byte-passphrase path closed in Phase 1).
- **What:** The `bip38` crate's internal scrypt KDF intermediate state and the AES round buffers are not Zeroize-wrapped. Toolkit can `Zeroizing`-wrap the *returned* `(privkey_bytes, compressed_flag)` tuple but cannot reach into the crate's stack frames during decrypt. Residual gap: scrypt intermediate state (~1 MiB by default cost factor) lives un-scrubbed on the stack/heap during the decrypt call. Closes when upstream `bip38` adds Zeroize discipline (or when the toolkit replaces with an internally-controlled scrypt + AES implementation).
- **Status:** `open` (upstream-blocked)
- **Tier:** `external`

### `secret-memory-hygiene-cycle-b` — `mlock(2)` / `VirtualLock` page-pinning infrastructure (Cycle B)

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 0 R3 architect-review SPLIT-CYCLE recommendation; opened as standalone tracker entry per Phase 3 hygiene-matrix R1 (Opus, finding C-1).
- **Where:** New module `crates/mnemonic-toolkit/src/mlock.rs` (533 LOC, Fix-B slice-fn-only design). Cross-repo: toolkit handles Sites 1-4; ms-cli handles Site 5 via inline copy. Sites realized:
  1. **Site 1** (4 per-handler clap-binding pins): `cmd/bundle.rs:129+133` (passphrase + per-slot), `cmd/verify_bundle.rs:143-150` (same), `cmd/convert.rs:668+` (effective_passphrase / effective_bip38_passphrase / primary_value), `cmd/derive_child.rs:122+` (from_value + stdin_passphrase).
  2. **Site 2** `ResolvedSlot._entropy_pin: Option<Rc<PinnedPageRange>>` at `synthesize.rs:604` (Rc not Arc per clippy `arc_with_non_send_sync` at `ddb371c`). 12 ctor sites populated (incl. `pub type CosignerKeyInfo = ResolvedSlot;` alias).
  3. **Site 3** `DerivedAccount._entropy_pin: PinnedPageRange` at `derive.rs:34`. 1 ctor site at `derive_slot.rs:89`. Cycle A `impl Drop for DerivedAccount` PRESERVED.
  4. **Site 4** bip85 7 function-local pins at `bip85.rs:{84,110,138,170,188,203,241}` (heap-promoted in Phase 1).
  5. **Site 5** ms-cli `parse::read_stdin()` pin at `parse.rs:65` (post Cycle A `Zeroizing<String>` shift).
- **What:** Phase 0 SPEC (`design/SPEC_secret_memory_hygiene_v0_9_B.md`) → Phase 1 (bip85 heap-promote precursor) → Phase 2 (mlock module Fix-B slice-fn-only + first Rust CI workflow + Miri) → Phase 3a Path B-lite (Sites 1-4 + main wire + release-build CI job; Cycle-A→Zeroizing field-type migration on ResolvedSlot/DerivedAccount carved out to v0.10.1 via [[resolved-slot-derived-account-zeroizing-field]]) → Phase 3b cross-repo (ms-cli inline mlock.rs copy + Site 5 + main wire) → PE (audit matrix + G6 invariant test + lockstep tags). POSIX-only (Linux + macOS); Windows VirtualLock deferred (SPEC §3 `OOS-windows-virtuallock`).
- **Why deferred:** R3 SPLIT-CYCLE finding — combining mlock with Zeroizing would have doubled Cycle A's review surface; splitting keeps each cycle's blast radius reviewable.
- **Status:** `resolved 9f63e8e` — `mnemonic-toolkit-v0.10.0` tag pushed 2026-05-13. Companion lockstep tag: `ms-cli-v0.3.0` (mnemonic-secret `2e7c275`). All 7 SPEC §6 gates satisfied (G1 functional / G2 soft-fail / G3 platform / G4.a Cycle A Drop preserved + G4.b Miri / G5 lockstep tags / G6 inline-copy invariant test / G7 wire-format unchanged). Cycle-close artifacts: audit matrix `design/agent-reports/v0_9_B-secret-memory-hygiene-matrix.md`; PE R0 report `design/agent-reports/v0_9_B-PE-r0.md`. Open continuation: [[resolved-slot-derived-account-zeroizing-field]] (v0.10.1 patch).
- **Tier:** `v0.9.x`
- **Companion:** `mnemonic-secret/design/FOLLOWUPS.md` — same `secret-memory-hygiene-cycle-b` short-id (cross-repo cycle entry).

### `cycle-b-pre-spec-questions` — pre-SPEC scoping questions blocking Cycle B drafting

- **Surfaced:** 2026-05-13, v1.0 roadmap-survey Bucket 1 drill-down (Opus scoping read-out, atop `mnemonic-toolkit-v0.9.2` ship). Companion to `secret-memory-hygiene-cycle-b`.
- **Where:** Resolves into the eventual `design/SPEC_secret_memory_hygiene_v0_9_B.md` Phase 0 (not yet drafted). Source artifacts surveyed: `design/FOLLOWUPS.md` Cycle B entry; `design/SPEC_secret_memory_hygiene_v0_9_0.md` §3 OOS-mlock-cycle-b (lines 271-305); `design/agent-reports/v0_9_0-secret-memory-survey.md` §4 (lines 161-210); `design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md` §4 (lines 247-269); `design/agent-reports/v0_9_0-phase-0-spec-plan-r1.md` (R3 SPLIT-CYCLE + R1 mlock-module-shape seed).
- **What:** Cycle B has enough material to start SPEC drafting, but 4 open questions + 1 architectural trap + ~4 unaddressed items must resolve before / during Phase 0 SPEC. Drafting paused until these are dispositioned.
  1. **Canonical 5-site list reconciliation.** SPEC §3 OOS-mlock-cycle-b lists `{clap-args, ResolvedSlot.entropy, DerivedAccount.entropy, bip85 [u8;64], ms-cli stdin String}`. Hygiene-matrix §4 substitutes `secp256k1::SecretKey` + `bip39::Mnemonic` interiors for slots 2 and 3 (defense-in-depth reframing). The two lists overlap by ~3 sites only. SPEC list is canonical per R4 I-R4-4 fold; Cycle B SPEC must explicitly pick one enumeration and document the matrix's alternative as OOS or supplementary coverage.
  2. **Toolkit-only scope vs ms-cli site #5.** Cycle B is framed "toolkit-only" (FOLLOWUPS + hygiene-matrix §4) but SPEC site #5 lives at `mnemonic-secret/.../ms-cli/parse.rs:45`. Either drop site #5 or revise scope to "toolkit + ms-cli stdin" (which makes Cycle B cross-repo, with a companion entry needed in `mnemonic-secret/design/FOLLOWUPS.md`).
  3. **bip85 `[u8; 64]` heap-promotion ordering.** SPEC site #4 is stack-resident `[u8; 64]` today. Survey §4 lines 206-210 says "heap-promote first; mlock those *if* they get heap-promoted." Either Cycle B absorbs heap-promotion as a precursor Phase 1, or a separate predecessor cycle does it first. Affects Cycle B's plan shape.
  4. **Platform commitment scope.** FOLLOWUPS commits to 3 platforms (Linux mlock + macOS mlock + Windows VirtualLock). Hygiene-matrix §4 narrows to "libc / Linux-specific." Pick one — the soft-fail abstraction shape (single backend with platform-gates vs three backends behind a trait) depends on this.

  **Architectural trap on record (R3 I-R3-2, Phase 0 R1 report lines 188-260):** The Phase 0 R1 prototype `try_mlock_region(&[u8])` byte-slice API "traps callers into page-vs-byte granularity wastefulness." `mlock(2)` pins pages, not bytes; SPEC §3 OOS-secret-arena defers proper page-aligned allocation to a future Cycle C (`dedicated-secret-arena`). Cycle B accepts residual page-residue from co-allocated non-secret data on locked pages; SPEC must document this and pick a signature shape that doesn't pretend byte-granularity is real.

  **Items not addressed in existing artifacts** (Phase 0 design decisions): soft-fail logging channel / level / format; `RLIMIT_MEMLOCK` exhaustion semantics (no soft-fail story beyond `EPERM` today); `CAP_IPC_LOCK` probe-up-front vs fail-per-call; cgroup memory limits.

  **Resolutions (2026-05-13 session, user decisions):**
  - **Q1 resolved:** SPEC §3 OOS-mlock-cycle-b 5-site list is canonical. Matrix's `secp256k1::SecretKey` + `bip39::Mnemonic` substitutions are out-of-Cycle-B supplementary coverage (filed in Cycle B SPEC §3 as `OOS-upstream-zeroize-mlock`); revisit when those upstreams gain Drop+Zeroize.
  - **Q2 resolved (toward cross-repo):** ms-cli site #5 stays IN Cycle B's target list. Cycle B becomes cross-repo (toolkit + ms-cli). Companion FOLLOWUP `secret-memory-hygiene-cycle-b` to be filed in `mnemonic-secret/design/FOLLOWUPS.md` at P0 SPEC ship. The "toolkit-only" framing in earlier artifacts is superseded by this SPEC's §5 cross-repo coordination.
  - **Q3 resolved:** Cycle B absorbs bip85 `[u8; 64]` heap-promotion as Phase 1 (P1 toolkit-only precursor refactor; P2 builds mlock module; P3a applies at toolkit sites; P3b applies at ms-cli; PE rollup).
  - **Q4 resolved:** Linux + macOS (POSIX path) committed for Cycle B. Windows `VirtualLock` deferred to a separate future cycle once the POSIX soft-fail abstraction has settled. Filed in Cycle B SPEC §3 as `OOS-windows-virtuallock`.

  **Architectural trap resolved (R3 I-R3-2):** Cycle B's `pin_pages_for(&[u8]) -> PinnedPageRange` returns the actual page range pinned (page-granularity explicit in the return type), NOT the byte-slice fiction. SPEC §3 `OOS-page-residue-elimination` documents that co-resident non-secret data on locked pages is incidentally pinned; full isolation deferred to Cycle C `dedicated-secret-arena`.

  **Brainstorming-session resolutions (5 additional Qs, 2026-05-13):**
  - **API shape:** hybrid — `MlockedZeroizing<T>` wrapper (sites 2/3/4) + `pin_pages_for(&[u8])` slice fn (sites 1/5). Matches libsodium's two-tier API.
  - **Capability detection:** try-and-soft-fail per call (no upfront probe). `MlockState` process-static singleton aggregates failures into a single 2-line stderr summary at end of process via `report_at_exit()`.
  - **Logging:** stderr plain-text, 2 lines, no suppression flag/env-var.
  - **Errno discipline:** all errnos soft-fail in release; `debug_assert!` on unreachable `EINVAL` in debug builds.
  - **Cross-repo sharing:** inline copy of `pin_pages_for` in both repos; CI invariant test diffs the two implementations (normalized) and fails on drift. No shared `mnemonic-mlock` crate; constellation stays at 4 crates.
- **Why deferred:** v1.0 roadmap pass; user direction is to capture pre-SPEC scope state so a future SPEC-drafting session starts cold-but-informed rather than re-discovering the discrepancies.
- **Status:** `resolved by P0 ship (commit 0c02247, 2026-05-13) — Cycle B SPEC at design/SPEC_secret_memory_hygiene_v0_9_B.md; reviewer-loop CLEAR 0C/0I across R1 (design/agent-reports/v0_9_B-phase-0-spec-r1.md: 2C/3I folded) and R2 (design/agent-reports/v0_9_B-phase-0-spec-r2.md: 0C/0I confirmed). All 4 pre-SPEC questions plus 5 brainstorming-session questions dispositioned; resolutions inlined in the What block above. Companion FOLLOWUP secret-memory-hygiene-cycle-b filed in mnemonic-secret at P0 close per SPEC §5.`
- **Tier:** `v0.9.x`
- **Companion:** `secret-memory-hygiene-cycle-b` (parent cycle entry) at `design/FOLLOWUPS.md`. If Q2 resolves toward "ms-cli stdin is in scope," a companion entry in `mnemonic-secret/design/FOLLOWUPS.md` is needed at SPEC drafting time.

### `md-mk-private-key-surface-watch` — reopen md/mk Cycle A participation if either repo grows a private-key surface

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 0 R3 architect-review I-R3-4 fold (drop md/mk symmetry-stubs); opened as standalone tracker entry per Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-md-mk` class.
- **Where:** `descriptor-mnemonic` repo (md-codec + md-cli) and `mnemonic-key` repo (mk-codec + mk-cli). Currently both hold xpub-only / descriptor-only material with no private-key buffer.
- **What:** Cycle A drops the no-scope-symmetry matrix stubs originally planned for md/mk repos because they have no secret material to audit. If either repo later gains a private-key surface (e.g., a future md-codec descriptor-binding with embedded xprv, or an mk-codec xprv passthrough), this FOLLOWUP fires and Cycle A's hygiene discipline (Zeroizing + SAFETY anchors + matrix delta) reopens for the affected sibling.
- **Why deferred:** No secret material to audit today.
- **Status:** `open` (monitoring)
- **Tier:** `cross-repo`
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-key/design/FOLLOWUPS.md`, `mnemonic-secret/design/FOLLOWUPS.md` — same `md-mk-private-key-surface-watch` short-id.

### `secret-memory-hygiene-v0_9-cycle-a` — cross-repo cycle: OWNED-buffer secret-memory hygiene v0.9.0 Cycle A

- **Surfaced:** 2026-05-13. Cycle SPEC at `design/SPEC_secret_memory_hygiene_v0_9_0.md`. Plan at `/home/bcg/.claude/plans/v0_9_0-secret-memory-hygiene.md`. Survey precursor at `design/agent-reports/v0_9_0-secret-memory-survey.md`. R1+R2+R3+R4+R5 architect-review disposition at `design/agent-reports/v0_9_0-phase-0-spec-plan-r1.md` (5 rounds: Sonnet/Sonnet/Opus/Opus/Sonnet, cleared CLEAR 0C/0I after R3 SPLIT-CYCLE pushback + user decisions on impl-Drop approach + drop md/mk symmetry-stubs).
- **Where:** mnemonic-toolkit Phases 1 (argv close: 9 toolkit flag-rows + 5 distinct impl changes), 2 (zeroize discipline: ~30 toolkit OWNED rows + `derive_master_seed` seed-step helper + `impl Drop for DerivedAccount` with `into_parts()` migration of 3 internal move-out sites at `bundle.rs:325-329`, `bundle.rs:421-425`, `synthesize.rs:741-744`), 3 (hygiene matrix file), E (rollup). Sibling participation: mnemonic-secret Phase 2 (ms-cli + ms-codec zeroize, 4 + 10 OWNED rows) + Phase 3 (matrix file).
- **What:** OWNED-buffer first-pass at secret-memory hygiene. Closes argv leakage on toolkit's `bundle` / `verify-bundle` / `derive-child` / `convert --bip38-passphrase` flags (via new `--*-stdin` flags + `slot_input.rs` `=-` parser extension). Adds zeroize-on-drop semantics to every OWNED secret allocation in ms-codec + ms-cli + mnemonic-toolkit. Cycle B (mlock infrastructure) is a separate post-Cycle-A cycle per R3 SPLIT-CYCLE finding.
- **Status:** `resolved 9035656` — `mnemonic-toolkit-v0.9.2` tag pushed 2026-05-13. Sibling-repo tags shipped in lockstep: `ms-codec-v0.1.3` (mnemonic-secret `b1694e2`), `ms-cli-v0.2.2` (mnemonic-secret `ab8c73f`). All 6 SPEC §6 gates satisfied; cycle B (mlock) deferred to a separate cycle.
- **Tier:** `cross-repo`
- **Companion:** `mnemonic-secret/design/FOLLOWUPS.md` — same `secret-memory-hygiene-v0_9-cycle-a` short-id. md / mk repos do NOT receive a companion entry this cycle (xpub-only material; SPEC §3 OOS-md-mk + R3 I-R3-4 fold).

### `bip-vector-adoption-v0_8` — cross-repo cycle: BIP-vector adoption v0.8.0

- **Surfaced:** 2026-05-13. Cycle SPEC at `design/SPEC_test_vector_audit_v0_8_0.md`. Plan at `/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md`. R1 review at `design/agent-reports/v0_8_0-phase-0-spec-plan-r1.md`.
- **Where:** mnemonic-toolkit Phase 3 = BIP-85 v85.3 (24-word BIP-39) cell at `crates/mnemonic-toolkit/tests/cli_derive_child.rs::cell_2b_bip39_24_words_reference_vector`. BIP-39 Trezor English fill (the other v0.7.1 §5 carry-over named in SPEC §2) was already closed by `feat(v0.8-phase-8)` commit `85694b2` *before* this cycle started; SPEC §2 row updated to record this. Net new for this cycle from the toolkit side: +1 vector (v85.3) plus Phase 4 audit-matrix cross-repo lift + Phase 0 SPEC + plan landed at `d0e6afc`.
- **What:** This repo's contribution to the v0.8.0 cross-repo vectors-only patch cycle. Closes when the cycle's audit-matrix successor doc lands at `design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` (Phase 4) and the patch tag is cut at Phase E. Sibling-repo phases: descriptor-mnemonic Phase 1 (BIP-341, committed `b464f3f`); mnemonic-secret Phase 2 (BIP-93, committed `7101c16`).
- **Status:** `resolved f036737` — mnemonic-toolkit-v0.9.1 tag pushed; cycle close PR #15 merged. Companion sibling-repo tags: ms-codec-v0.1.2 (mnemonic-secret 527c9c7), md-codec-v0.32.1 (descriptor-mnemonic ef00e07), mnemonic-key PR #10 (6d43115, docs-only no tag).
- **Tier:** `cross-repo`
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-secret/design/FOLLOWUPS.md`, `mnemonic-key/design/FOLLOWUPS.md` — same `bip-vector-adoption-v0_8` short-id in each.

### `bip340-schnorr-signing-surface-evaluation` — BIP-340 Schnorr test vectors deferred pending a signing surface

- **Surfaced:** 2026-05-13, v0.8.0 Phase 0 (SPEC §3 OUT-OF-SCOPE classification).
- **Where:** No file; cross-repo classification. No signing surface exists in any of the four sibling crates (grep for `schnorr` / `sign` / `signing_key` returns zero matches across `descriptor-mnemonic`, `mnemonic-toolkit`, `mnemonic-secret`, `mnemonic-key`).
- **What:** BIP-340 ships a sidecar CSV at `bip-0340/test-vectors.csv` with Schnorr signature test vectors. None of the four sibling crates exposes a signing surface, so BIP-340 is OUT-OF-SCOPE-PER-LAYER for the v0.8.0 vectors-only cycle. If a future cycle introduces signing (e.g., a `mnemonic sign-message` BIP-322 surface, or hardware-signer integration), this FOLLOWUP closes by mirroring the CSV into the relevant repo.
- **Status:** `open` (deferred until signing surface lands).
- **Tier:** `v1+`
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md` — `bip341-keypath-signing-vector-coverage` (the BIP-341 `keyPathSpending` corpus has the same gating).

### `bip39-japanese-wordlist-support` — BIP-39 Japanese vectors require JP wordlist surface

- **Surfaced:** 2026-05-13, v0.8.0 Phase 0 (SPEC §3 OUT-OF-SCOPE classification).
- **Where:** `ms-codec` and `mnemonic-toolkit` are English-only at v0.8.x. The Trezor python-mnemonic repo also publishes a Japanese vector corpus at `https://github.com/bip32JP/bip32JP.github.io/blob/master/test_JP_BIP39.json`.
- **What:** Extending BIP-39 support to the Japanese wordlist would add ~24 more Trezor-style vectors. Out-of-scope-per-product at v0.8.x; if a future cycle adds JP support, this FOLLOWUP closes by mirroring the JP vector file into `tests/`.
- **Status:** `open` (deferred; product decision, not a regression).
- **Tier:** `v1+`
- **Companion:** None (single-repo concern; `ms-codec` carries the wordlist plumbing).

### `md-cli-unspendable-key-v0.19-error-string-stale-companion` — companion: md-cli error string still references "v0.19+"

- **Surfaced:** 2026-05-11, toolkit-repo Phase 0.B audit review r1 (commit `713178c`). Surfaced from this repo's audit pass but the fix lives in the sibling `descriptor-mnemonic` repo.
- **Where:** `bg002h/descriptor-mnemonic/crates/md-cli/src/main.rs:224`.
- **What:** Companion entry — see the primary entry in `descriptor-mnemonic/design/FOLLOWUPS.md` (`md-cli-unspendable-key-v0.19-error-string-stale`) for the full action item. No toolkit-side action; closure will happen when the md1-repo entry resolves.
- **Status:** `resolved` (2026-05-11 alongside the primary md1-side fix; see the md1 FOLLOWUP entry for the resolving commit).
- **Tier:** `cross-repo`
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md` — `md-cli-unspendable-key-v0.19-error-string-stale`

### `manual-v0.18-stale-md1-scenario-phrases` — quickstart/workflow chapters carry v0.17-era md1 phrases that no longer round-trip under v0.18

- **Surfaced:** 2026-05-09, PR #11 architect review (manual-mirror for descriptor-mnemonic v0.18 cycle).
- **Where:** `docs/manual/src/30-workflows/31-singlesig-steel.md` (lines 87–89), `docs/manual/src/20-quickstart/22-first-bundle.md` (~line 63), `docs/manual/src/20-quickstart/23-verify.md` (~line 24), `docs/manual/src/40-cli-reference/44-mk-cli.md` (~line 54). All reference the v0.17-era 3-chunk `md1zsxdspqqqpm6jzzqq...` scenario phrase set (3-of-3 multisig).
- **What:** descriptor-mnemonic v0.18 is a wire-format break (`Tag::TrUnspendable` removed, `key_index_width` formula changed). v0.17 phrases now reject under v0.18 with `Error::UnknownExtensionTag(0x05)`. PR #11 limited scope to CLI surface (per `manual-cli-surface-mirror` invariant). The scenario phrases need regenerating from source (run the `mnemonic` derivation pipeline against the abandon mnemonic with v0.18 binaries) and the chapters re-published.
- **Why deferred:** PR #11 maintains narrow CLI-surface-mirror scope. Scenario-content refresh is a separate concern that involves running the full 4-format pipeline and regenerating multiple chunked phrases. Local-only impact; toolkit CI runs `make lint` not `make verify-examples`.
- **Status:** `open`
- **Tier:** `v0.2` (next minor; non-blocking for descriptor-mnemonic v0.18 release).

- **2026-05-31 (cycle B):** `docs/technical-manual` is NOT CI-gated; cycle B's output-class advisory re-word was intentionally NOT applied there (P5 reverted it) because the stale v0.17 md1 makes a clean advisory-only re-capture impossible and a full re-capture would bake a `md1_decode: fail WireVersionMismatch`. When this FOLLOWUP's stale-md1 is fixed, ALSO re-word the D9→output-class advisory in the technical-manual transcripts (`ms1-{encode,decode}`, `mnemonic-bundle-bip84-abandon`, etc.) in the same pass.

### `lint-md-flag-coverage-vacuous-with-md_bin-true` — CI flag-coverage step skipped for md/ms/mnemonic via `MD_BIN=true` substitution

- **Surfaced:** 2026-05-09, PR #11 architect review.
- **Where:** `.github/workflows/manual.yml` invokes `make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=mk`. The `flag-coverage` step in `tests/lint.sh` runs `eval "$cmd <subcommand> --help"`; when `cmd=true`, the shell builtin `true` ignores all args and emits no flags, triggering the `warn "no flags parsed"` skip path.
- **What:** Only `mk` actually executes flag-coverage in CI (mk is `cargo install`'d in the workflow). md/ms/mnemonic flag-coverage is silently vacuous. Pre-existing gap; not introduced by PR #11. The `lint` claim "every CLI flag is documented" is therefore not enforced for 3 of 4 binaries in CI — manual `make lint MD_BIN=/path/to/md` runs catch flag drift, but no CI gate.
- **What to fix:** install `md`, `ms`, and `mnemonic` binaries in the manual.yml workflow (similar to how `mk` is installed) and pass them to `make lint` instead of `=true`.
- **Why deferred:** orthogonal to PR #11's CLI-surface-mirror scope; pre-existing infrastructure gap.
- **Status:** `open`
- **Tier:** `v0.2` (CI hardening; non-blocking).

### `manual-cli-surface-mirror` — manual mirrors the four-format CLI/API surface

- **Surfaced:** 2026-05-07, m-format-star user manual v0.1 release (`manual-v0.1.0` tag; PR #1).
- **Where:** `docs/manual/src/40-cli-reference/` (`41-mnemonic.md`, `42-md.md`, `43-ms.md`, `44-mk-codec-rust.md`); CI gate at `docs/manual/tests/lint.sh` `flag-coverage` step (per-`<binary, subcommand>` pair).
- **What:** v0.1 of the manual mirrors `mnemonic` (this repo), `md-cli` (`descriptor-mnemonic`), `ms-cli` (`mnemonic-secret`), and the `mk-codec` Rust API (`mnemonic-key`) verbatim against toolkit v0.8.0. **Any flag addition or removal in any of those four surfaces must touch `docs/manual/src/40-cli-reference/` in lockstep with the implementing PR**; the manual's `flag-coverage` lint step gates on missing flags. Companion entries: `manual-cli-surface-mirror` in `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-secret/design/FOLLOWUPS.md`, and `mnemonic-key/design/FOLLOWUPS.md`.
- **Why filed:** the manual is a separate artifact (independent `manual-v*` versioning); without an explicit cross-repo mirror invariant, sibling-side flag changes would silently drift the manual.
- **Status:** `open` (mirror invariant active for the lifetime of `docs/manual/`)
- **Tier:** `cross-repo`

### `spec-5-5-kind-enum-gap` — SPEC §5.5 `kind` enum table omits `NetworkMismatch` and `FutureFormat`

- **Surfaced:** Phase 1 review r1 (L-1).
- **Where:** `design/SPEC_mnemonic_toolkit_v0_1.md` §5.5.
- **What:** SPEC §5.5 enumerates `"kind"` JSON values as `"BadInput" | "Bip39" | … | "ModeViolation"` but doesn't list `NetworkMismatch` and `FutureFormat`. The implementation correctly returns those discriminants; the SPEC prose is just incomplete.
- **Why deferred:** SPEC-prose-only; no code change required. Update during the next SPEC revision.
- **Status:** `resolved (this commit, 2026-05-13) — NetworkMismatch + FutureFormat added to §5.5 kind enum.`
- **Tier:** `v0.1-nice-to-have`

### `mk-codec-chunked-visual-grouping-helper` — mk-codec lacks a per-string visual grouping helper

- **Surfaced:** Phase 1 spike memo + Phase 1 review r1 (L-2).
- **Where:** `crates/mnemonic-toolkit/src/format.rs::chunk_mk1` (cross-repo: would consume a new `mk_codec::encode::render_grouped` if it existed).
- **What:** md-codec exposes `render_codex32_grouped(s, 5)` for engraving-friendly hyphenated 5-char groups; mk-codec has no equivalent. Toolkit's `chunk_mk1` falls back to space-separated 5-char groups via `chunk_5char`. v0.1 fixtures pin the space-separated behavior.
- **Why deferred:** non-blocking; functionally equivalent fallback. Library-API gap in mk-codec.
- **Status:** `open`
- **Tier:** `cross-repo`

### `plan-spike-md-codec-filler-bug` — IMPLEMENTATION_PLAN's `spike_md_codec.rs` snippet uses invalid SEC1 filler

- **Surfaced:** Phase 1 review r1 (Nit-1) + Task 1.1 spike memo.
- **Where:** `design/IMPLEMENTATION_PLAN_mnemonic_toolkit_v0_1.md` Task 1.1, `spike_md_codec.rs` snippet (~line 232–260).
- **What:** Plan-given snippet uses `[0x42; 65]` as `tlv.pubkeys` filler, which violates the SEC1-compressed-pubkey prefix invariant (must be 0x02/0x03) and panics with `InvalidXpubBytes`. Spike memo documents the working filler `[0x11; 32] || 0x02 || [0x22; 32]` from `md_codec::identity::deterministic_xpub`. Plan source not patched — future readers running the snippet verbatim will trip the same panic.
- **Why deferred:** spike memo supersedes plan source; cosmetic plan-source bug.
- **Status:** `resolved (this commit, 2026-05-13) — Task 1.1 snippet now uses 32B 0x11 chain_code || 0x02 SEC1 prefix || 32B 0x22 x-coordinate.`
- **Tier:** `v0.1-nice-to-have`

### `plan-trezor-24-fingerprint-stale` — IMPLEMENTATION_PLAN has wrong 24-word zero-entropy master fingerprint

- **Surfaced:** Task 2.1 implementer (verified via spike harness `/tmp/toolkit-spike/spike_trezor_fp.rs`).
- **Where:** `design/IMPLEMENTATION_PLAN_mnemonic_toolkit_v0_1.md` Task 2.1 test assertion (~line 1540) + Task 2.3 commit-message body.
- **What:** Plan asserts `73c5da0a` as the Trezor 24-word "abandon × 23 art" master fingerprint. That value is the **12-word** "abandon × 11 about" vector's fingerprint (rust-miniscript test corpus). Correct 24-word fingerprint is `5436d724`. Handoff doc was corrected during execution; plan source unpatched.
- **Why deferred:** test code uses correct value; only plan documentation is stale.
- **Status:** `resolved (this commit, 2026-05-13) — plan now references 5436d724 (24-word fingerprint) in all 3 sites.`
- **Tier:** `v0.1-nice-to-have`

### `friendly-mk-codec-mixedcase-wording` — `friendly_mk_codec` `MixedCase` text word-order differs from SPEC §6.4.4

- **Surfaced:** Phase 3 review r1 (L-1).
- **Where:** `crates/mnemonic-toolkit/src/friendly.rs:friendly_mk_codec` (`MixedCase` arm).
- **What:** SPEC §6.4.4 row says `"mixed case in mk1 input string"`. Code says `"mk1 mixed case in input string"`. Functionally equivalent; word order differs.
- **Why deferred:** no integration test pins the byte-exact text yet; cosmetic.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `bundle-emit-bypasses-chunk-mk1-alias` — `bundle.rs::emit()` calls `chunk_5char` directly for mk1; `chunk_mk1` alias dead

- **Surfaced:** Phase 3 review r1 (L-2) + Phase 5 review r1 (L-2).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::emit` + `crates/mnemonic-toolkit/src/format.rs::chunk_mk1`.
- **What:** `chunk_mk1` is a reserved alias for `chunk_5char`, retained against the future mk-codec grouping helper (see `mk-codec-chunked-visual-grouping-helper`). `bundle.rs::emit` calls `chunk_5char` directly, leaving `chunk_mk1` flagged as dead code. Switch the call site to `chunk_mk1` so the swap point is single-edit.
- **Why deferred:** functionally identical; one-line cleanup.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `watch-only-stderr-warning-suborder` — depth advisory ordering vs account-index hazard unspecified

- **Surfaced:** Phase 3 review r1 (L-3).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::bundle_watch_only`.
- **What:** Watch-only path emits the conditional depth advisory before the unconditional account-index hazard. SPEC §5.2 lists "watch-only mode warning" as item 3 without specifying the sub-order between these two. Phase 5 fixtures don't cover stderr ordering.
- **Why deferred:** SPEC-ambiguous; Phase 5 doesn't pin the ordering.
- **Status:** `resolved` — moot / resolved-by-refactor. The cited `bundle.rs::bundle_watch_only` was DELETED in `bc59ee3` (v0.4.2 Phase M.3, ~990 lines of legacy CLI dispatch removed); the watch-only path is now folded into `bundle.rs::run` (mode via `bundle.any_secret_bearing()`, `bundle.rs:678`). The specific two-advisory sub-order site no longer exists; re-file with current citations if a real ordering ambiguity persists. (cycle-prep recon 2026-05-22, SHA `1d6436d`.)
- **Tier:** `v0.1-nice-to-have`

### `spec-2-2-2-vs-5-4-checks-count-prose` — SPEC §2.2.2 prose says "four checks" but §5.4 schema mandates 9-element array

- **Surfaced:** Phase 4 review r1 (L-1).
- **Where:** `design/SPEC_mnemonic_toolkit_v0_1.md` §2.2.2 vs §5.4.
- **What:** §2.2.2 lists 4 substantive watch-only checks; §5.4 schema (line 552) requires all 9 check-name slots populated, with `skipped` for non-applicable. Implementation follows §5.4 (correct). §2.2.2 prose should clarify "4 substantive (5 of the 9 schema slots are `skipped` per §5.4)".
- **Why deferred:** SPEC-internal inconsistency; implementation behavior is correct per the schema.
- **Status:** `resolved (this commit, 2026-05-13) — §2.2.2 prose clarified: "four substantive checks" with explicit §5.4 9-slot schema reference.`
- **Tier:** `v0.1-nice-to-have`

### `bundle-mismatch-card-static-str-constraint` — `BundleMismatch.card: &'static str` constrains future runtime-id callers

- **Surfaced:** Phase 4 review r1 (L-2). Confirmed as Phase 0 mandatory fixup by 2026-05-05 v0.1 audit (`design/audit-v0_1-for-v0_2-extension.md` IMP-1).
- **Where:** `crates/mnemonic-toolkit/src/error.rs::ToolkitError::BundleMismatch`.
- **What:** Field type was `&'static str`. v0.2 multisig emits per-cosigner card identifiers like `"mk1[0]"` that are runtime-formatted; `&'static str` would force a breaking field-type change mid-v0.2-cycle. Resolved as part of v0.2 Phase 0.
- **Status:** `resolved 9396a58 — field changed to String; test construction sites updated to .into(); doc-comment clarified.`
- **Tier:** `v0.2`

### `verify-bundle-text-mode-trailing-space` — `"{}: {} {}"` produces trailing space when `detail` is empty

- **Surfaced:** Phase 4 review r1 (L-3).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run` text-mode output.
- **What:** Skipped checks with empty `detail` render as `"md1_xpub_match: skipped "` (trailing space). SPEC §5.4 only pins JSON byte-exact; text mode is unpinned.
- **Why deferred:** cosmetic; not test-covered.
- **Status:** `resolved by v0.5.0 Phase F (commit 85c678b) — branch on detail.is_empty() at 3 emit sites`
- **Tier:** `v0.1-nice-to-have`

### `error-allow-comments-staleness` — `error::Result<T>` and `BundleMismatch` doc-comments will rot

- **Surfaced:** Phase 4 review r1 (N-1, N-2) + Phase 5 review r1 (N-2). Bundled into Phase 0 fixup by 2026-05-05 v0.1 audit (`design/audit-v0_1-for-v0_2-extension.md` IMP-2).
- **Where:** `crates/mnemonic-toolkit/src/error.rs` `Result` alias + `BundleMismatch` variant doc.
- **What:** `Result<T>` allow-comment said "reserved for in-crate use" but the type is `pub type` (exported). `BundleMismatch` doc-comment said "Constructed by integration tests in Phase 5" — stale once v0.2 wires the variant as a live runtime error.
- **Status:** `resolved 9396a58 — Result<T> comment now reads "Convenience alias; exported for downstream-crate use." BundleMismatch comment now reads "Exit-4 verify-bundle mismatch variant; card identifies the mismatching card (e.g., mk1, md1, or mk1[N] for multisig cosigner N)."`
- **Tier:** `v0.1-nice-to-have`

### `cli-watch-only-test-hardcodes-fingerprint` — `cli_bundle_watch_only.rs` hardcodes `5436d724` rather than reading from decoded mk1

- **Surfaced:** Phase 5 review r1 (L-3).
- **Where:** `crates/mnemonic-toolkit/tests/cli_bundle_watch_only.rs`.
- **What:** Test extracts the xpub from the mk1 fixture via `mk_codec::decode` (correct), but passes `"5436d724"` as the master-fingerprint argument literally. Works because the Trezor 24-word zero vector's fingerprint is constant; future vector swap requires updating the fingerprint in two places. Read it from `card.origin_fingerprint` instead.
- **Why deferred:** works; two-place edit risk only.
- **Status:** `resolved (this commit, 2026-05-13) — both tests read fp_hex from card.origin_fingerprint via .to_string(); cargo test --test cli_bundle_watch_only passes.`
- **Tier:** `v0.1-nice-to-have`

### `changelog-sha-pin-no-reproduction-command` — CHANGELOG SHA pin doesn't document how to reproduce it

- **Surfaced:** Phase 5 review r1 (N-1).
- **Where:** `CHANGELOG.md` Wire-format SHA pin section.
- **What:** SHA `81828299c927783d915108f32c9752b3dbf815c1caba5b6f6e7ce7b810ddcbf6` is documented as `sha256(crates/mnemonic-toolkit/tests/vectors/v0_1/)` but doesn't specify the exact reproduction command (`shasum -a 256 *.txt | sort | shasum -a 256`). Verifiers may need to guess.
- **Why deferred:** verifiers can re-derive; doc-only clarity gap.
- **Status:** `resolved (2026-05-13 survey verified moot) — CHANGELOG.md v0.2 section (lines 1296-1301) already includes the reproduction command \`shasum -a 256 ... | sort | shasum -a 256\` with explicit "(resolves v0.1 FOLLOWUPS N-1)" attribution; this FOLLOWUPS entry's status update was missed in the v0.2 cycle.`
- **Tier:** `v0.1-nice-to-have`

### `cli-mode-violations-byte-exact-naming` — test names say "byte_exact" but use `str::contains`

- **Surfaced:** Phase 5 review r1 (N-3).
- **Where:** `crates/mnemonic-toolkit/tests/cli_mode_violations.rs`.
- **What:** Several test names use the suffix `_byte_exact` but the assertions use `predicate::str::contains(...)` (substring match). Tests are correct; naming overstates assertion strictness. Either rename to `_substring` or tighten the assertions to full-stderr equality (and pin the byte-exact stderr in fixtures).
- **Why deferred:** assertion strength is sufficient for current SPEC pinning; naming is the only mismatch.
- **Status:** `resolved (2026-05-13 survey verified moot) — \`tests/cli_mode_violations.rs\` no longer exists; the \`_byte_exact\` naming pattern is now distributed across per-feature test files (cli_bundle_full, cli_export_wallet_*, etc.) where the original test's scope was absorbed.`
- **Tier:** `v0.1-nice-to-have`

### `phase-2-review-byte-determinism-blind-spot` — process: byte-determinism invariants need a spike, not just a review

- **Surfaced:** Phase 5 implementer caught the bug; Phase 2 r1 + r2 reviews missed it.
- **Where:** Process / `feedback_spike_before_locking_wire_format` memory rule.
- **What:** Phase 2 reviews looked at code correctness against SPEC §4 but didn't run encode twice and diff the bytes. The result: `mk_codec::encode` drew `chunk_set_id` from CSPRNG, which broke v0.1's byte-reproducible-output contract. The fix (`derive_mk1_chunk_set_id` + `encode_with_chunk_set_id`) shipped in the Phase 5 release commit (`f2bd20a`). Process improvement: when a phase locks wire-format invariants that downstream phases will SHA-pin, the per-phase review checklist should include "encode twice, assert identical bytes".
- **Why deferred:** post-mortem item; resolved via the v0.1.0 release fix. Lesson worth carrying forward.
- **Status:** `resolved f2bd20a — Phase 5 fix shipped; process lesson captured here.`
- **Tier:** `v0.1-nice-to-have`

### `mk1-bip-chunk-set-id-determinism-guidance` — mk1 BIP recommendation for deterministic encoders

- **Surfaced:** Phase 5 byte-determinism fix (`f2bd20a`) — the toolkit-side derivation needs lifting into the mk1 BIP so other implementations producing reproducible corpora reach the same wire bits. Companion: same-id entry in `mnemonic-key/bip/bip-mnemonic-key.mediawiki`.
- **Where:** `bip/bip-mnemonic-key.mediawiki` String-layer header section in `mnemonic-key`.
- **What:** Toolkit shipped a `derive_mk1_chunk_set_id(&policy_id_stub)` helper deriving 20 bits from the leading bytes of the policy_id_stub. mk1 BIP edited to recommend this pattern (with the explicit formula `(stub[0] << 12) | (stub[1] << 4) | (stub[2] >> 4)`) and clarify decoders MUST accept any 20-bit value.
- **Why deferred:** mk1 BIP is a sibling-repo asset; toolkit's fix landed first.
- **Status:** `resolved 87bbc11 (mnemonic-key@main) — mk1 BIP §"String-layer header" updated 2026-05-04 with deterministic-encoder guidance + decoder-acceptance clarification. Pushed to bg002h/mnemonic-key.`
- **Tier:** `cross-repo`

### `dead-assert-tautological` — `synthesize.rs` invariant 1 debug-assert is tautological by construction

- **Surfaced:** v0.1 audit 2026-05-05 (LOW-1).
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs:99` (`debug_assert_eq!(&card.policy_id_stubs[0], &stub)`).
- **What:** `stub` is computed from `policy_id.as_bytes()[..4]` and immediately passed as `policy_id_stubs[0]`. The assertion can never fail at the construction site. Phase 2 r1 originally flagged this as L-4. Pre-existing; meaningful assertion is invariant 2 (`is_wallet_policy()`).
- **Why deferred:** v0.2 multisig will need a meaningful assertion that loops over all per-cosigner stubs; resolve as part of v0.2 Phase C.
- **Status:** `resolved (this commit, 2026-05-13) — tautological debug_assert_eq removed from both single-sig synthesize paths in synthesize.rs (now lines 139, 171); the meaningful is_wallet_policy assert is retained. v0.2 multisig's proper looped assertion never materialized; removing the dead code rather than waiting indefinitely.`
- **Tier:** `v0.2`

### `dead-inner-guard-bundle-watch-only` — redundant `--xpub`-needs-`--master-fingerprint` guard inside `bundle_watch_only`

- **Surfaced:** v0.1 audit 2026-05-05 (LOW-2).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs:200` (inside `bundle_watch_only`).
- **What:** A redundant guard exists that would emit `BadInput` (exit 1) if `--master-fingerprint` is missing. Unreachable in practice — the mode-violation pre-check at `cmd/bundle.rs:93` rejects the same condition earlier with exit 2 + byte-exact §6.6 text. Future-refactor inconsistency risk.
- **Why deferred:** not currently triggered; v0.2 will refactor mode dispatch and naturally clean this up.
- **Status:** `resolved` — resolved-by-refactor `bc59ee3` (v0.4.2 Phase M.3 deleted `bundle_watch_only` + its ~990 lines of legacy CLI dispatch). The redundant guard no longer exists — exactly the refactor the entry predicted. (cycle-prep recon 2026-05-22, SHA `1d6436d`.)
- **Tier:** `v0.2`

### `friendly-mapper-unit-test-gaps` — friendly-mapper unit tests cover only 3 of ~70 match arms

- **Surfaced:** v0.1 audit 2026-05-05 (LOW-3).
- **Where:** `crates/mnemonic-toolkit/src/friendly.rs::tests`.
- **What:** Unit tests cover `friendly_bip39::UnknownWord`, `friendly_ms_codec::WrongHrp`, `friendly_mk_codec::PathTooDeep`. Untested at unit level: 4 of 5 `friendly_bip39`, all 3 `friendly_bitcoin`, 8 of 9 `friendly_ms_codec`, 21 of 22 `friendly_mk_codec`, all 41 `friendly_md_codec`. Integration tests likely exercise some paths end-to-end but unit isolation is thin.
- **Why deferred:** v0.2 will add new error paths through these mappers; expand the tests in lockstep with v0.2 Phase E.
- **Status:** `open`
- **Tier:** `v0.2-nice-to-have`

### `hex-dep-unused` — `hex = "0.4"` declared in Cargo.toml but unused in non-test source

- **Surfaced:** v0.1 audit 2026-05-05 (LOW-4).
- **Where:** `crates/mnemonic-toolkit/Cargo.toml:27`.
- **What:** No `use hex` statement in any source module. Inert dependency carried from ms-cli precedent or SPEC §10.3 dep list.
- **Why deferred:** user's `feedback_dont_drop_reserved_deps` rule applies — confirm with user before removal. v0.2 may use `hex` for new error-message formatting (e.g., printing fingerprints in mode-violation output), in which case the dep activates naturally.
- **Status:** `resolved` — obsolete. `hex` is now used in non-test source across `bundle.rs` (`:565,:1286`), `convert.rs` (`:1132,:1164,:1415,:1520`), `import_wallet.rs` (`:2146,:2183,:2338`), and `nostr.rs` (`:50`, v0.34.0); the "unused" premise is false and removing the dep would break the build. (Citation `Cargo.toml:27` drifted to `:40`.) cycle-prep recon 2026-05-22, SHA `1d6436d`.
- **Tier:** `v0.1-nice-to-have`

### `parse_template-regex-line-ref` — SPEC v0.3 §4.9 step 2 cites wrong line range

- **Surfaced:** v0.3 SPEC architect review r2 2026-05-05.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_3.md` §4.9 step 2.
- **What:** Step 2 cites `descriptor-mnemonic/crates/md-cli/src/parse/template.rs:19-27` for the placeholder regex; the actual `Regex::new` call is at `:25-27` (line 19-24 are imports/doc-comments). Docs-only nit — implementation will read the actual regex from the source.
- **Why deferred:** non-blocking; can be patched alongside any v0.3 SPEC revision.
- **Status:** `resolved (this commit, 2026-05-13) — §4.9 step 2 line range updated to \`parse/template.rs:25-27\`.`
- **Tier:** `v0.3-nice-to-have`

### `unsupported-fragment-error-style` — SPEC v0.3 §6.8 error message text is verbose

- **Surfaced:** v0.3 SPEC architect review r2 2026-05-05.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_3.md` §6.8 (error message wording).
- **What:** The message reads `unsupported miniscript fragment: <fragment-string>; v0.3 walker covers BIP-388 surface modulo multi-leaf tap trees (deferred to v0.4)`. This is verbose for a CLI error; a tighter form (e.g. drop the parenthetical) would be friendlier.
- **Why deferred:** SPEC pins the message for byte-exactness; can be revisited at impl time if friendlier wording surfaces. Not blocking.
- **Status:** `open`
- **Tier:** `v0.3-nice-to-have`

### `walker-backport-to-md-cli` — toolkit's expanded walker should be backported to md-cli

- **Surfaced:** v0.3 SPEC architect review r2 2026-05-05.
- **Where:** cross-repo: `mnemonic-toolkit/crates/mnemonic-toolkit/src/parse_descriptor.rs` ↔ `descriptor-mnemonic/crates/md-cli/src/parse/template.rs`.
- **What:** v0.3 toolkit ships an expanded `walk_miniscript_node` covering all 24 v0.3-NEW `Terminal` arms (hash terminals, timelocks, wrappers, AND/OR/Thresh). md-cli's walker is the inspiration but currently rejects all of these. Backporting (or extracting both into a shared crate `descriptor-walker`) avoids divergence.
- **Why deferred:** scope of v0.3 is toolkit-only by user direction. Cross-repo coordination cycle in v0.4.
- **Status:** `open`
- **Tier:** `v0.4-cross-repo`

### `spike-report-citation` — v0.3 SPEC §9 Q2 closure should cite SPIKE report

- **Surfaced:** v0.3 SPEC architect review r2 2026-05-05.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_3.md` §9 Q2 closure.
- **What:** §9 Q2 declared "moot — v0.3 implements its own walker arms for hash terminals." Pre-Phase-A SPIKE produced `design/agent-reports/spike-toolkit-v0_3-pre-phaseA.md` §2 confirming hash-terminal round-trip. §9 Q2 updated to cite the report.
- **Status:** `resolved 2026-05-05` (closed inline with SPIKE report patches).
- **Tier:** `v0.3`

### `synthesize-descriptor-fn-naming` — single-vs-split synthesize entry-point decision

- **Surfaced:** v0.3 SPEC § resolved at IMPLEMENTATION_PLAN drafting 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs` (Phase C of v0.3 plan).
- **What:** v0.3 SPEC §10 originally named `synthesize_descriptor_full` / `synthesize_descriptor_watch_only` (mirroring v0.2's two-function shape). v0.3 plan resolves to a single `synthesize_descriptor` entry point that dispatches single-sig vs multisig internally. This is slightly asymmetric with v0.2's pattern.
- **Why deferred:** flagged for Phase C reviewer to confirm the single-entry-point shape doesn't regress code clarity. Not a blocker.
- **Status:** `resolved by IMPLEMENTATION_PLAN_v0_3 Phase C.1` (single entry point chosen)
- **Tier:** `v0.3`

### `v0.2-spec-§8-tier-citation` — v0.3 SPEC §8 citation against v0.2 SPEC §8

- **Surfaced:** v0.3 SPEC architect review r3 2026-05-05.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_3.md` §8 deferred-items table (K-of-N row).
- **What:** §8 cites v0.2 tier of K-of-N share encoding as "v0.3 (gates on ms-codec v0.2)". Verify against v0.2 SPEC §8 verbatim language at impl time for citation accuracy.
- **Why deferred:** non-blocking; doc-only.
- **Status:** `resolved (2026-05-13 survey verified) — v0.3 SPEC §8 line 313 cites v0.2 tier of K-of-N as "v0.3 (gates on ms-codec v0.2)", matching v0.2 SPEC §8 line 582 outcome wording verbatim. Citation is accurate.`
- **Tier:** `v0.3-nice-to-have`

### `ctx-for-descriptor-heuristic-misroutes` — Phase A `ctx_for_descriptor` is string-prefix heuristic

- **Surfaced:** v0.3 Phase A end-of-phase architect review I-2 (2026-05-05).
- **Resolved:** v0.3 Phase C end-of-phase r1 (2026-05-05). Replaced string-prefix heuristic with post-resolve n-based classification inside `parse_descriptor`: `n == 1 → SingleSig`, `n ≥ 2 → MultiSig`. The dead `ctx_for_descriptor` function was removed.
- **Status:** `resolved by Phase C.6 r2 (2026-05-05)`
- **Tier:** `v0.3`

### `parse-descriptor-allow-dead-code-audit` — module-level `#![allow(dead_code)]` audit

- **Surfaced:** v0.3 Phase A end-of-phase architect review L-1 (2026-05-05).
- **Resolved:** v0.3 Phase C end-of-phase r1 (2026-05-05). Lifted module-level `#![allow(dead_code)]`. Two items remained dead at the binary-compile boundary (`DescriptorMode` enum + `determine_mode` fn, used only in tests + Phase D verify-bundle re-parse path); both received per-item `#[allow(dead_code)]`.
- **Status:** `resolved by Phase C.6 r2 (2026-05-05)`
- **Tier:** `v0.3`

### `descriptor-mode-engraving-card` — engraving card omitted in descriptor mode

- **Surfaced:** v0.3 Phase C end-of-phase architect review L-5 (2026-05-05).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs` `descriptor_mode_emit` (Phase C.6).
- **What:** `engraving_card: None` for descriptor mode. The existing `engraving_card()` builder takes a `CliTemplate` + path-family + `EngravingMode`, which descriptor mode lacks. v0.3 ships without a descriptor-mode card; v0.4 should add a descriptor-aware engraving card (custom text including the descriptor string + per-cosigner xpub origins).
- **Why deferred:** out of v0.3 scope; engraving card logic is template-coupled.
- **Status:** `open`
- **Tier:** `v0.4`

### `engraving-card-unified-1-master-card` — Phase E unified engraving card deferred from v0.4.0 to v0.4.1

- **Surfaced:** v0.4 Phase E scope decision 2026-05-05 (autonomous-mode time risk).
- **Where:** `crates/mnemonic-toolkit/src/format.rs::engraving_card` + `EngravingMode` enum + `crates/mnemonic-toolkit/src/cmd/bundle.rs` per-mode card emit sites.
- **What:** SPEC §5.5 specifies a single unified `BundleInputForCard` shape + `engraving_card_unified` render function emitting one master card per bundle (in place of v0.2/v0.3's per-mode `EngravingMode` variants). Phase E was originally scoped to land this in v0.4.0 with deprecation of `EngravingMode::*`; deferred to v0.4.1 because it is tightly coupled to the BundleJson schema-4 cutover and the multi-source synthesis path (the unified card needs `MsField` + per-slot blocks). Will land in lockstep with `bundle-json-schema-4-cutover`.
- **Why deferred:** scope-coupling to schema-4 cutover; foundation-only Phase D made standalone Phase E delivery low-value.
- **Status:** `open`
- **Tier:** `v0.4.1`

### `verify-bundle-9-3plus6n-forensics` — Phase G verify-bundle 9/3+6N parity + per-cell forensics deferred from v0.4.0 to v0.4.1

- **Surfaced:** v0.4 Phase G scope decision 2026-05-05 (autonomous-mode time risk).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run_*` + `crates/mnemonic-toolkit/src/format.rs::VerifyCheck`.
- **What:** SPEC §5.7 specifies (a) descriptor-mode emits the same 9 / 3+6N check schema as template-mode (replacing v0.3's 3-element coarse ladder), (b) `VerifyCheck` gains four forensic fields (`expected`, `actual`, `diff_byte_offset`, `decode_error`), (c) verify-bundle dispatches on `schema_version` for schema-4 BundleJson with per-slot `MsField` array. All three sub-deliverables depend on the schema-4 cutover landing first. Bip388-distinctness symmetric enforcement (SPEC §4.11.c) IS shipping in v0.4.0 (Phase A wired `Bip388VerifyDistinctness` into `descriptor_mode_verify_run`).
- **Why deferred:** depends on `bundle-json-schema-4-cutover`; will land in lockstep with v0.4.1.
- **Status:** `open`
- **Tier:** `v0.4.1`

### `bundle-json-schema-4-cutover` — full BundleJson schema-4 cutover deferred from v0.4.0 to v0.4.1

- **Surfaced:** v0.4 Phase D scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/format.rs::BundleJson` + `crates/mnemonic-toolkit/src/cmd/bundle.rs::emit` + `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` + `crates/mnemonic-toolkit/src/synthesize.rs::Bundle`.
- **What:** v0.4.0 ships the `MsField = Vec<String>` type alias + multi-source synthesis primitives as a foundation, but DEFERS the full `BundleJson.ms1: Option<String>` → `ms1: MsField` migration + `schema_version: "3" → "4"` bump + verify-bundle schema-4 dispatch to v0.4.1. v0.4.0 retains the schema-3 envelope so all existing v0.2/v0.3 fixtures + JSON integration tests pass byte-identically. v0.4.1 lands the cutover with: (a) BundleJson.ms1 → MsField; (b) Bundle.ms1 → Vec<String>; (c) all integration test JSON assertions updated; (d) verify-bundle schema_version dispatch (read schema_version FIRST per SPEC §5.6); (e) regenerate or update v0.2/v0.3 carry-forward tests under the new envelope shape per SPEC §5.6 cross-schema invariant; (f) synthesize_multisig_multisource + synthesize_multisig_hybrid wired into bundle::run via BundleMode dispatch (Phase C foundation already in place); (g) **bundle::run top-level dispatch rewiring**: in v0.4.0 `args.slot` is parsed by clap into `BundleArgs.slot: Vec<SlotInput>` but `bundle::run` itself never reads it. v0.4.1 must wire `expand_legacy_to_slots(args.slot, ...)` → `validate_slot_set(&slots)?` → `detect_bundle_mode(&slots)?` → match-arm dispatch into the new `synthesize_multisig_multisource` / `synthesize_multisig_hybrid` paths AND rewrite the legacy `bundle_full` / `bundle_watch_only` / `bundle_multisig_*` calls to flow through the same SlotInput-driven path. This is a top-level surgery in `cmd/bundle.rs::run` itself, not just additions to the synthesis helper crate.
- **Why deferred:** scope risk in autonomous v0.4.0 release window — full surgery touches ≥10 source files + ~15 test assertions + fixture envelopes; landing without user oversight risks bugs the foundation-only approach avoids.
- **Status:** `open`
- **Tier:** `v0.4.1`

### `bip388-distinctness-path-normalization-phase-b-decision` — typed-vs-raw path semantics in check_key_vector_distinctness

- **Surfaced:** v0.4 Phase A end-of-phase architect review L-1 (2026-05-05).
- **Where:** `crates/mnemonic-toolkit/src/parse_descriptor.rs:1049` (`check_key_vector_distinctness`); SPEC `design/SPEC_mnemonic_toolkit_v0_4.md` §4.11.b.
- **What:** Phase A compares `cs[i].path.to_string()` on typed `bitcoin::bip32::DerivationPath`. The bitcoin library normalizes `48h/0h/0h/2h` ↔ `48'/0'/0'/2'` at `from_str` time, so collision detection is normalization-aware. SPEC §4.11.b says "raw user-supplied path string ... no path canonicalization". In Phase A this is safe because all paths arrive through the typed lex/cosigner parser; in Phase B the `--slot @N.path=` raw string flows into the binding directly. Phase B must lock whether `CosignerKeyInfo.path` stores typed `DerivationPath` (normalizing) or raw `String` (preserving), then update SPEC §4.11.b's normalization-domain paragraph in lockstep.
- **Why deferred:** Phase A's typed approach is correct under the v0.3 binding model; the decision is a Phase B design choice (slot input parsing).
- **Status:** `resolved by v0.5.0 Phase C.1 (commit 4a650aa) — typed DerivationPath equality replaces raw-string in check_key_vector_distinctness`
- **Tier:** `v0.4-nice-to-have`

### `verify-bundle-helper-and-full-forensics-rollout-v0.4.4` — Phase P.1-P.5 deferred from v0.4.3 to v0.4.4 — SUPERSEDED

- **Status:** `superseded by verify-bundle-helper-call-sites-rollout-v0.4.5 (2026-05-06)`. v0.4.4 P.1+P.2 landed the `emit_verify_checks` helper foundation (#[allow(dead_code)] with 4 unit tests + SuppliedCards struct + watch-only short-circuit + multisig TODO stub). The ~78-site call-site refactors (run_full / run_multisig / descriptor_mode_verify_run consolidation + descriptor-mode 9/3+6N parity + watch-only test migration) deferred again to v0.4.5 per the v0.4.4 plan scope reduction.

- **Surfaced:** v0.4.3 Phase P scope decision 2026-05-06 (P.0 struct shape correction landed; P.1-P.5 deferred for scope safety).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (~78 VerifyCheck push sites + new `emit_verify_checks` helper + descriptor-mode 9/3+6N parity refactor).
- **What:** v0.4.3 P.0 corrected the VerifyCheck struct shape per SPEC §5.7 (`result: &'static str` → `passed: bool`). The full SPEC §5.7 rollout — `emit_verify_checks` helper + refactor of run_full / run_multisig / descriptor_mode_verify_run + per-cell forensic field population at every push site + descriptor-mode 9/3+6N parity (closes `verify-bundle-9-3plus6n-descriptor-mode-parity` simultaneously) + skipped-check decode_error population — is deferred to v0.4.4. v0.4.3 ships passing checks with `passed: false` set on failures but forensic fields (expected/actual/diff_byte_offset/decode_error) only populated at the one v0.4.1 J.7 proof-of-shape site.
- **Why deferred:** scope-safety in v0.4.3 release window. Full helper + refactor estimated at ~800-1000 lines deleted in verify_bundle.rs alongside ~70 push-site updates.
- **Tier:** `v0.4.4`

### `verify-bundle-multisig-helper-full-mode-unit-test` — add unit-level coverage for emit_multisig_checks full-mode ms1 branch

- **Surfaced:** v0.4.5 final cross-phase review I-1 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` helper_tests mod.
- **What:** v0.4.5 ships `helper_multisig_watch_only_emits_3plus6n_checks_in_spec_order` (renamed from `_full_` after review confirmed the fixture exercises watch-only synthesis with empty `expected.ms1`). The full-mode multisig ms1 branch (`emit_multisig_checks` lines ~1096-1159: substantive ms1_decode + ms1_entropy_match per cosigner) has end-to-end coverage via `cli_bundle_multisig.rs` integration tests but no isolated unit-level test. Add a companion `helper_multisig_full_emits_3plus6n_checks_in_spec_order` that uses `synthesize_multisig_full` (or constructs a synthetic Bundle with non-empty `expected.ms1` strings) to exercise the substantive ms1 path.
- **Why deferred:** integration coverage is sufficient for v0.4.5; the unit-level gap is test isolation hygiene, not behavior.
- **Status:** `resolved by v0.5.0 Phase B.1 (commit 9f1a4e7) — helper_multisig_full_emits_3plus6n_checks_in_spec_order added`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-multisig-positional-fallback-condition-cosmetic` — cosmetic dead `unwrap_or(false)` in card_for_cosigner positional fallback

- **Surfaced:** v0.4.5 final cross-phase review L-2 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` (`card_for_cosigner` positional fallback condition).
- **What:** Condition `supplied_md_decoded.is_err() || supplied_md_decoded.as_ref().map(|d| d.tlv.pubkeys.is_none()).unwrap_or(false)` — the `.map().unwrap_or(false)` chain is unreachable when `supplied_md_decoded.is_err()` short-circuits OR semantically dead inside the Ok branch. Refactor to `match` for clarity.
- **Why deferred:** cosmetic; no logic impact.
- **Status:** `resolved by v0.5.0 Phase B.2 (commit 9f1a4e7) — refactored to clean match expression`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-multisig-md1-xpub-match-set-equality` — md1_xpub_match uses ordered Vec equality

- **Surfaced:** v0.4.5 Phase P.4 review I-1 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` (md1_xpub_match arm).
- **What:** Helper compares `expected_md1.tlv.pubkeys` and `supplied_md1.tlv.pubkeys` as ordered `Vec<[u8; 65]>` via `==`. SPEC §5.7 line 103 says the shared `md1_xpub_match` confirms "all N pubkeys match expected" — semantics are arguably set-equality (the script-level pubkey set must be identical), not ordered. Template-mode synthesis preserves cosigner-index order, so ordered equality is correct for that path. Descriptor-mode verify-bundle (P.5) where the user supplies a descriptor with arbitrary `@N` placement could false-fail under ordered equality even when the logical pubkey set is identical.
- **Why deferred:** template-mode P.4 doesn't trigger this; descriptor-mode P.5 lands in v0.4.5 but the SPEC clarification needed to choose set-vs-ordered semantics is itself open. Re-evaluate after P.5 implementation surfaces real-world cases.
- **Status:** `resolved by v0.5.0 Phase B.3 (commit 9f1a4e7) — sort-then-compare multiset equality`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-multisig-cosigner-mapping-diagnostic` — distinguish "card not supplied" from "xpub not in policy"

- **Surfaced:** v0.4.5 Phase P.4 review I-2 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` (`card_for_cosigner` mapping + `mk1_decode[i]` emission).
- **What:** When supplied md1 decodes successfully and pubkeys-TLV is present but a supplied mk1 card's xpub matches no entry, `card_for_cosigner[i]` stays `None` and `mk1_decode[i]` emits "skipped: mk1[i] not supplied or decode failed". This conflates two distinct failure modes:
  1. User forgot to supply --mk1 for cosigner i.
  2. User supplied an mk1 card whose xpub doesn't appear in the descriptor's pubkey set (wrong-key attack scenario).
- **Why deferred:** diagnostic clarity, not correctness. Could split into two distinct check names or add a per-card "policy-membership" field.
- **Status:** `resolved by v0.5.0 Phase B.4 (commit 9f1a4e7) — MappingFailure enum with precedence XpubNotInPolicy > DecodeFailed > NotSupplied`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-multisig-missing-ms1-passes-true` — full-mode multisig with no --ms1 supplied reports passed=true

- **Surfaced:** v0.4.5 Phase P.4 review N-1 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` ("Expected substantive but supplied missing/empty" branch).
- **What:** When `expected.ms1[i]` is non-empty (full-mode) but the caller supplies no corresponding --ms1 value, `ms1_decode[i]` and `ms1_entropy_match[i]` are emitted with `passed: true, decode_error: "skipped: ms1[i] not supplied"`. A full-mode multisig bundle verified without supplying any ms1 cards thus reports `result: ok` if mk1+md1 match. SPEC §5.7 line 104 specifies "skipped: watch-only slot" semantics ONLY for `ms1[i] == ""` (watch-only sentinel); the missing-but-expected case is unspecified.
- **Why deferred:** policy decision — should missing-but-expected ms1 be a hard fail (like missing mk1[i])? Or stays as soft skip (current behavior)? Defer for SPEC clarification.
- **Status:** `resolved by v0.5.0 Phase B.5 (commit 9f1a4e7) — SPEC §5.7 four-case table, case 4 passed=false on missing-but-expected ms1`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-watch-only-spurious-ms1-handling` — watch-only with user-supplied --ms1 produces ms1_entropy_match: fail

- **Surfaced:** v0.4.5 Phase P.3 review L-1 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run_watch_only` + `emit_verify_checks` watch-only short-circuit.
- **What:** Pre-v0.4.5 `watch_only_checks` ignored `args.ms1` (always emitted "watch-only mode: no entropy known to toolkit" passing-vacuously). Post-v0.4.5 P.3 wire-up: run_watch_only synthesizes the watch-only Bundle (`ms1: vec![""]`) and the helper compares supplied vs expected. If user spuriously supplies `--ms1 <non-empty>` in watch-only mode, `ms1_decode` runs against the supplied string, then `ms1_entropy_match` fails because `expected="" ≠ supplied=non-empty`. Behavior change vs v0.4.4: arguably more useful (tool flags the user's mistake) but not formally specified.
- **Why deferred:** non-blocking; SPEC §5.7 doesn't address this edge. Decide whether to short-circuit in run_watch_only (ignore args.ms1, force-empty SuppliedCards.ms1) or document the behavior in SPEC §2.2.2.
- **Status:** `resolved by v0.5.0 Phase C.2 (commit 4a650aa) — SPEC §5.7 case 1 codification + integration test`
- **Tier:** `v0.4-nice-to-have`

### `verify-bundle-helper-foundation-cleanup-v0.4.5` — 2 Low/Nit cleanups from v0.4.4 final cross-phase review

- **Surfaced:** v0.4.4 final cross-phase review 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_verify_checks` (and surrounding helper code).
- **What:**
  - **L-1** — Doc-comment in `emit_verify_checks` cites SPEC §5.8 for the watch-only sentinel discrimination (`expected.ms1[i].is_empty()`); the watch-only short-circuit logic actually lives in §5.7. §5.8 is the MsField wire-format definition. Fix: change `§5.8` → `§5.7` in the doc-comment near `verify_bundle.rs:1882`.
  - **L-2** — `MkField::Multi` arm in single-sig branch returns early with potentially fewer than 9 checks; this path is unreachable in production (single-sig bundles always have `MkField::Single`) and is documented with a comment, but the early return is an implicit invariant assumption. Fix: replace early return with `unreachable!("single-sig branch reached MkField::Multi — invariant violation")` or `debug_assert!(false, ...)`. Land alongside P.3 wiring in v0.4.5.
- **Why deferred:** non-blocking nits; helper is `#[allow(dead_code)]` so no runtime exposure. Bundle with the v0.4.5 P.3-P.7 call-site rollout.
- **Status:** `resolved by v0.4.5 Phase L (commit 40638c8)` — L-1 §5.7 cited; L-2 `unreachable!()` invariant assertion in place.
- **Tier:** `v0.4.5`

### `verify-bundle-helper-call-sites-rollout-v0.4.5` — Phase P.3-P.7 call-site rollout deferred from v0.4.4 to v0.4.5

- **Surfaced:** v0.4.4 Phase P scope decision 2026-05-06 (P.1+P.2 helper foundation landed; P.3-P.7 deferred for scope safety).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run_full` + `run_multisig` + `descriptor_mode_verify_run` + `crates/mnemonic-toolkit/tests/watch_only_tests.rs` + new integration tests for full forensic rollout.
- **What:** v0.4.4 P.1+P.2 shipped `emit_verify_checks` (single-sig 9-check shape per SPEC §5.7 ordering), `SuppliedCards<'a>` struct, `emit_md1_checks` shared md1 helper, watch-only short-circuit (passed=true + decode_error="skipped: watch-only slot"), multisig TODO stub returning `[VerifyCheck { name: "TODO_multisig_v0_4_5", passed: false, ... }]`, and 4 helper unit tests. The helper is `#[allow(dead_code)]`; v0.4.5 wires it up:
  - **P.3** — `run_full` (single-sig template-mode) calls `emit_verify_checks(SuppliedCards::singlesig(...), false)` and replaces ~30 push sites.
  - **P.4** — `run_multisig` (template-mode multisig) replaces TODO stub with the 3-shared-checks + 6N-per-cosigner pattern; emits real forensics.
  - **P.5** — `descriptor_mode_verify_run` emits the 9 / 3+6N schema (closes `verify-bundle-9-3plus6n-descriptor-mode-parity`) via the helper.
  - **P.6** — `watch_only_tests.rs` migrates to the new shape (`passed` + forensic field assertions).
  - **P.7** — Add integration tests for full forensic field population: tampered-cell roundtrips that assert `expected`/`actual`/`diff_byte_offset` populated; skipped checks assert `decode_error` populated.
- **Why deferred:** scope-safety in v0.4.4 release window. The helper-foundation pattern is the right shape; consolidating ~78 call sites at the same time was estimated at ~800-1000 lines deleted plus ~70 push-site updates and risked release timeline.
- **Status:** `resolved by v0.4.5 commits 679ded7 (P.3+P.6) + d3207dd (P.4) + 57f62eb (P.5) + 40638c8 (L+P.7)` — all 5 sub-phases shipped; net cmd/verify_bundle.rs delete ~660 lines; 3 forensic integration tests added.
- **Tier:** `v0.4.5`

### `verify-bundle-emit-checks-helper-and-full-forensics-rollout` — Phase J.2 + J.3 + full forensic field rollout deferred from v0.4.1 to v0.4.2 — SUPERSEDED

- **Status:** `superseded by verify-bundle-helper-and-full-forensics-rollout-v0.4.4 (2026-05-06)`. v0.4.3 P.0 landed the struct shape correction; the helper + full rollout deferred again to v0.4.4.

- **Surfaced:** v0.4.1 Phase J scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (~78 VerifyCheck push sites) + new `emit_verify_checks` helper.
- **What:** v0.4.1 ships the structural pieces of SPEC §5.7: VerifyCheck struct gains `expected` / `actual` / `diff_byte_offset` / `decode_error` Option fields with Default impl + serde skip_serializing_if (J.1), and the `--ms1` CLI repeating-flag migration (J.5). Forensic fields are populated on ONE prominent failure path (descriptor-mode `ms1_entropy_match` mismatch — proof-of-shape in cmd/verify_bundle.rs:1456-1469); the remaining ~70 push sites continue to default to `None` for forensic fields. The `emit_verify_checks` helper (J.2) and the run_full / run_multisig / descriptor_mode_verify_run refactor (J.3) to use it are deferred. Full per-cell forensics rollout requires the helper to land first; otherwise duplicating the population logic at every push site is unmaintainable.
- **Why deferred:** scope-safety in v0.4.1 release window. The 78-site refactor is mechanical but error-prone; helper-first approach is the right shape and lands cleanly in v0.4.2.
- **Status:** `open`
- **Tier:** `v0.4.2`

### `verify-bundle-9-3plus6n-descriptor-mode-parity` — Phase G/J descriptor-mode 9/3+6N parity deferred from v0.4.1 to v0.4.2

- **Surfaced:** v0.4.1 Phase J scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::descriptor_mode_verify_run`.
- **What:** SPEC §5.7 specifies descriptor-mode verify-bundle emits the same 9 / 3+6N check schema as template-mode (replacing v0.3's 3-element coarse ladder). v0.4.1 retains the v0.3 coarse ladder (cmd/verify_bundle.rs:1361 onward) with the H.1 shim for the schema-4 ms1 vec. v0.4.2 lands the parity refactor atomically with the `emit_verify_checks` helper (FOLLOWUP `verify-bundle-emit-checks-helper-and-full-forensics-rollout`).
- **Why deferred:** depends on the helper; bundled with the same v0.4.2 cycle.
- **Status:** `resolved by v0.4.5 Phase P.5 (commit 57f62eb)` — descriptor_mode_verify_run dispatches to emit_verify_checks(... is_multisig: descriptor.n > 1); single-sig descriptors emit the 9 schema, multisig descriptors emit 3+6N.
- **Tier:** `v0.4.2`

### `legacy-cli-flag-deletion` — delete --phrase / --xpub / --cosigner / --master-fingerprint / --cosigner-count / --cosigners-file CLI flags entirely

- **Surfaced:** v0.4.2 cycle planning 2026-05-06 (user-confirmed during scope brainstorm).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::BundleArgs` + `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::VerifyBundleArgs`; consumer test files under `crates/mnemonic-toolkit/tests/`.
- **What:** v0.4.2 lands the unified `--slot @N.<subkey>=<value>` dispatch and routes legacy CLI flags through `expand_legacy_to_slots` (option (a) per the v0.4.2 brainstorm). v0.5 takes the next step: delete the legacy CLI flags entirely from `BundleArgs` + `VerifyBundleArgs`. Estimated cost: rewrite ~25 integration tests (~1500 lines of test churn) to use `--slot` syntax. The unified path itself is unchanged; only the CLI surface contracts.
- **Why deferred:** the user accepted the bigger v0.4.2 scope (legacy-flag-deprecation under option a) but routes the cleaner-CLI-surface end-state to v0.5 to amortize the test-rewrite churn against a separate cycle. Captured as a follow-on after v0.4.2 ships.
- **Status:** `resolved by v0.5.1 commit d782a2d` — 6 legacy fields deleted from both `BundleArgs` and `VerifyBundleArgs`; `bundle_args_to_slots` + `expand_legacy_to_slots` shims deleted; 9 mode-violation guards + 11 mode-text consts removed; 3 retained guards covered by new `cli_mode_violations_v0_5.rs`. `bundle::resolve_slots` refactored to take an explicit args-tuple + promoted to `pub(crate)`; `verify_bundle.rs` dispatch reshaped to consume slots. 13 consumer test files rewritten per the v0.5.0 mapping table.
- **Tier:** `v0.5.1`

### `engraving-card-unified-legacy-migration` — migrate 4 legacy engraving_card() call sites to engraving_card_unified

- **Surfaced:** v0.4.1 Phase I scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs` 4 legacy call sites (bundle_full, bundle_watch_only, bundle_multisig_full, bundle_multisig_watch_only) + `crates/mnemonic-toolkit/src/format.rs` legacy `engraving_card` + `EngravingMode` enum.
- **What:** v0.4.1 ships `engraving_card_unified` + `BundleInputForCard` per SPEC §5.5 and wires only the new `bundle_run_unified` (--slot-driven) path through it. Migrating the 4 legacy call sites to the unified card requires removing 3 byte-exact format.rs unit tests for `EngravingMode::*` variants and verifying integration tests still pass with the new card layout. v0.4.2 lands the migration + drops `EngravingMode`.
- **Why deferred:** scope-safety in v0.4.1 release window; legacy call sites work unchanged via the existing `engraving_card` function.
- **Status:** `resolved by v0.5.0 Phase A.3 (commit 456c878) — BundleJson.engraving_card field deleted; doc-comment rewritten`
- **Tier:** `v0.4.2`

### `unified-slot-xpub-missing-path-origin-path-null` — origin_path empty-string vs null divergence

- **Surfaced:** v0.4.1 Phase H r1 review L-1.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::resolve_slots` (xpub branch) + `emit_unified` (single-sig N=1 origin_path emission).
- **What:** When `--slot @0.xpub=X` is supplied without `--slot @0.path=`, `emit_unified` emits `"origin_path": ""` in the JSON envelope. Legacy `emit` for the equivalent `--xpub X` (no path) invocation emits `"origin_path": null`. SPEC §4.11.b defines `""` as the absent-path sentinel for collision purposes but does not govern the JSON envelope value. Two paths diverge for semantically equivalent inputs.
- **Why deferred:** non-blocking; tooling that reads the envelope can treat `""` and `null` as equivalent. v0.4.2 unifies emission to `null`.
- **Status:** `resolved by v0.5.0 Phase E (commit 990ccad) — origin_path_for_json helper emits null on empty path_raw`
- **Tier:** `v0.4.2-nice-to-have`

### `unified-slot-additional-subkey-shapes` — entropy / xprv / wif / partial-xpub-only resolution deferred from v0.4.1 to v0.4.2

- **Surfaced:** v0.4.1 Phase H.5 scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::resolve_slots`.
- **What:** v0.4.1's unified `--slot` dispatch (`bundle_run_unified`) supports two slot subkey shapes: `{phrase}` (BIP-39 → derived xpub) and `{xpub, fingerprint, path}` (watch-only with full origin metadata). The remaining SPEC §6.6.b shapes (`{entropy}` raw entropy → ms-codec ENTR; `{xprv}` xpriv-direct; `{wif}` degenerate single-key; `{xpub}` alone; `{xpub, fingerprint}`; `{xpub, path}`) return BadInput with a pointer to this FOLLOWUP. v0.4.2 lands the resolution logic for each shape + integration tests per shape.
- **Why deferred:** scope-safety in v0.4.1 release window; the two supported shapes cover the headline multi-source-secrets and watch-only-multisig use cases.
- **Status:** `open`
- **Tier:** `v0.4.2`

### `unified-slot-descriptor-mode-support` — descriptor mode under unified --slot dispatch deferred from v0.4.1 to v0.4.2

- **Surfaced:** v0.4.1 Phase H.5 scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::bundle_run_unified`.
- **What:** v0.4.1's unified `--slot` dispatch supports `--template` only; supplying `--descriptor` alongside `--slot` is rejected with a pointer to this FOLLOWUP. Legacy descriptor-mode dispatch (no `--slot`) continues to work via `descriptor_mode_run`. v0.4.2 unifies the two paths so `--slot` works with both `--template` and `--descriptor`, including descriptor-mode multi-source via per-`@N` slot binding.
- **Why deferred:** scope-safety; the legacy descriptor-mode path remains the recommended invocation for descriptor-driven workflows in v0.4.1.
- **Status:** `open`
- **Tier:** `v0.4.2`

### `descriptor-binding-entropy-field-redundant` — DescriptorBinding.entropy field is redundant after v0.4.3 N

- **Surfaced:** v0.4.3 Phase N (CosignerKeyInfo type alias merge) 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/parse_descriptor.rs::DescriptorBinding`.
- **What:** v0.4.3 N merged CosignerKeyInfo into ResolvedSlot via type alias; ResolvedSlot has per-slot `entropy: Option<Vec<u8>>`. The bundle-level `DescriptorBinding.entropy: Option<Vec<u8>>` field is now semantically redundant with `binding.cosigners[0].entropy`. v0.4.4 retires the field; ~10 call sites (parse_descriptor.rs tests, verify_bundle.rs, bundle.rs::bundle_run_unified_descriptor) update to read `binding.cosigners[0].entropy.as_deref()` instead.
- **Why deferred:** non-blocking; harmless redundancy.
- **Status:** `resolved by v0.4.4 Phase S (commit c99a78b)` — DescriptorBinding.entropy field deleted; `entropy_at_0()` helper method (Option<&[u8]>) reads `cosigners[0].entropy`; bind_full_mode sets `cosigners[0].entropy` before construction; all readers migrated; 244 tests pass.
- **Tier:** `v0.4.4`

### `bundle-json-cli-flag-and-dispatch` — `--bundle-json <file>` verify-bundle intake + schema-version dispatch

- **Surfaced:** v0.4.1 Phase J.4 scope decision 2026-05-05 (per impl plan r1 review I2).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::VerifyBundleArgs` + new JSON-intake handler.
- **What:** SPEC §6.7 reserves `--bundle-json <file>` as a verify-bundle flag for round-tripping a `bundle --json` envelope. v0.4.3 added the CLI flag + the `serde_json::Value` peek-then-typed-decode dispatch on `schema_version` (schema-4 only; schema-2/3 retro-compat tracked at NEW FOLLOWUP `bundle-json-schema-2-3-retro-compat` at v0.4.4+).
- **Status:** `resolved by v0.4.3 Phase Q (commit pending)` — clap flag with `conflicts_with_all = ["ms1", "mk1", "md1"]`; `load_bundle_json_into_args` synthesizes a VerifyBundleArgs with extracted card vecs; rest of run() unchanged. 3 integration tests in `cli_bundle_json_intake.rs`.
- **Tier:** `v0.4.2` (target met)

### `cosigner-keyinfo-resolved-slot-merge` — retire CosignerKeyInfo into ResolvedSlot

- **Surfaced:** v0.4.1 Phase H.6 (impl plan r1 review I1).
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs::CosignerKeyInfo` + `ResolvedSlot`.
- **What:** v0.4.1 carried two near-identical typed shapes; v0.4.3 N merged them via `pub type CosignerKeyInfo = ResolvedSlot;` alias. ResolvedSlot is now the sole binding type. CosignerKeyInfo retained as a #[allow(dead_code)] alias for source-compat.
- **Status:** `resolved by v0.4.3 Phase N (commit 25581f3)` — type alias merge; per-slot entropy lives on ResolvedSlot; legacy DescriptorBinding.entropy field retained but redundant (tracked at NEW FOLLOWUP `descriptor-binding-entropy-field-redundant` at v0.4.4).
- **Tier:** `v0.4.2` (target met)

### `bundle-json-schema-2-3-retro-compat` — `--bundle-json` schema-2/3 retro-compat intake

- **Surfaced:** v0.4.3 Phase Q scope decision 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::load_bundle_json_into_args`.
- **What:** v0.4.3 ships schema-4-only intake. Schema-2/3 envelopes (theoretical; no real-world bundles exist since v0.4.1) error with byte-exact stderr pointing at this FOLLOWUP. v0.4.4+ adds schema-2/3 typed dispatch IF a real-world need surfaces.
- **Why deferred:** speculative; no real bundles to consume.
- **Status:** `resolved by v0.5.0 Phase D (commit 6e4b87e) — placeholder rejection branch deleted; schema-mismatch fails at field extraction`
- **Tier:** `v0.4.4-nice-to-have`

### `wif-multisig-resolution` — wif slots in multisig contexts

- **Surfaced:** v0.4.2 Phase K.3 (single-sig-only guard introduced; multisig deferred).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::resolve_slots`.
- **What:** v0.4.3 R lifted the single-sig-only guard. Wif slots in multisig produce ResolvedSlots with the wif's pubkey + zero chain code + empty path. BIP-388 distinctness applies normally (same WIF twice → row 13 collision).
- **Status:** `resolved by v0.4.3 Phase R (commit 610bef6)` — 3 new integration tests cover hybrid 2-of-3 + pure 2-of-2 + same-WIF-twice collision.
- **Tier:** `v0.4.3` (target met)

### `legacy-flag-deprecation` — full migration of --phrase / --xpub / --cosigner to alias-only deferred from v0.4.1 to v0.5+

- **Surfaced:** v0.4.1 Phase H.5 scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::run` legacy dispatch path.
- **What:** SPEC §9 v0.4 promises that legacy `--phrase` / `--xpub` / `--cosigner` flags become deprecation aliases that auto-expand into `--slot` form. v0.4.1 ships unified `--slot` as opt-in alongside the unchanged legacy dispatch. v0.5+ (a future BREAKING release) deletes the legacy dispatch entirely and routes everything through `bundle_run_unified` via `expand_legacy_to_slots`.
- **Why deferred:** would force fixture regeneration of 16+ v0.1 byte-exact fixture files + v0.2 carry-forward fixtures; too large for v0.4.1 release window.
- **Status:** `resolved by v0.5.1 commit d782a2d` — superseded by `legacy-cli-flag-deletion`. Legacy dispatch path is deleted entirely; `--slot` is the sole input shape.
- **Tier:** `v0.5.1`

### `bundle-removed-subcommand-trap-positional-eq-bypass` — `bundle multisig-full=value` token bypasses pre-clap trap

- **Surfaced:** v0.4 Phase 2 SPIKE r1 architect review L-2 (2026-05-05).
- **Where:** Phase C.1 `detect_removed_subcommand` (locked SPIKE shape at `design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md` SPIKE-2).
- **What:** Trap matches `argv[i+1] == "multisig-full"` with exact string equality. A token like `multisig-full=value` would not match and would fall through to clap's generic "unexpected argument" error rather than the byte-exact §6.6 row 1 message. Positional args do not idiomatically take `=value` form in shells, so this is essentially theoretical.
- **Why deferred:** no realistic user invocation produces this argv shape; a post-trap fallback in clap already rejects with exit 2.
- **Status:** `resolved by v0.5.0 Phase C.3 (commit 4a650aa) — entire detect_removed_subcommand trap deleted`
- **Tier:** `v0.4-nice-to-have`

### `bundle-removed-subcommand-trap-double-dash-bypass` — `mnemonic bundle -- multisig-full` bypasses pre-clap trap

- **Surfaced:** v0.4 Phase 2 SPIKE r1 architect review L-3 (2026-05-05).
- **Where:** Phase C.1 `detect_removed_subcommand` (locked SPIKE shape at `design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md` SPIKE-2).
- **What:** With a `--` separator inserted between `bundle` and `multisig-full`, the trap reads `argv[i+1] == "--"` and skips. Clap then processes `multisig-full` as a positional after `--` and emits a generic "unexpected argument" error rather than the byte-exact §6.6 row 1 text. UX difference matters only if a user intentionally inserts `--` before a removed subcommand name — not a realistic migration-error path.
- **Why deferred:** vanishingly unlikely user error; clap's fallback still rejects with exit 2.
- **Status:** `resolved by v0.5.0 Phase C.4 (commit 4a650aa) — entire detect_removed_subcommand trap deleted`
- **Tier:** `v0.4-nice-to-have`

### `tr-sortedmulti-a-via-upstream` — toolkit-side resolved in v0.3.1; v0.3.2 is the cleanup release

- **Surfaced:** v0.3 pre-Phase-A SPIKE 2026-05-05 (`design/agent-reports/spike-toolkit-v0_3-pre-phaseA.md` §1).
- **Resolution timeline:**
  - 2026-04-03: rust-miniscript PR #910 ("Add support for sortedmulti_a") merged; closed issue #320.
  - 2026-04-04: PR #915 ("refactor: remove SortedMultiVec and use Terminal::SortedMulti") merged.
  - 2026-05-05: upstream search confirmed both PRs on master rev `95fdd1c5773bd918c574d2225787973f63e16a66`; no published crate release contains them.
  - 2026-05-05: v0.3.1 adopted via `[patch.crates-io] miniscript = { git = ..., rev = "95fdd1c..." }` after a read-only build experiment confirmed feasibility; walker refactored for the post-#915 API; SPEC §4.9.a Layer 1+2 patched; new `Terminal::SortedMulti` + `Terminal::SortedMultiA` arms added; wire-bit-identical regression test passes (descriptor-mode `tr(@0, sortedmulti_a(...))` md1 == template-mode `--template tr-sortedmulti-a` md1).
- **Where:** `crates/mnemonic-toolkit/Cargo.toml` (`[patch.crates-io]` entry); `crates/mnemonic-toolkit/src/parse_descriptor.rs` (walker arms); `descriptor-mnemonic/crates/md-cli/src/parse/template.rs` (md-cli still pre-#910 — separate FOLLOWUP `walker-backport-to-md-cli`).
- **Toolkit-side status:** `partially resolved by v0.3.1` — `tr(K, sortedmulti_a(...))` works end-to-end via the master `[patch]`. md-cli divergence is the remaining cross-repo concern (FOLLOWUP `walker-backport-to-md-cli`).
- **v0.3.2 cleanup release** (mechanical, when miniscript crates.io publishes a post-#910+#915 release):
  1. Drop the `[patch.crates-io]` entry from `Cargo.toml`.
  2. Bump `miniscript` version in `crates/mnemonic-toolkit/Cargo.toml` to the new release.
  3. Update CHANGELOG; tag `mnemonic-toolkit-v0.3.2`.
  4. No code, SPEC, or test changes expected — the patched master and the new published release should be wire-identical for the surface this toolkit uses.
  5. Watch via `gh api repos/rust-bitcoin/rust-miniscript/tags --jq '.[].name' | grep -E 'miniscript-(13\.[1-9]|14|15)'`.
- **Status:** `partially resolved by v0.3.1; v0.3.2 cleanup pending miniscript crates.io release`
- **Tier:** `v0.3.2` (toolkit-side; was `v0.4-cross-repo` until v0.3.1 shipped)

### `secret-on-stdout-warning-bundle-retrofit` — apply convert's §7 secret-on-stdout warning to bundle

- **Surfaced:** v0.6.0 SPEC architect review r1 C-2 + impl decision 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs` emit_unified ms1 emission paths.
- **What:** v0.6.0 introduces a stderr warning `"warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')"` when convert emits secret-bearing material to stdout (phrase/entropy/xprv/wif/ms1). The bundle subcommand also emits secret-bearing ms1 strings to stdout but does NOT have the warning. Retrofit for cross-tool consistency.
- **Why deferred:** convert was the natural place to introduce the convention (ad-hoc one-shot operations where stdout-redirect-discipline is most likely overlooked); bundle retrofit is a separate scope-bounded change.
- **Status:** `resolved 66ff7c0` (v0.6.1 Phase D — `bundle.rs::emit_unified` emits the warning when `Bundle::any_secret_bearing()` returns true; SPEC §5.5.a; +1 positive (text mode) +1 positive (JSON mode) +2 negative (watch-only single + multisig) test assertions).
- **Tier:** `v0.6.1`

### `convert-seed-and-raw-privkey-nodes` — add seed / raw_privkey / xprv-via-ms1 / seed-via-ms1 nodes to convert when ms-codec v0.2 ships

- **Surfaced:** v0.6.0 SPEC §1 deferral 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs::NodeType` + edge table + SPEC §1.
- **What:** ms-codec v0.1.0's `SEED`, `XPRV`, `PRVK` tags are `RESERVED_NOT_EMITTED_V01`. v0.6.0 SPEC §1 documents `seed` and `raw_privkey` as deferred-not-rejected nodes. When ms-codec ships v0.2 with the reserved tags activated, add these nodes + their edges to convert (and update SPEC §1 / §2 accordingly).
- **Why deferred:** upstream codec library limit; additive.
- **Status:** `open`
- **Tier:** `cross-repo`

### `convert-phrase-to-leaf-wif` — implement phrase/entropy → wif (path-to-leaf-WIF derivation)

- **Surfaced:** v0.6.0 SPEC §10 deferral 2026-05-06 + impl r1 review.
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` Phrase|Entropy arm.
- **What:** v0.6.0 SPEC §2 lists `phrase/entropy → wif` as not directly defined; impl returns `BadInput` with deferral message. Implementing requires a leaf-depth BIP-32 path (`m/<purpose>'/<coin>'/<account>'/<chain>/<index>`, depth 5) and serializing the leaf privkey to WIF. v0.6.1+ adds the missing edge.
- **Why deferred:** scope-safety in v0.6.0; the headline conversion graph nodes were prioritized.
- **Status:** `resolved 62b4f23` (v0.6.1 Phase B — SPEC-A in `SPEC_convert_v0_6.md` §2 + §8; sibling helper `derive_slot::derive_bip32_at_path` for path-driven derivation; `bitcoin::PrivateKey { compressed: true, network, inner: leaf_xpriv.private_key }.to_wif()`; explicit `--path` REQUIRED with byte-exact `ConvertRefusal` stderr (exit 2) when absent; `edge_uses_pbkdf2` extended to include `Wif` so `--passphrase` does not spuriously fire the ignored-warning).
- **Tier:** `v0.6.1`

### `convert-slip0132-prefix-support` — accept zpub/ypub on input + emit modes (consolidated v0.6.1)

- **Surfaced:** v0.6.0 post-release UX audit 2026-05-06 (user prompt about SLIP-0132 prefix interpretation).
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` (input normalization + new edge); possibly cross-cutting into `crates/mnemonic-toolkit/src/cmd/bundle.rs::resolve_slots` Xpub branch (if input normalization is repo-wide rather than convert-only).
- **What:** SLIP-0132 (`ypub`/`Ypub`/`zpub`/`Zpub` mainnet, plus `tpub`/`upub`/`vpub`/`Upub`/`Vpub` testnet) extended-key prefixes encode the intended script type (BIP-49 single/multi, BIP-84 single/multi) in the version bytes. Bitcoin Core + rust-miniscript + BIP-388 wallet policies reject non-`xpub` prefixes; the canonical modern path is descriptor-native xpub + descriptor wrapper. v0.6.0 currently fails at `Xpub::from_str` for a SLIP-0132 prefix. Both directions ship together in v0.6.1:

  **Permissive input (mechanical):** add a SLIP-0132 → xpub normalizer (`src/slip0132.rs` helper or inline). On input, detect non-`xpub` prefix; recompute version bytes to the matching `xpub`/`tpub` neutral prefix and re-base58check. The 78 payload bytes are byte-identical across SLIP-0132 variants, so no ECC work — pure prefix swap. Applies to:
    - `convert --from xpub=<zpub-string>`
    - `convert --from xpub=<ypub-string>` (etc.)
    - Cross-cutting: `bundle --slot @0.xpub=<zpub>` and `verify-bundle --slot @0.xpub=<zpub>` normalize identically for input symmetry across the toolkit.

  **Expressive output (design fork — resolve early in the cycle):** add output-side SLIP-0132 emission. Two grammar shapes to choose between via a SPEC amendment + one architect review round at the start of the v0.6.1 cycle:
    - (a) New target nodes `ypub`/`zpub`/`Ypub`/`Zpub` plus testnet `upub`/`vpub`/`Upub`/`Vpub`. Adds 8 nodes to NodeType. Edges: `xpub → ypub` etc. (pure prefix swap; no derivation).
    - (b) Existing `--to xpub` plus a `--xpub-prefix <neutral|y|z|Y|Z>` modifier flag. Single new flag; no new nodes.
   Option (b) is grammar-lighter and preserves the convention that SLIP-0132 variants are *encodings of the same xpub*, not different artifact classes. Lock the choice before implementation begins.

- **Why deferred:** v0.6.0 prioritized the headline single-format conversion graph; SLIP-0132 is a UX-convenience layer over BIP-32 + BIP-388 descriptors. Both directions ship together in v0.6.1 to close the SLIP-0132 story in one release cycle.
- **Status:** `resolved bb77164` (v0.6.1 Phase C — Option (b) selected per architect convergence: `--xpub-prefix <variant>` modifier flag with 5 case-sensitive values (`xpub`/`ypub`/`Ypub`/`zpub`/`Zpub`) per SPEC §11.a; testnet variants are network-context-derived via `--network` (no separate flag values); `--network` REQUIRED when `--xpub-prefix` is non-default. Input normalizer in new `src/slip0132.rs` handles all 8 SLIP-0132 prefixes (4 mainnet + 4 testnet); cross-cut wired at `convert.rs:515`, `bundle.rs:327`, `bundle.rs:853`. New `(xpub, xpub)` edge in §2 for the §11.a round-trip primitive).
- **Tier:** `v0.6.1`

### `convert-test-coverage-tightening` — close convert subcommand test gaps (6 direct-edge + 2 deferral + 3 round-trip tests)

- **Surfaced:** v0.6.0 post-release coverage audit 2026-05-06 (user-prompted enumeration of supported edges vs. test coverage).
- **Where:** `crates/mnemonic-toolkit/tests/cli_convert_happy_paths.rs`.
- **What:** v0.6.0 ships 23 convert tests covering 14 of 20 supported direct edges. Three coverage gaps to close in v0.6.1:
  1. **6 untested supported direct edges** — add at least one happy-path test each:
     - `phrase → ms1`
     - `entropy → xpub`
     - `entropy → xprv`
     - `entropy → fingerprint`
     - `xprv → fingerprint`
     - `wif → fingerprint`
  2. **2 deferral-message negative tests** — assert the v0.6.0 BadInput stderr ("not yet supported in v0.6 (path-to-leaf-WIF derivation deferred)") for:
     - `phrase → wif`
     - `entropy → wif`
     These tests pin the deferral text byte-exactly so the v0.6.1+ implementation of `convert-phrase-to-leaf-wif` will need to update them in lockstep (intentional: forces the deferral-→-implementation transition to be explicit).
  3. **3 explicit round-trip loop tests** (A→B→A) for the supported bidirectional pairs:
     - `phrase ↔ entropy` — assert `phrase → entropy → phrase` produces the canonical phrase byte-for-byte.
     - `entropy ↔ ms1` — assert `entropy → ms1 → entropy` produces identical entropy bytes.
     - `phrase ↔ ms1` (via entropy intermediate) — assert `phrase → ms1 → phrase` produces the canonical phrase. v0.6.0 has one-direction tests on each leg but no full-loop assertion.
- **Why deferred:** v0.6.0 prioritized headline-edge coverage and refusal-taxonomy correctness; the missing tests are tightening, not net-new functionality. The 6 uncovered edges are exercised indirectly through the JSON envelope test (#3 in `cli_convert_json.rs`) and the v0.5.2 16-cell parametric byte-identity test, but lack explicit asserts.
- **Status:** `resolved 59140c5` (v0.6.1 Phase E — 6 direct-edge tests added to `cli_convert_happy_paths.rs`; 3 round-trip loop tests added in new `cli_convert_round_trips.rs`. The 2 deferral-message tests are explicitly NOT written — Phase B (62b4f23) implemented `phrase/entropy → wif` so the deferrals no longer exist).
- **Tier:** `v0.6.1`

### `convert-run-step-numbering-duplicate-8` — `cmd::convert::run` has duplicate `// 8)` step labels

- **Surfaced:** Phase B code-reviewer r1 (Nit, deferred — predates Phase B).
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs:382` and `:385`.
- **What:** The dispatch in `convert::run` numbers its steps `// 1)` through `// 9)`; both "Compute outputs" and "Emit" are labeled `// 8)`. The second should be `// 9)` to keep the comment numbering monotonic. Comment-only nit; no behavioral effect.
- **Why deferred:** Pre-existed Phase B; out of scope for the SPEC-A `phrase/entropy → wif` commit. Cleanly fixable in the next convert-touching patch.
- **Status:** `resolved a52b2aa` — v0.6.2 cosmetic micro-commit (Phase 5). `// 8) Emit.` → `// 9) Emit.`; `// 9) §7 secret-on-stdout warning.` → `// 10)`. Comment-only.
- **Tier:** `v0.6.2-nice-to-have`

### `slip0132-input-normalization-stderr-info` — emit a one-line stderr note when SLIP-0132 input is silently normalized

- **Surfaced:** v0.6.1 post-release UX discussion 2026-05-06.
- **Where:** new helper at `crates/mnemonic-toolkit/src/slip0132.rs`; emitter at the 3 production cross-cut sites (`convert.rs:515`, `bundle.rs:327`, `bundle.rs:853`).
- **What:** v0.6.1 Phase C silently normalizes SLIP-0132 prefix variants (zpub/ypub/Zpub/Ypub mainnet + uvpub/UVpub testnet) to neutral xpub/tpub on input. The user gets correct math but loses the intent signal their prefix carried (BIP-49 vs BIP-84, single-sig vs multisig). Add a one-line stderr informational note when normalization actually fires — pattern:
  ```
  info: normalized <variant> input to neutral <xpub|tpub> (encoding-only; no key change). Re-emit with --xpub-prefix <variant> if you need the SLIP-0132 form.
  ```
  Suppressed when input is already neutral. Quiet for users who already understand the normalization; informative for users discovering the round-trip primitive. The emitter must thread `&mut dyn Write` for stderr to all 3 cross-cut sites OR be implemented as an out-parameter on `normalize_xpub_prefix` so the caller decides where to write. Implementation tip: if `normalize_xpub_prefix` returns `Result<(String, Option<&'static str> /* variant-name */), ToolkitError>`, callers can match on `Some(_)` and emit per their stderr convention.
- **Why deferred:** v0.6.1 shipped the silent-normalization MVP intentionally (smaller blast radius; no new stderr bytes that could break byte-exact tests; Phase D's stderr-ordering invariant stays simple). UX-improvement work fits a v0.6.2 patch.
- **Caveat:** new stderr lines at the 3 cross-cut sites must NOT break the Phase D §5.5.a "secret-on-stdout warning is the LAST stderr write" invariant. Either fire the info note BEFORE the engraving card / before the secret-on-stdout warning, or relax the §5.5.a SPEC clause. Spike before SPEC-amending.
- **Status:** `resolved e4fedd7` — v0.6.2 lean cycle. SLIP-0132 input-normalization stderr info-line shipped; SPEC §5.5.a relaxation + multi-slot ordering + `--json` / `--no-engraving-card` independence locked. Phase 1 RED scaffold (`38c4272` + `740a917`), Phase 2 helper signature (`11c8edb` + `957db16`), Phase 3 emission (`e4fedd7` + `7bf1f1e` review-fix DRY refactor), Phase 4 SPEC + CHANGELOG (`39fa359` + `42561f3`), Phase 5 cosmetic step-numbering (`a52b2aa` + `96c2e3b`), Phase 6 release (`1fddf3b`).
- **Tier:** `v0.6.2`

### `slip0132-info-line-spec-text-not-byte-pinned` — SPEC §11 info-line wording isn't programmatically locked to the production format string

- **Surfaced:** v0.6.2 final cumulative review 2026-05-06.
- **Where:** `design/SPEC_convert_v0_6.md` §11 (canonical info-line paragraph); `crates/mnemonic-toolkit/src/slip0132.rs::render_slip0132_info_line` (production helper); `crates/mnemonic-toolkit/src/slip0132.rs::tests::render_slip0132_info_line_pins_canonical_text` (existing pin test, locks production ↔ slip0132 internal only).
- **What:** v0.6.2 introduced `render_slip0132_info_line(variant)` as the single production source of truth for the info-line text, with a unit test pinning the byte sequence for representative variants. The SPEC body in §11 carries the canonical text but as a templated example with `<variant>` and `<xpub|tpub>` placeholders. There is no test that asserts the SPEC body matches the production format-string structure. A future editor "improving" SPEC §11 prose (e.g., changing "Re-emit with" to "Re-encode with") would silently desync the SPEC from shipped behavior; CI catches nothing.
- **Why deferred:** v0.6.2 lean cycle scope; not a correctness bug. Test-side helpers in `tests/cli_*_slip0132_info.rs` provide bidirectional locking against production already (any production-text drift fails the integration tests), so the practical drift hazard is bounded — this entry is about catching SPEC-prose drift specifically.
- **Possible fix:** add a doc-test or unit test that grep-matches the SPEC §11 paragraph against a structural pattern, OR convert SPEC §11's example block into a fenced ```text block whose canonical form is read at test time. The first option is lower-overhead.
- **Status:** `resolved 354c945`
- **Resolution:** v0.7 Phase 7 — `slip0132::tests::spec_info_line_template_matches_production_render` reads `SPEC_convert_v0_6.md` §11 via `include_str!`, slices between HTML markers, and asserts byte-equality against `render_slip0132_info_line` for all 8 SLIP-0132 variants. SPEC↔production drift now CI-locked.
- **Tier:** `v0.7-nice-to-have`

### `verify-bundle-discards-slip0132-input-variant-asymmetry` — `verify-bundle` silently drops the SLIP-0132 input-normalization signal across 4 callsites

- **Surfaced:** v0.6.2 Phase 3 implementation (`e4fedd7`); confirmed in v0.6.2 final cumulative review 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:209`, `:261`, `:337`, `:407` — each callsite destructures `let (resolved, _slip0132_signals) = resolve_slots(...)?;` and discards the signal with the comment `// verify-bundle does not surface SLIP-0132 input-normalization signals.`
- **What:** v0.6.2 made `mnemonic convert` and `mnemonic bundle` emit a stderr informational line when a user-supplied SLIP-0132 prefix is silently normalized. `mnemonic verify-bundle` calls the same `pub(crate) resolve_slots` helper but discards the signal. Result: a user who pastes a `zpub` to `bundle` gets the info-line; pasting the same `zpub` to `verify-bundle` does not. The discard is semantically correct for v0.6.2's scope (verify-bundle is structurally a checker that emits check-pass/fail status, not a renderer of user inputs), but it creates a UX asymmetry within the toolkit.
- **Why deferred:** parity decision is its own UX policy question (does verify-bundle want to also emit the info-line? Or remain silent on stderr by design?). v0.6.2 lean cycle did not litigate this — the discard was the no-op-on-verify-bundle choice that minimized blast radius.
- **Possible fix (v0.7+ brainstorm):** decide whether `verify-bundle` should also emit the info-line for symmetry. If yes, thread `slip0132_signals` to a stderr emitter near each of the 4 callsites; SPEC §5.5.a's stderr-ordering invariant applies (notes precede any conditional warnings). If no, document the asymmetry intentionally in `SPEC_convert_v0_6.md` §11 / verify-bundle SPEC.
- **Status:** `resolved 354c945`
- **Resolution:** v0.7 Phase 7 — Option B locked per architect R1-I8. The 4 callsite-comments at `verify_bundle.rs:208/:261/:336/:406` gain a SPEC §11 v0.7 amendment cross-pointer; verify-bundle remains silent on SLIP-0132 input-normalization signals as intentional checker semantics. Zero new emission code.
- **Tier:** `v0.7-nice-to-have`

### `bip38-distinct-passphrase-flag` — split composite `(Phrase|Entropy, Bip38)` passphrase into two channels

- **Surfaced:** v0.7 Phase 1 code-quality review (commit `c3d0a85`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` composite arm + `convert::ConvertArgs` clap struct; SPEC §12.b reference.
- **What:** v0.7 ships dual-purpose `--passphrase` for composite paths flowing `phrase → wif → bip38` (or `entropy → wif → bip38`). One passphrase value drives both BIP-39 PBKDF2 mnemonic extension and BIP-38 Scrypt encryption. A user wanting distinct values must invoke `convert` twice. v0.8 may add `--bip38-passphrase` as a distinct flag so a single composite invocation can use different passphrases per layer. Implementation: thread the new flag through `compute_outputs`'s composite arms; if `--bip38-passphrase` is supplied, use it for the Scrypt step and use `--passphrase` (or `""` if absent) for the PBKDF2 step.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — `--bip38-passphrase` flag added with locked R1-I3 semantics (composite arm: independent passphrases, no fallback, BREAKING change from v0.7's dual-purpose dispatch; direct arm: fallback to `--passphrase`). CHANGELOG `[0.8.0]` migration sentence pinned. SPEC v0.8 §12.b amendment.
- **Tier:** `v0.8`

### `bip38-encrypted-wif` — accept + emit BIP-38 passphrase-encrypted privkeys (`6P...`)

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new `BipEncryptedWif` (or `Bip38`) `NodeType` variant in `convert.rs`; new edges from/to `wif`; SPEC §1 + §2 amendments.
- **What:** BIP-38 (`6P...`, base58check, 58 chars) is a passphrase-encrypted WIF format — widely used for paper wallets and key handoff. Two pieces: non-EC-multiplied form (encrypts an existing privkey under a passphrase via Scrypt) and EC-multiplied form (generates new privkey from passphrase + intermediate code; less common). Add as a new convert node so users can decrypt `6P → wif` (with `--passphrase`) and encrypt `wif → 6P` (with `--passphrase`); composite edges `phrase → 6P` follow naturally. Refusal class for `6P → 6P` and any cross-format pivot.
- **Why deferred:** v0.6.x focused on BIP-39 + BIP-32 + SLIP-0132 graph completeness; BIP-38 is its own well-defined Scrypt-backed format and merits a dedicated phase. Implementation likely uses the `bip38` crate or hand-rolled Scrypt against `secp256k1` primitives already in the dep tree.
- **Status:** `resolved c3d0a85`
- **Resolution:** v0.7 Phase 1 — `Wif↔Bip38` edges + composite paths shipped via `bip38 = "1.1"` crate (Apache-2.0). Security review at `design/agent-reports/v0_7-phase-1-bip38-security-review.md`. SPEC §12.
- **Tier:** `v0.7`

### `casascius-mini-private-key` — accept Casascius mini-key (`S...`, 22/26/30 chars) on input

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new `MiniKey` `NodeType` (or absorb into `Wif` arm with prefix-detect) in `convert.rs`; SPEC §1 + §2.
- **What:** Casascius mini private key is a compact base-58 alphabet encoding starting with capital `S` (22, 26, or 30 chars) used historically on physical Bitcoin coins. Format: `S` + N chars; SHA256 of the full string + `?` must hash to `0x00` prefix (typo-checksum). Decoding: SHA256 of the mini-key string yields a 32-byte privkey scalar. One-way edge `mini-key → wif` (encoding-only; no key change). No `wif → mini-key` (mini-key generation requires a search for the typo-checksum-passing string; not deterministic from a given privkey). Refusal class: encode direction is a `§3.b lossy compression barrier` (the typo-checksum embedded in the mini-key string is not recoverable from a raw privkey).
- **Why deferred:** small but distinct format with its own checksum spec (Casascius's typo-check rule); fits a v0.7 grab-bag of less-common formats.
- **Status:** `resolved 89d29ab`
- **Resolution:** v0.7 Phase 2 — `(MiniKey, Wif)` decode-only edge shipped; SHA256 self-checksum rule enforced; encode direction refused as one-way (§3.b lossy-compression barrier). SPEC §13.
- **Tier:** `v0.7`

### `bip85-deterministic-entropy` — derive child seeds from a BIP-32 master

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new top-level `mnemonic derive-child` subcommand OR new edge `xprv → entropy` (with `--bip85-path` modifier); SPEC §1 / new top-level SPEC.
- **What:** BIP-85 derives deterministic child entropy from a BIP-32 master xpriv via HMAC-SHA512 at the path `m/83696968'/<application>'/<index>'`. Use cases: managing many seeds from one master; per-application sub-seeds; password derivation; WIF derivation. Standard application codes per BIP-85: `39'` (BIP-39 entropy of length L words), `2'` (HD-seed), `32'` (xprv child), `128169'` (hex bytes), `707764'` (passwords). Output node depends on the application code: `entropy` for `39'`, `xprv` for `32'`, etc. Grammar lean: `mnemonic derive-child --from xprv=<master> --application <bip39|hd-seed|xprv|hex|password> --length <N> --index <N>` to keep convert's edge-table model untouched (BIP-85 is a *derivation* operation, not a *single-format conversion*).
- **Why deferred:** BIP-85 is a useful but narrow derivation utility; doesn't fit `convert`'s "single-format conversion" framing cleanly. Likely wants its own subcommand. SPEC question to resolve at brainstorm: subcommand vs. extending convert.
- **Status:** `resolved 965cc3e`
- **Resolution:** v0.7 Phase 6 — new `mnemonic derive-child` subcommand shipped with 6 in-scope applications (`bip39`, `hd-seed`, `xprv`, `hex`, `password-base64`, `password-base85`). RSA / RSA-GPG / DICE refused with v0.8 deferral stubs. New SPEC `design/SPEC_derive_child_v0_7.md`.
- **Tier:** `v0.7`

### `slip39-shamir-secret-sharing` — SLIP-39 Trezor Shamir backup format

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new `Slip39` `NodeType` (or new top-level subcommand `mnemonic slip39`); SPEC additions or new SPEC document.
- **What:** SLIP-39 (Trezor's standard) is a K-of-N Shamir-secret-sharing scheme for BIP-39 entropy with its own wordlist (1024 words, distinct from BIP-39's 2048). Shares carry: identifier, iteration exponent, group threshold, group count, member threshold, member index, share value, checksum (Reed-Solomon). Two-level scheme: groups × shares-within-group. Used by Trezor Model T for its native backup. Edges: `entropy → slip39-shares` (split; takes `--group-threshold` + per-group `--member-threshold`); `slip39-shares → entropy` (combine; needs ≥ K shares). Composite via entropy intermediate: `phrase → slip39-shares` etc. The 1024-word SLIP-39 wordlist must be embedded.
- **Why deferred:** Largest single addition in the queue — SLIP-39 is essentially an alternative to BIP-39's wordlist + a Shamir layer. The toolkit's secret-material slot would gain a second-class citizen alongside BIP-39 entropy. Significant SPEC + impl work. Trezor's `python-shamir-mnemonic` library is the reference impl. Note: the planned `mnemonic-secret` v0.2 cycle (sibling repo) is shipping K-of-N share encoding for ms1 codex32 — that may obviate the need for SLIP-39 in this toolkit, depending on user priorities. Brainstorm should resolve "do we want SLIP-39 *and* ms1-shares, or just ms1-shares?" before any impl. v0.7 cycle resolved to defer (lib audit returned hand-roll-required; no maintained Rust crate). v0.8 cycle re-tiered to v1+: scope is too large for a v0.8 minor cycle alongside the locked BIP-38/BIP-85/Electrum/export-wallet menu, and ms1-shares may obviate.
- **Status:** `resolved 6a80343` — shipped as `mnemonic-toolkit-v0.13.0` (2026-05-14). New `mnemonic slip39 split/combine` CLI subcommands + ~2000 LOC hand-rolled SLIP-39 library + 443-LOC canonical manual chapter. Trezor SLIP-0039 K-of-N threshold splitter; bit-identical to `python-shamir-mnemonic@17fcce14`; cross-impl smoke recipe validated against `shamir-mnemonic 0.3.0`. SPEC accumulated 9 patches across the cycle. Status drift (stale `open` until 2026-05-16 Bucket 5 v1.0 drill) flagged by the drill methodology and corrected here. Closes the K-of-N gap that `seed-xor-coldcard-compat` (v0.12.0) deferred. See [[project-v0-13-0-slip39-closed]].
- **Tier:** `v0.13.0-feature` (re-tiered from `v1+`, shipped).
- **Companion:** [[seed-xor-coldcard-compat]] (the v0.12.0 cycle's all-or-nothing counterpart; closed at `mnemonic-toolkit-v0.12.0` tag `63b4503`; introduced the multi-secret-on-stdout advisory class that v0.13.0 parameterizes for K-of-N).

### `slip39-cli-extendable-flag` — surface `--extendable` toggle on `mnemonic slip39 split`

- **Surfaced:** v0.13.0 P1c-E.1 R0 Q1 resolution 2026-05-14; refiled at v0.13.0 P2.1 RED 2026-05-14 per plan `design/PLAN_v0_13_0_p2.md` §2.3 + §3.1.
- **Where:** `crates/mnemonic-toolkit/src/cmd/slip39.rs` (`Slip39SplitArgs` flag table + `run_split` forwarding); SPEC §2.2 split flag table; manual `docs/manual/src/40-cli-reference/41-mnemonic.md`.
- **What:** v0.13.0 P2 hardcodes `extendable=false` for both `slip39 split` and `slip39 combine` per P1c-E.1 R0 Q1. The library's `slip39_split` and `slip39_combine` already accept `extendable: bool` (verified at P1c-E.2 LOCK; SPEC §2.5 row 22 `ExtendableMismatch` already exists for the combine-time refusal). v0.14 adds a user-facing `--extendable` CLI flag on `split` and a combine-time validation that all parsed shares share the bit. Refusal class is already wired at the library level so this is purely a CLI-surface add + manual mirror.
- **Why deferred:** v0.13.0 priority is SLIP-39 K-of-N parity with Trezor's reference behavior (which defaults to `extendable=false`); adding a CLI flag at P2 would expand the user-facing surface beyond the SPEC §2.2 v0.13.0 contract. Library parameter is already plumbed so v0.14 is a small CLI-only delta.
- **Status:** `open`
- **Tier:** `v0.14-feature`
- **Companion:** [[slip39-shamir-secret-sharing]] (the parent feature; this is a v0.14 follow-on that surfaces a library parameter already shipped at v0.13.0).

### `electrum-native-seed-format` — Electrum seed wordlist + version-prefix checksum

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new `ElectrumPhrase` `NodeType` in `convert.rs` (or distinct from BIP-39's `Phrase`); SPEC §1 + §2.
- **What:** Electrum's seed format is its own wordlist + checksum scheme distinct from BIP-39. The seed validates by HMAC-SHA512 of the phrase prefixed with `"Seed version"`; the resulting hash's hex prefix encodes the seed-type (`01` = standard, `100` = segwit, `101` = 2FA standard, `102` = 2FA segwit). Conversion: `electrum-phrase → entropy` (different mapping than BIP-39); `electrum-phrase → seed → master xpriv`. Edges symmetric to BIP-39's. Wordlist embedding required (Electrum English wordlist is similar to BIP-39's but differs).
- **Why deferred:** medium scope — own wordlist + checksum + seed-version dispatch. Used by Electrum users transitioning to / from BIP-39-based wallets. Less urgent than BIP-38 / BIP-85 because most Electrum users can re-derive into BIP-39 via the wallet. Brainstorm should weigh user demand.
- **Status:** `resolved 892139c`
- **Resolution:** v0.7 Phase 3 — `ElectrumPhrase ↔ Entropy` edges shipped with 4-version HMAC-SHA512 prefix dispatch (`01`/`100`/`101`/`102`); 2FA versions (`101`/`102`) refused. Corpus spike at `design/agent-reports/v0_7-phase-3-electrum-corpus-spike.md`. SPEC §14.
- **Tier:** `v0.7`

### `miniscript-beyond-bip388` — accept full miniscript policies beyond BIP-388's descriptor-template subset

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::bundle_run_unified_descriptor`; descriptor-input handling in `parse_descriptor.rs`; new SPEC §4.12 (v0.19.0 cycle); plan-doc at `design/PLAN_v0_19_0_non_canonical_descriptors.md`.
- **What:** v0.5+ accepts BIP-388-conformant descriptors (placeholder-template form `wpkh(@0/<0;1>/*)`, multipath, sortedmulti, etc.). The full miniscript language has many additional policies the toolkit doesn't surface as supported wallet types: `andor`, `thresh`, `pk_h`, time-locked branches via `older` / `after`, hash-locked `hash160` / `sha256` / `ripemd160`, `multi_a` (taproot multi without sortedness), arbitrary `tr` taproot trees with multi-leaf miniscript. Rust-miniscript supports parsing these; the gap is the toolkit's wallet-policy validation and the engraving-card / verify-bundle UX.
- **v0.19.0 cycle scope (locked 2026-05-16):** plan-doc V6 converged 0C/0I across opus R0-R5 + user-direction Q1-reversal at R4. Locked design: (Q1) silent default-path inference `m/48'/<coin>'/<account>'/2'` (BIP-48 cosigner path) for non-canonical wsh/sh-wsh/tr wrappers; (Q2) `tr(NUMS, <ms>)` sentinel substitution for script-path-only P2TR wallets; (Q3) trust rust-miniscript for fragment validity (toolkit gates wrapper class + per-`@N` origin coverage + slot grammar); (Q4) GUI Option-A inline `conditional.rs::bundle()` rules including canonicity-aware override of the existing `--account → pin_value(0)` rule. Lockstep release `mnemonic-toolkit-v0.19.0` + `mnemonic-gui-v0.8.0`.
- **Status:** `resolved 087d0e4` — shipped at `mnemonic-toolkit-v0.19.0` (2026-05-17) + `mnemonic-toolkit-v0.19.1` patch (`d4e7935`; clippy collapsible-if fix); lockstep `mnemonic-gui-v0.8.0` (`10a1abd`). Phases 1-7 across both repos converged 0C/0I. End-of-cycle Phase 7 opus review caught C1 (verify-bundle round-trip break for non-canonical default-inferred bundles); folded by mirroring bundle's canonicity-aware path-decl inference in `verify_bundle.rs::descriptor_mode_verify_run`. 3 new FOLLOWUPs filed at cycle close (`verify-bundle-multi-cosigner-mk1-chunk-assembly` pre-existing bug class deferred to v0.20-bugfix; `gui-schema-classify-descriptor-subcommand` diagnostic for drift gate, v0.20-feature; `gui-non-canonical-descriptor-banner-and-placeholder` GUI perceptibility patch, v0.8.1-gui-patch). User-run-the-feature smoke confirmed on user's flagship `wsh(andor(...))` invocation + `tr(NUMS, ...)` variant; stderr info notice byte-exact per SPEC §4.12.d.
- **Tier:** `v0.19-feature` (re-tiered from `v1+` per user direction 2026-05-16; shipped 2026-05-17).

### `vault-construction-covenant-based` — accept covenant-based vault descriptors (CTV / OP_CAT / OP_VAULT)

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** `parse_descriptor.rs` (descriptor parser extension); SPEC additions.
- **What:** Vault constructions use covenant opcodes (BIP-119 `OP_CHECKTEMPLATEVERIFY`, BIP-348 `OP_CAT` re-enable, BIP-345 `OP_VAULT`) to enforce spending paths beyond what current Bitcoin script allows: time-delayed spends, recovery paths, batch authorizations, etc. None of these opcodes are activated on mainnet today. When/if activated, vault descriptors become a wallet-type class distinct from current single-sig/multisig descriptors.
- **Why deferred:** **Gated on Bitcoin consensus activation.** No mainnet support today; signet test-cases exist for some of these. Re-evaluate when the relevant BIP advances to mainnet. The plumbing in this toolkit (descriptor parsing, mk1 xpub binding, md1 wallet-policy encoding) generalizes to vaults, so the impl gap is small once the script-side BIP activates — but speculative until then.
- **Status:** `open`
- **Tier:** `v1+`

### `address-derivation-from-xpub-path` — xpub + path + script-type → bitcoin address

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06; in-scope confirmed 2026-05-06 (SPEC §10 exclusion struck through, see `SPEC_convert_v0_6.md` §10 v0.6.1 amendment).
- **Where:** new edge `(xpub, address)` in `convert.rs::is_supported_direct_edge`; new `Address` `NodeType`; SPEC §1 + §2 amendments.
- **What:** Edge: `xpub` source + `--path` (or `--address-index N` + `--chain receive|change`) + script-type inferred from `--template` (or explicit `--script-type p2wpkh|p2sh-p2wpkh|p2tr|...`) → bech32 / bech32m / base58 address string. Composite from `phrase` / `entropy` via the existing BIP-32 derivation pipeline. Refusal classes: address → anything (one-way; addresses are hash160/SHA256 of pubkeys). Read-only display only — does NOT extend to PSBT / signing (PSBT remains out-of-scope per `bip174-psbt-signing` v1+).
- **Why deferred:** Useful but not blocking; v0.7 cycle slot. SPEC §10 amendment from "out of scope" to "in scope, deferred to v0.7" was committed alongside this entry update.
- **Status:** `resolved 940ec0b`
- **Resolution:** v0.7 Phase 4 — `(Xpub, Address)` edge shipped with `--path` mandatory + `--script-type` inferred from `--template` for BIP-44/49/84/86 → P2PKH/P2SH-P2WPKH/P2WPKH/P2TR. Composite paths via the existing BIP-32 derivation pipeline. SPEC §10.a.
- **Tier:** `v0.7`

### `bip327-musig2-collective-keys` — MuSig2 collective-key wallet-policy support

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** descriptor parser + new wallet-policy class.
- **What:** BIP-327 (MuSig2) defines a 2-round Schnorr multi-signature scheme that produces a single aggregated public key from N participants. Wallet-policy formats for MuSig2 collective keys are still maturing — there's no settled "BIP-388-equivalent" for MuSig2 wallets yet. When the spec settles, add support for MuSig2 collective keys as a wallet-policy variant alongside multisig (sortedmulti) and single-sig.
- **Why deferred:** **Standards-maturity gate.** No settled wallet-policy spec; rust-miniscript support is partial; hardware-wallet vendor adoption is preliminary. Re-evaluate when the wallet-policy spec for MuSig2 stabilizes.
- **Status:** `open`
- **Tier:** `v1+`

### `bip174-psbt-signing` — Partially Signed Bitcoin Transactions support

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** would be an entirely new subcommand surface area.
- **What:** BIP-174 PSBT (Partially Signed Bitcoin Transactions) is the standard format for unsigned/partially-signed transactions exchanged between wallets and signers. Adding PSBT support would expand the toolkit from "key/wallet management" into "transaction signing" — a fundamentally different problem class.
- **Why deferred:** **Out of scope per `convert` SPEC §10:** "different problem class." This toolkit is explicitly about key/wallet info, not transaction signing. PSBT belongs in a distinct tool (or in a separate signer subcommand of this toolkit if scope is reframed at v1+).
- **Status:** `open`
- **Tier:** `v1+`

### `frost-threshold-keys` — FROST threshold signature scheme support

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new wallet-policy class; cryptographic primitive set extension.
- **What:** FROST (Flexible Round-Optimized Schnorr Threshold signatures) is a K-of-N threshold signature scheme that produces a single aggregated Schnorr signature without requiring a trusted dealer. Distinct from MuSig2 in that it's threshold (K-of-N) rather than n-of-n. Wallet-policy and key-aggregation formats are still being standardized.
- **Why deferred:** **Standards-maturity gate.** No settled wallet-policy spec; cryptographic primitives not yet in `bitcoin` / `secp256k1` crates; hardware-wallet adoption preliminary. Re-evaluate when the spec stabilizes.
- **Status:** `open`
- **Tier:** `v1+`

### `liquid-confidential-extended-keys` — Liquid sidechain extended-key formats

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new network variant in `network.rs` (`Liquid` / `LiquidTestnet`); xpub/xprv version-byte table extension; SPEC §11 SLIP-0132 swap table additions.
- **What:** Elements/Liquid sidechain uses its own xpub/xprv version bytes plus blinding-key extensions for confidential transactions. Asset-blinding keys, value-blinding keys, and the address-blinding-key derivation (SLIP-0077) are Liquid-specific cryptographic primitives. Adding Liquid support would require: (a) network variant + version-byte table; (b) blinding-key derivation primitives; (c) Confidential Address derivation (different from mainnet bech32 addresses).
- **Why deferred:** Different chain context. Liquid is a federated sidechain with its own wallet ecosystem (Blockstream Green, Elements). The toolkit's primary user base is Bitcoin mainnet/testnet; Liquid users typically use Liquid-specific tooling. Re-evaluate if there's demand from cross-chain users.
- **Status:** `open`
- **Tier:** `v1+`

### `wallet-export-industry-formats` — `mnemonic export-wallet` (or `bundle --wallet-export <format>`) for Bitcoin Core / Sparrow / Specter / BIP-388 import

- **Surfaced:** v0.6.1 post-release UX discussion 2026-05-06.
- **Where:** new subcommand `mnemonic export-wallet` OR new flag on `bundle`; output formatters under `crates/mnemonic-toolkit/src/wallet_export.rs` (new module).
- **What:** Today the canonical "all wallet info, no secret" representation IS `mnemonic bundle --json` in watch-only mode (per SPEC §5.8: ms1 omitted-or-empty-sentinel; mk1 carries xpub bindings; md1 carries the descriptor/template). It is correct and complete BUT only the toolkit can re-ingest it. Users who want to feed the watch-only artifact to another wallet (Bitcoin Core, Sparrow, Specter, hardware-wallet HWI flows) must hand-translate. Add an industry-format export layer with at least:
  - **Bitcoin Core `importdescriptors` JSON** — `{"desc": "wpkh([fp/path]xpub.../{0,1}/*)#checksum", "active": true, "internal": false, "range": [0, 999], "timestamp": "now"}` per descriptor (one for receive, one for change; or `<0;1>` multipath split). Matches Bitcoin Core 25+ descriptor-wallet expectations.
  - **BIP-388 wallet policy** — formal `wallet_policy` JSON with `name`, `description_template`, `keys_info` array. Matches Ledger / hardware-wallet vendors that follow BIP-388.
  - **Sparrow / Specter wallet JSON** (optional; format is per-wallet). Lower priority — both can ingest output descriptors directly via the Bitcoin Core format.
  - **HWI signer JSON** (optional) — for cosigner export.
  Grammar lean: `mnemonic export-wallet --format <bitcoin-core|bip388|sparrow|specter> --output <path-or-->` with the same `--slot @N.<subkey>=<value>` input shape as `bundle`. Refuses if any slot supplies entropy/phrase (export-wallet is watch-only by definition). SPEC question to resolve at brainstorm: does this live as a new top-level subcommand OR a `bundle --wallet-export` flag? Lean: new subcommand because the input grammar is a strict subset of bundle (no entropy/phrase) and the output is a different wire format from `BundleJson`.
- **Why deferred:** v0.6.1 was a polish patch for `convert` + `bundle` UX. New subcommand or new bundle flag is its own minor scope. Brainstorm should resolve the format priority list (Bitcoin Core first vs BIP-388 first), the subcommand-vs-flag fork, and whether `range`/`timestamp` defaults need to be configurable.
- **Status:** `resolved 3821f66`
- **Resolution:** v0.7 Phase 5 — new `mnemonic export-wallet` subcommand shipped. Bitcoin Core `importdescriptors` JSON (default) + BIP-388 `wallet_policy` JSON. Sparrow / Specter formats stubbed (refuse with v0.8 deferral). `--range` / `--timestamp` / `--bitcoin-core-version` overrides. Watch-only enforced (refuses entropy/phrase slot input). New SPEC `design/SPEC_export_wallet_v0_7.md`.
- **Resolution-extended (v0.8.1 Phase 1):** Coldcard generic JSON skeleton (singlesig bip44/bip49/bip84) + Coldcard multisig text (wsh / sh-wsh, sorted and unsorted) + Blockstream Jade multisig text (byte-identical to Coldcard's, delegated emitter) shipped. New `wallet_export/{coldcard,jade}.rs`. `CliExportFormat::Coldcard` + `CliExportFormat::Jade` variants. `--wallet-name <STRING>` clap flag for formats publishing wallet names (Coldcard generic JSON, Sparrow / Specter / Electrum land in subsequent phases). New slot subkey `@N.master_xpub=` (depth-0 root xpub, optional, watch-only-class). Coverage now 2/8 → 4/8 of the SPEC §11 priority list; Sparrow/Specter/Electrum/Green land in Phases 2-5.
- **Resolution-extended (v0.8.1 Phase 5):** All six new vendor formats now shipped — `coldcard`, `jade`, `sparrow`, `specter`, `electrum`, `green`. The complete v0.8.1 SPEC §11 priority list (8 formats: `bitcoin-core`, `bip388`, `coldcard`, `jade`, `sparrow`, `specter`, `electrum`, `green`) is fully realized. New emitter modules: `wallet_export/{sparrow,specter,electrum,green}.rs`. New `CliExportFormat` variants: `Sparrow`, `Specter`, `Electrum`, `Green`. Per-format pinned byte-exact fixtures + SPEC §4 missing-info refusal channel exercised by Sparrow (Threshold) and Specter (WalletName). Status remains `resolved`.
- **Tier:** `v0.6.2`

### `coldcard-master-xpub-plumbing-pending` — `@N.master_xpub=` slot subkey parses but is dropped before reaching the Coldcard emitter

- **Surfaced:** v0.8.1 Phase 1 R1 reviewer-loop fold (I-2).
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs::ResolvedSlot` + `crates/mnemonic-toolkit/src/wallet_export/mod.rs::EmitInputs` + `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs::emit_coldcard_generic_json`.
- **What:** SPEC §2 + §5.1 ship two normative claims: (a) the slot grammar accepts `@N.master_xpub=<base58>` (shipped in Phase 1.1 via `SlotSubkey::MasterXpub`); (b) the Coldcard generic-JSON top-level `xpub` field is emitted iff `@0.master_xpub=` was supplied. Phase 1 shipped (a) but not (b); Phase 1.9.R1 added a refuse-on-supply guard in `cmd::export_wallet::run` so the gap was not silent.
- **Status:** `resolved` (v0.8.2 plumbing cycle).
- **Resolution:** v0.8.2 follow-up — `ResolvedSlot` gained a `master_xpub: Option<Xpub>` field populated in the `{Xpub, ...}` arm of `resolve_slots` via `crate::slip0132::normalize_xpub_prefix` + `Xpub::from_str`. `EmitInputs` gained `master_xpub_at_0: Option<Xpub>` plumbed from `resolved_slots[0].master_xpub` in `cmd::export_wallet::run`. `emit_coldcard_generic_json` now emits the top-level `xpub` field conditionally (`Some(x) → x.to_string()`, `None → field omitted via `#[serde(skip_serializing_if = "Option::is_none")]`). The Phase 1.9.R1 refuse-on-supply guard at `cmd::export_wallet::run:182-197` was retired. New byte-exact fixture `tests/export_wallet/coldcard_generic_bip84_mainnet_with_master_xpub.json`; new test cells `cell_8_coldcard_master_xpub_plumbing_byte_exact` (supplied case) and `cell_9_coldcard_master_xpub_absent_omits_top_level_xpub` (absent case). All other resolution arms (Phrase / Entropy / Wif / synthesize-test-helper / verify-bundle-rebuild) set `master_xpub: None` since master_xpub semantically only exists on user-supplied watch-only xpub slots.
- **Tier:** `v0.8.2`

### `coldcard-bip86-generic-export-pending-firmware` — `--template bip86 --format coldcard` refuses (BIP-86 not in upstream schema)

- **Surfaced:** v0.8.1 Phase 1 (SPEC R1-I2 reviewer-loop fold).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs::emit_coldcard_generic_json`.
- **What:** Coldcard's canonical `generic-wallet-export.md` (upstream master) documents only `bip44` / `bip49` / `bip84` sub-objects. BIP-86 (P2TR singlesig) has no slot in the schema. The toolkit refuses `--template bip86 --format coldcard` with the SPEC §5.1 byte-exact pointer until Coldcard firmware extends the schema. Workaround: use `--format bitcoin-core` (descriptor passthrough) or `--format sparrow` (native P2TR support).
- **Status:** open (pending Coldcard firmware). Last upstream-checked **2026-05-12**: `gh api repos/Coldcard/firmware/contents/docs/generic-wallet-export.md` — no `bip86` / `p2tr` / `taproot` mentions. `releases/ChangeLog.md` — no taproot / schnorr / bip86 entries.
- **Tier:** `v1+`

### `coldcard-tr-multi-a-pending-firmware` — `--template tr-multi-a` / `tr-sortedmulti-a` refuses under `--format coldcard`

- **Surfaced:** v0.8.1 Phase 1.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs::emit_coldcard_multisig_text`.
- **What:** Coldcard's multisig text emitter ingests only `P2WSH` / `P2SH-P2WSH` / `P2SH` formats per the SPEC §5.2 `Format` field. Taproot-multisig (tr-multi-a / tr-sortedmulti-a) is not in the firmware's import surface. The toolkit refuses with a pointer at `--format bitcoin-core` (descriptor) / `--format sparrow` for taproot multisig watch-only setup. Companion: Jade has the same gap (`jade-tr-multi-a-pending-firmware` below).
- **Status:** open (pending Coldcard firmware taproot-multisig support). Last upstream-checked **2026-05-12**: `releases/ChangeLog.md` — no taproot / schnorr entries; firmware most recent commit `ca06dfd2` 2026-04-25 is unrelated regression fix.
- **Tier:** `v1+`

### `jade-tr-multi-a-pending-firmware` — `--template tr-multi-a` / `tr-sortedmulti-a` refuses under `--format jade`

- **Surfaced:** v0.8.1 Phase 1.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/jade.rs::JadeEmitter::emit`.
- **What:** Blockstream Jade's `register_multisig.multisig_file` accepts the Coldcard §5.2 multisig text shape; taproot multisig is not yet in that surface. The toolkit refuses with a pointer at `--format bitcoin-core` / `--format sparrow` for taproot multisig. Companion: `coldcard-tr-multi-a-pending-firmware` above (Jade shares the schema; once Coldcard ships, Jade follows).
- **Status:** open (pending Blockstream Jade firmware taproot-multisig support). Last upstream-checked **2026-05-12**: `Blockstream/Jade:CHANGELOG.md` — singlesig BIP-86 P2TR SHIPPED (`Add support for signing bip86 single-key p2tr inputs and for registering bip86 p2tr(key) descriptors`); taproot **multisig** (`multi_a` / `sortedmulti_a`) NOT yet shipped. Entry remains accurately open for the multisig case; singlesig P2TR is already a separate emitter path (Sparrow `tr(@0/**)`).
- **Tier:** `v1+`

### `electrum-non-latin-wordlists` — Electrum native seed format hard-codes the English wordlist

- **Surfaced:** v0.7 Phase 3 review (commit `69ac560`).
- **Where:** `crates/mnemonic-toolkit/src/electrum.rs` (wordlist embedding + lookup).
- **What:** Electrum supports 9 wordlists across English, Japanese, Spanish, Chinese (simplified/traditional), Korean, Italian, Portuguese, Dutch, etc. v0.7 Phase 3 ships only the English wordlist; non-English Electrum users cannot decode their phrases through `mnemonic convert`. Add a `--language` parameter mirroring BIP-39 + bundle the additional embedded wordlists.
- **Status:** `resolved 5dc83eb` (v0.8 Phase 2).
- **Resolution:** v0.8 Phase 2 — embedded 4 non-English Electrum wordlists (zh-Hans, ja, pt, es) from `spesmilo/electrum` upstream commit `e1099925e30d91dd033815b512f00582a8795d25`. Plan correction noted: upstream Electrum has 5 total wordlists, not 9 (zh-Hant, German, French, Italian are NOT upstream). Separate `--electrum-language` flag distinct from `--language` (R1-I2 lock); `--electrum-language` wins on Electrum arms (R2-L2 lock). Portuguese is base-1626 (Monero copyright header); base-N arithmetic correctly parameterized.
- **Tier:** `v0.8`

### `electrum-encode-iteration-bound` — encode mining loop has no upper iteration cap

- **Surfaced:** v0.7 Phase 3 review (commit `69ac560`).
- **Where:** `crates/mnemonic-toolkit/src/electrum.rs::encode_phrase` (or equivalent encode-from-entropy path).
- **What:** Electrum's encode direction iterates by incrementing a nonce until the HMAC-SHA512 prefix matches the requested SeedVersion (`01` standard / `100` segwit). The loop has no iteration bound; on adversarially-chosen entropy or for rare versions, the search may run unboundedly long. Add a sane upper bound (e.g., `2^24` iterations) with a byte-exact stderr refusal on exhaustion.
- **Status:** `resolved 5dc83eb` (v0.8 Phase 2).
- **Resolution:** v0.8 Phase 2 — `MAX_ENCODE_ITERATIONS = 1<<20` cap on `entropy_to_phrase` rejection-search loop. New `ElectrumError::EncodeIterationBoundExceeded` mapped to user-visible refusal.
- **Tier:** `v0.8`

### `electrum-version-info-stderr` — decode emits the detected SeedVersion silently

- **Surfaced:** v0.7 Phase 3 review (commit `69ac560`).
- **Where:** `crates/mnemonic-toolkit/src/electrum.rs::decode_phrase` + `cmd::convert::run` electrum arm.
- **What:** When `mnemonic convert --from electrum-phrase=... --to entropy` decodes a phrase, the toolkit dispatches via the SeedVersion prefix (`01` / `100` / `101` / `102`) but does not surface which version it detected. Adding a stderr info-line (e.g., `info: Electrum SeedVersion=01 (standard)`) parallel to the SLIP-0132 input-normalization note (SPEC §11) would help users confirm the dispatch matches their wallet's expectation.
- **Status:** `resolved 5dc83eb` (v0.8 Phase 2).
- **Resolution:** v0.8 Phase 2 — `note: detected Electrum SeedVersion <01|100> (<standard|segwit>)` emitted to stderr on decode arms. `compute_outputs` extended to triple-tuple return surfacing the detected SeedVersion.
- **Tier:** `v0.8-nice-to-have`

### `tr-multi-a-tr-sortedmulti-a-export-wallet-support` — `mnemonic export-wallet` refuses taproot multisig templates

- **Surfaced:** v0.7 Phase 5 code-quality review (commit `f8369d3`).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export.rs` (descriptor pipeline + taproot-multisig validator).
- **What:** `mnemonic export-wallet` refuses `--template tr-multi-a` and `--template tr-sortedmulti-a` at runtime with an error pointing at this v0.8 deferral. Reason: taproot multisig descriptors require an internal-key designation (NUMS point or shared key) plus the script-path tree; the export-wallet pipeline doesn't yet thread the internal-key choice through to Bitcoin Core / BIP-388 formatters. Single-leaf `tr` (BIP-86) IS supported.
- **Status:** `resolved 86647ca` (v0.8 Phase 3).
- **Resolution:** v0.8 Phase 3 — new `--taproot-internal-key <nums|@N>` flag designates the BIP-341 internal key. NUMS uses the canonical reference `50929b74...0ac0` x-only point; `@N` makes cosigner N the key-path key (removed from multi_a leaf set). v0.7 stub refusal replaced with flag-pointing message. Bounds-checked + n=1 degenerate refusal.
- **Tier:** `v0.8`

### `export-wallet-descriptor-bip388-interop` — `--descriptor` mode + `--format bip388` is refused

- **Surfaced:** v0.7 Phase 5 code-quality review (commit `f8369d3`).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export.rs` (format dispatch + descriptor-mode validator).
- **What:** `mnemonic export-wallet --descriptor <user-supplied> --format bip388` is refused at runtime; `--descriptor` mode currently only supports `--format bitcoin-core`. Reason: user-supplied descriptors arrive as opaque strings; converting them to BIP-388 `wallet_policy` requires re-parsing into the placeholder-template form (`@0/<0;1>/*`), which the watch-only template-mode pipeline already does but the descriptor-mode pipeline skips.
- **Status:** `resolved 86647ca` (v0.8 Phase 3).
- **Resolution:** v0.8 Phase 3 — new `descriptor_to_bip388_wallet_policy` helper parses canonical descriptor via miniscript, iterates `iter_pk()` to collect `[fp/path]xpub` keys (stripping `/<0;1>/*`), strips `#checksum`, and replaces each full key-expression with `@N/**` placeholder via longest-first substitution. Refused for non-multipath descriptors.
- **Tier:** `v0.8`

### `bip85-rsa-rsa-gpg-dice-applications` — RSA / RSA-GPG / DICE BIP-85 applications deferred

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/bip85.rs` application dispatch + `crates/mnemonic-toolkit/src/cmd/derive_child.rs` clap surface.
- **What:** BIP-85 application codes `828365'` (RSA), `67797633'` (RSA-GPG), `89101'` (DICE) are refused with v0.8 deferral stubs. Reason: RSA derivation requires an RSA crate not currently in the dep tree; DICE is a niche application (deterministic dice-roll output) with limited demand. The 6 in-scope applications (`bip39`, `hd-seed`, `xprv`, `hex`, `password-base64`, `password-base85`) cover the primary use cases.
- **Status:** `split-resolved 1dde4dc` (v0.8 Phase 7); split into `bip85-dice-application` (resolved) + `bip85-rsa-rsa-gpg-applications` (re-tiered).
- **Resolution:** v0.8 Phase 7 — DICE shipped with BIP85-DRNG-SHAKE256 + rejection sampling per BIP-85 v1.3.0 §"DICE". Spec reference vector (`m/83696968'/89101'/6'/10'/0'` → `1,0,0,2,0,1,5,5,2,4`) pinned. New `--dice-sides` flag. New `sha3 = "0.10"` direct dep. RSA + RSA-GPG re-tiered to v0.9 / pending-rsa-crate-stability per Phase 6 SPIKE (`design/agent-reports/v0_8-phase-6-rsa-crate-security-review.md`): RUSTSEC-2023-0071 Marvin-attack timing sidechannel is **unpatched** (`patched = []`); rsa crate is in extended pre-release (`v0.10.0-rc.18`). Reopen criteria: rsa crate publishes patched stable release OR user requests with stated downstream use case.
- **Tier:** `v0.8` (DICE) / `v0.9` (RSA + RSA-GPG)

### `bip85-passphrase-protected-master` — `--from phrase=` + `--passphrase` direct path

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/derive_child.rs` clap surface.
- **What:** `mnemonic derive-child --from xprv=...` requires xprv input. A user with a passphrase-protected BIP-39 phrase must currently route through `mnemonic convert --from phrase=... --passphrase ... --to xprv` first, then pipe the xprv to `derive-child`. A direct `--from phrase=... --passphrase ...` path on `derive-child` would be more ergonomic.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — `--from phrase=...` accepted; internal `phrase → seed → master xprv` (mainnet, BIP-85-network-agnostic) before BIP-85 derivation. New `--passphrase` for BIP-39 mnemonic extension.
- **Tier:** `v0.8-nice-to-have`

### `bip85-non-english-bip39-language-codes` — `--language` flag inert for BIP-39 application

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/derive_child.rs` clap surface + `bip85::derive_bip39`.
- **What:** `--language` is plumbed through clap on `mnemonic derive-child` but ignored for BIP-85's `bip39` application. BIP-85 supports 9 wordlists (`0'` English through `8'` Czech) for the BIP-39 application via the language-index sub-path component. v0.7 hardcodes English (`0'`).
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — new `resolve_bip85_language` maps `CliLanguage` → (BIP-85 path code, `bip39::Language`). 9 BIP-85-coded languages supported. Portuguese refused (no BIP-85 code assigned).
- **Tier:** `v0.8`

### `bip85-testnet-emission` — `--network` flag inert for hd-seed / xprv applications

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/derive_child.rs` clap surface + `bip85::derive_hd_seed` / `derive_xprv`.
- **What:** `--network` is plumbed through clap but unused. v0.7 hardcodes mainnet WIF / xprv emission for `--application hd-seed` and `--application xprv`. Testnet users must post-process via `mnemonic convert` to swap version bytes.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — `format_hd_seed_wif` + `format_xprv_child` now take `NetworkKind` parameter. Testnet emits `c…` WIF / `tprv…` xprv. Driven by `--network` flag (default mainnet to match BIP-85 spec test vectors).
- **Tier:** `v0.8`

### `bip85-spec-prose-byte-formula-clarification` — SPEC §3 prose vs. worked-example formula mismatch

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `design/SPEC_derive_child_v0_7.md` §3 (BIP-39 byte slicing).
- **What:** SPEC §3 prose says BIP-39 byte slicing uses `2 * length_in_words / 3`; the worked examples (12 words → 16 bytes, 24 words → 32 bytes) match the correct formula `words * 4 / 3`. The two are equivalent for word counts divisible by 3 but the prose formula is not the canonical BIP-39 form. Pure SPEC text fix — implementation is correct.
- **Status:** `resolved 4dfea5a` (v0.8 Phase 0).
- **Resolution:** v0.8 Phase 0 — `2 * length_in_words / 3` → `length_in_words * 4 / 3` in SPEC §3.
- **Tier:** `v0.7-nice-to-have`

### `bip85-stdin-master-xprv` — `--from xprv=-` parses but does not read stdin

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/derive_child.rs::run`.
- **What:** `mnemonic derive-child --from xprv=-` parses through clap (the `=-` sentinel is recognized) but `derive_child::run` does not read stdin to populate the xprv value. `mnemonic convert` does honor `=-`. Add stdin-read parity for cross-subcommand consistency.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — `derive_child::run` now reads stdin when `args.from.value == "-"` via `crate::cmd::convert::read_stdin_to_string` (made `pub(crate)`). Works for both `xprv=-` and `phrase=-`.
- **Tier:** `v0.8`

### `derive-child-spec-2-grammar-uniformity-tension` — SPEC §2 prose-internal contradiction on `--length` mandatoriness

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `design/SPEC_derive_child_v0_7.md` §2.
- **What:** SPEC §2 has internal tension: it says `--length` is mandatory at clap level AND that `--length` is refused for `--application hd-seed` / `--application xprv`. Phase 6 implementation adopted a sentinel-0 convention: clap requires `--length`, and `hd-seed`/`xprv` arms refuse only when the supplied value is non-zero (`0` is treated as sentinel-absent). The SPEC text should be edited to reflect this; current prose reads as a contradiction.
- **Status:** `resolved 4dfea5a` (v0.8 Phase 0).
- **Resolution:** v0.8 Phase 0 — SPEC §2 + §4 prose updated to document the sentinel-0 convention canonically.
- **Tier:** `v0.7-nice-to-have`

### `bip38-ec-multiplied-encrypt-mode-support` — emit BIP-38 EC-multiplied form via intermediate codes

- **Surfaced:** v0.7.1 Phase 3 (BIP test vector audit cycle); rescoped from `bip38-ec-multiplied-mode-support` after Phase 3 forensics.
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` `(Wif, Bip38)` arm; `bip38 = "1.1"` crate.
- **What:** v0.7.1 supports BIP-38 EC-multiplied DECRYPT transparently (4 spec vectors pinned). ENCRYPT to EC-multiplied form requires the intermediate-code workflow per BIP-38 §"Generation of intermediate code": the passphrase owner generates a passphrase code; a third party combines it with random entropy to derive the encrypted privkey + the corresponding bitcoin address. Implementation: new subcommand `mnemonic intermediate-code` (or `--passphrase-code <code>` flag on the `(Wif, Bip38)` arm). Out of scope for v0.7.1 vectors-only audit.
- **Why deferred:** v0.8 Phase 4 SPIKE returned DEFER verdict (`design/agent-reports/v0_8-phase-4-bip38-ec-mult-encrypt-spike.md`). The `bip38 v1.1.1` `Generate` trait covers owner-only path only with internal `rand::thread_rng()` (non-deterministic) and exposes no intermediate-code workflow + no confirmation code. Hand-rolling spec-compliant API costs ~155 LOC of cryptographic code (AES + scrypt + secp256k1 + Unicode normalization). Marginal user value (paper-wallet niche). Re-tiered to `v0.8.1+`.
- **Status:** `open`
- **Tier:** `v0.8.1+`

### `bip38-spec-section-12-ec-multiplied-erratum` — SPEC §12 incorrectly claimed EC-multiplied was refused

- **Surfaced:** v0.7.1 Phase 3 (audit cycle).
- **Where:** `design/SPEC_convert_v0_6.md` §12.
- **What:** The v0.7.0 SPEC §12 stated the `bip38` crate's `Decrypt` impl rejected EC-multiplied codes. Empirical testing in Phase 3 disconfirmed: all 4 EC-multiplied spec vectors decrypt correctly. SPEC §12 corrected in this cycle (commit pinned in matrix). Filed for cross-referencing the erratum source: the v0.7 Phase 1 security review report at `design/agent-reports/v0_7-phase-1-bip38-security-review.md` likely contains the source claim — re-read on next sec-review touch.
- **Why deferred:** documentation-only; closed in this cycle. Filed for audit history continuity.
- **Status:** `resolved 2c59b27`
- **Tier:** `v0.7.1`

### `bip85-dice-application` — BIP-85 `89101'` dice rolls (split product of `bip85-rsa-rsa-gpg-dice-applications`)

- **Surfaced:** v0.8 Phase 6 SPIKE split decision.
- **Where:** `crates/mnemonic-toolkit/src/bip85.rs::format_dice_rolls` + `crates/mnemonic-toolkit/src/cmd/derive_child.rs` dispatch.
- **What:** BIP-85 §"DICE" deterministic dice rolls via SHAKE256 BIP85-DRNG + rejection sampling. Spec at BIP-85 v1.3.0 §"DICE".
- **Status:** `resolved 1dde4dc` (v0.8 Phase 7).
- **Resolution:** v0.8 Phase 7 — `--application dice` + new `--dice-sides <N>` flag. Spec reference vector pinned (`m/83696968'/89101'/6'/10'/0'` → `1,0,0,2,0,1,5,5,2,4`). New `sha3 = "0.10"` direct dep.
- **Tier:** `v0.8`

### `bip85-rsa-rsa-gpg-applications` — BIP-85 RSA + RSA-GPG (split product, deferred)

- **Surfaced:** v0.8 Phase 6 SPIKE split decision (`design/agent-reports/v0_8-phase-6-rsa-crate-security-review.md`).
- **Where:** `crates/mnemonic-toolkit/src/bip85.rs` (would need new app dispatchers); `crates/mnemonic-toolkit/src/cmd/derive_child.rs` (would lift `rsa` / `rsa-gpg` from out-of-scope refusal).
- **What:** BIP-85 application codes `828365'` (RSA) + `67797633'` (RSA-GPG) generate RSA keys deterministically from BIP-85 entropy. Implementation requires the `rsa` crate.
- **Why deferred:** v0.8 Phase 6 SPIKE returned DEFER verdict. RUSTSEC-2023-0071 (Marvin attack: timing sidechannel against PKCS#1 v1.5 decryption) is **unpatched** as of 2026-05-07 (`patched = []`). `rsa` crate is in extended pre-release (`v0.10.0-rc.18`). Adding it as direct dep would import an open advisory into mnemonic-toolkit's `cargo audit` output. BIP-85 RSA / RSA-GPG demand signal is absent.
- **Reopen criteria:** rsa crate publishes patched stable release (`patched = ["X.Y.Z"]` in advisory) OR a user requests BIP-85 RSA / RSA-GPG with a stated downstream use case.
- **Status:** `open`
- **Tier:** `v0.9 / pending-rsa-crate-stability`

### `18-remaining-bip39-trezor-corpus-vectors` — pin remaining 18 of 24 Trezor english corpus cells

- **Surfaced:** v0.7.1 Phase 1.B (BIP test vector audit cycle).
- **Where:** `crates/mnemonic-toolkit/tests/cli_convert_bip39_vectors.rs`.
- **What:** v0.7.1 Phase 1.B pinned 6 of 24 BIP-39 §"Test Vectors" English Trezor corpus cells via hand-rolled tests; the remaining 18 stayed MISSING per the v0.7.1 audit matrix. v0.8 lifts to a parametric loop over the full corpus.
- **Status:** `resolved 85694b2` (v0.8 Phase 8).
- **Resolution:** v0.8 Phase 8 — refactored `cli_convert_bip39_vectors.rs` to a single `bip39_trezor_english_corpus_full` test that loops over all 24 english entries via vendored `tests/bip39_trezor_vectors.json` (Trezor `python-mnemonic` SHA `b57a5ad77a981e743f4167ab2f7927a55c1e82a8`). Audit-matrix coverage 6/24 → 24/24 ✓.
- **Tier:** `v0.7.1-carry`

### `bip38-spec-vector-3-null-byte-passphrase` — V3 Unicode passphrase contains U+0000; not representable via argv

- **Surfaced:** v0.7.1 Phase 3.A (BIP test vector audit cycle).
- **Where:** `crates/mnemonic-toolkit/tests/cli_convert_bip38.rs::{encrypt,decrypt}_..._spec_vector3_unicode_nfc_passphrase` (`#[ignore]`'d); `crates/mnemonic-toolkit/src/cmd/convert.rs` passphrase input plumbing.
- **What:** BIP-38 §"Test vectors" vector 3 specifies a passphrase of 5 codepoints (U+03D2 + U+0301 + U+0000 + U+10400 + U+1F4A9). The U+0000 NULL byte cannot be passed via argv (POSIX `execve` truncates at NULL); the existing `--passphrase=-` stdin path also fails because `read_stdin_to_string` calls `.trim()`. To exercise this vector end-to-end the toolkit needs a NULL-safe input channel — e.g. `--passphrase-bytes-hex <hex>` accepting the raw byte sequence, or a stdin path that reads bytes verbatim (no trim, no UTF-8 reinterpretation). The `bip38` crate itself NFC-normalizes whatever string slice it receives; the gap is purely at the toolkit's input plumbing.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — new `--passphrase-stdin` flag with line-ending-only trim (preserves leading/trailing spaces + internal NULL). Both V3 ignored tests unignored and now active. Phase 1 review I1 added a separate `read_stdin_passphrase` helper distinct from `read_stdin_to_string` to prevent the trim issue.
- **Tier:** `v0.8`

### `electrum-seed-version-spike-pending` — Phase 4 step 0 interactive spike

- **Surfaced:** v0.8.1 Phase 4 (`design/agent-reports/v0_8-phase-4-electrum-seed-version-spike.md`).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/electrum.rs:33` — `ELECTRUM_SEED_VERSION_PIN = 17`.
- **What:** SPEC v0.8 §9 + IMPL_PLAN Phase 4 step 0 mandate an interactive spike against current Electrum (>= 4.5.x) to lock `ELECTRUM_SEED_VERSION_PIN` to a verified-cleanly-imports value.
- **Status:** `resolved` (2026-05-12 spike against Electrum 4.5.5).
- **Resolution:** Spike executed against Electrum 4.5.5 in `/tmp/electrum-spike-venv/`. Empirical result: a toolkit-emitted wallet file with `seed_version: 17` loads cleanly via `electrum --offline -w <file> listaddresses` (returns the expected BIP-84 receive set; Electrum migrates the in-memory state to FINAL_SEED_VERSION=59 on save). Source-code cross-check at `wallet_db.py:1195-1211` confirms `seed_version >= 12 → return seed_version` with no rejection at 17. Pin retained at 17 (the SPEC's "minimum cleanly-imports" specification matches 17; 59 is what Electrum WRITES, not the minimum it ACCEPTS). Full report: `design/agent-reports/v0_8-phase-4-electrum-seed-version-spike.md`.
- **Tier:** `v0.8.2`

### `electrum-tr-multi-a-pending-libsecp-taproot` — `--template tr-multi-a` refuses under `--format electrum`

- **Surfaced:** v0.8.1 Phase 4.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/electrum.rs` `emit()` guard; refusal fixture `tests/export_wallet/electrum_tr_multi_a_refusal.stderr`.
- **What:** Electrum's `wallet_db.py` does not currently ingest taproot multisig wallet shapes (pending libsecp-taproot integration in Electrum's signer surface). `--format electrum --template tr-multi-a` (or `tr-sortedmulti-a`) emits a byte-exact refusal with pointer to `--format bitcoin-core` (descriptor) or `--format sparrow` (which supports taproot multisig via descriptor-passthrough).
- **Status:** `open` (last upstream-checked 2026-05-12 against Electrum 4.5.5 source; `grep -E "'p2tr'|p2tr" electrum/transaction.py` returns no matches in the script-type enum, confirming taproot script type not yet wired).
- **Tier:** `v1+ / pending-electrum-firmware`

### `electrum-final-seed-version-drift` — track Electrum FINAL_SEED_VERSION upstream

- **Surfaced:** v0.8.1 Phase 4.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/electrum.rs` — `ELECTRUM_SEED_VERSION_PIN` doc-comment.
- **What:** Electrum's `wallet_db.py` `FINAL_SEED_VERSION` drifts upward over releases (4.5.5 = 59; the v0.8.1 SPEC §9 cited 71 from master at SPEC-write time). Toolkit pins to 17 (minimum cleanly-imports) and relies on Electrum's migration loader to walk forward. Track in case the loader ever drops support for old migration paths.
- **Status:** `open` (no fix scheduled; tracking only).
- **Tier:** `v1+ / informational`

### `electrum-root-fingerprint-roundtrip-quirk` — Electrum nulls `root_fingerprint` on load

- **Surfaced:** v0.8.1 Phase 4 step 0 spike (2026-05-12, Electrum 4.5.5).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/electrum.rs` `emit_electrum_standard_json` + `electrum/keystore.py` `BIP32_KeyStore`.
- **What:** The toolkit emits `keystore.root_fingerprint` per SPEC §9.1 (e.g., `"5436d724"`). Electrum 4.5.5's loader successfully imports the wallet, derives the correct BIP-84 addresses, but its re-serialized form has `"root_fingerprint": null` — the `_root_fingerprint` private attribute on the in-memory `BIP32_KeyStore` is not populated from the on-disk JSON field. Functionally inert for watch-only address derivation; required only for PSBT-with-origin flows. Likely an Electrum-side bug or intentional drop; cross-check against current master may surface a fix.
- **Status:** `open` (informational).
- **Tier:** `v1+ / informational`

### `green-native-multisig-pending-server-support` — `--format green` refuses multisig

- **Surfaced:** v0.8.1 Phase 5.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/green.rs` `emit()` guard; refusal fixture `tests/export_wallet/green_multisig_refusal.stderr`.
- **What:** Blockstream Green's multisig surface is server-mediated (Green Multisig Shield + Liquid), not a direct file-import shape. `--format green` is therefore singlesig-only; multisig templates return a byte-exact refusal with pointer to `--format bitcoin-core` (descriptor) or `--format sparrow`. Resolves once Green publishes a self-custody multisig file-import format.
- **Status:** `open`. Last upstream-checked **2026-05-12**: Green Help Center article `19340800530713-Set-up-watch-only-wallet` returns HTTP 403 to programmatic fetchers (Zendesk-hosted, browser-only). Status cannot be verified autonomously; entry remains open pending manual browser check.
- **Tier:** `v1+ / pending-green-server-support`

### `mnemonic-gui-schema-mirror` — companion to `bg002h/mnemonic-gui` schema gate

- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `mnemonic-gui-schema-mirror`; CI gate at `.github/workflows/schema-mirror.yml`.
- **Where:** This CLI's clap-derive `Args` blocks (currently `cmd/{bundle,verify_bundle,convert,export_wallet,derive_child}.rs`); the introspection subcommand at `cmd/gui_schema.rs`.
- **What:** The `mnemonic-gui` GUI mirrors this CLI's clap-derive flag surface at pinned tag `mnemonic-toolkit-v0.9.0` (was `v0.8.1` pre-v0.2). Any flag add / remove / rename / `conflicts_with` / `required_unless_present_any` change in this repo's CLI surface must land in lockstep with a companion `mnemonic-gui` PR that bumps the schema + the `pinned-upstream.toml` tag for this CLI. The `mnemonic-gui` CI gate runs `cargo install --locked --git <this-repo> --tag <pin>` + `cargo test --test schema_mirror`, so drift surfaces as a CI failure. Additionally, the GUI's `build.rs` codegen reads `crates/mnemonic-toolkit/src/cmd/convert.rs::NodeType::is_secret_bearing()` and `crates/mnemonic-toolkit/src/slot_input.rs::SlotSubkey::is_secret_bearing()` via `syn::parse_file` to populate its `SECRET_*` constants — drift in those impls is also caught by a runtime source-audit test in the GUI repo.
- **v0.2 update (2026-05-12, mnemonic-toolkit v0.9.0):** `mnemonic gui-schema` introspection subcommand shipped (SPEC §7 contract). The GUI consumes its JSON output instead of (or alongside) the `syn` codegen path. `cli_gui_schema.rs` (16 tests) pins the SPEC §7 contract on this side. The companion `mnemonic-gui` v0.2 Phase C.2 PR consumes the schema via `cargo run -p mnemonic-toolkit -- gui-schema` at build time.
- **Status:** `open` (mirror-invariant; tracking only — every flag-surface PR carries this lockstep work).
- **Tier:** `v1 / mirror-invariant`

### `mk-vectors-pretty-out-help-mismatch` — `mk vectors --pretty` help-text vs source behavior drift

- **Surfaced:** manual-gui v1.0 cycle batch 8 R0 review (2026-05-15), filed at toolkit `63397ef`+. Cited in `docs/manual-gui/src/70-mk/76-vectors.md` and `docs/manual-gui/src/90-appendices/94-release-history.md`.
- **Where:** `mk-cli` source at `crates/mk-cli/src/cmd/vectors.rs:23` (help-text doc-comment) and the mirror at `mnemonic-gui/src/schema/mk.rs:208` (schema help-text).
- **What:** `mk vectors --help` advertises `--pretty: Ignored when --out is supplied`. Source (vectors.rs:70-74 in the `write_per_fixture_files` arm) actually honors `--pretty` — each per-fixture .json file is written via `serde_json::to_string_pretty` when `pretty=true`. The manual-gui v1.0 manual sides with source-truth and notes the help-text drift.
- **Why deferred:** Source-side fix lives in the `bg002h/mnemonic-key` repo (mk-cli `cmd/vectors.rs`); the schema-mirror lives in the `bg002h/mnemonic-gui` repo. Three-cite fix at v1.1 cycle close.
- **Status:** `open`.
- **Tier:** `v1+ / cross-repo`
- **Companion:** intended companion entries in `bg002h/mnemonic-key/design/FOLLOWUPS.md` and `bg002h/mnemonic-gui/FOLLOWUPS.md` at the matching short-id; both currently missing (file with this entry at next cross-repo cycle).

### `secret-taxonomy-public-api-promotion` — promote `SECRET_*` to public toolkit crate API; retire `mnemonic-gui` build.rs source-walker

- **Surfaced:** 2026-05-16, during the `mnemonic-gui` v0.3.3 emergency security fix (closed at sibling `bg002h/mnemonic-gui` commit `6851d1b`). Architect-vetted in this session — see "Suggested fix" below.
- **Where:** New module `crates/mnemonic-toolkit/src/secret_taxonomy.rs` (proposed). Existing private modules at `crates/mnemonic-toolkit/src/cmd/convert.rs::NodeType::is_secret_bearing` (line 85) and `crates/mnemonic-toolkit/src/slot_input.rs::SlotSubkey::is_secret_bearing` (line 60). `crates/mnemonic-toolkit/src/lib.rs` (currently exposes `final_word`/`mlock`/`seed_xor`/`slip39`; would add `pub mod secret_taxonomy`).
- **What:** The `mnemonic-gui` repo's `build.rs` parses this toolkit's private `cmd/convert.rs` + `slot_input.rs` via `syn::parse_file` to extract the `is_secret_bearing()` match-arm sets and codegen `SECRET_NODE_TYPES` + `SECRET_SLOT_SUBKEYS` constants. The codegen is the GUI's workaround for the fact that these security-class taxonomies live in *private* toolkit modules (neither is `pub mod` in `main.rs` or re-exported from `lib.rs`), so the GUI has no versioned, addressable contract to depend on. Every fragility of the codegen path (cargo install sandbox having no adjacent toolkit checkout → empty `&[]` stub fallback → silent disable of `persistence::redact_for_persistence` → BIP-39 phrases leaking to `~/.config/mnemonic-gui/state.json` in plaintext; HIGH-severity bug in GUI v0.3.0..v0.3.2) descends from that contract gap. The GUI v0.3.3 tactical patch (committed canonical fallback in `build.rs` + drift gate) pins a second source of truth that must be manually kept in sync; the root issue remains.
- **Why deferred:** GUI v0.3.3 tactical fix is shipped + verified + released. Long-term fix requires a coordinated minor bump on both sides (toolkit `v0.14.0` + GUI `v0.4.0` lockstep). Filed here as the durable architectural cleanup; aim is the v0.4.x mnemonic-gui cycle.
- **Status:** `resolved 1a52612` (mnemonic-toolkit v0.14.0, 2026-05-16). Adds `pub mod secret_taxonomy` exposing `SECRET_NODE_TYPES` + `SECRET_SLOT_SUBKEYS` as `pub const &[&str]`. Per-variant parity tests in `cmd::convert::secret_taxonomy_parity_tests` (NodeType) + `slot_input::tests` (SlotSubkey) use a `declare_*_variants!` macro to make the variant array and the exhaustiveness check share a single source-of-truth list. R1 opus review caught a Critical (closure+driver design was not load-bearing) + 6 Importants; all folded in the same commit before tag. GUI half closed at `bg002h/mnemonic-gui@6fe44b6` (mnemonic-gui v0.4.0).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui/FOLLOWUPS.md` companion entry `secret-taxonomy-public-api-consumption` (resolved at `6fe44b6`).
- **Related memory:** `feedback_build_rs_stub_fallback_security_audit` (the pattern-class lesson) + `feedback_default_cargo_test_runs_sibling_dependent_tests` (same install-path-vs-CI-path divergence trap, opposite direction).

#### Suggested fix (architect-vetted, 2026-05-16)

##### Root structural issue

The GUI treats two safety-critical security predicates as **derivable from upstream source text at the GUI's build time**, with a degradation ladder that ends in a silent fallback. The taxonomy is owned by the toolkit binary's *private* modules — the GUI has no versioned, addressable contract to depend on. Codegen-from-source is the workaround for the missing contract; every fragility descends from that gap. The v0.3.3 patch added a second source of truth (committed `CANONICAL_FALLBACK_*` arrays) but did not fix the contract gap.

##### Recommended option: A — promote `SECRET_*` to public toolkit crate API

Toolkit exports `pub const SECRET_NODE_TYPES: &[&str]` and `pub const SECRET_SLOT_SUBKEYS: &[&str]` (string slices, **not** enum re-exports — avoids leaking enum semver surface) from a new `pub mod secret_taxonomy` re-exported by `lib.rs`. GUI depends on `mnemonic-toolkit` as a regular git dep and `use mnemonic_toolkit::secret_taxonomy::*`. No build.rs codegen.

**Why A wins (vs. the four alternatives evaluated):**
- **Eliminates the empty-array failure mode by construction.** GUI compile fails outright if the toolkit lib does not export the consts; no degradation ladder, no silent fallback.
- **`cargo install --git` works with zero ceremony.** Cargo's resolver pulls the toolkit lib through the normal dependency graph; no env vars, no clone scripts, no source-walker.
- **No load-bearing network call at install time** (unlike Option B: auto-clone by default).
- **Schema-mirror invariant becomes a stronger version-pin gate.** The drift question becomes "GUI's pinned `mnemonic-toolkit` git tag must include the secret-taxonomy module" — easier to enforce and reason about than parse-and-compare against private impl shape.

Alternatives considered + rejected: B (auto-clone always-on; load-bearing network), C (strict-mode build.rs; breaks `cargo install` for everyone), D (keep v0.3.3 tactical patch; preserves the codegen architecture + second source of truth), E (runtime JSON manifest via `mnemonic secret-taxonomy --json`; reintroduces silent-failure on a different axis).

##### Toolkit-side migration (v0.14.0)

1. New file `crates/mnemonic-toolkit/src/secret_taxonomy.rs`:
   ```rust
   //! Public secret-class taxonomy. Source of truth for the
   //! `is_secret_bearing()` predicates on `NodeType` / `SlotSubkey`,
   //! exposed for downstream tools (e.g., mnemonic-gui's persistence
   //! redaction) that cannot import the private enum modules.
   //!
   //! Drift-gated: see unit tests in `cmd/convert.rs` and `slot_input.rs`
   //! that assert `Self::as_str()` of every secret-bearing variant ∈
   //! these slices.

   pub const SECRET_NODE_TYPES: &[&str] = &[
       "phrase", "entropy", "xprv", "wif", "ms1", "bip38", "electrum-phrase",
   ];

   pub const SECRET_SLOT_SUBKEYS: &[&str] = &["phrase", "entropy", "xprv", "wif"];
   ```
2. `crates/mnemonic-toolkit/src/lib.rs`: add `pub mod secret_taxonomy;`.
3. In `cmd/convert.rs::NodeType::is_secret_bearing` and `slot_input.rs::SlotSubkey::is_secret_bearing`, add `#[cfg(test)]` unit tests that walk every variant `V` where `V.is_secret_bearing() == true` and assert `secret_taxonomy::SECRET_*_TYPES.contains(&V.as_str())`. This makes the in-tree predicate and the public constants a single source of truth, enforced at toolkit test time.
4. Update SPEC + manual chapter; minor bump to `v0.14.0` (new pub surface; pre-1.0 0.X-axis bump per repo policy).

##### GUI-side migration (v0.4.0)

1. `Cargo.toml`: add `mnemonic-toolkit = { git = "...", tag = "mnemonic-toolkit-v0.14.0" }` under `[dependencies]`.
2. **Delete** `build.rs` entirely (and the `[build-dependencies]` block).
3. `src/secrets.rs`: replace `include!(concat!(env!("OUT_DIR"), "/secrets_generated.rs"));` with `use mnemonic_toolkit::secret_taxonomy::{SECRET_NODE_TYPES, SECRET_SLOT_SUBKEYS};` (and `pub use` for crate-internal callers).
4. Delete `tests/secrets_canonical_fallback.rs`. Add a one-test backstop `tests/secret_taxonomy_pin.rs` asserting non-empty + minimum-membership (mirrors the v0.3.3 always-on guard).
5. `pinned-upstream.toml`: tag becomes documentary; load-bearing version pin lives in `Cargo.toml`.
6. `.github/workflows/schema-mirror.yml`: remove the `cargo-test-secrets-canonical-fallback` step; keep the flag-name parity job.
7. `install.sh`: no changes required.
8. GUI minor bump to `v0.4.0`.

##### One-cycle overlap (recommended)

In GUI `v0.4.0`, retain the v0.3.3 `CANONICAL_FALLBACK_*` constants AND add a compile-time assertion that they equal `mnemonic_toolkit::secret_taxonomy::SECRET_*`. Drop the fallback in `v0.5.0`. This catches a malformed-upstream-tag class of regression during the cycle where contributors still expect the old contract.

##### Non-obvious risks

1. **Toolkit dep tree bloats GUI compile** (bitcoin, miniscript, bip39, clap, etc. — none called from `secret_taxonomy`, all linked into the GUI's cargo graph; ~30-60s cold compile cost). Mitigation: feature-gate heavy modules under a `cli` default-on feature; GUI depends with `default-features = false, features = ["secret-taxonomy"]`. Optional; can defer if compile cost is acceptable.
2. **Toolkit becomes a load-bearing library API surface.** Renaming or relocating `secret_taxonomy` is now a semver event. Document in `lib.rs` and FOLLOWUPS.
3. **GUI git-dep pin must stay current** with toolkit's taxonomy releases. If `mnemonic-toolkit-v0.15` adds `ElectrumEntropy` as secret-bearing but GUI still pins `v0.14`, GUI silently lacks the new class. Mitigation: the CI flag-name schema-mirror gate (already running against the live `mnemonic` binary) catches the toolkit-side widening; add a parallel gate that asserts the GUI's pinned-toolkit `SECRET_*` equals the locally-installed `mnemonic`'s reported taxonomy (a future `mnemonic gui-schema` extension can emit it).
4. **`pub const &[&str]` vs `pub use SECRET_NODE_TYPES`.** Resist re-exporting `NodeType` / `SlotSubkey` enums — string slices are a far smaller semver surface and decouple GUI compile from internal enum shape (which evolves with every new node type).
5. **`mnemonic-toolkit` lib platform-compat audit.** Verify the lib builds cleanly on GUI's full platform matrix (macOS, Windows, Linux × x86_64 + aarch64) before the v0.14.0 release; `mlock.rs` uses `libc` and may need cfg-gating audit (already in place but should be revisited).
6. **One-shot lockstep required.** The toolkit v0.14.0 tag and the GUI v0.4.0 PR must be planned together (mirroring the manual-gui v1.0 lockstep pattern). Surface in both repos' FOLLOWUPS with `Companion:` lines per repo convention in `CLAUDE.md`.

### `secret-taxonomy-argv-superset-promotion` — promote `is_argv_secret_bearing()` wider set to a parallel public taxonomy

- **Surfaced:** 2026-05-16, toolkit v0.14.0 reviewer-loop (R1). Sibling to `secret-taxonomy-public-api-promotion` which promoted the narrower persistence-class set; this entry covers the wider argv-leakage set that includes MiniKey.
- **Where:** `crates/mnemonic-toolkit/src/secret_taxonomy.rs` (add `SECRET_NODE_TYPES_ARGV: &[&str]`); `crates/mnemonic-toolkit/src/cmd/convert.rs::NodeType::is_argv_secret_bearing` (line 117; cite refreshed v0.34.5; add parity test).
- **What:** `NodeType::is_argv_secret_bearing()` returns the **wider** set: `is_secret_bearing()` plus `MiniKey`. The v0.14.0 release promoted only the narrower `SECRET_NODE_TYPES` for persistence-redaction use. The wider set has no public mirror yet, so any future downstream consumer that needs argv-leakage protection (e.g., a GUI run-confirm modal that redacts the argv preview, per the `gui-run-confirm-modal-secret-redaction` GUI-side FOLLOWUP) will face the same private-symbol-scraping pressure that motivated v0.14.0's narrower promotion. Add `pub const SECRET_NODE_TYPES_ARGV: &[&str]` to `secret_taxonomy` as an additive minor surface (compatible with v0.14.0's stability contract). Add a parity test against `is_argv_secret_bearing` in the same shape as the existing narrower-set parity test.
- **Why deferred:** Reviewer flagged this as out-of-scope for v0.14.0 (which was explicitly scoped to closing the GUI v0.3.0..v0.3.2 persistence-leak bug). Filed here so the wider-set promotion doesn't silently fall off the radar.
- **Status:** resolved — v0.34.5. Added `pub const SECRET_NODE_TYPES_ARGV` to `secret_taxonomy.rs` (the wide argv-leakage set = `SECRET_NODE_TYPES` + `minikey`), mirrored by `NodeType::is_argv_secret_bearing` and locked by the new `secret_taxonomy_argv_parity_with_is_argv_secret_bearing` parity test (sibling of the narrow-set parity test). Additive public const — no GUI lockstep forced (the GUI's existing `SECRET_NODE_TYPES` snapshot is unchanged). The GUI-side `gui-run-confirm-modal-secret-redaction` consumer can now adopt it. Closed via cycle-prep recon (SHA `b17444b`).
- **Tier:** `cross-repo`
- **Companion:** `convert-minikey-stdout-redaction` (toolkit FOLLOWUP that tracks the narrow/wide asymmetry inside the toolkit); `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-run-confirm-modal-secret-redaction` (the GUI-side use case that would consume the wider set).

### `secret-taxonomy-feature-gate-toolkit-lib` — feature-gate the toolkit lib so consumers can pull `secret_taxonomy` without the heavy dep tree

- **Surfaced:** 2026-05-16, post-v0.14.0 architect-audit (Important #4). Surfaced separately from `secret-taxonomy-public-api-promotion` because the architect's original Option A blueprint contemplated it as an optional follow-up risk #1 mitigation, but no concrete FOLLOWUP was filed at v0.14.0 ship time.
- **Where:** `crates/mnemonic-toolkit/Cargo.toml` (`[features]` block, currently absent); `crates/mnemonic-toolkit/src/lib.rs` (`pub mod` declarations); downstream `mnemonic-gui/Cargo.toml` (`[dependencies] mnemonic-toolkit` — would add `default-features = false, features = ["secret-taxonomy"]`).
- **What:** Today `mnemonic-gui` v0.4.0+ depends on this crate as a regular cargo dep for `pub mod secret_taxonomy`. That pulls the toolkit's full dep tree into the GUI's cargo graph: `bitcoin`, `miniscript`, `bip39`, `bip38`, `clap`, `secp256k1`, `bech32`, `hmac`, `sha2`, `pbkdf2`, plus the new git-deps on `ms-codec` / `mk-codec` / `md-codec`. None of these are referenced from `secret_taxonomy`, but the cargo dep graph links them in. Cold-build cost for the GUI grew by ~30-60s on first-time builds (the architect's risk #1). Feature-gate the heavy modules behind a default-on `cli` feature; expose `secret-taxonomy` as a default-off small-surface feature. GUI depends with `default-features = false, features = ["secret-taxonomy"]`; toolkit's own bin builds with the default features (no change). Lib API for `secret_taxonomy` is preserved as-is.
- **Why deferred:** Architect risk #1 was flagged as optional ("Optional; can defer if compile cost is acceptable"). The v0.14.1 cfg-gate of `mlock` already unblocked the Windows compile failure that motivated this audit; the dep-tree cost is purely a quality-of-life issue for GUI cold builds, not a correctness gap.
- **Status:** `open`
- **Tier:** `v0.15` / `nice-to-have`
- **Companion:** Architect's "non-obvious risks" #1 from the resolved `secret-taxonomy-public-api-promotion` entry.

### `mnemonic-toolkit-cratesio-publish` — swap codec git deps for crates.io versions; publish toolkit to crates.io

- **Surfaced:** 2026-05-16, post-v0.14.2 crates.io publish audit (the cycle that yanked the vulnerable mnemonic-gui v0.3.0/v0.3.1 + published md-codec v0.33.1 + md-cli v0.5.2).
- **Where:** `crates/mnemonic-toolkit/Cargo.toml` (lines 20-22: `ms-codec` / `mk-codec` / `md-codec` are all `{ git = ..., tag = ... }`); the codec crate names already exist on crates.io (`ms-codec` v0.1.3, `mk-codec` v0.3.0, `md-codec` v0.33.1).
- **What:** `mnemonic-toolkit` cannot currently be published to crates.io because its three codec dependencies (`ms-codec`, `mk-codec`, `md-codec`) are git-only deps. crates.io's publish rules require version-or-version+git/path deps; pure-git deps are forbidden in published crates. The codec siblings are already on crates.io at versions that satisfy or supersede the toolkit's git pins:
   - `ms-codec`: toolkit pins git@v0.1.3; crates.io has v0.1.3 (✓ aligned)
   - `mk-codec`: toolkit pins git@v0.2.1; crates.io has v0.3.0 (newer — likely breaking; needs audit)
   - `md-codec`: toolkit pins git@v0.16.1; crates.io has v0.33.1 (significantly newer — multiple breaking minors; needs substantial audit)

   Work: bump toolkit's codec deps from git to crates.io versions in lockstep with whatever code changes those minor-bumps require, then `cargo publish --dry-run` and `cargo publish`. Once toolkit is on crates.io, `mnemonic-gui` can drop its git dep too (see `mnemonic-gui-cratesio-publish` companion).
- **Why deferred:** Significant lift — bumping `md-codec` from v0.16.1 to v0.33.1 is 17 minor versions of API drift to audit. Currently working around via the constellation's `install.sh` script (`cratesio=no` for toolkit). Only matters for users running `cargo install mnemonic-toolkit` directly (without `--git`); install-script users + GUI consumers (who pull via the script-managed git+tag path) are unaffected.
- **Status:** `open`
- **Tier:** `v1+ / nice-to-have`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` companion entry `mnemonic-gui-cratesio-publish` (blocked by this).

### `gui-schema-effect-on-dropdown-options-vocab` — dropdown-option-disable Effect grammar for SPEC §6.6 rows 9/10/11

- **Surfaced:** 2026-05-16, GUI v0.6.0 cycle (`mnemonic-toolkit-v0.17.0` + `mnemonic-gui-v0.6.0`) close. Filed per the §6.10.7 closing list — unblocked by the v3 SlotCount* predicate-machinery (now expressible) but the *effect* side requires a new Effect grammar.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.3 (Effect vocabulary extension); `crates/mnemonic-toolkit/src/cmd/gui_schema.rs::VisibilityProjection` (toolkit emitter); `mnemonic-gui/src/schema_check.rs::VisibilityProjection` (GUI consumer); `mnemonic-gui/src/form/widget.rs` (Dropdown widget — per-option-disable rendering).
- **What:** SPEC §6.6 rows 9/10/11 need a per-option Effect — e.g., row 9 disables `--threshold` values > N when slot-count is N; row 10 disables single-sig templates when N > 1; row 11 disables multisig templates when N == 1. v3 grammar offers `hidden` / `disabled` / `required` / `pin_value` — all acting on the whole flag. New variant candidate: `disable_options: { values: [...] }` for Dropdown FlagKind, paired with the `slot_count_*` Predicates already in v3.
- **Why deferred:** Predicate-machinery shipped in v3 unblocks the predicate side; Effect grammar extension is the next half. Out of v0.6.0 cycle scope; would need SPEC §6.10.3 extension + Dropdown widget rendering refactor.
- **Status:** `resolved c7ac604` — Batch B-1 cycle (`mnemonic-toolkit-v0.18.0` + `mnemonic-gui-v0.7.0`, 2026-05-16) shipped the disable_options Effect grammar (rows 10/11) + GUI-internal NumberMax::FromSlotCount FlagKind extension (row 9). Schema bumps `v3 → v4`. Row 9 closes GUI-side without a toolkit wire-format change (Option A per the v0.7.0 design doc — single-consumer pragma; promotable if a second `gui-schema` consumer ever appears). Toolkit emitter: VisibilityProjection::DisableOptions variant + 2 new bundle_conditional_rules entries (count 11 → 13). GUI consumer at `mnemonic-gui-v0.7.0` (`f86a696`) ships VisibilityProjection deserialize arm + Visibility::DisableOptions + NumberMax enum + render-time orthogonal composition.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-schema-effect-on-dropdown-options-vocab` (resolved at `f86a696`).

### `gui-schema-cross-slot-predicate-projection` — cross-slot relational predicate types for SPEC §6.6 rows 8/13/14

- **Surfaced:** 2026-05-16, GUI v0.6.0 cycle close. Filed per the §6.10.7 closing list — these rows need predicate types beyond the v3 `slot_count_*` extensions.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.2 (Predicate AST extension); `crates/mnemonic-toolkit/src/cmd/gui_schema.rs::Predicate` (toolkit emitter); `mnemonic-gui/src/schema_check.rs::Predicate` (GUI consumer); `mnemonic-gui/tests/gui_schema_conditional_drift.rs::synthesize_satisfying` (drift gate extension).
- **What:** §6.6 row 8 (cross-slot equality, e.g., "two slots must NOT share an xpub"), row 13 (BIP-388 distinct-key — pairwise distinctness across `@i` slots), row 14 (per-`@N` annotation consistency — e.g., if `@1.xpub` is `external`, all `@1.*` annotations must agree). Candidate Predicate variants: `slot_subkey_distinct: { subkey: "xpub" }`, `slot_annotation_consistent: { annotation: "external" }`.
- **Why deferred:** Predicate-machinery for relational predicates is missing in v3; full design requires SPEC §6.10.2 grammar extension. Out of v0.6.0 cycle scope.
- **Status:** `resolved 38ad066` — Batch B-2 close 2026-05-16. **Row 8 resolved Option A** (`mnemonic-gui-v0.7.1` `38ad066`): GUI-internal `slot_editor.rs::detect_slot_index_gaps` helper + inline warning banner. Pure GUI-side pre-check; no toolkit wire-format change (mirrors the v0.7.0 row-9 NumberMax::FromSlotCount pattern). 9 new test cells in `tests/slot_editor_contiguity.rs`. SPEC §6.10.7 row 8 flipped to `ENCODED v3 (GUI-internal)`. **Rows 13/14 wontfix** with rationale: row 13 (BIP-388 distinct-key) requires xpub derivation that the GUI can't replicate for phrase-bearing slots (toolkit-binding-logic duplication is high-cost low-value); row 14 (per-`@N` annotation consistency) requires descriptor-string parsing + cross-slot annotation cross-reference (similarly high-cost low-value). Both surface authoritatively at CLI run-time per §6.6 rows 13/14 stderr; GUI pre-check adds marginal UX over the existing CLI rejection. SPEC §6.10.7 "Runtime-deferred rules" section updated to enumerate row 13/14 under a new "CLI-rejection-sufficient (wontfix)" partition. All v0.6.0-cycle-close FOLLOWUPs now closed (Batch A v0.6.1 + Batch B-1 v0.7.0 + Batch B-2 v0.7.1).
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-schema-cross-slot-predicate-projection` (resolved at `38ad066`).

### `gui-schema-derive-child-meta-template-groups-spurious` — toolkit emits `meta.template_groups` on a subcommand with no `--template` flag

- **Surfaced:** 2026-05-16, GUI v0.6.0 cycle-close opus reviewer audit. Important finding (confidence 95): `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:244-259` `build_subcommand_meta` matches `name == "derive-child"` and emits a `template_groups` block, but `crates/mnemonic-toolkit/src/cmd/derive_child.rs` has ZERO `--template` references (grep-confirmed). SPEC §6.10.8 also lists derive-child as a template-consumer in error; toolkit test `derive_child_emits_meta_template_groups` enshrines the wrong invariant. Recurring `[feedback-r0-must-read-source-off-by-n]` failure mode.
- **Where:** `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:244-259` (the spurious match arm); `design/SPEC_mnemonic_toolkit_v0_5.md` §6.10.8 (matching mis-claim); `crates/mnemonic-toolkit/tests/cli_gui_schema_v3_extensions.rs` (`derive_child_emits_meta_template_groups`).
- **What:** Either (a) remove `derive-child` from the `build_subcommand_meta` match arm + delete the matching test cell + correct SPEC §6.10.8 prose, or (b) consciously document why derive-child gets the meta block despite having no `--template` widget. (a) is the source-faithful fix.
- **Why deferred:** Cosmetic — was scheduled for the next toolkit cycle, but folded faster as the v0.17.1 patch (the original "Folding into the next toolkit cycle" deferral rationale was outweighed by the cycle's small scope + lockstep-with-GUI-v0.6.1 patch cadence).
- **Status:** `resolved 7ed3784` — shipped at `mnemonic-toolkit-v0.17.1` (2026-05-16). Took option (a): P0 (`598b4ba`) removed `derive-child` from `build_subcommand_meta` match arm at `gui_schema.rs:248`; deleted `derive_child_emits_meta_template_groups` test cell + added replacement negative-cell `derive_child_omits_meta_template_groups` as regression guard; corrected SPEC §6.10.8 paragraph 2 prose + added parenthetical noting the v0.17.1 correction. TDD discipline: negative cell ran RED against unmodified source (panic showed the spurious `multisig: [...], single_sig: [...]` block), GREEN after the match-arm fix. 30 test binaries pass; 8 cells in `cli_gui_schema_v3_extensions` (was 8 before; one cell replaced). Companion GUI v0.6.1 picks up the cleaner JSON shape via the v0.17.1 pin bump.
- **Tier:** `cross-repo`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-schema-derive-child-meta-template-groups-spurious`.

### `gui-schema-classify-descriptor-subcommand` — diagnostic subcommand for the GUI canonicity-classifier drift gate

- **Surfaced:** 2026-05-16, v0.19.0 cycle Phase 7 end-of-cycle opus review I1.
- **Where:** new flag on `crates/mnemonic-toolkit/src/cmd/gui_schema.rs::run` (`--classify-descriptor <STR>`); manual chapter row in `docs/manual/src/40-cli-reference/41-mnemonic.md` under `mnemonic gui-schema`; mnemonic-gui's `tests/canonicity_drift.rs` (drift-gate kittest that shells out to the new flag).
- **What:** The GUI's `classify_descriptor_canonicity` (5-regex wrapper-form matcher at `mnemonic-gui-v0.8.0` `src/form/conditional.rs:99-126`) mirrors md-codec's `canonical_origin` table verbatim. To keep the GUI and toolkit views drift-free as md-codec evolves (e.g., adds a new canonical shape), the toolkit should expose a `mnemonic gui-schema --classify-descriptor <STR>` flag that prints `canonical` or `non-canonical` (exit 0 on success, exit 2 on parse failure). A new GUI kittest cell shells out to this flag for each fixture in the canonicity-corpus and asserts the verdict matches the GUI's classifier.
- **Why deferred:** Out of v0.19.0 scope per Phase 7 escalation gate. The GUI classifier is small (~50 LOC) and mirrors md-codec's 5 shapes verbatim today — drift risk is real but bounded; the cycle ships v0.19.0 + v0.8.0 without the drift gate. Closing this FOLLOWUP adds ~20 LOC toolkit + ~80 LOC GUI test + 1 manual chapter row.
- **Status:** `resolved 14c8119` — shipped at `mnemonic-toolkit-v0.20.0` (2026-05-17) as F2 of the three-FOLLOWUP fold cycle. New `GuiSchemaArgs::classify_descriptor: Option<String>` field + run() short-circuit branch at `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:52-61,1076-1085`. 8 toolkit test cells at `tests/cli_gui_schema_classify_descriptor.rs` (5 canonical shapes + 2 non-canonical + 1 parse-failure exit 2). Manual chapter row + cspell dict update. Lockstep with `mnemonic-gui-v0.8.1` drift gate at `tests/canonicity_drift.rs` (18-fixture corpus + floor assertion).
- **Tier:** `v0.20-feature` (small scope; can ride along with any v0.20+ patch).
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `canonicity-drift-gate-via-toolkit-classify-subcommand` (to file in lockstep).

### `gui-non-canonical-descriptor-banner-and-placeholder` — info banner + slot_editor path-placeholder for default-inferred non-canonical descriptors

- **Surfaced:** 2026-05-16, v0.19.0 cycle Phase 7 end-of-cycle opus review I2; predecessor lesson at `[[project-v0-18-1-v0-7-2-b1-bugfix-closed]]` (UX flaw missed by reviewer-loop + only caught by user-running-the-feature).
- **Where:** `mnemonic-gui-v0.8.0` `src/form/conditional.rs` (new `descriptor_non_canonical_default_path_notice` helper); `mnemonic-gui-v0.8.0` `src/form/slot_editor.rs:191` (path-field placeholder via egui `hint_text`); `mnemonic-gui-v0.8.0` `src/main.rs` (banner-render site adjacent to slot grid).
- **What:** v0.19.0 ships the canonicity-aware `--account` pin lift in `mnemonic-gui-v0.8.0` `src/form/conditional.rs::bundle()` (so the user's typed `--account N` flows through to the toolkit's default-path inference). What did NOT ship: (a) an inline info banner stating "non-canonical descriptor; @{N} will use default path m/48'/<coin>'/<account>'/2'"; (b) a `hint_text` placeholder in the slot_editor's path field showing the computed default. CLI users see the stderr info notice; GUI users get the correct behavior but NO visual cue that a default was assumed. Per the predecessor cycle's user-running-the-feature reviewer-dimension lesson, this surface is load-bearing for UX-perceptibility.
- **Why deferred:** Out of v0.19.0 scope per Phase 7 escalation. The canonicity-aware override behaves correctly; the perceptibility surface is a follow-on cosmetic. Targets a v0.8.1 GUI patch.
- **Status:** `resolved 14c8119 + mnemonic-gui-v0.8.1` — shipped at `mnemonic-toolkit-v0.20.0` (lockstep) as F3 of the three-FOLLOWUP fold cycle. New helpers in `mnemonic-gui/src/form/conditional.rs`: `coin_type_for_network` (inline mirror of `network::CliNetwork::coin_type`) + `descriptor_non_canonical_default_path_notice` (banner-text producer with empty-descriptor guard); new `FormState::number_value` accessor at `src/schema/mod.rs`; new `path_hint: Option<&str>` parameter on `src/form/slot_editor.rs::render` with `match (row.subkey, path_hint)` body for hint_text on Path subkey + empty value; `src/main.rs` orchestration with snapshot-then-render pattern + suppress-when-account-out-of-u32-range semantic (Phase 3 R0 I2 fold). 4+3 GUI test cells across `tests/descriptor_non_canonical_default_path_notice.rs` and `tests/slot_editor_path_hint_text.rs`. Banner text restored `--slot @N.path=m/...` literal flag-override syntax (Phase 3 R0 I1 fold). Companion drift gate at `mnemonic-gui/tests/canonicity_drift.rs` shells out to the `--classify-descriptor` flag from F2.
- **Tier:** `v0.8.1-gui-patch`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `gui-non-canonical-descriptor-banner-and-placeholder` (to file in lockstep).

### `verify-bundle-multi-cosigner-mk1-chunk-assembly` — verify-bundle's --mk1 / --bundle-json intake fails to reassemble multi-chunk cosigner mk1 cards

- **Surfaced:** 2026-05-16, v0.19.0 cycle Phase 7 end-of-cycle opus review C1-followup investigation. Pre-existing (not introduced by v0.19.0); confirmed by reproducing the same failure shape against a CANONICAL `wsh(sortedmulti(2,@0,@1))` descriptor (whose verify-bundle round-trip should have worked since v0.4.x).
- **Where:** root cause was at the synthesize-side `derive_mk1_chunk_set_id` callers (NOT the intake side as originally hypothesized). Four hazard sites at `crates/mnemonic-toolkit/src/synthesize.rs:246/391/561/754` derived csi from a shared `policy_id` stub for n>1 cosigners; downstream `emit_multisig_checks` BTreeMap-by-csi grouping collapsed all chunks into one bucket; mk-codec decode then errored with `ChunkedHeaderMalformed`.
- **What:** When a multi-cosigner descriptor (n ≥ 2) emits a bundle, each cosigner's mk1 string is chunked across multiple bech32-family strings (the canonical chunking for long encoded payloads). The bundle JSON envelope shape is `mk1: Vec<Vec<String>>` (outer per cosigner, inner per chunk). Two intake failure shapes:
  - **Flat `--mk1` repetition** (passing each chunk as a separate `--mk1` occurrence flat across all cosigners): md-codec's chunked reader sees `total_chunks = 2` from the header of the FIRST cosigner's chunk, then errors with `ChunkedHeaderMalformed("received 4 chunks, header declares total_chunks = 2")` when the second cosigner's first chunk arrives.
  - **`--bundle-json <path>`** (v0.4.3+ envelope intake): cosigner[1]'s mk1 is reported "not supplied" while cosigner[0]'s mk1 fails decode — the envelope reader's per-cosigner unpacking does not correctly hand each cosigner's chunk-vec to the per-cosigner decoder.
- **Why deferred:** Out of scope for v0.19.0 cycle (Phase 7 user-run-the-feature smoke confirmed `bundle --self-check` round-trip succeeds for the non-canonical default-inferred bundle, which is the load-bearing C1 verification). The mk1-chunk-assembly bug class predates v0.19.0 and affects canonical multi-cosigner bundles equally; v0.19.0's verify-bundle parity work is correctly anchored at the synthesize side. Fixing the intake side is a separate cycle.
- **Status:** `resolved 14c8119` — shipped at `mnemonic-toolkit-v0.20.0` (2026-05-17) as F1 of the three-FOLLOWUP fold cycle. Phase 0 reconnaissance confirmed C1 hypothesis (all 4 mk1 chunks shared `csi=907005`; xpub fingerprints distinct between cosigners). Phase 1 R0 surfaced 1 Important (4th hazard site at `:391` in `synthesize_multisig_full` was originally missed; plan-doc conflated this function with `synthesize_multisig_watch_only` at `:561`). All 4 sites now derive csi from `<loop_var>.xpub.fingerprint().to_bytes()`. Single-sig n=1 paths at `:228` + `:737` use `&stub` (bare) and are byte-identity-pinned via `cli_verify_bundle_multi_cosigner_mk1.rs` cell 5 + `tests/fixtures/v0_20_0_single_sig_bip84_bundle.json`. 6 new test cells exercise `bundle | verify-bundle --bundle-json` round-trip for canonical wsh-sortedmulti (template mode), non-canonical wsh(andor(...)) (descriptor mode), 3-cosigner non-canonical, flat `--mk1` argv repetition, and `--self-check` sanity (gap-B documented at `.v0_20_0-phase0-artifact.md`: self-check decodes per-cosigner chunks separately, bypassing csi-grouping). User's flagship v0.19.0 `wsh(andor(...))` 3-of-3 invocation round-trips end-to-end.
- **Tier:** `v0.20-bugfix` (or sooner if user demand surfaces — the mk1-chunk-assembly intake bug is invisible to users running `bundle --self-check` or `bundle | shasum` workflows; it primarily blocks the verify-bundle three-card forensic check on multi-cosigner bundles)
- **Companion:** none yet.

### `manual-yml-bind-real-mnemonic-bin` — CI manual.yml uses placeholder `MNEMONIC_BIN=true`, weakening flag-coverage gate

- **Surfaced:** 2026-05-17, v0.20.0 cycle Phase 5 end-of-cycle opus review I2.
- **Where:** `.github/workflows/manual.yml:81` invokes `make -C docs/manual lint` with `MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=mk`. The flag-coverage gate at `docs/manual/tests/lint.sh:84` then runs `true gui-schema --help` (exits 0 with empty stdout); the per-subcommand flag-extraction loop at `:85-87` short-circuits via `warn "no flags parsed... skipping"`.
- **What:** The bidirectional `cli-surface-mirror` invariant relies on the flag-coverage lint asserting that every clap-derive `--flag` is mirrored in the manual chapter. With placeholder binaries, that assertion never runs in CI — drift can ship and the manual lint will pass green. Local pre-commit lint catches the gap because developers run with real binaries on PATH, but the CI gate is a fig leaf.
- **Why deferred:** v0.20.0 cycle F2 inherited the gap; F2's `--classify-descriptor` is correctly documented at `41-mnemonic.md:962-996` and verified locally, but the CI gate didn't enforce it. Closing this FOLLOWUP requires either (a) building the four real binaries in `manual.yml` via cargo-install pinned-tag pre-step (slow but authoritative); or (b) bumping a probe binary that exits 1 on unknown subcommand so the placeholder approach fails-loud instead of silently skipping.
- **Status:** `resolved-partial 52f33f7` — manual-v0.2.0 cycle landed option (a) for the mnemonic-side: `cargo build --bin mnemonic` pre-step in `manual.yml` + `make audit MNEMONIC_BIN=<built-path>` invocation. mnemonic-side flag-coverage gate is now real-binary-bound. MD/MS sibling-bin promotion deferred to successor entries `manual-md-bin-real-binary-promote` + `manual-ms-bin-real-binary-promote` (filed below).
- **Tier:** `v0.20+-ci-hygiene`
- **Companion:** none.

### `canonicity-drift-gate-floor-too-lenient` — F2 drift gate floor of 50% allows broad regression to pass silently

- **Surfaced:** 2026-05-17, v0.20.0 cycle Phase 5 end-of-cycle opus review I4.
- **Where:** `mnemonic-gui/tests/canonicity_drift.rs:138` floor assertion `classified >= FIXTURES.len() / 2`.
- **What:** The drift gate iterates 18 canonical/non-canonical fixtures; today 15 classify successfully and 3 parse-fail (BIP-388 `@N/**` shorthand). The 50% floor (= 9) allows the toolkit's parser to regress such that only 9 of 18 fixtures classify and the gate still passes. The `feedback-ci-snapshot-test-substring-vacuity` lesson recommends tight floors; current floor has ~3x headroom which is generous.
- **Why deferred:** Bumping the floor to e.g. `FIXTURES.len() - 4` (allowing the 3 known parse-fails plus 1 cushion) is brittle if a future toolkit change makes one of the BIP-388 fixtures start parsing successfully (the floor would suddenly trip on the FIRST parse-fail it sees). Right answer is probably a per-fixture classified expectation table rather than a count-only floor.
- **Status:** `open`
- **Tier:** `v0.20+-test-hygiene`
- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry to file in lockstep.

### `descriptor-mode-all-or-nothing-slot-set` — should descriptor mode reject mixed phrase/watch-only slot sets?

- **Surfaced:** 2026-05-17, v0.21.0 cycle plan §1 D8 item 1.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs:1077-1237` (per-slot resolver loop). UX question raised by the v0.21.0 SPEC §5.8 per-slot emission rule: when a user supplies `--slot @0.phrase=X --slot @1.xpub=Y --slot @2.phrase=Z`, the toolkit happily emits a "hybrid" bundle with `ms1 = ["pop", "", "pop"]`. Is that ever a meaningful real-world configuration, or should the CLI refuse mixed-mode slot sets?
- **What:** The SPEC §5.8 emission rule defines the wire-format encoding for hybrid bundles, but doesn't take a position on whether the toolkit should EMIT them. A bundle where slots are inconsistently configured (some phrase-bearing, some watch-only) is hard to reason about in a real deployment: inheritance ceremonies typically want either "I have all phrases, just emit everything" OR "I have NONE of the phrases, this is watch-only". Mixed-mode is most plausibly a user error. A future cycle could add a CLI refusal guard (`error: descriptor mode requires either all-phrase or all-watch-only slot bindings; got mixed`) gated behind a `--allow-hybrid` opt-in flag for advanced use cases.
- **Why deferred:** UX question, not a correctness gap. The v0.21.0 cycle's scope was conformance to SPEC §5.8 emission, not slot-binding policy. Filing for the v0.22+-feature tier so a future cycle can revisit with full UX-research context.
- **Status:** `open`
- **Tier:** `v0.22+-feature`
- **Companion:** none.

### `synthesize-descriptor-deduplicate-with-unified` — refactor synthesize_descriptor and synthesize_unified into a shared helper

- **Surfaced:** 2026-05-17, v0.21.0 cycle plan §1 D8 item 2.
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs:200-275` (`synthesize_descriptor` body) + `crates/mnemonic-toolkit/src/synthesize.rs:709-774` (`synthesize_unified` body). After v0.21.0's per-slot ms1 emission fix, both functions now iterate `cosigners`/`slots` and emit ms1/mk1 the same way (cf. plan §2.2 mirror-comment at synthesize.rs:255-257 vs synthesize.rs:710-723).
- **What:** The two synthesis paths now share most of their logic (ms1 per-slot emission + mk1 per-cosigner chunking + md1 splitting). A shared helper `fn emit_unified_cards(descriptor, cosigners, privacy_preserving) -> Result<Bundle>` would deduplicate the bodies and reduce the maintenance surface. Pre-v0.21.0, the divergent ms1 emission rule (ms1[0]-only-for-descriptor vs per-slot-for-unified) blocked this refactor; the per-slot fix unlocks it.
- **Why deferred:** Pure refactor; no user-visible behavior change. v0.21.0 scope was the SPEC §5.8 conformance fix + manual regen; deduplication is a separate concern. The two call sites (cmd/bundle.rs:1259 + cmd/verify_bundle.rs:673) currently distinguish descriptor-mode vs template-mode via the calling code path, which a refactor could preserve via a `mode: BundleMode` enum dispatch inside the shared helper.
- **Status:** `open`
- **Tier:** `v0.22+-refactor`
- **Companion:** none.

### `inheritance-example-transcript-coverage` — add `41-inheritance.{cmd,out}` transcript fixture to `docs/manual/tests/transcripts/`

- **Surfaced:** 2026-05-17, v0.21.0 cycle plan §1 D8 item 3.
- **Where:** `docs/manual/src/40-cli-reference/41-mnemonic.md:209-216` (bundle command) + `:351-357` (verify-bundle command) — the inheritance worked example. Today's `make verify-examples` lint pass at `docs/manual/Makefile` doesn't cover chapter 41's `wsh(andor(...))` recipe.
- **What:** Add a transcript pair `docs/manual/tests/transcripts/41-inheritance.cmd` + `41-inheritance.out` that drives the chapter-41 bundle + verify-bundle commands end-to-end against the installed `mnemonic` binary, diffs the captured stdout against expected, and fails CI if the manual's documented output drifts from the binary's actual output. This would catch a future regression to the SPEC §5.8 per-slot emission (or any other example-block drift) at lint time instead of at user-complaint time.
- **Why deferred:** Phase 4 architect-must-run-prose discipline caught the post-v0.21.0 ms1 strings byte-exact in this cycle; the CI gap is real but the manual content is currently correct. Adding transcript coverage is test-hygiene work that can be done in a dedicated docs cycle.
- **Status:** `resolved f46ac70` (P1a capture) + `52f33f7` (P3 verify-examples.sh extension + CI wiring). The actual transcript-pair path is `docs/manual/transcripts/41-inheritance.{cmd,out}` (NOT `docs/manual/tests/transcripts/` as the body cited — this FOLLOWUP body had the wrong path infix; the actual `docs/manual/transcripts/` dir was the existing convention from v0.22's first 5 pairs). The recipe runs end-to-end in CI via `make audit` with the real v0.28.2 mnemonic binary.
- **Tier:** `v0.22+-test-hygiene`
- **Companion:** none.

### `pre-v0_21-bundle-shape-detection` — verify-bundle stderr advisory when supplied bundle matches pre-v0.21.0 broken descriptor-mode shape

- **Surfaced:** 2026-05-17, v0.21.0 cycle plan §1 D8 item 4.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (case-4 ms1 emission at lines ~1273-1276) + SPEC §5.8 v0.21.0 migration note paragraph at `design/SPEC_mnemonic_toolkit_v0_5.md:153`.
- **What:** When v0.21.0 verify-bundle encounters a bundle where `ms1[0]` is populated but `ms1[1+]` are all `""` AND the supplied slots are all `--slot @i.phrase=` (signaling the user thinks the bundle was full-mode), the failure mode is `ms1_decode[i]: fail cosigner[i] ms1 expected (full-mode bundle) but not supplied` per SPEC §5.7 case 4. The error is cryptic without context. A future enhancement: detect this specific shape (pre-v0.21.0 descriptor-mode bundle with phrases for @1+) and print a stderr advisory `info: bundle ms1 shape matches pre-v0.21.0 descriptor-mode @0-only emission. Regenerate with v0.21.0+ to fix per SPEC §5.8 emission rule.` before the check output.
- **Why deferred:** UX improvement; the SPEC migration note + the GitHub release notes already document the migration. The error message itself is technically accurate per SPEC §5.7 case 4. A stderr advisory would be a nice-to-have for users replaying old bundles but is not load-bearing.
- **Status:** `open`
- **Tier:** `v0.22+-ux`
- **Companion:** none.

### `self-check-ms1-iteration-audit` — should `self_check_bundle` iterate ms1 like verify-bundle now does?

- **Surfaced:** 2026-05-17, v0.21.0 cycle plan §1 D8 item 5. Companion to v0.20.0 cycle's `feedback-self-check-bypasses-csi-grouping` memo.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::self_check_bundle::MkField::Multi` (at bundle.rs:1478-1504 per [[feedback-self-check-bypasses-csi-grouping]] anchor); ms1 iteration is similarly skipped in self-check.
- **What:** The `--self-check` flag at `mnemonic bundle` only validates per-cosigner mk1 chunks decode; it does NOT iterate ms1 to validate the per-slot emission rule. Post-v0.21.0, this gap is more visible: self-check could regress to the @0-only emission pattern silently. Audit: should `self_check_bundle` add an ms1 iteration that asserts every phrase-bearing slot's ms1 decodes back to the supplied entropy? This would make self-check a regression guard for the SPEC §5.8 emission rule, complementing the verify-bundle --bundle-json round-trip in the new Cell 7 integration test.
- **Why deferred:** Test-coverage gap, not a correctness gap. The v0.21.0 Cell 7 (`descriptor_mode_3_of_3_emits_per_slot_ms1_post_v0_21`) and the SPEC §5.8 amendment together cover the user-facing regression surface; self-check is an internal sanity check. Audit + fix is a separate cycle.
- **Status:** `open`
- **Tier:** `v0.22+-test-coverage`
- **Companion:** none.

### `api-harvest-drift-on-synthesize-descriptor-signature` — technical-manual API table documents the dropped 3rd arg

- **Surfaced:** 2026-05-17, v0.21.0 cycle plan §1 D8 item 6 + R0 I7 fold.
- **Where:** `docs/technical-manual/transcripts/api-harvest-mnemonic-toolkit.md:261` documents the v0.20.x signature `synthesize_descriptor(descriptor, cosigners, entropy, privacy_preserving)`. Post-v0.21.0 the function is `synthesize_descriptor(descriptor, cosigners, privacy_preserving)` (3-arg).
- **What:** The technical manual's API harvest table is generated from a stale snapshot of the public surface. After v0.21.0 drops `entropy: Option<&[u8]>` from `synthesize_descriptor`, the doc line drifts. Closing this FOLLOWUP requires either (a) regenerating the API harvest at the next technical-manual cycle (which already has its own update cadence per the project's `docs/manual/` vs `docs/technical-manual/` distinction), or (b) adding a CI lint that grep-asserts each documented signature exists verbatim in the live source.
- **Why deferred:** `docs/technical-manual/` is a distinct surface from `docs/manual/` per CLAUDE.md. The v0.21.0 cycle's mirror invariant covers `docs/manual/` only. Technical-manual updates ride a separate cadence and shouldn't block toolkit releases.
- **Status:** `open`
- **Tier:** `v0.22+-doc-hygiene`
- **Companion:** none.

### `verify-bundle-xpub-parent-fingerprint-derivation` — extend xpub-vs-md1 parent_fingerprint check to depth ≥ 2

- **Surfaced:** 2026-05-17, v0.24.0 Tranche A.1 end-of-phase architect review (folded `verify-bundle-watch-only-xpub-path-internal-consistency` as stderr-warning helper, but architect noted the parent_fingerprint check is narrower than plan).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` — `emit_watch_only_xpub_path_cross_check` parent_fingerprint branch around `:1996-2029`. v0.24.0 implementation did structural sanity only: depth-0 enforces BIP-32 all-zeros invariant; depth-1 compares against claimed master fingerprint (mk1 origin_fingerprint or md1 TLV fingerprints); depth ≥ 2 was SKIPPED.
- **What:** Original framing said "derive parent xpub from supplied mk1" — this is **structurally impossible**: BIP-32 `derive_pub` is parent→child only; child→parent is the cryptographic one-way step. v0.25.0 corrects the mechanism: derive the parent xpub from the supplied **ms1** (seed) — `ms_codec::decode → entropy → BIP-39 → seed → master xpriv → derive_priv at path[..N-1] → from_priv → fingerprint`. New helper `emit_full_path_parent_fingerprint_check` at `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (after `emit_watch_only_xpub_path_cross_check`). Wired into `run_full`, `run_watch_only`, and `run_multisig`. Reuses passphrase / language / network from `VerifyBundleArgs` (no NOTICE-and-skip fallback). For the watch-only depth ≥ 2 case (no ms1 supplied for that cosigner), emits an explicit stderr NOTICE `notice: cosigner[{idx}] mk1 parent_fingerprint at depth {N} unverified (requires ms1 to derive parent xpub)` — explicit wontfix partition documenting the BIP-32 one-wayness ceiling. Failure mode: stderr WARNING / NOTICE only; verify-bundle exit code unchanged (permissive-input / expressive-output).
- **Status:** `RESOLVED in v0.25.0` (mechanism correction + ms1-driven derivation + watch-only NOTICE partition).
- **Tier:** `v0.25.0` (promoted from `v1+` per user direction).
- **Companion:** none.

### `gui-schema-global-flag-id-disjointness-debug-assert` — guard against future global-vs-local flag-id collision in v5+ emitter

- **Surfaced:** 2026-05-17, v0.24.0 Tranche B.1 end-of-phase architect review.
- **Where:** `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:1029-1031` skips args by `global_ids` set lookup; `:1042-1047` then re-emits globals via explicit list. `seen_flag_names` HashSet at `:1043` is dead defense — only the `global_ids` skip is load-bearing. Works correctly today because the only global flag is `--no-auto-repair` (boolean, no naming collision risk against subcommand-local flags). If a future cycle adds a global flag whose long-name collides with a subcommand-local flag of a DIFFERENT clap ID, the global would shadow the local.
- **What:** Add either (a) a debug-assert in `emit_flag` confirming `global_ids` IDs are disjoint from each subcommand's local `arg.get_id()` set, OR (b) a one-shot test cell asserting the same invariant. Optionally remove the `seen_flag_names` dead defense or rewrite it to be load-bearing (e.g., assert no name collisions across global-and-local rather than skip silently).
- **Why deferred:** Not a current correctness issue (no global flags besides `--no-auto-repair` today). Cheap to fix later; safer to file than to fold mid-cycle without expanding scope.
- **Resolution:** RESOLVED in v0.25.0 Phase 3 — added `pub(crate) fn assert_global_local_id_disjointness` helper in `cmd/gui_schema.rs` (load-bearing debug_assert; called from `build_subcommand` before global propagation). Deleted dead `seen_flag_names` defense per option (a) + supplementary cleanup. Cells: `global_local_id_disjointness_invariant_holds_in_current_schema` (integration, both debug + release) + `global_local_collision_triggers_debug_assert` (`#[cfg(debug_assertions)]`-gated `#[should_panic]` unittest).
- **Status:** resolved v0.25.0 cycle
- **Tier:** `v0.25.0`
- **Companion:** none.

### `cmd-repair-inspect-helper-duplication` — extract `count_dashes` / `expand_dashes` / `resolve_groups` shared between `cmd/repair.rs` and `cmd/inspect.rs`

- **Surfaced:** 2026-05-17, v0.24.0 Tranche C.1 end-of-phase architect review (during the D34/I5 fold).
- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/repair.rs` — `count_dashes`, `expand_dashes`, `resolve_groups` (~80 LOC).
  - `crates/mnemonic-toolkit/src/cmd/inspect.rs` — byte-for-byte duplicates of the same three helpers (~80 LOC, modulo a `inspect:` vs `repair:` substring difference in two `BadInput` error messages).
  - `validate_flag_hrp` already extracted to `crates/mnemonic-toolkit/src/repair.rs` in the C.1 D34/I5 fold (this FOLLOWUP picks up the remaining `cmd/`-local helpers).
- **What:** Move the three shared helpers to a new `crates/mnemonic-toolkit/src/cmd/_shared_card_args.rs` (or co-locate next to `validate_flag_hrp` in `src/repair.rs`). Signature for `resolve_groups` would need to be parameterized over the per-subcommand error-prefix string (`"repair"` vs `"inspect"`) and the args-struct shape — likely cleanest via a small trait or a free function consuming `(Option<String>, &[String], &[String], &[String])` (ms1/mk1/md1/extra_strings + a `&'static str` subcommand-name for error messages). Update both `cmd::repair::run` + `cmd::inspect::run` to delegate. Subsumes any future verify-bundle equivalent if positional intake grows beyond the three-flag set.
- **Why deferred:** Tranche C.1 architect review surfaced this as Important #3 alongside D34 (Critical) + I5 (Important — confusing case-mismatch error). The D34/I5 folds were on the critical path for v0.24.0 ship; the helper-extraction refactor is purely DRY (no correctness gap) and was deferred to keep C.1's diff focused. Pattern is well-known: `synthesize_unified` extraction was the analogous earlier cycle (see CLAUDE.md memory `synthesize-unified-is-cli-hotpath`); duplicated helpers between sibling `cmd/` modules carry the same maintenance-drift risk.
- **Resolution:** RESOLVED in v0.25.0 Phase 1 — co-located next to `validate_flag_hrp` in `crates/mnemonic-toolkit/src/repair.rs` (option B). Added `pub(crate) trait CardArgs` with `ms1()`/`mk1()`/`md1()`/`extra_strings()` accessors; implemented for both `RepairArgs` + `InspectArgs`. Three helpers (`count_dashes`, `expand_dashes`, `resolve_groups`) now consume `&impl CardArgs`. `resolve_groups` parameterized over `subcmd_name: &'static str` for the 2 error-message substrings. Net -19 LOC across the cmd files; 1142 baseline tests preserved.
- **Status:** resolved v0.25.0 cycle
- **Tier:** `v0.25.0`
- **Companion:** none.

### `convert-inspect-auto-fire-tty-gate-asymmetry` — convert/inspect auto-fire lacks TTY gate; design-intent says interactive-only

- **Surfaced:** 2026-05-17, v0.24.0 Tranche A.1 end-of-phase architect review (during the `MNEMONIC_FORCE_TTY` doc-promote fold).
- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/convert.rs:919` calls `crate::repair::try_repair_and_short_circuit(...)` UNCONDITIONALLY when `!no_auto_repair`.
  - `crates/mnemonic-toolkit/src/cmd/inspect.rs:85` same pattern.
  - `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:155` consults `MNEMONIC_FORCE_TTY` + `is_terminal()` correctly (D18 design).
  - `docs/manual/src/40-cli-reference/41-mnemonic.md:505-508` reads "Interactive users see the helpful auto-fire UX; piped consumers see the v0.22.0-and-earlier behavior unchanged" — generic-sounding but only true for verify-bundle.
  - v0.24.0 cycle Tranche A.1 narrowed the `MNEMONIC_FORCE_TTY` scope claim at `:537-541` to verify-bundle-only, surfacing the asymmetry.
- **What:** Two resolutions:
  - (a) **Extend TTY gate to convert/inspect** for design-intent parity. Wraps `try_repair_and_short_circuit` calls in `if MNEMONIC_FORCE_TTY-aware is_terminal()` blocks. This matches the stated design intent + makes the env-var semantically uniform.
  - (b) **Document the asymmetry intentionally.** Update the manual at `:505-508` to explicitly note convert/inspect auto-fire is unconditional regardless of TTY (existing behavior is preserved; the doc just reads as design-intent).
  - User+architect choice; (a) is the more principled fix, (b) is the cheaper backfill.
- **Why deferred:** Latent issue, not a correctness regression. v0.24.0 A.1's scope was force-tty doc-promote, NOT TTY-gate-mechanism extension. Folding (a) would require expanding A.1 scope mid-cycle. Filing for future-cycle resolution; folding (b) is cheap if user prefers that approach.
- **Resolution:** RESOLVED in v0.25.0 Phase 2 — chose option (a). Single-gate-at-entry pattern: `let effective_no_auto_repair = crate::repair::resolve_no_auto_repair(no_auto_repair);` at top of `cmd/convert.rs::run` + `cmd/inspect.rs::run`; downstream `try_repair_and_short_circuit` sites thread `effective_no_auto_repair`. Mirror verify_bundle.rs:168-173 pattern. Manual L504-541 hoist + revert: A.1's narrowing paragraph reverted; new shared `### Auto-fire behavior (all three subcommands) (v0.25.0)` H3 section above `## mnemonic convert`. Behavior change documented in CHANGELOG `### Changed` with before/after example. 6 cli_auto_repair cells updated with `MNEMONIC_FORCE_TTY=1`; 3 new cells lock TTY-negative legacy path. Piped users opt back in via `MNEMONIC_FORCE_TTY=1` — same mechanism the GUI uses globally.
- **Status:** resolved v0.25.0 cycle
- **Tier:** `v0.25.0`
- **Companion:** none.

### `verify-bundle-empty-ms1-watch-only-sentinel-or-explicit-flag` — pre-v0.24.0 `--ms1 ""` watch-only sentinel hard-fails v0.24.0 strict-HRP gate

- **Surfaced:** 2026-05-18, v0.25.0 Phase 4 R0 architect review. The pre-v0.24.0 multi-cosigner watch-only convention was `--ms1 <s1> --ms1 ""` (empty string `""` per SPEC §5.8 as the watch-only sentinel for a specific cosigner). v0.24.0 §2.C.1 added a strict per-flag HRP validation gate (`crate::repair::validate_flag_hrp` called from `cmd/verify_bundle.rs::run` at `:162`) which rejects empty-string `--ms1` values for the "HRP must be lowercase canonical 'ms'" check. v0.25.0 Phase 4 worked around this by omitting the flag entirely for watch-only cosigners (`supplied_ms1.get(idx).map(...).unwrap_or("")` falls through cleanly), and updated the `cmd/verify_bundle.rs:62-67` doc-comment to describe omission as the new sentinel convention. But the SPEC §5.8 reference text still describes empty-string-as-sentinel; this FOLLOWUP captures the design question.
- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:62-67` — doc-comment now describes omission (correct post-v0.25.0); historical pre-v0.24.0 doc said empty-string.
  - `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:162` — `validate_flag_hrp("--ms1", "ms", v)?` rejects `""` strings at clap-parse-time.
  - SPEC §5.8 prose — describes empty-string sentinel; needs reconciliation OR explicit version-fork note.
- **What:** Choose one of:
  - (a) **Document the omission-as-sentinel convention canonically in SPEC §5.8.** Update SPEC to remove the empty-string-sentinel claim; add omission-as-sentinel description with `--ms1 <s1>` (skip cosigner) examples. Cheapest fix; treats the strict-HRP gate as authoritative.
  - (b) **Add an explicit `--watch-only-cosigner-index N` flag** that takes an integer index, marking the Nth cosigner as watch-only without needing any ms1 value at all. More expressive but adds flag surface.
  - (c) **Relax `validate_flag_hrp` to accept empty strings as the SPEC §5.8 sentinel** (with a separate check that rejects non-empty non-`ms1`-HRP values). Restores pre-v0.24.0 behavior but creates a hidden empty-string-special-case in the validator.
- **Why deferred:** v0.25.0 Phase 4's doc-comment fix is sufficient for the cycle close; the deeper SPEC reconciliation OR flag-surface change is a v0.26+ design question. No correctness regression — the new convention works.
- **Resolution:** RESOLVED in v0.25.1 (chose option (c) per user direction — relax `validate_flag_hrp` to accept empty strings, with explicit NOTICE-on-stderr to guard the accidental-empty-shell-variable footgun, plus option (a) SPEC §5.8 wording clarification documenting both equivalent CLI input forms). Restores the pre-v0.24.0 positional empty-string sentinel convention so middle-cosigner watch-only (e.g., `--ms1 <s0> --ms1 "" --ms1 <s2>`) is expressible — flag-omission alone can't represent this case. Mechanism: `crate::repair::validate_flag_hrp` early-returns `Ok(())` on `value.is_empty()` (alongside the existing `"-"` stdin exemption); `cmd::verify_bundle::run` iterates `args.ms1` and emits the NOTICE per skipped cosigner; SPEC §5.8 gains a new "CLI input forms" subsection. 1 new cell `watch_only_empty_ms1_sentinel_marks_cosigner_skip_with_notice` exercises middle-cosigner skip (un-expressible via flag omission alone). Released 2026-05-18 as v0.25.1 (single-FOLLOWUP patch).
- **Status:** resolved v0.25.1 cycle
- **Tier:** `v0.25.1`
- **Companion:** none.

### `bsms-first-address-verify` — toolkit-side first-address derivation + mismatch WARNING for 6-line BSMS Round-2

- **Surfaced:** 2026-05-18, Phase 2 R0 architect review of v0.26.0 wallet-import cycle.
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:198-205` — `parse_six_line` path deferred verification; comment `let _ = &audit;` is the no-op fence.
  - `design/SPEC_wallet_import_v0_26_0.md` §4.1 (post-Phase-2-fold) — defers first-address verification to v0.27+.
  - `design/SPEC_wallet_import_v0_26_0.md` §2.4 row 3 (post-Phase-2-fold) — WARNING template struck through.
  - `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs::bsms_first_address_field_preserved_unverified` (post-fold rename) — pins audit-field preservation; no derivation-side assertions.
- **What:** v0.27+: derive descriptor at `<DERIVATION_PATH>/0/0` via existing toolkit derivation helpers, compute the address per the network detected in §4.2 step 8, and compare against `audit.first_address`. On mismatch, emit stderr WARNING per (restored) SPEC §2.4 row 3 template: `warning: import-wallet: bsms: first-address mismatch at path <P>: computed <C>, blob declares <D>` — informational only (exit 0, not hard-error). The check is BIP-129 §6's intended coordinator-output-self-consistency guard.
- **Why deferred:** Phase 2 surfaced that descriptor → address rendering at a specific derivation path is non-trivial in the v0.26.0 toolkit surface (no `derive_address_at_path` helper exists). The WARNING was informational-only (not hard-error), so deferring doesn't weaken the import-path correctness contract — concrete-keys checksum (BIP-380), xpub parse (`MsDescriptor::from_str`), and watch-only invariant remain load-bearing.
- **Status:** resolved — v0.27.0 Phase 3. **Closure narrative:** v0.27.0 lands `crate::derive_address::derive_first_address` (`crates/mnemonic-toolkit/src/derive_address.rs`) as a shared helper consumed by BOTH the new BSMS Round-2 emitter (line-4 first-address emission) AND the import-side parser's 6-line WARNING wire-up. The import-side wire-up at `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:198-244` parses the descriptor body via `miniscript::Descriptor::<DescriptorPublicKey>::from_str`, derives the receive-branch /0/0 address via `into_single_descriptors() → derive_at_index(0) → address(network)`, and compares against `audit.first_address`. Mismatch emits the SPEC §2.4 row 3 byte-exact template `warning: import-wallet: bsms: first-address mismatch at path <P>: computed <C>, blob declares <D>` (exit 0, informational). Taproot descriptors are explicitly skipped (BIP-386 not in BIP-129 §1 prerequisites). SPEC §2.4 row 3 un-struck. Integration cell renamed to `bsms_first_address_mismatch_warning` (was `bsms_first_address_field_preserved_unverified` during v0.26.0 deferral); `bsms_6_line_happy_path` updated to compute the real /0/0 address at test build time so it does NOT emit the WARNING and pins the byte-exact-match invariant.
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-signet-regtest-disambiguation` — coin-type-1 collapses signet/regtest to testnet

- **Surfaced:** 2026-05-18, Phase 0 R0 architect review I2 fold (during §7.0.a SPEC amendment) + cited in `wallet_import/bsms.rs:14-15` of Phase 2.
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:24-26` — module-level doc comment citing the FOLLOWUP for the canonical testnet collapse rule. (Cite refreshed against SHA `9b94a7d` 2026-05-22.)
  - `design/SPEC_wallet_import_v0_26_0.md` §4.2 step 8 — explicit normative text: "Signet and regtest are not distinguishable from testnet via origin-path inspection... imported as testnet."
  - Future `wallet_import/bitcoin_core.rs` (Phase 3) — same coin-type extraction will share this behavior.
- **What:** BIP-129 BSMS + Bitcoin Core `listdescriptors` origin annotations use coin-type `1` for testnet, signet, AND regtest — the blob is intrinsically ambiguous. v0.26.0 picks `Network::Testnet` as the canonical interpretation. v0.27+ may add either (a) a `--network signet|regtest` override on `import-wallet` (post-parse network re-binding), or (b) a separate origin-path-side disambiguator (e.g., a sibling `network_hint:` annotation that some wallets emit). User-direction needed before implementation.
- **Why deferred:** Surface-area trade-off: adding `--network` to `import-wallet` introduces a flag that 99% of users will never set (BIP-129 blobs don't carry signet/regtest as a separate type today), but the ambiguity exists and warrants explicit handling for users who run signet/regtest workflows. Testnet collapse is a safe v0.26.0 default.
- **Status:** resolved — v0.34.6. Added `import-wallet --network <mainnet|testnet|signet|regtest>` (option (a), the primary suggestion). Re-binds `ParsedImport.network` post-parse, guarded to the parsed coin-type class (testnet↔{testnet,signet,regtest}; mainnet↔mainnet); cross-class → `ImportWalletNetworkClassMismatch` (exit 1) since the blob's xpub prefix is coin-type-bound. New `CliNetwork::to_bitcoin_network` helper. 6 cells in `tests/cli_import_wallet_network_override.rs`. Paired GUI schema-mirror (`--network` Dropdown(NETWORKS) on import-wallet) + manual. Closed via cycle-prep recon (SHA `d330240`).
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-bsms-checksum-delegation-note` — SPEC §4.4 wording inaccuracy (checksum NOT auto-delegated)

- **Surfaced:** 2026-05-18, Phase 2 R0 architect review of v0.26.0 wallet-import cycle (implementer's finding 2).
- **Where:**
  - `design/SPEC_wallet_import_v0_26_0.md` §4.4 — text reads "BIP-380 8-character polymod checksum. Auto-validated when `MsDescriptor::from_str` is called by `parse_descriptor`."
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:26-27,140-145` — implementation calls `miniscript::descriptor::checksum::verify_checksum` explicitly up-front because `parse_descriptor::substitute_synthetic` (`parse_descriptor.rs:776`) swaps `@N` placeholders for synthetic xpubs BEFORE `MsDescriptor::from_str`, so the concrete-keys checksum never reaches `from_str`.
- **What:** Phase 7 SPEC-amend: rewrite §4.4 to describe the actual mechanism — "BIP-380 8-character polymod checksum. Validated UP-FRONT by `wallet_import::bsms::parse` via `miniscript::descriptor::checksum::verify_checksum` on the concrete-keys descriptor body, BEFORE the `concrete_keys_to_placeholders` adapter rewrites the body to `@N` placeholder form for `parse_descriptor`. The downstream `MsDescriptor::from_str` inside `parse_descriptor` operates on the synthetic-xpub-substituted form and cannot reach the original checksum." The implementation is correct; only the SPEC wording is wrong.
- **Why deferred:** Documentation-only fix; can ride the Phase 7 cycle-close SPEC-amend commit. No correctness change needed in code.
- **Status:** resolved (Phase 7 cycle-close commit; SPEC §4.4 amended with the up-front-validation prose; implementation unchanged at `wallet_import/bsms.rs:26-27,140-145`).
- **Tier:** `v0.26.0-cycle-close`
- **Companion:** none.

### `bsms-verify-signatures` — full BIP-129 HMAC token + signature verification on Round-2 ingest

- **Surfaced:** 2026-05-18, planned in `design/BRAINSTORM_wallet_import_v0_26_0.md` §6 item 2 as a cycle-close FOLLOWUP; cited in `crates/mnemonic-toolkit/src/wallet_import/mod.rs:65` of Phase 2.
- **Where:**
  - `design/BRAINSTORM_wallet_import_v0_26_0.md:264` — planned FOLLOWUP enumeration.
  - `design/SPEC_wallet_import_v0_26_0.md:150` — defers verification: "not verified in v0.26.0 (FOLLOWUP `bsms-verify-signatures`)".
  - `crates/mnemonic-toolkit/src/wallet_import/mod.rs:65` (approx; cite is in BsmsAuditFields doc-comment) — `signature_verified: bool` field locked to `false` in v0.26.0.
- **What:** v0.27+: implement full BIP-129 §5 HMAC token + signature verification flow. Coordinator's HMAC key derivation: PBKDF2(passphrase, salt, iterations) → HMAC-SHA256 over canonical Round-2 body. Toolkit-side flow: prompt for HMAC key (via `--coordinator-hmac-key <FILE>` or `@env:`), recompute HMAC, compare against `<SIGNATURE>` field; on success set `signature_verified: true`. Mismatch → exit 2 `ImportWalletParse` with stderr "bsms: signature mismatch — coordinator HMAC key does not match blob".
- **Why deferred:** Scope. v0.26.0 cycle target is parse + watch-only invariant + round-trip discipline; BIP-129 HMAC verification is a distinct security primitive that needs its own design pass (key-distribution UX, env-var/file/stdin input forms, refusal-on-missing-key default, etc.).
- **Status:** resolved — v0.27.0 Phase 2. **Closure narrative:** the FOLLOWUP body wording above (and the v0.26.0 SPEC framing it reflects) misreads BIP-129. Per the v0.27.0 Phase 2 BIP-129 recon at `design/agent-reports/v0_27_0-phase-2-bip129-recon.md`: BIP-129's signature surface is NOT HMAC; it is BIP-322 legacy-format ECDSA recoverable signatures on **Round-1** (Signer → Coordinator) records, not Round-2. v0.27.0 closes this FOLLOWUP by implementing BIP-129-faithful Round-1 verify (NEW input path `--bsms-round1 <FILE>`), NOT the HMAC-keyed Round-2 verify the FOLLOWUP body initially called for. Implementation: `crates/mnemonic-toolkit/src/wallet_import/bsms_round1.rs` (5-line parser, raw-pubkey + xpub KeyField dispatch) + `crates/mnemonic-toolkit/src/wallet_import/bsms_verify.rs` (BIP-322 ECDSA recoverable verify via `bitcoin::sign_message::signed_msg_hash` + `MessageSignature::recover_pubkey`). 6 unit cells in `bsms_verify::tests` against BIP-129 TVs 1/2/3 + 9 parser cells in `bsms_round1::tests` + 15 integration cells in `tests/cli_bsms_round1.rs`. Lenient default (stderr NOTICE) + `--bsms-verify-strict` mode (exit 2 BsmsSignatureMismatch). The HMAC primitives in BIP-129 are encryption-envelope MAC (PBKDF2-SHA512 + AES-256-CTR + HMAC-SHA256), separate from signature verify; that surface is filed at v0.27.0 cycle close as `bsms-bip129-full-cutover` for v0.28+.
- **Tier:** `v0.27`
- **Companion:** new sibling FOLLOWUP `bsms-bip129-full-cutover` (filed at v0.27.0 plan-revision time per Phase 2 recon pivot — see plan §8).

### `bsms-bip129-full-cutover` — complete BIP-129 conformance: 4-line Round-2 input parser + encryption envelope + deprecate v0.26.0 lenient parser

- **Surfaced:** 2026-05-18, v0.27.0 cycle Phase 2 BIP-129 recon (`design/agent-reports/v0_27_0-phase-2-bip129-recon.md`). v0.27.0's Path B-lite ships BIP-129 Round-1 verify (`--bsms-round1`) + BIP-129 Round-2 4-line emit (`--bsms-form 4-line`), but does NOT pivot the v0.26.0 6-line lenient input parser nor implement the encryption-envelope MAC surface.
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:146` — the v0.26.0 6-line lenient parser (`6 =>` arm of the line-count match) whose `signature` field has no agreed verify semantics under BIP-129. Citation refreshed against SHA `9b94a7d` 2026-05-22 (the `6 =>` arm of the line-count match).
  - `design/SPEC_wallet_import_v0_26_0.md:152` — the documented lenient-input framing that motivated the 6-line shape.
  - `design/agent-reports/v0_27_0-phase-2-bip129-recon.md` — full BIP-129 spec recon (verbatim quotes from §Specification → Round 1, Round 2, Encryption + 5 in-spec test vectors).
- **What:** sub-items (a)–(e) below; only (d) remains open:
  - (a) **Deprecate v0.26.0 6-line lenient parser.** Stderr DEPRECATION notice when 6-line input is detected; planned removal in a future minor version. *Shipped v0.28.0 (commit `1444c51`)*: the BIP-129-canonical 4-line Round-2 parser is now the default ingest path; 6-line lenient input still parses but fires a stderr DEPRECATION NOTICE. Final removal (d) deferred to a future minor.
  - (b) **Add BIP-129-faithful 4-line Round-2 input parser.** `BSMS 1.0` / `<descriptor>#<checksum>` / `<path-restrictions>` / `<first-address>`. Cross-validates the descriptor against the supplied path-restrictions + first-address (BIP-129 §Round 2 verify gate). *Shipped v0.28.0 (commit `1444c51`)*: 4-line parser + path-restrictions cross-validation + first-address byte-exact verify per SPEC §10.
  - (c) **Add encryption-envelope (STANDARD/EXTENDED) support.** *Shipped v0.31.0 (Cycle 7)*: `import-wallet --bsms-encryption-token <FILE|->` — PBKDF2-SHA512 + AES-256-CTR decrypt + HMAC-SHA256 verify per BIP-129 §Encryption (repeatable per-Signer at v0.32.2). Resolved as the dedicated sibling `bsms-bip129-encryption-envelope`.
  - (d) Drop the v0.26.0 6-line shape (and possibly the 2-line lenient excerpt) after a stable-version deprecation window. *Remains open* — the v0.28.0 deprecation NOTICE in sub-item (a) starts the window; removal deferred to a future minor.
  - (e) Document the v0.26.0 → v0.27 → v0.28 BSMS history in `design/SPEC_wallet_import_v0_28+.md` + manual chapter at `docs/manual/src/40-cli-reference/41-mnemonic.md`. *Shipped v0.28.0 (commit `d18787f` via P13)*: SPEC_wallet_import_v0_28_0.md §10 + manual chapter updates land in lockstep with the v0.28.0 ship.
- **v0.28.0 sub-deliverable note:** sub-item (b) BIP-129-canonical 4-line Round-2 parser shipped in commit `1444c51`; sub-item (a) deprecation NOTICE for 6-line shape ships alongside; sub-item (e) SPEC + manual coverage lands at P13. Sub-items (c) encryption envelope and (d) drop legacy shapes remain open and are tracked under the canonical entry (this one) plus the dedicated sibling FOLLOWUP `bsms-bip129-encryption-envelope` for the encryption-envelope work specifically.
- **Why deferred from v0.27.0:** Scope. v0.27.0 Path B-lite focuses on BIP-129 Round-1 verify + Round-2 emit (the two clean primitives that close the round-trip cycle). Adding the encryption-envelope primitives in v0.27.0 would ~double the cycle scope; deprecating v0.26.0's lenient parser pre-needs a stable BIP-129-faithful replacement input path (which requires the 4-line parser of (b) here). v0.28+ cycle.
- **Status:** open — ONLY sub-item (d) remains: final removal of the deprecated 6-line lenient parser arm (`wallet_import/bsms.rs:146`) + `ImportProvenance::BsmsSixLine`. (a)/(b)/(e) shipped v0.28.0; (c) shipped v0.31.0 (sibling `bsms-bip129-encryption-envelope`). (d) is a behavior change (the 6-line path still parses-with-deprecation-notice today) → future SemVer **MINOR**, not bundled into v0.34.3 hygiene. Sub-item scope corrected 2026-05-22 via wallet-cluster cycle-prep recon (SHA `9b94a7d`).
- **Tier:** `v0.27-cycle-close`
- **Tags:** `wallet`
- **Companion:** sibling of `bsms-verify-signatures` (v0.27.0 closes the Round-1 SIG subset of the original FOLLOWUP body's intent; this entry covers what stays open after that closure). Sibling carve-out: `bsms-bip129-encryption-envelope` (v0.28+; sub-item (c) tracked separately).

### `wallet-export-bsms-emitter` — `mnemonic export-wallet --format bsms` is unimplemented; blocks BSMS bundle round-trip cells

- **Surfaced:** 2026-05-18, Phase 4 implementer (commit `120e6b4`) noted at the "Phase 5 deferrals" section of the commit body; corroborated at Phase 4 R0 architect review.
- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` — current emitter enumeration lacks `bsms`; the toolkit has only Bitcoin Core JSON + descriptor-passthrough surfaces at v0.25.x.
  - `design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md` §4.5 (lines 308-318) — bundle round-trip BSMS direction is **structurally blocked** without the emitter.
  - `design/SPEC_wallet_import_v0_26_0.md` §7.1 — round-trip discipline assumes both formats can emit; BSMS direction was acknowledged blocked at Phase 4 R0.
- **What:** Implement the BSMS Round-2 export-side emitter so that `mnemonic export-wallet --format bsms` produces a BIP-129 lenient 2-line (or 6-line, with `--coordinator-hmac-key` for the optional MAC) output. Pairs with v0.26.0's import-side surface to close the bundle round-trip discipline cells deferred in §4.5. The 6-line shape additionally depends on `bsms-verify-signatures` (HMAC key material plumbing).
- **Why deferred:** Outside v0.26.0 scope; the cycle goal was import-side correctness + round-trip discipline. The 2-line emitter is feasible standalone (no HMAC required); 6-line emitter pairs with `bsms-verify-signatures`. Splitting into two FOLLOWUPs (2-line in v0.27.x, 6-line bundled with `bsms-verify-signatures`) is a viable plan.
- **Status:** resolved — v0.27.0 Phase 3. **Closure narrative:** the v0.27.0 plan-doc R6 pivot (post-Phase 2 BIP-129 recon) reframed the emitter scope to: BIP-129-canonical 4-line Round-2 plaintext (default) + 2-line lenient excerpt (symmetric with the v0.26.0 import-side parser). The 6-line emit shape was dropped — that shape commingled BIP-129 Round-2 plaintext lines with a (misframed) "envelope-side HMAC/signature" that BIP-129 §Specification → Round 2 does NOT carry (BIP-322 signatures are on Round-1 Signer→Coordinator records, not Round-2; see `design/agent-reports/v0_27_0-phase-2-bip129-recon.md`). Implementation: `crates/mnemonic-toolkit/src/wallet_export/bsms.rs` (`BsmsEmitter` + `BsmsForm` enum, ~180 LOC) + `crates/mnemonic-toolkit/src/derive_address.rs` (shared first-address helper). CLI surface: `--format bsms` + `--bsms-form 2-line|4-line` (default `4-line`). Taproot descriptors refuse with `BadInput` (BIP-386 not in BIP-129 §1 prerequisites). Path-restrictions emit rule per SPEC §3.5.1: structural per-key walk via `Descriptor::for_each_key` → `/0/*,/1/*` for all-canonical-multipath, `/0/*` for all-single-receive, `No path restrictions` for any divergent shape. 8 integration cells in `tests/cli_export_wallet_bsms.rs` covering 2-of-2 / 2-of-3 / 3-of-4 / path-restrictions / first-address byte-exact / tr-refuse / 2-line / 2-line→import round-trip. 6-line `--coordinator-hmac-key` continuation rolls into `bsms-bip129-full-cutover` (v0.28+) which subsumes the HMAC-envelope work that the FOLLOWUP body originally hinted at.
- **Tier:** `v0.27`
- **Companion:** none. (Pairs in spirit with `bsms-verify-signatures` for the 6-line shape.)

### `wallet-import-json-envelope-full-bundle` — `--json` envelope `bundle:` field is a parse-side summary, not the full toolkit-native `BundleJson`

- **Surfaced:** 2026-05-18, Phase 5 R0 architect review I2 (commit `ff1c85c` under review).
- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:336-359` — `emit_json_envelope` hand-builds a summary `bundle_view: { cosigners: [...], network, threshold }`.
  - `crates/mnemonic-toolkit/src/wallet_import/mod.rs:59` — `ParsedImport.descriptor: md_codec::Descriptor` is parsed but never read by `cmd::import_wallet::run` (carries a narrow `#[allow(dead_code)]` per Phase 5 fold pointing at this FOLLOWUP).
  - `design/SPEC_wallet_import_v0_26_0.md` §2.2 — post-Phase-5-fold lock: v0.26.0 ships the summary shape; full BundleJson tracked here.
- **What:** v0.27+: wire the `--json` envelope's `bundle:` field to emit the full toolkit-native `BundleJson` shape (the same `verify-bundle --bundle-json` consumes — with synthesized ms1/mk1/md1 cards). This requires invoking the synthesizer post-parse against the supplied / overlayed seeds; for watch-only cosigners, emit the ms1/mk1 sentinel forms per SPEC §5.8. The `descriptor: md_codec::Descriptor` field on `ParsedImport` becomes load-bearing in this wire-up (currently unused).
- **Why deferred:** v0.26.0's scope was parse + watch-only invariant + round-trip discipline; envelope-side synthesis is a distinct integration with `crate::synthesize` that exceeds the cycle budget. The summary shape is forward-compatible with v0.27: the envelope key remains `bundle`, the shape itself extends. Downstream consumers encoding against v0.26.0 should target the summary; v0.27 will treat the legacy summary as a strict subset of the full shape.
- **Status:** resolved (v0.27.0 Phase 4 — `emit_json_envelope` rewritten to synthesize full `BundleJson` via `synthesize_descriptor`; new `ParsedImport.original_descriptor: String` field carries the pre-strip raw descriptor for envelope wire emission; outer envelope gains `schema_version: "1"`; wire-shape replacement (NOT additive) per CHANGELOG `### Changed`; closes via per-phase R0 GREEN 0C/0I + 8 new test cells incl verify-bundle round-trip Cell 7 + envelope_v0_27_0.json byte-exact fixture pin).
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-fixture-corpus-expansion` — broaden BSMS + Bitcoin Core fixture coverage to SPEC §10.1/§10.2 full set

- **Surfaced:** 2026-05-18, Phase 4 R0 architect review I1 (commit `120e6b4` under review).
- **Where:**
  - `crates/mnemonic-toolkit/tests/fixtures/wallet_import/` — v0.26.0 ships 8 BSMS + 5 Bitcoin Core fixtures (post-Phase-4-fold close M3 adds `bsms-2line-multi-2of3.txt` for declaration-order discipline).
  - `design/SPEC_wallet_import_v0_26_0.md` §10.1 / §10.2 — full corpus enumerated (12-15 per format); §10.1 amended post-Phase-4-fold to lock the v0.26.0 shipped subset + cite this FOLLOWUP.
  - `design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md` §4.3 / §4.4 — full fixture list.
- **What:** v0.27+: ship remaining fixtures per SPEC §10.1/§10.2 — BSMS decay-4032, 6-line sortedmulti-2of3, sortedmulti-3of5, mainnet+ypub, mainnet+zpub, tr(NUMS,...) taproot; Core BIP-44 P2PKH, BIP-86 P2TR, wsh-sortedmulti 3-of-5, native `<0;1>/*` multipath, explicit `active: false` cell name. Each new fixture pairs with a per-fixture canonicalize-idempotency cell + a sniff cell.
- **Why deferred:** v0.26.0's shipped subset exercises the load-bearing canonicalize + idempotency + Core envelope-shape branches + declaration-order preservation invariant. Missing fixtures are coverage-expansion targets, not load-bearing for v0.26.0's correctness contract. Phase 4 implementer's commit body explicitly flagged the corpus reduction; the architect review confirmed the load-bearing paths are covered and the missing fixtures are expansion-class rather than correctness-class.
- **Status:** resolved (v0.28.0; d7a2859 (G3 BSMS fixtures) + 2a803e8 (H Core fixtures)). Phase G3 shipped 7 new BSMS fixtures (2-line/6-line sortedmulti variants, mainnet+ypub, mainnet+zpub, tr(NUMS,...) parity cell) + 9 cells. Phase H (Wave 2) shipped 8 new Bitcoin Core fixtures (BIP-44 P2PKH, BIP-86 P2TR, wsh-sortedmulti 3-of-5, native `<0;1>/*` multipath, explicit `active: false`, etc.).
- **Tier:** `v0.27`
- **Companion:** none.

### `gui-import-wallet-env-var-secret-channel` — v0.12.0+ GUI: auto-rewrite literal seeds in repeating `--ms1` widgets to `@env:MNEMONIC_MS1_<i>` sentinels + spawn-time env-var injection

- **Surfaced:** 2026-05-18, Phase 6 R0 architect review C1 (toolkit-manual cycle).
- **Where:**
  - `mnemonic-gui/src/runner.rs:74-114` — current spawn flow injects only `MNEMONIC_FORCE_TTY`; no per-cosigner secret env-var bag.
  - `mnemonic-gui/src/form/invocation.rs:236-251` — repeating-secret branch routes values verbatim.
  - `mnemonic-gui/src/main.rs:683-688` — run-confirm modal renders argv verbatim (per `[[feedback-run-confirm-modal-renders-argv-verbatim]]`).
  - `mnemonic-gui/tests/kittest_import_wallet_form.rs:154-213` — pins literal-pass-through contract.
  - `design/SPEC_wallet_import_v0_26_0.md` §9.3 — describes the aspirational behavior (toolkit-side accepts `@env:VAR` sentinels at parse time, but GUI does NOT pre-rewrite).
- **What:** v0.12.0+ `mnemonic-gui`: on subprocess spawn, collect per-cosigner-index secret values from `--ms1` repeating-widget state into a per-spawn env-var bag (`MNEMONIC_MS1_<i>=<value>`), rewrite `args[--ms1+1]` to `@env:MNEMONIC_MS1_<i>` sentinels, render the sentinel-bearing argv in the run-confirm modal, drop the env-vars on subprocess exit. Same pattern for `--passphrase`, `--share` (slip39-combine, seed-xor-combine), and other secret-bearing flags. Toolkit-side already accepts the sentinel at parse-time per Phase 1 cross-cutting helper.
- **Why deferred:** v0.26.0 scope is wallet-import-side parse + watch-only invariant + round-trip; the env-var-channel rewrite is GUI-side runner work that affects ALL repeating-secret surfaces, not just `--ms1`. Pre-existing `gui-run-confirm-modal-secret-redaction` (mnemonic-gui/FOLLOWUPS.md:454-462) covers the modal-redaction direction; this FOLLOWUP covers the argv-rewrite direction. Both need to land together in v0.12.0.
- **Status:** open
- **Tier:** `v0.12.0`
- **Companion:** `mnemonic-gui/FOLLOWUPS.md::gui-import-wallet-env-var-secret-channel` (primary). v0.26.0 manual prose calls out the user-must-type-explicitly fallback.

### `gui-import-wallet-cell-coverage-gap` — Phase 6 plan §6.4.a + §6.4.b + §6.8 + §6.9 cells deferred to v0.12.0+

- **Surfaced:** 2026-05-18, Phase 6 R0 architect review I2.
- **Where:**
  - `design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md` §6.4.a-§6.9 — plan items not shipped at `b2e281a`.
  - `mnemonic-gui/tests/kittest_import_wallet_form.rs` — 8 cells shipped; 4 plan items not exercised (file-picker extension filter; blob-paste-textarea → stdin; env-var unset → subprocess exit 1; env-var no parent leak).
- **What:** v0.12.0+: ship the 4 deferred kittest cells. §6.8 + §6.9 ride `gui-import-wallet-env-var-secret-channel` close (no point asserting env-var lifecycle until the GUI does env-var injection). §6.4.a + §6.4.b are independent and can ship earlier.
- **Why deferred:** §6.4.a + §6.4.b ride file-picker / textarea widget plumbing not yet enumerated in the v0.11.0 GUI's file-selection surface (currently a one-shot file path string; no extension filter; no textarea paste). §6.8 + §6.9 ride the env-var-channel FOLLOWUP.
- **Status:** open
- **Tier:** `v0.12.0`
- **Companion:** none.

### `wallet-import-sparrow` — Sparrow Wallet JSON export-format ingest

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/` — no Sparrow parser; sniff would need to extend `SniffOutcome` + new module.
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: add `wallet_import/sparrow.rs` (or merged dispatcher) parsing Sparrow's wallet-export JSON shape. Inverse of `wallet_export::sparrow::emit_sparrow_wallet_json` (at `wallet_export/sparrow.rs:103`); the wallet_export-side Sparrow emitter ships today, so this is the matching ingest side. Citation verified against origin/master SHA `1abd9d1` 2026-05-19.
- **Why deferred:** v0.26.0 scope was BSMS Round-2 + Bitcoin Core listdescriptors only. Sparrow/Specter/Coldcard/Jade are tracked individually for granular v0.27+ scope planning.
- **Status:** resolved (v0.28.0; b20a357). Phase P1 shipped `wallet_import/sparrow.rs` covering singlesig + `sortedmulti` wsh() / sh(wsh()) shapes. Taproot descriptor-passthrough support filed forward as `sparrow-taproot-descriptor-passthrough-import-support` (v0.29+).
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-specter` — Specter-DIY JSON descriptor export

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/` — no Specter parser; sniff would need to extend `SniffOutcome` + new module.
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: add Specter-DIY JSON descriptor parser (non-BSMS path). Specter's wallet-export schema diverges from BSMS Round-2's line-oriented shape and from Bitcoin Core's `listdescriptors` envelope; needs its own sniff signature + parser.
- **Why deferred:** v0.26.0 scope was BSMS Round-2 + Bitcoin Core listdescriptors only. Sparrow/Specter/Coldcard/Jade are tracked individually for granular v0.27+ scope planning.
- **Status:** resolved (v0.28.0; 8548258). Phase P2 shipped `wallet_import/specter.rs` covering Specter-DIY JSON descriptor exports.
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-electrum` — Electrum 4.x wallet file ingest

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/` — no Electrum parser; sniff would need to extend `SniffOutcome` + new module.
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: ingest Electrum 4.x wallet file (Python-dict-serialized JSON with `xpub` / `wallet_type` keys; multisig shapes via `x1`/`x2`/... per-cosigner subkeys). Encrypted variants (Electrum's stretched-key envelope) are out of scope; sibling FOLLOWUP for encrypted ingest if user-direction warrants.
- **Why deferred:** v0.26.0 scope was BSMS Round-2 + Bitcoin Core listdescriptors only. Sparrow/Specter/Coldcard/Jade are tracked individually for granular v0.27+ scope planning.
- **Status:** resolved (v0.28.0; 2031609). Phase P6 shipped `wallet_import/electrum.rs` covering Electrum 4.x wallet-file ingest (singlesig + multisig `x1`/`x2`/... per-cosigner subkeys). Encrypted-envelope variant filed forward as `wallet-import-electrum-encrypted` (v0.28+) per Q2 lock.
- **Tier:** `v0.27`
- **Companion:** `wallet-import-electrum-encrypted` (encrypted Electrum wallets require decrypting via Electrum CLI first).

### `wallet-import-coldcard` — Coldcard wallet.json export (single-sig)

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/` — no Coldcard parser; sniff would need to extend `SniffOutcome` + new module.
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: ingest Coldcard's single-sig `wallet.json` export shape (BIP-44 / BIP-49 / BIP-84 / BIP-86 per-path xpub blocks under a fixed envelope; Coldcard-specific provenance metadata). Multisig descriptor-text shape is tracked separately under `wallet-import-coldcard-multisig`.
- **Why deferred:** v0.26.0 scope was BSMS Round-2 + Bitcoin Core listdescriptors only. Sparrow/Specter/Coldcard/Jade are tracked individually for granular v0.27+ scope planning.
- **Status:** resolved (v0.28.0; 1304932). Phase P3 shipped `wallet_import/coldcard.rs` covering Coldcard single-sig wallet.json (BIP-44 / BIP-49 / BIP-84 / BIP-86 per-path xpub blocks). Legacy mk1/mk2 top-level xpub inference filed forward as `coldcard-legacy-mk1-mk2-top-level-xpub-inference` (v0.29+).
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-coldcard-multisig` — Coldcard multisig.txt (descriptor + cosigner list)

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/` — no Coldcard-multisig parser; sniff would need to extend `SniffOutcome` + new module.
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: ingest Coldcard's multisig descriptor-text export (`Name`, `Policy`, `Format`, per-cosigner `Derivation` + xpub blocks; output script type as a separate header). Distinct shape from Coldcard's single-sig `wallet.json`; line-oriented `Key: Value` grammar more similar to BSMS than to Sparrow's JSON.
- **Why deferred:** v0.26.0 scope was BSMS Round-2 + Bitcoin Core listdescriptors only. Sparrow/Specter/Coldcard/Jade are tracked individually for granular v0.27+ scope planning.
- **Status:** resolved (v0.28.0; 387a709). Phase P4 (instance D) shipped `wallet_import/coldcard_multisig.rs` covering Coldcard multisig text-file ingest (`Name`/`Policy`/`Format`/per-cosigner `Derivation`+xpub blocks).
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-jade` — Jade SeedQR or descriptor JSON export

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/` — no Jade parser; sniff would need to extend `SniffOutcome` + new module.
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: ingest Blockstream Jade's `register_multisig` JSON-like export shape (multisig descriptor + per-cosigner xpub + name + threshold; signer-fingerprint annotations). SeedQR formats — distinct surface — may be folded later as an inline mode rather than a wallet-import format if user-direction warrants.
- **Why deferred:** v0.26.0 scope was BSMS Round-2 + Bitcoin Core listdescriptors only. Sparrow/Specter/Coldcard/Jade are tracked individually for granular v0.27+ scope planning.
- **Status:** resolved (v0.28.0; 091a313). Phase P5 (instance E) shipped `wallet_import/jade.rs` covering Blockstream Jade `get_registered_multisig` reply (multisig descriptor + per-cosigner xpub + threshold + signer-fingerprint annotations). SeedQR-format intake deferred per Q1 lock; filed forward as `wallet-import-jade-seedqr` (v0.28+).
- **Tier:** `v0.27`
- **Companion:** `wallet-import-jade-seedqr` (SeedQR intake surface).

### `wallet-import-bsms-round-1` — BSMS Round-1 share ingest (multi-cosigner setup phase)

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs` — current parser handles Round-2 only (concrete descriptor + audit envelope).
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: ingest BSMS Round-1 share files (per-cosigner contribution prior to coordinator assembly; carries token + signer-fingerprint + xpub but NOT the assembled descriptor). Multi-share collation requires N-of-N Round-1 inputs to produce a single Round-2-equivalent bundle; semantically distinct from single-blob ingest.
- **Why deferred:** v0.26.0 scope was BSMS Round-2 only. Round-1 multi-share orchestration is a multi-input pipeline (vs Round-2's single-blob ingest); needs its own CLI surface (e.g., `--shares share1 share2 share3` repeating-flag) and threshold-consistency invariants.
- **Status:** resolved — superseded by v0.27.0 `import-wallet --bsms-round1 <FILE>` (repeating; BIP-129 Round-1 record BIP-322 verify; `--bsms-verify-strict`). The in-scope subset (Round-1 record ingest + verify) shipped; the body's remaining intent — coordinator-side *assembly* of a multisig descriptor from N Round-1 shares (the proposed `--shares` collation) — is OUT OF SCOPE for an import/verify/backup tool (same category as the deliberately-excluded signing/PSBT; opus architect disposition 2026-05-22: DISPOSITION A). Users coordinate in Sparrow/Specter/Coldcard, then `import-wallet` the resulting Round-2 blob (supported, plaintext or encrypted). If a concrete user wants coordinator mode, file a fresh, deliberately-scoped slug with its own brainstorm/R0. Cross-ref `bsms-verify-signatures` (v0.27.0 Round-1 SIG closure) + sibling `bsms-encryption-round1-decrypt-then-verify`. Closed 2026-05-22 via wallet-cluster cycle-prep recon (SHA `9b94a7d`).
- **Tier:** `v0.27`
- **Companion:** none.

### `wallet-import-bsms-encrypted` — BSMS encrypted-envelope decryption + Round-2 ingest

- **Surfaced:** 2026-05-18, Phase 6 cycle close (forward-reference from manual chapter `docs/manual/src/45-foreign-formats.md`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs` — current parser handles unencrypted Round-2 only.
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — cites this slug.
- **What:** v0.27+: decrypt BSMS encrypted-envelope shape per BIP-129 §5 (AES-CTR over the Round-2 payload keyed by a coordinator-shared token-derived key), then route the decrypted plaintext through the existing Round-2 parser. Requires CLI flag for the decryption key material (e.g., `--bsms-key <hex>` or `@env:BSMS_KEY` sentinel) and clear stderr templates for decryption failure vs format failure.
- **Why deferred:** v0.26.0 scope was unencrypted BSMS Round-2 only. Encrypted-envelope decryption is a distinct cryptographic surface that warrants its own design discussion (key material handling, argv leak vectors, key-derivation choice). The user can decrypt out-of-band today and pipe plaintext into `import-wallet`.
- **Status:** resolved — shipped v0.31.0 (`import-wallet --bsms-encryption-token <FILE|->`: BIP-129 §Encryption envelope = PBKDF2-SHA512 + AES-256-CTR + HMAC-SHA256 verify-before-decrypt). The CLI flag the body speculated as `--bsms-key` shipped as `--bsms-encryption-token`; encrypted Round-1 records landed v0.32.1, per-Signer tokens v0.32.2. The "current parser handles unencrypted Round-2 only" framing above is superseded. Resolved alongside sibling `bsms-bip129-encryption-envelope` (Cycle 7). Closed 2026-05-22 via wallet-cluster cycle-prep recon (SHA `9b94a7d`).
- **Tier:** `v0.27`
- **Companion:** none.

### `coordinator-runbook-into-design-dir` — promote merge-plan-doc into `design/`

- **Surfaced:** 2026-05-18, v0.26.0/v0.11.0 cycle close — per architect R2 fold.
- **Where:**
  - Source: `.v0_26_0-merge-plan.md` (project-root scratch file, R3 — folds R0+R1+R2 architect reviews).
  - Destination: `design/PLAN_v0_26_0_three_way_merge.md` (canonical-record location per project convention; cf. `design/PLAN_v0_26_0_xpub_search.md`).
  - `CLAUDE.md` should cross-cite as the multi-instance coordination playbook.
- **What:** Copy `.v0_26_0-merge-plan.md` into `design/PLAN_v0_26_0_three_way_merge.md` (verbatim or with a "canonical record" header). Delete the scratch file at project root. Add a one-line bullet in `CLAUDE.md` Conventions section pointing at the design-dir copy as the recipe for future multi-instance cycles.
- **Why deferred:** Cycle-close polish; no correctness regression. The scratch artifact at root continues to function as the audit trail for the cycle itself; promotion is a future-reference cleanup.
- **Status:** resolved — v0.27.0 Phase 1 (file relocated with canonical-record header; `CLAUDE.md` Conventions cross-cite added; presence-smoke `tests/design_artifacts_presence.rs::three_way_merge_runbook_lives_in_design_dir` guards future churn).
- **Tier:** `v0.27`
- **Companion:** memory entry `[[project-v0-26-0-cycle-shipped]]`.

### `gui-schema-arm-drop-detector` — formalize `build_subcommand_conditional_rules` grep-c assertion as regression test

- **Surfaced:** 2026-05-18, v0.26.0 cycle close — per architect R0 finding I1.
- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/gui_schema.rs::build_subcommand_conditional_rules` — the match-block dispatcher whose arm count grew from a few to 11+ across v0.26.0 (4 xpub-search + 1 import-wallet + 1 compare-cost).
  - Proposed test location: `crates/mnemonic-toolkit/tests/cli_gui_schema_arm_count.rs` (or as a new cell in `cli_gui_schema_conditional_rules.rs`).
- **What:** Three-way merge of the dispatcher arm-set across concurrent feature PRs is silently-dropping-risky when two PRs insert at adjacent positions. Mitigation that worked this cycle: manual `grep -c '=> .*_conditional_rules()' crates/mnemonic-toolkit/src/cmd/gui_schema.rs` per rebase, with a documented expected count. Formalize as a `#[test]` that asserts the live count against a pinned constant; bumping the constant becomes the explicit signal whenever a new arm is added (and forces conscious decision-making in multi-PR rebases).
- **Why deferred:** Per-PR rebase verification worked this cycle; codification is hardening rather than gap-fix.
- **Status:** resolved (93bf3ff; v0.27.2 Phase 1.4)
- **Tier:** `v0.27`
- **Companion:** `[[project-v0-26-0-cycle-shipped]]`.

### `error-rs-canonical-ordering-doc` — record alphabetical variant-ordering rule in CONVENTIONS

- **Surfaced:** 2026-05-18, v0.26.0 cycle close — per architect R0 finding I2.
- **Where:**
  - `crates/mnemonic-toolkit/src/error.rs` — `ToolkitError` enum and its `match self` blocks (Display impl, `exit_code`, `kind`).
  - Proposed doc location: `CLAUDE.md` Conventions section, OR a new `design/CONVENTIONS.md`.
- **What:** Adopt alphabetical-by-variant-name as the canonical ordering rule for:
  - the `enum ToolkitError` variant declarations, and
  - each `match self { ... }` block that exhaustively matches it (Display, exit_code, kind).
  Drift across concurrent feature PRs (9+ new variants in v0.26.0 cycle) is otherwise a guaranteed merge-conflict generator; the rule makes the resolution mechanical. Codify this in CONVENTIONS so future cycles converge on the same order without per-PR negotiation.
- **Why deferred:** v0.26.0 cycle resolved this in-flight via the plan-doc cheat-sheet (P-I2 fold); codification is for future cycles.
- **Status:** resolved (79734f8; v0.27.2 Phase 1.1)
- **Tier:** `v0.27`
- **Companion:** `[[project-v0-26-0-cycle-shipped]]`.

### `compare-cost-agent-reports-back-fill` — persist architect reviews verbatim, in real time

- **Surfaced:** 2026-05-18, v0.26.0 compare-cost cycle close — per multi-aspect code review finding (CLAUDE.md line 30 compliance gap).
- **Where:**
  - `crates/mnemonic-toolkit/CLAUDE.md` line 30 ("Per-phase opus reviews persist to `design/agent-reports/`") — load-bearing convention; the compare-cost cycle violated it.
  - `design/agent-reports/compare-cost-cycle-meta.md` — meta-record back-filling the audit trail with commit pointers, but verbatim review text was lost.
- **What:** Establish per-cycle discipline: when an architect-review agent dispatch completes, **write its verbatim output** to `design/agent-reports/phase-N-r0-review.md` (or similar) BEFORE the per-phase fold-and-commit step. The compare-cost cycle's reviews were inlined in the session transcript only; a back-fill meta-record exists but verbatim text is unrecoverable from outside the transcript. Future cycles MUST persist verbatim — recommend wiring into a per-phase task with an explicit "write report file" step. Optionally extend the plan-doc template at `.v0_26_0-merge-plan.md` to enumerate this discipline.
- **Why deferred:** Convention codification; no per-PR regression. Future cycles will benefit from real-time persistence.
- **Status:** resolved (08cf0a9; v0.27.2 Phase 1.2)
- **Tier:** `v0.27`
- **Companion:** none.

### `gui-workflow-trigger-include-release-branches` — CI gates silently skip PRs targeting release branches

- **Surfaced:** 2026-05-19, v0.11.0 GUI cycle — discovered mid-G2/G3 when no CI workflows queued for 14+ min after force-pushes on `compare-cost/p4-gui` and `worktree-xpub-search-v0-11-0`.
- **Where:**
  - Cross-repo: `mnemonic-gui/.github/workflows/build.yml` and `mnemonic-gui/.github/workflows/schema-mirror.yml`
  - Trigger blocks: `pull_request: branches: [master]`
- **What:** Both workflow files currently filter `pull_request: branches: [master]` — meaning **no CI fires for PRs targeting `release/v0.11.0`** (or any future integration branch). v0.11.0 cycle worked around this via local pre-merge vetting (`cargo build` + `cargo clippy --all-targets -- -D warnings` + `cargo test` with `MNEMONIC_BIN` pointing at the v0.26.0 toolkit binary) plus `--admin` merges against the integration branch. The integration PR (`release/v0.11.0 → master`) DID trigger workflows normally (base=master), so the load-bearing gate worked. Fix: extend trigger filter to `branches: [master, release/*]` so per-PR CI runs on integration branches too. Reduces reliance on out-of-band local vetting.
- **Why deferred:** Cycle workaround was sound and architecturally consistent (per plan-doc §G3.5.2, the integration PR is the load-bearing gate). Trigger-filter fix is a future-cycle ergonomics improvement.
- **Status:** resolved (sibling tag mnemonic-gui-v0.11.1; v0.27.2 Phase 3)
- **Tier:** `v0.27` (cross-repo companion in mnemonic-gui).
- **Companion:** `mnemonic-gui/FOLLOWUPS.md::gui-workflow-trigger-include-release-branches` (this cycle close).

### `cross-format-conversion-matrix-expansion` — N×M coverage beyond the BSMS → Bitcoin Core integration cell

- **Surfaced:** 2026-05-19, v0.27.0 Phase 6 cycle close.
- **Where:**
  - `crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs::cross_format_bsms_to_bitcoin_core_to_import_round_trip` — the v0.27.0 headline integration cell. Single source→destination pair (BSMS → Bitcoin Core).
  - `docs/manual/src/30-workflows/39-cross-format-conversion.md` — narrative recipe covering 3 paths (BSMS→Bitcoin Core, Bitcoin Core→bundle synth, BSMS→BIP-388); no automated cross-format coverage for the remaining N×M combinations.
- **What:** v0.28+: expand the integration cell into a parameterized N×M matrix covering every supported source × destination pair: BSMS / Bitcoin Core as sources × {bitcoin-core, bip388, bsms, sparrow*, jade*, coldcard*, electrum*, specter*, green} as destinations (* requires template-mode handling — wallet-export-from-import-json refuses descriptor-mode by current per-emitter contract). Acceptance: every cell loads a fixture envelope, emits the target format, parses the output (or asserts the expected refusal for template-only formats), and asserts cosigner xpub + descriptor preservation.
- **Why deferred:** v0.27.0 cycle scope was wiring + headline integration; matrix expansion is hardening work, not load-bearing for v0.27.0 correctness.
- **Status:** resolved (v0.28.0; 8bf78ff). Phase P11 shipped a parameterized N×M cross-format matrix in `tests/cli_export_wallet_from_import_json.rs`: 24 happy-path + 42 refusal cells (74 total) covering 8 sources (bsms, bitcoin-core, coldcard, coldcard-multisig, electrum, jade, sparrow, specter) × N destinations including refusal classes per per-emitter contract. Symmetry/inverse-mismatch matrix completion filed forward as `wallet-import-format-mismatch-matrix-completion` (v0.28+).
- **Tier:** `v0.28+`.
- **Companion:** `wallet-import-format-mismatch-matrix-completion` (per-arm symmetric mismatch wiring).

### `bsms-taproot-emit` — BIP-129 emit for tr() descriptors (BIP-386 prerequisite)

- **Surfaced:** 2026-05-19, v0.27.0 Phase 3 deferral (revised from `bsms-taproot-6-line`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_export/bsms.rs:64-79` — `BsmsEmitter::emit()` refuses taproot scripts (P2tr / P2trMulti) with `ToolkitError::BadInput`. Note: `BsmsEmitter::collect_missing` at `bsms.rs:58-62` returns an empty vec; the `IncompatibleFormatForTemplate` variant defined in `wallet_export/mod.rs` is not on the BSMS refusal path. Citation refreshed against origin/master SHA `9b94a7d` 2026-05-22.
  - BIP-129 §1 Prerequisites — enumerates BIPs 32/43/44/45/48/67/86/87/174/350 + the SegWit BIPs; conspicuously does NOT include BIP-386 (`tr(...)`).
- **What:** v0.28+: implement BSMS Round-2 emit for taproot descriptors (`tr(K)` and `tr(internal, multi_a(K,...))` / `tr(internal, sortedmulti_a(K,...))`). Blocked on a BIP-129 update adding BIP-386 to §1 prerequisites — without that prerequisite update, the BIP-129-canonical descriptor-line shape is undefined for tr() (specifically: how taproot internal-key + multi_a leaf set serialize into the line-2 descriptor body within the BIP-129 spec). Soft-blocker: track upstream BIP-129 PRs; revisit when a published canonicalization is available.
- **v0.28.0 sub-deliverable note:** P8A+P8B (commit `158897f`) shipped a refusal-scaffold UX improvement: `--format bsms` taproot refusal text is now per-script-type discriminated (P2tr / P2trMulti); refusal text cites this FOLLOWUP slug and points users at `--format bitcoin-core` / `--format sparrow` alternatives. Real emit remains upstream-blocked on BIP-129 §1 prerequisites adding BIP-386; the v0.28.0 work is refusal-side hardening only.
- **Why deferred:** Standards prerequisite not yet met; emitting against an unpublished canonicalization would commit the toolkit to a wire shape we'd need to break.
- **Status:** open (real emit remains upstream-blocked; v0.28.0 shipped refusal-scaffold UX improvements only).
- **Tier:** `v0.28+`.
- **Tags:** `wallet`
- **Companion:** `bsms-import-taproot-refusal-parity` (v0.28+; symmetric import-side refusal hardening + `extract_threshold` side-channel finding).

### `wallet-import-taproot-internal-key` — `tr(sortedmulti_a(...))` envelope consumers silently lose internal-key designation

- **Surfaced:** 2026-05-19, v0.27.0 Phase 5 R0 deferral.
- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/export_wallet.rs::run_from_import_json` — constructs `EmitInputs` with `taproot_internal_key: None` (envelope doesn't carry the designation; v0.27.0 doesn't surface it).
  - `crates/mnemonic-toolkit/src/wallet_export/bsms.rs` — refuses taproot at emit time (per `bsms-taproot-emit` deferral).
  - `crates/mnemonic-toolkit/src/wallet_export/{coldcard,jade,sparrow,electrum,specter}.rs` — accept descriptor-mode but lose the multi_a-vs-key-path internal-key designation when emitting from `--from-import-json` (each consumer uses `taproot_internal_key: None`).
- **What:** v0.28+: when an envelope's `bundle.descriptor` matches `tr(...)`, EITHER (a) refuse loudly at the `--from-import-json` typed-deser layer with `BadInput "taproot envelopes not yet supported by --from-import-json; supply --template + --slot args directly"`, OR (b) thread the internal-key designation through the envelope's `bundle` surface (which currently has no such field). Option (a) is simpler and matches v0.27.0's "envelopes are descriptor-mode only" framing.
- **Why deferred:** v0.27.0 cycle deferred per Phase 5 R0 (no v0.27.0 wire-format path produces a taproot envelope from `import-wallet`: BSMS rejects taproot at import; Bitcoin Core `listdescriptors` with `tr()` is a corner case). Refuse-loudly is hardening for the v0.28+ corner case.
- **Status:** `resolved ffcd336e76ee1e2b74a5a0b918b6bc78bef275ea` — mnemonic-toolkit-v0.28.7 cycle Fix-α (Framing B envelope-gate-only per P0 recon). Refusal added at `cmd/export_wallet.rs:622` immediately after `script_type_from_descriptor` via `matches!(script_type, WalletScriptType::P2tr | WalletScriptType::P2trMulti)`. Per-exporter framing dropped (P0 recon Framing B confirmed: all 8 `wallet_import/*.rs` parsers are uniformly taproot-agnostic; the gap was a single envelope-emit gate). New test cell `p_slug4_taproot_envelope_refused_on_from_import_json` covers 4 formats × 2 descriptors (P2tr + P2trMulti) = 8 sub-assertions. Fix-β (envelope wire-shape evolution to carry `taproot_internal_key`) remains open for v0.29+.
- **Tier:** `v0.28+`.
- **Companion:** `wallet-import-bitcoin-core-taproot-emission` (if a sibling-codec-side FOLLOWUP surfaces).

### `plan-smoke-step4-ms1-on-bundle-not-supported` — plan-doc §6.3 smoke recipe references nonexistent `--ms1` on bundle subcommand

- **Surfaced:** 2026-05-19, v0.27.0 Phase 5 R0 review M2.
- **Where:**
  - `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md` §6.3 step 4 (end-user smoke recipe).
- **What:** v0.28+ doc-only fix: the smoke recipe step 4 says `mnemonic bundle --import-json /tmp/env.json --ms1 "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"`. `BundleArgs` doesn't expose `--ms1` (that's import-wallet's surface); seed overlay on bundle is `--slot @N.phrase=` only. Rewrite step 4 as `--slot @0.phrase="..."`.
- **Why deferred:** Plan-doc bug; smoke recipe is informational and the load-bearing acceptance is the integration cell `cross_format_bsms_to_bitcoin_core_to_import_round_trip` (which uses the correct flags).
- **Status:** `resolved 9d7eeaf79332b188c2eacc74ec712591d215349e` — mnemonic-toolkit-v0.28.5 cycle replaced the nonexistent `--ms1` flag in §6.3 step 4 with `--slot @0.phrase=` per `mnemonic bundle --help`.
- **Tier:** `v0.28+` (doc-only).
- **Companion:** none.

### `error-rs-retroactive-alphabetical-sort` — apply alphabetical-by-variant-name ordering to existing ToolkitError variants + match blocks

- **Surfaced:** 2026-05-19, v0.27.2 Task 1.1 code-quality reviewer (R0). The CLAUDE.md alphabetical-ordering Convention was added forward-looking; existing pre-v0.27.2 variants in `error.rs::ToolkitError` (~50+ variants) + 4 exhaustive match blocks (`Display`, `exit_code`, `kind`, + any debug/extra) are not yet sorted.
- **Where:** `crates/mnemonic-toolkit/src/error.rs` — enum declaration + each `match self { ... }` block that exhaustively matches `ToolkitError`.
- **What:** Sort `ToolkitError` variant declarations alphabetically by name. Reorder the corresponding arms in each exhaustive `match self` block to match. No semantic change; pure refactor.
- **Why deferred:** Out of scope for v0.27.2 (item 2 scoped as "codify the Convention", not "apply retroactively"). Retroactive sort touches `error.rs` substantially (50+ variants moved + 4 match blocks × 50 arms reordered = ~250 line moves) — better as a dedicated cleanup commit in v0.27.3 or v0.28 cycle, where the diff is clearly scoped to "no semantic change, alphabetical sort only".
- **Status:** `resolved 49cb211f230eb4becd773e2053bb63eb15fe07cc` — mnemonic-toolkit-v0.29.0 cycle, **shipped as dedicated bisect-hygiene commit** `ea2695a` (per R0-I3 lock; separate from the version-bump commit). Pure reorder: 44 variants sorted alphabetically; ~132 arm reorders across `Display`, `exit_code`, `kind` exhaustive match blocks + 1 partial-match `details` (7 named arms). All `exit_code` multi-variant `|` groupings broken into single-variant arms post-sort (new FOLLOWUP `error-rs-exit-code-arm-fragmentation-post-sort` for future re-grouping decision). Diff stat: 317 insertions + 328 deletions. P0 recon corrected count: 44 variants × 3 exhaustive blocks (not 50+ × 4 per FOLLOWUPS body — body was overstated).
- **Tier:** `v0.28+`

### `pr-26-import-provenance-three-variant-cleanup` — three-variant cleanup for ImportProvenance::Bsms(Option<_>)

- **Surfaced:** 2026-05-19, v0.27.2 Phase 2 architect R0 (Minor M1; confidence 30). The shipped `ImportProvenance::Bsms(Option<BsmsAuditFields>)` is sound (representable-invalid pair eliminated) but hides the 2-line-vs-6-line BSMS shape distinction inside an Option.
- **Where:** `crates/mnemonic-toolkit/src/wallet_import/mod.rs::ImportProvenance` enum + `bsms_audit()` accessor.
- **What:** Promote to three variants:
  ```rust
  pub(crate) enum ImportProvenance {
      BsmsTwoLine,
      BsmsSixLine(BsmsAuditFields),
      BitcoinCore(CoreSourceMetadata),
  }
  ```
  The `bsms_audit()` accessor naturally returns `Some(&audit)` only for `BsmsSixLine` and `None` for `BsmsTwoLine` and `BitcoinCore`. Eliminates the residual Option inside the variant. Update the construction site at `bsms.rs:266` to match: `audit.map(ImportProvenance::BsmsSixLine).unwrap_or(ImportProvenance::BsmsTwoLine)` or pattern-match.
- **Why deferred:** Design-aesthetic improvement, not a correctness fix. The shipped shape already eliminates the representable-invalid pair the v0.27.1 audit flagged. Patch-tier cycle (v0.27.2) doesn't need this; v0.28+ wire-shape cycle is a natural home.
- **Status:** `resolved 49cb211f230eb4becd773e2053bb63eb15fe07cc` — mnemonic-toolkit-v0.29.0 cycle. **P0 STRICT-GATE locked the "3-variant" framing as a stale-since-filing scope; actual work is a 1-variant split** (`Bsms(Option<BsmsAuditFields>)` → `BsmsSixLine(BsmsAuditFields)` + `BsmsTwoLine` unit variant) since the enum had grown to 8 variants post-v0.28 expansion (BitcoinCore + Coldcard + ColdcardMultisig + Electrum + Jade + Sparrow + Specter + Bsms). Alphabetical insertion position: `BsmsSixLine < BsmsTwoLine` (`S` < `T`); both between `BitcoinCore` and `Coldcard`. Updated all 7 accessor `match self {}` blocks at `wallet_import/mod.rs:147-266` + 5 test cells + construction site at `bsms.rs:342-345`. `bsms_audit()` accessor now naturally returns `Some(&audit)` for `BsmsSixLine` and `None` for `BsmsTwoLine`. Construction-site line drifted `:266` → `:342` since slug filing.
- **Tier:** `v0.28+`

### `gui-schema-mirror-lockstep-discipline` — codify GUI schema-mirror lockstep invariant in CLAUDE.md

- **Surfaced:** 2026-05-19, v0.27.2 + v0.11.1 lockstep cycle end-of-cycle architect review (M3). The Phase 3 inline CI fix added 8 flags to `mnemonic-gui/src/schema/mnemonic.rs` that v0.27.0 + v0.27.1 toolkit cycles never paired with a GUI schema-mirror update. The gap is cumulative — not a v0.27.2 regression — but was only revealed when v0.11.1's pin bump fired the `schema_mirror` drift gate on the accumulated delta.
- **Where:** `CLAUDE.md` Conventions section + `mnemonic-gui/CLAUDE.md` (companion convention).
- **What:** Add a Convention line codifying that any toolkit CLI surface change (clap flag add/remove/rename) MUST also update the GUI's `src/schema/mnemonic.rs` schema-mirror in the same PR or as a paired sibling PR. The drift gate (`schema_mirror` test in mnemonic-gui) fires on pin-bump, which is a lagging indicator; the lockstep PR is the leading discipline. Companion-cite the existing "Mirror invariant" clause that covers the manual.
- **Why deferred:** v0.27.2 cycle is closed; the inline fix landed the catchup. Codifying the Convention is for future cycles. Patch-tier doesn't fit; v0.28+ tier (paired with the next CLI surface change).
- **Status:** resolved (toolkit a215f31 + mnemonic-gui f5c597e; codified in both CLAUDE.mds)
- **Tier:** `v0.28+` (resolved early as docs-only close-out on master)
- **Companion:** Cross-repo — mnemonic-gui CLAUDE.md companion added at f5c597e.

### `bsms-bip129-encryption-envelope` — STANDARD/EXTENDED encryption envelope (carved out of `bsms-bip129-full-cutover`)

- **Surfaced:** 2026-05-20, v0.28.0 cycle close (Phase P14A). Carved out from the canonical `bsms-bip129-full-cutover` entry sub-item (c) so the encryption-envelope work has a dedicated tracking slug independent of the parent entry's lenient-parser deprecation cadence.
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs` — plaintext parser (4-line / 6-line) unchanged; encrypted-Round-2 path is orchestrator-side at `cmd/import_wallet.rs` (per Cycle 6/7 precedent of pre-decrypting before passing plaintext to format parsers).
  - `crates/mnemonic-toolkit/src/bsms_crypto.rs` — library shipped Cycle 7a `62da111` (20 unit cells incl. BIP-129 TV-3 cross-validation).
  - `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — orchestrator decrypt block + new `--bsms-encryption-token <FILE|->` clap arg + `read_bsms_token` helper + stdin-contention guard.
  - `crates/mnemonic-toolkit/src/error.rs` — new `BsmsMacMismatch { token_len_hex }` variant (typed per the body's original recommendation).
  - `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms_encrypted.rs` — 12 integration cells.
  - `design/agent-reports/v0_27_0-phase-2-bip129-recon.md` §2 — byte-level construction of the STANDARD/EXTENDED encryption envelopes (inherited by Cycle 7 P0 recon; no decay).
- **What:** v0.28+: ship BIP-129 §Encryption STANDARD/EXTENDED envelope support. Key derivation: PBKDF2-SHA512(`"No SPOF"`, TOKEN_raw_bytes, c=2048, dkLen=32) → ENCRYPTION_KEY → HMAC_KEY = SHA256(ENCRYPTION_KEY). Decrypt: AES-256-CTR over ciphertext; verify HMAC-SHA256 MAC. CLI surface: new flag `--bsms-encryption-token <FILE|->` carrying the raw nonce. Cross-impl smoke against Coinkite Python ref (`github.com/coinkite/bsms-bitcoin-secure-multisig-setup` `test.py`). Refusal text on bad token / MAC mismatch should be discriminated from format errors with its own typed error variant.
- **AES-CTR variant disambiguation note:** the BIP citing RFC 3686 leaves the counter-width choice ambiguous (RFC 3686 uses a nonce+IV+counter split). Coinkite Python `bsms/encryption.py:34` (`pyaes.AESModeOfOperationCTR(key, pyaes.Counter(int(iv.hex(), 16)))`) treats the full 16-byte IV as a single 128-bit big-endian counter. Cycle 7a R0 opus review caught the `Ctr64BE` vs `Ctr128BE` ambiguity in the brainstorm skeleton; `Ctr128BE<Aes256>` is the locked variant, empirically confirmed by TV-3 decrypt cell.
- **Status:** resolved (Cycle 7 / v0.31.0).
- **Resolved by:** `mnemonic-toolkit-v0.31.0` (`e2e62ce`) + `mnemonic-gui-v0.16.0` (`a1aeb5a`). End-of-cycle: 12 integration cells in `tests/cli_import_wallet_bsms_encrypted.rs` + 20 unit cells in `bsms_crypto::tests`; install-pin-check CI green on tag; GUI schema_mirror green.
- **Tier:** `v0.28+`
- **Tags:** `wallet`
- **Companion:** parent canonical entry `bsms-bip129-full-cutover` (sub-item (c)). Cycle 7 child slugs: `bsms-encryption-per-signer-tokens`, `bsms-encryption-round1-decrypt-then-verify`, `bsms-encryption-cross-impl-coinkite-python-smoke`.

### `wallet-import-jade-seedqr` — Blockstream Jade SeedQR ingest surface

- **Surfaced:** 2026-05-20, v0.28.0 cycle close (Phase P14A). Deferred from Phase P5 per the Q1 lock; filed forward for v0.28+.
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/jade.rs` (v0.28.0 P5 ships the `get_registered_multisig` JSON reply shape; SeedQR shape diverges).
  - `docs/manual/src/45-foreign-formats.md` §"What's NOT supported" — references this slug as the SeedQR-side carve-out.
- **What:** v0.28+: ingest Blockstream Jade SeedQR shape — BIP-39 entropy encoded as a numeric-string QR payload per Jade firmware's SeedQR convention. May be folded into the `--format jade` parser via a sniff branch, OR shipped as a distinct `--format jade-seedqr` value depending on user-direction (single-blob vs multi-format ambiguity). Either path needs sniff signature, ms1 entropy extraction, and the same envelope shape as P5.
- **Why deferred:** Phase P5 cycle scope was the JSON `get_registered_multisig` reply shape only (per Q1 lock). SeedQR is a distinct surface (numeric encoding vs JSON; entropy-only vs wallet-policy) and warrants its own cycle.
- **Status:** resolved (superseded by `seedqr-encode-decode-subcommand` per Cycle 5 / v0.30.0). The architectural pivot in Cycle 5 brainstorm rejected the wallet-import framing — SeedQR carries a BIP-39 SEED (not a wallet policy), and is an open SeedSigner spec (not Jade-proprietary). Shipped as a top-level `mnemonic seedqr decode|encode` subcommand instead. See `design/BRAINSTORM_v0_30_0_seedqr.md` §"Architectural pivot" for the rationale.
- **Resolved by:** Cycle 5 / `mnemonic-toolkit-v0.30.0` (`56dd2b6`) + paired `mnemonic-gui-v0.15.0` (`5582e22`).
- **Tier:** `v0.28+`
- **Tags:** `wallet`
- **Companion:** parent `wallet-import-jade` (resolved v0.28.0; this is the deferred SeedQR carve-out, now resolved by a different surface). New slug `seedqr-encode-decode-subcommand` documents the v0.30.0 implementation.

### `wallet-import-electrum-encrypted` — encrypted Electrum 4.x wallet ingest

- **Surfaced:** 2026-05-20, v0.28.0 cycle close (Phase P14A). Deferred from Phase P6 per the Q2 lock; filed forward for v0.28+.
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/electrum.rs` (v0.28.0 P6 ships plaintext Electrum 4.x wallet-file ingest; v0.30.1 downgrades the encrypted-wallet refusal at L305-313 to a watch-only-passthrough advisory).
  - `docs/manual/src/45-foreign-formats.md` §"Encrypted wallets" — documents the watch-only-passthrough behavior + the out-of-band `electrum --decrypt-wallet` recipe for seed extraction.
- **What:** v0.28+ filing assumed full decrypt was needed. Cycle 6b R0 opus review (2026-05-21) corrected this: per Electrum's `electrum/keystore.py`, the field-level encryption protects ONLY `keystore.{seed,xprv,passphrase,keypairs}`. The fields the toolkit parser reads (`keystore.{xpub,derivation,root_fingerprint,label}` + multisig analogues) are PLAINTEXT under both encrypted and unencrypted wallets. The pre-v0.30.1 refusal was over-restrictive in principle — watch-only import has all the material it needs without touching the encrypted fields. v0.30.1 ships the watch-only-passthrough: refusal → stderr NOTICE advisory + parse continues with the plaintext xpub/derivation/etc.
- **Scheme citation correction:** the original body cited "stretched-key envelope" and "PBKDF2 + AES-CBC over the wallet file body". Both claims were wrong:
  - Electrum's actual field-level scheme is **`sha256d(password) + AES-256-CBC + PKCS7 + base64`** (no key-stretching; no PBKDF2). Verified at Cycle 6 P0 recon §A1 against `electrum/crypto.py::_pw_decode_raw`.
  - The "wallet file body" claim conflated Format A (field-level encryption inside plaintext JSON; Cycle 6 scope) with whole-file storage encryption (out of scope, filed as `wallet-import-electrum-encrypted-storage-format-b`). NOTE: the storage-encryption scheme was itself further mis-cited in that followup as "version-byte + 4-byte MAC" — corrected at Cycle 19 P0 recon (2026-05-21) to its actual form, ECIES `BIE1` (PBKDF2-SHA512→mod-n EC scalar + ECDH + sha512 KDF + AES-128-CBC + HMAC-SHA256 + zlib). See that followup's "CRYPTO RE-IDENTIFIED" note.
- **Status:** resolved (watch-only-passthrough per Cycle 6b R0 fold; v0.30.1).
- **Resolved by:** `mnemonic-toolkit-v0.30.1` (`11fd38f`). End-of-cycle opus review of Cycle 6a brainstorm (verdict RED → Path A fold; persisted at `design/agent-reports/v0_31_0-brainstorm-r0-review.md`); plan-doc R0 YELLOW (4 mechanical Importants folded inline; persisted at `design/agent-reports/v0_30_1-plan-doc-r0-review.md`).
- **Tier:** `v0.28+`
- **Tags:** `wallet`
- **Companion:** parent `wallet-import-electrum` (resolved v0.28.0). Cycle 6 child slugs: `electrum-crypto-seed-extraction-subcommand` (future use of the 6a-shipped library) + `wallet-import-electrum-encrypted-storage-format-b` (Format B carve-out).

### `wallet-import-format-mismatch-matrix-completion` — cross-format mismatch symmetry

- **Surfaced:** 2026-05-19, promoted from `design/v0_28_0-cycle-followups.md` during Phase P14A triage. Surfaced during Phase P1C-v2 + P2C-v2 execution (Site 2 wiring discovery); extended each per-parser P{N}C.
- **Where:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (each `Some("X")` arm at Site 2). **As-filed (v0.28.0 P1C-v2 era):** BSMS arm checks only BitcoinCore sniff; BitcoinCore arm checks only BSMS sniff; ColdcardMultisig arm checks BSMS + BitcoinCore; Sparrow arm checks BSMS + BitcoinCore + ColdcardMultisig; Specter arm checks BSMS + BitcoinCore + ColdcardMultisig + Sparrow. **Post-v0.28.0 expansion** (verified by recon dossier `cycle-prep-recon-followups-v0_28_plus.md` 2026-05-20): Coldcard arm has grown to 5-format coverage (P3C); Electrum arm to 6-format (P6C); Jade arm to 7-format complete matrix (P5C). The body's narrow-matrix framing pre-dates these expansions.
- **What:** v0.28+: complete the N×N format-mismatch matrix symmetrically so EVERY `--format X` arm refuses EVERY other parser's positive sniff. v0.26.0 wired the BSMS ↔ BitcoinCore pair; v0.28.0 P1C/P2C extended Sparrow's + Specter's coverage; v0.28.0 P3C/P5C/P6C extended Coldcard/Jade/Electrum further; v0.28.4 added ColdcardMultisig as an export variant but did NOT extend the import-side mismatch matrix to refuse its sniff symmetrically. **The narrow-arm residuals are now: BSMS (1 format only), BitcoinCore (1), ColdcardMultisig (2).** The inverse wires (e.g., `--format bsms` mismatching a Sparrow / Specter / Jade / Electrum / Coldcard sniff) are NOT wired; the mismatch lands in a benign fallthrough (`ImportWalletParse` exit 2 vs the symmetric `ImportWalletFormatMismatch` exit 1) — same user-visible "this doesn't work" message, different exit code + stderr template. **Re-validate the exact narrow-arm set at brainstorm-write** by grepping `ImportWalletFormatMismatch` blocks per arm in `cmd/import_wallet.rs`; the body's claim that Sparrow checks only 3 / Specter only 4 may have grown silently.
- **Why deferred:** Cosmetic + not load-bearing for v0.28.0 cycle correctness; full matrix completion is a hardening pass, not a correctness gap.
- **Status:** `resolved ffcd336e76ee1e2b74a5a0b918b6bc78bef275ea` — mnemonic-toolkit-v0.28.7 cycle Option B narrow set (per P0 user lock 2026-05-20). Extended the 3 narrow arms (BSMS / BitcoinCore / ColdcardMultisig) to full off-diagonal coverage: 17 new `ImportWalletFormatMismatch` return sites in `cmd/import_wallet.rs` + new test file `tests/cli_import_wallet_format_mismatch_matrix.rs` with 17 cells. P0 recon discovered 4 additional arms with residual gaps (Coldcard 2 + Sparrow 4 + Specter 3 + Electrum 1 = 10 more arms) — filed as NEW FOLLOWUP `wallet-import-format-mismatch-matrix-completion-discovered-gaps`.
- **Tier:** `v0.28+`
- **Tags:** `wallet`
- **Companion:** none (symmetric emit-side gap tracked at `green-emitter-multisig-refusal-template-only`).

### `bsms-import-taproot-refusal-parity` — BSMS parser should refuse tr() blobs at parse time (+ `extract_threshold` regex side-channel)

- **Surfaced:** 2026-05-19, promoted from `design/v0_28_0-cycle-followups.md` during Phase P14A triage. Surfaced during Phase P9B execution (instance G3, `v0.28.0/g3-bsms-fixtures`).
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs` `BsmsParser::parse` — current parser ACCEPTS taproot at parse time, only skipping the first-address-verify WARNING. Asymmetric with emit side (`wallet_export/bsms.rs:69-76`) which refuses taproot with `BadInput("--format bsms does not support taproot descriptors; BIP-129 §1 prerequisites pre-date BIP-386. ...")`.
  - `crates/mnemonic-toolkit/src/wallet_import/bsms.rs::extract_threshold` regex — does NOT match `sortedmulti_a(` (the `_a` taproot variant). For `tr(NUMS, sortedmulti_a(2, ...))`, the regex returns `Ok(None)` and the CLI summary emits `threshold=none`. **Side-channel finding** — a parser that refuses tr() at the top eliminates this stay-behind hazard entirely.
  - `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs::bsms_2line_tr_nums_current_behavior_no_refusal` — pins the current (pre-refusal) behavior; cell-name preserves plan-doc's forward-looking intent via the suffix `_current_behavior_no_refusal`.
- **What:** v0.28+: add a `Tr(_)` short-circuit at the top of `BsmsParser::parse` mirroring `wallet_export/bsms.rs:69-76`'s emit-side refusal. Refusal text re-uses the same substring ("does not support taproot descriptors; BIP-129 §1 prerequisites pre-date BIP-386") for parity. Cell renamed to `bsms_tr_nums_refused` per plan-doc R1-M2 wording and asserts exit-2 with `ImportWalletParse` containing the substring. Requires SPEC §10 amendment declaring tr() refusal alongside the 4-line shape lock.
- **Why deferred:** P9B's plan-doc scope was `~0 src + ~250 tests + 4 fixture files`. Modifying the parser to refuse tr() is a source-code change with normative-SPEC implications — out of P9B's authored scope. Low-priority because the emit-side refusal already prevents users from generating tr() blobs via the toolkit; import-side hole is only triggered by externally-coordinated tr() BSMS blobs (currently rare in the wild).
- **Status:** `resolved ffcd336e76ee1e2b74a5a0b918b6bc78bef275ea` — mnemonic-toolkit-v0.28.7 cycle. New `enum ToolkitError` variant `BsmsTaprootImportRefused` (alphabetically inserted BEFORE `BsmsTaprootRefused` per CLAUDE.md). Parse-entry `tr(` short-circuit in `wallet_import/bsms.rs::BsmsParser::parse` fires before expensive `parse_descriptor` work. Defense-in-depth `contains("sortedmulti_a(") || contains("multi_a(")` check at top of `extract_threshold` for any code path that bypasses parse-entry refusal. Renamed existing pin cell `bsms_2line_tr_nums_current_behavior_no_refusal` → `bsms_2line_tr_nums_refused` with assertion flipped exit-0 → exit-2. New cell `bsms_tr_sortedmulti_a_refused_via_extract_threshold_guard`. Defense-in-depth direct unit-test gap filed as NEW FOLLOWUP `bsms-extract-threshold-defense-in-depth-direct-unit-test`.
- **Tier:** `v0.28+`
- **Tags:** `wallet`
- **Companion:** `bsms-taproot-emit` (symmetric emit-side scaffold; cross-cite SPEC §10 amendment).

### `sparrow-taproot-descriptor-passthrough-import-support` — Sparrow taproot import via descriptor-passthrough

- **Surfaced:** 2026-05-19, promoted from `design/v0_28_0-cycle-followups.md` during Phase P14A triage. Surfaced during Phase P1B-v2 execution (instance A, `v0.28.0/p1-sparrow-v2`); SPEC §11.1 implementation discovery.
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs` Step 6 (path-split at v0.31.1; previously parse-step-6 taproot refusal).
  - `crates/mnemonic-toolkit/src/wallet_export/sparrow.rs:215-219` (emit-side taproot descriptor-passthrough).
- **What:** v0.29+: Sparrow's emit ships taproot wallets as DESCRIPTOR-PASSTHROUGH (concrete `[fp/path]xpub` keys embedded in `defaultPolicy.miniscript.script` instead of `@N/**` placeholders). The P1B parse path substitutes `@N/**` placeholders and refused taproot scripts; full taproot import required a parallel parse path that detects descriptor-passthrough shape via heuristic (e.g., `[fp/path]xpub` substring vs `@N/**`) and consumes the embedded concrete-keys descriptor verbatim via `concrete_keys_to_placeholders`.
- **Why deferred:** P1B was the first per-parser cycle; taproot import is a non-trivial second parse path with its own sniff/refusal matrix. Better to ship singlesig + sortedmulti coverage first and dedicate a follow-on cycle to taproot multisig + descriptor-passthrough.
- **Status:** resolved (Cycle 8 / v0.31.1).
- **Resolved by:** `mnemonic-toolkit-v0.31.1` (`3bf0794`). Implementation at `wallet_import/sparrow.rs::parse` Step 6 path-split (`has_tr && !has_at_placeholder` → descriptor-passthrough; skips Step 5 substitution; feeds `script_template` directly through `concrete_keys_to_placeholders` → `parse_descriptor`). 6 integration cells in `tests/cli_import_wallet_sparrow_taproot.rs`. Plan-doc R0 opus review YELLOW 2C/4I/3M (caught heuristic ambiguity between taproot multisig descriptor-passthrough and taproot singlesig template-mode; folded inline with narrow refusal for template-mode + follow-on FOLLOWUP).
- **Narrowing:** taproot SINGLESIG (Bip86: `tr(@0/**)` template-mode) is NOT shipped in v0.31.1 — preserved as a narrow refusal. Tracked at follow-on FOLLOWUP `sparrow-taproot-singlesig-template-mode-import`.
- **Tier:** `v0.29+`
- **Tags:** `wallet`
- **Companion:** parent `wallet-import-sparrow` (resolved v0.28.0). Follow-on `sparrow-taproot-singlesig-template-mode-import` (v0.31+) tracks the singlesig-template-mode work.


### `sparrow-taproot-singlesig-template-mode-import` — Bip86 `tr(@0/**)` template-mode import

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.31.1 Cycle 8 close. Plan-doc R0 opus review caught that the descriptor-passthrough heuristic (`!script_template.contains("@0/**")`) does NOT classify taproot SINGLESIG correctly: Sparrow's `Bip86` template emits `tr(@0/**)` (template-mode with placeholder), not descriptor-passthrough. Cycle 8 ships taproot MULTISIG descriptor-passthrough only; preserves narrow refusal for taproot singlesig template-mode.
- **Where:**
  - `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs` Step 6 path-split (the `has_tr && has_at_placeholder` branch refuses with stderr citing this FOLLOWUP slug).
  - `crates/mnemonic-toolkit/src/wallet_export/sparrow.rs:195` (`CliTemplate::Bip86 => "tr(@0/**)"`) — confirms the template-mode shape.
  - `crates/mnemonic-toolkit/tests/cli_import_wallet_sparrow.rs:305` (`sparrow_taproot_singlesig_refused`) — refusal-side regression cell continues to enforce.
  - `crates/mnemonic-toolkit/tests/cli_import_wallet_sparrow_taproot.rs::taproot_singlesig_template_still_refused` — Cycle 8 boundary cell.
- **What:** v0.31+: ship taproot SINGLESIG (Bip86) import via template-mode substitution. Implementation: under the path-split's `has_tr && has_at_placeholder` branch, substitute `@0/**` → `[fp/path]xpub/<0;1>/*` (mirrors the non-taproot template-mode substitution at the existing Step 5 loop), then feed through `concrete_keys_to_placeholders` → `parse_descriptor`. The pipeline likely accepts `tr([fp/path]xpub/<0;1>/*)` but this is UNVERIFIED — first Phase of the follow-on cycle is a TV-style cross-validation against rust-miniscript's taproot singlesig handling.
- **Why deferred:** Cycle 8 was scoped to descriptor-passthrough specifically. Expanding to template-mode taproot singlesig is a separate path-split branch + its own integration test surface + a verification step against rust-miniscript's taproot acceptance. Better to ship the descriptor-passthrough case alone and dedicate a follow-on cycle.
- **Status:** `resolved b42b1505` — mnemonic-toolkit-v0.31.2 Cycle 9. Removed the `has_tr && has_at_placeholder` narrow-refusal branch at `wallet_import/sparrow.rs::parse` Step 6; `tr(@0/**)` now flows through the standard Step 5 `@N/**` → `[fp/path]xpub/<0;1>/*` substitution loop, producing `tr([fp/86'/0'/0']xpub.../<0;1>/*)` which `concrete_keys_to_placeholders` + `parse_descriptor` accept cleanly (Phase 0 P0 recon empirically verified at master HEAD `7fa721d`). 3 refusal-asserting cells converted to happy-path counterparts (1 in-file lib + 2 integration); new boundary cell `taproot_singlesig_envelope_blocked_by_wallet_import_taproot_internal_key` documents the orthogonal `wallet-import-taproot-internal-key` FOLLOWUP that still blocks `--from-import-json` re-emission for ALL taproot envelopes; new fixture `sparrow-singlesig-p2tr.json` closes the p2wpkh/p2sh-p2wpkh/p2tr fixture-parity gap.
- **Tier:** `v0.31+`
- **Tags:** `wallet`
- **Companion:** parent `sparrow-taproot-descriptor-passthrough-import-support` (resolved v0.31.1). Follow-on M1 (defensive substring-vs-regex widening): `sparrow-import-detection-regex-defensive-widening`.

### `sparrow-import-detection-regex-defensive-widening` — widen `has_at_placeholder` from literal `@0/**` to regex `@\d+/\*\*`

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.31.2 Cycle 9 close. End-of-cycle opus architect review M1 finding.
- **Where:** `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs:338` (`let has_at_placeholder = script_template.contains("@0/**");`).
- **What:** The descriptor-passthrough discriminator's `has_at_placeholder` check matches only the literal `@0/**` substring. For a hypothetical Sparrow MULTISIG blob that emits `@N/**` with `N ≥ 1` and no `@0/**` (e.g. a 2-of-2 starting key at index 1), `is_descriptor_passthrough` would mis-classify as passthrough and skip Step 5 substitution. **Currently inert in production**: Sparrow's emit-side at `wallet_export/sparrow.rs` builds placeholders from `(0..n)` (always starts at index 0), and the leftover-placeholder regex at `sparrow.rs:383-389` would catch any stray `@N/**` and surface as a parse error rather than feeding garbage downstream. Hardening: widen to `Regex(r"@\d+/\*\*")` for robustness against future Sparrow emit-side drift.
- **Why deferred:** Defensive hardening only; not load-bearing under current Sparrow emit invariants. Cycle 9 scope was the singlesig refusal collapse.
- **Status:** `resolved d87bf52` — mnemonic-toolkit-v0.31.4 Cycle 11. `has_at_placeholder` at `wallet_import/sparrow.rs:348-351` widened from substring `@0/**` → inline `regex::Regex::new(r"@\d+/\*\*").expect("at-placeholder regex is a fixed string literal").is_match(&script_template)`. No behavior change under current Sparrow emit invariant (`wallet_export/sparrow.rs:230` always indexes from `(0..n)` so `@0/**` is always present in template-mode); regex strictly supersets the substring. R0 caught initial LazyLock plan was wrong (zero LazyLock/once_cell usages in crate); folded to inline-Regex pattern per the precedent at sparrow.rs:555/566/678. 2 new in-file unit cells: regex-unit (7+/5- cases) + backward-compat fixture regression. 2152 cells passing. End-of-cycle opus GREEN.
- **Tier:** `v0.32+`
- **Tags:** `wallet`
- **Companion:** parent `sparrow-taproot-singlesig-template-mode-import` (resolved v0.31.2).

### `coldcard-legacy-mk1-mk2-top-level-xpub-inference` — legacy Coldcard wallet.json top-level xpub support (PARSER IMPLEMENTED; fixture + tests remain)

- **Surfaced:** 2026-05-19, promoted from `design/v0_28_0-cycle-followups.md` during Phase P14A triage. Surfaced during Phase P3 execution (instance C, `v0.28.0/p3-coldcard-v2`).
- **Where:** `crates/mnemonic-toolkit/src/wallet_import/coldcard.rs:460-462` (legacy top-level `xpub` fallback) + `:471-494` (`infer_bip_from_xpub_prefix` with SLIP-132 prefix mapping: `xpub`/`tpub` → BIP-44, `ypub`/`upub` → BIP-49, `zpub`/`vpub` → BIP-84).
- **What:** **PARSER IMPLEMENTATION COMPLETED in commit `1304932` (v0.28.0 P3-v2 cycle).** The implementation is more nuanced than the original FOLLOWUP body anticipated — instead of falling back to BIP-44 default, it infers BIP from SLIP-132 xpub prefix per Coldcard firmware history. **Remaining gap:** no fixture (e.g., `coldcard-legacy-mk1-or-mk2-*.json`) in `tests/fixtures/wallet_import/`; no test cell exercising the legacy fallback in `tests/cli_import_wallet_coldcard.rs`. Future cycle: file a legacy-firmware fixture (authentic Mk1/Mk2 wallet.json export OR hand-crafted matching the firmware-historical shape) + add ≥1 test cell per SLIP-132 prefix (BIP-44/49/84) + a refusal cell for unrecognized prefixes (per the `Err` arm at L490-493).
- **Why deferred:** Test-coverage hardening. The implementation works (verified by recon dossier `cycle-prep-recon-followups-v0_28_plus.md` 2026-05-20); the gap is empirical coverage of the legacy shape.
- **Status:** `resolved c86d45eeeb6976fcf3cb6194f60b2befac2318fd` — mnemonic-toolkit-v0.28.6 cycle added 3 legacy fixtures (`coldcard-mk1-legacy-bip{44,49,84}-mainnet.json`) carrying canonical SLIP-132 published test vectors (xpub/ypub/zpub from the spec's "Bitcoin Test Vectors" section) + 4 test cells in `tests/cli_import_wallet_coldcard.rs` covering the SLIP-132 prefix inference (BIP-44/49/84 happy paths via `pkh(`/`sh(wpkh(`/`wpkh(` descriptor wrappers + 1 unrecognized-prefix refusal asserting `"unrecognized SLIP-132 prefix"` stderr). Parser implementation landed in commit `1304932` (v0.28.0 P3-v2 cycle); this cycle closes the test-coverage gap.
- **Tier:** `v0.29+-test-coverage`
- **Tags:** `wallet`
- **Companion:** parent `wallet-import-coldcard` (resolved v0.28.0).

### `green-emitter-multisig-refusal-template-only` — Green's multisig refusal misses descriptor-mode

- **Surfaced:** 2026-05-19, promoted from `design/v0_28_0-cycle-followups.md` during Phase P14A triage. Surfaced during Phase P11C execution (instance Wave-2, `v0.28.0/p11-cross-format-matrix`); cross-format refusal matrix probe.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/green.rs:30-44` (the `if let Some(t) = inputs.template` guard skips refusal entirely when `template == None`, which is the case on every `--from-import-json` invocation per `cmd/export_wallet.rs:603`).
- **What:** v0.28+: `GreenEmitter::emit` refuses multisig templates (P{2sh,2wsh,..}Multi), but the refusal is gated on `inputs.template.is_some()`. In descriptor-mode invocations (`--descriptor` or `--from-import-json`), `template` is `None`, so the multisig guard never fires — Green emits a multisig wsh()-descriptor text comment-block even though Green's actual file-import surface refuses multisig wallets at runtime. Refusal should be derived from the canonical descriptor's script-type (`script_type_from_descriptor`) when `template` is absent — `inputs.script_type: WalletScriptType` already encodes the multisig variants and is populated on both paths. Refactor: refuse when `inputs.script_type.is_multisig()` regardless of template presence.
- **Why deferred:** The matrix-test fix (filter green out of the multisig-refusal matrix and pin the current behavior with a regression cell) was scoped to P11C. Patching green's emitter is OOS for P11C (Phase 11 is matrix-coverage, not refusal-contract reshuffle); changing `GreenEmitter::emit` would affect `cli_export_wallet_green.rs` multisig-refusal cells that currently use templated input.
- **Status:** `resolved ffcd336e76ee1e2b74a5a0b918b6bc78bef275ea` — mnemonic-toolkit-v0.28.7 cycle. New `WalletScriptType::is_multisig()` method in `wallet_export/mod.rs` covers `P2shMulti | P2shP2wshMulti | P2wshMulti | P2trMulti`. Refactored `wallet_export/green.rs:30-44` refusal guard from `if let Some(t) = inputs.template { if t.is_multisig() { ... } }` → `if inputs.script_type.is_multisig() { ... }`. Closes the bug where descriptor-mode (`--from-import-json`) multisig green exports silently passed despite Green's import surface being singlesig-only. New test cell `cell_4_green_descriptor_mode_multisig_refuses` + pre-existing canary `p11c_green_descriptor_passthrough_current_behavior_no_refusal` flipped to `p11c_green_descriptor_passthrough_singlesig_passes_multisig_refused` (3 singlesig sources pass, 5 multisig sources refuse).
- **Tier:** `v0.28+`
- **Tags:** `wallet`
- **Companion:** `wallet-import-format-mismatch-matrix-completion` (symmetric import-side matrix-gap).

### `import-wallet-envelope-schema-version-narrative-drift` — outer envelope `schema_version` vs inner `BundleJson.schema_version` collision

- **Surfaced:** 2026-05-19, promoted from `design/v0_28_0-cycle-followups.md` during Phase P14A triage. Surfaced during Phase P11A execution; helper test `p11a_helper_envelope_carries_schema_version_and_source_format` asserted schema_version="4" per a misread.
- **Where:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:87` (outer envelope const `IMPORT_WALLET_ENVELOPE_SCHEMA_VERSION = "1"`) vs `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:975` (inner BundleJson literal `"4"`).
- **What:** v0.28+: the dual `schema_version` fields share a name but have independent rev numbers (envelope wire-shape vs BundleJson wire-shape). Per CLAUDE.md plan-doc verification discipline, this duality is a silent footgun for future readers / parser authors. Recommend renaming one to disambiguate (`envelope_schema_version` vs `bundle_schema_version`) OR adding a doc-comment at both sites cross-referencing the other.
- **Why deferred:** Rename is wire-shape-breaking; affects GUI schema mirror + every downstream JSON consumer. Documentation fix is low-risk but OOS for P11 (matrix coverage, not envelope redesign).
- **Status:** `resolved 9d7eeaf79332b188c2eacc74ec712591d215349e` — mnemonic-toolkit-v0.28.5 cycle added cross-reference doc-comments at both `schema_version` constant sites in `cmd/import_wallet.rs` (outer envelope L87 + inner BundleJson literal at L975); future readers / parser authors now have at-site disambiguation between the two fields.
- **Tier:** `v0.28+`
- **Tags:** `wallet`
- **Companion:** none.


### `export-wallet-coldcard-multisig-alias` — `--format coldcard-multisig` on export side (alias for `coldcard` + multisig template)

- **Surfaced:** 2026-05-20, manual-v0.2.0 content-audit cycle finding F4. P1b R0 architect classification at `design/agent-reports/manual-v0_2_0-p1b-r0-classification.md` §F4.
- **Where:**
  - Import side accepts `--format coldcard-multisig` (sniffs Coldcard's text multisig setup file): `crates/mnemonic-toolkit/src/wallet_import/coldcard_multisig.rs` (parser); CLI flag accepted by `cmd/import_wallet.rs` enum.
  - Export side does NOT accept `--format coldcard-multisig`: `crates/mnemonic-toolkit/src/wallet_export/mod.rs` `enum CliExportFormat` — only `Coldcard` variant exists. Multisig text emit is reached via `--format coldcard --template wsh-sortedmulti` (and equivalent multisig templates), which template-dispatches to `emit_coldcard_multisig_text` (`wallet_export/coldcard.rs:42-55`).
- **What:** v0.28+ ergonomic-surface fix: add `coldcard-multisig` as a `CliExportFormat` variant that aliases to `Coldcard` with a multisig-template precheck (refusal pointer for singlesig templates). Surfaces flag-name parity between the import and export value sets so a reader who sees `--format coldcard-multisig` accepted on import doesn't trip on the asymmetric export rejection. Requires paired `mnemonic-gui/src/schema/mnemonic.rs` update per CLAUDE.md schema-mirror invariant.
- **Why deferred:** manual-v0.2.0 cycle is content-audit-only at scope; F4's user-facing prose fix (use `--format coldcard --template wsh-sortedmulti --threshold 2` + add asymmetry note paragraph) is sufficient for the documentation deliverable. Toolkit surface-enlargement triggers GUI schema-mirror lockstep and warrants its own cycle.
- **Status:** `resolved 5ef6aae7d7f60a485eb10a1637be473f96fd9ab0` — mnemonic-toolkit-v0.28.4 cycle added `CliExportFormat::ColdcardMultisig` variant with multisig-template precheck. 4 dispatch arms (2 in run(), 2 in run_from_import_json()) delegate to existing `ColdcardEmitter::emit` for multisig templates; refuse singlesig with pointer text to `--format coldcard`. Chapter-45 asymmetry note rewritten to "Format-name parity (v0.28.4+)" historical-context. Paired GUI tag `mnemonic-gui-v0.13.0` bumps schema-mirror to consume the new value.
- **Tier:** `v0.28+`
- **Tags:** `wallet`
- **Companion:** mnemonic-gui-v0.13.0 (paired tag; schema-mirror lockstep).

### `emitinputs-canonical-descriptor-checksum-invariant-enforcement` — defensive type/assertion for the `#checksum` invariant on `EmitInputs.canonical_descriptor`

- **Surfaced:** 2026-05-20, manual-v0.2.0 cycle. Forward-looking observation from F9 c2-B fold review (P1b R1 architect §F9 Axis B → synthesis "Forward-looking toolkit observation").
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/mod.rs` (`EmitInputs.canonical_descriptor: &str`); `wallet_export/bsms.rs:86-90` (the documented invariant comment); construction sites: `cmd/export_wallet.rs` (`--template` path via `build_descriptor_string`; `--from-import-json` path via the F9 v0.28.2 `parsed_ms.to_string()` re-emit; `--descriptor` passthrough via the SPEC §6 pre-canonicalization).
- **What:** v0.29-cleanup: enforce the "`canonical_descriptor` ends with `#<8-char-csum>`" invariant beyond convention. Options: (a) make `EmitInputs::new(...)` a constructor that asserts the checksum suffix; (b) change `canonical_descriptor` type from `&str` to a newtype `CheckedDescriptor<'_>(&'_ str)` whose constructor validates `#<csum>`. Either approach guarantees the F9 class cannot regress silently when a future code path constructs `EmitInputs` from a stripped-body descriptor.
- **Why deferred:** F9 v0.28.2 fix (commit `615b10e`) restored the invariant at the active construction site (`--from-import-json` path); regression-tested by the 2 new test cells in `tests/cli_export_wallet_from_import_json.rs` (`f9_*`). This FOLLOWUP is defensive engineering — preventing recurrence — not a behavioral bug-blocker. The newtype option is the structurally cleaner of (a)/(b) but is a wider refactor.
- **Status:** `resolved d72e23cc13010e51f4befa54172f7161f9080322` — mnemonic-toolkit-v0.28.3 cycle landed option (b): `CheckedDescriptor<'_>(&'_ str)` newtype in `wallet_export/mod.rs` enforcing the BIP-380 `#<csum>` invariant at compile time. `EmitInputs.canonical_descriptor` field type changed from `&str` → `CheckedDescriptor<'_>`; 2 construction sites in `cmd/export_wallet.rs:438+609` wrap via `CheckedDescriptor::new(...)?`; 5 inline unit cells in `wallet_export/mod.rs#[cfg(test)] mod checked_descriptor_tests` lock the constructor contract; total toolkit cells 1996 → 2001.
- **Tier:** `v0.29-cleanup`
- **Tags:** `wallet`
- **Companion:** parent F9 fix (commit `615b10e`).


### `manual-md-bin-real-binary-promote` — promote `MD_BIN` from placeholder to real `md` binary in CI manual.yml

- **Surfaced:** 2026-05-20, manual-v0.2.0 cycle P4 partition. Successor to the partial closure of `manual-yml-bind-real-mnemonic-bin` (resolved-partial at commit `52f33f7`).
- **Where:** `.github/workflows/manual.yml` "Audit manual" step (post-`52f33f7`) passes `MD_BIN=true` to `make audit`. The flag-coverage gate at `docs/manual/tests/lint.sh` per-subcommand `--help` extraction short-circuits via "no flags parsed... skipping" warnings for every `md` subcommand. Same gap class as the pre-cycle mnemonic-side situation, restricted to the `md` sibling-codec CLI.
- **What:** v0.28+ ci-hygiene: add a `cargo install --git https://github.com/bg002h/descriptor-mnemonic --tag descriptor-mnemonic-md-cli-v<latest> md-cli` step to `manual.yml` analogous to the existing mk-cli install at lines 72-77. Then pass `MD_BIN=md` to `make audit`. The flag-coverage gate at `docs/manual/tests/lint.sh` will then exercise `md <subcommand> --help` against the real binary. Pin the tag to the install.sh-locked sibling-CLI tag (currently `descriptor-mnemonic-md-cli-v0.6.0` per `scripts/install.sh:35`).
- **Why deferred:** manual-v0.2.0 cycle scope was the v0.28.0 P13A/P13B audit (chapter-45 + chapter-39 + the chapter-41 inheritance composite). MD-sibling-CLI promotion is independent — it gates `md`-chapter (chapter-42) coverage, which wasn't audited in this cycle.
- **Status:** `resolved cefffcc63e97573de78ed3c34d335a0b435cc338` — manual-v0.2.1 cycle landed real `md` binary install step in `manual.yml` mirroring the mk-cli pattern at L72-77; flag-coverage gate now exercises `md <subcommand> --help` against the cargo-installed binary.
- **Tier:** `v0.28+-ci-hygiene`
- **Companion:** `manual-ms-bin-real-binary-promote` (sibling successor; same partition).

### `manual-ms-bin-real-binary-promote` — promote `MS_BIN` from placeholder to real `ms` binary in CI manual.yml

- **Surfaced:** 2026-05-20, manual-v0.2.0 cycle P4 partition. Successor to the partial closure of `manual-yml-bind-real-mnemonic-bin` (resolved-partial at commit `52f33f7`).
- **Where:** `.github/workflows/manual.yml` "Audit manual" step (post-`52f33f7`) passes `MS_BIN=true` to `make audit`. Same gap class as the mnemonic-side situation, restricted to the `ms` sibling-codec CLI.
- **What:** v0.28+ ci-hygiene: add a `cargo install --git https://github.com/bg002h/mnemonic-secret --tag ms-cli-v<latest> ms-cli` step to `manual.yml`. Then pass `MS_BIN=ms` to `make audit`. The flag-coverage gate will then exercise `ms <subcommand> --help` against the real binary. Pin the tag to the install.sh-locked sibling-CLI tag (currently `ms-cli-v0.4.0` per `scripts/install.sh:38`).
- **Why deferred:** Same scope-rationale as `manual-md-bin-real-binary-promote` — MS-sibling-CLI promotion gates `ms`-chapter (chapter-43) coverage, which wasn't audited in this cycle.
- **Status:** `resolved cefffcc63e97573de78ed3c34d335a0b435cc338` — manual-v0.2.1 cycle landed real `ms` binary install step in `manual.yml` mirroring the mk-cli pattern at L72-77; flag-coverage gate now exercises `ms <subcommand> --help` against the cargo-installed binary.
- **Tier:** `v0.28+-ci-hygiene`
- **Companion:** `manual-md-bin-real-binary-promote` (sibling successor; same partition).

### `manual-chapters-22-23-24-post-v0.15.0-wire-format-refresh` — chapters 22/23/24 transcripts + prose are pre-v0.15.0 wire-format-broken

- **Surfaced:** 2026-05-20, manual-v0.2.0 cycle P3 verify-examples.sh wiring. The new `make audit` CI gate surfaced the drift; pre-cycle the placeholder `MNEMONIC_BIN=true` masked it (the gate silently passed against a no-op binary).
- **Where:** `docs/manual/transcripts/{22-first-bundle,23-verify,24-recover,24-recover-md1}.{cmd,out}` + the corresponding chapter prose at `docs/manual/src/20-quickstart/{22-first-bundle,23-verify,24-recover}.md`. The captured `.out` files carry pre-v0.15.0 wire-format card strings (ms1/mk1/md1 prefixes that the v0.28.x decoder no longer accepts — `md1zsxdspq...` vs current `md1fgdxlpq...`). 23-verify's captured output is `result: mismatch` because the embedded md1 strings fail v0.28.x decode with `WireVersionMismatch { got: 1 }`. The drift dates from v0.15.0 (per memory `project_v0_15_0_md_codec_catchup_closed` "Wire-format clean break: v0.14.x bundles forward-incompatible").
- **What:** Refresh both the transcript captures AND the chapter prose. The captures need rerunning against v0.28.x; the prose needs re-audit (claim verification) against the new captured output. **Chapter scope (9 total)** = 3 quickstart (22-first-bundle, 23-verify, 24-recover) + 6 cross-reference chapters (31-singlesig-steel, 35-recovery-paths, 41-mnemonic, 42-md, 43-ms, 44-mk-cli) that mention the stale card strings (per `grep -l 'ms10entrsq\|mk1qprsqhp\|md1zsxdsp' docs/manual/src/**/*.md`). Per Q2 (manual-v0.2.0 cycle scope lock), this work is OUT-OF-SCOPE for v0.2.0 (the cycle is audit of v0.28.0 P13A/P13B files only). **(2026-05-20 P0 recon update for the manual-v0.3.0 cycle that consumes this FOLLOWUP):** Cycle 4 P0 recon discovered that the original "60 stale-string hits across 9 chapters" framing over-counted; ms1 encoding is wire-format-stable post-v0.15.0 (entropy bytes don't change) and the abandon-vector BIP-84 mk1 happens to be byte-stable too. Only md1 (wallet-policy) actually changed. **Actual refresh scope: 15 md1-stale hits across 6 chapters** — 22-first-bundle (4) + 23-verify (3) + 24-recover (3) + 31-singlesig-steel (3) + 42-md (1) + 44-mk-cli (1); chapters 35/41/43 are clean (only contain current ms1 hits). Effort estimate revised: ~half-day to 1 day, not 3-5 days.
- **Why deferred:** Scope partition: manual-v0.2.0 was scoped to the v0.28.0 P13A/P13B chapter set (45 + 39 + 41-inheritance). Refreshing the affected chapters (per the P0-recon-narrowed scope above) is a multi-chapter audit that warrants its own cycle.
- **Status:** `resolved 83ad6ddb62b8feb56f8aea9a42ee1c791314e53c` — manual-v0.3.0 cycle (Cycle 4 / Wave 2 second). P0 recon (`design/AUDIT_FINDINGS_manual_v0_3_0.md` at `fad38ab`) narrowed scope from 9-chapter / 60-stale-hits / 3-5 days to 6-chapter / 15-md1-hits / ~half-day. P1a: 4 transcript recaptures (22-first-bundle, 23-verify, 24-recover, 24-recover-md1) against v0.28.4 binary; cascade 22→23 md1-cmd-arg dependency handled. P2: 15 md1zsxdsp* → md1fgdxlpq* prose replacements across 6 chapters + 2 ellipsis-variant fixes. P3: SKIP_STEMS array + is_skipped helper + call-site filter removed from verify-examples.sh (−34 LOC). P4: local `make audit` GREEN 14/14. P5: opus end-of-cycle review 0C/0I/0M; cascade verified end-to-end. Sibling CLIs (md/ms/mk) invoked transitively through recaptured transcripts; no toolkit binary changes.
- **Tier:** `manual-v0.3+-audit`
- **Companion:** None remaining (SKIP_STEMS removed; verify-examples.sh now exercises all 14 transcripts).


### `cross-format-refusal-matrix-include-coldcard-multisig` — extend cross-format refusal matrix to cover new ColdcardMultisig variant

- **Surfaced:** 2026-05-20, mnemonic-toolkit-v0.28.4 cycle opus reviewer §Cross-cutting. Filed inline per architect's PROCEED-TO-COMMIT with Important-as-FOLLOWUP recommendation.
- **Where:** `crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json.rs:592-593` carries `TEMPLATE_ONLY_DESTS = &["coldcard", "electrum", "jade", "sparrow"]`; `REFUSAL_STDERR_PATTERNS` at L815 matches via substring `"requires --template"`; cell count assertion at L871 hardcodes `32 = 8 × 4` (sources × destinations).
- **What:** Add `"coldcard-multisig"` to `TEMPLATE_ONLY_DESTS` so the refusal-matrix exercises the new variant on the `--from-import-json` path. BUT: the new arm's refusal text is `"--format coldcard-multisig requires a multisig --template"` — the substring `"requires --template"` is NOT present (intervening word "a multisig"). Either: (a) broaden `REFUSAL_STDERR_PATTERNS` to ALSO match `"requires a multisig --template"`, OR (b) tighten the new arm's refusal text to `"--format coldcard-multisig requires --template (must be multisig...)"`. Plus bump the matrix size assertion 32 → 40 (5 dests × 8 sources).
- **Why deferred:** v0.28.4 cycle scope was the variant + dispatch arms + GUI lockstep. Matrix-test extension is independent test hygiene; would add ~30 LOC of test updates and is non-blocking (no test currently fails, since the matrix doesn't include the new variant).
- **Status:** `resolved c86d45eeeb6976fcf3cb6194f60b2befac2318fd` — mnemonic-toolkit-v0.28.6 cycle extended `TEMPLATE_ONLY_DESTS` to include `"coldcard-multisig"` (5 entries; was 4), broadened `REFUSAL_STDERR_PATTERNS` to match the v0.28.4 arm's refusal text `"requires a multisig --template"` (the intervening word `"a multisig"` made the original `"requires --template"` substring miss), and bumped the cell-count assertion 32 → 40 (8 sources × 5 dests). The pattern broadening was chosen over option (b) tightening the toolkit's refusal text — option (a) keeps the toolkit's user-facing message stable.
- **Tier:** `v0.28+-test-hygiene`
- **Tags:** `wallet`
- **Companion:** parent v0.28.4 cycle commit (toolkit side).


### `wallet-import-format-mismatch-matrix-completion-discovered-gaps` — Coldcard / Sparrow / Specter / Electrum arm residuals (post-Cycle-3 discovery)

- **Surfaced:** 2026-05-20, during Cycle 3 P0 STRICT-GATE recon (`design/cycle-3-p0-recon.md` Slug 3). The original `wallet-import-format-mismatch-matrix-completion` FOLLOWUP body listed only BSMS / BitcoinCore / ColdcardMultisig as narrow-arm residuals. P0 recon found 4 additional arms with residual gaps: Coldcard (2 missing: electrum, jade), Sparrow (4 missing: coldcard, electrum, jade, specter), Specter (3 missing: coldcard, electrum, jade), Electrum (1 missing: jade). Total: **10 additional missing `ImportWalletFormatMismatch` arms / ~10 additional test cells**.
- **Where:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — Coldcard, Sparrow, Specter, Electrum dispatch arms.
- **What:** Extend each arm to refuse all wrong-format sniff outcomes symmetrically. Closes the 8×7 = 56-cell full off-diagonal matrix.
- **Why deferred:** Cycle 3 scope was locked at Option B (original 3-arm narrow set) per user decision 2026-05-20.
- **Status:** resolved — v0.34.4. All 10 residual off-diagonal arms added (coldcard→electrum,jade; electrum→jade; sparrow→coldcard,electrum,jade,specter; specter→coldcard,electrum,jade); the 8×7 = 56-cell off-diagonal matrix is now complete (bitcoin-core/bsms/coldcard-multisig/jade were already 7/7). 10 new cells in `tests/cli_import_wallet_format_mismatch_matrix.rs`; the 4 modified block comments refreshed. Closed via cycle-prep recon audit (SHA `f4d553e`).
- **Tier:** `v0.28+-test-hygiene`
- **Tags:** `wallet`
- **Companion:** parent `wallet-import-format-mismatch-matrix-completion` (resolved v0.28.7).


### `bsms-extract-threshold-defense-in-depth-direct-unit-test` — defense-in-depth guard at `extract_threshold` is unit-test-unreachable

- **Surfaced:** 2026-05-20, mnemonic-toolkit-v0.28.7 Phase 6 end-of-cycle opus review (`design/agent-reports/v0_28_7-phase-6-end-of-cycle-review.md`).
- **Where:** `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:496-497` (`extract_threshold` defense-in-depth guard added v0.28.7 Slug 1 returning `Err(BsmsTaprootImportRefused)` for `sortedmulti_a(` / `multi_a(` substrings; cite refreshed against SHA `9b94a7d` 2026-05-22, was `~493`).
- **What:** Add a `#[cfg(test)] mod tests` unit test in `wallet_import/bsms.rs` that directly invokes `extract_threshold("tr(NUMS,sortedmulti_a(2,@0,@1))")` and asserts `Err(BsmsTaprootImportRefused)`. The integration test cell `bsms_tr_sortedmulti_a_refused_via_extract_threshold_guard` at `tests/cli_import_wallet_bsms.rs` cannot reach the guard because the parse-entry refusal at `bsms.rs:215` fires FIRST on `tr(` substring. The guard at L493 is therefore shipped untested at v0.28.7.
- **Why deferred:** Low priority — purely defense-in-depth regression-guard gap. Functional behavior is already pinned by the parse-entry guard; the guard at L496-497 is fallback protection if a future code path bypasses parse-entry refusal.
- **Status:** resolved — v0.34.3. Added `extract_threshold_refuses_taproot_multi_a_directly` to `wallet_import/bsms.rs::tests` directly asserting `extract_threshold("tr(NUMS,{sortedmulti_a,multi_a}(...))") == Err(BsmsTaprootImportRefused)` (guard at `bsms.rs:496-497`; parse-entry refusal at `:215`). Cite drift fixed (was `~493`). Closed via wallet-cluster cycle-prep recon (SHA `9b94a7d`).
- **Tier:** `v0.28+-test-hygiene`
- **Tags:** `wallet`
- **Companion:** parent `bsms-import-taproot-refusal-parity` (resolved v0.28.7).


### `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` — document that GUI schema-mirror only gates clap flag-name parity, NOT JSON wire-shape

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.29.0 Cycle 4 plan-doc R0 opus review (`design/agent-reports/v0_29_0-plan-doc-r0-review.md` §I1). Plan-doc's first draft assumed the GUI's `schema_mirror.rs` integration test would catch JSON wire-shape drift from the xpub-search tagged-enum conversion. Opus R0 caught that the gate enforces **clap flag-name set parity** between hand-maintained `SubcommandSchema` and `gui-schema` JSON output — NOT runtime JSON wire-shape from CLI subcommands.
- **Where:** `mnemonic-gui/tests/schema_mirror.rs:91-121` + `mnemonic-gui/tests/xpub_search_schema_mirror.rs` — gate iterates flag names + dropdown enum values. `mnemonic-toolkit`'s `gui-schema` subcommand at `cmd/gui_schema.rs` — emits clap surface JSON only.
- **What:** GUI's runtime consumers of `mnemonic xpub-search --json` output (or any other subcommand's `--json` output) have NO automated drift gate. They must self-update when the wire-shape changes. Options: (a) extend `gui-schema` to include per-subcommand `--json` output-shape declarations; (b) file separate per-consumer regression tests on the GUI side that exercise the `--json` output and assert shape invariants; (c) document the gap in CLAUDE.md + accept manual coordination on wire-shape evolution. **Recommended:** option (c) for v0.29.x (document); option (b) at v0.30+ for the high-traffic subcommands (xpub-search, import-wallet, export-wallet).
- **Why deferred:** v0.29.0 Cycle 4 documented the gap inline (CHANGELOG note + this FOLLOWUP) but didn't extend the gate. Extending the gate is a non-trivial design + implementation pass spanning both repos.
- **Status:** open — option (c) [document the gap in CLAUDE.md] **shipped v0.34.3** (CLAUDE.md "GUI schema-mirror coverage" section now states the gate enforces clap flag-NAME parity only, NOT runtime `--json` wire-shape). Residual = option (b): per-consumer `--json` wire-shape regression tests on the GUI side for high-traffic subcommands (`xpub-search`/`import-wallet`/`export-wallet`), v0.30+. Narrowed 2026-05-22 (SHA `9b94a7d`).
- **Tier:** `v0.29+`
- **Tags:** `wallet`
- **Companion:** none (cross-repo discipline gap).


### `error-rs-exit-code-arm-fragmentation-post-sort` — record that post-sort, all `exit_code` arms are single-variant; re-grouping for readability is a separate decision

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.29.0 Cycle 4 plan-doc R0 opus review (`design/agent-reports/v0_29_0-plan-doc-r0-review.md` §I2). Pre-Cycle-4 `error.rs::exit_code` match used multi-variant `|` groupings (e.g., 18 variants `=> 2`); post-Cycle-4 alphabetical sort interleaves different-exit variants, forcing every arm to single-variant form. Post-sort the file has 44 single-variant arms.
- **Where:** `crates/mnemonic-toolkit/src/error.rs` `exit_code` match block (post-Cycle-4: L428-473).
- **What:** Decide future readability stance. Three options: (a) keep single-variant arms forever (clearest 1:1 variant→code mapping; CLAUDE.md alphabetical lock takes priority over grouping); (b) re-group by exit code in a non-alphabetical block (sacrifices alphabetical-by-variant lock); (c) introduce a separate `const EXIT_CODE_TABLE: &[(&str, u8)]` and use a function dispatch (decouples ordering from grouping; new abstraction layer). **Recommended:** (a) — accept the fragmentation as the cost of alphabetical lock + low-friction grep.
- **Why deferred:** Readability decision, not a correctness or convention question. The 44-arm fragmented form ships v0.29.0; re-grouping if desired can land any future cycle.
- **Status:** `resolved` — closed with decision (a): keep the single-variant `exit_code` arms (the CLAUDE.md alphabetical-by-variant lock takes priority over grouping; 1:1 variant→code mapping + low-friction grep). Post-v0.34.0 the block is 45 single-variant arms (`error.rs:438`). No code change. cycle-prep recon 2026-05-22, SHA `1d6436d`.
- **Tier:** `v0.29+`
- **Tags:** none
- **Companion:** parent `error-rs-retroactive-alphabetical-sort` (resolved v0.29.0).


### `seedqr-encode-decode-subcommand` — canonical reference for the v0.30.0 SeedQR subcommand (resolved)

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.30.0 Cycle 5 brainstorm (`design/BRAINSTORM_v0_30_0_seedqr.md`). Replaces the predecessor wallet-import framing under the slug `wallet-import-jade-seedqr`.
- **Where:** `crates/mnemonic-toolkit/src/seedqr.rs` (library: `decode` / `encode` + `SeedqrError`) + `crates/mnemonic-toolkit/src/cmd/seedqr.rs` (CLI: `SeedqrArgs` + `map_seedqr_error`) + `docs/manual/src/40-cli-reference/41-mnemonic.md` (manual chapter) + `mnemonic-gui/src/schema/mnemonic.rs` (schema-mirror entries).
- **What:** Top-level `mnemonic seedqr decode|encode` subsubcommand. Standard SeedQR only; 12 + 24 word phrases; English-locked. Library-local `SeedqrError` → `ToolkitError::BadInput` boundary mapper. JSON envelope `{schema_version, operation, variant, word_count, phrase, digits}`. Vendor-neutral slug (SeedSigner-originated open spec; adopted by Jade / Coldcard / Cobo / Krux).
- **Status:** resolved (Cycle 5 / v0.30.0).
- **Resolved by:** `mnemonic-toolkit-v0.30.0` (`56dd2b6`) + `mnemonic-gui-v0.15.0` (`5582e22`). End-of-cycle opus review GREEN (0C/0I/1M cosmetic; `design/agent-reports/v0_30_0-end-of-cycle-review.md`). install-pin-check CI green on tag.
- **Tier:** `v0.30+` (entry kept as cross-cite anchor; child v0.30+ slugs reference this as parent).
- **Tags:** none
- **Companion:** parent `wallet-import-jade-seedqr` (resolved-superseded by this slug); child slugs `seedqr-compact-variant`, `seedqr-15-18-21-word-counts`, `seedqr-bundle-slot-integration`, `seedqr-digits-from-input-unification`.


### `seedqr-compact-variant` — CompactSeedQR (binary entropy QR encoding)

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.30.0 Cycle 5 brainstorm close. Deferred from Cycle 5 per the variant-scope lock.
- **Where:** new variant code path in `crates/mnemonic-toolkit/src/seedqr.rs` + new CLI flag `--variant compact` in `crates/mnemonic-toolkit/src/cmd/seedqr.rs` + SeedSigner ref impl at `src/seedsigner/models/encode_qr.py::CompactSeedQrEncoder` (binary mode).
- **What:** Add CompactSeedQR ingest + emit. Per SeedSigner spec, CompactSeedQR encodes raw BIP-39 entropy bytes in QR's binary mode (16 bytes for 12-word phrases; 32 for 24-word). Implementation requires: (a) explicit `--variant <standard|compact>` flag (default `standard`); (b) explicit `--word-count <12|24>` flag (binary mode has no length-based disambiguation); (c) JSON envelope `variant` field already-locked at `"standard"|"compact"`. Sniff is ambiguous (16/32 raw bytes carry no distinguishing signature), so explicit flags are required.
- **Why deferred:** Cycle 5 locked Standard SeedQR only. Sniff-ambiguity for binary mode requires UX design (explicit flags + word-count required). Out of scope for the v0.30.0 introductory cycle.
- **Status:** `resolved 3dedfe7` — mnemonic-toolkit-v0.32.0 Cycle 14. New `--variant <standard|compact>` derived ValueEnum flag (default standard) on `seedqr encode` + `decode`. `encode_compact`/`decode_compact` library primitives + 3 `SeedqrError` variants. CompactSeedQR payload = raw BIP-39 entropy bytes as lowercase hex (16B/12-word, 32B/24-word); SeedSigner-faithful (12/24 only; 15/18/21 refused). Primary-source verified vs SeedSigner `CompactSeedQrEncoder`. **Superseded the FOLLOWUP-body's `--word-count` requirement** — byte-count (16/32) disambiguates word-count on decode + the phrase determines it on encode, so the explicit `--word-count` flag (item b) is NOT needed; the user-confirmed design uses hex-text representation (item: no binary file I/O this cycle) + `--variant` (item a) + dynamic JSON `variant` field (item c, already present since v0.30.0). 18 new cells; 2192 total. End-of-cycle opus GREEN. With this closure, **all four v0.30.0 SeedQR follow-ons are shipped** (bundle-slot v0.31.3 + 15/18/21-word-counts v0.31.5 + --from-unification v0.31.6 + compact-variant v0.32.0).
- **Tier:** `v0.30+`
- **Tags:** none
- **Companion:** parent `seedqr-encode-decode-subcommand` (resolved v0.30.0); follow-on `gui-seedqr-variant-flag-mirror` (GUI v0.17.0 schema_mirror lockstep — `--variant` net-new flag on seedqr-encode + seedqr-decode).

### `gui-seedqr-variant-flag-mirror` — mnemonic-gui schema mirror for the v0.32.0 seedqr --variant flag

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.32.0 Cycle 14 close. `--variant` is a NET-NEW flag NAME on BOTH `mnemonic seedqr encode` AND `mnemonic seedqr decode` — trips the GUI `schema_mirror` flag-NAME-parity gate on both subcommand schemas.
- **Where:** `mnemonic-gui/src/schema/mnemonic.rs` (`SEEDQR_ENCODE_FLAGS` + `SEEDQR_DECODE_FLAGS`); `mnemonic-gui/pinned-upstream.toml` + `Cargo.toml` toolkit pin → `mnemonic-toolkit-v0.32.0`.
- **What:** Add `--variant` (Dropdown `["standard", "compact"]`, default standard) to BOTH seedqr schema entries; bump toolkit pin v0.31.6 → v0.32.0. No `SECRET_NODE_TYPES` change this cycle (no new node type), so the supply-chain drift gate stays quiet. GUI v0.17.0 (MINOR — paired with toolkit MINOR).
- **Why deferred:** Cross-repo; ships as Cycle 14b immediately following the toolkit tag (lockstep).
- **Status:** `resolved 456d2a2` — mnemonic-gui-v0.17.0 (Cycle 14b). Added `--variant` Dropdown (`["standard","compact"]`, default standard) to both `SEEDQR_ENCODE_FLAGS` + `SEEDQR_DECODE_FLAGS` (new `SEEDQR_VARIANTS` const). Toolkit pin v0.31.6 → v0.32.0. No `SECRET_NODE_TYPES` change (CompactSeedQR added no new NodeType) so the supply-chain drift gate stayed quiet. 353 GUI cells; schema_mirror green with `--variant` on both subcommands.
- **Tier:** `v0.32+-gui-lockstep`
- **Tags:** none
- **Companion:** parent `seedqr-compact-variant` (resolved v0.32.0).


### `seedqr-15-18-21-word-counts` — extend SeedQR encode/decode to 15/18/21-word phrases

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.30.0 Cycle 5 brainstorm close. Deferred from Cycle 5 per the word-count-scope lock.
- **Where:** `crates/mnemonic-toolkit/src/seedqr.rs` (`SeedqrError::InvalidWordCount` matches `12|24` only at v0.30.0; widen to `12|15|18|21|24`); `crates/mnemonic-toolkit/tests/cli_seedqr.rs` (the 5 wrong-word-count refusal cells exercise 13/15/18/21/25 — 15/18/21 must move from refusal to happy-path on this FOLLOWUP).
- **What:** Widen SeedQR encode/decode word-count support from `{12, 24}` to `{12, 15, 18, 21, 24}`. SeedSigner's SeedQR spec is explicit about 12+24; 15/18/21 are BIP-39 standard but not in the original SeedQR specification. Validate against SeedSigner's reference implementation: if their encoder accepts these word-counts cleanly, ship; if it doesn't, file a spec-clarification.
- **Why deferred:** Cycle 5 locked 12+24 only per the SeedQR spec's explicit canonical word-counts. Widening to all BIP-39 word-counts requires (a) verification that the SeedSigner ref impl accepts them; (b) extending the unit + integration test matrix; (c) JSON envelope's `word_count` field already supports arbitrary values so no envelope evolution needed.
- **Status:** `resolved 76fdc6c` — mnemonic-toolkit-v0.31.5 Cycle 12. Two gates at `crates/mnemonic-toolkit/src/seedqr.rs` widened: `decode` digit-length from `48 | 96` → `matches!(len, 48 | 60 | 72 | 84 | 96)`; `encode` word-count from `12 | 24` → `matches!(words.len(), 12 | 15 | 18 | 21 | 24)`. Error texts updated. SeedSigner spec body documents 12+24 explicitly; the encoding format itself is word-count-agnostic (4 decimal digits per BIP-39 word index). 13 new test cells (9 lib happy-path + 1 lib boundary refusal `encode_rejects_22_word_count` + 3 CLI flips + 1 CLI JSON-envelope cell). Canonical Trezor zero-entropy vectors derived empirically via `mnemonic convert --from entropy=<20/24/28-byte-zeros>`: 15-word `abandon ×14 + address` (BIP-39 index 27); 18-word `abandon ×17 + agent` (index 39); 21-word `abandon ×20 + admit` (index 29). End-of-cycle opus cross-verified vector accuracy against the BIP-39 English wordlist. 2162 cells passing. End-of-cycle opus GREEN.
- **Tier:** `v0.30+`
- **Tags:** none
- **Companion:** parent `seedqr-encode-decode-subcommand` (resolved v0.30.0).


### `seedqr-bundle-slot-integration` — `mnemonic bundle --slot @N.seedqr=<file>` auto-decode at slot-emit

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.30.0 Cycle 5 brainstorm close.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs` slot-input pipeline + `crates/mnemonic-toolkit/src/slot_input.rs` (slot value-source variants).
- **What:** Add a new slot input source `--slot @N.seedqr=<file>` that reads a SeedQR file at slot-emit time and decodes it inline (via the `seedqr::decode` library primitive) into a BIP-39 phrase, then feeds the phrase into the normal slot machinery. Tightens the integration between SeedQR and bundle emission without forcing a two-step `mnemonic seedqr decode | mnemonic bundle --slot @N.phrase=-` shell pipeline.
- **Why deferred:** Cycle 5 locked the seedqr surface as a standalone subcommand (paralleling seed-xor/slip39/final-word). Bundle-slot integration is a separate cross-subcommand wiring decision; user-direction may prefer the standalone form indefinitely.
- **Status:** `resolved b08645b` — mnemonic-toolkit-v0.31.3 Cycle 10. New `SlotSubkey::Seedqr` variant declared at enum position 1 (after Phrase, before Entropy) so derived `Ord` produces ascending-sorted legal-set patterns `[Seedqr]`, `[Seedqr, Path]`, `[Seedqr, Fingerprint, Path]` mirroring the v0.19.0 SPEC §6.6.b exception for Phrase. `--slot @N.seedqr=<digit-string>` is now accepted on `mnemonic bundle` + `mnemonic verify-bundle` (refused on `mnemonic export-wallet` per the SPEC §3 watch-only-by-definition invariant). Value is decoded inline via `mnemonic_toolkit::seedqr::decode` at slot-emit time; resulting phrase materializes through the same `derive_full` + `ResolvedSlot` path as `--slot @N.phrase=`. `cmd/seedqr.rs::map_seedqr_error` promoted to `pub(crate)` for canonical error-text reuse across the 3 consumer sites. 15 new cells (9 integration + 6 lib unit); 2150 cells passing total. End-of-cycle opus review GREEN 0C/0I/0M. SemVer-PATCH per the GUI schema_mirror gate scope clarification (flag-NAME parity gate doesn't fire on value-content additions). 1 follow-on FOLLOWUP filed: `gui-seedqr-slot-subkey-help-mirror`.
- **Tier:** `v0.30+`
- **Tags:** none
- **Companion:** parent `seedqr-encode-decode-subcommand` (resolved v0.30.0); follow-on `gui-seedqr-slot-subkey-help-mirror` (optional GUI help-text mirror; non-blocking).


### `gui-seedqr-slot-subkey-help-mirror` — optional mnemonic-gui mirror of the v0.31.3 seedqr slot subkey

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.31.3 Cycle 10 close. Cycle 10 R0 I1 fold pivoted the SemVer from MINOR to PATCH after verifying that the GUI `schema_mirror` integration test compares clap flag-NAME parity, NOT value-enumeration content. A new `--slot` value-enumeration token (`seedqr`) therefore does NOT auto-fire the drift gate. Filing this entry as the load-bearing tracker for the optional GUI-side help-text + dropdown surface update.
- **Where:** `mnemonic-gui/src/schema/mnemonic.rs` (slot-subkey help-text + dropdown enumerations if any); `mnemonic-gui/pinned-upstream.toml` toolkit pin bump to `mnemonic-toolkit-v0.31.3`.
- **What:** Update the GUI schema-mirror's `--slot` help-text to enumerate `seedqr` + extend any dropdown surfaces that expose the slot-subkey list. Bump the toolkit pin so GUI users can invoke `--slot @N.seedqr=<digit-string>` via the GUI's input controls. Bump GUI version to `v0.16.1` (PATCH; GUI-internal help-text + pin bump only — no schema-mirror gate violation).
- **Why deferred:** The schema_mirror gate compares clap flag-NAME parity, not value-enumeration content; the new `seedqr` token does NOT fire the gate. GUI help/dropdown improvement is desirable for discoverability but not blocking.
- **Status:** `resolved 4c1dde5` — mnemonic-gui-v0.16.1 (Cycle 10b). Toolkit pin `mnemonic-toolkit-v0.31.0 → v0.31.3` (cumulative catch-up across v0.31.1 + v0.31.2 + v0.31.3); `src/form/slot_editor.rs::SlotSubkey::Seedqr` variant added at enum position 1 (mirrors toolkit enum-position correctness); supply-chain drift snapshot at `src/secrets.rs::v0_3_canonical_fallback::SECRET_SLOT_SUBKEYS` extended to `["phrase", "seedqr", "entropy", "xprv", "wif"]` (compile-time drift gate fired as designed; acknowledged by snapshot update); `tests/secrets.rs::secret_slot_subkeys_set_pinned` expectation updated. 353 cells passing.
- **Tier:** `v0.32+-gui-help-only`
- **Tags:** none
- **Companion:** parent `seedqr-bundle-slot-integration` (resolved v0.31.3).


### `seedqr-digits-from-input-unification` — extend `FromInput` with `seedqr=<value>` and deprecate `--digits`

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.30.0 Cycle 5 plan-doc R0 opus review (`design/agent-reports/v0_30_0-plan-doc-r0-review.md` §I4).
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` `FromInput` + `NodeType` enum + `parse_from_input`; `crates/mnemonic-toolkit/src/cmd/seedqr.rs::SeedqrDecodeArgs.digits` (current `--digits` flag).
- **What:** Long-term surface unification across all `--from`-shaped subcommands. Today: `convert` uses `--from <node>=<value>`; `seed-xor` / `slip39` / `seedqr-encode` use `--from phrase=...`; but `seedqr-decode` uses a bespoke `--digits <value>` flag because SeedQR digits are a distinct surface from the existing `phrase/xpub/xprv/ms1/...` types. The asymmetry creates a long-term inconsistency. **Proposed:** extend `FromInput` with a `seedqr=<value>` node type, then deprecate `--digits` in favor of `--from seedqr=...`. Migration path: v0.30+ accepts BOTH `--digits` (deprecated; emits stderr warning) AND `--from seedqr=...` (canonical); a future v0.31+ removes `--digits`.
- **Why deferred:** Cycle 5 scope was the standalone seedqr surface. Extending `FromInput` to include `seedqr=` would have been a global change touching every consumer of `FromInput`; out of scope for the introductory cycle.
- **Status:** `resolved 5f0b7b4` — mnemonic-toolkit-v0.31.6 Cycle 13. Added `NodeType::Seedqr` (enum position 1) wired as a first-class input node through `classify_edge` + `is_supported_direct_edge` + `compute_outputs`. `mnemonic convert --from seedqr=<digits> --to <node>` end-to-end (Option 3). `mnemonic seedqr decode --from seedqr=` canonical; `--digits` deprecated (stderr notice + clap `conflicts_with`, exit 64). **Design note:** the FOLLOWUP-body migration ("v0.31+ removes `--digits`") + the R0 substitute-to-Phrase approach were BOTH superseded — `--digits` is kept as a deprecated alias (not removed) and Seedqr is wired natively (substitution would have collapsed the `(Seedqr, Phrase)` decode into the `(Phrase, Phrase)` identity barrier). 12 integration cells; 2174 total. End-of-cycle opus GREEN. `--digits` removal tracked as a future cycle (no slug filed yet — revisit after one deprecation-window release).
- **Tier:** `v0.30+`
- **Tags:** none
- **Companion:** parent `seedqr-encode-decode-subcommand` (resolved v0.30.0); follow-on `gui-seedqr-decode-from-flag-mirror` (GUI v0.16.2 schema_mirror lockstep — `--from` net-new flag on seedqr-decode).

### `gui-seedqr-decode-from-flag-mirror` — mnemonic-gui schema mirror for the v0.31.6 seedqr-decode --from flag

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.31.6 Cycle 13 close. Unlike the value-content additions of Cycles 10/12 (which the schema_mirror gate ignores), v0.31.6 adds a NET-NEW flag NAME (`--from`) to `mnemonic seedqr decode` — this DOES trip the GUI `schema_mirror` flag-NAME-parity gate.
- **Where:** `mnemonic-gui/src/schema/mnemonic.rs` (`seedqr-decode` SubcommandSchema flag list); `mnemonic-gui/pinned-upstream.toml` + `Cargo.toml` toolkit pin → `mnemonic-toolkit-v0.31.6`.
- **What:** Add `--from` to the `seedqr-decode` schema entry (and document `--digits` as deprecated-but-present); bump toolkit pin v0.31.3 → v0.31.6 (cumulative catch-up across v0.31.4/v0.31.5/v0.31.6). The supply-chain drift gate at `src/secrets.rs::v0_3_canonical_fallback::SECRET_NODE_TYPES` will fire on the pin bump (v0.31.6 added `"seedqr"` to `SECRET_NODE_TYPES`) — acknowledge via snapshot update. GUI v0.16.2 (PATCH).
- **Why deferred:** Cross-repo; ships as Cycle 13b immediately following the toolkit tag (lockstep).
- **Status:** `resolved 0c55cfd` — mnemonic-gui-v0.16.2 (Cycle 13b). Added `--from` to `SEEDQR_DECODE_FLAGS` (`NodeValueComposite(["seedqr"])`) + `--digits` → `required: false` (deprecated). Toolkit pin v0.31.3 → v0.31.6 (cumulative catch-up across v0.31.4 + v0.31.5 + v0.31.6). `SECRET_NODE_TYPES` supply-chain drift snapshot + `tests/secrets.rs` pinned-set both += `"seedqr"` (compile-time drift gate fired on pin bump; acknowledged). 353 GUI cells passing; schema_mirror green. Known gap deferred (no FOLLOWUP): `convert` GUI form shares one `NODE_TYPES` const for `--from`/`--to` dropdowns; `seedqr` (input-only) not added to avoid wrongly offering `--to seedqr`; not gate-affecting.
- **Tier:** `v0.32+-gui-lockstep`
- **Tags:** none
- **Companion:** parent `seedqr-digits-from-input-unification` (resolved v0.31.6).


### `electrum-crypto-seed-extraction-subcommand` — future use of v0.30.1's electrum_crypto library for seed extraction

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.30.1 Cycle 6b close (the Path A R0 fold made the 6a-shipped library unused-by-CLI).
- **Where:** `crates/mnemonic-toolkit/src/electrum_crypto.rs` (library shipped Cycle 6a `1724477`; 18 unit cells + cross-impl smoke vs Python `cryptography` backend). Currently referenced by no CLI module.
- **What:** Surface a new CLI consumer for the `electrum_crypto::decrypt_field` primitive. Two candidate shapes: (a) extend `mnemonic convert` with a new `--from electrum-encrypted-seed=<base64>` source; (b) dedicated `mnemonic electrum-decrypt` subcommand taking an encrypted seed string + password and emitting the plaintext seed. The library's `derive_key` + `decrypt_field` + `encrypt_field` (symmetric helper) are all production-ready; only the CLI integration is missing.
- **Why deferred:** Cycle 6 was reinterpreted as watch-only-passthrough (no decryption needed for import-wallet per opus R0). The library is correct but has no user-visible surface yet. A future cycle ships the consumer when the seed-extraction use case is prioritized.
- **Status:** `resolved fa71e77` — mnemonic-toolkit-v0.33.0 Cycle 18. Chose candidate shape (b): dedicated `mnemonic electrum-decrypt` subcommand (NOT a `convert` source — architect-locked Option A: the decrypted node-type (phrase vs xprv) is unknowable pre-decryption, which `convert`'s commit-types-up-front model cannot express; also collides with the `(Phrase, ElectrumPhrase)` artifact-class refusal). Surface: `--ciphertext <VALUE|->` + 3-form password group (struct-level `ArgGroup` exactly-one-required+exclusive: `--decrypt-password <VAL>` / `-file` / `-stdin`) + `--json-out <PATH>` ({schema_version, operation, plaintext}; no password echo). Secret hygiene: password + plaintext `Zeroizing` + `mlock`-pinned; inline-pw argv advisory + plaintext-on-stdout advisory + json-out world-readable advisory. Format A carries no MAC → wrong-pw (PKCS7-unpad refusal) + non-UTF-8 unified into one "decryption failed" message (no failure-mode leak). New `secret_advisory::secret_on_stdout_warning_unconditional` (Ms1-gated helper delegates). 12 integration cells; 2221 total (+12). Plan-doc opus R0 YELLOW 0C/3I (folded); end-of-cycle opus GREEN. First of the final v0.32+ Electrum pair; `wallet-import-electrum-encrypted-storage-format-b` (Format B whole-file) remains and reuses this `--decrypt-password*` surface + sha256d crypto.
- **Tier:** `v0.31+`
- **Tags:** `wallet`
- **Companion:** parent `wallet-import-electrum-encrypted` (resolved v0.30.1 as watch-only-passthrough); follow-on `gui-electrum-decrypt-subcommand-mirror` (mandatory GUI v0.18.0 lockstep — NEW subcommand).


### `gui-electrum-decrypt-subcommand-mirror` — GUI SubcommandSchema for `mnemonic electrum-decrypt`

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.33.0 Cycle 18 close. The toolkit added the NEW `electrum-decrypt` subcommand (gui-schema now lists 21 subcommands). The GUI `schema_mirror` gate compares the full subcommand list + per-subcommand flag-NAME set, so a NEW subcommand is a HARD gate trip — GUI lockstep is MANDATORY.
- **Where:** `mnemonic-gui/src/schema/mnemonic.rs` (new `ELECTRUM_DECRYPT_FLAGS` SubcommandSchema + registration in the subcommand list); `mnemonic-gui/pinned-upstream.toml` + `Cargo.toml` toolkit pin → `v0.33.0`.
- **What:** Add the `electrum-decrypt` SubcommandSchema mirroring the clap surface: `--ciphertext` (text/path), `--decrypt-password` (text), `--decrypt-password-file` (path), `--decrypt-password-stdin` (boolean), `--json-out` (path), plus the global `--no-auto-repair`. Register in the subcommand list (alphabetically after `derive-child`). Bump the toolkit pin → v0.33.0. GUI v0.18.0 (MINOR — new subcommand surface).
- **Why deferred:** Cross-repo authoring; shipped as the paired Cycle 18b GUI release immediately after the toolkit tag.
- **Status:** `resolved mnemonic-gui d5ec089` — mnemonic-gui-v0.18.0 Cycle 18b. Added `ELECTRUM_DECRYPT_FLAGS` SubcommandSchema + `electrum-decrypt` registration (after `derive-child`); toolkit pin v0.32.0 → v0.33.1. **Surfaced + fixed a secret-classification gap mid-lockstep:** the toolkit v0.33.0 `gui-schema` emitted NO secret flags for electrum-decrypt (the CLI fires `secret_in_argv_warning` for `--decrypt-password`, but `secrets::flag_is_secret` omitted it), so the `schema_mirror_secret_drift` gate would have rejected a secure GUI mirror. Fixed in toolkit **v0.33.1** (`4bd9053`: added `--decrypt-password` + `--decrypt-password-stdin` to `flag_is_secret`); GUI then pinned v0.33.1 + mirrored `secret: true` on the two inline password forms (`--decrypt-password-file` = path, non-secret; `--ciphertext` = encrypted material, non-secret). 353 GUI cells; `schema_mirror` (flag-name parity incl. electrum-decrypt) + `schema_mirror_secret_drift` both green vs the pinned v0.33.1 binary.
- **Tier:** `v0.33+-gui-lockstep`
- **Tags:** none
- **Companion:** parent `electrum-crypto-seed-extraction-subcommand` (resolved v0.33.0; secret-classification fix v0.33.1).


### `wallet-import-electrum-encrypted-storage-format-b` — Electrum whole-file storage encryption (ECIES BIE1)

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.30.1 Cycle 6b close (Cycle 6 P0 recon §A2 distinguished Format A field-level encryption (in-scope, resolved as watch-only-passthrough) from "Format B whole-file storage encryption" (out-of-scope)).
- **CRYPTO RE-IDENTIFIED — 2026-05-21 (Cycle 19 P0 recon, verified against `github.com/spesmilo/electrum` `electrum/crypto.py` + `electrum/storage.py`, fetched HEAD).** The prior body (and Cycle-6 recon §A2) **misidentified the scheme**. Whole-file Electrum wallet storage encryption is **ECIES**, NOT the `version_byte ‖ iv ‖ aes-cbc ‖ sha256(pt)[:4]` scheme previously written. That `pw_encode_with_version_and_mac` scheme is real but is Electrum's **Lightning** helper (`lnworker.py` / `lnutil.py` / `qewallet.py`), unrelated to wallet storage. The `sha256d(password)` key claim was also wrong for storage. This is the SECOND crypto-citation error in this followup family (the original parent body claimed PBKDF2 for Format A, corrected to sha256d at Cycle 6 P0) — re-grep primary source before any plan.
- **Where:** `crates/mnemonic-toolkit/src/wallet_import/electrum.rs` (sniff + parse pipeline). A storage-encrypted wallet file is NOT JSON — it is a single base64 blob whose decoded first 4 bytes are the magic `BIE1` (user-password) or `BIE2` (hardware-device/xpub). The current `ElectrumParser::parse` fails at JSON-parse with an unrelated error. `electrum/storage.py::StorageEncryptionVersion` = `{PLAINTEXT=0, USER_PASSWORD=1, XPUB_PASSWORD=2}`; magic dispatch at `storage.py:182-188`.
- **What (verified BIE1 / USER_PASSWORD decrypt — the only password-decryptable mode):**
  1. base64-decode the whole file; require len ≥ 85 and magic `blob[:4] == b"BIE1"`.
  2. EC privkey scalar = `int.from_bytes(PBKDF2-HMAC-SHA512(password, salt=b"", iterations=1024, dklen=64)) mod secp256k1_order` (Electrum `storage.py:193-197` `get_eckey_from_password` → `ECPrivkey.from_arbitrary_size_secret`). **NOT `sha256d`.**
  3. `ecdh_point = (ephemeral_pubkey × privkey_scalar)` compressed (33B), where `ephemeral_pubkey = blob[4:37]`; `key = sha512(ecdh_point)`; `iv, key_e, key_m = key[0:16], key[16:32], key[32:64]` (`crypto.py:ecies_decrypt_message`, magic `BIE1`).
  4. verify `HMAC-SHA256(key_m, blob[:-32]) == blob[-32:]` (wrong-password detector).
  5. `compressed = AES-128-CBC-decrypt(key_e, iv, blob[37:-32])` (PKCS7) — note **AES-128**, the codebase currently only uses AES-256.
  6. `wallet_json = zlib.decompress(compressed)` — then feed into the existing Electrum JSON parser.
- **Effort / deps (NOT a parser-only cycle):** real ECIES build, ~Cycle-7-BIP129-encryption-sized or larger. NEW dep `flate2` (zlib `decompress`); secp256k1 point-multiplication via the existing `bitcoin`/`secp256k1` dep PLUS a 512-bit-mod-n scalar reduction (PBKDF2 output is 64 bytes; `SecretKey::from_slice` wants a valid 32-byte scalar — needs a bignum mod-n, e.g. `num-bigint` or manual reduction); `aes::Aes128` (new alongside the existing `Aes256`); `sha2::Sha512`; `hmac` (all present). Reusing `decrypt_field` is NOT possible (it is AES-256-CBC with an `sha256d` key; ECIES uses AES-128-CBC with an ECDH/sha512-derived key + ephemeral pubkey + 32-byte HMAC + zlib).
- **LIBRARY RECON (2026-05-21, Cycle 19):** no usable pre-built crate. (a) NO standalone "Electrum/BIE1 ECIES" Rust crate exists; Electrum is Python and publishes none. (b) The BIE1 scheme exists in Rust ONLY inside **BSV-blockchain SDKs** (`bsv-wasm` MIT, `bsv-rs` MIT/Apache, `bsv-sdk` non-standard-license) — BSV inherited bitcore-ecies, which is the same scheme. `Firaenix/bsv-wasm src/ecies/mod.rs` was verified byte-identical (`b"BIE1"` magic + `sha512(compressed(pubkey×privkey))`→`iv/ke/km = [0:16]/[16:32]/[32:64]` + AES-128-CBC + HMAC-SHA256). **User decision (Cycle 19): IGNORE BSV** — do not pull a BSV-branded SDK (k256/wasm-bindgen baggage; poor fit + supply-chain surface for a Bitcoin self-custody tool). (c) The popular `ecies` crate is a DIFFERENT scheme (HKDF-SHA256 + AES-256-GCM) and will NOT interoperate with Electrum. (d) NO full-pipeline Rust Electrum-wallet-file reader exists (storage decrypt + PBKDF2 + zlib). **Conclusion:** hand-roll the ~40–50-line BIE1 storage decrypt from the verified Electrum source above, reusing the focused primitives already in-tree (`bitcoin`/secp256k1 point-mul, `sha2`, `aes::Aes128`+`cbc`, `hmac`, `pbkdf2`) + new `flate2` (zlib). Even a "library" would only cover the ECIES envelope, not the storage-specific PBKDF2-SHA512→mod-n password key + zlib + magic dispatch. Use Electrum (Python) and/or bitcore-ecies (JS) as the cross-impl test-vector oracle (Cycle 6a/Cycle 7 vendored-fixture pattern), NOT as a runtime dep.
- **CLI surface:** the v0.33.0 `--decrypt-password*` family lives on the NEW `electrum-decrypt` subcommand, NOT on `import-wallet`. Wiring storage-decrypt into `import-wallet` adds those flags to `import-wallet` (net-new flag NAMEs on that subcommand → MANDATORY GUI schema-mirror lockstep) OR routes via a new dedicated path. So the "password infra already exists" framing is only half-true.
- **Out of scope — BIE2 / XPUB_PASSWORD (hardware-device):** cannot be decrypted from a password at all; the EC privkey is the wallet's own master key held by the hardware device. A password-only toolkit has no decrypt path. Carve out as a distinct followup (or document as permanently unsupported) at implementation time.
- **Why deferred:** Re-scoped 2026-05-21 (Cycle 19): the queue treated this as a small "sniff + reuse decrypt_field" cycle, but P0 recon showed it is a full ECIES implementation. User decision (Cycle 19): **correct the followup + defer** — do NOT build on the corrected (larger) premise without a dedicated brainstorm/sizing pass.
- **Status:** `resolved` — Cycle 19, two phases. **Phase A** `a62cf15` (mnemonic-toolkit master): `electrum_crypto.rs` ECIES BIE1 library (`derive_storage_eckey` PBKDF2-SHA512→mod-n via `crypto-bigint`; `ecies_decrypt_message`; `ecies_decrypt_storage` + `flate2` zlib; `EciesDecryptError`), verified byte-exact against Electrum's OWN committed `test_decrypt_message` KATs (pw123 + 3 BIE1 blobs) + a Python-stdlib zlib oracle. **Phase B** `mnemonic-toolkit-v0.33.2` (Cycle 19 ship): `import-wallet --decrypt-password{,-file,-stdin}` (optional exclusive ArgGroup) + `detect_storage_magic` + orchestrator decrypt-before-sniff (mirrors BSMS decrypt-then-parse) + BIE2 refusal + 3-way stdin guard + manual. SemVer PATCH (net-new flags on existing subcommand; Cycle-13 `--from` precedent). 2253 cells (+16); independent pure-Python `ecdsa` fixture (cross-impl witness of the zlib→ECIES framing). Plan-doc opus R0 (Phase A YELLOW→R1 GREEN; Phase B YELLOW→R1 GREEN); end-of-cycle opus GREEN. **BIE2 carve-out:** detected + refused (no password decrypt path; permanently unsupported). New FOLLOWUPs filed: `import-wallet-blob-zeroizing` (hygiene) + `gui-import-wallet-decrypt-password-mirror` (GUI lockstep).
- **Tier:** `v0.31+`
- **Tags:** `wallet`
- **Companion:** parent `wallet-import-electrum-encrypted` (Format A resolved v0.30.1 as watch-only-passthrough; this is the storage-encryption carve-out); sibling `electrum-crypto-seed-extraction-subcommand` (resolved v0.33.0).


### `import-wallet-blob-zeroizing` — scrub the decrypted/plaintext wallet blob `Vec<u8>`

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.33.2 Cycle 19 Phase B (end-of-cycle opus I1).
- **Where:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — `read_blob` returns a plain `Vec<u8>`; the BIE1 decrypt path does `blob = decrypted.to_vec()`, dropping the `Zeroizing<Vec<u8>>` wrapper that `ecies_decrypt_storage` returns.
- **What:** the import-wallet `blob: Vec<u8>` can hold secret material — a plaintext Electrum wallet with `use_encryption:false` already carries a seed in that Vec for ALL formats, and v0.33.2 newly writes *decrypted* BIE1 wallet JSON (which may carry seed/xprv) into it. The bytes are `mlock`-pinned (no swap) but never scrubbed before drop. Migrate the blob to a zeroizing field type (mirror `resolved-slot-derived-account-zeroizing-field`), scoped to the shared field so all 8 import paths benefit.
- **Why deferred:** pre-existing property of the import path (not introduced by, but made slightly more load-bearing by, the BIE1 decrypt); the import OUTPUT is watch-only (non-secret); mlock-pin is in place. A field-type migration is its own focused change.
- **Status:** `resolved` — `mnemonic-toolkit-v0.33.3`. `read_blob` → `Result<Zeroizing<Vec<u8>>>`; `blob` binding → `Zeroizing<Vec<u8>>`; BIE1 reassign `blob = plaintext.to_vec()` → `blob = plaintext;` (preserves the `ecies_decrypt_storage` `Zeroizing` wrapper, drops the lossy clone); BSMS Round-2 reassign re-wraps via `Zeroizing::new(into_bytes())`. All ~12 read sites compiled unchanged via `Zeroizing<Vec<u8>> → Vec<u8> → [u8]` deref coercion (no `&blob[..]` fixups). Type-only; 2253 cells unchanged. Plan opus R0 GREEN 0C/0I/3M → end-of-cycle GREEN 0C/0I/0M. SemVer PATCH; no GUI/manual/schema-mirror surface. Cited the `resolved-slot-derived-account-zeroizing-field` precedent.
- **Tier:** `v0.33+`
- **Tags:** `wallet`
- **Companion:** parent `wallet-import-electrum-encrypted-storage-format-b` (resolved v0.33.2). Follow-ons filed at close: `bsms-decrypt-record-string-zeroizing` + `import-wallet-plaintext-blob-mlock-pin`.


### `bsms-decrypt-record-string-zeroizing` — wrap `decrypt_bsms_record`'s plaintext `String` in `Zeroizing`

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.33.3 close (blob-zeroizing R0 M1).
- **Where:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` `decrypt_bsms_record` (returns a plain `String`); consumed at the BSMS Round-2 reassign (`~:1043`) + the Round-1 decrypt path.
- **What:** v0.33.3 scrubs the `blob` copy (the re-wrapped `Zeroizing::new(into_bytes())`), but the intermediate decrypted `String` returned by `decrypt_bsms_record` is itself un-zeroized before that wrap. Migrate the return type to `Zeroizing<String>`. Low sensitivity (the BSMS Round-2 plaintext is a watch-only descriptor, not seed/xprv), hence not folded into v0.33.3.
- **Why deferred:** out of scope for the focused `blob`-binding migration; separate function-signature change with its own (minor) call-site churn.
- **Status:** `resolved` — mnemonic-toolkit-v0.34.1. `decrypt_bsms_record` return type → `Result<Zeroizing<String>, ToolkitError>` (wrapped at the `String::from_utf8` site); Round-2 consumer `into_bytes()` → `as_bytes().to_vec()` (cannot move out of `Zeroizing`); Round-1 `else`-arm wraps `raw_text` so the `if/else` unifies to `Zeroizing<String>` (`parse_round1(&str)` via deref). Type-only; 66 BSMS cells unchanged; opus plan R0→R1 GREEN.
- **Tier:** `v0.33+`
- **Tags:** `wallet`
- **Companion:** sibling `import-wallet-blob-zeroizing` (resolved v0.33.3).


### `import-wallet-plaintext-blob-mlock-pin` — mlock-pin the non-BIE1 wallet blob

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.33.3 close (blob-zeroizing R0 M2).
- **Where:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` `run()` — only the BIE1 decrypt branch calls `mlock::pin_pages_for(&blob)`; the plaintext-import path (`use_encryption:false` Electrum wallet, seed-bearing) + all other formats never pin the blob.
- **What:** the blob is now `Zeroizing` (scrubbed on drop, v0.33.3) but a plaintext seed-bearing wallet Vec sits swappable (un-`mlock`-pinned) for the lifetime of `run()`. Pin `&blob` after `read_blob` for ALL formats (not just BIE1). Orthogonal to zeroize-on-drop. NOTE the existing BIE1 pin is itself arm-scoped (dropped at the end of the decrypt arm) — a holistic fix should pin once at the `blob` binding for the whole `run()` scope.
- **Why deferred:** orthogonal to the zeroize-on-drop migration; a pin-lifetime redesign (pin-at-binding) is its own change.
- **Status:** `resolved` — mnemonic-toolkit-v0.34.1. A single `let mut _pin_blob` guard is pinned at the `blob` binding (covers ALL formats incl. the plaintext seed-bearing path) and re-pinned via `drop(std::mem::replace(&mut _pin_blob, pin_pages_for(&blob)))` at the BIE1 + Round-2 reassigns — so exactly one live guard pins the current buffer (the prior arm-local `_pin_pt` is replaced). R0 caught the stale-`munlock` hazard of a parallel run-scoped guard (mlock locks don't stack); the re-pinned-single-guard avoids it. 94 import-wallet cells unchanged; opus plan R0→R1 GREEN.
- **Tier:** `v0.33+`
- **Tags:** `wallet`
- **Companion:** sibling `import-wallet-blob-zeroizing` (resolved v0.33.3).


### `gui-import-wallet-decrypt-password-mirror` — GUI import-wallet FlagSchema for `--decrypt-password*`

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.33.2 Cycle 19 Phase B close. NEW flag NAMEs (`--decrypt-password`, `--decrypt-password-file`, `--decrypt-password-stdin`) on `import-wallet` → the GUI `schema_mirror` flag-NAME-parity gate trips. MANDATORY lockstep.
- **Where:** `mnemonic-gui/src/schema/mnemonic.rs` (`import-wallet` SubcommandSchema); `pinned-upstream.toml` + `Cargo.toml` toolkit pin → v0.33.2.
- **What:** add the three flags to the import-wallet FlagSchema: `--decrypt-password` (Text, **secret**), `--decrypt-password-file` (Path, non-secret), `--decrypt-password-stdin` (Boolean, **secret**). The GUI `secrets::flag_is_secret` mirror already covers these names (v0.33.1 / GUI v0.18.0 lockstep) — `schema_mirror_secret_drift` stays green; confirm. Bump pin → v0.33.2. GUI v0.18.1 (PATCH).
- **Why deferred:** cross-repo authoring; shipped as the paired Cycle-19-Phase-B GUI release immediately after the toolkit tag.
- **Status:** `resolved mnemonic-gui 655e8f5` — mnemonic-gui-v0.18.1. Added the three `--decrypt-password*` flags to the import-wallet SubcommandSchema (`secret:true` on `--decrypt-password` + `--decrypt-password-stdin`; `--decrypt-password-file` non-secret); toolkit pin v0.33.1 → v0.33.2. `schema_mirror` (flag-name parity incl. the new flags) + `schema_mirror_secret_drift` (the secret projection on import-wallet) both green vs the pinned binary. 353 GUI cells.
- **Tier:** `v0.33+-gui-lockstep`
- **Tags:** none
- **Companion:** parent `wallet-import-electrum-encrypted-storage-format-b` (resolved v0.33.2).


### `bsms-encryption-per-signer-tokens` — per-Signer BIP-129 TOKEN variants

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.31.0 Cycle 7b close. Per BIP-129 line 74: "Depending on the use case, the Coordinator can decide whether to share one common TOKEN for all Signers, or to have one per Signer." Cycle 7b ships SHARED-TOKEN mode only (single `--bsms-encryption-token <FILE|->`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (orchestrator decrypt block; currently consumes one token regardless of how many `--bsms-round1` records are also supplied).
- **What:** v0.31+: extend the orchestrator decrypt to accept per-Signer tokens (e.g., `--bsms-encryption-token <FILE>` becomes repeatable + each invocation pairs with a `--bsms-round1 <FILE>` record). Required API: change the flag from `Option<PathBuf>` to `Vec<PathBuf>` (CLI break — careful with the existing single-flag consumers); add per-record token-to-Signer pairing logic.
- **Why deferred:** Cycle 7 scope was BIP-129 §Encryption MVP (shared TOKEN). Per-Signer variants are a separate orchestration with their own UX design (how does the user know which token goes with which Signer's record?). Worth its own cycle.
- **Status:** `resolved 72a55f1` — mnemonic-toolkit-v0.32.2 Cycle 16. `--bsms-encryption-token` → `Vec<PathBuf>` (clap auto-Append). Pairing: 1 token = SHARED (decrypts all encrypted Round-1 records + Round-2 blob; backward-compatible byte-identical); N>1 = PER-SIGNER positional (token[i] ↔ --bsms-round1 record[i]). Edge guards: gap-h (N>1 + 0 records → BadInput), multi-token + encrypted-Round-2-blob → BadInput (single share = single token), mixed plaintext/encrypted under N>1 → BadInput, single-stdin-token guard generalized. `verify_bsms_round1_files(tokens: &[BsmsToken])` with positional pre-checks (count + all-encrypted) + per-record token selection. 8 integration cells; 2206 total. SemVer-PATCH (purely additive — no break; the FOLLOWUP-body "CLI break" framing was loose). End-of-cycle opus GREEN 10/10 (per-token MAC-verify isolation audited).
- **Tier:** `v0.31+`
- **Tags:** `wallet`
- **Companion:** parent `bsms-bip129-encryption-envelope` (shared-TOKEN resolved v0.31.0); follow-on `gui-bsms-encryption-token-repeating-mirror` (optional GUI v0.17.1).

### `gui-bsms-encryption-token-repeating-mirror` — GUI repeating flip for the v0.32.2 per-Signer token

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.32.2 Cycle 16 close. The toolkit made `--bsms-encryption-token` repeatable; the GUI's `FlagSchema` for it is `repeating: false` (`src/schema/mnemonic.rs:1736`). The schema_mirror gate compares flag-NAME parity only (the flag name is unchanged), so this does NOT auto-fire — GUI lockstep is OPTIONAL.
- **Where:** `mnemonic-gui/src/schema/mnemonic.rs` (`--bsms-encryption-token` FlagSchema `repeating` field); `mnemonic-gui/pinned-upstream.toml` + `Cargo.toml` toolkit pin → `v0.32.2`.
- **What:** Flip `repeating: false → true` so the GUI's flag-repeat UI can add multiple `--bsms-encryption-token` rows for per-Signer mode; bump the toolkit pin. GUI v0.17.1 (PATCH; GUI-functional, not gate-forced).
- **Why deferred:** Non-blocking (gate not tripped). Bundle with the next GUI lockstep OR ship standalone when convenient.
- **Status:** `open`
- **Tier:** `v0.32+-gui-help-only`
- **Tags:** none
- **Companion:** parent `bsms-encryption-per-signer-tokens` (resolved v0.32.2).


### `bsms-encryption-round1-decrypt-then-verify` — encrypted Round-1 KEY records

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.31.0 Cycle 7b close. BIP-129 §Round 1 Signer specifies encrypted Round-1 KEY records (5-line shape: `BSMS 1.0\nhex-TOKEN\nKEY\ndescription\nbase64-SIG`) which the Coordinator receives + decrypts + verifies via the per-Signer Round-1 BIP-322 signature verify path (already shipped v0.27.0 via `--bsms-round1`). Cycle 7b's orchestrator decrypts an encrypted Round-2 wire but does NOT integrate decrypt-then-verify with the `--bsms-round1` flow.
- **Where:**
  - `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` orchestrator: `--bsms-round1 <FILE>` flow (plaintext-only) + the new `--bsms-encryption-token <FILE|->` decrypt are currently separate paths.
  - `crates/mnemonic-toolkit/src/wallet_import/bsms_round1.rs` + `bsms_verify.rs` — Round-1 parser + BIP-322 verify (plaintext-only).
  - `design/agent-reports/v0_27_0-phase-2-bip129-recon.md` §"Test Vectors" — TV-3 + TV-4 contain encrypted Round-1 records with full cross-validated values.
  - `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms_encrypted.rs::tv3_decrypt_emits_notice_advisory` — currently exercises the decrypt-success-then-parse-refusal boundary (TV-3's 5-line plaintext fails the Round-2-only `BsmsParser`).
- **What:** v0.31+: extend the orchestrator to detect encrypted Round-1 records (the user supplies `--bsms-round1 <FILE>` containing a hex `MAC||ciphertext` blob), decrypt via `bsms_crypto::decrypt` + verify MAC, then dispatch to the existing Round-1 BIP-322 verify path. Closes the TV-3 boundary; enables actual Coordinator-side workflow.
- **Why deferred:** Cycle 7b scope was Round-2 encrypted decrypt only (Signer-side workflow). Round-1 encrypted-then-verify needs additional orchestration + integration test surface.
- **Status:** `resolved bc8fe1b` — mnemonic-toolkit-v0.32.1 Cycle 15. `--bsms-round1` auto-detects encrypted records (`is_encrypted_bsms_record`: raw hex, no `BSMS 1.0` header) + decrypts with the shared `--bsms-encryption-token` (new `BsmsToken` struct, read+width-validated once, shared with the Round-2 block via the new `decrypt_bsms_record(text, token, ctx)` helper; stdin guard hoisted above the Round-1 verify path) → MAC-verify → existing `parse_round1` + BIP-322 verify. Encrypted-without-token → BadInput exit 1; MAC fail → exit 2. Round-2 path byte-identical. 5 integration + 1 unit cell (incl. decrypt-OK-but-sig-FAIL lenient+strict, fixture via test-time re-encryption). TV-3 decrypted record BIP-322-verifies. 2198 cells. End-of-cycle opus GREEN 10/10 (MAC-verify ordering audited). No new flag → no GUI lockstep.
- **Tier:** `v0.31+`
- **Tags:** `wallet`
- **Companion:** parent `bsms-bip129-encryption-envelope` (resolved v0.31.0).


### `bsms-encryption-cross-impl-coinkite-python-smoke` — automated cross-impl test against Coinkite Python ref

- **Surfaced:** 2026-05-21, mnemonic-toolkit-v0.31.0 Cycle 7b close. Cycle 7a/7b cross-checks `bsms_crypto` against the BIP-129 + v0.27.0-recon-dossier locked TV-3 values (already byte-exact). Cycle 7b does NOT clone Coinkite's `bsms-bitcoin-secure-multisig-setup` repo + run `test.py` against the toolkit binary as a CI-gated cross-impl smoke.
- **Where:** new CI workflow + new test file (TBD). Coinkite ref: `https://github.com/coinkite/bsms-bitcoin-secure-multisig-setup` (`bsms/bip129.py`, `bsms/encryption.py`, `test.py`).
- **What:** v0.31+: ship an automated cross-impl test that (a) clones the Coinkite repo to a `tests/external/bsms-coinkite-python` location (gitignored or git-submodule); (b) runs `python3 test.py` to verify the Python ref still passes its own TVs; (c) re-encrypts a known plaintext + token via Python, hex-encodes the wire, feeds into `mnemonic import-wallet --bsms-encryption-token`; (d) cross-validates the round-trip. Gates the toolkit's BIP-129 implementation against any future Coinkite-Python changes (TV-pinning + behavior-pinning).
- **Why deferred:** Cycle 7b ships byte-exact cross-validation against the BIP-129 §Test Vectors values inherited from the v0.27.0 dossier; automated cross-impl smoke against the Python ref is a separate CI surface. Lower-priority than the integration coverage already shipped.
- **Status:** `resolved f442f7d` — mnemonic-toolkit-v0.32.3 Cycle 17. **Scope NARROWED to VENDORED-ONLY per a deliberate user scope-lock** (2026-05-21). Shipped: a Coinkite-`encrypt()`-generated Round-2 descriptor wire (`bsms-coinkite-xref-round2-2of3.dat`, EXTENDED 16-byte token) as a committed fixture + `tests/external/regen_coinkite_vectors.py` (deterministic, self-verifying) + README (pinned Coinkite SHA `c30abe3a6d9823b6a3003e89acd66b9f38e11f1c`, pyaes venv recipe). 3 integration cells (full-plaintext byte-equality + end-to-end CLI descriptor-equality + wrong-token MAC mismatch). Both Round-1 (existing TV-3 STANDARD) + Round-2 (new EXTENDED) directions + both token widths cross-validated against the independent Coinkite reference. **WAIVED (not deferred): items (a) live clone + (b) run `python3 test.py` + (c) live-CI-gating from the original FOLLOWUP body.** Rationale: the Coinkite repo is frozen (last push 2023-01-24), the toolkit crypto is already byte-exact against BIP-129 TV-3, and a live external-clone + pip CI surface adds fragility for marginal drift-detection value. The `regen_coinkite_vectors.py` script is the documented manual-refresh path. This intentional narrowing is recorded in CHANGELOG §Scope-note + `tests/external/README.md`. **With this closure, the parent `bsms-bip129-encryption-envelope` Cycle-7 follow-on arc is FULLY RETIRED** (all 3 child slugs closed: round1-decrypt-then-verify v0.32.1 + per-signer-tokens v0.32.2 + cross-impl-coinkite-smoke v0.32.3).
- **Tier:** `v0.31+`
- **Tags:** `wallet`
- **Companion:** parent `bsms-bip129-encryption-envelope` (resolved v0.31.0; arc fully retired v0.32.3).


### `nostr-import-spending-descriptors` — spending (private) importdescriptors on `mnemonic nostr`

- **Surfaced:** 2026-05-22, mnemonic-toolkit-v0.34.2 (`nostr --import readonly` close). v0.34.2 ships READ-ONLY only; `--import spending|both` is reserved (parser rejects with a "deferred" message).
- **Where:** `crates/mnemonic-toolkit/src/cmd/nostr.rs` (`parse_import_mode` + the `run` emission); a new spending-descriptor builder (the watch-only path uses `wallet_export::import_array_single` on the pubkey descriptor).
- **What:** enable `--import=spending|both`: emit a SPENDING importdescriptors recipe embedding the WIF — `wpkh(<WIF>)#csum` / `pkh(<WIF>)` / `sh(wpkh(<WIF>))` / `tr(<WIF>)` (Bitcoin Core descriptor wallets support `tr(<WIF>)` key-path spend, BIP-86 — unlike Electrum). Requires a secret input (`--secret*`); `--pubkey` + spending → refuse. Adds secret material to stdout → fire the secret-on-stdout advisory (the read-only path does NOT). Companion: a spending importdescriptors surface on `convert` (`--from wif=/xprv=`).
- **Why deferred:** v0.34.2 scope-locked to watch-only (user direction 2026-05-22). Spending embeds the private key in the descriptor + import JSON — its own secret-handling + UX pass.
- **Status:** `open`
- **Tier:** `v0.34+`
- **Tags:** `wallet`
- **Companion:** parent `nostr-key-wrappers` (v0.34.0); read-only shipped v0.34.2.


### `export-wallet-timestamp-default-zero` — make `--timestamp` default `0` everywhere (currently `now` on export-wallet)

- **Surfaced:** 2026-05-22, mnemonic-toolkit-v0.34.2. `nostr --timestamp` defaults to `0` (rescan-from-genesis — discovers an existing key's funds); `export-wallet`'s `--timestamp` defaults to `"now"` (`crates/mnemonic-toolkit/src/cmd/export_wallet.rs:117`). User wants `0` as the consistent default everywhere.
- **Where:** `crates/mnemonic-toolkit/src/cmd/export_wallet.rs:117` (`default_value = "now"` on `--timestamp`).
- **What:** change export-wallet's `--timestamp` default `"now"` → `"0"`. This is a **behavior change** to the emitted `importdescriptors` recipe (default rescan-from-genesis instead of watch-going-forward) — a heavier default rescan; warrants its own SemVer call + a deliberate decision (some users prefer `now` for a fresh wallet). Pair with the docs sweep below.
- **Why deferred:** behavior change to an existing flag's default; not bundled into the v0.34.2 additive-flag PATCH.
- **Status:** `open`
- **Tier:** `v0.34+`
- **Tags:** `wallet`
- **Companion:** sibling `timestamp-zero-default-docs-sweep`.


### `timestamp-zero-default-docs-sweep` — update docs for the `0` default once it lands everywhere

- **Surfaced:** 2026-05-22, mnemonic-toolkit-v0.34.2.
- **Where:** `docs/manual/` (any chapter implying `--timestamp` defaults to `now`) + any SPEC mentioning the timestamp default.
- **What:** once `export-wallet-timestamp-default-zero` lands, update all documentation that states/implies `--timestamp` defaults to `now` to reflect the `0` default. (v0.34.2's `nostr` manual already documents `0`.)
- **Why deferred:** docs-only follow-on to the behavior change above.
- **Status:** `open`
- **Tier:** `v0.34+`
- **Tags:** none
- **Companion:** sibling `export-wallet-timestamp-default-zero`.


### `cargo-lock-version-bump-lockstep` — version bumps must regenerate + commit `Cargo.lock`; add a `--locked` CI guard

- **Surfaced:** 2026-05-22, mnemonic-toolkit-v0.34.2. During v0.34.2 staging, `Cargo.lock`'s `mnemonic-toolkit` entry was found at `0.34.0` while `Cargo.toml` had been bumped through `0.34.1` → so **the `mnemonic-toolkit-v0.34.1` tag shipped with a stale lock** (`git show mnemonic-toolkit-v0.34.1:Cargo.lock` → `version = "0.34.0"`; `git show mnemonic-toolkit-v0.34.1:crates/mnemonic-toolkit/Cargo.toml` → `version = "0.34.1"`).
- **Impact:** the default installer path for `mnemonic` is `cargo install --locked --git … --tag mnemonic-toolkit-v0.34.1 mnemonic-toolkit` (`scripts/install.sh` uses `$LOCKED="--locked"` + git+tag for the toolkit). `--locked` refuses to update the lock, so a `Cargo.toml`/`Cargo.lock` version mismatch makes that install **fail** on the v0.34.1 tag. v0.34.2 corrects the lock (`cargo build --locked -p mnemonic-toolkit` passes), restoring the path — but v0.34.1 remains a broken-for-`--locked`-install intermediate tag.
- **Why CI missed it:** `install-pin-check` only greps the `install.sh` self-pin string against the tag; it never runs an actual `--locked` install/build, so a stale lock is invisible to it.
- **What:** (a) version-bump discipline — every `Cargo.toml` version bump MUST be followed by `cargo build` (or `cargo update -p mnemonic-toolkit --precise <ver>`) to regenerate `Cargo.lock`, and the lock change committed in the same release commit; (b) add a CI guard — a `cargo build --locked -p mnemonic-toolkit` (or `cargo metadata --locked`) step in the release/check workflow so a `Cargo.toml`/`Cargo.lock` mismatch fails fast at tag time.
- **Resolution (2026-05-27):** part (b) shipped — `.github/workflows/rust.yml` now runs a fail-fast `cargo metadata --locked --format-version 1 > /dev/null` step ("Verify Cargo.lock is up to date") before the build, on both ubuntu + macos. A `Cargo.toml`/`Cargo.lock` version mismatch (the v0.34.1 failure mode) now errors in CI on every push/PR that touches `crates/**`, `Cargo.toml`, `Cargo.lock`, or the workflow. Part (a) (regenerate+stage the lock at bump time) is now enforced by (b) rather than relying on discipline alone. actionlint-clean.
- **Status:** `resolved` (CI `--locked` guard added 2026-05-27)
- **Tier:** `v0.34+`
- **Tags:** `wallet`


### `silent-payment-change-address-m0` — emit the BIP-352 m=0 change address (explicitly, never-publish-labeled)

- **Surfaced:** 2026-05-23, v0.35.0 `mnemonic silent-payment` (architect consult C2). The cycle refuses `--label 0` (m=0 is the reserved change label that must NEVER be published) and defers emitting the change address.
- **Where:** `crates/mnemonic-toolkit/src/cmd/silent_payment.rs` (`--label 0` refusal) + `src/silent_payment.rs` (`labeled_spend_key`, which already handles any m).
- **What:** add a dedicated `--change-address` flag that emits the m=0 labeled address explicitly tagged "change — DO NOT publish" (per BIP-352 §Labels). The crypto already supports it (`labeled_spend_key(secp, b_scan, b_spend, 0)`); this is a UX/safety surface only. Needs a deliberate footgun-guard (clear labeling) so a user can't paste it as a receiving address.
- **Status:** `resolved` (v0.36.1)
- **Tier:** `v0.35+`
- **Tags:** none


### `silent-payment-passphrase` — `--passphrase` (BIP-39 passphrase) on `mnemonic silent-payment`

- **Surfaced:** 2026-05-23, v0.35.0 `mnemonic silent-payment`. v1 resolves seed-bearing secrets (phrase/ms1/entropy/xprv) with an EMPTY BIP-39 passphrase (`resolve_master_xpriv` calls `derive_master_seed(&mnemonic, "")`), so the derived address is for the no-passphrase wallet only.
- **Where:** `crates/mnemonic-toolkit/src/cmd/silent_payment.rs::resolve_master_xpriv` (the `derive_master_seed(&mnemonic, "")` calls).
- **What:** add a `--passphrase` / `--passphrase-stdin` flag (already secret-classed in `secrets.rs::flag_is_secret`) threaded into `derive_master_seed`, so a passphrase-protected wallet's silent-payment address can be derived. (The xprv input path is passphrase-independent — the xprv IS the master.)
- **Status:** `resolved` (v0.36.1)
- **Tier:** `v0.35+`
- **Tags:** none

### `verify-message-bip322-full-format` — `--format bip322-full` (BIP-322 full encoding)
- **Where:** `crates/mnemonic-toolkit/src/verify_message.rs::verify_bip322` (currently `verify_simple_encoded` only).
- **What:** v0.36.0 ships BIP-322 *simple* verify only. The crate also exposes `verify_full_encoded` (a different signature encoding: full to_sign transaction base64 vs witness-stack base64 — NOT interchangeable, so it needs a distinct `--format bip322-full`, not an auto-fallback). Add it if/when a full-format vector is requested.
- **Status:** `open`
- **Tier:** `v0.36+`
- **Tags:** none

### `electrum-phrase-address-refusal-honest-wording` — refine the electrum→address refusal message
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs:460` (the shared one-way-barrier message, interpolated for `(electrum-phrase, address)`).
- **What:** the shared message says "cryptographically unrecoverable … one-way derivation barrier" — technically imprecise for electrum-phrase→address (it's *unimplemented*, not unrecoverable; Electrum uses a different PBKDF2 salt + non-BIP-44 paths). Add a dedicated `(ElectrumPhrase, _)` refusal arm with honest wording before the shared barrier. Deferred from v0.36.0 (R0 disposition (b): don't widen scope / touch shared plumbing this cycle).
- **Status:** `open`
- **Tier:** `v0.36+`
- **Tags:** none

### `electrum-native-seed-address-derivation` — derive Electrum-correct addresses from an Electrum native seed
- **Where:** `crates/mnemonic-toolkit/src/electrum.rs` (currently seed-version + phrase↔entropy only) + `cmd/convert.rs` (the `(ElectrumPhrase, address)` edge is refused).
- **What:** real feature surfaced by the v0.36.0 electrum spot-check: implement Electrum's own derivation — `PBKDF2-HMAC-SHA512(seed, "electrum"+passphrase, 2048)` → BIP-32 root → legacy `m/0/i`,`m/1/i` (version 01) / segwit paths (version 100) — so an Electrum seed can produce its Electrum-correct addresses. Distinct from BIP-39/BIP-44; today the edge is honestly refused rather than producing wrong addresses. Validate against Electrum's `test_wallet_vertical.py` vectors.
- **Status:** `open`
- **Tier:** `v0.36+`
- **Tags:** none

### `verify-message-format-requested-debug-string` — decouple `format_requested` JSON field from Debug
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_message.rs` (`format!("{:?}", args.format).to_lowercase()`).
- **What:** the `--json` `format_requested` field is derived from the clap enum's `Debug` impl. Correct today (`Bip322`→`"bip322"`), but a future multi-word `VerifyFormat` variant (e.g. `Bip322Full` per the full-format FOLLOWUP) would silently emit `"bip322full"`. Map explicitly when that lands. (Per-phase review M2, confidence-below-threshold.)
- **Status:** `open`
- **Tier:** `v0.36+`
- **Tags:** none

### `gui-decode-address-verify-message-schema-mirror` — GUI SubcommandSchemas for the two v0.36.0 subcommands
- **Where:** `mnemonic-gui/src/schema/mnemonic.rs`.
- **What:** v0.36.0 adds `decode-address` + `verify-message` (net-new clap surface). The GUI schema mirror must add both `SubcommandSchema`s (no secret flags → no secret-projection delta) + bump the toolkit pin. Shipped in lockstep as `mnemonic-gui` MINOR (Phase 5). Companion record; closed when the paired GUI release ships.
- **Status:** `open`
- **Tier:** `v0.36+`
- **Tags:** none

### `lint-argv-secret-flags-canonical-table-rebuild-from-clap` — rebuild the argv-leakage audit table (decay)
- **Surfaced:** 2026-05-24, v0.36.1 end-of-cycle review.
- **Where:** `crates/mnemonic-toolkit/tests/lint_argv_secret_flags.rs` (`CANONICAL_FLAG_ROWS` + `assert_eq!(CANONICAL_FLAG_ROWS.len(), 28)`).
- **What:** the hand-curated `CANONICAL_FLAG_ROWS` table (self-described as "the canonical enumeration of secret-bearing argv flag-rows") froze at v0.13.0 and now omits every post-v0.13.0 secret-bearing argv flag: `nostr --secret` (v0.34.0), `silent-payment --secret` (v0.35.0), and `silent-payment --passphrase`/`--passphrase-stdin` (v0.36.1). It fails no test (curated list + hardcoded count + per-row evidence, with NO closure deriving rows from clap), so the decay is silent. The security-load-bearing projections (`secrets::flag_is_secret` + runtime `secret_in_argv_warning`) ARE correct + complete; this is the AUDIT table only. Fix: either rebuild `CANONICAL_FLAG_ROWS` from the clap surface (closure — fails when a new secret argv flag is unlisted) or backfill all missing post-v0.13.0 rows + bump the count. Prefer the closure (leading gate, not a lagging checklist).
- **Status:** `resolved` (v0.36.2)
- **Tier:** `v0.36+`
- **Tags:** none

### `import-wallet-ms1-argv-advisory-gap` — `import-wallet --ms1` fires no secret-in-argv advisory
- **Surfaced:** 2026-05-24, v0.36.2 R0 (I3 NOTE).
- **Where:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (`--ms1` intake).
- **What:** `import-wallet --ms1` is secret-bearing argv but fires NO `secret_in_argv_warning` (unlike most secret-argv flags). Its non-argv channel is the `@env:VAR` sentinel only — so NOT a missing-route leak (the v0.36.2 closure anchors it on `@env:`), but the runtime argv-leak ADVISORY is absent for consistency with sibling secret flags. Add a `secret_in_argv_warning(stderr, "--ms1", "@env:VAR / --ms1-stdin")`-style advisory. Pre-existing; out of scope for the v0.36.2 test-only cycle.
- **Status:** `open`
- **Tier:** `v0.36+`
- **Tags:** none

### `manual-prose-command-execution-gate` ✅ RESOLVED 2026-05-27 — execute documented recipes in the manual lint
- **Surfaced:** 2026-05-24, v0.36.3 documentation audit (G5).
- **Where:** `docs/manual/tests/lint.sh` (6 stages) + `docs/manual/src/45-foreign-formats.md` (round-trip recipes).
- **What:** the manual lint validates flag NAMES (stage 4), spelling, links, glossary, index — but NEVER EXECUTES the documented commands. The v0.28.1 round-trip breakage (all 6 chapter-45 `export-wallet` recipes failed; `design/AUDIT_FINDINGS_manual_v0_28_0_content.md`) shipped silently for exactly this reason and was fixed reactively. Build a lint stage / integration test that extracts the documented round-trip recipes (or a curated subset) and RUNS them against the pinned binary, asserting success + (where feasible) expected output. `feedback_architect_must_run_prose_commands` records the manual discipline; this automates it. Also consider adding lychee `--include-fragments` to validate intra-doc `#anchor` links (the v0.36.3 cycle found `#mnemonic-xpub-search` would dangle with no lint backstop). Meatier; own cycle + R0.
- **v0.36.4 finding (coupling):** test-running the 6 chapter-45 round-trips revealed **5 are impossible as written** (sparrow/coldcard/jade/electrum: `--from-import-json` carries a descriptor + `conflicts_with --template`, but these template-requiring file-formats reject a bare descriptor — "descriptor passthrough is not supported"). They are BLOCKED on the `export-wallet-from-import-json-template-format-reemit` CLI fix. The gate can initially cover the WORKING recipes (specter via `--wallet-name`; descriptor-passthrough re-emits to bitcoin-core/bip388/bsms) and expand once the CLI fix lands. (specter @:405 works; sparrow @:313 / coldcard @:481,:564 / jade @:639 / electrum @:752 broken.)
- **Resolution (2026-05-27):** the harness was already complete and CI-wired before this cycle (`verify-examples.sh` + Makefile `audit` + `manual.yml` "Audit manual" step). The cycle added the missing transcript coverage: **6 chapter-45 round-trip recipes** in `docs/manual/transcripts/foreign-formats/` (sparrow/specter/coldcard-SS/coldcard-MS/jade/electrum), all unblocked by v0.37.0's `export-wallet-from-import-json-template-format-reemit`. Plus harness hardening: `verify-examples.sh` `:=true` defaults → `:?` required so a misconfigured CI errors fast instead of vacuously passing (Piece 2). Plus chapter-45 prose addenda noting the two non-empty-diff recipes (sparrow `:321` + coldcard-MS `:573`). Full audit green: `[verify-examples] OK (20 transcripts pass)`. Piece 3 (lychee `--include-fragments`) deferred to FOLLOWUP `manual-anchor-dangler-backlog-cleanup` (pre-flight surfaced 174 build-output danglers — broader backlog). New side-FOLLOWUPs filed: `manual-yml-sibling-pin-vs-install-sh-drift-gate` (defense-in-depth) + `sparrow-from-import-json-wallet-name-preservation` (CLI-fix; sparrow recipe's non-empty diff). Specs + R0/R1 reviews at `design/SPEC_manual_prose_execution_gate.md` + `design/agent-reports/manual-prose-gate-R{0,1}-review.md` (R0 RED 1C/4I/3M → R1 GREEN).
- **Status:** `resolved` (2026-05-27)
- **Tier:** `v0.36+`
- **Tags:** none

### `manual-yml-and-install-sh-sibling-gui-pin-staleness` — manual.yml + install.sh non-`mnemonic` pins lag
- **Surfaced:** 2026-05-24, v0.36.3 R0 (M2) + end-of-cycle (M1).
- **Where:** `.github/workflows/manual.yml:77/84/88`; `scripts/install.sh:35/38/41` (siblings) + `:44` (GUI).
- **What:** TWO pin-staleness sites, neither gated by install-pin-check (which only checks the `mnemonic` self-pin @install.sh:32):
  (1) `manual.yml:77/84/88` installs `mk-cli-v0.4.1` / `descriptor-mnemonic-md-cli-v0.6.0` / `ms-cli-v0.4.0` while `install.sh` pins `v0.4.2` / `v0.6.1` / `v0.4.1`.
  (2) **`install.sh:44` pins `mnemonic-gui-v0.10.0` — far behind the live GUI v0.21.1** (impacts actual installs: the default all-5 `install.sh` would install a 10-version-stale GUI). Bump to the current GUI tag.
  Pre-existing drift; neither affects the `mnemonic` flag-coverage this cycle wired. Bump both to current tags (ideally derive from a shared pin / add a gate). The GUI pin (2) is the higher-impact one.
- **Status:** `resolved` (v0.36.4 — manual.yml siblings + quickstart.yml:71 mk-cli-v0.2.0→v0.4.2 + install.sh:44 gui v0.10.0→v0.21.1)
- **Tier:** `v0.36+`
- **Tags:** none

### `export-wallet-from-import-json-template-format-reemit` — `--from-import-json` re-emit to template-requiring formats
- **Surfaced:** 2026-05-24, v0.36.4 cycle (prose-gate recon test-run + user decision).
- **Where:** `crates/mnemonic-toolkit/src/cmd/export_wallet.rs::run_from_import_json` (the `--from-import-json` path); `wallet_export/mod.rs:211 script_type_from_descriptor`.
- **What:** `export-wallet --from-import-json <envelope>` re-emits via the envelope's descriptor and works for descriptor-passthrough formats (bitcoin-core/bip388/bsms) — but REFUSES template-requiring file-import formats (sparrow/coldcard/jade/electrum) with "descriptor passthrough is not supported by <format>'s file-import surface". `--from-import-json` `conflicts_with` `--template`, so the user can't supply the needed template either → these formats CANNOT round-trip via `--from-import-json` (5 documented chapter-45 recipes are impossible; see `manual-prose-command-execution-gate`). FIX: auto-derive the `--template` from the envelope's `script_type` (the envelope carries it; `script_type_from_descriptor` already derives `WalletScriptType` from the descriptor) so template-requiring formats re-emit. **SemVer MINOR** (new export-wallet behavior); own R0; behavior-only (no flag change → likely no GUI schema_mirror lockstep, but manual round-trip recipes update in lockstep). **R0 MUST budget for the multisig inverse-ambiguity:** `WalletScriptType → CliTemplate` is 1:1 for singlesig (P2wpkh→bip84, P2pkh→bip44, P2shP2wpkh→bip49, P2tr→bip86) but AMBIGUOUS for multisig (P2wshMulti ← WshMulti|WshSortedMulti; P2trMulti ← TrMultiA|TrSortedMultiA, `script_type_from_template:191-203`) — the envelope may need to carry the precise template, or the fix scopes to singlesig first.
- **Resolution (v0.37.0):** Fixed by deriving the template from the *parsed descriptor* (new `template_from_descriptor`, `wallet_export/mod.rs`) rather than the lossy `WalletScriptType` — which dissolves the inverse-ambiguity entirely (the descriptor carries `multi` vs `sortedmulti` verbatim). Injected only for template-requiring formats via `format_requires_template` (`cmd/export_wallet.rs`); passthrough formats keep `template: None`. Non-taproot scope (taproot stays walled by the pre-existing `wallet-import-taproot-internal-key` refusal). Manual chapter-45 recipes stripped of `--template` in lockstep. Plan opus R0 GREEN; per-phase review GREEN.
- **Status:** `resolved`
- **Tier:** `v0.36+`
- **Tags:** none

### `m-format-incorrect-length-recovery` — recover m*1 (md1/mk1/ms1) strings of incorrect length (indel recovery)
- **Surfaced:** 2026-05-24, user feature request (post-v0.37.0). Full handoff: `design/CONTINUITY_m_format_incorrect_length_recovery.md`.
- **Where:** `crates/mnemonic-toolkit/src/repair.rs` (BCH substitution correction; `bch_code_for_length` picks the code variant FROM the length, so wrong-length inputs error as `RepairError::ReservedInvalidLength` `:406` / `UnsupportedCodeVariant` `:414`); `cmd/repair.rs` + `cmd/inspect.rs` (`inspect` reports `byte_length` `cmd/inspect.rs:195` but no recovery); `cmd/final_word.rs` (the enumerate-candidates-validate-by-checksum analogue); sibling codecs' `decode`/`decode_with_correction` (the per-candidate validation oracle).
- **What:** recover an `m*1` string where a character was **inserted (too long)** or **dropped (too short)** during hand-copy/engraving — so it no longer decodes. **Distinct from `mnemonic repair`**, which is BCH *substitution* correction at FIXED length; an indel shifts every subsequent symbol and breaks the BCH codeword, so this needs a different algorithm. Likely **toolkit-side enumerate-and-validate** (delete each position for too-long; insert each of 32 charset symbols at each position for too-short) using the codec `decode` as oracle — probably no sibling-codec change. **Open decisions (brainstorm):** surface (flag on `repair` vs new subcommand — affects GUI/manual lockstep); indel direction + budget (default off-by-1); which HRPs (md1 chunked / mk1 long-codes / ms1 secret-bearing); ambiguity output contract; combine-with-substitution (likely defer); secret-on-stdout advisory for ms1. bech32m checksum is the validity oracle (charset = `ALPHABET`, `repair.rs:28`). **DO NOT plan on the bech32 upstream `Corrector`** — still unavailable (v0.11.1).
- **SemVer:** MINOR if new subcommand; PATCH if additive flag on `repair`.
- **Resolution (v0.37.1):** Shipped `mnemonic repair --max-indel <N>` (ms1+mk1; toolkit-only enumerate-and-validate around the existing BCH decode — `indel.rs` engine, two producers (prefix restore + data-part delete/placeholder-solve), per-kind oracles in `repair.rs`). j≤4 (the BCH t=4 error-correction ceiling); exit 0/5/4/2; `HrpMismatch` joins the indel trigger so prefix-region indels recover. md1 (chunked) refused — FOLLOWUP `m-format-indel-md1-chunked`. Brainstorm + plan opus R0→R1 GREEN; per-phase reviews GREEN.
- **Status:** `resolved`
- **Tier:** `v0.37+`
- **Tags:** none

### `m-format-indel-erasure-decode-extend-to-8` — extend too-short recovery from j≤4 to j≤8 via erasure-decode
- **Surfaced:** 2026-05-24, v0.37.1 cycle-close (FOLLOWUP (a)).
- **What:** v0.37.1 too-short recovery is bounded at j≤4 because it reuses the existing BCH error-decoder (capacity t=4). Erasure-decode (position-known symbols) has capacity 2t=8 per the BCP code theory; adding a `decode_with_erasures(positions: &[usize])` primitive to the sibling codecs would double the reach for dropped-character recovery. **Sibling-codec change required** — this is a new primitive in `mnemonic-secret` (`ms_codec`), `mnemonic-key` (`mk_codec`), and `descriptor-mnemonic` (`md_codec`); the toolkit consumes it. Companion FOLLOWUP entries are to be filed in each sibling repo `design/FOLLOWUPS.md` in lockstep when this is scheduled (per CLAUDE.md cross-repo convention).
- **SemVer:** PATCH if flag-extension only (j stays ≤4 → ≤8 with no surface change, just a wider default budget).
- **Status:** `open`
- **Tier:** `v0.37+`
- **Tags:** none
- **Companion:** `mnemonic-secret/design/FOLLOWUPS.md`, `mnemonic-key/design/FOLLOWUPS.md`, `descriptor-mnemonic/design/FOLLOWUPS.md` — entries to be filed in lockstep when scheduled.

### `m-format-indel-md1-chunked` — md1 indel recovery (currently refused)
- **Surfaced:** 2026-05-24, v0.37.1 cycle-close (FOLLOWUP (b)).
- **Where:** `crates/mnemonic-toolkit/src/repair.rs::recover_indel_card` — `md1` hits the `CardKind::Md1 => Err(ToolkitError::BadInput(...))` refusal arm.
- **What (corrected framing — verified against md-codec source):** md1 is chunked, BUT this is **conceptually analogous to the shipped mk1 path** (per-chunk BCH-solve + a cross-chunk reassembly oracle), NOT a deep new problem. The header (`version`/`chunked-flag`/`chunk_set_id`/`count`/`index`) is the LEADING bits of each chunk's data payload and is **fully BCH-protected** (the checksum covers it) — it is NOT a special unprotected/corruptible surface; a header-region indel is recovered by the same per-chunk length-restore + BCH-solve. The genuine friction is two-fold: (1) **the toolkit has no md per-chunk BCH path** — post-D29 it dropped its md target (`CardKind::target_residue(Md1) => None`, `repair.rs:79`) and delegates md1 wholesale to `md_codec::decode_with_correction(&[&str])`; and (2) **validation is cross-chunk**: a recovered chunk is only confirmed by reassembling the whole set (shared `chunk_set_id`/`count`, complete `0..count` index coverage). **Resolution path (toolkit-only, no sibling change — RECON-CONFIRMED):** md-codec exposes `bch::MD_REGULAR_CONST`, `bch::polymod_run`, `bch_decode::decode_regular_errors`, and `chunk::reassemble(&[&str])`; and md's `GEN_REGULAR` is byte-identical to mk's (shared codex32 generator). So re-acquire `MD_REGULAR_TARGET = md_codec::bch::MD_REGULAR_CONST`, extend `target_residue(Md1, Regular) => Some(...)` (Long stays None — md is regular-only), and mirror `Mk1IndelOracle`/`mk1_chunk_solve` as `Md1IndelOracle`/`md1_chunk_solve` with the cross-chunk oracle = `md_codec::chunk::reassemble` (which, unlike `mk_codec::decode`, does NOT self-correct → no unguarded-correction concern). Locate the failing chunk via `repair_chunk_one(Md1, i, c).is_err()` (works once the md target exists). Own R0.
- **SemVer:** PATCH (additive; `--max-indel` flag already exists; md1 just becomes un-refused).
- **Resolution (v0.37.2):** Shipped — md1 un-refused in `repair --max-indel` by mirroring the mk1 per-chunk path onto md (`MD_REGULAR_TARGET` re-acquired from `md_codec::bch::MD_REGULAR_CONST`; `md1_chunk_solve`/`Md1IndelOracle` with `md_codec::chunk::reassemble` as the cross-chunk oracle; shared codex32 generator ⇒ toolkit-only). Per-chunk recovery + reassembly validation, same as mk1.
- **Status:** `resolved`
- **Tier:** `v0.37+`
- **Tags:** none

### `m-format-indel-cross-region-split` — recover indels distributed across BOTH prefix and data-part simultaneously
- **Surfaced:** 2026-05-24, v0.37.1 cycle-close (FOLLOWUP (c)).
- **Where:** `crates/mnemonic-toolkit/src/indel.rs` — v0.37.1 P1 (prefix producer) and P2 (data-part producer) run INDEPENDENTLY; a candidate requires ALL corrections to be within a single region (prefix OR data). A cross-region split (e.g. one prefix drop + one data drop) is not attempted.
- **What:** v1 is single-region-per-attempt. Cross-region recovery (j_prefix indels + j_data indels, j_prefix + j_data ≤ N) would require a combined search over both regions simultaneously. Combinatorial cost: O(len_prefix × len_data × 32^j_insert) — likely too expensive at j≥2 without a smarter search strategy (early-BCH pruning). Defer until there is a real user case.
- **SemVer:** PATCH (extends the search space of an existing flag; no new surface).
- **Resolution (v0.37.3):** Shipped: `recover_indel` restructured into a two-level prefix×data search (`prefix_restorations` × `data_variants`); `IndelRegion::CrossRegion`; subsumes the single-region producers (byte-identical at N=1).
- **Status:** `resolved`
- **Tier:** `v0.37+`
- **Tags:** none

### `m-format-indel-plus-substitution` — combine indel recovery with substitution correction sharing the t=4 budget
- **Surfaced:** 2026-05-24, v0.37.1 cycle-close (FOLLOWUP (d)).
- **Where:** `crates/mnemonic-toolkit/src/indel.rs::collect_data_delete` / `collect_data_insert` / `collect_prefix` (candidate producers) + the per-kind `IndelOracle` (`repair.rs::Ms1IndelOracle` / `mk1_chunk_solve`) — v0.37.1 accepts a candidate iff its BCH corrections are a **subset of the inserted-placeholder positions** (∅ for delete/prefix). Any residual BCH correction at a non-placeholder position signals a simultaneous substitution and causes the candidate to be rejected.
- **What:** allow mixed indel+substitution recovery sharing the t=4 budget: j_indel + e_subst ≤ 4. A 1-indel + 1-substitution simultaneous corruption (j=1, e=1) is plausible (a handwritten card with one transposed character AND one wrong character). The placeholder-subset check would relax from `corrections ⊆ placeholders` to `|corrections \ placeholders| ≤ e_budget`. Cost is bounded (same BCH decode; just a weaker accept gate); the real risk is false-positive rate from the wider accept window. Own R0.
- **SemVer:** PATCH (behavior extension of existing `--max-indel` flag; no new surface).
- **Resolution (v0.37.3):** Shipped: new `--max-subst <E>` (0..=4, default 0); oracle gate relaxed to `|corrections\placeholders| ≤ E`; `IndelCandidate.subst_count`; candidate-list + verify advisory + exit 4 on substitution-bearing (exit 5 reserved for pure-indel-unique). Toolkit-only error-decoder approximation; `erasure-decode-extend-to-8` stays open for the half-price-erasure version.
- **Status:** `resolved`
- **Tier:** `v0.37+`
- **Tags:** none

### `m-format-indel-hrpmismatch-suggestion-fallback` — fall back to the HrpMismatch "did you mean" suggestion when indel recovery fails
- **Surfaced:** 2026-05-24, v0.37.1 Phase-5 review (Minor m-hrp) + end-of-cycle.
- **Where:** `crates/mnemonic-toolkit/src/cmd/repair.rs::run` (the `IndelOutcome::Unrecoverable` arm returns `RepairError::IndelUnrecoverable`) + `repair.rs::is_indel_trigger` (now includes `HrpMismatch` so prefix-region indels engage) + `resolve_groups`' `relax_hrp_for_indel`.
- **What:** v0.37.1 makes `HrpMismatch` an indel trigger so prefix-region indels recover (`--ms1 s10…` → restore `ms1`). Opt-in cost: at `--max-indel ≥ 1`, a *genuine* wrong-HRP typo (e.g. `mk1real` to `--ms1`) now enters indel search and, on failure, returns the generic `IndelUnrecoverable` (exit 2) instead of the `HrpMismatch` "did you mean 'mk'?" Levenshtein-1 suggestion (the default `--max-indel 0` path still gives it). Refinement: when indel recovery returns `Unrecoverable` AND the originating `repair_card` error was `HrpMismatch`, surface the ORIGINAL `HrpMismatch` (with its suggestion) rather than `IndelUnrecoverable` — keeps prefix recovery AND the helpful typo hint. (Documented v0.37.1 in plan §1.7 + CHANGELOG as the known opt-in tradeoff.)
- **SemVer:** PATCH (error-message/exit refinement; no surface change).
- **Resolution (v0.37.3):** Shipped: on `Unrecoverable` for an originating `HrpMismatch`, surface the original suggestion instead of `IndelUnrecoverable`.
- **Status:** `resolved`
- **Tier:** `v0.37+`
- **Tags:** none

### `m-format-indel-asymmetric-delete-budget` — allow a larger budget for too-long/delete recovery (the `t`-unbounded direction)
- **Surfaced:** 2026-05-24, post-v0.37.1 (controller + user, tracing the spec's too-long/too-short asymmetry).
- **Where:** `crates/mnemonic-toolkit/src/cmd/repair.rs` (`--max-indel` `value_parser!(u8).range(0..=4)`) + `indel.rs::collect_data_delete` (too-long producer) vs `collect_data_insert` (too-short producer).
- **What:** v0.37.1 caps BOTH directions at `--max-indel ≤ 4`. That ceiling is the BCH error-decoder's `t = 4` capacity — but it only binds the **too-short** direction (which leans on the error-decoder to solve the inserted placeholder). The **too-long / delete-and-validate** direction needs NO correction: deleting the truly-inserted char reproduces the exact original codeword (`residue == 0`), so it is bounded only by enumeration cost `C(L, j)` and the ~32⁻¹³ false-positive floor (negligible well past j=4). Refinement: expose a larger budget for the delete direction — either a separate `--max-delete <N>` (allowing e.g. j≤6) or an internal asymmetric cap that lets too-long search further than too-short — independent of, and orthogonal to, `m-format-indel-erasure-decode-extend-to-8` (which lifts only the too-short side, and requires a sibling-codec change). Real risk is runtime: `C(L, j)` for L≈108 grows fast (j=6 ≈ 10⁹ candidates → seconds-to-minutes), so any extension needs a runtime guard/notice and likely a benchmark-driven cap. Own R0.
- **SemVer:** PATCH if internal-cap-only; if a new `--max-delete` flag is added ⇒ still PATCH (additive flag) BUT triggers GUI `schema_mirror` + manual mirror lockstep.
- **Status:** `open`
- **Tier:** `v0.37+`
- **Tags:** none

### `mk1-depth-child-compensating-check-watch` — toolkit depth-check compensates for mk1's unenforced depth/child reconstruction

- **Surfaced:** 2026-05-29, mk-codec test-hardening cycle.
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs:494-503` (the SPEC §4.5 path-depth==xpub-depth check) compensates for `mk-codec`'s unvalidated depth/child reconstruction.
- **What:** If `mk-codec` resolves its `mk1-depth-child-lossless-by-construction-unenforced` FOLLOWUP via option (a) (encode-time `XpubDepthMismatch`), this toolkit-side compensating check may become redundant and reviewable for removal. Until then it is load-bearing — do not drop it.
- **Status:** `resolved 1cce14c` — mk-codec shipped option (a) (the guard, 0.3.2 + extended to depth-0 in 0.4.0). v0.37.10 re-pinned the toolkit to mk-codec 0.4.0 and **removed** the `synthesize_multisig_watch_only:494-503` compensating reject — it is now superseded by the `mk1_origin_path` helper, which makes every mk1 card consistent-by-construction. See `mk1-card-origin-path-vs-xpub-depth-consistency`.
- **Tier:** `monitoring`
- **Companion:** `mnemonic-key` (mk-codec) FOLLOWUP `mk1-depth-child-lossless-by-construction-unenforced`.

### `mk1-card-origin-path-vs-xpub-depth-consistency` — mk1 card origin_path must round-trip its xpub; the descriptor origin differs in depth

- **Surfaced:** 2026-05-30, re-pinning the toolkit to mk-codec 0.4.0 (whose encode-guard `XpubOriginPathMismatch` rejected 74 pre-existing tests). Subsumes the WIF-bundle-depth0 concern.
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs` (8 `KeyCard::new` sites), `cmd/verify_bundle.rs` (cross-checks), `wallet_import/json_envelope.rs` (`envelope_to_resolved_slots`).
- **What:** The toolkit built every mk1 card with `origin_path = the DESCRIPTOR origin` (e.g. a depth-4 BIP-48 path) while the card carried an xpub at a different depth (e.g. a depth-3 account xpub exported by foreign multisig formats). On 0.3.1 (no guard) this silently emitted wrong-metadata mk1 cards. **Fix (v0.37.10):** a centralized `mk1_origin_path(xpub, descriptor_path)` helper derives the mk1 card's path from the xpub's own depth/child (truncate/extend/pad); md1's `path_decl` keeps the full descriptor origin independently; the verify-bundle cross-checks compare the decoded mk1 origin against md1's depth-`d` prefix (overlap-prefix); `bundle --import-json` sources the cosigner origin from the envelope metadata (md1), not the now-account-level mk1 card.
- **Why deferred:** N/A — shipped in v0.37.10.
- **Status:** `resolved 1cce14c` — toolkit v0.37.10 (mk-codec 0.4.0 adoption). Toolkit-only PATCH; no GUI/manual lockstep.
- **Tier:** `cross-repo`
- **Companion:** `mnemonic-key` (mk-codec) FOLLOWUP `mk1-no-path-depth0-support`.

### `output-type-stderr-advisory` — stderr one-line classification of what landed on stdout

- **Surfaced:** 2026-05-31, A1 (descriptor-form symmetry) brainstorm — the user's "tell me what kind of wallet hit stdout" idea, deferred as a separate cycle per the brainstorm-stage architect review §(B).
- **Where:** ~12 output-producing commands (`convert`, `bundle`, `export-wallet`, `addresses`, `import-wallet`, `derive-child`, `seedqr`, `silent-payment`, `electrum-decrypt`, `slip39`, `seed-xor`, `final-word`, `nostr`); the D9 secret-on-stdout advisory at `secret_advisory.rs:48-64` + the 4 inlined literals (`convert.rs:1099`, `bundle.rs:910`, `derive_child.rs:306`, `slip39.rs:684`).
- **What:** Emit a one-line stderr classification of the stdout artifact's security nature: `private key material (can spend)` / `watch-only` / `template`. Subsume D9 by complement — keep D9's exact (transcript-pinned) text for the secret case, add positive lines for the rest, route both through ONE shared helper, consolidate the 4 inlined literals. **Hard requirement: all output-producing commands or none** (half-coverage reads as false safety — "no line = safe" when it means "uncovered"); inert-output commands (`decode-address`/`verify-message`/`inspect`/`compare-cost`) explicitly emit nothing, documented as such. 3 classes only (no single/multi or network secondary axes — the user's boundary is spend-capability). Serves the user's standing no-new-key-hazards constraint.
- **Why deferred:** all-surfaces coverage + transcript re-capture is bigger than A1's 3-surface scope; the brainstorm-stage architect review made it the explicit next cycle (B).
- **Status:** Phase 1 SHIPPED 2026-05-31 (cycle B): mnemonic-toolkit v0.38.2 + ms-cli v0.5.1 — always-emit 3-class advisory on all mnemonic + ms output surfaces. Phase 2 (mk + md) = `output-type-stderr-advisory-sibling-sweep-mk-md`.
- **Tier:** `next-cycle`

### `descriptor-origin-extraction-dedup` — consolidate the 6×build_slot_fields / 4×origin_capture_regex / 7th key_regex copy

- **Surfaced:** 2026-05-31, A1 SPEC R0 (M4) — the new `descriptor_concrete_to_resolved_slots` (`wallet_import/pipeline.rs`) added a 7th origin-extraction pass (reusing the canonical widened `key_regex`).
- **Where:** `build_slot_fields` duplicated in 6 import parsers (`bsms.rs:399`, `specter.rs`, `sparrow.rs`, `coldcard.rs`, `coldcard_multisig.rs`, `electrum.rs`); `extract_origin_components`/`origin_capture_regex` in 4 (`bsms.rs:362/:516`, `specter.rs:362`, `sparrow.rs:582`, `bitcoin_core.rs:413`); the new A1 recovery loop in `pipeline.rs`.
- **What:** Consolidate all origin-extraction onto the single canonical (h-form-widened) `key_regex` + one shared `extract_origin_components`/`build_slot_fields` in `pipeline.rs`, parameterizing the per-parser error prefix. Dissolves the `import-parser-hform-origin-tolerance` follow-on automatically.
- **Why deferred:** the per-parser error-prefix differences make it a moderate refactor; out of scope for A1's PATCH.
- **Status:** open.
- **Tier:** `hygiene`

### `import-parser-hform-origin-tolerance` — import-wallet parsers' origin_capture_regex stays apostrophe-only

- **Surfaced:** 2026-05-31, A1 SPEC R0 (C2 option-b decision) — A1 widened only `key_regex` (`pipeline.rs:38`) to accept `h`-form hardened paths; the import parsers' separate, byte-identical `origin_capture_regex` copies (`bsms.rs:516` et al.) were left apostrophe-only because the new `--descriptor` path does not call them.
- **Where:** `wallet_import/{bsms,specter,sparrow,bitcoin_core}.rs` `origin_capture_regex`.
- **What:** Whether the `import-wallet` per-format parsers should also accept `h`-form wallet-file descriptors (Core/Sparrow exports) is pre-existing scope; A1 did not change it. Dissolved automatically if `descriptor-origin-extraction-dedup` lands (single canonical widened regex).
- **Why deferred:** pre-existing; orthogonal to A1's `bundle`/`verify-bundle --descriptor` surface.
- **Status:** open.
- **Tier:** `hygiene`

### `stale-foreign-format-transcripts-recapture-audit` — verify-examples baselines drifted from prior-cycle fixture/behavior changes

- **Surfaced:** 2026-05-31, A1 (descriptor-form symmetry) end-of-cycle — re-capturing the coldcard transcript revealed master's `manual` CI verify-examples gate has accumulated stale baselines.
- **Where:** `docs/manual/transcripts/` (`make -C docs/manual verify-examples`).
- **What:** Two stale transcripts found; one fixed in A1, one OPEN:
  1. **`foreign-formats/roundtrip-coldcard-multisig.out`** — RESOLVED in A1 (`<this cycle's commit>`). The v0.37.8 baseline encoded a since-fixed cosigner-xpub corruption (`B7F7DFEA: xpub6Do6nv…`, a key that exists nowhere else in the tree); the v0.37.9 `path_raw` deletion + v0.37.10 `mk1_origin_path` rework fixed the `envelope_to_resolved_slots` path but never re-captured this foreign-format transcript. Re-captured against current (correct) behavior: all 3 account xpubs preserved byte-for-byte, BIP-67 lex-sorted cosigner order (cosmetic per Coldcard upstream — order is not load-bearing for `sortedmulti`). Architect-reviewed.
  2. **`cross-format-recipes/recipe-2-bitcoin-core-to-bundle.{out,err}`** — RESOLVED 2026-05-31 (re-captured). Triage finding: the v0.37.8 baseline `blob declares xpub6DXqiLU…` referenced a key **that was NEVER in the fixture** — `git show 6ae7372:…core-mainnet-receive-change-pair.json` already had `xpub6FQya…` in its active-receive descriptor (`active:true, internal:false`, account xpub at `[b8688df1/84'/0'/0']`), and `git log -S xpub6DXqiLU` on the fixture is empty. So the v0.37.8 import declared a wrong-depth xpub (`xpub6DXqiLU` ≈ the `/0` receive-chain child of `xpub6FQya`) in the seed-vs-blob check — the SAME account-vs-derived-xpub bug class the v0.37.9 `path_raw` deletion + v0.37.10 `mk1_origin_path` rework fixed, and the SAME root cause as the coldcard transcript. **Current behavior is correct** (declares the fixture's actual account xpub `xpub6FQya`; the abandon-seed mismatch — seed fp `73c5da0a` vs fixture fp `b8688df1` — is by-design teaching of the seed-vs-blob consistency check). Re-captured `.out` + `.err`; full `make audit` GREEN (verify-examples 20/20). A1-independent (recipe-2 uses `bundle --import-json`, not `--descriptor`; apostrophe-path fixture, so A1's P0 `key_regex` superset-widening can't affect it).
- **Why deferred (recipe-2):** the `--select-descriptor` behavior change needs its own investigation (correct vs regression) before the baseline is updated; out of A1's descriptor-symmetry scope.
- **Status:** RESOLVED 2026-05-31 — both stale transcripts re-captured (coldcard in A1; recipe-2 follow-on). Full transcript-suite audit done: after both fixes, `make audit` is GREEN (verify-examples 20/20), so no further stale baselines remain. Both were the same root cause: a v0.37.8-era account-vs-derived-xpub bug fixed by v0.37.9/.10's origin-path rework, whose foreign-format transcripts were never re-captured at the time.
- **Tier:** `manual-hygiene`

### `output-type-stderr-advisory-sibling-sweep-mk-md` — extend the output-class stderr advisory to mk-cli + md-cli (Phase 2)

- **Surfaced:** 2026-05-31, cycle B Phase 1 ship (mnemonic + ms). Phasing per the brainstorm-stage architect review: secret-bearing surfaces (mnemonic, ms) first; the benign watch-only/template siblings (mk, md) second.
- **Where:** `mnemonic-key/crates/mk-cli` (NO advisory module today — greenfield), `descriptor-mnemonic/crates/md-cli`.
- **What:** Add the always-emit 3-class stderr advisory (byte-identical wording to `mnemonic-toolkit/src/secret_advisory.rs` — `private key material (can spend)` / `watch-only` / `template`) to: **mk** — `mk decode`/`derive`/`address`/`inspect` → watch-only; **md** — `md decode`/`encode` → **template** (the class's first real exercise — md1 IS a keyless template), `md address` → watch-only; inert subcommands emit nothing. Cross-repo byte-parity tests. Completes the constellation-wide "no advisory line ⟺ inert output" invariant.
- **Why deferred:** mk/md outputs are non-secret (the false-safety asymmetry makes their interim silence benign — over-caution, no fund-loss path), unlike the secret-bearing mnemonic/ms surfaces shipped in Phase 1. mk-cli additionally has no advisory scaffold (only `process_hardening`).
- **Status:** Resolved by the output-class-advisory Phase 2 cycle — mk-cli **v0.6.1** + md-cli **v0.6.2** + toolkit **v0.38.3** add the always-emit 1-line stderr output-class advisory (mk→watch-only; md→template, plus watch-only for `md address`); completes the constellation-wide 'no advisory line ⟺ inert stdout' invariant. Per-phase reviews persisted in mnemonic-toolkit `design/agent-reports/output-type-advisory-phase2-*`.
- **Tier:** `next-cycle`
- **Companion:** `mnemonic-key`, `descriptor-mnemonic` (mirror entries); `mnemonic-secret` companion (Phase 1 shipped ms).

### `output-class-advisory-byte-parity-test-tautological` — cross-repo byte-parity tests are within-repo tautologies, not cross-repo drift gates

- **Surfaced:** 2026-05-31, output-class-advisory Phase 2 cycle; Phase A (M1) + Phase B (M2) per-phase reviews.
- **Where:** `mnemonic-toolkit/crates/mnemonic-toolkit/src/secret_advisory.rs` (tests), `mnemonic-secret/crates/ms-cli/` (tests), `mnemonic-key/crates/mk-cli/` (tests), `descriptor-mnemonic/crates/md-cli/` (tests).
- **What:** The per-repo `byte_parity_advisory_lines` tests (and equivalents) assert each module's advisory-line constants equal an inline literal copy in the **same file** — a within-repo drift guard, not a cross-repo one. They do not `include_str!` or read the sibling source, so a divergence of the advisory wording in one repo will not be caught by the others. Real cross-repo byte-parity is currently enforced by convention and the paired-PR discipline only.
- **Options:**
  - Option (a): one canonical repo `include_str!`s the others (fragile across separate checkouts; requires explicit path coupling between independent repos).
  - Option (b): pin the 3 literals in a shared committed fixture file that each repo's test reads (viable if repos share a monorepo structure or a checked-in fixture is propagated via a CI step).
  - Option (c): accept convention-only enforcement and reword the module docs to stop claiming the test "enforces cross-repo parity" — replace the claim with "anchors this repo's emitted line to this source file."
- **Severity:** Low — the positive cells anchor each repo's emitted line to its own source (within-repo drift is caught), so advisory wording can only diverge via a paired-PR miss, which the FOLLOWUP discipline + pairing gate guards against.
- **Status:** open
- **Tier:** `cross-repo` / `hardening`
- **Companion:** Affects the advisory-line constants ported into `mnemonic-key` (mk-cli), `descriptor-mnemonic` (md-cli), and `mnemonic-secret` (ms-cli) — each carries the same self-tautological `byte_parity_advisory_lines` pattern. Mirror FOLLOWUP entries are **not yet filed** in those repos (this toolkit entry is the canonical tracker); they would be added alongside whichever option is pursued. While `open`, cross-repo enforcement is convention + paired-PR discipline only (the status quo this entry documents).

### `sibling-pin-check-skips-manual-prose-install-commands` — pin gate scans workflows but not manual/quickstart prose

- **Surfaced:** 2026-06-01, mk SLIP-0132 (A2) end-of-cycle review (C1).
- **Where:** `.github/workflows/sibling-pin-check.yml` (scans only `.github/workflows/*.yml` `cargo install --git … --tag` lines); blind to `docs/manual/src/**` prose install commands.
- **What:** `docs/manual/src/40-cli-reference/44-mk-cli.md` carried a literal `cargo install … --tag mk-cli-v0.6.0 …` install command that the `sibling-pin-check.yml` gate does NOT scan (prose, not a workflow). It drifted across TWO cycles — the v0.6.1 re-pin (`752801f`) and the v0.7.0 re-pin both bumped install.sh + the workflows but skipped this manual line — so the manual told readers to install a version lacking the very feature the same chapter documents. Fixed at A2 (→ `mk-cli-v0.7.0`), but the gate gap remains. (Currently only `44-mk-cli.md:12` carries such a prose install command; md/ms chapters have none — but the class is unguarded.)
- **Options:** extend the `sibling-pin-check.yml` scan to also match `cargo install --git … --tag <pkg>` lines in `docs/manual/src/**` (+ `quickstart` prose) against the install.sh canonical table; OR add a small `make -C docs/manual` lint that greps manual prose install commands against the pinned tags.
- **Severity:** Medium — a stale prose pin silently ships a wrong-version install instruction to end users; undetected across 2 cycles.
- **Status:** open
- **Tier:** `ci-hardening`

### `toolkit-mnem-ms1-wire-shape-downstream-consumers` — `mnem` ms1 is a new on-wire string shape; downstream consumers need ms-codec ≥0.3.0

- **Surfaced:** 2026-06-02, ms `mnem` cycle Phase 3 Step 7 (R0-M3).
- **Where:** Any consumer that receives a toolkit-emitted `bundle --json` or `export-wallet` envelope and re-decodes the `ms1[]` strings: notably `mnemonic-gui`'s bundle re-decode path. Prior consumers expected only `entr`-kind ms1 strings (lengths 48/55/61/68/75 for 16/20/24/28/32 B entropy, corresponding to ms1 codex32 lengths).
- **What:** A toolkit-emitted `mnem` ms1 card (produced for non-English-phrase bundle slots, ms-codec 0.3.0+) carries a one-byte language prefix in the payload, yielding different wire lengths: 51/58/64/70/77 instead of 48/55/61/68/75. ms-codec < 0.3.0 (or any consumer that range-checks the string length against the `entr` table) will reject the string as `UnexpectedStringLength`. Wire-shape is **NOT** `schema_mirror`-gated (the flag-name set is unchanged; this is a runtime payload shape change). Consumers of the `--json` wire shape must self-update to ms-codec ≥0.3.0 when they encounter `mnem` ms1 strings, coordinated via the paired-PR rule.
- **Why deferred:** Affects `mnemonic-gui` (and any third-party toolkit consumer) independently of the toolkit's implementation cycle. The fix is a ms-codec pin bump on the consumer side; the toolkit cannot gate it. Filed here to ensure the gap is tracked and the paired-PR discipline is applied when ms-codec 0.3.0 publishes to crates.io.
- **Status:** `open`
- **Tier:** `cross-repo`
- **Companion:** `mnemonic-gui` — the primary known downstream consumer. A companion entry should be filed in `mnemonic-gui`'s FOLLOWUPS tracker (or equivalent) once ms-codec 0.3.0 is published to crates.io and the GUI pin-bump cycle begins.
- **Companion:** none (toolkit-only; the manual lives here).
