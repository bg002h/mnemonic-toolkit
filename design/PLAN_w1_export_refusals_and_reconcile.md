# PLAN — Open-followups Maturity Program, Wave 1 remainder (export refusals + reconcile)

**Date:** 2026-06-22  **Cycle:** toolkit **v0.70.1** (PATCH) + doc reconciliation
**Source SHA pinned:** `origin/master` = `1cea85ea` (the Wave-1 fmt re-baseline tip). All citations below grep-verified against this SHA.
**Parent:** `design/PROGRAM_open_followups_maturity_2026-06-22.md` Wave 1.
**Status:** producer pass; **awaiting R0** (0C/0I gate before any code).

---

## 0. Scope after recon

Wave 1's fmt-gate keystone shipped (`1cea85ea`). The remaining Wave-1 items, after 4 parallel recon agents verified each FOLLOWUP against current source:

| # | Slug | Verdict after recon | Disposition |
|---|---|---|---|
| 2 | `export-wallet-green-tr-policy-singlesig-emission` | REAL bug, funds-adjacent (mislabel) | **SHIP** (PATCH) |
| 3 | `export-wallet-direct-descriptor-unsorted-multi-generic-refusal` | REAL, message-quality only | **SHIP** (NO-BUMP) |
| 4 | `xpub-search-descriptor-md1-detection-bech32-validate` | LOW-value tightening + **protocol trap** | **DEFER** (recommend) — R0 to rule |
| 5 | `lint-md-flag-coverage-vacuous-with-md_bin-true` + `manual-yml-bind-real-mnemonic-bin` + `manual-md-bin-real-binary-promote` + `manual-ms-bin-real-binary-promote` | **ALREADY RESOLVED in CI** | **Doc reconcile only** |

Combined SemVer: **PATCH v0.70.1** (driven by #2's behavior change; #3 NO-BUMP folds in; #4 deferred; #5 doc-only).

---

## 1. Item #2 — green emitter refuses a tap-script-tree policy (funds-adjacent, PATCH)

### Current behavior (verified)
`crates/mnemonic-toolkit/src/wallet_export/green.rs:36` — `emit()` refuses only `inputs.script_type.is_multisig()`. A **general taproot policy** (`tr(internal,{...})` with a tapscript tree) is classified `WalletScriptType::P2tr` by `script_type_from_descriptor` (no `multi_a(`/`sortedmulti_a(` substring), so it is **not** multisig → falls through → green.rs emits the 3-line file with the static `# … (singlesig)` header (green.rs:41-44). The descriptor *inside* is faithful, so this is a **wrong-LABEL, not wrong-address** bug (funds-adjacent, on the override axis).

### Restore-side precedent (to mirror)
`crates/mnemonic-toolkit/src/cmd/restore.rs:2767` already refuses: `"--format green cannot emit a taproot policy descriptor — Green's file-import surface is singlesig-only, and this md1 restores a tap-script-tree policy. Use --format bitcoin-core or --format descriptor for a watch-only import."` Restore refuses **all** `P2tr` because in its route-around arm a `P2tr` is *always* a tap-script-tree policy.

### Why export-wallet discriminates (refuse tap-script-tree) rather than blanket-refusing all P2tr
`CliTemplate::Bip86` exists (`template.rs:23-24`); `export-wallet --template bip86 --format green` produces a **keypath-only** single-sig taproot `tr(KEY/…)` — also `WalletScriptType::P2tr` — and emits to green **today** (an untested structural fall-through). The reported bug is the **tap-script-tree policy** mislabel only. **R0 I-2 correction:** whether Green's *file* import actually accepts a `tr(KEY)` keypath descriptor is **unverified** — neither the manual, repo tests, nor Blockstream's Help Center confirm it (the "singlesig-only" wording is stated only in contrast to *multisig*). We therefore choose the **conservative** disposition: **discriminate** — refuse the tap-script-tree policy (the reported bug), and **leave the existing bip86-keypath emission unchanged** (do not *remove* an existing capability without positive justification, per project convention). Both discriminate and blanket-refuse fix the reported bug; discriminate is preferred because it does not silently drop a working path. The open question (does Green import a `tr(KEY)` file?) is **tracked, not assumed** — see the verify-FOLLOWUP below.

### The discriminator MUST be structural, not substring
A single-leaf taptree renders **without** braces: miniscript Display gives `tr(NUMS,pk(A))` for a one-leaf tree (`,{` appears only for branches). So the recon-draft's `rendered.contains(",{")` probe is **unsound** — it would misclassify a single-leaf policy as keypath-only and emit a mislabeled card. Use miniscript's structural accessor: parse the canonical descriptor and test `Tr::tap_tree().is_some()`.

`EmitInputs.canonical_descriptor` is a `CheckedDescriptor<'a>` string newtype (`wallet_export/mod.rs:504`, derefs to `&str`); the parsed descriptor is not in `EmitInputs`. Parsing inside `green.rs::emit` is correct and **path-uniform** (the canonical descriptor string is the final form for BOTH template and descriptor paths), so one probe handles bip86-template and direct-descriptor alike.

### The change (green.rs::emit, after the multisig guard at :40)
```rust
// A general taproot POLICY (tap-script tree) is classified P2tr but is not a
// Green-importable singlesig wallet. Distinguish keypath-only (BIP86, allow)
// from a tap-script-tree policy (refuse) STRUCTURALLY — a single-leaf tree
// renders without `,{`, so a substring probe is unsound. Mirrors the
// restore-side refusal (restore.rs ~:2767). FOLLOWUP
// `export-wallet-green-tr-policy-singlesig-emission`.
if inputs.script_type == WalletScriptType::P2tr {
    use miniscript::{Descriptor, DescriptorPublicKey};
    use std::str::FromStr;
    let parsed = Descriptor::<DescriptorPublicKey>::from_str(&inputs.canonical_descriptor)
        .map_err(|e| ToolkitError::DescriptorParse(format!("green taproot probe: {e}")))?;
    if let Descriptor::Tr(tr) = parsed {
        if tr.tap_tree().is_some() {
            return Err(ToolkitError::BadInput(
                "--format green cannot emit a taproot policy descriptor — Green's file-import surface is singlesig-only, and this descriptor carries a tap-script-tree policy. Use --format bitcoin-core or --format descriptor for a watch-only import.".into(),
            ));
        }
    }
}
```
- **Error:** reuse `ToolkitError::BadInput` (exit 1) — same variant/exit as the restore-side refusal; no new `enum` variant (no alphabetical-ordering concern).
- **Import:** `WalletScriptType` is already in scope in green.rs via `super::` (used by `is_multisig()` on `script_type`); confirm at impl time (re-grep the `use super::{…}`); add `WalletScriptType` to the import if absent.

### Tests (TDD — write first, in `crates/mnemonic-toolkit/tests/cli_export_wallet_green.rs`)
1. **`cell_5_green_general_taproot_refuses`** — `export-wallet --format green --descriptor 'tr(NUMS,{pk(A),pk(B)})…'` → exit 1, stderr contains `singlesig-only`. (Branch policy — the `,{` case.)
2. **`cell_6_green_single_leaf_taproot_refuses`** — `tr(NUMS,pk(A))` (single-leaf, **no** `,{`) → exit 1. **This is the test that proves the structural check beats the substring draft** — it would PASS-wrongly (emit) under `,{`.
3. **`cell_7_green_bip86_keypath_emission_unchanged`** — `export-wallet --template bip86 … --format green` → exit 0, stdout has `tr(` + the singlesig header. **R0 I-2 reframe:** this is a **behavior-pinning, no-regression** guard ("the fix does not blanket-refuse P2tr; bip86 keypath emission is byte-unchanged from current behavior"), **NOT** a correctness assertion that Green can import the file (which is unverified). Comment the test accordingly.

### New FOLLOWUP to file (R0 I-2)
`green-taproot-keypath-file-import-unverified` — verify against Blockstream Green whether a `tr(KEY)` keypath descriptor *file* actually imports. If it does NOT, a future cycle escalates `green.rs::emit` to blanket-refuse all `P2tr` (keypath included). Until verified, the bip86→green emission is preserved as status-quo. Tier: `verify`/`next-cycle`. Companion: `export-wallet-green-tr-policy-singlesig-emission` (this cycle).

### SemVer: **PATCH** (behavior change: was silently emitting a mislabeled descriptor, now refuses loudly; keypath single-sig unchanged).

---

## 2. Item #3 — typed unsorted-multisig error on the direct-descriptor path (NO-BUMP)

### Current behavior (verified — M-1 live citations)
Typed variant `ToolkitError::ExportWalletUnsortedMultisigUnsupported { format }` already shipped (v0.62.0), at `error.rs:177-179` (the `:170-176` span is the doc comment), exit 2. The existing H10 guard at `cmd/export_wallet.rs:124-137` (the `:109-123` span is the doc comment) fires only when `inputs.template == Some(WshMulti|ShWshMulti)`. On the **direct-descriptor** path (`--descriptor 'wsh(multi(…))'`, `template == None`) the H10 guard does not fire; each field-less emitter's own `ok_or_else` refuses first with **generic `BadInput`** ("requires --template") — `wallet_export/electrum.rs:51`, `jade.rs:34`, `coldcard.rs:111`. Already funds-safe (refused, never coerced); only the message is generic. (Re-grep all spans at impl time — citations decay.)

A unit test pins the *current* generic boundary: `cmd/export_wallet.rs` `template_none_falls_through_to_generic_badinput_not_h10` (~:1105) asserts the refusal is **NOT** the typed kind; its fixture `inputs_with_template(None)` (~:951) carries `wsh(multi(2,…))#…` (unsorted) + `script_type = P2wshMulti`. **This test must be flipped** by the fix (the fixture is already consistent with the flipped `assert_eq!`).

### The change (M-2 — a SECOND ARM of the existing H10 guard, not a new guard)
The H10 guard and its typed variant already exist. This item adds a **second arm** for the direct-descriptor path: at the `emit_payload` chokepoint (`cmd/export_wallet.rs` ~:73, immediately after the existing H10 guard region :124-137 — verified REACHABLE before any per-format emitter `ok_or_else`, so not dead code), guard on `template.is_none()` ∧ `script_type ∈ {P2wshMulti, P2shP2wshMulti}` ∧ `format ∈ {Electrum, Coldcard, ColdcardMultisig, Jade}` ∧ the descriptor is **unsorted** multi. Return `ExportWalletUnsortedMultisigUnsupported { format }`.

**Sorted-ness detection:** mirror the existing in-repo idiom `wallet_export/mod.rs:264` (`template_from_descriptor` uses `d.to_string().contains("sortedmulti(")`). Test `canonical_descriptor.contains("multi(") && !canonical_descriptor.contains("sortedmulti(")`. (`"sortedmulti("` contains `"multi("`, so the negative clause is required.) **R0 question (Q-C):** prefer reusing `template_from_descriptor` to derive the effective template and route through the existing H10 guard, vs the inline substring? Substring is consistent with the existing helper; reuse is DRYer but couples to that fn's return contract. Recommend the **inline substring guard** (smallest diff, same idiom as the only other sorted-ness check in the file).

### Tests
- **Flip** `template_none_falls_through_to_generic_badinput_not_h10` → `template_none_hits_typed_h10_error` (assert `err.kind() == "ExportWalletUnsortedMultisigUnsupported"` for each FIELDLESS format).
- **Extend** `tests/cli_export_wallet_unsorted_multi_refusal.rs` `direct_descriptor_unsorted_multi_refused_not_silently_coerced` to also assert the stderr message is the typed one (mentions sortedmulti / a faithful format), pinning message-quality.
- **No-change guard:** a `sortedmulti(` direct descriptor to a field-less format still refuses via the emitter's own generic `--template`-required path (unchanged) — assert it is NOT the typed unsorted error (the typed error must be specific to *unsorted*).

### SemVer: **NO-BUMP** (still refuses, same exit code 2; message-quality only).

---

## 3. Item #4 — xpub-search md1 detection tightening: **DEFER** (R0 to confirm)

### Why defer (R0 I-1 corrected justification)
- **No live defect to fix.** Current behavior is already loud-and-safe: `descriptor_intake.rs:156` routes by case-insensitive `"md1"` prefix → `parse_md1` (:210) → `md_codec::chunk::reassemble` (:224) → typed `ToolkitError::MdCodec` (exit 1) on checksum failure. A `md1`-prefixed non-md1 string surfaces a clean typed error, **never** a silent misroute. The FOLLOWUP itself calls today's behavior "defensible."
- **The bech32 approach in the slug is fundamentally wrong (protocol trap, verified).** md1 is a **custom BCH(93,80,8) code** — 13-symbol checksum, 5×65-bit generator, target residue `md_codec::bch::MD_REGULAR_CONST` (≠ bech32 residue `1` / bech32m `0x2bc830a3`). `bitcoin::bech32::decode("md1…")` fails the checksum on **every** valid md1 string → the recon-draft's `bech32::decode(t).is_ok()` would misroute valid md1 cards to literal-xpub = **funds-feature regression**. The recon's "all real md1 chunks pass bech32 decode" claim is false.
- **A correct validator exists and is cheap** — `md_codec::bch::bch_verify_regular(hrp, data_with_checksum) -> bool` (`descriptor-mnemonic/crates/md-codec/src/bch.rs:89`), a public per-chunk checksum check the toolkit already partially uses (`repair.rs:46` imports `md_codec::bch::MD_REGULAR_CONST`; `repair.rs:652-692` has its own per-chunk residue check). So this is **NOT** sibling scope creep. (Note md1 is multi-chunk up to 64 chunks; the whole-descriptor `decode_md1_string` at `decode.rs:86` ≠ the multi-chunk `reassemble` at `chunk.rs:306`, which is why a naive per-token full-decode is also wrong — a per-token check must be chunk-level like `bch_verify_regular`.)

### Recommendation (R0-ruled: DEFER)
**Defer** — there is no live defect, and the value of tightening (which error you get for `md1garbage`) is negligible over the existing typed `MdCodec` error. **Amend the FOLLOWUP** to (a) record that `bitcoin::bech32::decode` is wrong for md1's custom BCH code, and (b) point any future attempt at the **correct** primitive `md_codec::bch::bch_verify_regular` (per-chunk), NOT a sibling change. Do not implement in this cycle.

---

## 4. Item #5 — vacuous flag-coverage lint family: **doc reconciliation only**

### Verified already-resolved in CI
`.github/workflows/manual.yml:98-109` runs `make audit` (= lint + verify-examples) binding **all four real binaries**: `MNEMONIC_BIN="$GITHUB_WORKSPACE/target/debug/mnemonic"` (built at :96), `MD_BIN=md` / `MS_BIN=ms` / `MK_BIN=mk` (installed via `cargo install --git … --tag …` at :79/:86/:90). The flag-coverage gate is fully real-binary-bound; the `MD_BIN=true`/`MNEMONIC_BIN=true` vacuity the FOLLOWUPs describe is **gone**. The md/ms promotions tracked by the successor slugs landed since those slugs were filed (stale-open).

### Action (doc-only, no code, no R0 needed for the flips themselves — but enumerated here for the cycle commit)
Flip to `resolved` with a one-line "verified real-binary-bound at manual.yml:98-109 (`make audit`), 2026-06-22":
- `lint-md-flag-coverage-vacuous-with-md_bin-true`
- `manual-yml-bind-real-mnemonic-bin` (was `resolved-partial`)
- `manual-md-bin-real-binary-promote`
- `manual-ms-bin-real-binary-promote`

**Out of scope (Wave 3):** the `descriptor-mnemonic-md-cli-v0.6.2` pin on `manual.yml:86` is stale vs install.sh canonical (and vs the actual latest md-cli) — that is the Wave-3 pin-staleness cluster (`manual-yml-sibling-pin-vs-install-sh-drift-gate`), NOT this item. Do **not** touch it here. (Re-grep at flip time to confirm none of these four was already flipped by an intervening cycle.)

### SURFACED (R0 M-4) — a SECOND live-RED CI gate, NOT folded into v0.70.1
**Confirmed:** `scripts/install.sh:35` canonical md-cli = `v0.7.1`, but `manual.yml:86` (+ a second workflow) pin `v0.6.2` → the already-`resolved` `sibling-pin-check.yml` gate (`exit 1` on `tag != canonical`) is **live-RED on master right now** (drifted since the v0.7.1 bump landed without a manual.yml update). Independently, install.sh's own canonicals are broadly stale (md v0.7.1 vs published **v0.9.2**; ms v0.7.0 vs **v0.10.0**; mk v0.8.0 vs **v0.10.1**). **The proper fix is a dedicated cycle** (lockstep bump install.sh + all workflow pins to latest + reconcile any md/ms/mk flag-coverage deltas in the manual chapters — the bump can cascade into the flag-coverage gate), NOT a v0.70.1 one-liner. **Ship-gate discipline:** v0.70.1's "CI green" criterion must be evaluated as "*my changes'* gates (fmt, clippy, test, schema_mirror) are green AND I did not introduce new RED" — it must NOT be read as "all of master CI is green" (sibling-pin-check was already RED, pre-existing). Surface to the user; recommend it as the next broken-gate cycle after v0.70.1.

---

## 5. Build/release plan

1. Branch `fix/w1-export-refusals` off `1cea85ea` (worktree for the single-subagent TDD impl).
2. TDD per item (#2 then #3): tests first (RED) → impl → GREEN. **M-3 (definite step):** add `WalletScriptType` to green.rs's `use super::{…}` (currently `green.rs:19` = `use super::{EmitInputs, MissingField, WalletFormatEmitter};` — `WalletScriptType` is NOT imported). Full `cargo test -p mnemonic-toolkit` (NOT targeted — per the stale-lint lesson). `cargo clippy`. `cargo +1.95.0 fmt --all -- --check` (mlock-exempt) stays GREEN.
3. Doc: flip #5 family slugs (4); flip #2's slug; leave #3's slug resolved-noted; amend #4's slug (defer + trap doc → `bch_verify_regular`); file the new `green-taproot-keypath-file-import-unverified` FOLLOWUP.
4. Version: bump toolkit `0.70.0 → 0.70.1` at ALL 6 version sites — `Cargo.toml:3`, `README.md:13`, `crates/mnemonic-toolkit/README.md:9`, **`scripts/install.sh:32`** (self-pin; M-5 — path is `scripts/install.sh`, not root), `fuzz/Cargo.lock:575`, regenerate root `Cargo.lock:727` — per the release-ritual memory; the `both_readmes`/version lints gate this.
5. **GUI schema-mirror:** this cycle adds **no clap flag/subcommand/dropdown** (only refusal behavior + messages) → `schema_mirror` is **not** tripped; **no GUI paired-PR needed**. Manual mirror: no flag surface change → no `40-cli-reference` edit required (confirm at impl time that no `--help` text changed).
6. Post-impl: mandatory independent adversarial whole-diff review (per the 5-step ultracode pattern). Persist verbatim to `design/agent-reports/`.
7. Ship: direct-FF to master + tag `mnemonic-toolkit-v0.70.1`. No codec/mk/ms bump; no GUI bump.

---

## 6. Open questions — R0-RULED (round 1, 0C/2I/5M → folded)

- **Q-A (#2):** **DISCRIMINATE — ruled.** Refuse tap-script-tree (structural `Tr::tap_tree().is_some()`, accessor verified at `src/descriptor/tr/mod.rs:104` on pinned rev `95fdd1c5773…`), preserve bip86-keypath emission. The "Green imports a `tr(KEY)` file" premise is unverified → the bip86 test is behavior-pinning (not correctness) + verify-FOLLOWUP filed (I-2). The substring draft is empirically disproven (single-leaf `tr(NUMS,pk(A))` has no `,{` but `tap_tree().is_some()`).
- **Q-B (#4):** **DEFER — ruled** (I-1 corrected justification: no live defect; bech32 wrong for custom BCH; trap-doc cites `md_codec::bch::bch_verify_regular`).
- **Q-C (#3):** **INLINE SUBSTRING — ruled** (smallest diff; matches `mod.rs:264` idiom; more robust than `template_from_descriptor` for the placeholder-xpub fixture).
- **Q-D (SemVer):** **CONFIRMED** — single toolkit **PATCH v0.70.1**, one tag, no codec/mk/ms/GUI bump.
- **Q-E (scope):** **CONFIRMED** — #5 doc-only; the md-cli-v0.6.2 pin-drift is held for a dedicated cycle, BUT surfaced (M-4) as a live-RED `sibling-pin-check` gate, not silently deferred.

## 7. R0 status
- **Round 1:** 0 Critical / **2 Important** (I-1, I-2) / 5 Minor → all folded above (review persisted verbatim at `design/agent-reports/w1-export-refusals-r0-round1-review.md`). Folds are reframe/justification + one new FOLLOWUP; no code-design change (the #2 structural fix + #3 second-arm are R0-verified correct).
- **Round 2:** re-dispatch architect to confirm convergence to 0C/0I (per the "reviewer-loop continues after every fold" rule). Implementation gated on round-2 GREEN.
