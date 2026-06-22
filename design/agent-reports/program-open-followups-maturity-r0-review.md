# Adversarial Review — PROGRAM_open_followups_maturity_2026-06-22.md

> Reviewer: opus architect (adversarial) · target `design/PROGRAM_open_followups_maturity_2026-06-22.md` · ground truth = 5 FOLLOWUPS registries + live source at the 2026-06-22 in-sync reconcile.

**Verdict: NOT GREEN (pre-fold).** 0 Critical / 3 Important / 5 Minor. The plan's load-bearing conclusions survive scrutiny — the funds-safety headline ("no open item can silently emit a wrong address / mis-build a wallet") is **verified true**, the fmt-gate red state is **verified accurate** (79 hunks, ~29 files reproduced under `+1.95.0`), and the BLOCKED §7 classifications (miniscript >13.1.0 w/ PR #953+#910, zeroize cluster, codex32 dormant, vendor firmware) are **all genuine**. The Important findings are a mis-ranked top-secret-value item, a stale "schedule work that already shipped" funds-register row, and a resolved item mislabeled as a permanent tripwire.

## IMPORTANT

### I-1. §4 row 7 schedules an advisory that has already shipped (stale RESOLVED-direction)
**Claim:** §4 row 7 + §3 Wave-4 list `bundle-engraves-unrestorable-pk-keyed-cards` as **SCHEDULABLE (Wave 4)**, "with no warning — add an emit-time advisory."
**Ground truth:** `mnemonic-toolkit/design/FOLLOWUPS.md:204-210`. The pk-keyed concern is **RESOLVED 2026-06-12** (PART 2 / `to-miniscript-check-pkh-double-wrap`, v0.54.1), and the emit-time advisory itself — `bundle-unrestorable-shape-advisory` — is **RESOLVED 2026-06-16, toolkit v0.57.1** (`src/unrestorable_advisory.rs`, fires at engrave time on both `bundle` and `import-wallet` for all three unrestorable shapes incl. sortedmulti-in-combinator, "fires IFF restore refuses"). The advisory the plan proposes to "add" already exists. The only OPEN remainder is the two *reconstruction* halves, separately tracked (one BLOCKED on miniscript>13.1.0).
**Correction:** Drop the "add an emit-time advisory" framing; note the advisory shipped v0.57.1; the survivor is reconstruction of sortedmulti-in-combinator (§4 row 6, partly upstream-gated). Schedules dead work as written.

### I-2. Wave-2 "highest-value residual" mis-ranks the secret; the master-Xpriv item is entirely absent
**Claim:** §3 Wave-2 calls `derive-slot-account-xpriv-scrub-confinement` the "Highest-value residual: account `Xpriv`."
**Ground truth:** `mnemonic-secret/design/FOLLOWUPS.md:470-481` `ms-cli-derive-xpriv-master-not-zeroized` is **OPEN (PARTIAL, cycle-15 Lane M; lifetime-min only landed)** and holds the **master/root `Xpriv`** — strictly higher spending authority than an *account* Xpriv. `ms derive` is "the only place an actual xpriv is materialized." Full close is upstream-blocked via `rust-bitcoin-xpriv-zeroize-upstream` (which the plan lists in §7.2), but the in-repo leg is never named, and the value-ranking that drives Wave-2 ordering is wrong.
**Correction:** Add `ms-cli-derive-xpriv-master-not-zeroized` to Wave 2 as the master-key counterpart, re-rank "highest-value residual" accordingly. (The rest of the ms hygiene family — inspect-report / decode-scrub / inspect-intake / repair-intake / json-output / verify-derived — is correctly omitted: all RESOLVED at ms-codec 0.6.0 / ms-cli 0.10.0.)

### I-3. §5.3 standing-discipline bucket includes a RESOLVED item mislabeled as a permanent tripwire
**Claim:** §5.3 lists `gui-schema-mirror-lockstep-discipline` as one of the 5 PERMANENT tripwires.
**Ground truth:** `mnemonic-toolkit/design/FOLLOWUPS.md:3198-3206`. Status is **`resolved`** (toolkit `a215f31` + mnemonic-gui `f5c597e`) — a one-time docs-codification task, not an open tripwire. The genuinely-permanent mirror-invariant is `mnemonic-gui-schema-mirror` (FOLLOWUPS.md:2385-2392, open), which the plan **already** lists as §5.3 row 2. So row 5 both mislabels a closed item and duplicates row 2.
**Correction:** Remove `gui-schema-mirror-lockstep-discipline`; the bucket then has 4 genuine permanent tripwires (`manual-cli-surface-mirror`, `mnemonic-gui-schema-mirror`, `md-mk-private-key-surface-watch`, `ms-kofn-json-wire-shape-ungated`), all verified open/permanent.

## MINOR
- **M-1.** "57/58 funds/correctness findings (1 won't-fix)" appears in no registry (bug-hunt labels are H1-H13); it's a report-internal tally. Soften / cite the report explicitly. Qualitative conclusion holds.
- **M-2.** §8.2 assumption 2 "all sub-findings RESOLVED" overstates `audit-2026-06-10-backlog` — 22/23 RESOLVED, one (`addresses-env-sentinel-overapplied`) is WONTFIX. Reword "RESOLVED or WONTFIX." Net burndown effect nil.
- **M-3.** Fmt-gate count: reproduced 79 hunks across **29 unique files**, one of which is g6-exempt `mlock.rs` → ~28 non-exempt. Plan says "~30." Tighten; the FOLLOWUP itself says "~29 non-mlock." Re-grep at fix time (set drifts each rustfmt release).
- **M-4.** `gui-run-confirm-modal-secret-redaction` (`mnemonic-gui/FOLLOWUPS.md:713-721`, verified open/high-severity) has a mandated co-lander `gui-import-wallet-env-var-secret-channel` (`:697-711`) specified to "land together." Plan schedules the display half without its companion. Add the cross-reference.
- **M-5.** §2.1 broken-gate framing is toolkit-centric. Sibling fmt-drift items md `…stable-rust-1-95-toolchain-fmt-clippy-drift` (FOLLOWUPS:1892) and mk `…rustfmt-drift-fn-signature-collapse-3-files` (FOLLOWUPS:352) are both **RESOLVED** (mk v0.8.0; md on the merged test-hardening branch) — correctly NOT open work. md's entry noted master is "latently CI-red until that branch merges"; the plan asserts all 5 repos in-sync, implying it has. Worth a one-line footnote.

## Items checked and CONFIRMED accurate (no finding)
- **Funds-safety headline:** TRUE — every funds entry RESOLVED (H1/H7/H8/H10/H12/H13 @ v0.61.0/v0.62.0), loud-refusal-BLOCKED, WONTFIX (permanent loud refusal), or label/diagnostic residual. No silent-wrong-output path in any of the 5 registries.
- **§7.1 miniscript >13.1.0 BLOCKED:** PR #953 (taptree Display, merged 2026-05-25) + PR #910 (SortedMultiA, merged 2026-04-03), both merged-to-master-unreleased; latest crates.io 13.1.0 contains neither; 3-step cross-repo unblock accurate (FOLLOWUPS:4260-4275). §4 rows 4/5/8/9 tags correct.
- **`error-rs-retroactive-alphabetical-sort` RESOLVED v0.29.0:** confirmed (FOLLOWUPS:3178).
- **`schema-mirror…` option (c) shipped v0.34.3; option (b) open:** confirmed (FOLLOWUPS:3455).
- **ScrubbedXpriv helper v0.70.0; remaining = 7-site lift + pub-struct-Drop caveat:** confirmed (FOLLOWUPS:4515-4528).
- **`green-tr` open / restore refuses v0.55.1; `unsorted-multi-generic-refusal` already funds-safe (cosmetic):** confirmed (FOLLOWUPS:4277-4285, 4413-4419).
- **mlock-g4 "leave tracked"; codex32 dormant decision; zeroize cluster:** confirmed (ms FOLLOWUPS:137-146, 176-196, 470-493).
- **BIP-38 §6.2 catalog:** confirmed (FOLLOWUPS:2289-2305).
- **§6 CATALOG-ONLY constraint:** compliant — identification + one-line value + blocked-flag, no API/sequencing/design.
- **Wave sequencing / fmt-gate-first:** justified; no dependency inversions.
- **mk `mk1-depth-child-lossless-by-construction-unenforced` (line 332):** checked as a potential missed funds hole — it's xpub-only (watch-only), reconstruction-from-origin-path is canonical → invariant-enforcement gap, not silent-wrong-address. Acceptable omission from §4.

(I-1/I-2/I-3 drop or mis-rank real work; M-1..M-5 are precision/completeness. None rises to Critical because the funds-safety override-axis conclusion is sound.)
