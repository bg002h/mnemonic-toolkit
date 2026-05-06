# Phase A — BIP-388 conformance review — r1

**Date:** 2026-05-05
**Commit under review:** `bbae0a7`
**Reviewer:** feature-dev:code-reviewer (sonnet)
**Verdict:** APPROVE WITH NITS — 0C / 0I / 1L / 2N. Terminates the iterative-review loop per `feedback_iterative_review_every_phase` (no Critical or Important; convergence reached).

## Critical / Important

None.

## Low

**L-1 — `path.to_string()` normalization vs SPEC §4.11.b "raw user-supplied path string" — Phase B migration risk**

`crates/mnemonic-toolkit/src/parse_descriptor.rs:1049`

`check_key_vector_distinctness` compares `cs[i].path.to_string()` against `cs[j].path.to_string()` where `path: bitcoin::bip32::DerivationPath`. The bitcoin library normalizes hardening notation at `from_str` time: both `48h/0h/0h/2h` and `48'/0'/0'/2'` round-trip to `48'/0'/0'/2'`. SPEC §4.11.b reads "the raw path string as supplied by the user ... raw-string equality (no path canonicalization, no xpub network normalization)."

In Phase A all paths arrive through `DerivationPath::from_str` (lex_placeholders annotation parser; cosigner spec parser), so the normalization is safe — no false non-collision is possible, and the unit tests use `'`-notation throughout. The risk materializes in Phase B: `--slot @N.path=48h/0h/0h/2h` will supply the path as a raw CLI string. Phase B must explicitly decide whether `CosignerKeyInfo.path` stores a typed `DerivationPath` (normalizing) or a raw `String` (preserving). Either choice is valid; the decision must be locked before Phase B's end-of-phase review and SPEC §4.11.b must be updated to reflect the chosen normalization stance.

Routed to `design/FOLLOWUPS.md` as `bip388-distinctness-path-normalization-phase-b-decision` at tier `v0.4-nice-to-have`.

## Nits (resolved inline, not deferred)

**N-1 — Stale `SELF-MULTISIG WARNING` text in `synthesize.rs:176` doc-comment.** Falls under A.4 remit (audit-flagged stale comments); fixed inline in this commit.

**N-2 — Stale `#[allow(dead_code)]` and "Phase C.2/C.3" comment on `synthesize.rs:178::synthesize_descriptor`.** `synthesize_descriptor` is live (called from `descriptor_mode_run` at bundle.rs:1129). Fixed inline in this commit.

## Verified

**A.1 — Deletion completeness:** Grep over `crates/`: zero live references to `SELF_MULTISIG_WARNING` or `check_self_multisig_warning`. Surviving prose-only mentions in CHANGELOG/design docs are inert.

**A.2 — Algorithm vs SPEC §4.11.b:** Pairwise O(n²) scan; `(xpub.to_string(), path.to_string())` raw-equality; first-collision (i < j) returned. All 7 plan A.2 cases unit-tested.

**A.3 — Wire-up symmetry:**
- `descriptor_mode_run` (cmd/bundle.rs:1125-1126): `check_key_vector_distinctness(&binding)?` → `Bip388Distinctness {i,j}` → exit 2 + SPEC §6.6 row 13 stderr byte-exact (`cli_bip388_distinctness.rs` confirms via live binary).
- `descriptor_mode_verify_run` (cmd/verify_bundle.rs:1343-1344): `Err(Bip388Distinctness {..})` re-wrapped to `Bip388VerifyDistinctness` → exit 4 + SPEC §4.11.c text. Discarding `{i, j}` is correct — §4.11.c text references no slot indices.

**A.3 — Hard-reject placement in `bundle_multisig_full`:** Guard at bundle.rs:652 fires before phrase read / language warning / passphrase warning. No row 1-12 in `run()` independently rejects `cosigner_count > 1` ahead of this call. Stderr is clean (only the row-13 error emits).

**A.4 — Cleanups land:** L-3 (parse_descriptor.rs:725 stale comment trimmed), L-4 (CosignerKeyInfo `#[allow(dead_code)]` removed), L-7 (verify_bundle.rs dead lines removed) — confirmed.

**A.5 — Fixture corpus exclusions:** Two test files marked `#[ignore]`. No other test uses `--cosigner-count > 1`. Watch-only-multisig + descriptor-mode tests use distinct xpubs and are unaffected. (Reviewer initially missed three additional tests already marked `#[ignore]` by the implementer in `cli_account_flag.rs`, `cli_privacy_preserving.rs`, and `cli_self_check.rs`; total marked = 5, matching the impl plan A.5.)

**A.6 — SPEC §4.11 cross-check:** Exit codes, byte-exact stderr, raw-equality semantics, absent-path collision case (`DerivationPath::master()` → `"m"`) all align.

**Friendly mapper / JSON details:** `Bip388Distinctness` correctly routes through `message()` (not `friendly.rs`; not a sibling-codec wrapper). `details()` shape `{"i": i, "j": j}` consistent with `CosignerSpec` precedent. `Bip388VerifyDistinctness.details() = None` correct.

**Callers of `bundle_multisig_full`:** single call site at bundle.rs:317. No external plumb-through risk.

## Verdict

**APPROVE WITH NITS** — 0C / 0I / 1L / 2N. Phase A complete; Phase B green-light. L-1 routed to FOLLOWUPS; N-1 + N-2 fixed inline.
