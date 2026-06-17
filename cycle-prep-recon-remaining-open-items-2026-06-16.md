# cycle-prep recon — 2026-06-16 — REMAINING OPEN ITEMS (consolidated)

**Origin/master SHA:** toolkit `15d43c9` (clean, in sync). Supersedes the 12 surviving
per-item `cycle-prep-recon-*.md` after the 2026-06-16 staleness sweep (15 stale recons
pruned @ `15d43c9`). This is the current-state map of genuinely-open work; each item is
re-grounded against current source + `design/FOLLOWUPS.md` status. Feeds the architect's
tiered/phased plan.

Repos: `mnemonic-toolkit` (toolkit, v0.56.0+), `descriptor-mnemonic` (md-codec/md-cli),
`mnemonic-secret` (ms-codec/ms-cli), `mnemonic-key` (mk-codec/mk-cli), `mnemonic-gui`.

---

## A. UPSTREAM-BLOCKED — gated on rust-miniscript > 13.1.0 (#953); NOT schedulable now

- **A1. `sortedmulti_a`-under-a-taptree render + per-index derive-time lowering** — md-codec's
  crates.io miniscript 13.0.0 cannot render `Terminal::SortedMultiA`. Slug
  `md-codec-sortedmulti-a-to-miniscript-rendering-gap` (open). Recons:
  `sortedmulti-a-and-non-nums-internal-key` (item #2), `taproot-coverage-gaps` (sub-1).
- **A2. depth-≥2 taproot restore** — the shipped binary refuses depth-≥2 (restore.rs
  `ensure_taptree_depth_le_one`); only the experimental `mnemonic-depth2` POC does it.
  Umbrella slug `taproot-coverage-cycle-on-miniscript-gt-13-1-0` (open).
- **A3. `upstream-miniscript-taptree-depth2-display-asymmetry`** (open, toolkit companion).
- **Action:** none until the upstream release lands; keep the refusal gates + POC. The
  toolkit ALREADY pins fork `95fdd1c` (has SortedMultiA) and routes restore/derive around
  md-codec for taproot — so the toolkit side is as far as it can go; the gap is md-codec's
  `to_miniscript` + a coordinated multi-repo flip when upstream ships.

## B. ACTIONABLE TEST-HARDENING — NO-BUMP, test-only, low risk

- **B1. STRESS-A taproot leg (GAP 4b)** — `tests/prop_backup_restore_roundtrip.rs` is still
  wrapper-`wsh`-only. Add a tr leg generating concrete `tr(NUMS,{multi_a|sortedmulti_a}(k,…))`
  strings (bypassing build-descriptor's wsh-only `WrapperKind`, entering at `bundle
  --descriptor`); O1/O2/O3 oracles reuse unchanged (the patched fork parses+derives both).
  PLUS the never-shipped non-NUMS-tr / `@`-in-both loud-refusal negative cell. Toolkit. Recon:
  `differential-harness-breadth` (Cycle 2). No open slug yet (recon said "file one for the
  defer"). ~120-180 LOC.
- **B2. `toolkit-arm-dup-if-ignored-stub`** (open) — de-ignore `parse_descriptor.rs::arm_dup_if`
  AND write the body (`wsh(or_i(pk(@0/<0;1>/*),dv:older(144)))` → assert a `Tag::DupIf` node).
  De-ignoring alone = vacuous (empty body). Toolkit, 1 test. Recon: `seven-fragment-render-tests`.
- **B3. ms-codec test-hardening themes 1/2/3** — md-codec + mk-codec shipped all three (indel
  reject-contract + correction property + …); **ms-codec is spec-only** (SPEC commit `6e984d7`,
  no impl, no `indel_reject_contract.rs`). Extend ms-codec's existing proptest with the three
  themes. Sibling (`mnemonic-secret`), test-only. Recon: `codec-test-hardening-themes-1-2-3`.
- **B4. md-codec bitcoind corpus breadth** — `bitcoind-differential-corpus-breadth` (open,
  descriptor-mnemonic). +4-6 `Shape` rows to md-codec's bitcoind differential (plain `multi`,
  a hashlock, `after`, `or_d`/`andor`; no SortedMultiA per A1). Sibling, test-only, cheap.
  Recon: `differential-harness-breadth` (c1).

## C. ACTIONABLE FEATURES / UX — versioned (PATCH/MINOR) + lockstep

- **C1. bundle-time unrestorable-shape advisory** — `bundle-accepts-sortedmulti-in-combinator-restore-cannot`
  → re-scoped into umbrella `bundle-unrestorable-shape-advisory` (open). `bundle`/`export-wallet`
  ACCEPT shapes restore later refuses LOUDLY (sortedmulti-in-combinator, per-key use-site
  overrides, hardened wildcard) — engrave-but-can't-restore. Warn at engrave time so the user
  knows before the steel. Toolkit, PATCH (advisory-only; no wire change). The contract-pin
  proptest already shipped; the UX advisory did NOT (grep `unrestorable` in src/ = empty).
  Recon: `sortedmulti-in-combinator-contract`.
- **C2. verify-bundle BIP-388 policy intake** — `verify-bundle-bip388-policy-intake` (open;
  carved out of the v0.49.0 expansion which landed on `export-wallet`/`bundle --descriptor`).
  Let `verify-bundle` also accept a leading-`{` BIP-388 wallet-policy JSON (expand via the
  existing `wallet_import::pipeline::expand_bip388_policy`). Toolkit, MINOR (new input format).
  Recon: `minor-coverage-gaps` (deferred tail).
- **C3. single-sig batch emit** — `single-sig-multi-script-type-batch-emit-not-surfaced` (open,
  filed today). `addresses --all-script-types` / `export-wallet --all-single-sig` loop over
  {p2pkh, p2sh-p2wpkh, p2wpkh, p2tr}, mirroring `nostr --all-script-types`. Toolkit, MINOR +
  GUI schema_mirror + manual flag-coverage lockstep. Recon: `all-single-sig-batch-emit`.
- **C4. Theme-A A1 — concrete-descriptor ingest door** — accept a raw `wsh(sortedmulti(…))`
  WITHOUT `@N` placeholders at intake (today only the BIP-388 JSON-policy door + `@N` template
  door exist). Toolkit. Recon: `theme-a-wallet-interop` (A1). No open slug — confirm it's
  genuinely missing vs subsumed by `bundle --descriptor` concrete-key path first.
- **C5. Theme-A A3 — `import-wallet --format green`** — `green` is absent from the import
  format enum (export-side only). Toolkit, MINOR + lockstep. Recon: `theme-a-wallet-interop` (A3).
- **C6. mk1-card SLIP-0132 preservation** — `mk1-card-slip0132-variant-not-preserved-on-card`
  (open, filed today). PRODUCT QUESTION first: is on-card ypub/zpub preservation wanted, or is
  normalize-in/re-emit-out sufficient? Decide before any code. Toolkit (+ mk-codec wire field).

## D. INFRA / REFACTOR / MAINTENANCE

- **D1. CI action majors v6/v7/v8** — `ci-actions-catch-up-to-latest-majors` (open; precondition
  met). Needs a throwaway-tag rehearsal for `download-artifact@v8` (no-auto-decompress + hash
  semantics) + `checkout@v6` ($RUNNER_TEMP creds) before bumping the tag-gated release jobs.
  CI-only. Recon: `ci-actions-catch-up-and-self-trigger`.
- **D2. library error + language surface promotion** — `library-error-and-language-surface-promotion`
  (open, DEFERRED). ~80-file blast radius; blocked on a type-entanglement decision (shim vs
  reroute). Big. Recon: `refactor-and-wireshape-gate-debt`. Needs an architecture decision before
  any cycle.
- **D3. mlock.rs fmt-exemption closure** — `mlock-rs-fmt-exempt` (standing-open). The exemption
  shipped (dbdacfb); the prepped CLOSURE = reformat mlock.rs at the next ms-cli pin bump
  (install.sh still pins ms-cli-v0.7.0). Rides a future pin-bump cycle, not standalone.
- **D4. fuzz nightly bump** — `fuzz-nightly-quarterly-bump` (recurring-maintenance, parked
  ~2026-09). Not due.

## E. NO ACTION — permanent trackers

- `technical-manual-codec-g2-not-enforceable-in-single-repo-ci` — accepted-wontfix discoverability
  marker (bare CI can't check codec-G2 without the sibling repos). No work.

---

## Quick sizing for the architect

| item | repo | tier | risk | blocked? |
|---|---|---|---|---|
| B1 STRESS-A tr leg | toolkit | test/NO-BUMP | low (may FIND a bug) | no |
| B2 arm_dup_if de-stub | toolkit | test/NO-BUMP | trivial | no |
| B3 ms-codec themes 1-3 | ms-codec | test/NO-BUMP | low | no |
| B4 md-codec bitcoind breadth | md-codec | test/NO-BUMP | low | no |
| C1 unrestorable-shape advisory | toolkit | PATCH | low | no |
| C2 verify-bundle BIP-388 intake | toolkit | MINOR | med | no |
| C3 single-sig batch emit | toolkit+GUI+manual | MINOR | med | no |
| C4 concrete-desc ingest door | toolkit | ? | med | confirm-need-first |
| C5 import --format green | toolkit+GUI+manual | MINOR | med | no |
| C6 mk1 SLIP-0132 on-card | toolkit+mk-codec | product Q | med | decide-first |
| D1 CI v6/7/8 | CI | infra | med (release jobs) | rehearse-first |
| D2 lib surface promotion | toolkit | refactor | HIGH (80 files) | arch-decision-first |
| A1-A3 taproot upstream | md-codec+ | feature | — | UPSTREAM-blocked |

Recommended framing for the plan: a **fast test-hardening tier** (B1-B4 — independent, NO-BUMP,
ship in parallel), a **small-feature tier** (C1-C3, C5 — versioned, lockstep-aware), a
**decision-gated tier** (C4, C6, D2 — need a product/architecture call before coding), and a
**rehearsal-gated infra item** (D1). A-items stay parked behind upstream.
