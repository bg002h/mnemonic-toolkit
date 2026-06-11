# Implementation review — toolkit v0.53.5 ms-codec pin-bump 0.4.0→0.4.2 (2026-06-10)

Reviewer: Fable 5 implementation-review agent (post-impl, pre-commit). Plan @ design/PLAN_ms_codec_pin_bump_0_4_2.md (R0 GREEN r2). Verdict: GREEN (0 Critical / 0 Important / 2 cosmetic Minor, no action). Review verbatim below.

---

## Critical
None.
## Important
None.

## Minor

**M1 — Cosmetic:** the silent-payment address-extraction helper (`cli_hrp_case_insensitive.rs:420`) uses `l.trim().strip_prefix("address:")` — whitespace-sensitive; works (verified extracts `sp1qqfqnnv8cz…`), would `panic!` (not assert-fail) if the label format changed. Acceptable for a characterization test.

**M2 — Confirmation, no action:** the mnemonic-secret companion's "consumed by toolkit v0.53.5" note lives in the sibling repo (outside this tree); update it in lockstep when staging that repo.

## Verdict

**GREEN — 0 Critical / 0 Important.**

- **Pin isolation:** Cargo.lock — exactly 3 line-pairs change (toolkit 0.53.4→0.53.5, ms-codec 0.4.0→0.4.2, ms-codec checksum); no transitive dep moved. Cargo.toml:20 `ms-codec = "0.4.2"`; :23 exact-pin comment accurate.
- **4 inversions + new security cell** — all spot-checked vs target/release/mnemonic v0.53.5:
  - inspect (:307) exit 0 + kind:ms1/tag:entr + advisory + no card echo;
  - silent-payment (:408) uppercase ≡ lowercase BYTE-IDENTICAL (`sp1qqfqnnv8cz…`, diff);
  - **ms_shares_combine_uppercase_secret_at_s_refused_no_leak (:447)** exit 2 + `the secret share (index 's') must not be combined` + stdout COMPLETELY EMPTY (no entropy, no card). **Non-vacuous: pre-bump the bypass returned the secret at exit 0 → the .code(2)+prose+empty-stdout asserts would all fail if the guard were absent. Asserts refusal specifically.** Delegation `cmd/ms_shares.rs:385` → `ms_codec::combine_shares` confirmed; guard entirely ms-codec-side; SecretShareSuppliedToCombine → exit 2 (error.rs:390) + prose (friendly.rs:131).
  - repair (:382) exit 0 + uppercase passthrough + advisory; old HrpMismatch string gone;
  - verify-bundle (:341) positive `ms1_decode: ok` asserted; mismatch from absent mk1;
  - module `//!` header (:1-35) fully rewritten — no stale claims; the R0-r2 M-a silent-payment bullet now reflects address derivation.
- **Ritual:** CHANGELOG [0.53.5] PATCH + SECURITY framing true; version at Cargo.toml:3 / README.md:13 / crates/…/README.md:9 / install.sh:32 / Cargo.lock (readme_version_current passes); manual note (41-mnemonic.md:3113) includes ms1 end-to-end + preserves the mixed-case-rejection clause; FOLLOWUPS `toolkit-ms-codec-pin-bump-0-4-1-combine-fix` resolved + hrp-classifier ms1-leg-closed note; the :82 "nothing flips green automatically" phrasing correctly intact.
- **Tests/lint:** workspace 2911 passed / 0 failed; clippy clean; full manual lint (release binary + pinned siblings) all 6 stages OK incl. flag-coverage (no surface drift).
- Tree left exactly as found.
