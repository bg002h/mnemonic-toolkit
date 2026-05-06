# mnemonic-toolkit v0.4.5 — verify-bundle helper call-site rollout

**Status:** DRAFT (pre-architect-review).

**Cycle theme:** finish the v0.4.4 helper-foundation work by wiring `emit_verify_checks` into all four production verify-bundle dispatch paths, dropping the legacy v0.1 leftover `stub_linkage` check, and adding forensic-field integration tests.

**License:** "no users yet → ignore migration" — internal Rust APIs, JSON envelope check arrays, and CLI error messages may break freely. The single test file (`cli_json_envelopes.rs`) that hard-pins the v0.4.4 check-name shape is updated in lockstep.

## Closes

- FOLLOWUP `verify-bundle-helper-call-sites-rollout-v0.4.5` (P.3-P.7).
- FOLLOWUP `verify-bundle-helper-foundation-cleanup-v0.4.5` (L-1 + L-2).
- FOLLOWUP `verify-bundle-9-3plus6n-descriptor-mode-parity` (closed transitively via P.5).

## Pre-flight survey (informs phase scope)

Source state read 2026-05-06:

- `cmd/verify_bundle.rs` is 2365 lines, 84 total `checks.push(VerifyCheck { ... })` sites.
- `run_full` (line 300, single-sig template-mode full+phrase): 31 push sites, 8 distinct check names: `ms1_entropy_match`, `mk1_decode`, `mk1_xpub_match`, `mk1_fingerprint_match`, `mk1_path_match`, `md1_decode`, `md1_wallet_policy`, `md1_xpub_match`, `stub_linkage`.
- `run_watch_only` (line 616, single-sig watch-only): same 8 distinct check names; logically same shape with `ms1_entropy_match` always passing-vacuously (skipped).
- `run_multisig` (line 920): only 3 distinct names emitted (`ms1_entropy_match`, `md1_decode`, `md1_wallet_policy`) — does NOT emit the SPEC §5.7 3+6N schema.
- `descriptor_mode_verify_run` (line 1375): only 3 distinct names emitted (`ms1_entropy_match`, `mk1_match`, `md1_match`) — coarse v0.3 ladder, fundamentally different from template-mode shape.
- Helper `emit_verify_checks` (line 1888-2102) emits the SPEC §5.7 9-check ordering: `ms1_decode, ms1_entropy_match, mk1_decode, mk1_xpub_match, mk1_fingerprint_match, mk1_path_match, md1_decode, md1_wallet_policy, md1_xpub_match`. Helper is `#[allow(dead_code)]`.
- Multisig path in helper returns a TODO stub (`name: "TODO_multisig_v0_4_5"`); P.4 replaces with real 3+6N emission.
- Test pins on check names: `cli_json_envelopes.rs` lines 96-105 (1 location, 9 names) — pins the run_full output. No other test file pins check-name strings.

**Helper-vs-current divergence under wire-up:**
- **+** `ms1_decode` joins (always position 0).
- **−** `stub_linkage` is dropped (was a v0.1 leftover with no SPEC §5.7 equivalent).
- Net: same 9-element count, different shape; SPEC §5.7-correct.

## Phase plan

### Phase P.3 — Wire helper into `run_full` + `run_watch_only`

**Why bundle them:** the helper's watch-only short-circuit triggers on `expected.ms1[i].is_empty()`, which is exactly the watch-only sentinel set by `synthesize_watch_only`. Both single-sig dispatch paths construct the same 9-check shape; only the SuppliedCards source differs. Wiring both at once avoids re-stating the per-cell logic across two near-duplicate functions.

- Replace `run_full`'s body (post mode-violation pre-checks) with:
  1. Construct `expected: Bundle` via existing `synthesize_full` call.
  2. Read user-supplied ms1/mk1/md1 from `args` (--ms1/--mk1/--md1 vecs).
  3. Build `SuppliedCards { ms1: &args.ms1, mk1: &args.mk1, md1: &args.md1 }`.
  4. Call `emit_verify_checks(&expected, &supplied, false)`.
  5. Append checks to the caller's `Vec<VerifyCheck>`.
- Replace `run_watch_only`'s body symmetrically: `synthesize_watch_only(...)` produces a Bundle with `expected.ms1 = vec!["".to_string()]` (empty-string sentinel), and the helper's watch-only short-circuit handles ms1_decode + ms1_entropy_match accordingly.
- `run_full` retains: stdin-phrase reading + check_no_concurrent_stdin pre-check + return wiring.
- `run_watch_only` retains: xpub-arg validation + master-fingerprint-required pre-check + return wiring.

**Estimated lines changed:** ~31 push sites in `run_full` and ~28 in `run_watch_only` collapse to ~5 lines each (synthesize + SuppliedCards + helper call). Net delete: ~250-350 lines.

**Verification gate:**
- `cargo test verify_bundle::` lib tests pass (helper unit tests still green).
- Integration test `cli_verify_bundle_full.rs` + `cli_verify_bundle_watch_only.rs` pass without modification (these assert on text-mode stdout; the new check-names are still emitted in the same `name: ok|fail detail` format).
- Integration test `cli_json_envelopes.rs` line 96-105 will FAIL — fixed in P.6.

### Phase P.4 — Real multisig 3+6N emission in helper + wire into `run_multisig`

**Helper expansion:** replace the multisig TODO stub with the SPEC §5.7 3+6N schema (verbatim from SPEC line 103):

```
3 shared (md1, single-decode of the shared descriptor card):
    "md1_decode"
    "md1_wallet_policy"
    "md1_xpub_match"

6 per cosigner @i (i in 0..N), per-slot bracket-indexed:
    "ms1_decode[i]"
    "ms1_entropy_match[i]"
    "mk1_decode[i]"
    "mk1_xpub_match[i]"
    "mk1_fingerprint_match[i]"
    "mk1_path_match[i]"
```

**Naming convention (SPEC line 103):** indexed checks use bracket notation `[i]`, not `_@i`. Format string: `format!("ms1_decode[{}]", i)`. Single-sig reuses unindexed names (`ms1_decode`, etc., no brackets).

**Output ordering:** SPEC line 103 lists "3 shared + 6 per cosigner" but does not pin emission order. Plan: emit the 6N per-cosigner block first (interleaved by slot — all 6 ms1+mk1 checks for @0, then all 6 for @1, …), then the 3 shared md1 checks. Rationale: parallels the single-sig 9-check ordering (ms1, mk1, md1) and produces the most diagnostically useful output (per-cosigner forensics grouped together). Confirm this is consistent with SPEC §5.7's prose during execution; if SPEC mandates the reverse order, SPEC wins.

**Hybrid + watch-only handling (SPEC lines 104-106):**
- Hybrid mode (some slots secret-bearing, some watch-only): same 3+6N schema; for watch-only slots (`expected.ms1[i] == ""`), `ms1_decode[i]` and `ms1_entropy_match[i]` short-circuit with `passed: true, decode_error: "skipped: watch-only slot"`.
- Pure watch-only multisig: all `ms1_decode[i]` / `ms1_entropy_match[i]` short-circuit identically.
- `wif` slots: verify-bundle treats wif slots as watch-only for ms1 checks (`expected.ms1[i] == ""`); mk1 checks run normally against the supplied wif's derived pubkey. The slot-resolution layer ensures wif-bearing bundles emit `expected.ms1[i] = ""`, so the helper needs no special-case wif logic.

**Indexing:**
- `SuppliedCards.mk1` indexing convention (already documented in v0.4.4 P.2): `mk1[i]` is the cosigner-`i` mk1 card, with placeholder strings for absent slots.
- `SuppliedCards.ms1` for multisig is a length-N slice (one entry per cosigner; empty string for watch-only/wif slots).
- `SuppliedCards.md1` is the shared descriptor card chunks (same as single-sig; not per-cosigner).
- `run_multisig` body collapses to: build expected Bundle, build SuppliedCards, call helper with `is_multisig: true`, append checks.

**Estimated lines changed:** helper grows ~150-200 lines (multisig branch + per-cosigner loop). `run_multisig` shrinks similarly. Net roughly neutral; gain is consolidated forensic-field population.

**Verification gate:**
- New unit test in `helper_tests` mod: `helper_multisig_full_emits_3plus6n_checks_in_spec_order` (matches the existing single-sig pattern).
- `cli_bundle_multisig.rs` + `cli_verify_bundle_*.rs` integration tests still pass (assert on text-mode stdout; same emission shape).

### Phase P.5 — Rewrite `descriptor_mode_verify_run` to use the helper (closes 9/3+6N parity)

- Current: emits 3 coarse-ladder checks (`ms1_entropy_match`, `mk1_match`, `md1_match`) — pre-SPEC v0.4 leftover.
- New: synthesize-from-descriptor-binding → Bundle, build SuppliedCards from user-supplied --ms1/--mk1/--md1, call `emit_verify_checks` with `is_multisig` derived from binding shape. Single-sig descriptor → 9 checks; multisig descriptor → 3+6N checks. Same shape as template-mode.
- Closes FOLLOWUP `verify-bundle-9-3plus6n-descriptor-mode-parity`.

**Estimated lines changed:** descriptor_mode_verify_run body shrinks from ~495 lines to ~30; net delete ~460 lines (most of the 84-site total).

**Verification gate:**
- `cli_descriptor_mode.rs` integration test still passes (assert on text-mode "result: ok|mismatch" line).
- `cli_json_envelopes.rs` may need a new test pinning the descriptor-mode shape; defer to P.7.

### Phase P.6 — Test migration (`cli_json_envelopes.rs`)

- Update lines 96-105 of `cli_json_envelopes.rs`: replace the v0.4.4-shape vec with the helper-emitted shape (`ms1_decode` joins; `stub_linkage` drops). Single 7-line edit.
- Audit other test files for any incidental check-name dependency (none found in pre-flight survey, but re-verify after P.3-P.5 land).

**Verification gate:** `cargo test --no-fail-fast` returns all-pass.

### Phase P.7 — Forensic-field integration tests

Add test cases to verify the post-rollout JSON envelope emits the v0.4.1 J.1 `expected`/`actual`/`diff_byte_offset`/`decode_error` fields at every push site, not just the one v0.4.1 J.7 proof-of-shape site.

- Add `cli_verify_bundle_forensics.rs` (new file, ~3 tests):
  1. Tampered ms1 (1-byte mutation in payload): assert checks[1] `ms1_entropy_match` has `passed=false`, `expected/actual/diff_byte_offset` populated.
  2. Tampered mk1 xpub (1-byte mutation in card): assert checks[3] `mk1_xpub_match` has the same forensic-field shape.
  3. Watch-only verify: assert checks[0] and checks[1] (`ms1_decode`, `ms1_entropy_match`) have `passed=true`, `decode_error="skipped: watch-only slot"`.
- Multisig forensic test deferred to v0.4.6+ (scope-safety; the multisig path is structurally novel in P.4).

**Verification gate:** new file's 3 tests pass; existing tests still pass.

### Phase L — Helper-foundation cleanups (L-1 + L-2)

Lands alongside P.3-P.5 (specifically: with the commit that flips `#[allow(dead_code)]` off, since both nits are in helper-foundation code that becomes live).

- **L-1:** in `emit_verify_checks` doc-comment near line 1882, change `§5.8` → `§5.7` (the watch-only sentinel discrimination paragraph).
- **L-2:** in `emit_verify_checks` single-sig branch, line 2017-2019 (the `MkField::Multi` early return arm), replace with `unreachable!("single-sig branch reached MkField::Multi — caller invariant violation")`. Add a corresponding caller-side comment in `run_full` confirming the invariant.

**Verification gate:** clippy passes (`unreachable!` is the canonical idiom for "compiler-can't-prove-but-runtime-can't-reach").

### Phase R — Release prep

- Remove `#[allow(dead_code)]` from `emit_verify_checks` (it's now live).
- CHANGELOG.md `[0.4.5]` entry describing P.3-P.7 + L-1 + L-2.
- Cargo.toml version bump 0.4.4 → 0.4.5.
- design/FOLLOWUPS.md: mark `verify-bundle-helper-call-sites-rollout-v0.4.5`, `verify-bundle-helper-foundation-cleanup-v0.4.5`, and `verify-bundle-9-3plus6n-descriptor-mode-parity` as resolved (cite shipping commits).
- Final cross-phase architect review (transcript-only).
- Tag `mnemonic-toolkit-v0.4.5` + GitHub release.
- Update memory: `mnemonic_toolkit_v0_4_state.md` + MEMORY.md index entry.

## Phase ordering & dependencies

```
P.3 (run_full + run_watch_only)
  → P.4 (multisig helper expansion + run_multisig)
    → P.5 (descriptor_mode_verify_run)
      → P.6 (test migration)
        → P.7 (forensic integration tests)
          → L (helper cleanup, lands with the dead_code-flip commit; can be folded into R)
            → R (release prep)
```

Each phase: implement → cargo test → per-phase architect review until 0C/0I.

## Scope reductions to consider during execution

If reviewer flags any phase as too risky for the v0.4.5 window:

- **P.4 multisig** is the highest-risk phase (helper grows a structurally novel path). Fall-back: keep `run_multisig` calling its current 3-check ladder; defer to v0.4.6 with FOLLOWUP `verify-bundle-helper-multisig-rollout-v0.4.6`. P.5 and P.7 still proceed (P.5 single-sig descriptor path is fine; multisig descriptor paths inherit P.4's TODO stub).
- **P.7 forensic tests** can be reduced to 1 happy-path + 1 tampered test if scope tightens.
- **P.6 test migration** is mechanical and not deferrable (cli_json_envelopes.rs will fail to compile against the new check-name vec otherwise).

## Out-of-scope for v0.4.5

- `legacy-cli-flag-deletion` (still v0.5).
- `unified-slot-xprv-resolution-needs-ms-codec-extension` (still v0.5+).
- `unified-slot-xpub-missing-path-origin-path-null` (still v0.4-nice-to-have).
- `bip388-distinctness-path-normalization-phase-b-decision` (still v0.4-nice-to-have).
- Multisig forensic-field integration tests (push to v0.4.6 if P.4 lands).

## Estimated test-count delta

- v0.4.4 baseline: 244 lib + integration.
- P.3 + P.4: +1 helper unit test (multisig 3+6N shape).
- P.7: +3 integration tests (forensic shape).
- Net: 248 tests at v0.4.5 ship.

## Final cross-phase review gate

This plan is itself an artifact subject to architect review per `feedback_iterative_review_every_phase`. Dispatch `feature-dev:code-architect` on this file before ExitPlanMode-equivalent (i.e., before starting P.3 execution). Iterate until 0C/0I.
