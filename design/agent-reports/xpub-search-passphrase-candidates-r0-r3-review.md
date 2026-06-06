# R0 Architect Re-Review (round 3) — SPEC_xpub_search_passphrase_candidates_file.md

**Reviewer:** opus `feature-dev:code-reviewer`. **Date:** 2026-06-05. **Branch:** `xpub-search-passphrase-candidates-file` (master `45e83fe`).
**Verdict:** **0 Critical / 1 Important — RED.** (2 Minors.) Round-2 folds landed correctly; the new Important is a STANDING memory-hygiene gap (present since r1, not fold-introduced) — exactly the after-every-round scrutiny the gate exists for. Re-verified at source by the orchestrator.

> Persisted verbatim per CLAUDE.md before folding.

---

## Verified clean (the r3 fold checks)
- **Variant-name consistency (I-A/M-3) — LANDED.** Scan exit uses `XpubSearchPassphraseCandidatesExhausted` at §1:12, §3:53, §4:68, §7:88; `XpubSearchNoMatch` survives ONLY in the §3/§4 "do NOT overload (its `error.rs:785` Display is wrong)" rationale. New variant sorts after `XpubSearchNoMatch` (`error.rs:328`); exit 4 matches.
- **M-1 serde — LANDED.** `skip_serializing_if="Option::is_none"` on the new optionals → single-`--passphrase` envelope byte-unchanged.
- **M-2 ArgGroup — LANDED.** `ArgGroup(required, single)` + removing the pairwise constraints preserves exactly-one-REQUIRED; existing `code(64)` mutex/required tests (`cli_xpub_search_passphrase_of_xpub.rs:92,105,129`) stay green via `main.rs:147`.
- Round-1 folds (I1 dispatch, I2 variant, I3 secret:false/FlagKind::Path) all still coherent + implementable.

## Critical
None.

## Important

### I-r3 — §3/§4 omit in-memory `Zeroizing` wrapping of the candidate passphrase line + `matched_passphrase`; violates the project owned-secret hygiene invariant + contradicts the `--passphrase` path §3 mirrors. Confidence 85.
The SPEC's only secret-hygiene treatment (§7:95) covers STDOUT exposure of `P` only. Nothing addresses IN-MEMORY hygiene of the new owned-secret allocations:
- §3 pseudocode (`:42-49`) reads each candidate as a plain `line` → `derive_master_seed(&mnemonic, &line)`. The single-passphrase path it mirrors wraps in `Zeroizing<String>` (`passphrase_of_xpub.rs:260`) + mlock-pins (`:291`). The scan uses a bare `String` — contradicts the mirrored path.
- §4:66 declares `matched_passphrase: Option<String>` (plain).
Violates `design/SPEC_secret_memory_hygiene_v0_9_0.md §1 item 2` ("Zeroizing on every OWNED secret allocation") + the `lint_zeroize_discipline.rs:46-47` documented process ("adding a new OWNED-secret allocation → add a row AND wrap in `Zeroizing`"). `ZEROIZE_ROWS` is a FIXED-ROW evidence-anchor lint (`:48`), NOT a scanner — it will NOT auto-fail the unwrapped `String`, so R0 must catch it (leading gate). The per-candidate derived seed is NOT part of this — `derive_master_seed -> Zeroizing<[u8;64]>` (`derive_slot.rs:31`) auto-scrubs. The candidate-line wrap is load-bearing; `matched_passphrase` is weaker (deliberately serialized to `--json` by opt-in).
**Fix (SPEC text):** §3 wrap the candidate line in `Zeroizing<String>` before `derive_master_seed` (mirror `:260`); §6/§7 add a `ZEROIZE_ROWS` entry for the new `passphrase_search.rs` owned-secret site (per the lint's "add a row AND wrap" process); §4 one line on `matched_passphrase` (wrap until emitted, or state why not).

## Minor
- **M-r3a — stale Source SHA.** SPEC line 5 records `86a59bb`; current tip `45e83fe`. Refresh per CLAUDE.md "document the source SHA." (All citations matched the live tree — doc refresh, not a re-verify trigger.)
- **M-r3b — §3:53 settled.** "R0-r2 verifies whether existing no-match prints the envelope THEN errors" is now answered (it DOES, `passphrase_of_xpub.rs:365-384`) — reword from a TODO to the settled statement.

## VERDICT: 0 Critical / 1 Important — RED.
One hygiene-plus-mechanical fold (I-r3 candidate-line `Zeroizing` + `ZEROIZE_ROWS` entry; M-r3a SHA; M-r3b phrasing), then re-dispatch r4.

---

## Fold note (applied after persisting)
- **I-r3 — FOLDED:** §3 wraps each candidate `line` in `Zeroizing<String>` (mirrors `passphrase_of_xpub.rs:260`); §4 wraps `matched_passphrase` until emission; §6/§7 add a `ZEROIZE_ROWS` entry for `passphrase_search.rs` (the "add a row AND wrap" gate over the new code).
- **M-r3a — FOLDED:** SPEC line 5 SHA → current tip.
- **M-r3b — FOLDED:** §3 parenthetical reworded to the settled statement (run() prints the `--json` no-match envelope THEN errors).
- Re-dispatched R0-r4.
