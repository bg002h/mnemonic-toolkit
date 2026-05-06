# v0.6.1 Phase 0 SPEC review — r1 (architect: feature-dev:code-architect)

**Verdict:** 0 Critical / 3 Important / 3 Low / 2 Nit

## Critical

(none — initial C1 about `edge_uses_pbkdf2` was self-downgraded to non-issue: the SPEC §8 v0.6.1 invariant correctly mandates the predicate change Phase B will apply at `convert.rs:351-357`.)

## Important

- **I1 — `parse_descriptor.rs:946` cross-cut mis-guidance.** The SPEC §11 implementation-hooks paragraph listed `parse_descriptor.rs:946` (`bind_watch_only_singlesig`) as a site to "verify reachability." Verification confirms it's reachable only through `bind_descriptor_keys`, which has no production caller in `cmd/bundle.rs` post-v0.5.1. SPEC should direct Phase C to confirm-and-skip, not patch.
- **I2 — `Xpub::from_str` enumeration omits test/dead sites.** An implementer doing the exhaustive grep will find `Xpub::from_str` at `parse.rs:129/196` (dead post-v0.5 flag deletion), `parse_descriptor.rs:946` (dead production path), `parse_descriptor.rs:1632/1660` (test bodies), `parse_descriptor.rs:1702/1705/1708` (test fixtures). SPEC §11 needs explicit "no normalizer at" coverage for these or implementers will spend time grepping call chains.
- **I3 — `--xpub-prefix` missing from §5 grammar block.** §11.a documents the flag but §5's clap-grammar template doesn't list it; an implementer building the clap arg definition from §5 will miss the new flag.

## Low

- **L1 — `normalize_xpub_prefix` (SPEC) vs `normalize_slip0132_xpub` (spike memo) divergence.** Two names in the same review artifact; canonicalize to one.
- **L2 — `--network` row in §8 doesn't list wif edge.** The §2 edge description does mention `--network` for WIF, but the §8 summary table is inconsistent with it.
- **L3 — `mnemonic` binary-name pipe example.** Non-issue on inspection; downgraded to no-finding.

## Nit

- **N1 — SPEC-A amendment summary cites the SLIP-0132 spike but SPEC-A is unrelated.** SPEC-A's basis is the v0.6.0 convert-cycle spike (`spike-convert-v0_6_0-pre-spec.md`), not the SLIP-0132 spike.
- **N2 — 74-byte / 78-byte description in §11 is internally confusing.** The parenthetical opens by naming the 74-byte payload, then immediately invokes the 78-byte invariant; simplify.

## Action

All 3 Importants and the 2 actionable Lows + both Nits will be addressed in r2 before commit. L3 has no actual issue.
