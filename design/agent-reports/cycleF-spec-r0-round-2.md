# SPEC R0 review — ms1-repair-demote-to-candidate — round 2

**Verdict: NOT GREEN (0 Critical / 1 Important / 2 Minor)**
**Reviewer:** Fable architect (funds-weighted, read-only), per user directive. Verified @ toolkit `4c554295`, ms `c2fd4eb`, mk `1c9fbf7`. SPEC @ `e07e86e5`.
**Dispatched:** 2026-07-09 (Cycle F, SPEC R0 loop round 2). Persisted verbatim per CLAUDE.md.

Round-1 Critical fully resolved; the two load-bearing round-1 Importants (C1, I1) verified funds-sound against source. One new Important in the folded §4 table (a factual error §6 would mirror into 4 manual chapters — the exact staleness this cycle fixes). Two Minors. No funds hole remains.

## Round-1 fold verification
- **C1 → RESOLVED, sound.** Single-sig site reachable only in `!watch_only` (`watch_only=expected.ms1.first().is_empty()` @:2027) → `expected_ms1` @:2047 non-empty; multisig site under `!watch_only_slot` (`exp_ms1.is_empty()` @:2450) → `exp_ms1` @:2449 non-empty. NO un-handled no-ground-truth branch. ms1 checks PASS only when `corrected==expected.ms1[i]` (the typed-seed card) — a miscorrection on a different valid seed can't pass unless it EQUALS the ground truth (then it IS the right card). Wrong-bundle attack → `corrected(A)≠expected(E)` → `ms1_entropy_match` FAIL → exit 4 (§5.5). HOLE CLOSED. mk1/KeyCard dep, reorder, passphrase, clean-mk1 precond, multisig degrade, new variant — all correctly evaporated; mk-codec not even read → NO-BUMP trivial.
- **I1 → RESOLVED, sound.** `indel.rs::recover_indel` (:78-121) enumerates candidates → `Ms1IndelOracle` (`repair.rs:1259-1274`) calls `decode_with_correction` (full residue check vs `MS_REGULAR_CONST` + defensive re-verify, `decode.rs:252-297`), accepts only if off-placeholder corrections `≤ e_subst` (pure-indel ⇒ off==0) → the checksum is an independent TEST, not the correction source (architecturally distinct from substitution; identical to how mk1/md1 indel already blesses). `dedup_by_recovered` → 1⇒Unique / ≥2⇒Ambiguous→exit 4. Keeping Unique indel at exit 5 = the existing default (zero code change); carve-out = docs + a test. Funds-sound.
- **I3 → RESOLVED.** `Ok(if any_fail {4} else {0})` @:548 — failed `ms1_entropy_match` row → full table + exit 4, no typed error. §7 drops the variant.

## Important — I-R2-1: §4 table attributes indel behavior to 3 surfaces with NO indel path (§6 would mirror it into 4 manual chapters)
Grep-verified: `ms repair` (ms-cli `RepairArgs={ms1,json}`) has NO `--max-indel`/indel recovery; auto-repair inline sites (convert/inspect/xpub/verify-bundle) have NO indel plumbing (`try_repair_and_short_circuit`→`repair_card` substitution-only; a length-corrupted ms1 → `Err(_)=>Ok(())` @:1684 fall-through, never indel); indel is reachable ONLY from `mnemonic repair --max-indel` (`cmd/repair.rs:64`→`recover_indel_card` @:170). So 3 §4 cells are FALSE: `ms repair`|indel|"exit 5", `toolkit auto-repair`|indel|"applies (exit 5)", `verify-bundle`|indel|"applies". Only the `mnemonic repair --ms1` indel cell is correct. Shipping fresh false cells that §6 copies into `41/42/43/44-*.md` is self-defeating (the exact staleness class this cycle is chartered to fix). **Fix:** set those 3 cells to `n/a — no indel path (indel is \`mnemonic repair --max-indel\` only)`; scope the exit-5 indel carve-out in §3/§6 to `mnemonic repair --ms1` specifically (not "ms1" generically). Blocks GREEN until folded.

## Minors
- **M-R2-1** — §0.3↔§4 advisory inconsistency: §0.3 places the I2 advisory inside `try_repair_and_short_circuit` (shared by all 9 sites incl. the 2 verify-bundle ms1 sites), but §4's verify-bundle row shows check-evaluation. Literal impl → a verify-bundle MATCH emits "a candidate correction exists but cannot be self-verified" ALONGSIDE passing ms1 checks — contradictory on the safety-sensitive path (exit/verdict stay correct → Minor). Fix: advisory fires ONLY at the standalone-inline sites (convert/inspect/xpub), NOT the 2 verify-bundle sites — cleanest if the verify-bundle C1 wiring gets the corrected string via a direct `repair_card` call rather than the advisory-emitting helper.
- **M-R2-2** — §6 `--json verdict` parenthetical mis-attributes indel: indel emits via `IndelJson` (`cmd/repair.rs:350-365`, `confident:bool`, no verdict), NOT `RepairJson`. So `RepairJson.verdict` is `{blessed(clean 0-corr), candidate(subst)}` only. Drop "/indel" (or also address `IndelJson`). Doc-accuracy.
- *(Non-finding: `verdict` + advisory + 5→4 will churn ms1 `repair` verify-examples/`.examples-build` goldens — §6/§7 cover regen.)*

## Funds-attack re-run — all SAFE
wrong-bundle / multisig cross-cosigner / no-ground-truth / miscorrection-lands-on-expected / indel-unique / indel-multi-hit / demotion false-bless+false-candidate / clean=0 / uncorrectable=2 → ALL SAFE (evidence per row in the review body).

## SemVer/cross-repo/release — holds, re-verified
toolkit MINOR v0.81.0 + ms-cli MINOR 0.14.0; ms-codec + mk-codec (now trivially, C1 removed all mk1 reads) + md* NO-BUMP; no new variant; no clap surface → no GUI/schema_mirror; 4-site pin advance complete; release order correct.

**Gate: I-R2-1 (2-cell table fix + §3/§6 scoping) + 2 Minors block GREEN. Fold, persist, re-dispatch round 3 — expected GREEN quickly; design is funds-sound + structurally converged, remaining defects are doc-accuracy.**
