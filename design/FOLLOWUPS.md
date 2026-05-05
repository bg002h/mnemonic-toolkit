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
