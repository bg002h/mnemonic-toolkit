# Plan-R0 (toolkit bitcoind end-to-end oracle) round 3 — architect review (verbatim)

> Persisted verbatim per CLAUDE.md. **Verdict: GREEN (0C/0I).** The plan-doc clears the R0
> gate; implementation (Phase 1 onward) may begin. Round-2's single Important (deprecated
> `at_derivation_index` + mishandled `into_single_descriptors()` Result) is fully folded —
> the plan now replicates STRESS-A's `derive_receive` helper verbatim (non-deprecated
> `derive_at_index` + `.unwrap()` Result handling). Verified against pinned fork `95fdd1c`.

---

## Verbatim agent output

## Verdict

**GREEN (0C/0I)**

### Round-2 Important — RESOLVED (verified against live source `95fdd1c` / `master@ffdf3d3`)

1. **Embedded helper exactly matches the live STRESS-A helper.** The plan's inline `derive_receive` is semantically line-for-line identical to `prop_backup_restore_roundtrip.rs:383-401` — same `from_str(desc).unwrap()`, `is_multipath()`, `into_single_descriptors().unwrap().remove(0)`, `has_wildcard()`, `derive_at_index(i).unwrap()`, `try_from(receive.clone()).unwrap()`, `address(bitcoin::Network::Bitcoin).unwrap().to_string()`. Only deltas are whitespace + a self-contained local `use`. No divergence that fails to compile or re-introduces a deprecated call.

2. **`derive_at_index` is non-deprecated.** Confirmed at `95fdd1c/src/descriptor/mod.rs:706` (returns `Result<Descriptor<DefiniteDescriptorKey>, NonDefiniteKeyError>`). The deprecated `at_derivation_index` carries `#[deprecated(since="13.0.0", note="use derive_at_index…")]` at `:726-729`. Live toolkit + STRESS-A use `derive_at_index` (`derive_address.rs:49,:96`; `prop_backup_restore_roundtrip.rs:394`; et al.). `cargo clippy --all-targets -- -D warnings` (`rust.yml:199`) → a replicated copy compiles clean. `into_single_descriptors` returns `Result<Vec<…>, Error>` (`mod.rs:946`), matching the `.unwrap()` handling.

3. **Imports resolve.** `miniscript::{DefiniteDescriptorKey, Descriptor, descriptor::DescriptorPublicKey}` resolve in `95fdd1c`; `bitcoin` (Cargo.toml:57) + `miniscript` (Cargo.toml:44) dev-reachable; `TryFrom` in prelude. `FromStr` comes from the verbatim md-codec module lift (`bitcoind_differential.rs:42` carries `use std::str::FromStr;`). Benign abstraction, not a defect.

4. **No residual deprecated/unhandled-Result occurrences.** The only `at_derivation_index` in the plan is the NEGATIVE warning reference (`:45`). Every `into_single_descriptors()` is `.unwrap()`-handled (`:17`, `:50`).

5. **Prior-resolved items hold, nothing regressed.** I-1 (shape-6 default-suite cell, §3 + Phase 2a — now using the same non-deprecated `derive_receive`), I-2 (independent golden architecture), M-1 (verbatim origins), M-2 (GAP-4a sequencing), M-3 (`cross-tool-differential.yml` @v5 model), M-4 (`getdescriptorinfo` checksum agreement) all intact. The fold was surgically scoped to §2 prose + §3 helper/golden text exactly as round-2 prescribed.

No Critical, no Important, no new defect. This plan-doc clears the R0 gate; implementation may begin.
