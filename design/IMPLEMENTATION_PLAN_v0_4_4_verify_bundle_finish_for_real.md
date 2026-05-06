# mnemonic-toolkit v0.4.4 implementation plan — verify-bundle finish for real

**Cycle scope:** close the 2 v0.4.4-tier FOLLOWUPS. Theme: **finish verify-bundle for real** — land the `emit_verify_checks` helper, refactor all 3 run_* entry points to use it, populate forensic fields at every push site, achieve descriptor-mode 9/3+6N parity, retire the redundant `DescriptorBinding.entropy` field.

**Authoritative SPEC:** `design/SPEC_mnemonic_toolkit_v0_4.md` esp §5.7 (verify-bundle 9 / 3+6N + per-cell forensics). v0.4.4 IMPLEMENTS what §5.7 already specifies; no SPEC drift expected. Any drift surfaces during architect review and is amended in lockstep.

**Discipline (per `feedback_iterative_review_every_phase`):**
- Per-phase architect review at end-of-phase; iterate to 0C/0I.
- Per-implementation-phase reports persist to `design/agent-reports/phase-<id>-<slug>-review-r<N>.md`.
- L/nit findings → `design/FOLLOWUPS.md` at `v0.4.5-nice-to-have`.
- TDD-first per phase where practical; mechanical refactors skip the red phase.

## Locked decisions (user-confirmed defaults)

- **Q1 (Phase P granularity): (a) helper-first.** Write `emit_verify_checks(expected, supplied, is_multisig) -> Vec<VerifyCheck>`, validate against existing checks via tests, then refactor each `run_*` in turn to call it. Helper is the architectural shape SPEC §5.7 mandates; landing it last would invert the design.
- **Phase ordering: P → S.** Phase S (descriptor-binding entropy retirement) is independent of P but lands after to avoid confusion during the bigger refactor.
- **Bundles `verify-bundle-9-3plus6n-descriptor-mode-parity`** (separate FOLLOWUP from v0.4.2-tier) into Phase P automatically — descriptor_mode_verify_run shares the helper, so descriptor mode gains 9/3+6N parity for free.

## Phase ordering

```
Phase P (verify-bundle helper + full forensics + descriptor parity)
  ↓
Phase S (DescriptorBinding.entropy field retirement)
  ↓
Cleanup + Release
```

## Phase P — verify-bundle helper + full forensics + descriptor parity

**Goal:** sole verify-bundle check-emission path is `emit_verify_checks(expected: &Bundle, supplied: &SuppliedCards, is_multisig: bool) -> Vec<VerifyCheck>`. All 3 run_* entry points (run_full, run_multisig, descriptor_mode_verify_run) become thin wrappers that call the helper. Forensic fields populated at every fail-path. Descriptor mode emits the same 9 / 3+6N schema as template mode.

Closes FOLLOWUPs `verify-bundle-helper-and-full-forensics-rollout-v0.4.4` + `verify-bundle-9-3plus6n-descriptor-mode-parity` (the latter is an automatic byproduct).

### P.1 — `SuppliedCards` struct + `emit_verify_checks` helper signature

```rust
/// User-supplied --ms1/--mk1/--md1 vectors packaged for the helper.
pub struct SuppliedCards<'a> {
    /// Per-slot ms1; len(ms1) == n; "" sentinel for watch-only slots.
    pub ms1: &'a [String],
    /// Per-slot mk1; flat for n=1 single-sig; per-cosigner-flattened for n>1.
    pub mk1: &'a [String],
    /// Chunked md1; one logical card always (multi-chunk allowed).
    pub md1: &'a [String],
}

pub fn emit_verify_checks(
    expected: &Bundle,
    supplied: &SuppliedCards,
    is_multisig: bool,
) -> Vec<VerifyCheck>
```

`is_multisig` discriminates the schema: false → 9-check single-sig; true → 3 + 6N multisig.

Per-slot watch-only inferred from `expected.ms1[i].is_empty()` (no separate mode parameter needed).

### P.2 — Helper internal logic — schema + forensics

Single-sig (`is_multisig: false`) emits 9 checks in order:
1. `ms1_decode` — decode supplied ms1[0]; populate `decode_error` on Err.
2. `ms1_entropy_match` — if ms1 decodes, compare to expected.ms1[0]; populate expected/actual/diff_byte_offset on string mismatch.
3. `mk1_decode` — decode supplied mk1; populate decode_error on Err.
4. `mk1_xpub_match` — extract xpub from decoded mk1; compare to expected card's xpub.
5. `mk1_fingerprint_match` — compare fingerprint.
6. `mk1_path_match` — compare path string.
7. `md1_decode` — reassemble + decode supplied md1.
8. `md1_wallet_policy` — verify is_wallet_policy.
9. `md1_xpub_match` — verify the per-slot xpub matches what the descriptor binds.

Watch-only short-circuit: if `expected.ms1[0].is_empty()`, ms1_decode + ms1_entropy_match emit `passed: true` + `decode_error: Some("skipped: watch-only slot")` + all forensic fields None.

Multisig (`is_multisig: true`) emits 3 shared + 6 per-cosigner:
- 3 shared: md1_decode + md1_wallet_policy + md1_xpub_match (deduplicated; descriptor binds all xpubs in one card).
- 6 per cosigner @i: ms1_decode[i] + ms1_entropy_match[i] + mk1_decode[i] + mk1_xpub_match[i] + mk1_fingerprint_match[i] + mk1_path_match[i].

Per-slot watch-only short-circuit (same as single-sig but per-slot): `expected.ms1[i].is_empty()` → ms1[i] checks pass-vacuously.

**Forensic field rules** (SPEC §5.7):
- **Pass cases**: all forensic fields None.
- **String-mismatch** (mk1_xpub_match, mk1_fingerprint_match, mk1_path_match, ms1_entropy_match, md1_xpub_match): populate `expected: Some(...), actual: Some(...), diff_byte_offset: Some(VerifyCheck::diff_offset(exp, act))`. If lengths differ, diff_byte_offset = `min(len_exp, len_act)`.
- **Decode-failure** (ms1_decode, mk1_decode, md1_decode, md1_wallet_policy): populate `decode_error: Some(<error_text>)`.
- **Watch-only skipped**: `passed: true` + `decode_error: Some("skipped: watch-only slot")` + all other forensic fields None.

**Wif-slot handling (per r1 review L-1):** wif slots produce `expected.ms1[i] = ""` in the emitted bundle (per v0.4.2 K.3 wif resolution emits empty-string sentinel). The watch-only short-circuit (`expected.ms1[i].is_empty()`) covers wif slots without special handling — `ms1_decode[i]` and `ms1_entropy_match[i]` pass-vacuously per SPEC §5.7. The mk1 checks for wif slots run normally against the wif's compressed pubkey (depth-0 xpub).

**`SuppliedCards.mk1` indexing convention (per r1 review L-2):** `mk1[i]` is the mk1 card for cosigner `i`, 0-indexed, with `len(mk1) == N` expected. CLI `--mk1 X --mk1 Y --mk1 Z` for N=3 multisig produces a Vec<String> where `mk1[0]` is cosigner @0's card, `mk1[1]` is @1's, etc. Single-sig: `len(mk1) == 1`. Helper assumes the pre-check ladder (existing) has already validated the count; if `mk1.len() != N` mismatch surfaces as a `mk1_decode[i]` Err for the missing index.

### P.3 — Refactor `run_full` to use helper

Currently ~600 lines of inline check generation (50 push sites). After refactor: ~80 lines.

Sequence:
1. Decode supplied cards (no — that's the helper's job; pass raw `--ms1` / `--mk1` / `--md1` through SuppliedCards).
2. Re-derive expected Bundle from `--phrase` / `--xpub` / `--cosigner` / `--slot` flags (existing logic).
3. Build `SuppliedCards { ms1: &args.ms1, mk1: &args.mk1, md1: &args.md1 }`.
4. `let checks = emit_verify_checks(&expected, &supplied, false);`
5. Compute `result = if checks.iter().all(|c| c.passed) { "ok" } else { "mismatch" }`.
6. Print `VerifyBundleJson { schema_version: "4", result, checks }` per `--json` flag.

### P.4 — Refactor `run_multisig` to use helper

Same shape as P.3; pass `is_multisig: true`. Currently ~700 lines, ~25 push sites; after refactor ~80 lines.

### P.5 — Refactor `descriptor_mode_verify_run` to use helper

This is where the SPEC §5.7 descriptor 9/3+6N parity is gained: descriptor_mode_verify_run currently emits the v0.3 3-element coarse ladder (~6 push sites). After refactor → calls `emit_verify_checks` like the other run_*'s, gaining the full 9/3+6N schema for free.

Currently ~150 lines + 6 push sites. After refactor ~70 lines.

### P.6 — Watch-only test module migration

`cmd/verify_bundle.rs::watch_only_tests` mod has ~30 unit-test assertions on the legacy 3-element ladder shape. Migration: keep the helper-driven flow; assertions update to the 9-check / 3+6N count.

### P.7 — Tests

- Unit tests on `emit_verify_checks` directly (independent of run_* binding) — cover all 9-check shapes for single-sig + 3+6N for multisig + watch-only short-circuit + forensic field population.
- Integration tests for descriptor-mode 3+6N parity (currently asserts 3-element).
- Integration test that asserts forensic field population on a tampered-mk1 bundle (already exists; expand assertions to include diff_byte_offset).

**Phase P architect review:** mid-phase after P.2 (helper API + internal logic) + end-of-phase after P.7.

## Phase S — `DescriptorBinding.entropy` field retirement

**Goal:** delete the redundant `entropy: Option<Vec<u8>>` field from `DescriptorBinding`; per-slot entropy lives on `binding.cosigners[i].entropy` after v0.4.3 N. Closes FOLLOWUP `descriptor-binding-entropy-field-redundant`.

### S.1 — Delete the field

`crates/mnemonic-toolkit/src/parse_descriptor.rs::DescriptorBinding`: remove `entropy` field.

### S.2 — Update bind_full_mode + bind_watch_only_singlesig + bind_watch_only_multisig

These functions currently set `binding.entropy = Some(<bytes>)` in full mode. After S.1, they set `binding.cosigners[0].entropy = Some(<bytes>)` directly (the @0 slot only; @1+ cosigners are watch-only by definition).

### S.3 — Update callers

~10 call sites read `binding.entropy.as_deref()`:
- `cmd/verify_bundle.rs:1430` (descriptor_mode_verify_run) → `binding.cosigners.first().and_then(|c| c.entropy.as_deref())`.
- `cmd/bundle.rs::bundle_run_unified_descriptor` (already calls `synthesize_descriptor` with explicit entropy_at_0; verify pattern matches).
- `parse_descriptor.rs` test functions (~5 sites): same migration.
- `parse_descriptor.rs::tests::self_multisig_warning_*` legacy tests (deleted in v0.4.0; verify they're gone).

### S.4 — Update construction sites

`bind_full_mode`'s final `Ok(DescriptorBinding { keys, fingerprints, cosigners, entropy: Some(entropy_bytes) })` → set `cosigners[0].entropy = Some(entropy_bytes)` before the binding is constructed.

### S.5 — Tests

Existing descriptor-mode tests should pass unchanged (they test behavior, not the binding shape). Update tests that assert on `binding.entropy` directly (~5 sites in parse_descriptor.rs).

**Phase S architect review:** end-of-phase only. Mechanical refactor with bounded surface.

## Cleanup + Release (post-Phase S)

Final architect review across all phases (transcript-only). CHANGELOG v0.4.4 entry. Tag `mnemonic-toolkit-v0.4.4`. GitHub release.

`cargo publish` for the toolkit remains gated on ms-codec / mk-codec / md-codec landing on crates.io. v0.4.4 distributed via GitHub tag only.

## Test impact summary

- Phase P: ~1500 lines deleted from verify_bundle.rs; ~3 new run_* thin wrappers (~250 lines total); ~1 new helper (~400 lines incl. shared logic). Net delete: ~850 lines.
- Phase P: ~30 watch_only_tests assertions migrated; +5 new helper unit tests; +2 descriptor-mode 9/3+6N integration tests; +1 forensic field assertion expansion.
- Phase S: ~10 call sites mechanically updated; ~5 test assertions updated.

Estimated post-v0.4.4: ~245-250 lib + integration tests; verify_bundle.rs collapses from 1859 → ~1000 lines.

## Out of scope (deferred to v0.4.5+)

- `bundle-json-schema-2-3-retro-compat` (v0.4.5+; gated on real need).
- `unified-slot-xpub-missing-path-origin-path-null` (v0.4.5-nice-to-have; cosmetic).
- v0.4-nice-to-have trap-bypass nits (vanishingly unlikely user paths).
- v0.5: `legacy-cli-flag-deletion`.
- v0.5+: `unified-slot-xprv-resolution-needs-ms-codec-extension`.
