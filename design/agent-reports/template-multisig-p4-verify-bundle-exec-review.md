# P4 — VERIFY-BUNDLE (multisig/general template completion, #28 phase 2) — per-phase R0 EXECUTION review (opus architect, verbatim)

> Reviewer: opus architect (Opus 4.8, 1M) — per-phase R0 EXECUTION review of P4, `feature/bundle-md1-template-multisig`, commits `aaa67b74` (feat) + `1c553ea0` (test). Reviewed delta `git diff 9d17d35c..1c553ea0` over `cmd/restore.rs`, `cmd/verify_bundle.rs`, `tests/cli_verify_bundle_md1_template_multisig.rs`. Source-verified against md-codec 0.37.0 (the registry crate the build actually resolves), SPEC §5/§2/§4.2a/§6, the P3a-r2 + P3b reviews, and the P6 plan-doc. Tests + clippy run by me on HEAD=`1c553ea0`.

**Verdict: GREEN — 0 Critical, 0 Important.**

P4 may advance to P5. What follows is the evidence per the seven mandated checks, plus two Minors (non-blocking).

---

### 1. Refactor preserves restore's funds-safety semantics EXACTLY (no regression by construction)

I performed a normalized semantic diff of the completion core suffix (cosigner-parse onward) pre (`9d17d35c`) vs post (`1c553ea0`), rewriting `args.` → `ctx.` to isolate the genuine deltas. **The only differences are arg-struct neutralization** — no floor/gate was dropped, weakened, or reordered:

- `--own-account-max` I-1 gate: moved from the restore wrapper into the core (`restore.rs:1480`, `if ctx.own_account_max.is_some()`), message byte-identical. Now fires for **both** surfaces. Restore wrapper no longer carries it (the wrapper comment at `restore.rs:1397-1399` documents the move); verify passes `own_account_max: None` (`verify_bundle.rs:944`) so verify can never over-supply.
- Floor 1(ii) under/over-supply (`pool.len() < n` / `> n`), Floor 2 `reject_duplicate_keys`, the `--account` duplicate-account pre-check, `realized_s = perm_count_u128(n,n)` with `debug_assert_eq!(pool.len(), n)`, id/addr mode select, the sorted-shape carve-out, `permutation_search`, and the `None`/`Ambiguous`/`Unique` resolution arms are **all present and in the same order** in the post core (grep-confirmed at `restore.rs:1591/1600/1611/1623/1660-1680`).
- **C1 invariant holds.** `build_candidate` (`restore.rs:1683-1700`) builds triples directly from `pool[pi]` and calls `build_keyed_template_descriptor(d, &triples)`; the carried template `path_decl` is never read on the completion path. `grep compute_default_origin_path` over both files returns exactly one hit — `verify_bundle.rs:1373` — which is inside `run_full`'s **keyed** wallet-policy path (`is_non_canonical` path-decl rebuild), NOT the completion path. Own origins come only from `own_origin_from_family` (`restore.rs:1582/1590`) with the lazy `canonical_fallback`. Confirmed `build_keyed_template_descriptor` (`synthesize.rs:317-320`) clones `keyless_template.use_site_path`, `.tree`, and `.tlv.use_site_path_overrides` — never deriving the tree from anything but the supplied template.
- The seed-resolution prefix (argv-leak advisory, stdin-coexist gate, `@env:`/stdin handling, the seed-source node gate) was extracted verbatim into `resolve_template_completion_seed` (`restore.rs:1334-1429`); restore and verify call it with their own `no_from` ModeViolation. The `&mut &mut *stdin` reborrow (`restore.rs:1352/1361`) is the correct unsized→sized adaptation so the `<R: Read>` stdin helpers still monomorphize. Restore's single-sig template suite (which shares this helper) and the multisig restore suite both pass unchanged (see §6).

### 2. verify-bundle's completion is the SAME engine with matching parity

`verify_multisig_template` (`verify_bundle.rs:892`) imports and calls `complete_multisig_template`, then recomposes via `candidate_descriptor_string` (`restore.rs:1955`, now `pub(crate)`) — the identical #25 multipath engine restore emits with. The parity test `verify_bundle_canonical_multisig_template_id_search_ok` is a **real three-way byte-equal assertion** on identical inputs (`md1`, `--from phrase=SEED_A`, `--account 0`, `--expect-wallet-id id`, cosigner `mk1_b`): `j["first_receive"] == golden[0]` AND `j["first_receive"] == restore_addr`. The golden (`golden_addresses`, test:211-248) is built with `miniscript::Descriptor::from_str(...).derive_at_index().address()` — rust-miniscript, entirely outside the toolkit/md-codec synth path. Not vacuous.

### 3. Binding soundness / non-vacuity

I verified the implementer's key finding **at source** in the resolved crate `md-codec-0.37.0/src/identity.rs:71-104`: `compute_wallet_descriptor_template_id` hashes ONLY `use_site_path.write()` + `write_node(tree)` + the `UseSitePathOverrides` TLV. The doc (lines 49-53) and the body confirm it **excludes** the `Fingerprints` TLV, the origin-path-decl, and never touches `pubkeys`/xpubs. So the template-id is key-invariant AND identical across different cosigner SETS of the same shape — exactly as claimed.

Consequences, both verified:
- (a) A successful completion **always** shares the template-id with the supplied md1 **by construction** — `build_keyed_template_descriptor` clones the supplied `d`'s tree + use_site_path (synthesize.rs:317-320), the only template-id inputs. Therefore `md1_template_match` can never fail on a completed wallet of the same md1, so it can never be the **sole** funds-safety gate. The implementer's comment block (test:476-489) states this accurately, and the cross-mix test is correctly built on a **shape** difference (`wsh-multi` md1 + `wsh-sortedmulti` recorded id) so the binding's non-vacuity is exercised via search NO-MATCH.
- (b) The real funds-safety boundary — the completion search resolving the unique correct assignment, with `None`/`Ambiguous` → refuse — is **the same code** on both surfaces (single shared core). `verify_bundle_multisig_template_wrong_cosigner_no_match` (outsider at @1) and `..._binding_cross_mix_fails` both assert `status.code() != Some(0)` AND absence of `"result":"ok"` in stdout — genuine refusals, ran green. Defense-in-depth: even the impossible-by-construction `md1_template_match:false` would set `any_fail → exit 4`, never silent OK (`verify_bundle.rs:996,1027`). The verify search boundary is **not weaker** than restore's — it is byte-identical.

`mk1_template_stub_bind` inverts the **same** `derive_mk1_chunk_set_id_for_slot` (`synthesize.rs:90`) that `bundle.rs:1283` uses to emit the stubs — sound, not coincidental. `None` when no `--mk1` supplied (skipped; completion still gates).

### 4. The 7 new flags match restore's semantics exactly

All 7 new `#[arg`s in the diff: `--from`, `--cosigner`, `--search-address`, `--search-addr-min` (default 0), `--search-addr-max` (default 20), `--search-chain` (value_enum, default Receive), `--accept-search-time`. Defaults and types are **byte-identical** to restore (`restore.rs:146/153/157/165`), and `--search-chain` **reuses** `crate::cmd::restore::CliSearchChain` (verify_bundle.rs:50) rather than re-declaring an enum. Parsers are the shared `resolve_template_completion_seed` / `complete_multisig_template` / `chunk_set_id_extract` (`format.rs:355`) / `CliSearchChain::to_scope` — no divergent re-implementation. `--account` stays `u32` (verify_bundle.rs:64) → `own_accounts: vec![args.account]`; the single-own restriction is documented in both the flag help and an inline comment (verify_bundle.rs), and is fail-safe: a wallet needing ≥2 own slots would NO-MATCH (refuse), never silently complete wrong — multi-own is correctly restore-only this cycle.

### 5. Lockstep recorded, not silently owed

The 7 flags exactly match the P6 schema-mirror list enumerated in `design/IMPLEMENTATION_PLAN_bundle_md1_template_multisig_2026-06-20.md:47` (`--from`/`--cosigner` on verify-bundle + `--account`/`--own-account-max`/`--search-address`/`--search-addr-min/max`/`--search-chain` + dropdown values/`--accept-search-time`). No flag added that is NOT on the P6 list. Deferral of the GUI schema-mirror + manual to P6 is the planned single-coupled-cycle behavior. The lagging-indicator risk is contained because it ships in the same cycle.

### 6. Regression / build evidence (all run by me on HEAD `1c553ea0`)

- `cli_verify_bundle_md1_template_multisig`: **6 passed** ✓
- `cli_restore_md1_template_multisig`: **25 passed** ✓ (behaviorally identical)
- `cli_bundle_md1_template_multisig`: **13 passed** ✓
- `--lib`: **158 passed**, 3 ignored ✓
- `prop_backup_restore_roundtrip`: **13 passed** ✓
- `cli_verify_bundle_md1_template` (single-sig template verify — must be untouched): **6 passed** ✓
- Every other `cli_verify_bundle_*`: entropy_slot 5, forensics 3, full 4, hashlock_and_bip388 4, multi_cosigner_mk1 8, seedqr_slot 2, watch_only 10 — all ✓
- `cli_restore_md1_template` (shares the refactored seed helper): **6 passed** ✓
- `cargo test -p mnemonic-toolkit --no-run`: all integration targets compile clean.
- `cargo clippy -p mnemonic-toolkit --tests --bins --lib -- -D warnings`: clean after a forced recompile of both touched files (4.07s, 0 warnings).
- `git diff --name-only 9d17d35c..1c553ea0`: 3 files, **excludes mlock.rs**; **no error.rs change** → no new `ToolkitError` variant (alphabetical-ordering rule N/A).

### 7. Panic / borrow scan of the funds-safety path

No attacker/user-controlled panic introduced. `serde_json::to_string(&json).unwrap()` (verify_bundle.rs JSON emit) serializes an owned-Value of strings/bools/numbers — infallible, and identical to `verify_singlesig_template`'s pattern. `template_id[0..3]` indexing in `check_mk1_template_stubs` is over `&[u8; 16]` (statically in-bounds). The two `.as_deref().unwrap()` (restore.rs:1698/1723) are pre-existing and guarded by the `id_search`/`addr_search` `.is_some()` booleans. All errors (`from_str` parse, `derive_receive_addresses`, the "no first receive" case) are `?`/`.ok_or_else(...)?`-propagated → clean refusal. The `&seed.entropy` borrow holds the mlock pins for the ctx's lifetime via `TemplateSeed` owned by the caller — no use-after-free; pins drop after emit.

---

### Minor (non-blocking; defer to P5/P6 at author's discretion)

- **M-1 (test coverage gap, not a code defect):** the new multisig-template verify suite asserts the byte-equal golden/parity for `--expect-wallet-id` (id-search) and `--search-address` (address-search) on the **canonical sortedmulti** and the **general** shape, but does **not** add a verify-side golden assertion for an **order-DEPENDENT** `wsh-multi` happy-path (only the cross-mix negative uses wsh-multi). Restore's suite covers wsh-multi positively, and the engine is shared, so funds-safety is not at risk — but a verify-side wsh-multi id-search golden would close the parity matrix symmetrically. Reasonable to fold into P5's differential breadth.
- **M-2 (doc-only):** `complete_multisig_template`'s rustdoc says "Restore behavior is byte-identical to the pre-factor inline body" — true and verified — but the `stdout` parameter no longer exists on the core (emit moved to the caller); the sentence "`stderr` carries the explicit-mode warning + the search progress line" is accurate. No action required; noting for the reader that the core is now emit-free.

Both Minors are explicitly **not** gate-blocking. The hard gate (0C/0I) is satisfied: the refactor preserved every floor by construction, the verify search boundary equals restore's, the binding/parity tests are non-vacuous, and the full regression set is green with a clean -D warnings clippy. **P4 is GREEN — advance to P5.**
