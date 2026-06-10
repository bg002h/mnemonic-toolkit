# R0 round-2 architect review — PLAN_secret_flag_gate_non_circular (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 2, post-fold verification). Plan @ design/PLAN_secret_flag_gate_non_circular.md, source e2a09ba. Verdict: GREEN (0 Critical / 0 Important / 1 new Minor M-5). Review verbatim below.

---

## Critical

None.

## Important

None.

## Minor

**M-5 (NEW) — The stdin-toggle instance tally is 25, not 26.** Exhaustive enumeration of the live v0.53.0 `gui-schema` (all 30 subcommands, every `flags[].name` ending `-stdin`, all boolean, none in `cli`/`meta`/`positionals`): `--passphrase-stdin` ×12, `--ms1-stdin` ×3, `--phrase-stdin` ×3, `--decrypt-password-stdin` ×2, `--secret-stdin` ×2, `--bip38-passphrase-stdin` ×1, `--message-stdin` ×1, `--xpub-stdin` ×1 = **25 instances**. (Raw `grep -c` of the JSON over-counts via `conflicts_with` arrays — likely the round-1 error source.) The error originates in round-1's own I-2 text ("26 instances"), which the fold reproduced faithfully — so this is not fold-drift, but the plan now presents a wrong number as a verified census at plan line 30. No design impact: Cell 2 enumerates dynamically, the plan never instructs freezing an instance count, and the load-bearing facts (8 distinct names; every toggle has a same-subcommand base; the two non-secret pairs) all verify. Fix: change "26 instances" → "25 instances" at plan :30. One-token edit; fold without re-dispatch.

## Fold-verification

**I-1 (RED-matrix mis-attribution) — FOLDED-OK, and empirically re-verified.** Plan :35 now reads Cell 1 RED on `--phrase` ×3 / Cell 2 RED on `--ms1-stdin` ×3 ONLY / Cell 3 RED on the 3-name set diff, with the explicit "`--phrase-stdin` is GREEN pre-D1 … covered ONLY by Cell 3 … Do NOT distort Cell 2" warning, and Phase 1 (:64) repeats the corrected matrix. Live re-verification against the v0.53.0 binary: Cell-2 mismatches are exactly `(--ms1-stdin toggle=false vs --ms1 base=true)` on the 3 xpub-search modes; all three `--phrase-stdin` pairs are false==false (GREEN); Cell-1 simulation of the plan's exact net over kinds ∉ {path, number, boolean} yields exactly 3 REDs, all `--phrase` (text) on the 3 modes; live distinct `secret:true` names = 11 (so Cell 3 11-vs-14 REDs as stated).

**I-2 (stdin census 6→8) — FOLDED-OK except the instance tally (→ M-5).** Plan :30 carries all 8 distinct names including `--message-stdin` (verified live: `verify-message`, base `--message` present, both non-secret) and `--xpub-stdin` (verified live: `xpub-search-address-of-xpub`, base `--xpub` present, both non-secret); states the two non-secret pairs are in Cell 2's domain by design; requires the `(subcommand, toggle, base)` triple in the failure message; "all 8 have a same-subcommand base" verified (base_exists=true for all 25 instances). Only the "26 instances" figure fails verification (25) — see M-5; round-1 carried the same error, so the fold is faithful.

**I-3 (vacuous evidence anchors) — FOLDED-OK.** Plan :42-47 uses `&["pub phrase_stdin", "fn phrase_stdin"]` on all 3 rows and explains the suffix trap. Re-verified at e2a09ba: `pub phrase_stdin: bool` at `crates/mnemonic-toolkit/src/cmd/xpub_search/path_of_xpub.rs:45`, `passphrase_of_xpub.rs:63`, `account_of_descriptor.rs:46`; `fn phrase_stdin` at :124/:155/:136 — exactly the plan's cited lines. Neither needle is a substring of any `passphrase` token (`pub passphrase_stdin` / a hypothetical `fn passphrase_stdin` do not contain them); both vanish if the `--phrase-stdin` wiring is deleted, so they discriminate under the plain `source.contains` check (verified at `tests/lint_argv_secret_flags.rs:240`). Dropping round-1's third example needle is fine — the check is `.any()` and round-1's list was illustrative. Bonus re-check: axis-1 derives only `secret==true && kind!="boolean"` (`lint_argv_secret_flags.rs:157`, doc :21-22), confirming the plan's claim that the two boolean flips need NO new Route rows and exactly 3 `--phrase` rows restore set-equality.

**M-1 (bind to existing GUI entries + ms.rs scope honesty) — FOLDED-OK.** Plan :52 says do NOT file a fresh third GUI-side entry; add `Companion:` lines to the two existing open entries — line numbers re-verified: `xpub-search-inline-phrase-not-secret-classified` at `mnemonic-gui/FOLLOWUPS.md:73`, `ms-repair-ms1-not-secret-classified` at `:81`, both `Status: open`. Scope honesty present both in the header Unblocks line (:5) and in D4 (:52); the toolkit's own `repair --ms1` is indeed already secret (`--ms1` in the `matches!`, confirmed live).

**M-2 (9 GUI sites not 3) — FOLDED-OK.** Plan :52 states 9 hand-coded `secret:` sites = 3 flags × 3 subcommands and that audit I4's :2286/2448/2718 are the `--phrase` entries only. Re-verified in `mnemonic-gui/src/schema/mnemonic.rs`: `--phrase` at :2280/:2442/:2712 (their `secret: false` lines are exactly :2286/:2448/:2718), `--phrase-stdin` at :2291/:2453/:2723, `--ms1-stdin` at :2312/:2474/:2744 — 9 sites.

**M-3 (`priv` speculative vocabulary) — FOLDED-OK.** Plan :29 keeps `priv` with the required cell-comment instruction ("currently speculative vocabulary … `--privacy-preserving` is boolean-excluded"). Cell-1 simulation confirms `priv` matches nothing in the included kinds today.

**M-4 (no-leading-dashes row exists) — FOLDED-OK.** Plan :21 now says the existing row is "kept as-is"; re-verified at `crates/mnemonic-toolkit/src/secrets.rs:120` (`assert!(!flag_is_secret("passphrase"))`).

## Additional re-checks (clean)

- D1 placement instruction is consistent with the current `matches!` list (`secrets.rs:49-64` order: …`--ms1`, `--secret`…) — inserting `--ms1-stdin` after `--ms1` and the two `phrase` names before `--secret` preserves the local ordering convention.
- Frozen-literal arithmetic: 11 live distinct `secret:true` names + 3 = 14 ✓.
- `:284` cell (`secret_flag_enumeration_matches_authoritative_predicate`) and `gui_schema.rs:1196` tautology cites still accurate at e2a09ba; both FOLLOWUPS backlog ids present (`design/FOLLOWUPS.md` lines 15 and 28).
- Route-row style (relative `source_file` paths, subcommand naming) matches existing `FLAG_ROUTES` rows.
- No new issue introduced by the folds beyond M-5; full plan re-scan found nothing round 1 missed.

## Verdict

**GREEN** — 0 Critical / 0 Important / 1 new Minor (M-5: census instance tally 26 → 25 at plan :30; one-token fold, no re-dispatch needed). The R0 gate is satisfied; implementation may begin after folding M-5.
