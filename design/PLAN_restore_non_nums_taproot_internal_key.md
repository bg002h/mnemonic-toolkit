# `restore --md1` non-NUMS ("real key at the trunk") taproot — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lift `restore --md1`'s blanket refusal of non-NUMS (`is_nums:false`, "real key at the trunk") taproot wallet-policy cards so they faithfully reconstruct to a watch-only descriptor + receive address, while a STRUCTURAL classify-time guard refuses-and-defers the funds-unsafe `@-in-both` shape.

**Architecture:** Carry a `TaprootInternalKey` in the `TaprootRestore` enum; read the trunk key off the wire (`Body::Tr.key_index`) instead of hard-coding NUMS; split routing — `multi_a`/`sortedmulti_a` single-leaf → Template path (already emits a real internal key via the `Cosigner(idx)` arm), general leaf → GeneralFaithful route-around (md-codec's `is_nums:false → lookup_key` already renders it). The `@-in-both` guard (trunk `key_index ∈ leaf indices`) is a classify-time structural precondition because the Display-fidelity guard provably cannot catch the Template path's "leaf = all-others" wrong-leaf. `--format bip388` is refused only in the general route-around arm (the Template arm emits it faithfully).

**Tech Stack:** Rust, the toolkit's split lib+bin crate (`cmd::restore` is bin-only), md-codec 0.35.3 (crates.io), miniscript (patched git rev for taproot). Tests are `assert_cmd` integration tests in `crates/mnemonic-toolkit/tests/`.

**Source SHA at plan write:** `29613f3` (== branch base; re-grep on rebase). **Spec:** `design/SPEC_restore_non_nums_taproot_internal_key.md` (R0 GREEN, 3 rounds). **Branch:** `restore-non-nums-tr-internal-key`. **SemVer:** PATCH (→ v0.55.3). Watch-only, zero clap delta → no GUI `schema_mirror`, no paired-PR.

**Key source facts (grep-verified at `29613f3`):**
- `cmd/restore.rs:661-668` — `enum TaprootRestore { Template(CliTemplate), GeneralFaithful }` (no internal-key field yet).
- `cmd/restore.rs:692-733` — `classify_taproot_restore`; `:700-706` the `is_nums:false → ModeViolation` blanket gate to lift; `:718-720` `MultiA`/`SortedMultiA` → Template; `:721-731` general (sortedmulti_a-subtree + depth gates, then `GeneralFaithful`).
- `cmd/restore.rs:1206-1209` — call site, both arms hard-code `Some(TaprootInternalKey::Nums)`.
- `cmd/restore.rs:1287` — Display-fidelity guard (`parsed.to_string() != descriptor`).
- `cmd/restore.rs:800-845` — `build_multisig_import_payload`; `:827-845` the `match template { Some(t) => …, None => … }`; `:836-842` the existing `green` `P2tr` refusal inside the `None` branch.
- `cmd/restore.rs:1155` — `if !d.is_wallet_policy()` step-2 gate (template-only `ModeViolation` at `:1159`).
- `cmd/restore.rs:796-798` — comment "`taproot_internal_key` is `Some(Nums)` … (R0 v2 I2.)" (m1 target).
- `wallet_export/mod.rs:87` — `enum TaprootInternalKey { Nums, Cosigner(u32) }` (already exists). `:164-173` `WalletScriptType` (`P2tr`, `P2trMulti`). `:210-247` `script_type_from_descriptor`.
- `wallet_export/bip388.rs:97-130` — `TrMultiA|TrSortedMultiA` arm; `:109-114` `Nums`; `:115-128` `Cosigner(idx)` emits `tr(@{idx}/**,{leaf_op}(k,{leaf}))`.
- `wallet_export/pipeline.rs:113-156` — `build_tr_multi_a_descriptor`; `Cosigner(idx)` arm hard-codes `leaf = key_segs \ {idx}`.
- md-codec `tree.rs`: `Node { tag: Tag, body: Body }`; `Body::Tr { is_nums: bool, key_index: u8, tree: Option<Box<Node>> }`; `Body::MultiKeys { k: u8, indices: Vec<u8> }`. `encode.rs:50-52` `is_wallet_policy()` = `matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty())`.
- Tests `tests/cli_restore_taproot.rs`: `bundle_md1(desc)` helper (`:43`), `restore_args` (`:74`), consts `K0/K1/K2` (`:35-37`), `NUMS_HEX` (`:40`); existing refusal test `cosigner_internal_key_tr_bundles_but_restore_refuses_non_nums` (`:171-182`, `tr(K2,multi_a(2,K0,K1))` → exit 2); existing `general_tr_format_bip388_refused` (`:290-302`, NUMS, `code(1)` + `/<0;1>/*`); `general_tr_format_descriptor_and_bitcoin_core_emit` (`:306+`).
- `error.rs:502` `BadInput → 1`; `ModeViolation → 2`.

---

## File structure

- **Modify** `crates/mnemonic-toolkit/src/cmd/restore.rs` — enum + classify routing + @-in-both guard + call-site threading + None-branch bip388 refusal + comment hygiene.
- **Modify** `crates/mnemonic-toolkit/tests/cli_restore_taproot.rs` — success goldens (general + distinct-trunk multisig), invert the existing refusal test, @-in-both refusal (direct md_codec construction), `--format` matrix; update the NUMS `general_tr_format_bip388_refused` message assertion.
- **Modify** `docs/manual/src/40-cli-reference/41-mnemonic.md` — non-NUMS-now-supported prose (3 sites).
- **Modify** `design/FOLLOWUPS.md` — resolve this slug, file the `@-in-both` defer.
- **Release (final phase, post whole-branch review):** `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, `README.md`, `crates/mnemonic-toolkit/README.md`, `scripts/install.sh`.

**Per-phase discipline (CLAUDE.md):** each Task is a phase. Per-phase TDD (test before impl). After each Task's code lands, the per-phase opus architect review runs (persist verbatim to `design/agent-reports/restore-non-nums-taproot-phase-N-rM-review.md`, fold to 0C/0I) BEFORE moving to the next phase. Stage paths explicitly (no `git add -A`). Commit trailer: `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.

---

## Task 1: Carry `TaprootInternalKey` in `TaprootRestore`, lift the gate, split routing, thread the call site (success path)

This Task makes the non-NUMS general-tr and distinct-trunk multisig reconstruct. It deliberately does NOT add the `@-in-both` guard yet — Task 2 RED-proves that guard's necessity against this baseline.

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/restore.rs:661-668` (enum), `:692-733` (classify), `:1206-1209` (call site)
- Test: `crates/mnemonic-toolkit/tests/cli_restore_taproot.rs`

- [ ] **Step 1: Write the failing success tests** (append to `cli_restore_taproot.rs`, after the existing faithful-reconstruction tests). Use placeholder goldens — they'll be captured in Step 4.

```rust
// ─── Non-NUMS ("real key at the trunk") faithful reconstruction (v0.55.3) ────

/// (N1) Non-NUMS GENERAL single-leaf tr(D, and_v(v:pk(B),older(N))): the trunk
/// is a real cosigner key (live key-path spend). bundle emits a faithful
/// is_nums:false card; restore reconstructs the descriptor (real trunk key) +
/// a receive address. Golden captured-once from the binary (v0.49.1 precedent).
#[test]
fn non_nums_general_tr_leaf_restores_faithfully() {
    // K2 distinct from the leaf key K0 → not @-in-both.
    let desc = format!("tr({K2},and_v(v:pk({K0}),older(144)))");
    let (md1, emitted) = bundle_md1(&desc);
    assert!(!md1.is_empty(), "non-NUMS general-tr card must be emitted");
    assert_eq!(emitted, desc, "non-NUMS general-tr must round-trip on the wire");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .success()
        .stdout(
            predicate::str::contains(GOLDEN_DESC_NON_NUMS_GENERAL)
                .and(predicate::str::contains(GOLDEN_ADDR_NON_NUMS_GENERAL)),
        );
}

/// (N2) Non-NUMS DISTINCT-trunk multisig tr(D, multi_a(2,B,C)): trunk D NOT a
/// leaf key. Template path + Cosigner(idx). Golden captured-once.
#[test]
fn non_nums_distinct_trunk_multi_a_restores_faithfully() {
    let desc = format!("tr({K2},multi_a(2,{K0},{K1}))");
    let (md1, emitted) = bundle_md1(&desc);
    assert!(!md1.is_empty(), "non-NUMS multisig card must be emitted");
    assert_eq!(emitted, desc, "non-NUMS multisig must round-trip on the wire");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .success()
        .stdout(
            predicate::str::contains(GOLDEN_DESC_NON_NUMS_MULTI_A)
                .and(predicate::str::contains(GOLDEN_ADDR_NON_NUMS_MULTI_A)),
        );
}

/// (N3) Non-NUMS DISTINCT-trunk sortedmulti_a tr(D, sortedmulti_a(2,B,C)):
/// Template path (TrSortedMultiA) routes AROUND md-codec's SortedMultiA gap.
#[test]
fn non_nums_distinct_trunk_sortedmulti_a_restores_faithfully() {
    let desc = format!("tr({K2},sortedmulti_a(2,{K0},{K1}))");
    let (md1, emitted) = bundle_md1(&desc);
    assert!(!md1.is_empty(), "non-NUMS sortedmulti_a card must be emitted");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&md1))
        .assert()
        .success()
        .stdout(
            predicate::str::contains(GOLDEN_DESC_NON_NUMS_SORTEDMULTI_A)
                .and(predicate::str::contains(GOLDEN_ADDR_NON_NUMS_SORTEDMULTI_A)),
        );
}
```

Add placeholder golden consts near the other `GOLDEN_*` consts (top of file):

```rust
// Captured-once from the binary in Step 4 — DO NOT hand-construct (md-codec
// depth-0 xpub661My… reconstructions, not the bundle-input account xpubs).
const GOLDEN_DESC_NON_NUMS_GENERAL: &str = "PLACEHOLDER_DESC_GENERAL";
const GOLDEN_ADDR_NON_NUMS_GENERAL: &str = "PLACEHOLDER_ADDR_GENERAL";
const GOLDEN_DESC_NON_NUMS_MULTI_A: &str = "PLACEHOLDER_DESC_MULTI_A";
const GOLDEN_ADDR_NON_NUMS_MULTI_A: &str = "PLACEHOLDER_ADDR_MULTI_A";
const GOLDEN_DESC_NON_NUMS_SORTEDMULTI_A: &str = "PLACEHOLDER_DESC_SORTEDMULTI_A";
const GOLDEN_ADDR_NON_NUMS_SORTEDMULTI_A: &str = "PLACEHOLDER_ADDR_SORTEDMULTI_A";
```

- [ ] **Step 2: Run the new tests — verify they FAIL with the current refusal**

Run: `cargo test --manifest-path crates/mnemonic-toolkit/Cargo.toml --test cli_restore_taproot non_nums_ -- --nocapture`
Expected: FAIL — current code refuses with exit 2 "non-NUMS (cosigner) internal key" (the `:700` blanket gate), so `.success()` assertion fails.

- [ ] **Step 3: Implement — enum, classify routing, call-site threading**

In `restore.rs`, change the enum (`:661-668`) to carry the internal key on both variants:

```rust
/// §3 outcome for a `Tag::Tr` wallet-policy md1: which reconstruction arm,
/// and the internal ("trunk") key to thread (NUMS or a real cosigner key).
enum TaprootRestore {
    /// Single-leaf `multi_a`/`sortedmulti_a` — the byte-identical template
    /// path (`build_descriptor_string`). NUMS or distinct-trunk Cosigner(idx).
    Template(CliTemplate, TaprootInternalKey),
    /// General single-leaf / depth-1 two-leaf `tr(<internal>,…)` policy — the
    /// faithful route-around (`faithful_multisig_descriptor`). NUMS or Cosigner.
    GeneralFaithful(TaprootInternalKey),
}
```

In `classify_taproot_restore` (`:692-733`): capture `key_index` from the `Body::Tr`, drop the blanket `is_nums:false` refusal, and map the internal key. Replace the `match &tree.body { … }` block so both `is_nums` cases bind `inner` + the internal key:

```rust
fn classify_taproot_restore(tree: &md_codec::tree::Node) -> Result<TaprootRestore, ToolkitError> {
    use md_codec::tree::Body;
    let (inner, internal_key) = match &tree.body {
        Body::Tr { is_nums: true, tree: Some(inner), .. } => {
            (inner, TaprootInternalKey::Nums)
        }
        Body::Tr { is_nums: false, key_index, tree: Some(inner) } => {
            // Read the real trunk key off the wire — no inference. (key_index
            // is a 0..n placeholder index into the cosigner table; u8, and
            // TaprootInternalKey::Cosigner is also u8 — no cast.)
            (inner, TaprootInternalKey::Cosigner(*key_index))
        }
        Body::Tr { tree: None, .. } => {
            return Err(bad(
                "--md1 taproot tree has no script leaf (keypath-only tr is single-sig, not multisig)",
            ));
        }
        _ => {
            return Err(bad("--md1: internal error — taproot handler on a non-Tr tree"))
        }
    };
    match inner.tag {
        md_codec::Tag::MultiA => {
            Ok(TaprootRestore::Template(CliTemplate::TrMultiA, internal_key))
        }
        md_codec::Tag::SortedMultiA => {
            Ok(TaprootRestore::Template(CliTemplate::TrSortedMultiA, internal_key))
        }
        _ => {
            if subtree_contains_sortedmulti_a(inner) {
                return Err(ToolkitError::ModeViolation {
                    mode: "restore",
                    flag: "--md1",
                    message: "taproot md1 carries sortedmulti_a under a tap-script tree — md-codec cannot yet render it back as a non-root tap leaf (FOLLOWUP md-codec-sortedmulti-a-to-miniscript-rendering-gap); the engraved card remains a faithful backup",
                });
            }
            ensure_taptree_depth_le_one(inner)?;
            Ok(TaprootRestore::GeneralFaithful(internal_key))
        }
    }
}
```

> NOTE: the `@-in-both` guard is added in Task 2 (in the `MultiA`/`SortedMultiA` arms). Leaving it out here is the intentional RED baseline.

Update the call site (`:1206-1209`) to thread the carried key:

```rust
            match classify_taproot_restore(&d.tree)? {
                TaprootRestore::Template(t, ik) => (Some(t), Some(ik)),
                TaprootRestore::GeneralFaithful(ik) => (None, Some(ik)),
            }
```

Also update the classify doc-comment (`:676-679`) — see Task 4 (folded there to keep this Task code-only; or update inline now and skip the Task-4 duplicate).

- [ ] **Step 4: Capture the goldens, replace placeholders**

Run each restore and copy the exact `descriptor:` + `first recv:` lines into the consts. Example for N1:

Run: `cargo run --manifest-path crates/mnemonic-toolkit/Cargo.toml --bin mnemonic -- bundle --descriptor "tr([28645006/87'/0'/0']xpub6DB7...K2.../<0;1>/*,and_v(v:pk(...K0...),older(144)))" --network mainnet --json` → extract `md1` chunks → feed to `restore`. Simpler: temporarily `--nocapture` print in the test, or use this one-liner harness pattern from the existing suite (bundle → restore). Capture stdout's `descriptor:` and `first recv:` lines verbatim into `GOLDEN_DESC_NON_NUMS_GENERAL` / `GOLDEN_ADDR_NON_NUMS_GENERAL`, and likewise N2/N3.

**Sanity-eyeball before pinning:** the descriptor MUST start `tr(<K2's depth-0 xpub661My…>,…)` — a REAL x-only-free xpub trunk (NOT `50929b74…` NUMS). If the trunk renders as NUMS hex, the call-site threading is wrong — STOP and fix.

- [ ] **Step 5: Run the new tests — verify they PASS**

Run: `cargo test --manifest-path crates/mnemonic-toolkit/Cargo.toml --test cli_restore_taproot non_nums_ -- --nocapture`
Expected: PASS (3 tests).

- [ ] **Step 6: Invert the existing refusal test**

The existing `cosigner_internal_key_tr_bundles_but_restore_refuses_non_nums` (`:171-182`) tests `tr(K2,multi_a(2,K0,K1))` → exit 2. That shape is now SUPPORTED. It is now REDUNDANT with N2 — **delete it** (N2 covers the same `tr(K2,multi_a(2,K0,K1))` shape as a success). Remove the `#[test] fn cosigner_internal_key_tr_bundles_but_restore_refuses_non_nums` block and its doc comment (`:166-182`).

- [ ] **Step 7: Run the full taproot-restore suite + regression check**

Run: `cargo test --manifest-path crates/mnemonic-toolkit/Cargo.toml --test cli_restore_taproot`
Expected: PASS (the NUMS goldens N from v0.49.1/v0.55.1 stay byte-identical; the deleted refusal test is gone; N1-N3 pass).

- [ ] **Step 8: Commit**

```bash
git -C /scratch/code/shibboleth/mnemonic-toolkit add crates/mnemonic-toolkit/src/cmd/restore.rs crates/mnemonic-toolkit/tests/cli_restore_taproot.rs
git -C /scratch/code/shibboleth/mnemonic-toolkit commit -m "feat(restore): reconstruct non-NUMS (real-trunk) taproot — general + distinct-trunk multisig

Thread TaprootInternalKey through TaprootRestore; read the trunk key off the
wire (Body::Tr.key_index) instead of hard-coding NUMS; split routing (Template
for multi_a/sortedmulti_a single-leaf, GeneralFaithful route-around otherwise).
@-in-both guard follows in the next commit (this is its RED baseline).

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: The `@-in-both` structural guard (funds-safety crux)

The Template path's `Cosigner(idx)` mode reconstructs the leaf as `{all cosigners EXCEPT idx}`. For `tr(@i, multi_a(k, …@i…))` (trunk index also a leaf index) that emits a DIFFERENT multisig — a silently-wrong wallet the Display-fidelity guard cannot catch. Refuse it at classify time.

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/restore.rs:718-720` (the Template arms in `classify_taproot_restore`)
- Test: `crates/mnemonic-toolkit/tests/cli_restore_taproot.rs`

- [ ] **Step 1: Write the @-in-both refusal test (direct md_codec construction)**

`bundle --descriptor` REJECTS `@-in-both` at intake (BIP-388 distinct-key gate), so build the md1 directly. Add to `cli_restore_taproot.rs`. Confirm md_codec's public API names (`md_codec::tree::{Node, Body}`, `md_codec::Tag`, `md_codec::chunk::split`, the `Descriptor`/`tlv` constructors) by reading `~/.cargo/registry/src/*/md-codec-0.35.3/src/{lib.rs,tree.rs,chunk.rs,encode.rs}` before writing — adjust the struct literal to the actual fields. Skeleton:

```rust
use md_codec::tree::{Body, Node};

/// (N4) @-in-both refusal: tr(@0, multi_a(2, @0, @1)) — the trunk key IS also a
/// leaf key. The Template Cosigner(idx) shortcut would emit multi_a(2,@1) (drop
/// @0) — a different, silently-wrong wallet the Display-fidelity guard CANNOT
/// catch (it self-prints). MUST refuse structurally at classify time.
/// bundle rejects this shape at intake, so the md1 is constructed directly.
#[test]
fn at_in_both_tr_refuses_structurally() {
    // Build a Descriptor whose tree is Tr{is_nums:false,key_index:0,
    // tree:Some(MultiA leaf with indices [0,1])} and populate tlv.pubkeys with
    // a non-empty concrete pubkey per slot so is_wallet_policy() passes the
    // step-2 gate (else it trips the wrong "template-only" refusal). See
    // md-codec encode.rs:50 (is_wallet_policy) + restore.rs:1155.
    let d = build_at_in_both_descriptor(); // helper below
    let chunks = md_codec::chunk::split(&d).expect("split @-in-both md1");
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(restore_args(&chunks))
        .assert()
        .code(2)
        .stderr(
            predicate::str::contains("restore-non-nums-tr-internal-key-also-in-leaf")
                .and(predicate::str::contains("also a leaf key")),
        );
}
```

Write `build_at_in_both_descriptor()` using md_codec's actual public constructors (read the crate first). **`md_codec::Descriptor` has FIVE public fields** (`encode.rs:17-28`) — ALL must be set or it won't compile: `n: u8`, `path_decl: PathDecl`, `use_site_path: UseSitePath`, `tree: Node`, `tlv: TlvSection`. Set:
- `n: 2` (two placeholders).
- `tree = Node { tag: Tag::Tr, body: Body::Tr { is_nums: false, key_index: 0, tree: Some(Box::new(Node { tag: Tag::MultiA, body: Body::MultiKeys { k: 2, indices: vec![0, 1] } })) } }` — the `@-in-both` shape (trunk `@0` ∈ leaf `[0,1]`).
- `tlv.pubkeys = Some(vec![<two valid 65-byte [chaincode‖pubkey] entries>])` (non-empty so `is_wallet_policy()` passes — see Step-1 note).
- `path_decl` + `use_site_path`: read `tree.rs`/`encode.rs`/the existing direct-construction tests for the canonical standard-multipath values (e.g. a shared origin + `<0;1>/*` use-site); reuse whatever the suite already constructs for a wallet-policy md1.

The `@-in-both` payload DOES encode cleanly: `validate_placeholder_usage` registers `@0` (Tr body) first, then the leaf's `[0,1]` (skips the seen `0`, adds `1`) → `first_occurrences=[0,1]`, canonical, passes. (R0-confirmed.) Derive the two 65-byte pubkey entries from K0/K1 via md_codec's expand, or reuse the codec's own fixtures.

- [ ] **Step 2: Run — verify the RED-proof (currently reconstructs WRONG, no refusal)**

Run: `cargo test --manifest-path crates/mnemonic-toolkit/Cargo.toml --test cli_restore_taproot at_in_both_ -- --nocapture`
Expected: FAIL — without the guard, Task 1's code returns `Template(TrMultiA, Cosigner(0))`, reconstructs `tr(<@0 seg>, multi_a(2, <@1 seg>))` (a different, 1-cosigner-dropped multisig), and **exits 0** (success, wrong output). The test's `.code(2)` assertion therefore fails — that IS the RED-proof (the Display-fidelity guard self-prints the wrong-but-consistent descriptor, so only the structural classify-time guard can catch it). **CONFIRM the exit code is exactly 0** (a wrong-but-successful reconstruction), NOT some other exit-2 refusal — an exit-2 failure for an unrelated reason would falsely "pass" the RED. Capture the wrong stdout (`--nocapture`) and note it in the commit body.

- [ ] **Step 3: Implement the guard in the Template arms**

In `classify_taproot_restore`, replace the two Template arms (`:718-720` post-Task-1) with a guarded helper call:

```rust
        md_codec::Tag::MultiA => {
            refuse_at_in_both(&internal_key, inner)?;
            Ok(TaprootRestore::Template(CliTemplate::TrMultiA, internal_key))
        }
        md_codec::Tag::SortedMultiA => {
            refuse_at_in_both(&internal_key, inner)?;
            Ok(TaprootRestore::Template(CliTemplate::TrSortedMultiA, internal_key))
        }
```

Add the helper near `classify_taproot_restore`:

```rust
/// The Template path's `Cosigner(idx)` mode reconstructs the leaf as
/// `{all cosigners EXCEPT idx}` (`pipeline.rs:134-156`). When the non-NUMS
/// trunk index is ALSO a leaf index (`@-in-both`: `tr(@i, multi_a(k,…@i…))`),
/// that shortcut would emit a DIFFERENT multisig — and the Display-fidelity
/// guard (`restore.rs:1287`) cannot catch it (the Template output is its own
/// re-print). So refuse STRUCTURALLY here, never via Display. NUMS trunks
/// (`is_nums:true`) are not in a cosigner slot → never trip this.
fn refuse_at_in_both(
    internal_key: &TaprootInternalKey,
    leaf: &md_codec::tree::Node,
) -> Result<(), ToolkitError> {
    use md_codec::tree::Body;
    // Cosigner(u8); indices: Vec<u8> — all u8, no casts.
    if let TaprootInternalKey::Cosigner(i) = internal_key {
        if let Body::MultiKeys { indices, .. } = &leaf.body {
            if indices.iter().any(|&idx| idx == *i) {
                return Err(ToolkitError::ModeViolation {
                    mode: "restore",
                    flag: "--md1",
                    message: "taproot md1 has a non-NUMS internal (trunk) key that is ALSO a leaf key (@-in-both) — the engraved card is a faithful backup, but reconstructing it needs a leaf-membership-aware rebuild not yet supported; refusing rather than emit a silently-different multisig (FOLLOWUP restore-non-nums-tr-internal-key-also-in-leaf)",
                });
            }
        }
    }
    Ok(())
}
```

- [ ] **Step 4: Run — verify the guard refuses (GREEN)**

Run: `cargo test --manifest-path crates/mnemonic-toolkit/Cargo.toml --test cli_restore_taproot at_in_both_ -- --nocapture`
Expected: PASS — exit 2 + the slug + "also a leaf key".

- [ ] **Step 5: Run the full taproot-restore suite (N1-N4 + NUMS goldens)**

Run: `cargo test --manifest-path crates/mnemonic-toolkit/Cargo.toml --test cli_restore_taproot`
Expected: PASS (N1-N3 distinct-trunk still pass — they're not @-in-both; N4 refuses; NUMS goldens unchanged).

- [ ] **Step 6: Commit**

```bash
git -C /scratch/code/shibboleth/mnemonic-toolkit add crates/mnemonic-toolkit/src/cmd/restore.rs crates/mnemonic-toolkit/tests/cli_restore_taproot.rs
git -C /scratch/code/shibboleth/mnemonic-toolkit commit -m "feat(restore): structural @-in-both guard for non-NUMS taproot (funds-safety)

Refuse tr(@i, multi_a(k,…@i…)) at classify time — the Template Cosigner(idx)
'leaf = all-others' shortcut would emit a silently-different multisig the
Display-fidelity guard cannot catch (it self-prints). RED-proof: without the
guard the crafted @-in-both md1 reconstructs multi_a dropping the trunk key
and exits 0. Test builds the md1 directly via md_codec (bundle rejects the
shape at intake). Deferred: restore-non-nums-tr-internal-key-also-in-leaf.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: `--format` matrix for non-NUMS taproot (bip388 refusal in the route-around arm only)

The Template path emits bip388 faithfully (`bip388.rs:115-127` `Cosigner(idx)`). The general route-around arm cannot express a tap-script-tree as a BIP-388 wallet policy → refuse. The NUMS general-tr currently refuses bip388 *incidentally* (no-multipath); unify it explicitly.

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/restore.rs:832-844` (the `None` branch of `build_multisig_import_payload`)
- Test: `crates/mnemonic-toolkit/tests/cli_restore_taproot.rs`

- [ ] **Step 1: Write the format tests**

```rust
/// (N5) Non-NUMS general-tr --format bip388 → refused (route-around arm cannot
/// express a tap-script tree as a BIP-388 wallet policy). exit 1.
#[test]
fn non_nums_general_tr_format_bip388_refused() {
    let desc = format!("tr({K2},and_v(v:pk({K0}),older(144)))");
    let (md1, _e) = bundle_md1(&desc);
    let mut a = restore_args(&md1);
    a.push("--format".into());
    a.push("bip388".into());
    Command::cargo_bin("mnemonic").unwrap().args(&a).assert()
        .code(1)
        .stderr(predicate::str::contains("BIP-388 wallet policy"));
}

/// (N6) Non-NUMS DISTINCT-trunk multisig --format bip388 → SUCCEEDS (Template
/// path + bip388.rs Cosigner(idx) arm emits tr(@idx/**,multi_a(k,…))).
#[test]
fn non_nums_distinct_trunk_multi_a_format_bip388_succeeds() {
    let desc = format!("tr({K2},multi_a(2,{K0},{K1}))");
    let (md1, _e) = bundle_md1(&desc);
    let mut a = restore_args(&md1);
    a.push("--format".into());
    a.push("bip388".into());
    let out = Command::cargo_bin("mnemonic").unwrap().args(&a).assert()
        .success().get_output().stdout.clone();
    let s = String::from_utf8(out).unwrap();
    // The trunk cosigner is at @-some-index; the emitted policy is a tr(@i/**,…).
    assert!(s.contains("tr(@") && s.contains("multi_a("),
        "bip388 wallet policy must carry tr(@idx/**,multi_a(…)): {s}");
}

/// (N7) Non-NUMS --format descriptor / bitcoin-core → emit faithfully (both arms).
#[test]
fn non_nums_format_descriptor_and_bitcoin_core_emit() {
    for desc in [
        format!("tr({K2},and_v(v:pk({K0}),older(144)))"),
        format!("tr({K2},multi_a(2,{K0},{K1}))"),
    ] {
        let (md1, _e) = bundle_md1(&desc);
        for fmt in ["descriptor", "bitcoin-core"] {
            let mut a = restore_args(&md1);
            a.push("--format".into());
            a.push(fmt.into());
            Command::cargo_bin("mnemonic").unwrap().args(&a).assert().success();
        }
    }
}

/// (N8) Non-NUMS general-tr --format green → refused (existing P2tr green gate).
#[test]
fn non_nums_general_tr_format_green_refused() {
    let desc = format!("tr({K2},and_v(v:pk({K0}),older(144)))");
    let (md1, _e) = bundle_md1(&desc);
    let mut a = restore_args(&md1);
    a.push("--format".into());
    a.push("green".into());
    Command::cargo_bin("mnemonic").unwrap().args(&a).assert().failure();
}
```

Update the existing NUMS test `general_tr_format_bip388_refused` (`:290-302`): the unified explicit guard changes the message (exit 1 stays). Change the assertion from `predicate::str::contains("/<0;1>/*")` to `predicate::str::contains("BIP-388 wallet policy")` and refresh its doc comment to say the refusal is now the explicit taproot route-around guard (no longer the incidental no-multipath failure).

- [ ] **Step 2: Run — N5/N8 may already pass (green), N6 FAILS (bip388 wrongly refused today via no-multipath? no — multipath trunk emits), the NUMS test FAILS on the new message**

Run: `cargo test --manifest-path crates/mnemonic-toolkit/Cargo.toml --test cli_restore_taproot _format_ -- --nocapture`
Expected: **N6 already PASSES** (after Task 1 the distinct-trunk multisig takes the `Some(t)` Template branch → never reaches the `None` branch → bip388 emits faithfully; Task 3's `None`-branch guard can't touch it). **N5 is the RED**: it currently EMITS a (wrong) bip388 payload from the route-around arm — the hole — so `.code(1)` fails. The edited NUMS `general_tr_format_bip388_refused` also FAILS on the new message string (it still exits 1 but the message changed). N8 (green) likely already passes (existing `P2tr` green gate). Confirm N5's failure output shows the silent-emit hole.

- [ ] **Step 3: Implement the explicit bip388 refusal in the `None` branch**

In `build_multisig_import_payload`, inside the `None` branch (`:832-844`), after `let script_type = …;` and alongside the existing green `P2tr` refusal, add:

```rust
            if format == CliExportFormat::Bip388
                && matches!(
                    script_type,
                    wallet_export::WalletScriptType::P2tr
                        | wallet_export::WalletScriptType::P2trMulti
                )
            {
                return Err(ToolkitError::BadInput(
                    "--format bip388 cannot express this taproot policy as a BIP-388 wallet policy — a tap-script-tree reconstructed via the general route-around has no named-template form. Use --format descriptor or --format bitcoin-core for a watch-only import. (A distinct-trunk tr-multisig md1 DOES export bip388 via its template path.)".into(),
                ));
            }
```

(Confirm the `CliExportFormat::Bip388` variant name via `grep -n 'enum CliExportFormat' -A 20` in the cmd module; adjust if it's `Bip388`/`BIP388`/etc.)

- [ ] **Step 4: Run — verify GREEN**

Run: `cargo test --manifest-path crates/mnemonic-toolkit/Cargo.toml --test cli_restore_taproot _format_ general_tr_format_bip388_refused -- --nocapture`
Expected: PASS — N5 refused (exit 1, new msg), N6 succeeds (Template path), N7 emits, N8 refused, NUMS test passes the new message assertion.

- [ ] **Step 5: Run the full taproot-restore suite**

Run: `cargo test --manifest-path crates/mnemonic-toolkit/Cargo.toml --test cli_restore_taproot`
Expected: ALL PASS.

- [ ] **Step 6: Commit**

```bash
git -C /scratch/code/shibboleth/mnemonic-toolkit add crates/mnemonic-toolkit/src/cmd/restore.rs crates/mnemonic-toolkit/tests/cli_restore_taproot.rs
git -C /scratch/code/shibboleth/mnemonic-toolkit commit -m "feat(restore): explicit bip388 refusal for route-around taproot; multisig bip388 still emits

The general route-around (template==None) arm cannot express a tap-script tree
as a BIP-388 wallet policy → refuse (BadInput, exit 1), unifying the previously
incidental NUMS general-tr refusal AND closing the non-NUMS multipath-trunk
hole. The Template path (Some(t)) is untouched — non-NUMS distinct-trunk
multisig still emits tr(@idx/**,multi_a(k,…)) faithfully via bip388.rs Cosigner.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Docs — comment hygiene, manual prose, FOLLOWUPS

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/restore.rs:676-679, :796-798`
- Modify: `docs/manual/src/40-cli-reference/41-mnemonic.md` (re-grep `:771/:794/:1027`)
- Modify: `design/FOLLOWUPS.md`

- [ ] **Step 1: Update the classify doc-comment** (`restore.rs:676-679`) — replace "Supports only `is_nums:true` … `is_nums:false` … is deferred" with: "Supports `is_nums:true` (NUMS) AND `is_nums:false` (real cosigner trunk key), the latter for general single-leaf/depth-1 (route-around) and distinct-trunk multisig (Template); the `@-in-both` shape (trunk key also a leaf key) refuses (`restore-non-nums-tr-internal-key-also-in-leaf`)."

- [ ] **Step 2: Update the `build_multisig_import_payload` comment** (`restore.rs:796-798`) — "`taproot_internal_key` is `Some(Nums)` or `Some(Cosigner(idx))` for a taproot md1 (threaded from §3), `None` for wsh/sh-wsh." Replace the trailing `(R0 v2 I2.)` provenance tail (it cites a PRIOR cycle).

- [ ] **Step 3: Re-grep + update the manual** (3 sites). Run: `grep -n 'non-NUMS\|NUMS' docs/manual/src/40-cli-reference/41-mnemonic.md`. Update the refusal prose at the 3 sites (was `:771/:794/:1027`) to: non-NUMS key-path taproot (general single-leaf/depth-1 + distinct-trunk multisig) now reconstructs; `@-in-both` + depth-≥2 remain refused; non-NUMS general-tr emits `descriptor`/`bitcoin-core` only (`bip388`/`green` refused), distinct-trunk multisig also emits `bip388`.

- [ ] **Step 4: Run the FULL manual lint** (the v0.50.0 lesson — run the whole lint, not just flag-coverage):

Run: `make -C docs/manual lint MNEMONIC_BIN=$(pwd)/target/debug/mnemonic MD_BIN=... MS_BIN=... MK_BIN=...` (build the binaries first; or follow `.github/workflows/manual.yml`'s invocation). Expected: PASS (this is docs-only prose; no flag delta, but cspell/markdownlint must pass).

- [ ] **Step 5: Update FOLLOWUPS.md** — first read the file to learn its current section structure (open vs resolved archive, restore/taproot sub-cluster) and place entries accordingly. (a) file + mark RESOLVED `restore-non-nums-taproot-internal-key` (this cycle; cite the shipped commit + tag); (b) file (open) `restore-non-nums-tr-internal-key-also-in-leaf` (the `@-in-both` defer — leaf-membership-aware route-around-for-multi_a is the eventual mechanism, adjacent to the md-codec SortedMultiA gap). Match the existing file's section/ordering conventions; cross-cite the spec/plan paths.

- [ ] **Step 6: Commit**

```bash
git -C /scratch/code/shibboleth/mnemonic-toolkit add crates/mnemonic-toolkit/src/cmd/restore.rs docs/manual/src/40-cli-reference/41-mnemonic.md design/FOLLOWUPS.md
git -C /scratch/code/shibboleth/mnemonic-toolkit commit -m "docs(restore): non-NUMS taproot now supported — comment hygiene, manual, FOLLOWUPS

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Whole-branch review gate → release ritual (PATCH v0.55.3)

> Do NOT start until Tasks 1-4 have each passed their per-phase architect review (0C/0I, persisted) AND a final whole-branch review is GREEN. The release ritual is irreversible (public tag) — confirm with the user before tagging.

- [ ] **Step 1: Final whole-branch review.** Dispatch an architect over the full diff (`git diff master...HEAD`). Persist verbatim, fold to 0C/0I.

- [ ] **Step 2: Full suite + fuzz build (the v0.55.2 lesson).**
Run: `cargo test --manifest-path crates/mnemonic-toolkit/Cargo.toml` (whole crate, not just one test file), `cargo clippy --manifest-path crates/mnemonic-toolkit/Cargo.toml --all-targets`, AND build the fuzz workspace `cargo +nightly build --manifest-path crates/mnemonic-toolkit/fuzz/Cargo.toml` (or the repo's fuzz check) — the v0.55.2 fuzz-smoke red came from skipping this. Expected: all green.

- [ ] **Step 3: Version bump (PATCH → 0.55.3).** `Cargo.toml` `version = "0.55.3"`; refresh `Cargo.lock` (`cargo update -p mnemonic-toolkit --precise 0.55.3` or a build). README markers (BOTH `README.md:13` and `crates/mnemonic-toolkit/README.md:9` `<!-- toolkit-version: 0.55.3 -->`). `scripts/install.sh` toolkit pin → `mnemonic-toolkit-v0.55.3`. CHANGELOG `[0.55.3]` entry. (These are the 4 post-tag-CI gates from the v0.55.2 incident: `both_readmes_carry_current_version_marker`, install-pin-check, changelog-check, readme markers.)

- [ ] **Step 4: Re-run the FULL suite AFTER the version bump** (the v0.55.2 lesson: re-run after the bump, not before). Expected: green incl. `readme_version_current`.

- [ ] **Step 5: Commit the release + tag (after user confirmation).**

```bash
git -C /scratch/code/shibboleth/mnemonic-toolkit add Cargo.toml Cargo.lock CHANGELOG.md README.md crates/mnemonic-toolkit/README.md scripts/install.sh
git -C /scratch/code/shibboleth/mnemonic-toolkit commit -m "release: mnemonic-toolkit 0.55.3 — restore reconstructs non-NUMS (real-trunk) taproot

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
# then: merge to master per finishing-a-development-branch, push, tag mnemonic-toolkit-v0.55.3, watch CI.
```

---

## Self-review (writing-plans)

**Spec coverage:** §2 decision (a)+(b) → Task 1 (N1-N3); §4 @-in-both guard → Task 2 (N4); §6 format matrix → Task 3 (N5-N8 + NUMS test); §3 routing/enum/call-site → Task 1; §5 comment hygiene → Task 4; §7 tests → Tasks 1-3 (incl. the RED-proof in Task 2 Step 2, the inverting/now-delete in Task 1 Step 6, the existing-NUMS-message update in Task 3 Step 1); §8 SemVer/manual/FOLLOWUPS → Tasks 4-5. All covered.

**Placeholder scan:** golden consts are deliberately PLACEHOLDER until captured-from-binary (Task 1 Step 4) — the established v0.49.1 "derive-once" discipline, not a plan gap. `build_at_in_both_descriptor()` and exact md_codec constructor names are flagged "read the crate first" because the public API surface (e.g. how `tlv.pubkeys` is set, whether `Descriptor` has a public ctor vs builder) must be confirmed against md-codec 0.35.3 at implementation time — do NOT guess the struct literal.

**Type consistency (VERIFIED at `29613f3`):** `TaprootInternalKey::Cosigner(u8)` (`wallet_export/mod.rs:95`) and md-codec `Body::Tr.key_index: u8` + `Body::MultiKeys.indices: Vec<u8>` — all `u8`, no casts (plan corrected). `TaprootRestore::{Template(CliTemplate, TaprootInternalKey), GeneralFaithful(TaprootInternalKey)}` used consistently at the enum def, classify returns, and call site. `CliExportFormat::Bip388` (`export_wallet.rs:26`) and `::Green` (`:40`) variant names CONFIRMED. Still confirm-at-impl: md_codec's public `Descriptor`/`tlv` constructor surface for `build_at_in_both_descriptor()` (Task 2).
