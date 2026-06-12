# CONTINUITY — miniscript-coverage-audit program (2026-06-12)

Rolling resume doc. Memory index: `project_miniscript_coverage_audit_program.md` (+ `project_g6_fmt_exemption_and_asymmetric_pin.md`).

## Program: a constellation-wide miniscript-wallet coverage audit → R0-gated cycles
Triggered by user questions ("how many tests of custom miniscript wallets / what types unsupported / ASCII vs UTF"). Gap analysis → cycle-prep recon (`cycle-prep-recon-*.md`, untracked scratch in toolkit root) → R0-gated cycles (Fable agents; plan + reviews persisted to `design/agent-reports/`).

## SHIPPED (all CI green) — DO NOT redo
- **Hygiene pass** — both repos, 6 FOLLOWUPs filed/flipped.
- **C1** md-codec NO-BUMP `96aaab3` — 7 render goldens (or_b/or_c/and_b/d:/j:/n:/True/False) + T_TARGET_TAGS 23→30.
- **C2** toolkit NO-BUMP `2f03eb0` — cross-tool md-cli differential corpus 8→17 rows.
- **C3** toolkit NO-BUMP `6c27585` — `tests/cli_restore_taproot_refusal.rs` pins 3 taproot restore-refusal contracts.
- **C4** md-codec `a3abdc8` + toolkit `f0587ab`, both NO-BUMP — md-codec goldens (multi 17-20/after/hash256/ripemd160/hash160) + toolkit verify-bundle hashlock + BIP-388 refusal.
- **C5** toolkit NO-BUMP `1971ffa` — sortedmulti-in-combinator refusal contract + FOLLOWUP root-cause fix.
- **C6** md-codec **0.35.3 PUBLISHED** `7dd2ff0` (tag md-codec-v0.35.3) + toolkit tail `77a361b` — reject mixed-case md1 per BIP-173. (Earlier same session: md-codec **0.35.2 PUBLISHED** = k>n encoder gate.)
- **C7** toolkit **v0.55.1** tagged `3ec2119` — general-tr faithful restore (details below).

Current origins: mnemonic-toolkit `3ec2119` (master, v0.55.1, tag mnemonic-toolkit-v0.55.1); descriptor-mnemonic `eb9f368` (main, md-codec 0.35.3).

## Cycle 7 — SHIPPED 2026-06-12 (CI status: see latest runs)
**GAP-1 T3-partial: faithful `restore --md1` of single-leaf + depth-1 two-leaf `tr(NUMS,<general miniscript>)`.** Toolkit **v0.55.1** tagged (`mnemonic-toolkit-v0.55.1`), git-tag-only, no md-codec change.

- Commits: `21f947e` (plan + R0 r1 RED 0C/1I/5m + R0 r2 **GREEN** 0C/0I/4m), `111e8ae` (impl, TDD red→green, test file renamed `tests/cli_restore_taproot.rs`, 11 cells), `3ec2119` (release ritual: 6 version sites + manual surgery + FOLLOWUPs + impl review **GREEN** 0C/0I/4m persisted). Sibling: descriptor-mnemonic `eb9f368` (display-asymmetry FOLLOWUP sharpened, left-branch-specific + lift note).
- Shipped design: 3-way `classify_taproot_restore` (Template | GeneralFaithful | refuse), strict-NUMS `Single` pass-through, structural depth≥2 + sortedmulti_a-under-TapTree refusals (ModeViolation exit 2, slug-citing), §5 Display-fidelity parse→print guard, `--format green` P2tr explicit refusal (exit 1). Goldens cross-verified vs a Bitcoin Core v25 oracle (impl review).
- NEW FOLLOWUP filed: `export-wallet-green-tr-policy-singlesig-emission` (impl m1 — export-wallet still emits a "singlesig" green payload for a tr policy; restore-side fixed).
- T3 remainder (all FOLLOWUP-tracked, blocked or fixture-only): depth≥2 (upstream #953), sortedmulti_a-in-tree (md-codec), keypath-only `tree:None` wire fixture, `:689` wording nit, is_nums:false.

## Conventions (load-bearing)
- **NEVER `cargo fmt --all` in the toolkit** — it reformats mlock.rs, breaking the g6 cross-repo byte-sync (the toolkit fmt gate excludes mlock.rs; format ONLY the touched file via `rustfmt +1.95.0 --edition 2021 <file>`). descriptor-mnemonic is repo-wide fmt-clean → `cargo fmt --all` is fine there.
- **Bash cwd persists within one call** — use `git -C <repo>` for cross-repo ops (a bare `cd` persists and bites the next command).
- **md-codec crates.io publishes are user-authorized** (AskUserQuestion; user said "Full release" twice). The toolkit is git-tag-only.
- R0/architect/impl-review agents = Fable (commit trailer `Co-Authored-By: Claude Fable 5`). Empirically-verified test-only cycles may use a self-review for the impl phase.
- Cycle-prep recon docs are untracked scratch (toolkit root); plan-docs + agent-reports ARE committed.

## Deferred / open (not this program unless asked)
GAP-1 T2 (md-codec sortedmulti_a per-index lowering, user-authorized publish); GAP-4b/c (STRESS-A tr leg; bitcoind oracles); `bundle-unrestorable-shape-advisory` umbrella (C5 deferred); fuzz-nightly-quarterly-bump (~2026-09). All FOLLOWUP-tracked.
