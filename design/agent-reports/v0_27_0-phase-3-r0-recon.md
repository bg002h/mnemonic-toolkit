# v0.27.0 Phase 3 R0 recon — BSMS Round-2 emitter

**Date:** 2026-05-18. **Plan reference:** `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md` §3.5 / §3.5.1 / §4.3 (R6).

## Helper signature lock

```rust
// crates/mnemonic-toolkit/src/derive_address.rs  (NEW module)

use crate::error::ToolkitError;
use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};

/// SPEC v0.27.0 §3.5 — derive the wallet's first address at canonical /0/0
/// (receive branch index 0, address index 0). Used by:
/// - BSMS Round-2 4-line emitter (line 4 — `wallet_export/bsms.rs`)
/// - import-wallet BSMS parser's first-address WARNING (closes
///   `bsms-first-address-verify` FOLLOWUP at `design/FOLLOWUPS.md:2083`).
///
/// For multipath `<0;1>/*` descriptors: splits via miniscript's
/// `into_single_descriptors()` and derives from the receive (index 0) branch.
/// For single-branch `/0/*` or non-multipath descriptors: derives at definite
/// index 0 directly. Taproot descriptors are NOT supported and the caller
/// must reject them before calling (BIP-129 §1 prerequisites exclude BIP-386).
pub(crate) fn derive_first_address(
    descriptor: &MsDescriptor<DescriptorPublicKey>,
    network: bitcoin::Network,
) -> Result<String, ToolkitError>;
```

**Rationale for location:** shared module (not `wallet_export/bsms.rs`-private) because the import-side parser (`wallet_import/bsms.rs`) will consume the same helper for its first-address WARNING. New module `derive_address.rs` at crate root keeps the surface focused — exactly one public helper, no other concerns mixed in.

**API basis (miniscript v13):**

```rust
impl Descriptor<DescriptorPublicKey> {
    pub fn is_multipath(&self) -> bool;
    pub fn into_single_descriptors(self) -> Result<Vec<Self>, ConversionError>;
    pub fn at_derivation_index(&self, index: u32) -> Result<Descriptor<DefiniteDescriptorKey>, ...>;
}
impl Descriptor<DefiniteDescriptorKey> {
    pub fn address(&self, network: bitcoin::Network) -> Result<Address, ...>;
}
```

Pattern matches `wallet_export/bitcoin_core.rs:51-83` for `is_multipath()` + `into_single_descriptors()`, and `wallet_export/coldcard.rs:158-177` for derivation-from-xpub (singlesig). The new helper is the descriptor-level equivalent — operates on a parsed `Descriptor` (multi-cosigner-capable) rather than per-xpub.

## Path-restrictions emit rule (§3.5.1)

Implementation strategy: inspect the canonical_descriptor string directly. The toolkit canonicalizes to `<0;1>/*` multipath form via `wallet_export::pipeline::build_descriptor_string`, so the suffix pattern is uniform across all cosigners.

Pseudocode:

```rust
fn path_restrictions_line(canonical_descriptor: &str, parsed: &MsDescriptor<DescriptorPublicKey>) -> &'static str {
    if parsed.is_multipath() {
        // Common case: canonicalized <0;1>/* multipath
        if canonical_descriptor.contains("<0;1>/*") && !contains_divergent_multipath(canonical_descriptor) {
            return "/0/*,/1/*";
        }
        return "No path restrictions";
    }
    // Non-multipath: look for /0/* or /1/* across keys
    if canonical_descriptor.contains("/0/*") && !canonical_descriptor.contains("/1/*") {
        return "/0/*";
    }
    "No path restrictions"
}
```

The "divergent multipath" check would be a regex-based per-key suffix scan; we adopt a conservative interpretation: any `<N;M>` shape other than the canonical `<0;1>` → `No path restrictions`. For v0.27.0, the toolkit's canonical builder only emits `<0;1>/*`, so this check is primarily for descriptor-passthrough mode (user-supplied `--descriptor`).

## Taproot rejection

Detection: `WalletScriptType::P2tr` or `WalletScriptType::P2trMulti` per the existing `script_type_from_descriptor` helper at `wallet_export/mod.rs:182-219`. Surface in `BsmsEmitter::emit` via:

```rust
if matches!(inputs.script_type, WalletScriptType::P2tr | WalletScriptType::P2trMulti) {
    return Err(ToolkitError::BadInput(
        "--format bsms does not support taproot descriptors; BIP-129 §1 prerequisites \
         pre-date BIP-386. Use --format bitcoin-core or --format sparrow for taproot.".into()
    ));
}
```

## Form selection (`--bsms-form` flag)

Two-line vs four-line is controlled by a new CLI flag, not by an existing arg's value. Threading: `BsmsForm` enum on `ExportWalletArgs`, plumbed through `EmitInputs` (NEW field `bsms_form: BsmsForm`).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum BsmsForm {
    #[value(name = "2-line")] TwoLine,
    #[value(name = "4-line")] FourLine,
}
```

The new `EmitInputs` field is ignored by all 8 existing emitters (per the per-format ignored-input contract). Only `BsmsEmitter::emit` consumes it.

## FOLLOWUP closure

Phase 3 closes two FOLLOWUPS:

- `wallet-export-bsms-emitter` (FOLLOWUPS.md:2153) — closed by emitter impl.
- `bsms-first-address-verify` (FOLLOWUPS.md:2083) — closed by `derive_first_address` helper availability + import-side wire-up into `wallet_import/bsms.rs` (stderr WARNING on mismatch when 6-line audit fields present), per the FOLLOWUP body's exact "informational WARNING on mismatch" specification. SPEC §4.1 amendment removes the "deferred to v0.27+" framing.

## Test plan (8 cells)

Per plan §3.5:
1. `bsms_4line_emit_2of2_wsh_sortedmulti_mainnet`
2. `bsms_4line_emit_2of3_wsh_multi_testnet`
3. `bsms_4line_emit_sortedmulti_3of5`
4. `bsms_4line_path_restrictions_emits_slash_0_star_slash_1_star_for_multipath`
5. `bsms_4line_first_address_byte_exact_against_descriptor_derivation`
6. `bsms_4line_taproot_descriptor_errors_explicit_deferred`
7. `bsms_2line_lenient_excerpt_emits_descriptor_only`
8. `bsms_4line_then_import_byte_exact_idempotent`

Plus 1 cell exercising the new import-side WARNING (optional; can be folded into existing `cli_import_wallet_bsms.rs` or a new cell here).

## Phase 3 R0 dispatch scope (post-implementation)

Per plan §4.3 + memory `[[feedback-verify-bundle-round-trip-per-phase-r0-scope]]`:
- Path-restrictions rule correctness for multipath vs single-branch.
- First-address byte-exact match against independent miniscript-driven derivation.
- `tr()` rejection wins before any other validation.
- 4-line emit → v0.26.0 6-line lenient parser idempotent (4-line is a SUBSET of 6-line — actually it's a different shape; the test verifies emit→parse closes the round-trip for the supported 2-line path; per plan: "v0.27.0 ingest does NOT add 4-line parser — that's `bsms-bip129-full-cutover` FOLLOWUP for v0.28+").

Cycle proceeds to implementation.
