# v0.6.1 Phase C code review — r1 (reviewer: feature-dev:code-reviewer)

**Verdict:** APPROVED 0 Critical / 0 Important.

## Source-cross-checks the reviewer ran

- **SLIP-0132 prefix bytes** — all 8 variant bytes in `swap_target_for` (`src/slip0132.rs:75-85`) and the normalizer match table (`src/slip0132.rs:116-124`) confirmed against [satoshilabs/slips/slip-0132.md](https://github.com/satoshilabs/slips/blob/master/slip-0132.md).
- **`(Xpub, Xpub)` edge addition — refusal-bypass risk:** None. The §3.a refusal list never included `xpub → xpub`; the catch-all `!is_supported_direct_edge` was previously emitting a *false* one-way refusal. The new edge is a genuine fix. `classify_edge` special-cases `xpub→mk1` and sibling pivots before the catch-all; neither is disturbed.
- **Post-compute `--xpub-prefix` swap on compound `--to`:** `convert.rs:414-420` iterates `outputs.iter_mut()` guarded on `*node == NodeType::Xpub`; compound `--to xpub,fingerprint` only mutates the xpub entry.
- **Cross-cut completeness:** exactly 3 production `Xpub::from_str` sites exist (`bundle.rs:329`, `bundle.rs:857`, `convert.rs:581`); all are preceded by `normalize_xpub_prefix`. `verify_bundle.rs` has no direct xpub parse calls; all 3 of its dispatch paths route through `cmd::bundle::resolve_slots` which contains the normalizer. Transitive coverage matches SPEC §11.
- **`wif → xpub` + `--xpub-prefix`:** technically permitted (the sentinel xpub is version-swapped); SPEC §11.a does not carve out the WIF sentinel case, so the behavior is SPEC-conformant.
- **SPEC §2 edge table addition:** internally consistent with §11/§11.a; the `(with --xpub-prefix: --network required)` side-input note matches the refusal enforcement at `convert.rs:371-375`.

## Test coverage

9 lib unit tests (`src/slip0132.rs::tests`) + 15 CLI integration tests (`tests/cli_convert_slip0132.rs`) + 1 template-mode bundle cross-cut + 1 descriptor-mode bundle cross-cut. Covers normalizer, all 5 output variants, 2 testnet variants, the refusal, the silent-ignore, the round-trip, and both `bundle` cross-cuts. No SPEC-claimed-behavior gaps.

## Nits (deferred — none rise to FOLLOWUPS-worthy)

- `src/slip0132.rs:93`: `raw.clone()` — minor; current form is clearer than in-place mutation.
- `convert.rs:424`: `// 8) Emit` numbering already tracked at FOLLOWUPS `convert-run-step-numbering-duplicate-8` (logged in Phase B); the new `// 8.a)` label adds to the same comment-numbering churn, which the FOLLOWUP will close in v0.6.2+.
- `tests/cli_convert_slip0132.rs`: dual fingerprint constants with clarifying comments — descriptive; no improvement needed.

## Cleared for Phase C commit

`cargo test --workspace` reports 239 lib + integration tests pass; +9 lib + 17 integration tests added in this phase.
