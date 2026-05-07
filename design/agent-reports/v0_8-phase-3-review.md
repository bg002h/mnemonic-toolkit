# v0.8 Phase 3 Review — export-wallet enhancements

**Scope:** `wallet_export.rs`, `cmd/export_wallet.rs`, `tests/cli_export_wallet.rs`. 2 items: tr-multi-a / tr-sortedmulti-a with `--taproot-internal-key`; `--descriptor + --format bip388` interop.

**Verdict:** No critical bugs. One Important finding (n=1 cosigner-internal degenerate path).

---

## Critical

None.

- **NUMS hex** (`50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`): correct. Canonical unspendable NUMS point per BIP-341 supplementary material.
- **`tr(NUMS, multi_a(...))` parse round-trip:** `build_descriptor_string` calls `MsDescriptor::from_str` then `.to_string()`; success test passes.
- **Cosigner-internal key removal ordering:** `enumerate().filter(|(i, _)| *i != idx).map(|(_, s)| s)` preserves ascending index order. 3-cosigner `@1` correctly leaves indices 0+2 in order.

---

## Important

### I1 — n=1 cosigner-internal produces opaque parse error (confidence 82)

**File:** `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` lines ~244–277.

With `n=1` and `--taproot-internal-key @0`:
- `leaf_count = n - 1 = 0`
- `k = threshold.unwrap_or(0)` → 0
- `k <= leaf_count` passes (both 0)
- `idx=0 < n=1` passes bounds check
- `build_tr_multi_a_descriptor` emits `tr([origin]xpub/<0;1>/*,multi_a(0,))` — threshold 0, empty leaf set

Miniscript rejects this with a `DescriptorParse` error, not a legible `BadInput`. User sees an internal error instead of an actionable message.

**Fix:** add a guard right after the `leaf_count` assignment:

```rust
if leaf_count == 0 {
    return Err(ToolkitError::BadInput(
        "--taproot-internal-key @N with a single cosigner leaves no multi_a leaves; \
         supply at least 2 cosigners".into(),
    ));
}
```

Plus a regression test pinning the new clean refusal.

---

## Verified-correct items

1. **`descriptor_to_bip388_wallet_policy` substitution.** Longest-first sort is sound. `iter_pk()` strings have form `[fp/path]XPUB/<0;1>/*`; two such strings share a prefix only when they share xpub+path, which miniscript rejects at parse time as duplicate keys. Inserted placeholders (`@N/**`, 5 chars) cannot be substrings of subsequent full key strings (~100+ chars). No silent mis-substitution possible.

2. **`#checksum` strip from canonical descriptor.** `rfind('#')` is correct. BIP-380 checksums are `#` + exactly 8 base32 chars; `#` does not appear in valid descriptor bodies.

3. **`leaf_count = n-1` default threshold.** Auto-default `k = leaf_count = n-1` is correct for cosigner-internal taproot. Test pins `--threshold 1` with 2 cosigners (leaf_count=1), matching default.

4. **`#[allow(dead_code)]` on `taproot_multisig_unsupported_message`.** Annotation + rationale comment are clear; retention is intentional for `ToolkitError` variant message stability.

---

## Resolution actions applied

- **I1:** added n=1 guard with clean `BadInput` refusal text + regression test pinning the new behavior.
