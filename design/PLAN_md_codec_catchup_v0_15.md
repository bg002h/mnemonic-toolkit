# PLAN_md_codec_catchup_v0_15.md — md-codec v0.16.1 → v0.33.1 catchup

Architect-vetted 2026-05-16. Drives the toolkit's v0.15.0 cycle:
unblock `cargo publish` of `mnemonic-toolkit` (and downstream
`mnemonic-gui`) by swapping the git@v0.16.1 dep for crates.io v0.33.1.

## §0 Background and scope

Today the toolkit pins `md-codec` at git tag `md-codec-v0.16.1`
(`crates/mnemonic-toolkit/Cargo.toml:22`). Target is crates.io
`md-codec = "0.33.1"` so the toolkit becomes publishable. The
CHANGELOG between v0.16.1 and v0.33.1 contains **eight md-codec
entries** — note the **v0.20 → v0.29 numbering gap is a real skip**;
v0.30 is the next release after v0.19, not the twentieth. Two
versions (v0.30.0 + v0.32.0) are load-bearing breakage points; the
rest are additive or test-only.

`mk-codec` catchup (v0.2.1 → v0.3.0) is **folded into Phase 5** of
this plan — single minor bump, smaller blast radius than the
md-codec churn, no reason to split into its own cycle (§7).

## §1 CHANGELOG drift inventory (v0.17.0 .. v0.33.1)

| Version | Class | Toolkit impact |
|---|---|---|
| v0.16.2 | audit-only | None — test pin, no API change |
| v0.17.0 | additive wire | `Tag::TrUnspendable` added; toolkit never emits it, safe |
| v0.18.0 | **BREAKING wire** | `Tag::TrUnspendable` REMOVED, NUMS sentinel rule on `Body::Tr`. Toolkit's `template.rs` `TrMultiA`/`TrSortedMultiA` wrappers use `key_index: 0` (placeholder), never the NUMS sentinel — unaffected |
| v0.19.0 | additive error | `Error::DecodeRecursionDepthExceeded { depth, max }` added |
| v0.30.0 | **MAJOR BREAKING** | Wire-format clean break; **5 Error variants REMOVED** that the toolkit currently matches on; new variants added; `key_index_width` formula changed |
| v0.31.0 | additive error | `Error::OperatorContextViolation` decoder-side TopLevel rejection wired |
| v0.32.0 | **BREAKING (derive feature)** | `Error::UnsupportedDerivationShape` REMOVED; replaced by `Error::AddressDerivationFailed { detail }`. New `derive` feature gates miniscript dep (default-on) |
| v0.32.1 | tests only | No API change |
| v0.33.1 | docs only | No API change |

### §1.1 Error-variant delta (vs the toolkit's 30 referenced variants)

**Removed at v0.30** (toolkit matches all 4 — **MUST be deleted** from `error.rs` + `friendly.rs`):
- `ReservedHeaderBitSet`
- `UnsupportedVersion { got }` — also gone from the `From<md_codec::Error>` interceptor at `error.rs:463-472` (`FutureFormat` routing point disappears)
- `UnknownPrimaryTag(u8)`
- `UnknownExtensionTag(u8)`

**Removed at v0.32** (toolkit matches 1):
- `UnsupportedDerivationShape` — used in `error.rs:233` and `friendly.rs:242-244`

**Added** (new variants the toolkit MUST route):
- v0.19 `DecodeRecursionDepthExceeded { depth, max }` — exit 2
- v0.30 `WireVersionMismatch { got }` — **THE NEW `FutureFormat` ROUTING POINT** (replaces `UnsupportedVersion` semantically)
- v0.30 `MalformedHeader { detail }` — exit 2
- v0.30 `TagOutOfRange { primary }` — exit 2 (replaces UnknownPrimaryTag + UnknownExtensionTag)
- v0.30 `NUMSSentinelConflict` — exit 2
- v0.30 `OperatorContextViolation { tag, context: ContextKind }` — exit 2
- v0.32 `AddressDerivationFailed { detail }` — exit 2 (replaces UnsupportedDerivationShape)

`md_codec::Error` is **not** `#[non_exhaustive]` — toolkit's existing exhaustive-match assumption holds.

### §1.2 Module / type-shape stability (verified vs HEAD)

- All five modules toolkit imports (`chunk`, `encode`, `origin_path`, `use_site_path`, `tree`, `tag`) **still exist** as `pub mod`.
- `encode::render_codex32_grouped`, `chunk::split`, `chunk::reassemble`, `compute_wallet_policy_id` — **signatures unchanged**.
- `Descriptor { n, path_decl, use_site_path, tree, tlv }` — field set unchanged.
- `TlvSection { use_site_path_overrides, fingerprints, pubkeys, origin_path_overrides, unknown }` — field set unchanged.
- `OriginPath`, `PathComponent`, `PathDecl`, `PathDeclPaths::{Shared,Divergent}` — unchanged.
- `UseSitePath`, `Alternative`, `UseSitePath::standard_multipath()` — unchanged.
- `tree::{Node, Body}` — `Body` gained `MultiKeys { k, indices }` in v0.30 Phase C. **VERIFICATION SPIKE NEEDED**: confirm `Body::Variable` is still accepted for `Tag::Multi/SortedMulti/MultiA/SortedMultiA` by `encode::write_node`. If acceptance dropped, `template.rs:151-213` and `parse_descriptor.rs:327-350` need rewriting.
- `tag::Tag` — enum gained no toolkit-relevant variants.

### §1.3 Load-bearing semantic changes

- **v0.30 wire-format clean break**. The toolkit's 16 byte-pinned fixtures (`tests/vectors/v0_1/{template}-{network}.txt`) WILL mismatch — they were generated against v0.11-era wire format. **Fixture regeneration is REQUIRED.**
- **v0.18 NUMS sentinel rule on `Body::Tr`**: toolkit always uses `key_index: 0` for `Tr*MultiA` templates — semantically safe.
- **v0.32 miniscript-backed `derive_address`**: toolkit doesn't call `Descriptor::derive_address` (grep confirms zero calls); `derive` feature is default-on so no explicit feature opt-in needed.

## §2 Per-file blast-radius cluster

| File | md_codec refs | Drift category | Bucket |
|---|---|---|---|
| `src/error.rs` | 55 | **Error variant churn** (5 removed, 7 added, 1 `From` interceptor change) | Large |
| `src/friendly.rs` | 5 listed but ~115 LOC of match arms in `friendly_md_codec` | Same churn as error.rs | Large |
| `src/synthesize.rs` | 22 | None expected (Descriptor/TlvSection/OriginPath shapes stable; signatures stable) | Small |
| `src/parse_descriptor.rs` | 10 | Verify `build_multi_node` still emits Body::Variable correctly | Small (likely zero changes) |
| `src/template.rs` | 9 | Verify `wrapper_node` Body::Variable for multisig still encodes | Small-Medium |
| `src/cmd/bundle.rs` | 8 | `chunk::reassemble`, `compute_wallet_policy_id` — stable | Tiny |
| `src/cmd/verify_bundle.rs` | 4 | `chunk::reassemble` only — stable | Tiny |
| `src/format.rs` | 1 | `encode::render_codex32_grouped(s, 5)` — stable | Tiny |

**Conclusion: 80% of churn lives in `error.rs` + `friendly.rs`** (Phase 2). Everything else is verification + fixture regeneration.

## §3 Phasing

Per CLAUDE.md: per-phase TDD (test-then-impl), per-phase reviewer-loop until 0 critical / 0 important.

### Phase 1 — Dependency bump + capture compile-error set (Small)
**Deliverable**: `cargo check -p mnemonic-toolkit` ERROR set captured.
**Files**: `Cargo.toml` only.
- Change line 22 from `md-codec = { git = ..., tag = "md-codec-v0.16.1" }` to `md-codec = "0.33.1"`.
- **Do not bump mk-codec yet** (Phase 5).
- Run `cargo check` and capture the full compiler error set as a checkpoint.

**Decision: go straight to v0.33.1**. An intermediate stop at v0.19.0 is theoretically possible but yields no useful intermediate compile-green state — v0.19.0 only adds an error variant; v0.30.0 brings the wire-format break that forces fixture regeneration regardless. Stepping-stone doubles the work.

**Reviewer checkpoint**: skip (mechanical, single-line diff). Bundled with Phase 2 review.

### Phase 2 — error.rs + friendly.rs variant migration (Large)
**Deliverable**: `cargo build -p mnemonic-toolkit` succeeds.
**Files**: `error.rs`, `friendly.rs`.

**Tasks (TDD order)**:
1. **Test**: Extend `md_codec_inner_variant_routing` test (`error.rs:574`) with cells for the 7 new variants — `WireVersionMismatch` → exit 3 via `FutureFormat`; the other 6 → exit 2.
2. **Impl**:
   - In `From<md_codec::Error>` (`error.rs:463`), swap the `UnsupportedVersion { got }` arm for `WireVersionMismatch { got }` (retain "unsupported version" phrasing for CLI message stability).
   - In `md_codec_exit_code` (`error.rs:193`), delete the 5 exit-2/3 arms for the removed variants. Add exit-2 arms for the 6 new variants + exit-3 arm for `WireVersionMismatch`.
   - In `friendly_md_codec` (`friendly.rs:128`), parallel deletion + addition of match arms.
3. **Test**: Add a friendly-message cell verifying `WireVersionMismatch` flows through `From` → `FutureFormat` → `friendly_md_codec` is never reached for that variant.

**Reviewer checkpoint**: full opus dispatch. Verify SPEC §6.4.5 doesn't enumerate the deleted variants by name (if it does, SPEC patch needed in lockstep).

### Phase 3 — Body::Variable verification + multi-family pin (Small-Medium)
**Deliverable**: `cargo test -p mnemonic-toolkit --lib` green.
**Files**: `synthesize.rs`, `parse_descriptor.rs`, `template.rs`, `cmd/bundle.rs`, `cmd/verify_bundle.rs`, `format.rs`.

**Tasks**:
1. **Verification spike**: read `md-codec/src/encode.rs::write_node` HEAD; confirm `Body::Variable` is still accepted for `Tag::Multi/SortedMulti/MultiA/SortedMultiA`, OR rewrite `template.rs:151-213` (`wrapper_node`) and `parse_descriptor.rs:327-350` (`build_multi_node`) to emit `Body::MultiKeys { k, indices: 0..n }`.
2. Run all in-crate `#[test]` suites (`cargo test --lib`).
3. **Test**: add one new pin in `synthesize.rs::tests` that constructs a 2-of-3 `WshSortedMulti` Bundle and round-trips via `chunk::split` + `chunk::reassemble` + `compute_wallet_policy_id`.

**Reviewer checkpoint**: opus dispatch focused on the Body::Variable-vs-MultiKeys decision rationale + the new pin.

### Phase 4 — Integration test fixture regeneration (Medium)
**Deliverable**: `cargo test -p mnemonic-toolkit` green end-to-end.
**Files**: `tests/vectors/v0_1/*.txt` (16 fixtures consumed by `cli_bundle_full.rs`) + likely fixtures in `cli_bundle_multisig.rs`, `cli_descriptor_mode.rs`, `cli_verify_bundle_full.rs`, `cli_verify_bundle_watch_only.rs`.

**Tasks**:
1. **Pre-test**: `cargo test -p mnemonic-toolkit --tests` and capture failing fixture set.
2. Regenerate each failing fixture by running the offending command directly.
3. Manually inspect 1-2 regenerated fixtures to verify the v0.30 header byte appears.
4. Stage every regenerated fixture explicitly per CLAUDE.md stage-explicit rule. **No `git add -A`.**

**Reviewer checkpoint**: opus dispatch. Critical question: does any test decode md1 and assert on `md_codec::Error` Debug match?

### Phase 5 — mk-codec v0.2.1 → v0.3.0 fold-in (Small)
**Deliverable**: same as Phase 4, with mk-codec also on the new minor.
**Files**: `Cargo.toml` + whatever per-variant churn the mk-codec CHANGELOG documents.

**Tasks**:
1. Read `mnemonic-key/CHANGELOG.md` 0.2.1 → 0.3.0 entries.
2. Apply same error.rs / friendly.rs mapping discipline as Phase 2.
3. Regenerate mk1-dependent fixtures (Phase 4 likely already swept these).

**Reviewer checkpoint**: bundled with Phase 4 if drift is trivial; standalone opus dispatch otherwise.

### Phase 6 — Final integration + publish-readiness (Small)
**Deliverable**: full CI green; `cargo publish --dry-run -p mnemonic-toolkit` succeeds.
**Tasks**: bump toolkit version, update CHANGELOG, run `cargo publish --dry-run`, file the mnemonic-gui-side companion entry.

## §4 Test surface map

| Test | What it pins | Expected behavior under naive Phase-1 bump |
|---|---|---|
| `tests/cli_bundle_full.rs` | 16 byte-exact stdout fixtures | **FAIL** on every cell (v0.30 wire-format break) |
| `tests/cli_bundle_multisig.rs` | multisig fixture text | FAIL |
| `tests/cli_descriptor_mode.rs` | descriptor-mode round-trip | FAIL (md1 bytes change) |
| `tests/cli_verify_bundle_full.rs` | verify-bundle pass on full bundles | FAIL unless input fixtures regenerated |
| `tests/cli_self_check.rs` | bundle `--self-check` round-trip | likely PASS (decoder symmetric) |
| `tests/cli_bundle_json_intake.rs` | bundle `--json` round-trip | FAIL on md1 fields |
| `tests/cli_argv_leakage.rs`, `lint_*` | secret-handling discipline | PASS (no md-codec coupling) |
| `src/error.rs::tests` | exit-code routing per variant | FAIL pre-Phase-2 — covered by new test cells in Phase 2 |
| `src/synthesize.rs::tests` | cross-binding round-trip | PASS once Phase 2 compile-clean |

**New tests required**:
- Phase 2: 6 new exit-code routing cells.
- Phase 3: 1 multi-family Body::Variable round-trip pin.
- Optional Phase 6: end-to-end golden cell decoding a v0.16-era engraved md1 string and asserting `Err(WireVersionMismatch { got: 3 })`. Documents the forward-incompatibility contract.

## §5 Non-obvious risks

1. **SPEC §6.4.5 mentions deleted variants by name** — Toolkit's `error.rs:191` cites "§6.4.5 routing". SPEC almost certainly enumerates `ReservedHeaderBitSet`, `UnknownPrimaryTag`, `UnknownExtensionTag`, `UnsupportedDerivationShape`. **Companion SPEC patch needed.**
2. **`#[non_exhaustive]` regression risk** — `md_codec::Error` is not currently `#[non_exhaustive]`; if maintainers add it, toolkit's exhaustive match becomes a compile error. File FOLLOWUP `md-codec-error-non-exhaustive-monitor`.
3. **CosignerKeyInfo ResolvedSlot alias trap** — per MEMORY `[v0.10.1 patch closed]`, the alias has bitten reviewers twice. Phase 3 verification of `parse_descriptor.rs:11-21` MUST confirm the alias still points at `ResolvedSlot`.
4. **Workspace lock churn** — md-codec v0.30+ depends on `miniscript = workspace = optional = true` under the `derive` feature. Toolkit already pins `miniscript = "13"` — should be compatible.
5. **mlock G6 invariant** — `tests/mlock_g6_invariant.rs` is `#[ignore]`-gated per MEMORY entry; CI `--include-ignored` job must still pass.
6. **`encode::render_codex32_grouped` symbol stability** — confirmed present at HEAD `encode.rs:98`.
7. **GUI lockstep gate** — mnemonic-gui consumes only the CLI surface (not md-codec directly); risk low but confirm gui has no md1 byte snapshots of its own.
8. **Address-derivation feature default** — v0.32 introduced `default-features = ["derive"]` on md-codec. Toolkit will pick up the default. File a FOLLOWUP for awareness if toolkit ever sets `default-features = false`.
9. **Manual mirror invariant** — bumping md-codec means `md --help` output may have changed; confirm `docs/manual/src/40-cli-reference/` still mirrors current `md --help`.

## §6 Build sequence checklist

- [ ] Phase 1: Bump Cargo.toml line 22 to `md-codec = "0.33.1"`; `cargo check` to capture error set.
- [ ] Phase 2.a: Add 6 new exit-code routing test cells (RED).
- [ ] Phase 2.b: Rewrite `error.rs::md_codec_exit_code` + `From<md_codec::Error>` + `friendly.rs::friendly_md_codec` (GREEN).
- [ ] Phase 2 reviewer-loop.
- [ ] Phase 3.a: Verification spike — read `md-codec/src/encode.rs::write_node`; decide Body::Variable vs Body::MultiKeys.
- [ ] Phase 3.b: Add multi-family round-trip pin test.
- [ ] Phase 3.c: Apply template.rs / parse_descriptor.rs changes if needed; `cargo test --lib` green.
- [ ] Phase 3 reviewer-loop.
- [ ] Phase 4.a: Run `cargo test --tests`; enumerate failing fixtures.
- [ ] Phase 4.b: Regenerate failing fixtures; stage explicitly; commit with "vendored from md-codec-v0.33.1 wire-format break" in message.
- [ ] Phase 4 reviewer-loop (spot-check 2 regenerated fixtures for v=4 header byte).
- [ ] Phase 5: mk-codec 0.2.1 → 0.3.0 fold-in; second fixture-regen pass if needed.
- [ ] Phase 6: Toolkit version bump; CHANGELOG entry; `cargo publish --dry-run`; file mnemonic-gui companion FOLLOWUP.

## §7 Open question — fold mk-codec in?

**Recommendation: FOLD IN as Phase 5.** Justification:
- mk-codec is one minor bump (0.2.1 → 0.3.0); blast radius is smaller than md-codec by an order of magnitude.
- Splitting forces a second fixture-regeneration cycle on the next PR — wasted work.
- Both blockers must clear before mnemonic-toolkit can publish; a single reviewer-loop is cheaper than two.
- Risk mitigation: Phase 5 is **after** Phase 4 succeeds (md-codec already green), so the diff bisects naturally.
