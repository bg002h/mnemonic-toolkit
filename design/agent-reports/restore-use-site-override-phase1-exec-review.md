# #25 Phase 1 (md-codec + md-cli) execution review — per-key use-site override (opus architect, verbatim)

> Mandatory per-phase R0 over the IMPLEMENTED diff (`descriptor-mnemonic` `feature/per-key-use-site-override`, commits `0bd9088`+`ffe2fcc` off `c85cd49`). Reviewer ran tests/clippy/build + independently re-derived the funds-safety golden. **Verdict: GREEN — 0 Critical, 0 Important.** 3 non-blocking Minors. Cleared for P1.6 (version + publish 0.37.0/0.7.1).

---

**Verdict: GREEN — 0 Critical, 0 Important.**

## Verified correct

**Test runs (executed by reviewer):**
- `cargo test -p md-codec -p md-cli` → md-codec 214 lib + 49 proptest_to_miniscript + 21 wallet_policy + 21 address_derivation + 14 per_key_use_site_override + …, md-cli 106 unittests + 17 cmd_address + …, all `0 failed`. New `per_key_use_site_override.rs` 14/14; md-cli `cmd_address` 17/17 (3 new). Implementer's "392/202" aggregate consistent.
- `cargo clippy --all-targets -p md-codec -p md-cli` → clean.
- `cargo build -p md-codec --no-default-features` → clean (`to_miniscript`/`derive` correctly `#[cfg(feature="derive")]`-gated).
- `bitcoind_differential.rs` `#[ignore]`+env-gated (not run); reviewer independently recomputed its pinned golden — see I1.

**[I1 — the gate, NON-VACUOUS] (`per_key_use_site_override.rs:258-304`, `bitcoind_differential.rs:62`):** Divergent goldens computed OUTSIDE md-codec via rust-bitcoin `Xpub::derive_pub` + hand-assembled `multi(2,…)` witnessScript / `Address::p2wsh` (`leaf_pubkey:72-84`, no md-codec helper). Core P1.2 test asserts `@1` derives at its own `[2,0]` AND carries an anti-vacuity `assert_ne!(expected, wrong)` (`:300-303`). Reviewer independently re-derived the differential constant `DIVERGENT_WSH_MULTI_CHAIN0_IDX0_GOLDEN` (abandon mnemonic → `m/48'/0'/1'/2'` → `[2,0]`): `bc1qja66mak5p34f6fhc3z8lt5at5ndayx5z9h8734z0qc8qr27ly9jskzxxcu` — exact match; the baseline-collapse address (`bc1qpa7l8h70…csg0`) genuinely differs. Differential gates on `divergent_golden_asserted` so the shape can't silently drop. A baseline-collapse regression FAILS these tests.

**[P1.2 per-key VALUE] (`to_miniscript.rs:60-66`):** `build_descriptor_public_key(e, &e.use_site_path, chain)` — `e` is the per-`@N` `ExpandedKey`; `expand_per_at_n` (`canonicalize.rs:434-472`) pushes one record per `idx in 0..d.n` in order ⇒ Vec-position == `e.idx` == `@N`; `assemble_origin_and_xkey` keys origin/xpub/fp off the same `e`. `wildcard_for()` reads `use_site.wildcard_hardened`; for non-override cards `e.use_site_path == d.use_site_path` ⇒ existing corpus derivation byte-identical (21 address_derivation + wallet_policy + proptest pass).

**[P1.3 multipath builder] (`to_miniscript.rs:178-253`):** `build_descriptor_multi_public_key` emits `MultiXPub{DescriptorMultiXKey}` with one `DerivationPath` per alt (`<2;3>`→`[m/2],[m/3]`) from `e.use_site_path`, or single-path `XPub` for `None`. Shared `assemble_origin_and_xkey` byte-identical to single-path logic (extracted, not changed). `into_single_descriptors` + sortedmulti per-index sorting exercised by `multipath_builder_sortedmulti_divergent_independent_golden:464` — sorts per-key-DERIVED pubkeys (`pks.sort_by_key(|p| p.serialize())`), not lexicographic-on-xpub — matches.

**[P1.1 derive_address + I2] (`derive.rs:105-122`, `to_miniscript.rs:89-108,277-292`):** `has_hardened_use_site` scans baseline AND every override for `wildcard_hardened` OR any `Alternative.hardened` — truth table `[baseline-h, override-h-wildcard, override-h-alt, clean]→[T,T,T,F]`. Chain-range check survived (`:110-122`); real per-key backstop `use_site_to_derivation_path:282`. I2 hardened-alt reject UNTOUCHED at original locus (now `:287`), called ONLY by single-path builder NOT the multipath STRING builder — matches SPEC I2. No xpub-hardened-child path: all hardened cases route to `HardenedPublicDerivation` before derive.

**[P1.4 decode rejects] (`validate.rs:127-177`, `decode.rs:57-66`, `error.rs:189-209`):** `validate_use_site_overrides_canonical` rejects `idx==0` (`BaselineUseSiteOverride`, checked first) and `usp==baseline` (`RedundantUseSiteOverride`), wired into `decode_payload` inside the `if let Some(overrides)` guard. Tests confirm both fire on hand-crafted wire AND a genuine divergent / Some-None mix still decode. Two new variants placed by-domain adjacent to `MultipathAltCountMismatch`; md-codec `Error` verifiably NOT alphabetical (wire-order) ⇒ the CLAUDE.md alphabetical rule (toolkit `ToolkitError` only) does not apply. No exhaustive consumer `match` over `md_codec::Error` (md-cli wraps `CliError::Codec(md_codec::Error)`) ⇒ no unhandled-variant gap; md-cli suite compiles+passes.

**[P1.4 D5(b)] (`validate.rs:127-148`):** `validate_multipath_consistency` skips `None` entries ⇒ Some-baseline + None-override accepted (test `decode_accepts_some_baseline_none_override_mix`); only differing non-`None` alt-counts rejected.

**[Scope/fidelity]:** No restore.rs/toolkit/advisory files touched (Phase 2 deferred). No `unwrap`/`expect`/`panic`/`todo` in production md-codec source. `has_hardened_use_site` + `to_miniscript_descriptor_multipath` `pub` + re-exported `lib.rs:58-60` for Phase 2. The `DerivPaths::new(...).ok_or(...)` empty-group case (`:198`) unreachable on real cards (`AltCountOutOfRange` enforces ≥2 at wire) but defensively handled, not a panic.

## CRITICAL
None.

## IMPORTANT
None.

## MINOR
1. **`some_none` mix tested only at chain 0** (`per_key_use_site_override.rs:342`). The `None`-override key correctly ignores `chain`, so `derive_address(1,…)` is well-defined and safe — but no test pins chain-1 behavior of the bare-`/*` key alongside a multipath baseline. Correct by reasoning; a one-line chain-1 assertion closes the coverage gap. Non-blocking.
2. **`multipath_builder_some_none_mix_string` counts `<` characters** (`:451`) as a proxy for "exactly one multipath group." Robust here, brittle if rendering ever introduces `<` elsewhere. The address-equivalence test is the real anchor; cosmetic. Non-blocking.
3. **P1.6 not done (by design):** versions remain 0.36.0/0.7.0, no 0.37.0 CHANGELOG, md-cli dep `=0.36.0`. Explicit P1.6 deferral — flagging so the publish (version ×2 + dep pin + CHANGELOG 2 entries + crates.io publish) is not forgotten before Phase 2 pins 0.37.0.

## To turn GREEN
Already GREEN. May proceed to P1.6 (version + CHANGELOG + ship + publish md-codec 0.37.0 / md-cli 0.7.1). The three MINORs are optional polish, not gates.
