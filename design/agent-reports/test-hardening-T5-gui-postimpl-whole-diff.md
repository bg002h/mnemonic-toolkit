# Post-impl whole-diff R0 — T5 GUI test-hardening — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Repo `/scratch/code/shibboleth/mnemonic-gui` (master, v0.58.0); binary `mnemonic 0.75.0`; SPEC `mnemonic-toolkit/design/SPEC_test_hardening_T5_gui.md`. Uncommitted, test-only. All probes by execution; tree left byte-clean.

## Probes (all by execution)
**1. Green — PASS.** `MNEMONIC_BIN=mnemonic cargo test --workspace --jobs 2`: **687 passed / 0 failed / 4 ignored**. T5 targets: `canonicity_drift` 1/1, `non_canonical_descriptor_account_pin` 10/10 (8 old + 2 new), `bundle_restore_independent_oracle` 1/1. clippy `--all-targets -D warnings` clean; CI-shaped `--no-default-features -D warnings` clean (the `--no-default-features --all-targets` failure is the established convention — build.yml:39-41 runs headless clippy WITHOUT `--all-targets`; pre-existing files fail identically at HEAD). No `cargo fmt`.

**2. S1 Expect correctness (load-bearing) — PASS all 6 rows** (each run against the pinned binary): `pkh([…/44h/0h/0h]…)`→canonical; `wpkh(@0/<0;1>/*h)`→canonical; `tr([…/86h/0h/0h]@0)`→canonical; `wpkh([…/84'/0h/0']…)` mixed→canonical; `pkh([…/44'h/0'/0']…)`→**exit 2 ParseFails** "invalid child number format"; `wsh(multi(2,[…/48h/0h/0h/2h]@0,@1,@2))`→canonical. S2: `sh(sortedmulti(2,[…/45h/0h]@0,@1))`→non-canonical. No false oracle. Count `17C+4NC+4PF=25` verified.

**3. S1 RED-proof + wsh exemption — PASS.** `'?h?`→`'?` (9→0 occ, regexes 1-3 :110/:112/:114): `canonicity_drift` REDs on EXACTLY the 4 h-rows (pkh/wpkh-use-site/tr/mixed), each gui=NonCanonical vs toolkit=Canonical; the `wsh(multi …48h…)` row ABSENT from failures (wrapper-prefix regexes 4-5 :116/:118, no `'?h?` — exemption correct); none of the 19 pre-existing apostrophe rows fired. Reverted; conditional.rs sha256 identical.

**4. S2 RED-proof + direction — PASS.** Under the same mutation: `h_notation_canonical_descriptor_pins_account_to_zero` REDs (vis has NO `--account` PinValue — mis-lifted, the funds-relevant L12 direction); the NonCanonical case + 8 pre-existing cells green. Reverted byte-clean.

**5. S3 oracle independence (load-bearing) — PASS.** All four assertions compare against CONSTANTS, never re-emitted output. Executed the exact chain: `bundle --network mainnet --template bip84 --json --slot "@0.phrase=abandon…about"` → `ms1[0]==ms10entrsqqqq…cj9sxraq34v7f`, `master_fingerprint==73c5da0a`, 3 md1 chunks; `restore --from ms1=<emitted> --md1 c1 --md1 c2 --md1 c3 --json` → `wallets[0].first_addresses[0]==bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`, `verification.status=="verified"`. Provenance re-verified (`ms vectors` first entry carries this ms1 + abandon mnemonic + entropy 00…00). Slot idiom load-bearing (no-slot control → "no --slot inputs supplied"); md1 fed as repeating `--md1`; zero entropy/seed/phrase keys in restore JSON (not asserting restore-entropy is correct). Bogus-bin control → fails loudly, not skip.

**6. NO-BUMP + tree — PASS.** Final `git status`: only the 2 modified + 1 new test file; `git diff src/` empty; Cargo.toml/lock/pin untouched; all 35 `src/` files sha256-match the pre-review baseline. Post-revert re-run green. New file rustfmt-clean.

## Findings
Critical: none.
**Important I1 — spec-mandated FOLLOWUP filings missing (tracking fold, zero test-code change).** The in-code note (`bundle_restore_independent_oracle.rs:32-34`) references FOLLOWUP `gui-descriptor-mode-bundle-restore-independent-oracle` — filed nowhere; §S3 scope-guard mandates it. S4 (correctly excluded) then requires filing `schema-mirror-defaults-drift-md-ms-mk-extension`. Fold: file both in the toolkit's `design/FOLLOWUPS.md` with the shipping commit.
Minor: M1 — suffix-origin `@N[fp/path]` h-notation blind spot (proven by targeted mutation; one row `wpkh(@0[deadbeef/84h/0h/0h]/<0;1>/*)`, empirically canonical, closes it — not spec-required). M2 — one rustfmt-divergent hunk (no fmt gate; no action). M3 — pre-existing stale pin citation `canonicity_drift.rs:7` (`v0.47.3` header vs 0.75.0 fixtures; optional touch-up).

## VERDICT: OPEN (0C / 1I)
All six probes pass; both load-bearing checks byte-match the pinned binary; tree byte-clean. The single Important is a tracking/ledger fold — converges to GREEN on filing the two FOLLOWUP slugs.

---
**FOLD STATUS (opus, 2026-07-10):** I1 folded — 3 FOLLOWUPs filed in `mnemonic-toolkit/design/FOLLOWUPS.md`: `gui-descriptor-mode-bundle-restore-independent-oracle`, `schema-mirror-defaults-drift-md-ms-mk-extension`, + `gui-canonicity-suffix-origin-h-fixture` (M1, captured rather than added — avoids scope-creeping a GREEN leg with a Minor needing its own RED-proof). M2 no-action; M3 pre-existing/optional. T5 GREEN → GUI PR (test code) + toolkit design commit (FOLLOWUPs + SPEC + reports).