# SPEC Convergence R0 — SPEC_toolkit_repin_sh_wpkh.md (round 3) — Opus

**Persisted per CLAUDE.md.** Verified vs `7ffce786`. VERDICT: **GREEN (0C/0I)** — all round-2 folds complete, correct, drift-free. Sound to advance to the plan phase.

## Verified
- **I1 (8-consumer audit COMPLETE):** independent grep of `canonical_origin::canonical_origin(` in `src/` = 10 sites; 8 production (map 1:1 to the table rows: `bundle.rs:1418`, `verify_bundle.rs:1414`, `restore.rs:1645`, `synthesize.rs:1201`, `bundle.rs:1199`, `restore.rs:319`, `verify_bundle.rs:391`, `gui_schema.rs:1320`) + 2 in `#[cfg(test)]` (`synthesize.rs:2413/:2455`, correctly excluded). No 9th consumer. Both new rows exist at cited lines; `restore.rs:317-320` + `verify_bundle.rs:389-392` are the `is_singlesig_template` gates with the neutralizing `&& cli_template_from_tree(&d.tree).is_some()` conjunct (320/392) — no-`Sh`-arm → FALSE for sh(wpkh) → gate stays FALSE regardless of flip. Acceptance #2 names the verify-side guard.
- **M-a:** F-A9 no-op accurate (`friendly.rs:376-379` re-derives its own message from `{chunk_index, bound}`, independent of md-codec Display).
- **M-b:** exactly 3 `decode_md1_string` calls (prop_repair_never_wrong.rs:483, cli_inspect.rs:234, cli_repair_md1_non_chunked.rs:126), all non-chunked. Correct.
- **M-c:** only `41-mnemonic.md:418` has "five … wrapper shapes" (the mandatory edit); `:469` is display-grouping (untouched); ~2989-3005 has no "five" string. Correct.
- **No drift:** "6-consumer" appears only in the round-1 Status history narration; all live claims = 8; table = 8 rows. The two "THREE" usages distinct + coherent. F-A9 coexists coherently with "THREE headline deltas" (framed as an also-a-message-correction no-op).

**GREEN (0C/0I) — advance to plan phase.**

---
**STATUS (opus, 2026-07-11):** Track B SPEC R0 loop CONVERGED GREEN (C1/M4 correctly rejected as contamination; I1 8-consumer audit + verify guard + M-a/M-b/M-c folded). Next: Track B plan-doc → plan R0 (Fable).
