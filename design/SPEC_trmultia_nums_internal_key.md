# SPEC — Cycle A: `toolkit-trmultia-nums-internal-key` (emit NUMS for bundled tr-multisig)

**Cycle:** toolkit wire-conformance fix (FOLLOWUP `toolkit-trmultia-nums-internal-key`).
**Date:** 2026-06-08.
**Source SHA:** `origin/master` == local `HEAD` == `b642fbe`.
**Disposition:** toolkit **MINOR** — `v0.47.4 → v0.48.0` + `mnemonic-toolkit-v0.48.0` tag (this is a **wire-content change** to the **md1 AND mk1** cards emitted for bundled tr-multisig — the descriptor change propagates through `wallet_policy_id` → the stub that seeds both; R0-r1 M1).
**Recon:** `cycle-prep-recon-trmultia-friendly-canonicity-chunkmk1.md` + `cycle-prep-recon-trmultia-nums-design.md` (the design decision — user chose **A1**).
**Decision (user):** **A1** — ship NUMS as a standalone BIP-388 standards-conformance fix; it does **NOT** unblock restore (re-scope that FOLLOWUP).

---

## 0. What + why

`bundle` (via `template.rs::wrapper_node`) emits, for `tr-multi-a`/`tr-sortedmulti-a` templates, `Body::Tr { is_nums: false, key_index: 0 }` — i.e. cosigner @0 doubles as the taproot key-path internal key (`tr(@0, multi_a(@0,…,@n-1))`). This is non-standard: BIP-388 script-path-only multisig uses a provably-unspendable **NUMS** internal key (`tr(NUMS, multi_a(…))`, exactly what `export-wallet --taproot-internal-key nums` already emits). Fix: emit `is_nums: true`.

**Scope honesty (recon-established):** this is a **standards-conformance fix only**. It does NOT unblock `restore-multisig-taproot-reconstruction`: `tr-sortedmulti-a` is blocked UPSTREAM (rust-miniscript v13.0.0 has no `Terminal::SortedMultiA` fragment — `md-codec to_miniscript.rs:406-410` unconditionally errors, independent of `is_nums`); `tr-multi-a` already renders but its restore needs a separate `restore.rs` pre-gate lift. Item 5 re-scopes that FOLLOWUP accordingly.

---

## Item 1 — `template.rs::wrapper_node` TrMultiA/TrSortedMultiA arm

`crates/mnemonic-toolkit/src/template.rs:209` (CITATION FIX — `is_nums: false,` is at **`:209`**, NOT `:208`; `:208` is the comment line `// SHOULD emit is_nums: true here.`, the comment block is `:203-208`, and `key_index: 0` is `:210` — the as-filed `:208` was wrong, caught empirically when a `208s/...` edit no-op'd against the comment). Change `is_nums: false` → **`is_nums: true`**. **CAUTION:** the identical-indentation `is_nums: false,` at `:150` is a DIFFERENT arm (single-sig `Bip86` tr) and MUST NOT be touched — use a line-anchored edit (`209s/...`), not a global pattern. Keep `key_index: 0` (md_codec **ignores** `key_index` when `is_nums: true` — confirmed `md-codec validate.rs:85-96`: the `key_index >= n` → `NUMSSentinelConflict` check is gated on `!is_nums`, so `is_nums:true, key_index:0` is valid). Keep the leaf `indices: (0..n).collect()` unchanged (all n cosigners remain in the `multi_a`/`sortedmulti_a` leaf — only the internal key changes from @0 to the NUMS H-point).

**Empirically confirmed wire delta** (real `tr-multi-a` 2-of-2 bundle, decoded via `md decode`): BEFORE = `tr(@0/<0;1>/*, multi_a(2, @0/<0;1>/*, @1/<0;1>/*))`; AFTER = `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0, multi_a(2, @0/<0;1>/*, @1/<0;1>/*))` (the BIP-341 NUMS H-point; leaf unchanged). **Whole-bundle blast radius (R0-r1 M1):** the change shifts BOTH the **md1 AND the mk1** cards — the new descriptor → `compute_wallet_policy_id` → the 4-byte stub that seeds both the md1 and the mk1 `chunk_set_id` (all 4 md1 chunks + the mk1 set changed in the empirical run). Any full-bundle golden regen is a FULL re-bless, not just md1 lines.

Rewrite the `:203-208` comment: it currently says "currently use key_index:0 … FOLLOWUP `toolkit-trmultia-nums-internal-key` filed to revisit whether … SHOULD emit is_nums:true." Now it DOES — reword to state the templates emit the BIP-388 NUMS internal key (`is_nums:true`); cite the resolved FOLLOWUP + SPEC.

---

## Item 2 — flip the `template.rs:446` test lock

`template.rs:446-449` (a test asserting the wrapper_node shape) currently has `assert!(!is_nums, "TrMultiA wrapper currently uses key_index=0 (real key), not NUMS sentinel")` + `assert_eq!(key_index, 0)` + leaf-is-MultiA. Flip `assert!(!is_nums, …)` → **`assert!(is_nums, "TrMultiA wrapper now emits the NUMS internal key (BIP-388 script-path-only)")`**. Keep `assert_eq!(key_index, 0)` (template still sets it; ignored). Keep the leaf assertion.

---

## Item 3 — dispose of the ORPHANED tr-multisig goldens (R0-r1 I1 — they false-green)

**The premise "the suite identifies the changed fixtures" is FALSE.** The `tests/vectors/v0_2/tr-{multi-a,sortedmulti-a}-*-0-false-false.txt` md1 goldens are **orphaned** — NO test harness reads them (no `read_dir`/`insta::glob`/`WalkDir`; `cli_self_check.rs` reads only `bip84-…`; `cli_bundle_full.rs` reads only `vectors/v0_1/` single-sig; each `tr-*` vector's md1 prefix appears only in the vector file itself). So after the code change, `cargo test` reports **GREEN** while those vectors silently still encode the OLD `is_nums:false` wire — a stale-fixture-ship with zero NUMS-value assertion.

**Required (decisive, not "the suite finds them"):**
1. **Dispose of the orphaned `tr-{multi-a,sortedmulti-a}-*-0-false-false.txt` vectors.** Default: **regenerate** them to the new NUMS wire AND wire a harness test that reads + decode-asserts them (so they stop being dead). If regeneration+harnessing is out of proportion, **delete them as dead goldens** and say so. Decide explicitly; do not leave them stale.
2. The change shifts the WHOLE bundle (md1 + mk1 — M1) → any regenerated vector is a FULL re-bless (decode-verify each, never mass-`--bless`).
3. The live EXECUTION tests that bundle tr-multisig (`cli_restore_multisig.rs::tr_multisig_refused_exit2`, `cli_tr_bip48_advisory.rs` ×4) keep passing (their assertions — restore-refusal exit 2, the advisory string — are `is_nums`-independent), so they prove no-panic but do NOT pin `is_nums`; Item 4 is what pins it.

---

## Item 4 — MANDATORY gating characterization test (R0-r1 I1 — the ONLY thing that pins the new wire)

This is **required**, not "add/confirm" — it is the regression guard the orphaned goldens cannot provide.
- **`tr-multi-a` (the pin):** add a `#[test]` that bundles `--template tr-multi-a` (a real 2-of-N, e.g. the `cli_tr_bip48_advisory.rs` T24/P1 phrases), decodes the emitted md1, and asserts: (a) the internal key **`is_nums == true`** / renders to the NUMS H-point `50929b74…803ac0` (NOT `@0`); (b) the leaf still carries all n cosigners (`indices: 0..n`, i.e. `multi_a(k, @0,…,@n-1)`); (c) the first cosigner xpub in `tlv.pubkeys` is unchanged. `md_codec::to_miniscript_descriptor` renders `tr(NUMS, multi_a(…))` (MultiA supported `to_miniscript.rs:394-398`; NUMS via `build_nums_internal_key` `:161-165`). This is the empirically-proven before/after (Item 1) turned into a guard.
- **`tr-sortedmulti-a` (pin the limit):** a `#[test]` asserting the md1 encodes/decodes (wire round-trip OK, `is_nums:true`) BUT `to_miniscript` still errors on the `SortedMultiA` leaf (the rust-miniscript v13 gap) — so the upstream limit is pinned, not silently regressed.
- **md_codec validation:** the test path confirms the bundle-emitted `is_nums:true` md1 passes `md_codec` decode (no `NUMSSentinelConflict`).

---

## Item 5 — re-scope `restore-multisig-taproot-reconstruction` (FOLLOWUP)

Update its entry: trmultia-nums is now **resolved as a standalone conformance fix and does NOT unblock restore**. Its TRUE blocker is the **rust-miniscript v13.0.0 `SortedMultiA` gap** (`md-codec to_miniscript.rs:406-410`) for `tr-sortedmulti-a`, plus a `restore.rs` taproot-pre-gate lift for `tr-multi-a` (which already renders). Change its "blocked on `toolkit-trmultia-nums-internal-key`" to "blocked on the upstream rust-miniscript SortedMultiA fragment (+ a tr-multi-a pre-gate lift)." **(R0-r1 M3) name the specific lift target:** `restore.rs:777` (`d.tree.tag == md_codec::Tag::Tr` refuses ALL Tr md1 at the pre-gate, even the renderable `tr-multi-a`). **File a sibling md-codec FOLLOWUP** (not "consider") for the SortedMultiA-rendering gap, with a `Companion:` cross-cite per the CLAUDE.md cross-repo convention (mirror an entry in `descriptor-mnemonic/design/FOLLOWUPS.md`).

---

## Item 6 — docs/manual: POSITIVE verification (R0-r1 I2 — the manual ALREADY says NUMS)

**Reframed: the manual already documents the bundle as NUMS — this fix makes the CODE conform to already-shipped docs (a docs-vs-code-drift closure, a correctness win), NOT the other way round.** `docs/manual/src/30-workflows/33-taproot-multi.md` already states NUMS at `:13-14` ("NUMS internal key — a verifiable nothing-up-my-sleeve point"), `:50` (table row `nums | BIP-341 reference NUMS point`), `:54-58` ("NUMS variant — the *default* for tr-multi-a and tr-sortedmulti-a … the bundle embeds the BIP-341 reference NUMS point as the internal key"), `:83-85` (`tr(NUMS_POINT,sortedmulti_a(2,@0,@1,@2))`), and the `:70-81` `bundle --template tr-sortedmulti-a --self-check` example whose `:84` claim was **only true after this fix**.

**So: NO manual EDIT is needed** (confirmed: no stale @0-internal *bundle* prose exists — the technical-manual documents the wire mechanism generically; the `tr(@0/**)` hits in `45-foreign-formats.md` are BIP-86 single-sig template-mode, unaffected). **What IS required: positively VERIFY** that `33-taproot-multi.md:84` + the `:70-81` self-check example are now ACCURATE post-fix (the earlier `sed`-era empirical run already confirms a `tr-multi-a` bundle decodes to `tr(NUMS, …)`; do the equivalent for the documented `tr-sortedmulti-a` self-check). Record that the fix closed a latent manual-vs-code drift.

---

## 7. Verification
1. `cargo test -p mnemonic-toolkit` → all green (incl. the flipped `:446` lock + regenerated Item-3 fixtures + Item-4 round-trip).
2. `cargo build` + `cargo clippy --all-targets` clean.
3. **Decode-verify** a fresh `mnemonic bundle … --template tr-multi-a` md1: `md decode` shows the internal key is NUMS (`is_nums:true` / NUMS H-point), and the descriptor renders `tr(NUMS, multi_a(…))`.
4. **Wire-change confirmation:** the tr-multi-a bundle md1 bytes differ from the pre-fix bytes at the `is_nums` position (the intended change), and decode to NUMS.
5. No md-codec source change (md_codec already supports `is_nums:true` end-to-end). **No GUI `schema_mirror`** — `schema_mirror` gates clap flag-NAME parity + dropdown value-enums only (CLAUDE.md), and this changes neither (no flag/subcommand/value added/removed/renamed); it's a wire-output change. **No manual-CLI-surface mirror** (no `--help` surface change). The only manual touch is the positive verification in Item 6.

## 8. Ship plan
1. Apply Items 1-6.
2. Verify §7.
3. Bump `crates/mnemonic-toolkit/Cargo.toml` version `0.47.4 → 0.48.0`; update `Cargo.lock`.
4. `design/FOLLOWUPS.md`: flip `toolkit-trmultia-nums-internal-key` → resolved (A1 conformance fix); re-scope `restore-multisig-taproot-reconstruction` (Item 5).
5. Stage explicitly; commit (`git commit -F -`, Co-Authored-By). Push to `master` (CI `rust.yml` validates). Then tag `mnemonic-toolkit-v0.48.0` + push the tag (fires `install-pin-check` — incidentally closes the last v5 `checkout@v5` gap).
6. Memory.

### Out of scope
- The restore unblock (re-scoped to its true upstream blocker; Item 5).
- Any rust-miniscript / md-codec `SortedMultiA` rendering fix.
- `tr-multi-a` restore pre-gate lift (the A3 path the user did not pick).
- Cycle C (canonicity, GUI repo).
