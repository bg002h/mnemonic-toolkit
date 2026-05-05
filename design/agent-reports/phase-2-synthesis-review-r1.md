# Phase 2 Synthesis Review — r1

**Date:** 2026-05-04
**Commit under review:** `38165fe` (parent: `d579459`)
**Reviewer:** opus phase-review

## Verdict

0 critical / 1 important / 6 low / nits

I-1 fixed inline in r1 fixup (test rename + comment); r2 will confirm 0C/0I.

## Critical

(none)

## Important

### I-1: `derive_passphrase_empty_string_equals_unset` test does not test what its name claims

**File:** `crates/mnemonic-toolkit/src/derive.rs:160–181`

The test name claims to verify SPEC §4.1 step 3 (`--passphrase "" ≡ unset`), but both calls pass the literal `""` — proving only `"" == ""`. The CLI-layer wiring (absent → `""`) is Phase 3.

**Fix (applied in r1 fixup):** Renamed to `derive_passphrase_empty_string_is_stable` with comment noting the §4.1 step 3 invariant is enforced at the CLI boundary in Phase 3. Test continues to pin determinism of the empty-string path.

## Low / Nit (defer to design/FOLLOWUPS.md)

- **L-1 (carryover from Phase 1 r1):** SPEC §5.5 omits `NetworkMismatch` / `FutureFormat` from `kind` table.
- **L-2 (carryover):** `chunk_mk1` fallback — corroborated by Phase 2 observed chunk counts (mk1=2 chunks, md1=3 chunks for BIP-84 single-sig).
- **L-3 (carryover):** md1 hyphens vs mk1 spaces.
- **L-4:** `debug_assert_eq!(&card.policy_id_stubs[0], &stub)` is tautological (stub was just passed in). Not a defect; release builds elide it. The meaningful assertion is `descriptor.is_wallet_policy()`.
- **L-5:** Plan source has stale 24-word fingerprint `73c5da0a` (should be `5436d724`). Patched in handoff during Task 2.1; plan source unpatched.
- **L-6:** ms1 not round-tripped in 16-cell test. SPEC §4.4 doesn't require synthesis-side round-trip; `full_bundle_emits_three_cards` confirms prefix. Phase 5 fixtures lock byte-exact output. Not a gap.

## Verified

- **§4.1 derive correctness:** 32-zero entropy, master fp `5436d724`, belt-and-braces network cross-check (defensive, never trips for full-mode), `bad_phrase` returns `Bip39(_)`.
- **§4.6.1 xpub-65 transform:** byte-exact `[0..32]=chain_code, [32..65]=compressed_pubkey`. Layout matches md_codec `identity.rs::deterministic_xpub` convention.
- **§4.6 build_descriptor:** `n=1`, `path_decl.n=1`, `PathDeclPaths::Shared`, `UseSitePath::standard_multipath()`, fp + xpub at placeholder index 0; all other tlv fields None/empty.
- **§4.5 mk1 synthesis:** `KeyCard::new(vec![stub], Some(fp), path, xpub)`; stub = `policy_id.as_bytes()[0..4]`; `origin_fingerprint = Some(_)` per §4.5.1.
- **§4.7 cross-binding:** invariants 1+2 debug-asserted; 16-cell round-trip test passes for all 4×4 = 16 cells (stub linkage, `is_wallet_policy()`, xpub, fingerprint).
- **§4.4 ms1 synthesis:** `ms_codec::encode(Tag::ENTR, &Payload::Entr(entropy))` returns `String`; stored as `Some(String)` in full mode, `None` in watch-only.
- **Error propagation:** `From` impls correctly intercept `ReservedTagNotEmittedInV01` / `UnsupportedVersion` → FutureFormat (exit 3); other variants fall through per inner-routing tables.
- **Chunk-count behavior:** mk1=2 chunks, md1=3 chunks for BIP-84 single-sig (corroborates Phase 1 spike memo).

## Smoke checks

- `cargo test -p mnemonic-toolkit`: 44 passed (32 Phase 1 + 7 derive + 5 synthesize)
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`: clean
- `cargo fmt --check -p mnemonic-toolkit`: clean
