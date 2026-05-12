# v0.8.1 Phase 2 R1 — reviewer report

## Verdict
**1C / 1I — fold needed**

## Findings

### C-1 — Taproot multisig descriptor passthrough emits `#checksum` suffix inside `miniscript.script`

**File:** `crates/mnemonic-toolkit/src/wallet_export/sparrow.rs:209`

**What:** For `TrMultiA` / `TrSortedMultiA`, `build_miniscript_script` returns `inputs.canonical_descriptor.to_string()`. The `canonical_descriptor` field is produced by `pipeline::build_descriptor_string` which calls `parsed.to_string()` on a `miniscript::Descriptor` — that Display impl produces the BIP-380 form with `#checksum` suffix (e.g., `tr(50929b...,multi_a(2,[fp/path]xpub/<0;1>/*,...))#xxxxxxxx`).

**Why it matters:** Sparrow's `defaultPolicy.miniscript.script` field is a Miniscript policy expression, not a BIP-380 descriptor. For every non-taproot template the emitter writes a bare policy string — `wpkh(@0/**)`, `wsh(sortedmulti(2,@0/**,...))` — with no `#checksum`. The taproot passthrough breaks this invariant by injecting the BIP-380 checksum. Sparrow's `Miniscript.java` applies substring and regex matching on the `script` field to detect policy type and extract the threshold; the `#checksum` portion is not valid miniscript grammar and can cause Sparrow's policy parser to reject or mis-classify the imported wallet.

**Fix:** In `build_miniscript_script` for the taproot passthrough arm, strip the `#<checksum>` suffix:
```rust
CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA => {
    let desc = inputs.canonical_descriptor;
    let script = desc.rfind('#').map_or(desc, |pos| &desc[..pos]);
    Ok(script.to_string())
}
```

**Confidence:** 87

---

### I-1 — cell_5 justification for structural-only test is factually incorrect

**File:** `crates/mnemonic-toolkit/tests/cli_export_wallet_sparrow.rs:231-233`

**What:** The comment reads: "the descriptor contents include derived checksums that change with BIP-32 library updates." BIP-380 checksums are a deterministic function of the descriptor string content. The xpubs in cell_5 are the same compile-time constants used in cells 1-3. Nothing in a BIP-32 library version update changes the checksum of a descriptor whose content is fixed.

**Why it matters:** The incorrect rationale creates technical debt: future maintainers may generalize "use structural assertions whenever descriptors appear" without recognizing that the test fixtures for taproot multisig are perfectly byte-pinneable (once C-1 is fixed and the `#checksum` is stripped from the `miniscript.script` field). The absence of a byte-exact fixture also means a future regression in the keystore `extendedPublicKey` field or `keyDerivation` fields would not be caught.

**Fix:** After folding C-1 (checksum stripping), add a byte-exact fixture `tests/export_wallet/sparrow_tr_multi_a_nums_2of3.json` pinned against the 3-cosigner tr-multi-a input from cell_5. Replace the structural assertions with the same byte-exact comparison pattern used in cells 1-3.

**Confidence:** 82

---

## Confidence-filtered: omitted findings

- **`normalize_derivation` singlesig branch ignores user-supplied `path_raw`** — for singlesig templates the function always returns `template.origin_path_str(...)`. Consistent with fixed canonical paths for singlesig; no fixture exercises this path for Sparrow. Confidence 58 — below threshold.
- **`keystores[].label` is `wallet_name` for all cosigners, not per-cosigner label** — confirmed by fixture. Not a defect. Confidence 50.
- **Missing Phase 2 test coverage for `--template bip44` and `--template bip49`** — `P2PKH` and `P2SH_P2WPKH` script-type mapping untested. Low priority. Confidence 62 — below threshold.
- **`collect_missing` does not guard TrMultiA/TrSortedMultiA emit-side `multi_arg` call** — the taproot passthrough arm in `emit()` does not call `multi_arg` and therefore would succeed without threshold even though `collect_missing` enforces it. The two-layer defense holds correctly; the emit-side cannot be reached without passing `collect_missing`. Not a defect. Confidence 50.
