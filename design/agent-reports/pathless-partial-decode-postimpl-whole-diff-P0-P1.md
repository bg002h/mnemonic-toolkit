# Post-impl whole-diff R0 — pathless partial-decode P0 (md-codec) + P1 (md-cli+BIP) — adversarial

**Persisted per CLAUDE.md** (+ opus fold at end). Combined P0+P1 uncommitted diff on descriptor-mnemonic `a39c9d9f` (12 modified + 5 untracked; ~735/49). Versions un-bumped (0.41.0/0.12.0). VERDICT: **OPEN (0C / 1I + 2M record-only).**

## Funds-load-bearing (all PASS, RED-proven)
- **Content-id oracle intact:** gated `ChunkSetIdMismatch` (`chunk.rs:410`) behind `!opts.allow_unresolved_origin` → `partial_decode_multi_chunk_dead_card_doctored_csid_still_rejects` FAILED (oracle-drop caught); reverted byte-clean; check is unconditional + AFTER the opts-honoring decode. HOLDS.
- **Strict-default byte-identity:** pre-change HEAD `md` vs current, 4 canonical shapes × decode/inspect × text/json/stderr/exit = byte-identical; dead shapes flip 1→4; all pre-existing `MissingExplicitOrigin` pins green.
- **No strict-consumer leak:** `allow_unresolved_origin:true`/`*_with_opts` ONLY in `cmd/decode.rs`+`cmd/inspect.rs`; `verify`/`bytecode`/`address`/`repair`(`decode_with_correction`) all strict; inspect policy-id gated at COMPUTATION (`Option<WalletPolicyId>`, never calls compute under partial). `cli_repair_dead_card_strict.rs` pins exit-2.
- **Empty-override fatal-in-partial:** `validate_no_empty_origin_overrides` unconditional before the origin match; partial swallows ONLY `MissingExplicitOrigin`; expand-side rejects independently.
- Seam (`cmd/partial.rs` shared marker/note), JSON contract (additive `partial`, `path_decl:"m"` single-rep, policy-id omitted, SCHEMA `md-cli/1`), BIP text — all PASS. Suites: md-codec 461/0, md-cli 265/0; clippy clean.

## Important (1)
**I-1 — `emit_pathless_advisory` (`md-cli/src/cmd/encode.rs:183-196`) keys on the wrong oracle.** Gates on `path_arg.is_none() && canonical_origin(tree).is_none()` instead of the final descriptor's resolvability. (a) FALSE POSITIVE: `md encode "sh(sortedmulti(2,@0/48'/0'/0'/1'/<0;1>/*,@1/…))"` (inline per-@N origins, no --path) → advisory fires claiming partial/exit-4, but the card FULL-decodes (exit 0) — never-misrepresent violation (safe-but-wrong direction). (b) FALSE NEGATIVE: `--path m` (0 components) → `path_arg.is_some()` early-return suppresses the advisory, but the card partial-decodes at exit 4 — the footgun bypassed. Fix: pass the post-`--path` `&descriptor`, gate on `!descriptor.unresolved_origin_indices().is_empty()`; +2 edge tests. Warn-only (no exit/byte/funds path) → Important, not Critical.

## Minor (record-only)
- **M-1:** `validate_no_empty_origin_overrides` (validate.rs:407) is a NEW pub fn beyond the SPEC's enumerated additive surface — list it in the 0.42.0 CHANGELOG/API ledger. (Handled at release.)
- **M-2:** `DecodeOpts` is a plain pub struct w/ pub field, no `#[non_exhaustive]` → a future option = SemVer-major. Add `#[non_exhaustive]` + Default before the crates.io freeze.

## VERDICT: OPEN (0C/1I) — fold I-1 + M-2, scoped convergence, THEN publish md-codec 0.42.0 + md-cli 0.13.0.

---
**FOLD STATUS (opus, 2026-07-11):** I-1 (advisory → `unresolved_origin_indices()` on final descriptor + 2 edge tests) + M-2 (`#[non_exhaustive]` DecodeOpts + Default + fix call sites) sent to the in-context P1 implementer. M-1 = a CHANGELOG line at release. Partial-decode plan P1.2 wording refined to the resolvability-oracle. Scoped convergence (Opus — fable exhausted) after the fold. Model note: this review completed on fable before the quota wall; all subsequent reviews → OPUS per user 2026-07-11.
