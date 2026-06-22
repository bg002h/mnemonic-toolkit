> Reviewer: opus architect — per-phase R0 EXECUTION review of P4 (verify-bundle exposure of the own-account subset-search flags)

**Verdict: GREEN — 0 Critical, 0 Important.**

Commit `22cff1ed` (feat(verify-bundle): P4 expose own-account subset-search flags). Branch `feature/own-account-subset-search`. The gate advances to P5.

P4 adds `--own-account-max` + `--search-cosigner-subset` to `VerifyBundleArgs` and threads them into the SAME `MultisigCompletionCtx` / `complete_multisig_template` engine `restore` uses. verify == restore over the subset-search by construction (one engine, not a reimplementation), and the parity is proven non-vacuous by an independent rust-miniscript golden, a live byte-compare, and a mutation test. Funds-safety class clears: a verify that completes a DIFFERENT wallet, or silent-OKs a wrong one, would be caught.

---

## 1. Parity non-vacuity — the heart (verify == restore == INDEPENDENT golden)

**PASS — and proven non-vacuous three independent ways.**

The headline test `verify_bundle_own_account_max_completes_at_nonzero_account` (`tests/cli_verify_bundle_md1_template_multisig.rs:721`) builds a 2-of-2 with the OWN key at **account 3** (`cos = &[(SEED_A, 3u32), (SEED_B, 0u32)]`), runs verify-bundle with `--own-account-max 5`, and asserts `first_receive == golden[0] == restore_addr`.

- **The golden is genuinely INDEPENDENT** (`golden_addresses`, `:205-242`): it builds `wsh(multi(2, [fp/48h/0h/3h/2h]xpub/<0;1>/*, ...))` and parses it with `Descriptor::<DescriptorPublicKey>::from_str` → `into_single_descriptors()` → `derive_at_index(0)` → `address(Bitcoin)`. This is rust-miniscript, NOT md-codec — the two address-derivation paths are disjoint, so agreement is a real cross-check, not a tautology.
- **Non-vacuity of the NON-ZERO account** — I computed the golden myself via a throwaway example bin against the workspace's pinned `bitcoin 0.32` / `miniscript 13` / `bip39 2`:
  - own@3 (the asserted golden): `bc1qqz0ggyuhz02f88496t536s5e5d2jpu8h0zk6lzy2rqlyycyl8a5sqgpy4y`
  - own@0 (the vacuous "always-account-0" answer): `bc1q2a4vm7ww7v0c02qerrgg0znr4ck4kez2la82kzpm9gturx68q4nsfdl000`
  These are **distinct**. An implementation that ignored `--own-account-max` and used account 0 would produce the second and FAIL the assertion. The test therefore binds to genuine account-3 resolution found via the range.
- **Mutation test (the decisive funds-safety probe).** I reverted the threading in place (`own_account_max: args.own_account_max → None`, `search_cosigner_subset: args.search_cosigner_subset → false`) and re-ran the file: the headline parity test went **RED**, along with `verify_bundle_search_cosigner_subset_completes`, `..._ceiling_refuses`, and `..._at_account_mutex_with_explicit_cosigner_assignment`. Restoring the file returned all 14 to GREEN (and `git diff` vs HEAD is empty — clean restore). This proves the tests are wired to the threading, not passing incidentally: drop the thread and verify silently fails to complete the right (account-3) wallet → caught.
- **Live end-to-end byte-compare.** I drove the CLIs directly (bundle → verify-bundle `--own-account-max 5` AND restore `--own-account-max 5`, same md1/mk1/wallet-id). Both produced **byte-identical** behavior — even where my hand-built harness inputs mis-matched the recorded id, verify and restore returned the IDENTICAL exit code (4) and the IDENTICAL `✗ NO MATCH` error. Parity holds in success AND in failure: verify never diverges from restore. (My manual NO-MATCH was a harness card-derivation artifact, not a feature defect — the in-tree test's own bundle→verify→restore byte-compare passes, and the mutation test above independently confirms the success path.)
- **Anti-vacuity of the refusal direction.** `verify_bundle_search_cosigner_subset_wrong_cosigner_no_match` (`:980`) over-supplies a pool that LACKS a true cosigner (one outsider + C at a WRONG account) and asserts exit ≠ 0 AND no `"result":"ok"` in stdout — a wrong wallet can never silent-OK exit 0.

## 2. Shared-engine threading — no divergent reimplementation

**PASS.** `verify_multisig_template` (`src/cmd/verify_bundle.rs:883`) constructs the SAME `MultisigCompletionCtx` defined in `restore.rs:1154` and calls the SAME `complete_multisig_template` (`restore.rs:1463`) at `verify_bundle.rs:907`. The two new fields are passed straight through: `own_account_max: args.own_account_max` / `search_cosigner_subset: args.search_cosigner_subset` (`:897-898`), replacing the old hardcoded `None` / `false`. `own_accounts` stays SCALAR: `vec![args.account]` (`:887`) — NOT widened to a Vec (restore's is `args.account.clone()`, a list; verify keeps the single scalar and over-supplies own-only via the range). Inside the shared engine the own pool is built from `0..K` at `restore.rs:1686-1691`, identical for both callers. The keyed `md1_template_match` / `mk1_template_stub_bind` binding path (`verify_bundle.rs:911-927`) is UNTOUCHED. verify's subset-search IS restore's subset-search.

## 3. Mutex + refusals mirror restore

**PASS — all inherited from the shared engine, all test-covered on verify-bundle.**
- `--own-account-max` ALONE passes: `verify_bundle_own_account_max_alone_passes` (`:856`) — GREEN, and stayed GREEN under mutation (pure clap guard). The `conflicts_with = "account"` (`verify_bundle.rs:96`) does not fire on the default-supplied `--account` (`:63`, `default_value = "0"`) — the I-4 subtlety is correct.
- `--account N --own-account-max K` → clap conflict exit 64: `verify_bundle_account_and_own_account_max_conflict` (`:885`).
- `@N=` ⊕ subset-search → BadInput: `verify_bundle_own_account_max_at_account_mutex_with_explicit_cosigner_assignment` (`:909`); fires the shared gate at `restore.rs:1612-1618`. RED under mutation → genuinely exercised.
- §6 ceiling K_own > 256 refuses: `verify_bundle_own_account_max_ceiling_refuses` (`:943`); `restore.rs:1486` (`OWN_ACCOUNT_MAX_CEILING`). RED under mutation. (k==0 floor also lives at `restore.rs:1481`.)
- `--search-cosigner-subset` opt-in completes over an over-supplied pool with restore parity: `verify_bundle_search_cosigner_subset_completes` (`:783`). RED under mutation.

## 4. Off-by-default + regression

**PASS.** Full suite re-run by me: **3474 passed / 0 failed / 15 ignored** (summed across all binaries via `test result:` grep + awk). This matches the orchestrator's independent count and **refutes the implementer's reported "597 passed"** — 597 is a single test binary's count (the partial run the commit message recorded), not the full `-p` run. The true full-suite GREEN gate is satisfied. Off-by-default: without the flags `own_account_max` defaults to `None` (clap `Option<u32>`) and `search_cosigner_subset` to `false` (clap `bool`), reproducing the pre-P4 ctx exactly. Existing untouched verify tests GREEN: `cli_verify_bundle_md1_template_multisig` (14, of which 7 pre-existing), `cli_verify_bundle_md1_template` single-sig (6).

## 5. Clean

**PASS.** `cargo clippy -p mnemonic-toolkit --tests --bins --lib -- -D warnings` → Finished, zero warnings. `git diff --name-only 22cff1ed^..22cff1ed` = exactly the 2 declared files (`src/cmd/verify_bundle.rs`, `tests/cli_verify_bundle_md1_template_multisig.rs`). No mlock, no Cargo.toml/Cargo.lock change, no new dependency, no new `ToolkitError` variant.

## 6. Lockstep debt recorded

**PASS (deferral to P6 acceptable).** P4 adds 2 NEW clap flag NAMES on verify-bundle — `--own-account-max` (`verify_bundle.rs:96`) and `--search-cosigner-subset` (`:109`). With restore's `--search-cosigner-subset` from P3, that is 3 flag names owing a GUI `schema_mirror` update + a `docs/manual/src/40-cli-reference/` update. The commit message lists them and explicitly defers the lockstep to P6 ("P6 lockstep (GUI schema_mirror + manual) deferred per the plan"). Acceptable within a single coupled cycle — but P6 MUST land all 3, or the lagging `schema_mirror` gate will fire on the next GUI pin bump (the v0.27.x historical case).

---

## Minor (non-blocking — no fix required before P5)

- **M-1 (cosmetic fmt churn, src/cmd/verify_bundle.rs `helper_tests`):** P4's edit ran rustfmt over two pre-existing non-canonical spots — reordering `use bitcoin::bip32::DerivationPath;` below the `crate::` imports (`:4060-4062`) and line-wrapping a `md_codec::chunk::split(...)` call (`:4302`). Confirmed pre-existing (present in `22cff1ed^`), confined to the same file P4 edits, a legitimate rustfmt normalization, and NOT mlock.rs — so it is not a gate violation and not cross-file. Noted only because CLAUDE.md flags fmt churn; no action needed.

## Conclusion

0 Critical, 0 Important. verify == restore (same engine, threaded straight through — proven by mutation test, independent golden, and live byte-compare), the non-zero own account is genuinely resolved (not vacuously account-0), no wrong wallet can silent-OK, all mutexes/refusals mirror restore and are exercised, off-by-default reproduces pre-P4 behavior, the full 3474/0 suite is GREEN (the 597 was a mis-reported partial), clippy clean, the 2-file diff is exactly as declared, and the 3-flag P6 lockstep is recorded. **GREEN — the gate advances to P5.**
