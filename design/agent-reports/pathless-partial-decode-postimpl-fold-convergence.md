# Scoped convergence — pathless partial-decode post-impl fold (I-1 + M-2) — Opus

**Persisted per CLAUDE.md.** VERDICT: **GREEN (0C / 0I).** P0+P1 sound to publish (md-codec 0.42.0 + md-cli 0.13.0).

## Mutation proof (conclusive)
Reverted `emit_pathless_advisory` to the OLD heuristic (`path_arg.is_none() && canonical_origin(&tree).is_none()`) → `tests/cli_pathless_encode_advisory.rs`:
- `inline_per_at_n_origins_no_path_full_decodes_and_is_not_warned` FAILED (advisory wrongly fired on a card decoding at exit 0 — the never-misrepresent false-positive).
- `path_m_zero_components_on_dead_shape_still_warns` FAILED (`--path m` suppressed the advisory on a card partial-decoding at exit 4 — the false-negative bypass).
Both guard tests genuinely RED under the old heuristic → they truly guard the fix. Restored byte-clean (encode.rs sha256 matches; `git diff --stat` unchanged).

## Confirmed
Advisory ⟺ decode-exits-4 on the binary (dead/no-path → advisory+exit4; canonical → none+exit0; inline-per-@N/no-path → none+exit0; `--path m`/dead → advisory+exit4); keys on `unresolved_origin_indices()` on the FINAL post-`--path` descriptor — same query decode/inspect use. `DecodeOpts` `#[non_exhaustive]` + Default + `partial()`; no downstream struct-literal; md-cli uses `partial()`, strict uses `default()`. md-codec 461/0, md-cli 280/0; clippy clean; versions un-bumped.
Minor (release-time): M-1 — list `validate_no_empty_origin_overrides` (new pub) in the 0.42.0 CHANGELOG/API ledger.

## VERDICT: GREEN — Track A P0+P1 (partial-decode) sound to publish md-codec 0.42.0 + md-cli 0.13.0.

---
**STATUS (opus, 2026-07-11):** Track A partial-decode P0+P1 CONVERGED GREEN through the full R0 pipeline (P0 per-phase → P1 per-phase → post-impl whole-diff → I-1/M-2 fold → scoped convergence, all mutation-proven). Ready for the descriptor-mnemonic release ritual (md-codec 0.42.0 + md-cli 0.13.0 lockstep publish). Code uncommitted in descriptor-mnemonic working tree.
