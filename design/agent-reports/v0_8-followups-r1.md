# v0.8.2 Followups R1 — reviewer report

## Verdict
**1C / 0I — fold required** (folded in this turn; see commit hash below).

## Findings

### C-1 — Three test-only `CosignerKeyInfo` struct literals missing `master_xpub`

**Confidence:** 100 (compile-time enforcement).

**Files:**
- `crates/mnemonic-toolkit/src/parse_descriptor.rs:1713` (`cinfo()` test helper)
- `crates/mnemonic-toolkit/src/parse_descriptor.rs:1726` (`cinfo_raw()` test helper)
- `crates/mnemonic-toolkit/src/synthesize.rs:1026` (`descriptor_fixture()` test helper)

**What:** Commit `aef7f17` added `master_xpub: Option<Xpub>` to `ResolvedSlot` and updated all six PRODUCTION construction sites correctly. Three test-only `CosignerKeyInfo` (= `ResolvedSlot` type alias) struct literals inside `#[cfg(test)]` modules were missed. `cargo test -p mnemonic-toolkit --tests` (run during the follow-up cycle) only compiles integration tests under `tests/`, not unit tests inside `src/` — so the gap went undetected. `cargo test -p mnemonic-toolkit` (no `--tests`) fails with three E0063 missing-field errors.

**Fix:** Added `master_xpub: None,` after `entropy: None,` at each of the three sites. Full `cargo test -p mnemonic-toolkit` now green; zero failures across unit + integration surface.

## Confidence-filtered: omitted

- **6 production ResolvedSlot construction sites** all correctly populated (Xpub arm: `Some(parsed)` from user input; all others: `None`).
- **SLIP-132 normalization on master_xpub input** harmless — swaps version bytes only; re-emitted as BIP-32 neutral via `.to_string()`.
- **EmitInputs.master_xpub_at_0 = first().and_then(|s| s.master_xpub)** correct per SPEC §5.1 (only `@0.master_xpub=` is consumed).
- **`#[serde(skip_serializing_if = "Option::is_none")]`** correctly omits absent-case xpub field; 3 existing fixtures byte-identical.
- **BIP-32 spec test vector 1 master xpub** in fixture is a cross-seed placeholder, documented in test comment; emitter does no cross-validation contract.
- **Refuse-on-supply guard retirement** clean (no orphan imports / comments).
- **Test coverage** cell_8 + cell_9 adequate for two-state conditional.
- **Electrum doc-comment citations** verified against actual `wallet_db.py:1207` line.
- **Spike report integrity** matches actual source line ranges.
- **FOLLOWUPS refresh** internally consistent; Jade singlesig BIP-86 claim accurate.

---

**Fold commit:** `e2fffdc fix(v0.8.2 followup R1): fold C-1 — add master_xpub: None to 3 test-only CosignerKeyInfo struct literals` (commit hash filled at commit time)
