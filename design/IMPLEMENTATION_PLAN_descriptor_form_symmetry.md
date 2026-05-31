# Descriptor-Form Symmetry (A1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make a *bare concrete* descriptor (`wsh(sortedmulti(2,[fp/84h/0h/0h]xpub…/<0;1>/*,…))`, what real wallets emit) accept on `bundle --descriptor` and `verify-bundle --descriptor` (today `@N`-only), and give `export-wallet --descriptor` a clear redirect on the keyless `@N` form — so all three descriptor surfaces are consistent.

**Architecture:** A `pub(crate)` probe classifier (`classify_descriptor_form`) + a `descriptor_concrete_to_resolved_slots` helper (both in `wallet_import/pipeline.rs`) reuse the existing `concrete_keys_to_placeholders` converter. `bundle::run` early-forks (mirroring `--import-json`) to a new `bundle_run_concrete_descriptor`; `verify-bundle` forks before `lex_placeholders`; `export-wallet` guards with an `@N`-only probe. One regex (`key_regex`) is widened for `h`-form hardened paths. No new flag → PATCH v0.38.1, no GUI lockstep.

**Tech Stack:** Rust (edition 2021), `regex`, `miniscript 0.32`, `bitcoin 0.32`, `md_codec`, `assert_cmd` for integration tests.

**SPEC:** `design/SPEC_descriptor_form_symmetry.md` (R0 gate GREEN, R3 0C/0I). **Source SHA at plan-write:** `ea8ba88`.

**Branch:** `theme-a-descriptor-form-symmetry` (already exists; design artifacts committed). Run `cargo build` / `cargo test` from the repo root.

---

## File Structure

| File | Responsibility | Change |
|---|---|---|
| `crates/mnemonic-toolkit/src/wallet_import/pipeline.rs` | the converter; **NEW** `key_regex` widening, `AT_N_PROBE`, `classify_descriptor_form`, `descriptor_concrete_to_resolved_slots` | modify + add (`pub(crate)`) |
| `crates/mnemonic-toolkit/src/cmd/bundle.rs` | `run` early-fork + **NEW** `bundle_run_concrete_descriptor` | modify + add |
| `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` | Concrete fork before `lex_placeholders` | modify |
| `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` | `@N`-probe redirect guard | modify |
| `crates/mnemonic-toolkit/tests/cli_descriptor_concrete.rs` | **NEW** integration tests (bundle/verify/export concrete) | create |
| `crates/mnemonic-toolkit/tests/cli_wallet_cross_format_convergence.rs` | convergence + distinctness cells | modify |
| `docs/manual/src/40-cli-reference/41-mnemonic.md` | 3 `--descriptor` prose blocks | modify |
| `crates/mnemonic-toolkit/Cargo.toml` + `CHANGELOG`/version sites | v0.38.0 → v0.38.1 | modify |

**Shared test fixtures** (real, valid testnet tpubs lifted from the existing `pipeline.rs` test `two_keys_preserve_declaration_order`):

```rust
// 2-of-2 testnet multisig, apostrophe-hardened, explicit BIP-48 origins.
const CONCRETE_MULTI_APOS: &str = "wsh(sortedmulti(2,[704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*,[97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*))";
// Same descriptor, h-form hardened paths (Core/Sparrow style).
const CONCRETE_MULTI_HFORM: &str = "wsh(sortedmulti(2,[704c7836/48h/1h/3h/2h]tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*,[97139860/48h/1h/2h/2h]tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*))";
```

---

## Task 0 (Phase 0): Widen `key_regex` for `h`-form hardened paths

**Files:**
- Modify: `crates/mnemonic-toolkit/src/wallet_import/pipeline.rs:38`
- Test: `crates/mnemonic-toolkit/src/wallet_import/pipeline.rs` (inline `#[cfg(test)] mod tests`)

- [ ] **Step 1: Write the failing test** — append to the inline `mod tests`:

```rust
    #[test]
    fn hform_hardened_paths_accepted() {
        // Core/Sparrow emit `h`-form (`/48h/1h/...`); the converter must
        // accept it identically to apostrophe form.
        let hform = "wsh(sortedmulti(2,[704c7836/48h/1h/3h/2h]tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*,[97139860/48h/1h/2h/2h]tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*))";
        let (placeholder, keys, fps) = concrete_keys_to_placeholders(hform).unwrap();
        assert_eq!(keys.len(), 2);
        assert_eq!(fps[0].fp, [0x70, 0x4c, 0x78, 0x36]);
        // The h-form path string is preserved verbatim into the @N form.
        assert!(placeholder.contains("@0[704c7836/48h/1h/3h/2h]/<0;1>/*"), "{placeholder}");
    }
```

- [ ] **Step 2: Run it — verify it FAILS**

Run: `cargo test -p mnemonic-toolkit --lib hform_hardened_paths_accepted`
Expected: FAIL — `concrete_keys_to_placeholders` returns `Err(ImportWalletParse("…no [fp/path]xpub keys found…"))` because the current `key_regex` path group `(?:/\d+'?)+` does not match `h`.

- [ ] **Step 3: Widen the regex** — `pipeline.rs:38`, change ONLY the path group `(?:/\d+'?)+` → `(?:/\d+(?:'|h)?)+`:

```rust
        Regex::new(r"\[([0-9a-fA-F]{8})((?:/\d+(?:'|h)?)+)\]([xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+)")
            .expect("key_regex is a fixed string literal")
```

(Do NOT touch the import parsers' `origin_capture_regex` copies in `bsms.rs:516`, `specter.rs`, `sparrow.rs`, `bitcoin_core.rs` — the new path never calls them; filed as FOLLOWUP `import-parser-hform-origin-tolerance`.)

- [ ] **Step 4: Run — verify PASS + no regression**

Run: `cargo test -p mnemonic-toolkit --lib pipeline::` then `cargo test -p mnemonic-toolkit --test cli_import_wallet` (or `cargo test -p mnemonic-toolkit import` if the import test file name differs — run `ls crates/mnemonic-toolkit/tests | grep import` first).
Expected: PASS — `hform_hardened_paths_accepted` + the existing `two_keys_preserve_declaration_order` / `no_keys_errors` + all import-wallet integration tests green (widening is a strict superset).

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/wallet_import/pipeline.rs
git commit -m "feat(descriptor): widen key_regex to accept h-form hardened paths (A1 P0)"
```

---

## Task 1 (Phase 1): `classify_descriptor_form` + `AT_N_PROBE`

**Files:**
- Modify: `crates/mnemonic-toolkit/src/wallet_import/pipeline.rs` (add near `key_regex`)
- Test: inline `mod tests`

- [ ] **Step 1: Write the failing tests** — append to `mod tests`:

```rust
    #[test]
    fn classify_atn_concrete_mixed_garbage() {
        // @N template → AtN.
        assert_eq!(
            classify_descriptor_form("wsh(sortedmulti(2,@0[704c7836/48'/1'/3'/2']/<0;1>/*,@1[97139860/48'/1'/2'/2']/<0;1>/*))").unwrap(),
            DescriptorForm::AtN
        );
        // bare concrete → Concrete.
        assert_eq!(
            classify_descriptor_form("wpkh([704c7836/84'/0'/0']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/0/*)").unwrap(),
            DescriptorForm::Concrete
        );
        // mixed @N + inline xpub → error (rule 1).
        let mixed = "wsh(sortedmulti(2,@0[704c7836/48'/1'/3'/2']/<0;1>/*,[97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*))";
        assert!(classify_descriptor_form(mixed).unwrap_err().message().contains("mixes @N"));
        // origin-less / keyless → rule-4 origin-required error.
        let err = classify_descriptor_form("wpkh(0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798)").unwrap_err();
        assert!(err.message().contains("must carry a key origin"), "{}", err.message());
    }
```

- [ ] **Step 2: Run — verify it FAILS**

Run: `cargo test -p mnemonic-toolkit --lib classify_atn_concrete_mixed_garbage`
Expected: FAIL — `classify_descriptor_form` / `DescriptorForm` not defined.

- [ ] **Step 3: Implement** — add to `pipeline.rs` (after `key_regex`, `pub(crate)` so `cmd::*` reach it):

```rust
/// Cheap `@\d`-presence probe (the toolkit's `@N` placeholder form). NEW.
fn at_n_probe() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"@\d").expect("AT_N_PROBE literal"))
}

/// Which descriptor form a user string is. Discriminant only — no payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DescriptorForm {
    /// `@N`-placeholder template (keys sourced per-surface).
    AtN,
    /// Bare-concrete form with inline `[fp/path]xpub` keys.
    Concrete,
}

/// Classify a descriptor string via cheap probes. Pure; no conversion.
/// Rule 1: both probes → mixed error. 2: `@\d` only → AtN. 3: key_regex
/// only → Concrete. 4: neither → origin-required error (md-codec is NOT
/// reached on this branch, so the error originates here — SPEC §3.1).
pub(crate) fn classify_descriptor_form(input: &str) -> Result<DescriptorForm, ToolkitError> {
    let has_at_n = at_n_probe().is_match(input);
    let has_concrete = key_regex().is_match(input);
    match (has_at_n, has_concrete) {
        (true, true) => Err(ToolkitError::DescriptorParse(
            "descriptor mixes @N placeholders with inline keys; use one form".into(),
        )),
        (true, false) => Ok(DescriptorForm::AtN),
        (false, true) => Ok(DescriptorForm::Concrete),
        (false, false) => Err(ToolkitError::DescriptorParse(
            "descriptor has neither @N placeholders nor [fp/path]-annotated keys; \
             concrete descriptors must carry a key origin, e.g. [<fp>/84h/0h/0h]xpub…"
                .into(),
        )),
    }
}
```

- [ ] **Step 4: Run — verify PASS**

Run: `cargo test -p mnemonic-toolkit --lib classify_atn_concrete_mixed_garbage`
Expected: PASS. (Confirm `ToolkitError::DescriptorParse(String)` is the correct variant: `grep -n 'DescriptorParse' crates/mnemonic-toolkit/src/error.rs` — it is a tuple variant carrying `String`, and `ToolkitError::message()` returns the inner text.)

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/wallet_import/pipeline.rs
git commit -m "feat(descriptor): classify_descriptor_form + AT_N_PROBE probe classifier (A1 P1)"
```

---

## Task 2 (Phase 2): `descriptor_concrete_to_resolved_slots` helper

Recovers the full `Xpub` + `DerivationPath` per cosigner (the `[u8;65]` `ParsedKey` payload is lossy) by re-scanning the body with the widened `key_regex`, mirroring `bsms.rs:219-265` + `build_slot_fields` (`bsms.rs:399-416`).

**Files:**
- Modify: `crates/mnemonic-toolkit/src/wallet_import/pipeline.rs` (add helper + imports)
- Test: inline `mod tests`

- [ ] **Step 1: Write the failing test:**

```rust
    #[test]
    fn concrete_to_resolved_slots_recovers_typed_fields() {
        let body = "wsh(sortedmulti(2,[704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*,[97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*))";
        let (_descriptor, slots) = descriptor_concrete_to_resolved_slots(body).unwrap();
        assert_eq!(slots.len(), 2);
        // typed fp + path recovered from the original base58, in declaration order.
        assert_eq!(slots[0].fingerprint, bitcoin::bip32::Fingerprint::from([0x70, 0x4c, 0x78, 0x36]));
        assert_eq!(slots[0].path, bitcoin::bip32::DerivationPath::from_str("m/48'/1'/3'/2'").unwrap());
        assert_eq!(slots[1].fingerprint, bitcoin::bip32::Fingerprint::from([0x97, 0x13, 0x98, 0x60]));
        // watch-only: no entropy on any slot.
        assert!(slots.iter().all(|s| s.entropy.is_none()));
    }

    #[test]
    fn concrete_helper_error_drops_bsms_prefix() {
        // A descriptor with no keys → neutral DescriptorParse, NOT "import-wallet: bsms:".
        let err = descriptor_concrete_to_resolved_slots("wsh(thresh(2,older(144),older(288)))").unwrap_err();
        assert!(!err.message().contains("bsms"), "leaked converter prefix: {}", err.message());
    }
```

- [ ] **Step 2: Run — verify FAIL**

Run: `cargo test -p mnemonic-toolkit --lib concrete_to_resolved_slots`
Expected: FAIL — helper not defined.

- [ ] **Step 3: Implement** — add ONLY the genuinely-new imports at the top of `pipeline.rs`. [R0-I1 — `pipeline.rs` ALREADY has `use std::str::FromStr;` (:21), `use bitcoin::bip32::Xpub;` (:19), `use crate::slip0132::normalize_xpub_prefix;` (:18); re-importing them is E0252/E0254. Verify the existing `use` block first.]

```rust
use bitcoin::bip32::{DerivationPath, Fingerprint};   // Xpub already imported at :19
use crate::synthesize::{ResolvedSlot, xpub_to_65};
use crate::parse_descriptor::parse_descriptor;
// std::str::FromStr (:21) + crate::slip0132::normalize_xpub_prefix (:18) already present.
```

Then the helper:

```rust
/// Bare-concrete (checksum-stripped) descriptor body → (parsed md_codec
/// Descriptor, watch-only ResolvedSlots). Mirrors bsms.rs:219-265; recovers
/// the full Xpub + path from the original base58 (the ParsedKey [u8;65]
/// payload is lossy). SPEC §3.2.
pub(crate) fn descriptor_concrete_to_resolved_slots(
    body: &str,
) -> Result<(md_codec::Descriptor, Vec<ResolvedSlot>), ToolkitError> {
    // Remap the converter's hard-coded "import-wallet: bsms:" prefix to a
    // neutral DescriptorParse (SPEC §3.3 — the calling command is bundle/
    // verify-bundle, not import-wallet).
    let (placeholder_form, keys, fps) = concrete_keys_to_placeholders(body)
        .map_err(|e| ToolkitError::DescriptorParse(
            e.message().replace("import-wallet: bsms: parse error: ", ""),
        ))?;
    let descriptor = parse_descriptor(&placeholder_form, &keys, &fps)
        .map_err(|e| ToolkitError::DescriptorParse(e.message()))?;

    // Recover (fp, path, xpub) per key in one pass over `body` with the
    // widened key_regex (group1=fp, group2=path, group3=xpub). Same iterator
    // order/count as concrete_keys_to_placeholders → slot i ↔ @i ↔ keys[i].
    let mut slots: Vec<ResolvedSlot> = Vec::with_capacity(keys.len());
    for (idx, cap) in key_regex().captures_iter(body).enumerate() {
        let fp_hex = cap.get(1).expect("group 1").as_str();
        let path_inner = cap.get(2).expect("group 2").as_str();
        let xpub_str = cap.get(3).expect("group 3").as_str();
        let mut fp_bytes = [0u8; 4];
        for b in 0..4 {
            fp_bytes[b] = u8::from_str_radix(&fp_hex[b * 2..b * 2 + 2], 16)
                .map_err(|e| ToolkitError::DescriptorParse(format!("fingerprint hex: {e}")))?;
        }
        let path = DerivationPath::from_str(&format!("m{path_inner}"))
            .map_err(|e| ToolkitError::DescriptorParse(format!("derivation path: {e}")))?;
        let (neutral, _variant) = normalize_xpub_prefix(xpub_str)?;
        let xpub = Xpub::from_str(&neutral)
            .map_err(|e| ToolkitError::DescriptorParse(format!("xpub decode: {e}")))?;
        debug_assert_eq!(xpub_to_65(&xpub), keys[idx].payload);
        slots.push(ResolvedSlot {
            xpub,
            fingerprint: Fingerprint::from(fp_bytes),
            path,
            entropy: None,
            master_xpub: None,
            _entropy_pin: None,
        });
    }
    Ok((descriptor, slots))
}
```

- [ ] **Step 4: Run — verify PASS**

Run: `cargo test -p mnemonic-toolkit --lib concrete_to_resolved_slots concrete_helper_error_drops_bsms_prefix`
Expected: PASS. If `ResolvedSlot` has additional fields beyond `{xpub, fingerprint, path, entropy, master_xpub, _entropy_pin}`, the build fails — re-check `synthesize.rs:616-644` and add any missing field with its watch-only default (`None`). Confirm `normalize_xpub_prefix` returns `Result<(String, _), ToolkitError>` (it does — `slip0132.rs`).

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/wallet_import/pipeline.rs
git commit -m "feat(descriptor): descriptor_concrete_to_resolved_slots helper (A1 P2)"
```

---

## Task 3 (Phase 3): Wire `bundle` + `verify-bundle`

### 3a — `bundle::run` early-fork + `bundle_run_concrete_descriptor`

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/bundle.rs` (`run` ~`:223-227`; add new fn)
- Test: `crates/mnemonic-toolkit/tests/cli_descriptor_concrete.rs` (create)

- [ ] **Step 1: Write the failing integration test** — create the file:

```rust
//! A1 — bare-concrete descriptor acceptance on bundle/verify/export.
use assert_cmd::Command;

const CONCRETE_MULTI_APOS: &str = "wsh(sortedmulti(2,[704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*,[97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*))";

fn mnemonic() -> Command { Command::cargo_bin("mnemonic").unwrap() }

#[test]
fn bundle_concrete_descriptor_produces_watch_only_cards() {
    let out = mnemonic()
        .args(["bundle", "--descriptor", CONCRETE_MULTI_APOS, "--network", "testnet", "--json"])
        .output().unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    // Real BundleJson wire-shape (format.rs:119-145): md1 = Vec<String>,
    // ms1 = length-N array with "" sentinels for watch-only, mode = "watch-only".
    assert_eq!(v["mode"], "watch-only", "{v}");
    assert!(v["md1"].as_array().map_or(false, |a| !a.is_empty()), "md1 array: {v}");
    assert!(
        v["ms1"].as_array().unwrap().iter().all(|s| s == ""),
        "watch-only ms1 must be all empty-string sentinels: {v}"
    );
}
```

(`BundleJson` shape confirmed at `format.rs:119-145`: `md1: Vec<String>`, `mk1: MkField` untagged (multisig = array-of-arrays), `ms1: MsField` length-N array with `""` watch-only sentinels, `mode: "full"|"watch-only"`. Do NOT use `.as_str()` on `md1`/`mk1` — they are arrays.)

- [ ] **Step 2: Run — verify FAIL**

Run: `cargo test -p mnemonic-toolkit --test cli_descriptor_concrete bundle_concrete_descriptor_produces_watch_only_cards`
Expected: FAIL — current `bundle --descriptor` rejects bare concrete (`descriptor must contain at least one @N placeholder`) OR dies at the empty-slot gate.

- [ ] **Step 3: Implement the early-fork** — in `bundle.rs run`, **AFTER the descriptor-mode mode-violation guards** (`bundle.rs:235-279`: `DESCRIPTOR_AND_TEMPLATE`, `DESCRIPTOR_AND_DESCRIPTOR_FILE`, `--threshold`/`--multisig-path-family` mutexes), immediately before the `bundle_run_unified(...)` dispatch. [R0-I3 — `--descriptor` has NO clap `conflicts_with` for `--template`; the mutexes are code-level at `:235-279`, so forking at `:227` would let a concrete `--descriptor --template` silently ignore `--template`. The `@N` path errors at those guards, so the Concrete path must run them first.] Locate the `bundle_run_unified(args, ...)` call that follows the guard block and insert just above it:

```rust
    if descriptor_mode {
        // Read the descriptor body here (the `:1056-1064` read lives inside
        // bundle_run_unified_descriptor, off the Concrete early-fork path).
        let body = match (&args.descriptor, &args.descriptor_file) {
            (Some(s), None) => s.clone(),
            (None, Some(p)) => std::fs::read_to_string(p)
                .map_err(|e| ToolkitError::DescriptorParse(format!("--descriptor-file {}: {e}", p.display())))?
                .trim_end()
                .to_string(),
            _ => unreachable!("DESCRIPTOR_AND_DESCRIPTOR_FILE guard above rules out both"),
        };
        use crate::wallet_import::pipeline::{classify_descriptor_form, DescriptorForm};
        if classify_descriptor_form(&body)? == DescriptorForm::Concrete {
            return bundle_run_concrete_descriptor(&args, body, stdout, stderr);
        }
        // AtN: fall through to bundle_run_unified (re-reads the file as today).
    }
```

Then add the new function (place near `bundle_run_from_import_json`; note `args: &BundleArgs` by-ref, mirroring `bundle_run_from_import_json` at `bundle.rs:1491` — `run`'s `args` is `&BundleArgs`):

```rust
fn bundle_run_concrete_descriptor<W: Write, E: Write>(
    args: &BundleArgs,
    body: String,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    use crate::wallet_import::pipeline::descriptor_concrete_to_resolved_slots;
    // Strip the BIP-380 checksum (the @N rewrite invalidates it).
    let body_no_csum = crate::wallet_import::json_envelope::descriptor_body_no_csum(&body, "--descriptor")?;
    let (descriptor, resolved_slots) = descriptor_concrete_to_resolved_slots(body_no_csum)?;

    // BIP-388 distinctness — the from_import_json tail omits it (trusted
    // cards) but a pasted descriptor is untrusted; mirror the @N/template
    // paths (bundle.rs:367 / :1407).
    check_resolved_slots_distinctness(&resolved_slots)?;

    let bundle = synthesize_descriptor(&descriptor, &resolved_slots, args.privacy_preserving)?;
    let n = resolved_slots.len();
    let any_secret = resolved_slots.iter().any(|s| s.entropy.is_some()); // always false here
    let any_watch = resolved_slots.iter().any(|s| s.entropy.is_none());
    let mode = match (n, any_secret, any_watch) {
        (1, true, _) => BundleMode::SingleSigFull,
        (1, false, _) => BundleMode::SingleSigWatchOnly,
        (_, true, true) => BundleMode::MultisigHybrid,
        (_, true, false) => BundleMode::MultisigMultiSource,
        (_, false, _) => BundleMode::MultisigWatchOnly,
    };

    // Emit: the real descriptor is already in args.descriptor/_file, so
    // emit_unified's descriptor_field picks it up — NO synthetic injection
    // (unlike from_import_json's :1680-1681).
    emit_unified(args, &bundle, &resolved_slots, mode, &[], stdout, stderr)?;
    if args.self_check {
        self_check_bundle(&bundle, args)?;
    }
    Ok(())
}
```

- [ ] **Step 4: Run — verify PASS**

Run: `cargo test -p mnemonic-toolkit --test cli_descriptor_concrete bundle_concrete_descriptor_produces_watch_only_cards`
Expected: PASS. (Confirm `descriptor_body_no_csum` is `pub(crate)` and reachable as `crate::wallet_import::json_envelope::descriptor_body_no_csum`; check its arity via `grep -n 'fn descriptor_body_no_csum' crates/mnemonic-toolkit/src/wallet_import/json_envelope.rs` and match the 2nd arg label. Confirm `self_check_bundle`'s signature — `grep -n 'fn self_check_bundle' bundle.rs` — and match `&args` vs `args`.)

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/cmd/bundle.rs crates/mnemonic-toolkit/tests/cli_descriptor_concrete.rs
git commit -m "feat(bundle): accept bare-concrete --descriptor (watch-only cards) (A1 P3a)"
```

### 3b — `verify-bundle` Concrete fork

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (after `descriptor_str` read ~`:611`, before `lex_placeholders` `:614`)
- Test: `crates/mnemonic-toolkit/tests/cli_descriptor_concrete.rs`

- [ ] **Step 1: Write the failing test** — append. **Card extraction is shape-sensitive** (`md1: Vec<String>`, multisig `mk1` = array-of-arrays), so MIRROR the produce→extract→verify pattern from an existing passing test rather than hand-navigating JSON: read `tests/cli_verify_bundle_full.rs` (and `cli_verify_bundle_multi_cosigner_mk1.rs`) for the exact helper that turns a `bundle --json` envelope into the `--md1`/`--mk1` flag vector (`--md1`/`--mk1` are `Vec<String>`, `num_args = 1..`, `verify_bundle.rs:82/:89`). Then:

```rust
#[test]
fn verify_bundle_concrete_matches_self_produced_cards() {
    // Produce a bundle from the concrete descriptor, then verify the SAME
    // descriptor against those cards → exit 0. `flags_from_bundle_json` is
    // the helper mirrored from cli_verify_bundle_multi_cosigner_mk1.rs that
    // expands md1 (Vec<String>) + mk1 (array-of-arrays) into --md1/--mk1 args.
    let produced = mnemonic()
        .args(["bundle", "--descriptor", CONCRETE_MULTI_APOS, "--network", "testnet", "--json"])
        .output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&produced.stdout).unwrap();
    let mut args: Vec<String> =
        vec!["verify-bundle".into(), "--descriptor".into(), CONCRETE_MULTI_APOS.into(),
             "--network".into(), "testnet".into()];
    args.extend(flags_from_bundle_json(&v)); // pushes --md1 <chunk>… --mk1 <chunk>…
    let out = mnemonic().args(&args).output().unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
}
```

If no reusable helper exists in those files, write `flags_from_bundle_json` to expand `v["md1"].as_array()` (each → `--md1 <chunk>`) and `v["mk1"].as_array()` (each cosigner's chunk array → `--mk1 <chunk>`), per the real `MkField` multisig shape.

- [ ] **Step 2: Run — verify FAIL**

Run: `cargo test -p mnemonic-toolkit --test cli_descriptor_concrete verify_bundle_concrete_matches_self_produced_cards`
Expected: FAIL — verify-bundle's `lex_placeholders` rejects bare concrete.

- [ ] **Step 3: Implement** — three parts.

**(a) Extract the reusable emit tail.** The existing `descriptor_mode_verify_run` (`verify_bundle.rs:589`) tail at `:864-902` is partly `@N`-specific. Factor ONLY the form-agnostic part (`:867` synthesize + `:871-902` `SuppliedCards`/`emit_verify_checks`/output/`Ok(if any_fail {4} else {0})`) into a helper. **Do NOT include `:856` (`parse_descriptor(&descriptor_str,&keys,&fingerprints)` — re-parses the `@N` form, would FAIL on concrete) or `:864-866` (`if is_non_canonical { descriptor.path_decl = descriptor_resolved… }` — `descriptor_resolved` exists only on the `@N` path).**

```rust
fn verify_emit_from_expected<W: Write, E: Write>(
    args: &VerifyBundleArgs,
    descriptor: md_codec::Descriptor,
    cosigners: &[crate::synthesize::ResolvedSlot],
    no_auto_repair: bool,
    json_context: bool,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let expected = synthesize_descriptor(&descriptor, cosigners, args.privacy_preserving)?;
    let supplied = SuppliedCards { ms1: &args.ms1, mk1: &args.mk1, md1: &args.md1 };
    let checks = emit_verify_checks(&expected, &supplied, descriptor.n > 1, no_auto_repair, json_context, stdout, stderr)?;
    let any_fail = checks.iter().any(|c| !c.passed);
    let result_str = if any_fail { "mismatch" } else { "ok" };
    if args.json {
        let json = crate::format::VerifyBundleJson { schema_version: "4", result: result_str, checks };
        serde_json::to_writer(&mut *stdout, &json).ok();
        writeln!(stdout).ok();
    } else {
        for c in &checks {
            let status = if c.passed { "ok" } else { "fail" };
            if c.detail.is_empty() { writeln!(stdout, "{}: {}", c.name, status).ok(); }
            else { writeln!(stdout, "{}: {} {}", c.name, status, c.detail).ok(); }
        }
        writeln!(stdout, "result: {}", result_str).ok();
    }
    Ok(if any_fail { 4 } else { 0 })
}
```

Then in the `@N` path, replace its inline `:867-902` block with `return verify_emit_from_expected(args, descriptor, &cosigners, no_auto_repair, json_context, stdout, stderr);` (after its existing `:864-866` path_decl mutation). Confirm `cosigners`/`keys`/`fingerprints` variable names against `:850-866` and that `cosigners: Vec<CosignerKeyInfo>` (= `Vec<ResolvedSlot>`).

**(b) Insert the Concrete fork** right after `descriptor_str` is materialized (`:603-611`) and before `let occs = lex_placeholders(...)` (`:614`):

```rust
    use crate::wallet_import::pipeline::{classify_descriptor_form, descriptor_concrete_to_resolved_slots, DescriptorForm};
    if classify_descriptor_form(&descriptor_str)? == DescriptorForm::Concrete {
        let body_no_csum = crate::wallet_import::json_envelope::descriptor_body_no_csum(&descriptor_str, "--descriptor")?;
        let (descriptor, cosigners) = descriptor_concrete_to_resolved_slots(body_no_csum)?;
        // Verify-flavored distinctness (exit-4), mirroring the @N path's
        // re-wrap at verify_bundle.rs:852-854. Explicit-origin concrete needs
        // NO path_decl mutation (the helper's descriptor is already correct).
        if dup_xpub_path(&cosigners) {
            return Err(ToolkitError::Bip388VerifyDistinctness);
        }
        return verify_emit_from_expected(args, descriptor, &cosigners, no_auto_repair, json_context, stdout, stderr);
    }
```

**(c) Distinctness scan helper** (verify-bundle has no `&[ResolvedSlot]` distinctness fn; `bundle.rs:402`'s is private — inline a scan):

```rust
fn dup_xpub_path(slots: &[crate::synthesize::ResolvedSlot]) -> bool {
    for i in 0..slots.len() {
        for j in (i + 1)..slots.len() {
            if slots[i].xpub.to_string() == slots[j].xpub.to_string() && slots[i].path == slots[j].path {
                return true;
            }
        }
    }
    false
}
```

- [ ] **Step 4: Run — verify PASS + no regression**

Run: `cargo test -p mnemonic-toolkit --test cli_descriptor_concrete verify_bundle_concrete_matches_self_produced_cards` then the existing verify-bundle suites (`ls crates/mnemonic-toolkit/tests | grep verify_bundle` first — they are `cli_verify_bundle_full`, `cli_verify_bundle_multi_cosigner_mk1`, `cli_verify_bundle_watch_only`, etc.; run each with `--test <name>`).
Expected: PASS, existing verify-bundle tests still green.

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/cmd/verify_bundle.rs crates/mnemonic-toolkit/tests/cli_descriptor_concrete.rs
git commit -m "feat(verify-bundle): accept bare-concrete --descriptor (A1 P3b)"
```

---

## Task 4 (Phase 4): `export-wallet` `@N`-probe redirect

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/export_wallet.rs` (the `--descriptor` branch, `:328-334`)
- Test: `crates/mnemonic-toolkit/tests/cli_descriptor_concrete.rs`

- [ ] **Step 1: Write the failing tests** — append:

```rust
#[test]
fn export_wallet_atn_descriptor_redirects() {
    let atn = "wsh(sortedmulti(2,@0[704c7836/48'/1'/3'/2']/<0;1>/*,@1[97139860/48'/1'/2'/2']/<0;1>/*))";
    let out = mnemonic().args(["export-wallet", "--descriptor", atn, "--network", "testnet"]).output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("only concrete descriptors") && err.contains("--from-import-json"), "{err}");
}

#[test]
fn export_wallet_originless_concrete_still_accepted() {
    // Regression guard (SPEC R2-I1): origin-less concrete must NOT be rejected.
    let originless = "wpkh(tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/0/*)";
    let out = mnemonic().args(["export-wallet", "--descriptor", originless, "--network", "testnet"]).output().unwrap();
    assert!(out.status.success(), "origin-less concrete must pass: {}", String::from_utf8_lossy(&out.stderr));
}
```

- [ ] **Step 2: Run — verify FAIL**

Run: `cargo test -p mnemonic-toolkit --test cli_descriptor_concrete export_wallet_atn_descriptor_redirects export_wallet_originless_concrete_still_accepted`
Expected: the `@N` redirect test FAILS (today miniscript errors with a generic parse message, not the redirect); the origin-less test should already PASS (guard against future regression).

- [ ] **Step 3: Implement** — in `export_wallet.rs`, at the top of the `if let Some(desc) = &args.descriptor` branch (`:328`), before `MsDescriptor::from_str`:

```rust
    let canonical = if let Some(desc) = &args.descriptor {
        // @N-probe ONLY (NOT classify_descriptor_form — its rule 4 would
        // reject origin-less concrete that passthrough accepts). SPEC §3.4.
        if crate::wallet_import::pipeline::is_at_n_form(desc) {
            return Err(ToolkitError::BadInput(
                "export-wallet --descriptor accepts only concrete descriptors with inline keys; \
                 for keyless @N templates use --template <T> --slot @N.xpub=… or --from-import-json".into(),
            ));
        }
        // Descriptor passthrough: parse + canonicalize via miniscript.
        use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
        use std::str::FromStr;
        let d = MsDescriptor::<DescriptorPublicKey>::from_str(desc)
            .map_err(|e| ToolkitError::DescriptorParse(format!("export-wallet --descriptor: {e}")))?;
        d.to_string()
    } else {
```

Add the tiny public predicate to `pipeline.rs` (next to `at_n_probe`):

```rust
/// `@N`-form probe for callers that must NOT trigger the rule-4 origin
/// error (export-wallet passthrough accepts origin-less concrete). SPEC §3.4.
pub(crate) fn is_at_n_form(s: &str) -> bool {
    at_n_probe().is_match(s)
}
```

(Confirm the exit code: `export-wallet --network testnet` with an `@N` descriptor returns `ToolkitError::BadInput`'s exit code — check `error.rs` `exit_code()` for `BadInput`; the test asserts `!success`, which holds for any non-zero.)

- [ ] **Step 4: Run — verify PASS**

Run: `cargo test -p mnemonic-toolkit --test cli_descriptor_concrete export_wallet`
Expected: both PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/cmd/export_wallet.rs crates/mnemonic-toolkit/src/wallet_import/pipeline.rs crates/mnemonic-toolkit/tests/cli_descriptor_concrete.rs
git commit -m "feat(export-wallet): @N-probe redirect, preserve origin-less passthrough (A1 P4)"
```

---

## Task 5 (Phase 5): Convergence + distinctness + taproot cells

**Files:**
- Modify: `crates/mnemonic-toolkit/tests/cli_wallet_cross_format_convergence.rs`

- [ ] **Step 1: Write the failing convergence test** — append a module. The metamorphic claim: concrete `--descriptor` ≡ the equivalent `@N --descriptor + --slot @N.xpub=` → byte-identical md1/mk1. Both inputs explicitly origin-bearing (SPEC M3).

```rust
#[test]
fn concrete_vs_atn_descriptor_converge_md1_mk1() {
    use assert_cmd::Command;
    let mnem = || Command::cargo_bin("mnemonic").unwrap();
    // Out-of-lexicographic-order sortedmulti (97139860 before 704c7836).
    let concrete = "wsh(sortedmulti(2,[97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*,[704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*))";
    let atn = "wsh(sortedmulti(2,@0[97139860/48'/1'/2'/2']/<0;1>/*,@1[704c7836/48'/1'/3'/2']/<0;1>/*))";

    let c = mnem().args(["bundle", "--descriptor", concrete, "--network", "testnet", "--json"]).output().unwrap();
    let a = mnem().args(["bundle", "--descriptor", atn, "--network", "testnet",
        "--slot", "@0.xpub=tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3",
        "--slot", "@1.xpub=tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC",
        "--json"]).output().unwrap();
    assert!(c.status.success() && a.status.success(), "c={} a={}",
        String::from_utf8_lossy(&c.stderr), String::from_utf8_lossy(&a.stderr));
    let cv: serde_json::Value = serde_json::from_slice(&c.stdout).unwrap();
    let av: serde_json::Value = serde_json::from_slice(&a.stdout).unwrap();
    assert_eq!(cv["md1"], av["md1"], "md1 diverged");
    assert_eq!(cv["mk1"], av["mk1"], "mk1 diverged");
}

#[test]
fn concrete_duplicate_cosigner_rejected_bip388() {
    use assert_cmd::Command;
    // Same (xpub, path) twice → Bip388Distinctness (parity with @N path).
    let dup = "wsh(sortedmulti(2,[704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*,[704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*))";
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["bundle", "--descriptor", dup, "--network", "testnet"]).output().unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).to_lowercase().contains("distinct"),
        "{}", String::from_utf8_lossy(&out.stderr));
}
```

- [ ] **Step 2: Run — verify behavior**

Run: `cargo test -p mnemonic-toolkit --test cli_wallet_cross_format_convergence concrete_`
Expected: both should PASS once Tasks 0-3 are in (the `@N`+`--slot` path already works; the concrete path is the new code). If `md1`/`mk1` diverge, STOP — that's a real bug in `bundle_run_concrete_descriptor`'s synthesis (likely a `path_decl` / default-inference mismatch); surface it before proceeding.

- [ ] **Step 3: (taproot cell)** Append a `tr(NUMS, …)` convergence cell ONLY if a valid testnet `tr(<NUMS>,multi_a(...))` concrete fixture is available; otherwise file a FOLLOWUP `a1-taproot-concrete-convergence-cell` and skip (the `key_regex` already skips the NUMS literal, so the path is exercised by the multisig cell). Do not invent an invalid taproot fixture.

- [ ] **Step 4: Run the full convergence + concrete suites**

Run: `cargo test -p mnemonic-toolkit --test cli_wallet_cross_format_convergence --test cli_descriptor_concrete`
Expected: all PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/tests/cli_wallet_cross_format_convergence.rs
git commit -m "test(descriptor): concrete↔@N convergence + BIP-388 distinctness cells (A1 P5)"
```

---

## Task 6 (Phase 6): Manual prose + version bump

**Files:**
- Modify: `docs/manual/src/40-cli-reference/41-mnemonic.md` (bundle / verify-bundle / export-wallet `--descriptor` blocks)
- Modify: `crates/mnemonic-toolkit/Cargo.toml` (version) + any version-marker sites
- Test: `make -C docs/manual lint` + `make -C docs/manual audit`

- [ ] **Step 1: Update the three `--descriptor` prose blocks.** In `41-mnemonic.md`, find the `--descriptor` flag rows for `bundle`, `verify-bundle`, `export-wallet` (`grep -n 'descriptor' docs/manual/src/40-cli-reference/41-mnemonic.md`). State:
  - bundle / verify-bundle `--descriptor`: "accepts either a BIP-388 `@N` template (keys from `--slot`) **or a bare concrete descriptor** with inline `[fp/path]xpub` keys (watch-only); `h`-form and apostrophe hardened paths both accepted."
  - export-wallet `--descriptor`: "accepts a concrete descriptor (with or without key origins); a keyless `@N` template is rejected with a pointer to `--template --slot` or `--from-import-json`."

- [ ] **Step 2: Run the manual lint** (binaries must be pre-built — `cargo build` first; after `cargo clean` rebuild all four: `mnemonic`, `md`, `ms`, `mk`):

Run: `cargo build -p mnemonic-toolkit && make -C docs/manual lint MNEMONIC_BIN=../../target/debug/mnemonic MD_BIN=... MS_BIN=... MK_BIN=...`
Expected: PASS — flag-coverage lint green (no new flag added, so it should already pass; the prose update keeps it consistent).

- [ ] **Step 3: Bump the version — FOUR sites.** [R0-I4 — `tests/readme_version_current.rs:27` requires the `<!-- toolkit-version: 0.38.1 -->` marker in BOTH READMEs, so missing one reds the suite at Step 4.]
  - `crates/mnemonic-toolkit/Cargo.toml` `version = "0.38.0"` → `"0.38.1"`.
  - `crates/mnemonic-toolkit/README.md` — the `<!-- toolkit-version: 0.38.0 -->` marker (~:9).
  - repo-root `README.md` — the same marker.
  - `CHANGELOG.md` — add the `## [0.38.1]` entry (per recent-cycle convention).
  Verify with `grep -rn '0\.38\.0' crates/mnemonic-toolkit/README.md README.md CHANGELOG.md crates/mnemonic-toolkit/Cargo.toml` (all four must flip).

- [ ] **Step 4: Full suite + audit**

Run: `cargo test -p mnemonic-toolkit` then `cargo build` (all 4 binaries) and `make -C docs/manual audit MNEMONIC_BIN=... MD_BIN=... MS_BIN=... MK_BIN=...`
Expected: full suite green; manual audit (lint + verify-examples + anchor-check) green.

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/Cargo.toml docs/manual/src/40-cli-reference/41-mnemonic.md
# + any version-marker test files touched
git commit -m "docs+release(descriptor): manual prose for both --descriptor forms; v0.38.1 (A1 P6)"
```

---

## Task 7: End-of-cycle R0 + ship

- [ ] **Step 1:** Dispatch the opus architect for the end-of-cycle R0 review against the full diff (`git diff master...HEAD`). Persist verbatim to `design/agent-reports/descriptor-form-symmetry-end-of-cycle-R0-review.md`. Fold → re-dispatch until GREEN (0C/0I).
- [ ] **Step 2:** Confirm clean tree, full suite green (`cargo test -p mnemonic-toolkit`), `cargo +stable fmt --check --all`, manual audit green.
- [ ] **Step 3:** File the three FOLLOWUPs from SPEC §9 in `design/FOLLOWUPS.md` (`output-type-stderr-advisory`, `descriptor-origin-extraction-dedup`, `import-parser-hform-origin-tolerance`).
- [ ] **Step 4:** ff-merge `theme-a-descriptor-form-symmetry` → master, push, tag `mnemonic-toolkit-v0.38.1` (toolkit is tag-only, NOT crates.io). No GUI lockstep (no flag change); no sibling-pin change.

---

## Self-Review (controller, post-write)

**Spec coverage:** §3.1 classifier → Task 1; §3.2 helper → Task 2; §3.3 h-form + remap → Task 0 (regex) + Task 2 (remap); §3.4 bundle/verify/export wiring → Tasks 3a/3b/4; §3.5 errors → Tasks 1,2,4 (pinned strings); §4 SemVer → Task 6; §5 tests 1-9 → Tasks 0-5; §6 phases → Tasks 0-6; §9 FOLLOWUPs → Task 7. No gap.

**Placeholder scan:** the verify-bundle synthesize+compare tail (Task 3b Step 3) is now given as literal `verify_emit_from_expected` code (post plan-R0-I2 fold) with the exact reusable span (`verify_bundle.rs:867 + :871-902`) and the `@N`-only lines to exclude (`:856`, `:864-866`) named. The taproot cell (Task 5 Step 3) is conditional with a named FOLLOWUP fallback, not a placeholder. Plan-R0 (2C/4I) folded: args by-ref, real `--json` assertions, dedup imports, verify-tail spec, fork-after-guards, 4 version sites.

**Type consistency:** `DescriptorForm { AtN, Concrete }`, `classify_descriptor_form`, `is_at_n_form`, `descriptor_concrete_to_resolved_slots`, `ResolvedSlot { xpub, fingerprint, path, entropy, master_xpub, _entropy_pin }`, `check_resolved_slots_distinctness` used consistently across Tasks 1-5 and matching the SPEC.

**Known verification points for the implementer** (each task names them): bundle `--json` field keys; verify-bundle card-input flag names; `descriptor_body_no_csum` arity; `self_check_bundle` signature; `ResolvedSlot` full field set; `BadInput` exit code. All are `grep`-checkable before the test is written.
