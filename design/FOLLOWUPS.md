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
- **Status:** `open`
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
- **Status:** `open`
- **Tier:** `v0.6.2`

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
- **Status:** `open`
- **Tier:** `v0.6.2`
