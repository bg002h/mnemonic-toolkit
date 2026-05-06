# v0.6.1 Phase 0 SPEC review — r2 (architect: feature-dev:code-architect)

**Verdict:** APPROVED 0 Critical / 0 Important.

## r1 closure verification

| Finding | Resolution | Verified at |
|---------|------------|-------------|
| I1 — dead `parse_descriptor.rs:946` cross-cut | Documented under §11 "No normalizer call needed at" with dead-code rationale | `SPEC_convert_v0_6.md:296-300` |
| I2 — `Xpub::from_str` enumeration | Full production-site list (4) + dead/test-site exclusion list | `SPEC_convert_v0_6.md:289-300` |
| I3 — `--xpub-prefix` missing from §5 | Added to grammar block with §11.a comment | `SPEC_convert_v0_6.md:159` |
| L1 — function-name divergence | Spike memo annotated with SPEC's canonical name `normalize_xpub_prefix` | `spike-slip0132-v0_6_1-pre-spec.md:50` |
| L2 — `--network` row in §8 | Extended to cover wif emission + `--xpub-prefix` selection | `SPEC_convert_v0_6.md:224` |
| N1 — SPEC-A spike citation | Replaced wrong SLIP-0132 spike citation with v0.6.0 convert-cycle spike + "no spike required" framing | `SPEC_convert_v0_6.md:14` |
| N2 — 74-byte / 78-byte description | Cleanly split: 78-byte raw buffer = 4-byte version + 74-byte payload; invariant `raw.len() == 78` | `SPEC_convert_v0_6.md:284` |

## Source-verification cross-checks (architect-side)

- `verify_bundle.rs` grep'd for `Xpub::from_str` at lines 208/259/334/403 — none present; coverage is transitive via `bundle::resolve_slots`. Confirms §11 "no `Xpub::from_str` post-v0.5.1" claim.
- `bind_descriptor_keys` callers: only test sites (lines 1530/1550/1569/1589/1610) + a `synthesize.rs:186` documentation comment. No production caller. Confirms §11 dead-site claim for `parse_descriptor.rs:946`.

## No new findings

The two staged SPEC files are internally consistent and source-verified.

## Cleared for Phase 0 commit

Proceed with the SPEC commit + bundled spike-memo function-name annotation.
