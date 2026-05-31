# Plan-doc R2 review (after R1 fold) — output-type-stderr-advisory Phase 1
**Date:** 2026-05-31 · **Reviewer:** opus architect · **Verdict: RED (0C/1I/3m) → folded.**

## R1 folds re-verified correct vs source
I-new (4 of 5 named cells real negative !contains; inverting correct; P1 re-pin keeps P1 green); M1 (3 orphaned IsTerminal: slip39:54/seed_xor:24/final_word:20; compare_cost:6 + repair:34 correctly untouched); M2 (cli_auto_repair.rs:104 md1 stdout-only → backfill mk1/md1 class cells); M3 (encode.rs:92-96 dual branch). Legacy-helper removal SAFE at P3 (full caller set migrated by P2: derive_child:308/silent:286/electrum:149 P1, bundle:931/convert:1102/repair:216/inspect:156/nostr:251 P2, repair.rs:1333 P3). Phase greenness holds (with the I1 fix). SemVer/naming/byte-literals consistent.

## Important (FOLDED)
**I1 — the "ALL FIVE" enumeration listed only FOUR; missed `cli_slip39_advisories.rs:265` (slip39 SPLIT's `advisory_k_of_n_stdout_silent_when_piped`, `!contains("SLIP-39 shares on stdout")`).** slip39 split is P1; dropping its gate (`slip39.rs:544`) re-emits the addendum "SLIP-39 shares on stdout" (`:548`) unconditionally on piped stdout → flips the cell PASS→FAIL → P1 commit RED. *Fix (folded):* added `cli_slip39_advisories.rs:265-266` as the 5th cell in P1 Step 4 + P5 Step 1; INVERT; discovery via `grep -rn silent_when_piped tests/` (the `does_not_emit` regex missed both slip39 cells).

## Minor (folded/cosmetic)
m1 discovery regex → `silent_when_piped`. m2 repair loop at `:140` not `:144` (cosmetic). m3 ms-cli `:90/:135` stale vs live `:92-95` — P4 re-grep flagged.

## Controller: 5th cell + regex folded; re-dispatch R3.
