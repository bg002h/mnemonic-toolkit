# SPEC — `mnemonic restore --md1` taproot (NUMS) multisig reconstruction

> ⛔ **RED / SUPERSEDED — DO NOT IMPLEMENT (2026-06-05).** This SPEC FAILED R0
> (2 Critical — see `design/agent-reports/restore-multisig-taproot-r0-r1-review.md`)
> on a **false premise**: it assumes `bundle` emits a NUMS-internal tr md1, but
> `bundle`/`wrapper_node` emits `is_nums:false, key_index:0` → `tr(@0,
> sortedmulti_a(k, @0…@n-1))` (`template.rs:209`), which no reconstruction path
> reproduces. The work is **BLOCKED on prerequisite `toolkit-trmultia-nums-internal-key`**
> (fix bundle tr emit → NUMS); user-deferred 2026-06-05. Retained as an audit record
> of the approach + why it doesn't work yet. The §1–§8 below describe the
> (invalid-as-scoped) NUMS-reuse plan; revive only after the prerequisite lands. See
> `design/FOLLOWUPS.md` `restore-multisig-taproot-reconstruction` + `toolkit-trmultia-nums-internal-key`.

**Status:** ⛔ RED (R0-r1, 2C) — superseded/blocked; do not implement (was: R0 gate pre-implementation).
**Resolves:** FOLLOWUP `restore-multisig-taproot-reconstruction` (NUMS-internal-key scope).
**Source SHA:** branch `restore-multisig-taproot-reconstruction` off master `b9d6ea1` (toolkit v0.45.0); md-codec pinned `0.35.0` (no bump).
**SemVer:** MINOR — un-refuses an existing `restore --md1` invocation for a taproot md1 (new capability, no new flag).

---

## 1. Summary

`restore --md1` reconstructs wsh/sh(wsh) multisig from a wallet-policy md1, but **refuses taproot** at the `Tag::Tr` pre-gate (`restore.rs:857`, exit 2). The original FOLLOWUP framed this as needing a bespoke tr-string emitter + hand-computed BIP-386 checksum — **runtime re-check (2026-06-05) proved that wrong**: the toolkit's own `build_descriptor_string(template, &slots, k, network, account, Some(TaprootInternalKey::Nums))` (`wallet_export/pipeline.rs:18`, used by `export-wallet`) ALREADY emits a valid `tr(sortedmulti_a(2,…))#caply36x` (rust-miniscript v13's STRING parser handles it; only md-codec's manual-`Terminal` path lacks `SortedMultiA`). So restore just needs to **route around md-codec's `to_miniscript_descriptor`** for the taproot branch: read template + internal-key + k + slots off `d.tree` and reuse `build_descriptor_string`. ~50-120 LOC; toolkit-only.

**Scope: NUMS-internal-key only.** Every *toolkit-emitted* tr md1 is NUMS-internal — `bundle` (the sole md1 emitter) renders `tr(sortedmulti_a(K,@0,…,@N-1))` with all N cosigners in the leaf and has **no** `--taproot-internal-key` flag. A cosigner-internal-key tr md1 (`Body::Tr{is_nums:false}`) is not toolkit-emittable and needs leaf-key juggling (`build_tr_multi_a_descriptor` removes cosigner N from the leaf, `pipeline.rs:135`) → **refuse + spawn FOLLOWUP** rather than build it now.

## 2. Empirical ground truth (v0.45.0 binary)

- `export-wallet --template tr-sortedmulti-a --taproot-internal-key nums --format descriptor` → `tr(sortedmulti_a(2,…))#caply36x` (valid). `build_descriptor_string` round-trips via `MsDescriptor::from_str` (`pipeline.rs:28`) → rust-miniscript parses/serializes tr(sortedmulti_a).
- `bundle --template tr-sortedmulti-a --threshold 2 --slot @0..2.phrase=…` succeeds, emits a 6-chunk md1; `bundle --help` shows `tr(sortedmulti_a(K,@0,…,@N-1))` + NO `--taproot-internal-key` → NUMS-only.
- `extract_multisig_threshold(&d.tree)` recurses `Body::Tr{tree:Some(inner),..}` (`bundle.rs:1021`) → returns k.
- `Body::Tr { is_nums: bool, key_index: u8, tree: Option<Box<Node>> }` (`md-codec tree.rs:49-56`, `pub`); the Tr child `Node.tag` (`tree.rs:11`) is `Tag::SortedMultiA` or `Tag::MultiA`.
- `TaprootInternalKey { Nums, Cosigner(idx) }` (`wallet_export/mod.rs:86`); `build_descriptor_string_inner` routes `TrMultiA|TrSortedMultiA` → `build_tr_multi_a_descriptor` (`pipeline.rs:101`), `Nums` → all cosigners in the `sortedmulti_a`/`multi_a` leaf (`:128`).

## 3. Implementation — `restore.rs` `run_multisig`

### 3.1 Replace the `Tag::Tr` refusal (`:857`) with template/internal-key classification

Today: `if d.tree.tag == Tag::Tr { return ModeViolation }`, then `to_miniscript_descriptor(&d,0)` (`:873`, refuses Tr) → `template_from_descriptor` (`:875`, refuses Tr). Restructure to compute `(template, tap_ik)` up front:

```rust
let (template, tap_ik): (CliTemplate, Option<TaprootInternalKey>) =
    if d.tree.tag == md_codec::Tag::Tr {
        match &d.tree.body {
            md_codec::tree::Body::Tr { is_nums: true, tree: Some(child), .. } => {
                let t = match child.tag {
                    md_codec::Tag::SortedMultiA => CliTemplate::TrSortedMultiA,
                    md_codec::Tag::MultiA       => CliTemplate::TrMultiA,
                    other => return Err(ToolkitError::ModeViolation { mode:"restore", flag:"--md1",
                        message:"taproot md1 leaf is not (sorted)multi_a — unsupported" }),
                };
                (t, Some(TaprootInternalKey::Nums))
            }
            md_codec::tree::Body::Tr { is_nums: false, .. } => {
                return Err(ToolkitError::ModeViolation { mode:"restore", flag:"--md1",
                    message:"taproot md1 with a cosigner-internal-key (tr(@N,…)) is not yet supported \
                             (toolkit emits NUMS-internal tr only) — FOLLOWUP restore-multisig-taproot-cosigner-internal-key" });
            }
            _ => return Err(ToolkitError::ModeViolation { mode:"restore", flag:"--md1",
                message:"taproot md1 has no script tree — unsupported" }),
        }
    } else {
        // wsh / sh(wsh): classify via md-codec (unchanged path).
        let ms0 = md_codec::to_miniscript::to_miniscript_descriptor(&d, 0)
            .map_err(|e| bad(format!("--md1 → descriptor: {e}")))?;
        (wallet_export::template_from_descriptor(&ms0)?, None)
    };
```

The `is_wallet_policy()` gate (`:864`) stays (applies to both branches; move it just before this block or keep at `:864` so it precedes classification). `k = extract_multisig_threshold(&d.tree)` and the `expand_per_at_n` slot-building (`:880-915`) are **shared, unchanged** (template-agnostic). The final descriptor build becomes `build_descriptor_string(template, &slots, k, network, args.account, tap_ik)` (was hard-coded `None`).

### 3.2 First-address derivation (`:917-924`) — taproot can't use `d.derive_address`

`d.derive_address` (md-codec) routes through the blocked `to_miniscript` → unusable for tr. Branch on `tap_ik.is_some()`:
- **taproot:** parse the built `descriptor` string via `MsDescriptor::<DescriptorPublicKey>::from_str`, then derive indices `0..args.count` from the **receive** branch via `into_single_descriptors()[0].derive_at_index(i)?.address(network)` — generalize the existing `derive_address::derive_first_address` (`:26`, which does index 0 only) into a `derive_receive_address(&Descriptor, index: u32, network) -> Result<String>` (have `derive_first_address` call it with 0 to avoid duplicating the multipath-split + wildcard logic). rust-miniscript renders tr addresses natively.
- **wsh/sh(wsh):** keep `d.derive_address(0, i, network)` unchanged (no regression).

### 3.3 Cross-check (6a/6b), output (steps 7-9), `--format` — UNCHANGED

The `--from`/`--cosigner` cross-check is key-derivation + 65-byte compare (`derive_bip32_from_entropy_at_path` → `xpub_to_65`), template-agnostic — works for taproot as-is. The descriptor/json/text output + banners are unchanged. **`--format` (v0.45.0) works for the taproot descriptor automatically** — `build_multisig_import_payload` builds a multisig `EmitInputs` with `template: Some(TrSortedMultiA)` + `taproot_internal_key`… **NB for R0:** the v0.45.0 multisig `--format` path sets `taproot_internal_key: None`. For a taproot wallet that is WRONG — it must pass `tap_ik`. Fix `build_multisig_import_payload` to take + forward `taproot_internal_key` (currently hard-coded `None`, `restore.rs:~690`). (Without this, `--format` on a taproot md1 would mis-build. Add a taproot `--format` test cell.)

## 4. Refusals
- `Body::Tr{is_nums:false}` (cosigner-internal-key) → ModeViolation exit 2 + FOLLOWUP pointer.
- A taproot md1 whose Tr child isn't `(Sorted)MultiA`, or has no tree → ModeViolation exit 2.
- Template-only md1 (`!is_wallet_policy()`) → existing refusal (unchanged).

## 5. Tests — `tests/cli_restore_multisig_taproot.rs` (new)
Fixture: `bundle --template tr-sortedmulti-a|tr-multi-a --threshold 2 --slot @0..2.phrase=C0/C1/C2` → md1 (NUMS).
- **tr-sortedmulti-a reconstruct:** `restore --md1` exit 0 → stdout contains `tr(sortedmulti_a(2,` + `<0;1>/*` + `#` checksum + a `bc1p…` taproot address + the 3 cosigner fps + UNVERIFIED.
- **tr-multi-a reconstruct:** → `tr(multi_a(2,`.
- **Round-trip oracle:** the reconstructed descriptor equals the wallet's true descriptor — compare `restore --md1 --json` descriptor against the descriptor derived independently by deriving each cosigner's account xpub via `convert --to xpub --template tr-sortedmulti-a` and asserting the same sortedmulti_a key set + checksum (or a pinned constant captured from a known-correct build — see [[feedback_recapture_golden_only_when_current_correct]]: capture only after proving correct via an independent miniscript derivation).
- **`--from` own-seed cross-check** on a tr md1 → position inferred, PARTIAL (taproot cross-check parity).
- **`--cosigner @N=mk1|xpub`** on a tr md1 → cross-check ok.
- **mismatch** (`--from` FOREIGN) → exit 4, no descriptor.
- **watch-only-out:** no `xprv`/`tprv` in any channel.
- **`--format descriptor`** on a tr md1 → emits the `tr(sortedmulti_a)` descriptor (guards §3.3 the `tap_ik`-forwarding fix); **`--format bitcoin-core`** → payload `desc` carries `tr(sortedmulti_a(2,`.
- **`--count 3`** → 3 distinct `bc1p…` receive addresses (guards the `derive_receive_address` index loop).
- **`is_nums=false` refusal** — if a cosigner-internal-key tr md1 is constructible in-test (likely NOT via bundle; if unconstructible, assert via a unit/doc note instead) → exit 2 + FOLLOWUP pointer.

## 6. Lockstep / FOLLOWUPs
- **GUI `schema_mirror`: NO change** (no clap flag/value-enum change — `--md1` pre-existing; templates aren't a restore flag). Confirm at R0.
- **Manual:** `docs/manual/src/40-cli-reference/41-mnemonic.md` `### Multisig-cosigner restore` **Scope** line (`:961`-ish, currently "`wsh`/`sh(wsh)` only … taproot refused") → taproot **NUMS** now reconstructs; cosigner-internal-key tr refused. Run `make audit` (anchor-check + verify-examples), not just `make lint`.
- **NEW FOLLOWUP `restore-multisig-taproot-cosigner-internal-key`:** the `is_nums=false` case (tr(@N, sortedmulti_a(N-1 leaves))) — leaf-key juggling; not toolkit-emittable, so low priority.

## 7. Phased plan
- **Phase 1 (RED):** `tests/cli_restore_multisig_taproot.rs` — all cells; fail (today: `Tag::Tr` → exit 2). Verify RED-for-the-right-reason. **First** confirm at runtime the two R0 verify-items: `expand_per_at_n` returns the N cosigner keys for a NUMS tr md1 and `is_wallet_policy()` is true for it (a quick reconstruct-probe or the RED test's error shape).
- **Phase 2 (GREEN):** §3 (Tag::Tr classification branch; `tap_ik` threaded into `build_descriptor_string` + `build_multisig_import_payload`; `derive_receive_address` generalization + taproot first-address branch). Workspace `cargo test --no-fail-fast` + clippy GREEN. Per-phase opus review → persist.
- **Phase 3 (docs + release):** manual Scope-line update + `make audit`; CHANGELOG; version v0.45.0 → **v0.46.0**; README markers; install.sh self-pin; FOLLOWUP resolve + spawn. Per-phase review.
- **Phase 4 (ship):** clean tree → `git checkout master && ff-merge` → tag `mnemonic-toolkit-v0.46.0` → push master + tag → watch CI (rust, install/sibling-pin-check, manual).

## 8. Risk
Low-moderate. The descriptor build is proven reuse; the genuinely-new code is (a) the `Body::Tr` classification (small, well-typed), (b) the `derive_receive_address` index generalization (mirrors the tested `derive_first_address`), (c) threading `tap_ik` into `build_multisig_import_payload` (a 1-arg change + the existing dispatch). The round-trip test oracle is the main correctness anchor — R0 must ensure it's an INDEPENDENT derivation, not a tautology against restore's own output.
