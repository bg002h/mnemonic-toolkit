# P3b — RESTORE completion (GENERAL/thresh templates, #28 phase 2) — per-phase R0 EXECUTION review (opus architect, verbatim)

> Reviewer: opus architect (adversarial, source-verified; tests + clippy run + an independent end-to-end CLI reproduction of the wrong-family/positive-control pair + the three floor messages). HEAD `6fbc96c45501ed53694fa730f65a07e4dd0e05d1`, branch `feature/bundle-md1-template-multisig`. Reviewed delta: `git diff 92ddbfbd..6fbc96c4 -- crates/mnemonic-toolkit/src/cmd/restore.rs crates/mnemonic-toolkit/tests/cli_restore_md1_template_multisig.rs`; the code+test commit is `6fbc96c4^..6fbc96c4` (only `restore.rs` + the integration test; the `.gitignore`/agent-report files in the wider range are the P3a-persist commits `c79e947a`/`2249e433`). Verified against md-codec **0.37.0** (`Cargo.lock` checksum `fec7cad2…`), mk-codec 0.4.0, rust-miniscript 13.x.

**Verdict: GREEN — 0 Critical, 0 Important.**

The funds-safety surface (silent-wrong-wallet class) is sound for the general path. I-A is closed and **proven non-vacuous by execution** (BIP-48 own origin → exit-4 `✗ NO MATCH`; the same wallet at the correct BIP-84-via-cosigner-family → exit-0 match). C1 carried-origin-never-loaded holds for the harder Divergent case — structurally (the builder never reads `keyless_template.path_decl`) and by a tamper test that survives a wrong `m/99'/0'/N'` Divergent carried origin. The lazy `canonical_fallback` is a pure refactor that preserves the three-way resolution order, introduces no panic, and does not regress the canonical (P3a) path. The routing widening fails safe on unsupported/keyed shapes. All floors are non-vacuous and hit the intended gate. Counts match: 25 / 13 / 158 / 13, tracked-surface clippy exit 0, mlock.rs untouched, no new clap flag. **The gate may advance to P4.**

---

## 1. I-A-for-general — CLOSED, and proven non-vacuous by execution

The own origin for the general path is built at the cosigners' **actual** family, never BIP-48. Mechanism (`restore.rs:1340-1344` `cosigner_family` = first cosigner's mk1 `origin`; `:1404-1416` precedence; `own_origin_from_family` `:1154-1163` substitutes the `--account` into path-component index 2 of *whatever family the cosigner carries*). For degrade2's BIP-84 cosigners (`84'/0'/1'`, `84'/0'/2'`) the own `@0` at `--account 0` derives at `84'/0'/0'` — correct. `compute_default_origin_path` is **never referenced anywhere in `restore.rs`** (grep: zero hits) — the completion path provably cannot force BIP-48; the only BIP-48 source is the lazy `canonical_fallback`, which `canonical_origin` returns `None` for a general policy, so it errors with a `--origin`-naming message instead of silently substituting BIP-48.

**Adversarial probe (the load-bearing evidence).** I reproduced the test's exact setup end-to-end against the built binary, plus a positive control:
- Wrong family `--origin m/48'/0'/0'/2'` (what `compute_default_origin_path`/`canonical_origin` would yield) → **exit 4, stderr `✗ NO MATCH`**. This is a genuine permutation-engine miss (the `RestoreMismatch` arm at `restore.rs:1614-1619`), NOT a routing/floor/prefix refusal — the general template was genuinely *routed to completion*.
- Same setup, correct `--account 0` (BIP-84 own origin via cosigner-family) → **exit 0, `✓ wallet-id (completed): bda5ff40…`**.

The exit-4-vs-exit-0 contrast on the single own-origin-family variable demonstrates `general_policy_wrong_family_no_match` is non-tautological: a BIP-48-sourced completion fails the search; the BIP-84-per-slot build is what's implemented. `general_policy_id_search_completes_to_golden` (`:1264`) also asserts at runtime `canonical_origin(&decoded.tree).is_none()` and `!decoded.is_wallet_policy()` and `n==3` — confirming the shape is general + keyless before relying on it.

## 2. C1-Divergent — carried-origin-never-loaded holds

`build_keyed_template_descriptor` (`synthesize.rs:283-327`) builds `path_decl.paths` **fresh** from `slots[i].origin` (`:294-300`; `Divergent` when origins differ, exactly the three-distinct-BIP-84 degrade2 case) and writes it into the output (`:313-315`). The function body contains **no read of `keyless_template.path_decl`** (the only `path_decl` tokens are the fresh-build and the output-write). So the carried/stale Divergent path_decl — which for a Divergent template has real per-`@N` content that *could* leak — is structurally unreachable from `compute_wallet_policy_id`/derivation. `general_policy_carried_origin_never_loaded` (`:1167`) tampers the carried Divergent path_decl to wrong non-canonical `m/99'/0'/N'`, re-encodes via `md_codec::chunk::split`, asserts the tamper target is general, and still completes to the independent 2-address golden — non-vacuous (a build that read the carried origin would derive at `m/99'/…` → different addresses → assert fail). Verified the test passes.

## 3. Lazy `canonical_fallback` — safe; no canonical regression, no panic

The previously-eager `canonical_origin(...).ok_or_else(...)?` (which would error "not canonical-origin" for *every* general policy before checking `--origin`/cosigner-family) is now a closure (`restore.rs:1365-1374`) invoked only at the two use-sites (`:1409` cosigner-family-but-`own_origin_from_family`-returned-None; `:1414` all-own-no-`--origin`-no-cosigner). (a) The canonical path still resolves the BIP-48 fallback when reached — the all-own/no-cosigner/no-`--origin` arm (`:1411-1415`) computes `fb = canonical_fallback()?` then substitutes the account, identical to pre-fold modulo laziness; P3a's canonical suites (18 tests incl. `default_family_bip87_*`, `multi_account_own_resolves_both_slots`) stay green. (b) No `.unwrap()`/`.expect()`/`panic!` introduced — the closure returns `Result`, both use-sites `?`-propagate, and a general policy in the all-own-no-origin state returns a clean `BadInput` (exit 1) naming `--origin m/<purpose>'/<coin>'/<account>'`, never a panic. (c) Resolution order `--origin → cosigner-family(account-substituted) → fallback` is preserved exactly (`:1404-1416`).

## 4. Routing widening — fails safe

`is_multisig_template = !d.is_wallet_policy() && d.n >= 2` (`:312`) routes any keyless n≥2 template to completion. `is_wallet_policy()` (md-codec 0.37.0 `encode.rs:50`) is `matches!(pubkeys, Some(v) if !v.is_empty())` — a KEYED wallet-policy md1 is `true` → excluded → falls through to `run_multisig` (`:317`), correct. For a hand-crafted **unsupported** keyless shape (tr(sortedmulti_a)/hardened): every evaluator branch fails safe — the id evaluator's `compute_wallet_policy_id` error → `Err(_) => false` (`:1543`); the address evaluator's `build_candidate`/`candidate_descriptor_string`(→`faithful_multisig_descriptor`→`to_miniscript`, which errors on root `SortedMultiA`)/`from_str`/`script_pubkey_at` each `let Ok(..) else { return false }` (`:1578-1590`) — so no permutation matches → `SearchOutcome::None` → `✗ NO MATCH` + `RestoreMismatch` exit 4 (refuse), never a silent completion. The no-mode arm refuses with `ModeViolation` exit 2 (`:1601-1608`).

## 5. Floors — non-vacuous, intended gate (CLI-captured)

I captured the actual refusal messages end-to-end:
- **no-from** → exit 2 `ModeViolation`: "restore of a keyless MULTISIG TEMPLATE md1 requires --from" — names `--from`; message proves the general template reached completion (floor 1(i)).
- **unsupplied-slot** → exit 2: "not enough keys to fill every cosigner slot … must EQUAL the wallet's cosigner count" — the every-slot floor (`pool.len() < n`, `:1459-1466`), NOT a routing reject.
- **duplicate-cosigner** → exit 4 `RestoreMismatch`: "positions 1 and 2 are identical … collide on both address and id" — fires BEFORE the search (floor 2).

The `!stderr.contains("template-only")` hardening on the first two is correct and necessary: none of these messages contains "template-only", so the assertion genuinely confirms the floor fired (general routed to completion), not the pre-P3b routing refusal.

## 6. Regression + build evidence (run by reviewer)

- `cargo test -p mnemonic-toolkit --test cli_restore_md1_template_multisig` → **25 passed; 0 failed** (18 P3a + 7 P3b).
- `--test cli_bundle_md1_template_multisig` → **13 passed; 0 failed**.
- `--lib` → **158 passed; 0 failed; 3 ignored**.
- `--test prop_backup_restore_roundtrip` → **13 passed; 0 failed** (no #25 full-policy regression).
- `cargo clippy -p mnemonic-toolkit --tests --bins --lib -- -D warnings` → **exit 0**, clean.
- `git diff --name-only 92ddbfbd..6fbc96c4` does NOT include `mlock.rs`; the code commit (`6fbc96c4^..6fbc96c4`) is only `restore.rs` + the test file. No new clap flag: grep of the `restore.rs` diff for `#[arg`/`long =`/`long(`/`short =` → zero additions. No GUI schema-mirror / manual lockstep triggered.
- The headline differential gate (`general_golden_addresses`, `:1251`) is a genuinely **independent** rust-miniscript derivation (`Descriptor::<DescriptorPublicKey>::from_str` → `into_single_descriptors` → `derive_at_index` → `.address(Bitcoin)`), NOT an md-codec reconstruction; completed addresses asserted byte-equal. The shape is order-dependent (`or_i` ∉ `is_order_independent_shape`'s sorted set → the 3! = 6 search is NOT collapsed).

## Minor (non-blocking, NOT in the reviewed diff — does not gate P3b)

- **m-1 (carried from P3a, unchanged disposition).** `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` is still RED with 2 `clippy::format_collect` errors in `crates/mnemonic-toolkit/examples/{idsearch,addrsearch}_bench.rs`. P3b's `.gitignore` addition (`crates/mnemonic-toolkit/examples/*_bench.rs`, commit `c79e947a`) keeps these untracked but does **not** keep cargo from compiling them under `--all-targets` (gitignore affects git, not cargo target discovery). These are untracked developer scratch outside the reviewed surface; the gate-relevant tracked-surface clippy (`--tests --bins --lib`) is exit 0. To fully silence `--all-targets`, delete the two scratch files or move them out of `examples/` (or formalize as `#[ignore]` benches per plan §9). No code-under-review change required.

## Closing

**GREEN — 0 Critical, 0 Important.** The two load-bearing changes (routing widening + lazy fallback) are correct and minimal; the rest of the `restore.rs` diff is rustfmt churn. I-A is closed and execution-proven non-vacuous; C1 holds for the Divergent case both structurally and under tamper; the lazy fallback regresses nothing and cannot panic; unsupported/keyed shapes fail safe to refuse (never silent-wrong-wallet); all floors are non-vacuous and hit the intended gate. Tests 25/13/158/13 green, tracked-surface clippy clean, mlock untouched, no schema/manual lockstep. The only loose end (m-1) is untracked scratch outside the diff, carried from P3a. **P3b may close; advance to P4.**
