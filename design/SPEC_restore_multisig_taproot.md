# SPEC v2 — `mnemonic restore --md1` taproot multisig reconstruction (tr-multi-a + tr-sortedmulti-a)

**Repo:** `mnemonic-toolkit`. **Resolves:** `restore-multisig-taproot-reconstruction` (+ unblocks the toolkit-side of `md-codec-sortedmulti-a-to-miniscript-rendering-gap` via route-around).
**Date:** 2026-06-09. **Source SHA:** `origin/master` == `5a22552`. **miniscript rev:** git `95fdd1c` (HAS `Terminal::SortedMultiA` — `decode.rs:161`).
**Disposition:** toolkit **PATCH** (widens what `restore --md1` accepts; no new flag/subcommand → **no GUI `schema_mirror` lockstep**). Watch-only-out (no xpriv).
**SUPERSEDES the RED v1** (`design/agent-reports/restore-multisig-taproot-r0-r1-review.md`, 2026-06-05, 2 Critical). **Both Criticals are dissolved by v0.48.0** (`toolkit-trmultia-nums-internal-key`) — see §0. Recon: `cycle-prep-recon-restore-multisig-taproot.md`.

---

## 0. Why v1 was RED and why v2 is not (the premise correction)

v1's R0-r1 found 2 Criticals, both rooted in the **pre-v0.48.0** bundle tr emit:
- **C1:** bundle emitted `Body::Tr { is_nums: false, key_index: 0 }` with leaf `indices:(0..n)` — internal key = cosigner @0, also in the leaf. v1 reconstructed `is_nums:true` and refused `is_nums:false` → fired on **no real card**.
- **C2:** that `tr(@0, sortedmulti_a(@0..@n-1))` shape (`@0` internal AND in leaf) is **unreproducible** by `build_descriptor_string` under any `TaprootInternalKey`.

**v0.48.0 (`toolkit-trmultia-nums-internal-key`, shipped `223b538`) flipped `template.rs:213` `is_nums: false → true`.** Bundle now emits `Body::Tr { is_nums: true }` → descriptor `tr(NUMS, multi_a/sortedmulti_a(@0..@n-1))`. Consequences:
- **C1 dissolved:** `is_nums:true` is now THE bundle shape; reconstructing it fires on every v0.48.0+ tr multisig md1.
- **C2 dissolved:** `build_descriptor_string(template, slots, k, …, Some(TaprootInternalKey::Nums))` reproduces `tr(NUMS, multi_a/sortedmulti_a(@0..@n-1))` **exactly** — empirically reconfirmed at `5a22552`: `export-wallet --template tr-sortedmulti-a --taproot-internal-key nums --format descriptor` → `tr(50929b74…803ac0,sortedmulti_a(2,…))#essg384w`; `tr-multi-a` → `tr(NUMS,multi_a(2,…))#…`. The reconstructed tree round-trips to the input md1 by construction (same `build_descriptor_string` bundle uses, same `is_nums:true` tree).

v2 therefore handles **`is_nums:true` only** and **defers** `is_nums:false` (refuses with a pointer — §2). The degenerate pre-v0.48.0 bundle `@0-in-both` shape is `is_nums:false` AND unreproducible; a *genuine* third-party cosigner-internal `is_nums:false` shape IS reconstructable but needs a leaf-membership analysis to tell the two apart (see §2 M1 note). Deferring the whole `is_nums:false` axis is the safe scope: v2 covers every v0.48.0+ bundle-emitted taproot multisig md1 (all `is_nums:true`) and emits a wrong wallet for none.

The slot/threshold/`is_wallet_policy`/`expand_per_at_n` reuse v1 verified clean carries forward unchanged (tree-shape-agnostic).

## 1. The fix — a taproot-multisig reconstruction branch in `run_multisig`

`crates/mnemonic-toolkit/src/cmd/restore.rs::run_multisig` currently refuses ALL taproot at `:777` (`if d.tree.tag == md_codec::Tag::Tr { ModeViolation }`). Replace that blanket refusal with a taproot handler that reconstructs the descriptor by reading the tree **directly** — bypassing `to_miniscript_descriptor` (which errors on `SortedMultiA`, `to_miniscript.rs:406-410`).

### Reconstruction (both leaf types, one path)
1. `d.tree.tag == Tag::Tr` → enter the taproot handler (instead of refusing).
2. Read `Body::Tr { is_nums, key_index, tree: Some(inner) }` from `d.tree.body` (the `Body::Tr` pattern `extract_multisig_threshold` already uses, `bundle.rs`).
   - **`is_nums == false` → REFUSE** with a clear pointer (§2). Do NOT attempt reconstruction.
   - `is_nums == true` → `internal_key = TaprootInternalKey::Nums`.
3. `template = match inner.tag { Tag::MultiA => CliTemplate::TrMultiA, Tag::SortedMultiA => CliTemplate::TrSortedMultiA, _ => refuse "taproot leaf is not a multisig (multi_a/sortedmulti_a)" }`.
4. `k = extract_multisig_threshold(&d.tree)` (recurses `Body::Tr`; already correct).
5. `slots: Vec<ResolvedSlot> = expand_per_at_n(&d)` → ResolvedSlots (same construction the wsh path uses; `canonicalize.rs`).
6. `descriptor = build_descriptor_string(template, &slots, k, network, args.account, Some(internal_key))` (`wallet_export/pipeline.rs:18`). This is the ONLY behavioural change vs the wsh path (which passes `None`).
7. **Derive the first receive address from the reconstructed descriptor STRING — NOT via `d.derive_address` (C1(new) fold).** The existing multisig emission derives `first_recv` via `d.derive_address(0, i, network)` (`restore.rs:840-842`), which **internally re-enters md-codec's `to_miniscript_descriptor`** (`md-codec derive.rs:120`) — the exact function that ERRORS on `SortedMultiA` (`to_miniscript.rs:406-410`). So `d.derive_address` would hard-fail at address derivation for a `tr-sortedmulti-a` md1 **after** the descriptor string was correctly reconstructed at step 6 — killing the headline capability (tr-multi-a would survive since `MultiA` renders, but sortedmulti_a would not). **Therefore the taproot branch MUST derive the first address from the reconstructed `descriptor` string via the toolkit's pinned miniscript** (rev `95fdd1c`, which HAS `SortedMultiA`): `MsDescriptor::<DescriptorPublicKey>::from_str(&descriptor)` → `into_single_descriptors()` → `derive_at_index(0)` → `.address(network)` — the exact three-call sequence `derive_address.rs::derive_first_address` (`:26`, body `:34-66`) already implements (it renders `bc1p` for tr multipath per the v1-review M1). `derive_first_address`'s header caveat "the caller must reject `tr(...)`" (`derive_address.rs:24-25`) is a BIP-129/BSMS-context note, NOT a technical limit — relax it deliberately (or inline the identical sequence in the taproot branch). The rest of the emission (master-fingerprint banner, `--format`/`--json`/`--output`) is unchanged.

The wsh/sh-wsh path (steps using `to_miniscript_descriptor → template_from_descriptor → d.derive_address`) is untouched; only `Tag::Tr` routes into the new branch (which builds the descriptor via `build_descriptor_string` AND derives the address via the miniscript descriptor-string path — both routing around md-codec). NOTE: there is no single-sig analogy here — single-sig restore renders addresses via `render_address_from_xpub` (xpub-based, `restore.rs:362` path), so bip86's success says nothing about the multisig md1 address path; that false analogy is what masked this gap in v1-of-this-SPEC.

### `--format` multisig payloads (`restore --md1 --format <X>`)
The v0.45.0 `restore --md1 --format` path (`build_multisig_import_payload`, `:636` doc) hardcodes `taproot_internal_key: None` **internally at `restore.rs:662`** (NOT `:696` — that line is `xpub_from_65_bytes` at `5a22552`; the original cite was stale). Fix = **add a `taproot_internal_key: Option<TaprootInternalKey>` parameter to `build_multisig_import_payload`** (it currently hardcodes `None`) and pass the reconstructed `Some(internal_key)` from its call site (`restore.rs:1034`). (Leave the separate single-sig `build_import_payload` `None` at `:606`.) NOTE: the `--format` payload path emits via `emit_payload` and does **NOT** call `d.derive_address`, so `--format` is NOT hit by the C1(new) address-derivation hole — but it still needs the correct `internal_key` so the emitted taproot descriptor in the payload is correct. Covered by tests.

## 2. Scope

**IN (v2):** `restore --md1` reconstruction of a **wallet-policy** md1 whose tree is `Tr{ is_nums:true(NUMS), MultiA|SortedMultiA{k, indices:0..n} }` — i.e. every v0.48.0+ bundle-emitted tr-multi-a / tr-sortedmulti-a card. Both `--format`/`--json`/text output forms.

**REFUSED (clear pointer, not silent):**
- `is_nums:false` taproot md1 → `ModeViolation`: *"taproot multisig md1 with a non-NUMS (cosigner) internal key is not supported by `restore` yet — re-engrave from seed with mnemonic ≥ v0.48.0 to get a NUMS tr md1, or see FOLLOWUP restore-multisig-taproot-reconstruction."* **Honest framing (M1 fold):** `is_nums:false` is NOT a non-existent shape — a genuine third-party `tr(@cosigner, multi_a(k,…))` (cosigner internal key NOT in the leaf) *does* render (md-codec `to_miniscript.rs:161-165` `lookup_key` + `:394-398` MultiA) and *is* reconstructable via `build_descriptor_string`'s `Cosigner(idx)` arm (`pipeline.rs:135`). v2 **deliberately defers** it (it needs a leaf-membership analysis to distinguish the genuine cosigner-internal shape from the degenerate pre-v0.48.0 bundle `@0-in-both` shape that is NOT reproducible) — refusing is the safe v2 scope call, not a claim the shape doesn't exist. Filed as out-of-scope.
- A `Tag::Tr` whose leaf is not `MultiA`/`SortedMultiA` (e.g. a single-key tr or a non-multisig taptree) → refuse "taproot leaf is not a recognized multisig."

**OUT (deferred, filed):** is_nums:false reconstruction (the cosigner-internal shape — needs a leaf-membership analysis + is non-functional from bundle); the md-codec-native `sortedmulti_a → sorted multi_a` lowering (a separate descriptor-mnemonic cycle, the `md-codec-sortedmulti-a-to-miniscript-rendering-gap` option (b) — NOT needed since v2 routes around md-codec); arbitrary multi-leaf taptrees.

## 3. Safety — round-trip integrity (dissolves the C2 spirit: never emit a wrong wallet)

The reconstruction is **exact by construction** for `is_nums:true` (same `build_descriptor_string` + same `is_nums:true` tree bundle emits). v2 PINS this with a **round-trip oracle test** (§5): a tr-multi-a / tr-sortedmulti-a bundle → its md1 → `restore --md1` → assert the reconstructed descriptor's `to_string()` equals the descriptor `export-wallet --template … --taproot-internal-key nums --format descriptor` emits for the same cosigners (the canonical NUMS shape), AND that re-encoding the reconstructed descriptor through the bundle/template path yields the **byte-identical input md1 chunks**. A mismatch fails the test (no silent wrong-wallet). This is non-tautological (the oracle is the independent export-wallet emit + the input md1), per `feedback_recapture_golden_only_when_current_correct`.

## 4. Citations (grep-verified at `5a22552` / miniscript `95fdd1c` / md-codec crates.io `0.35.0`)
- `restore.rs:777` `Tag::Tr` blanket refusal (the replace site); `run_multisig` at `:742`; the descriptor build at `:835` (`None`→`Some(internal_key)`); **the `:840-842` `d.derive_address` that MUST be replaced for taproot (C1(new))**; `--format` `taproot_internal_key:None` at **`:662`** (inside `build_multisig_import_payload`; call site `:1034`; `:636` doc). (Single-sig `None` at `:606` left alone.)
- `derive_address.rs:26` `derive_first_address` (body `:34-66`: `MsDescriptor::from_str → into_single_descriptors → derive_at_index(0) → .address`) — the descriptor-string address path the taproot branch reuses; caveat header `:24-25` to relax/inline.
- `template.rs:213` `is_nums: true` (v0.48.0, shared `TrMultiA|TrSortedMultiA` arm `:194`) — the dissolved-C1 anchor; `:450 assert!(is_nums…)` (flipped from v1's `!is_nums`).
- `wallet_export/pipeline.rs:18` `build_descriptor_string(…, Option<TaprootInternalKey>)`; `:128` Nums arm (all cosigners in leaf — byte-mirrors bundle), `:135` Cosigner arm.
- `wallet_export/mod.rs:86` `enum TaprootInternalKey { Nums, Cosigner(u8) }`.
- `template.rs:39/42` `CliTemplate::{TrMultiA, TrSortedMultiA}`.
- `bundle.rs:1036` `extract_multisig_threshold` (recurse `Body::Tr { tree: Some(inner), .. }` at `:1042`). **(was mis-cited `:1021`.)**
- `canonicalize.rs:420` `expand_per_at_n` (tree-shape-agnostic; iterates `0..d.n`).
- md-codec `0.35.0` `tree.rs:49-54` `Node { pub tag, pub body }`, `Body::Tr { is_nums, key_index, tree }`; `to_miniscript.rs:406-410` SortedMultiA error (the path we BYPASS for BOTH descriptor build AND address derivation), `:394-398` MultiA renders; `derive.rs:120` `derive_address` re-enters `to_miniscript_descriptor` (why `d.derive_address` is unusable for sortedmulti_a).

## 5. Test plan (per-phase TDD; RED before impl)
- **Reconstruction (lib/CLI):** build a 2-of-3 tr-multi-a bundle + a 2-of-3 tr-sortedmulti-a bundle (real distinct cosigner xpubs) → decode its md1 → `restore --md1` reconstructs the descriptor; assert it equals the independent `export-wallet --template tr-{multi-a,sortedmulti-a} --taproot-internal-key nums --format descriptor` output (the NUMS oracle). RED today (hits the `:777` refusal).
- **Round-trip md1 oracle (§3):** reconstructed descriptor re-encodes to the byte-identical input md1 chunks.
- **`--format`:** `restore --md1 --format bitcoin-core` (and one more) on a tr md1 emits importable taproot watch-only payloads (threshold/internal-key correct).
- **Refusals:** an `is_nums:false` tr md1 → the §2 pointer message (exit 2); a single-key `Tag::Tr` md1 → "leaf is not a multisig."
- **Address derivation (C1(new) regression — load-bearing):** the FULL `restore --md1` CLI on a **`tr-sortedmulti-a`** md1 must succeed end-to-end and emit a `bc1p…` first receive address (proving the address path routes around md-codec, not through `d.derive_address` which errors on SortedMultiA). Mirror for `tr-multi-a`. This cell goes RED against a naive `d.derive_address` implementation.
- **Watch-only invariant:** output carries zero xpriv; first address is `bc1p…` (taproot).
- **Negative discrimination:** flip the reconstructed internal key (Nums→Cosigner) in a temp mutation → the oracle test FAILs (proves it pins the NUMS shape, not a vacuous compare).

## 6. Verification
Full `cargo test` (post version bump) + `cargo clippy --all-targets -- -D warnings`; `lint_argv_secret_flags` N/A (no new secret flag); no `schema_mirror` / gui-schema surface change.

## 7. Ship plan
1. Implement §1 in `restore.rs` (replace the `:777` refusal with the taproot handler; thread `Some(internal_key)` into `--format`'s `build_multisig_import_payload`). Per-phase reviewer-loop → GREEN.
2. Tests (§5).
3. Version bump **PATCH** (next patch): `Cargo.toml` + `Cargo.lock` + **BOTH** README markers + `scripts/install.sh` self-pin — all before the tag; re-run full suite AFTER the bump (the v0.48.0 release-gate lessons: `readme_version_current.rs` + `install-pin-check.yml`).
4. Manual: a prose note under `restore`/recovery recipes that taproot multisig md1 (NUMS) now reconstructs (no flag-coverage delta).
5. Tag → push; confirm `rust.yml` + `install-pin-check` green.
6. Flip `restore-multisig-taproot-reconstruction` → resolved; **collapse its layered-correction body** to the clean v2 state; file the **descriptor-mnemonic companion** for `md-codec-sortedmulti-a-to-miniscript-rendering-gap` (currently only in the toolkit FOLLOWUPS — the promised mirror does not exist) noting restore no longer depends on it (route-around).
7. Memory.

### Out of scope
- is_nums:false reconstruction; md-codec-native sortedmulti_a lowering; multi-leaf taptrees; single-sig tr (already shipped via bip86 restore); any signing.
