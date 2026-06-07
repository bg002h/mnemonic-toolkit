# SPEC — code-hygiene: error.rs Bip388Distinctness comment + CosignerKeyInfo vestigial #[allow(dead_code)]

**FOLLOWUPs:** `error-rs-bip388-distinctness-stale-raw-string-comment` + `synthesize-descriptor-vestigial-dead-code-allow` (the latter STRUCTURALLY-WRONG as filed → re-scoped to CosignerKeyInfo, re-titled `cosigner-key-info-vestigial-dead-code-allow`).
**Source SHA:** `3ea612a`.
**Recon:** `cycle-prep-recon-cycle2-residual-followups.md` (slug 2 ACCURATE; slug 3 STRUCTURALLY-WRONG → re-scoped).
**Cycle type:** source comment + attribute cleanup. **No behavior, no public-API shape, no CLI surface change** → no GUI `schema_mirror`, no manual mirror, no sibling-codec. SemVer: **R0 to rule no-bump-commit vs PATCH+tag** (a non-behavioral `src/*.rs` change; SPEC leans no-bump, consistent with the session's docs/no-observable-change precedent).

---

## 1. Changes

### 1a. `crates/mnemonic-toolkit/src/error.rs:13-16` — reword the stale comment
The `Bip388Distinctness` variant doc-comment reads:
```
/// `i` and `j` are the colliding slot indices (i < j) under
/// `(xpub, derivation_path_string)` raw-string equality per §4.11.b
/// normalization domain.
```
This is stale: both distinctness layers compare TYPED `DerivationPath` (`cmd::bundle::check_resolved_slots_distinctness` `bundle.rs:429/433` `slots[i].path == slots[j].path`; `parse_descriptor::check_key_vector_distinctness` `parse_descriptor.rs:1208/1212` `cs[i].path == cs[j].path`) since v0.37.9/v0.5 (`SPEC_path_raw_bracketed_bare_unification.md` A2). Reword to **match the already-resynced twin at `bundle.rs:423-428`** (R0 Minor — the xpub leg uses `.to_string()`, only the path leg is typed):
```
/// `i` and `j` are the colliding slot indices (i < j) under
/// `(xpub.to_string(), path)` typed-`DerivationPath` equality per §4.11.b
/// (`h`/`'`-notation folds; mirrors the `cmd::bundle` twin comment).
```
Comment-only; no behavior change. (The `bundle.rs:423-428` comment was already resynced; `error.rs` is the lone residual lag.)

### 1b. `crates/mnemonic-toolkit/src/synthesize.rs:218` — remove the vestigial `#[allow(dead_code)]` on `CosignerKeyInfo`
The `#[allow(dead_code)]` at `:218` is on `pub type CosignerKeyInfo = ResolvedSlot;` (`:219`) — **NOT** on `synthesize_descriptor` (`:229`, which has no such attribute). `CosignerKeyInfo` is genuinely USED (production: `synthesize_descriptor`'s signature `cosigners: &[CosignerKeyInfo]` at `:231`, a live fn called by `synthesize_unified` at `:826`; plus test sites), so the `#[allow(dead_code)]` is vestigial. Remove it.

**Verification gate (this IS the test for 1b):** after removal, `cargo build -p mnemonic-toolkit` + `cargo clippy -p mnemonic-toolkit --all-targets` must be **warning-clean**. A `dead_code` warning would prove the allow was NOT vestigial → revert + close the FOLLOWUP instead. (No new test cell needed — the build-clean check is the discriminating verification.)

### 1c. FOLLOWUP bookkeeping (Phase 2 ship)
- `error-rs-bip388-distinctness-stale-raw-string-comment` → resolved.
- `synthesize-descriptor-vestigial-dead-code-allow` → resolved (re-titled note: the cited `:218` allow was on `CosignerKeyInfo`, not synthesize_descriptor; synthesize_descriptor has no allow; the cycle-2 chapter was already correct — no doc fix needed; removed the vestigial `CosignerKeyInfo` allow instead).

## 2. Verification
- `cargo build -p mnemonic-toolkit` + `cargo clippy -p mnemonic-toolkit --all-targets` → 0 warnings (proves 1b's allow vestigial).
- `cargo test -p mnemonic-toolkit --no-fail-fast` → 0 failed (the comment + attribute change is non-behavioral; full suite confirms no regression).

## 3. Phasing
- **Phase 1 (implement):** 1a + 1b. Run §2. (No RED phase — 1a is a comment; 1b's "RED-equivalent" is: the build would warn if the allow weren't vestigial.)
- **Phase 2 (review + ship):** per-phase opus review → 0C/0I → flip the 2 FOLLOWUPs (1c) → ship per the R0 SemVer ruling (no-bump ff-merge to `master`, OR PATCH v0.47.4→v0.47.5 + tag).

## 4. R0 must decide / confirm
1. **SemVer:** no-bump-commit vs PATCH+tag for a non-behavioral `src/*.rs` comment + `#[allow]` removal. SPEC leans no-bump.
2. **CosignerKeyInfo allow is genuinely vestigial** (removable warning-clean) — confirm `CosignerKeyInfo` is used in non-test production code (`synthesize.rs:231`) so removal won't fire `dead_code`. (If R0 suspects it WILL warn, the fallback is: keep the allow, close the FOLLOWUP as "not vestigial".)
3. **The error.rs reword introduces no new falsehood** (both layers are typed — confirmed in the api-harvest cycle).
4. **No other consumer** of the `Bip388Distinctness` doc-comment / `CosignerKeyInfo` allow that this breaks.

## 5. Out of scope
- `technical-manual-residual-line-ref-drift` (Cycle B).
- Any behavior/API/CLI change.
