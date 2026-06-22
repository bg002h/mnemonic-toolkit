# R0 REVIEW (verbatim) — cycle-15 Group A brainstorm-spec — round 2 (confirmation)

**Dispatched:** 2026-06-21. **Base:** `9b7c78a7` (v0.68.0). **Spec:** `design/BRAINSTORM_cycle15_groupA_bsms_lint.md`.
**Verdict:** GREEN (0C/0I). Spec ready for the plan-doc phase.

---

The spec line 42 says the naive move "panics at `:561`" — the actual panic surfaces at the `assert!` macro (`:562-569`; my simulation reported `:568`, the panic-location heuristic landed on the last token line of the macro). Line 561 is a blank line. This is a cosmetic citation imprecision inherited verbatim from the round-1 review (which also wrote `:561`). It is purely descriptive (motivating WHY the union is needed) — it does not drive any plan/impl action, and the empirically-proven resolution (union both consumers) is correct regardless of which exact line the panic prints. Not a defect worth a re-spin; the load-bearing fact (the naive move is RED on `undeclared`) is true and proven.

I have everything I need. Final verdict.

---

## VERDICT: GREEN (0C/0I)

All round-1 findings are folded correctly and verified against real source at `9b7c78a7`; no new defects introduced. Empirically confirmed in the worktree (simulated then fully reverted — `git status` clean).

**I1 (gate) — RESOLVED.** Spec §#4 (lines 42-46) + Risk-3 (line 79) now unambiguously PIN both consumers reading `NON_ROW_SECRET_FILES ∪ TEST_ONLY_SECRET_FILES`: (1) `every_secret_bearing_src_file_is_declared_or_allowlisted` — `allowlisted` set at `:547-548` chained; (2) `non_row_secret_allowlist_is_non_empty_and_each_entry_still_bears_a_secret` (`:584`/`:585`) — non-empty assert + still-bears-secret iteration over the new tier. Floor stays 37 (38 live, 1 slack); bundle_unified.rs stays counted. I applied the exact fold and ran the suite: **4/4 lint tests GREEN with the union; RED (`undeclared` non-empty) without it** — the union is load-bearing, not vacuous. Correct and proven.

**#3 SemVer — RESOLVED.** Option A / MINOR 0.69.0 locked as DECIDED (lines 23/66/77). The "8 test sites need deref" claim is removed and explicitly flagged WRONG (line 26: "impl MUST NOT add deref noise"). ZERO caller edits stated (lines 26/78). Corrected census present: 1 prod (`import_wallet.rs:2428`) + 8 in-module hmac_key-touching sites + 1 integration helper (`cli_import_wallet_bsms_encrypted.rs:316-317`, line 19). I applied Option A and confirmed **20/20 bsms tests + the integration helper compile and pass with zero caller edits**.

**M2 — RESOLVED.** Line 51 says "Reword to avoid hardcoded decaying counts" and names all three (54, "36 + 16 = 52", "60 to 66"); confirmed all three live in the `:428-434` message.

**M3 — RESOLVED.** Line 48 documents the comment-substring false-trip as "Accepted friction (R0 M3) … intended."

**Version — RESOLVED.** 0.69.0 throughout (lines 66/67/74); sites enumerated, R0-verified at 0.68.0; `install.sh:32` self-pin confirmed.

**Fresh adversarial pass — no blockers.** Two cosmetic citation imprecisions noted, neither defect-class, neither warrants a re-spin: (a) line 42 says the naive move "panics at `:561`" — the actual `assert!` is `:562-569` (`:561` is blank; inherited verbatim from the round-1 review); purely motivational, doesn't drive impl. (b) The "8 in-module test sites" header could be misread as "8 `derive_hmac_key` calls" — it's actually 2 `derive_hmac_key(` calls + 6 `compute_mac(&hmac_key,…)` consumption sites (incl. `:429`/`:437` bare-`[0x42;32]` literals unaffected by the return-type change); the load-bearing claim (no deref noise at any of them) is correct and proven. No unsound test, no missed file, no internal contradiction, no funds/secret-hygiene error. compute_mac correctly stays bare (public MAC tag).

The spec is ready for the plan-doc phase.

---
*(Author note: both cosmetic citation imprecisions — the `:561`→`:562-569` panic line and the "8 in-module" clarification — were folded into the spec post-GREEN for accuracy; they were non-defect-class per this review.)*
