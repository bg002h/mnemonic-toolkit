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
- **`cross-repo`**: depends on coordination with sibling repos (`descriptor-mnemonic`, `mnemonic-key`, `mnemonic-secret`). Mirrored by a companion entry in the affected sibling's tracker; both cite each other.
- **`v1+`**: deferred indefinitely.
- **`external`**: depends on upstream work (e.g., a sibling crate exposing a helper).

---

## Open items

### `spec-5-5-kind-enum-gap` — SPEC §5.5 `kind` enum table omits `NetworkMismatch` and `FutureFormat`

- **Surfaced:** Phase 1 review r1 (L-1).
- **Where:** `design/SPEC_mnemonic_toolkit_v0_1.md` §5.5.
- **What:** SPEC §5.5 enumerates `"kind"` JSON values as `"BadInput" | "Bip39" | … | "ModeViolation"` but doesn't list `NetworkMismatch` and `FutureFormat`. The implementation correctly returns those discriminants; the SPEC prose is just incomplete.
- **Why deferred:** SPEC-prose-only; no code change required. Update during the next SPEC revision.
- **Status:** `open`
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
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `plan-trezor-24-fingerprint-stale` — IMPLEMENTATION_PLAN has wrong 24-word zero-entropy master fingerprint

- **Surfaced:** Task 2.1 implementer (verified via spike harness `/tmp/toolkit-spike/spike_trezor_fp.rs`).
- **Where:** `design/IMPLEMENTATION_PLAN_mnemonic_toolkit_v0_1.md` Task 2.1 test assertion (~line 1540) + Task 2.3 commit-message body.
- **What:** Plan asserts `73c5da0a` as the Trezor 24-word "abandon × 23 art" master fingerprint. That value is the **12-word** "abandon × 11 about" vector's fingerprint (rust-miniscript test corpus). Correct 24-word fingerprint is `5436d724`. Handoff doc was corrected during execution; plan source unpatched.
- **Why deferred:** test code uses correct value; only plan documentation is stale.
- **Status:** `open`
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
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `spec-2-2-2-vs-5-4-checks-count-prose` — SPEC §2.2.2 prose says "four checks" but §5.4 schema mandates 9-element array

- **Surfaced:** Phase 4 review r1 (L-1).
- **Where:** `design/SPEC_mnemonic_toolkit_v0_1.md` §2.2.2 vs §5.4.
- **What:** §2.2.2 lists 4 substantive watch-only checks; §5.4 schema (line 552) requires all 9 check-name slots populated, with `skipped` for non-applicable. Implementation follows §5.4 (correct). §2.2.2 prose should clarify "4 substantive (5 of the 9 schema slots are `skipped` per §5.4)".
- **Why deferred:** SPEC-internal inconsistency; implementation behavior is correct per the schema.
- **Status:** `open`
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
- **Status:** `open`
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
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `changelog-sha-pin-no-reproduction-command` — CHANGELOG SHA pin doesn't document how to reproduce it

- **Surfaced:** Phase 5 review r1 (N-1).
- **Where:** `CHANGELOG.md` Wire-format SHA pin section.
- **What:** SHA `81828299c927783d915108f32c9752b3dbf815c1caba5b6f6e7ce7b810ddcbf6` is documented as `sha256(crates/mnemonic-toolkit/tests/vectors/v0_1/)` but doesn't specify the exact reproduction command (`shasum -a 256 *.txt | sort | shasum -a 256`). Verifiers may need to guess.
- **Why deferred:** verifiers can re-derive; doc-only clarity gap.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `cli-mode-violations-byte-exact-naming` — test names say "byte_exact" but use `str::contains`

- **Surfaced:** Phase 5 review r1 (N-3).
- **Where:** `crates/mnemonic-toolkit/tests/cli_mode_violations.rs`.
- **What:** Several test names use the suffix `_byte_exact` but the assertions use `predicate::str::contains(...)` (substring match). Tests are correct; naming overstates assertion strictness. Either rename to `_substring` or tighten the assertions to full-stderr equality (and pin the byte-exact stderr in fixtures).
- **Why deferred:** assertion strength is sufficient for current SPEC pinning; naming is the only mismatch.
- **Status:** `open`
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
- **Status:** `open`
- **Tier:** `v0.2`

### `dead-inner-guard-bundle-watch-only` — redundant `--xpub`-needs-`--master-fingerprint` guard inside `bundle_watch_only`

- **Surfaced:** v0.1 audit 2026-05-05 (LOW-2).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs:200` (inside `bundle_watch_only`).
- **What:** A redundant guard exists that would emit `BadInput` (exit 1) if `--master-fingerprint` is missing. Unreachable in practice — the mode-violation pre-check at `cmd/bundle.rs:93` rejects the same condition earlier with exit 2 + byte-exact §6.6 text. Future-refactor inconsistency risk.
- **Why deferred:** not currently triggered; v0.2 will refactor mode dispatch and naturally clean this up.
- **Status:** `open`
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
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `parse_template-regex-line-ref` — SPEC v0.3 §4.9 step 2 cites wrong line range

- **Surfaced:** v0.3 SPEC architect review r2 2026-05-05.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_3.md` §4.9 step 2.
- **What:** Step 2 cites `descriptor-mnemonic/crates/md-cli/src/parse/template.rs:19-27` for the placeholder regex; the actual `Regex::new` call is at `:25-27` (line 19-24 are imports/doc-comments). Docs-only nit — implementation will read the actual regex from the source.
- **Why deferred:** non-blocking; can be patched alongside any v0.3 SPEC revision.
- **Status:** `open`
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
- **Status:** `open`
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

### `bundle-json-schema-4-cutover` — full BundleJson schema-4 cutover deferred from v0.4.0 to v0.4.1

- **Surfaced:** v0.4 Phase D scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/format.rs::BundleJson` + `crates/mnemonic-toolkit/src/cmd/bundle.rs::emit` + `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` + `crates/mnemonic-toolkit/src/synthesize.rs::Bundle`.
- **What:** v0.4.0 ships the `MsField = Vec<String>` type alias + multi-source synthesis primitives as a foundation, but DEFERS the full `BundleJson.ms1: Option<String>` → `ms1: MsField` migration + `schema_version: "3" → "4"` bump + verify-bundle schema-4 dispatch to v0.4.1. v0.4.0 retains the schema-3 envelope so all existing v0.2/v0.3 fixtures + JSON integration tests pass byte-identically. v0.4.1 lands the cutover with: (a) BundleJson.ms1 → MsField; (b) Bundle.ms1 → Vec<String>; (c) all integration test JSON assertions updated; (d) verify-bundle schema_version dispatch (read schema_version FIRST per SPEC §5.6); (e) regenerate or update v0.2/v0.3 carry-forward tests under the new envelope shape per SPEC §5.6 cross-schema invariant; (f) synthesize_multisig_multisource + synthesize_multisig_hybrid wired into bundle::run via BundleMode dispatch (Phase C foundation already in place).
- **Why deferred:** scope risk in autonomous v0.4.0 release window — full surgery touches ≥10 source files + ~15 test assertions + fixture envelopes; landing without user oversight risks bugs the foundation-only approach avoids.
- **Status:** `open`
- **Tier:** `v0.4.1`

### `bip388-distinctness-path-normalization-phase-b-decision` — typed-vs-raw path semantics in check_key_vector_distinctness

- **Surfaced:** v0.4 Phase A end-of-phase architect review L-1 (2026-05-05).
- **Where:** `crates/mnemonic-toolkit/src/parse_descriptor.rs:1049` (`check_key_vector_distinctness`); SPEC `design/SPEC_mnemonic_toolkit_v0_4.md` §4.11.b.
- **What:** Phase A compares `cs[i].path.to_string()` on typed `bitcoin::bip32::DerivationPath`. The bitcoin library normalizes `48h/0h/0h/2h` ↔ `48'/0'/0'/2'` at `from_str` time, so collision detection is normalization-aware. SPEC §4.11.b says "raw user-supplied path string ... no path canonicalization". In Phase A this is safe because all paths arrive through the typed lex/cosigner parser; in Phase B the `--slot @N.path=` raw string flows into the binding directly. Phase B must lock whether `CosignerKeyInfo.path` stores typed `DerivationPath` (normalizing) or raw `String` (preserving), then update SPEC §4.11.b's normalization-domain paragraph in lockstep.
- **Why deferred:** Phase A's typed approach is correct under the v0.3 binding model; the decision is a Phase B design choice (slot input parsing).
- **Status:** `open`
- **Tier:** `v0.4-nice-to-have`

### `bundle-removed-subcommand-trap-positional-eq-bypass` — `bundle multisig-full=value` token bypasses pre-clap trap

- **Surfaced:** v0.4 Phase 2 SPIKE r1 architect review L-2 (2026-05-05).
- **Where:** Phase C.1 `detect_removed_subcommand` (locked SPIKE shape at `design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md` SPIKE-2).
- **What:** Trap matches `argv[i+1] == "multisig-full"` with exact string equality. A token like `multisig-full=value` would not match and would fall through to clap's generic "unexpected argument" error rather than the byte-exact §6.6 row 1 message. Positional args do not idiomatically take `=value` form in shells, so this is essentially theoretical.
- **Why deferred:** no realistic user invocation produces this argv shape; a post-trap fallback in clap already rejects with exit 2.
- **Status:** `open`
- **Tier:** `v0.4-nice-to-have`

### `bundle-removed-subcommand-trap-double-dash-bypass` — `mnemonic bundle -- multisig-full` bypasses pre-clap trap

- **Surfaced:** v0.4 Phase 2 SPIKE r1 architect review L-3 (2026-05-05).
- **Where:** Phase C.1 `detect_removed_subcommand` (locked SPIKE shape at `design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md` SPIKE-2).
- **What:** With a `--` separator inserted between `bundle` and `multisig-full`, the trap reads `argv[i+1] == "--"` and skips. Clap then processes `multisig-full` as a positional after `--` and emits a generic "unexpected argument" error rather than the byte-exact §6.6 row 1 text. UX difference matters only if a user intentionally inserts `--` before a removed subcommand name — not a realistic migration-error path.
- **Why deferred:** vanishingly unlikely user error; clap's fallback still rejects with exit 2.
- **Status:** `open`
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
