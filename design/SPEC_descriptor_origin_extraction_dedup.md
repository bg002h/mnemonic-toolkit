# SPEC — consolidate the duplicated import-parser origin extraction into shared `pipeline` helpers

**Status:** R0 gate (pre-implementation). MUST converge to 0 Critical / 0 Important before any code.
**Resolves:** FOLLOWUP `descriptor-origin-extraction-dedup` + (automatically) `import-parser-hform-origin-tolerance`.
**Source SHA:** branch `descriptor-origin-extraction-dedup` off master `e9ab49a`.
**SemVer:** PATCH — behavior-preserving refactor; the only intentional behavior delta is that the import parsers' origin regex widens to accept `h`-form hardened paths (the `import-parser-hform-origin-tolerance` resolution — a superset, no existing input changes) + a few convergent internal error-message strings. v0.46.2 → **v0.46.3**.

---

## 1. Summary

Origin extraction from a concrete descriptor body (`[fp/path]xpub` → typed `(Xpub, Fingerprint, DerivationPath)`) is **duplicated across the import parsers**, each copy differing only in (i) its error-message prefix (`"import-wallet: <fmt>: parse error: …"`) and (ii) which regex it uses (the parsers carry apostrophe-only copies; `pipeline::key_regex` is the canonical `h`-form-widened one). Recon corrected the FOLLOWUP's file set.

**The real duplication (verified against `e9ab49a`):**
- `fn build_slot_fields` in **6** parsers — `bsms.rs:400`, `bitcoin_core.rs:449`, `sparrow.rs:618`, `coldcard.rs:501`, `specter.rs:397`, `electrum.rs:912`. (FOLLOWUP wrongly listed `coldcard_multisig.rs`, which has **none**; the real 6th is `bitcoin_core.rs`.) **Three distinct signatures:** `(body, slot_idx)` {bsms, specter, sparrow, electrum}; `(body)` {coldcard, single-key}; `(body, slot_idx, entry_idx)` {bitcoin_core}.
- `fn extract_origin_components` in **4** — `bsms.rs:363`, `bitcoin_core.rs:414`, `specter.rs:363`, `sparrow.rs:583` (coldcard + electrum inline the regex instead).
- `fn origin_capture_regex` (apostrophe-only `(?:/\d+'?)+`) in **4** — `bsms.rs:514`, `bitcoin_core.rs:557`, `specter.rs:355`, `sparrow.rs:565` — PLUS inline apostrophe-only copies in `coldcard.rs:507` + `electrum.rs:918`.
- Canonical `pipeline::key_regex` (`h`-form-widened `(?:/\d+(?:'|h)?)+`) at `pipeline.rs:37`.

The inner logic is byte-identical modulo (i)+(ii): captures (1=fp-hex, 2=path, 3=xpub) → 4-byte fp parse → `DerivationPath::from_str("m"+path)` → `slip0132::normalize_xpub_prefix` → `Xpub::from_str`.

## 2. Shape — extract the shared INNER helpers (NOT one uniform `build_slot_fields`)

Because the 3 `build_slot_fields` signatures genuinely differ (slot selection / `entry_idx` / single-key), DO NOT force them into one signature. Instead lift the two truly-shared inner steps into `wallet_import/pipeline.rs`, parameterized by a `format_name: &str` (the per-parser error prefix — the legitimate per-site difference):

```rust
/// Lift every `[fp/path]xpub` origin tuple from a concrete descriptor body
/// via the canonical (h-form-widened) `key_regex`. `format_name` is the
/// per-parser error prefix. Returns declaration order; empty → error.
pub(crate) fn extract_origin_components(
    body: &str,
    format_name: &str,
) -> Result<Vec<(Fingerprint, DerivationPath, String)>, ToolkitError> {
    let mut out = Vec::new();
    for cap in key_regex().captures_iter(body) { /* fp-hex→[u8;4], path, xpub_str — verbatim, prefix=format_name */ }
    if out.is_empty() {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: {format_name}: parse error: no origin annotations in descriptor"
        )));
    }
    Ok(out)
}

/// Finalize one origin tuple → typed slot fields (slip0132-neutralize → Xpub).
pub(crate) fn finalize_slot_fields(
    fp: Fingerprint, path: DerivationPath, xpub_str: &str, format_name: &str,
) -> Result<(Xpub, Fingerprint, DerivationPath), ToolkitError> {
    let (neutral, _variant) = crate::slip0132::normalize_xpub_prefix(xpub_str)?;
    let xpub = Xpub::from_str(&neutral).map_err(|e| ToolkitError::ImportWalletParse(format!(
        "import-wallet: {format_name}: parse error: xpub decode: {e}")))?;
    Ok((xpub, fp, path))
}
```

Each parser's `build_slot_fields` keeps its thin per-parser signature + selection logic, but its body collapses to: `extract_origin_components(body, "<fmt>")` → select (`.nth(slot_idx)` / `[0]` / `entry_idx` logic) → `finalize_slot_fields(...)`. The 6 inline regex/fp/path/xpub blocks (~30 lines each) are deleted; the 4 `extract_origin_components` + 4 `origin_capture_regex` + 2 inline regexes go.

## 3. The `h`-form widening (resolves `import-parser-hform-origin-tolerance`)

Routing all parsers through `key_regex()` (`(?:/\d+(?:'|h)?)+`) means a Core/Sparrow wallet-file descriptor using `h`-form hardened markers (`84h/0h/0h`) now parses where the apostrophe-only copies refused it. This is a **superset** — every apostrophe-form input still matches identically — so no existing behavior changes; it only ADDS `h`-form acceptance. This is the intended resolution of `import-parser-hform-origin-tolerance` (dissolved, not separately fixed). Document in the CHANGELOG.

## 4. Convergent error-message strings (decision: accept)
A few per-parser internal messages converge to the unified wording: coldcard's `"no origin annotation in synthesized descriptor (internal bug)"` and electrum's `"slot index N out of range in synthesized descriptor"` become the shared `"no origin annotations in descriptor"` / generic out-of-range. **No test pins these** (grep of `tests/` for `"internal bug)"` / `"synthesized descriptor"` / `"no origin annotation"` returns only an unrelated comment). These are internal "can't happen on a self-synthesized descriptor" guards, so the reword is invisible in practice. Accept + note in CHANGELOG. (R0 to confirm no test/manual pins them + that the per-parser slot-selection error messages a USER can hit are preserved where they carry distinct user-facing meaning.)

## 5. Tests
- **Green-stays-green:** the existing `import-wallet` per-format suites + the foreign-format transcript suite (`make -C docs/manual verify-examples`) cover every parser's origin extraction end-to-end — a behavior-preserving refactor. Run the full workspace suite + `make audit`.
- **Phase-1 RED cell:** add ONE `import-wallet` cell feeding an `h`-form hardened-path descriptor (e.g. Core/Sparrow export with `84h/0h/0h`) through a parser that previously used the apostrophe-only regex — RED against current (refused), GREEN after §2/§3. This pins the `import-parser-hform-origin-tolerance` resolution + proves the canonical regex reaches the parsers.
- Full `cargo test --no-fail-fast` + clippy `--all-targets` GREEN (clippy flags any now-unused `origin_capture_regex`/imports → remove).

## 6. Lockstep / scope
- **NONE.** No clap flag/value/subcommand change → no GUI `schema_mirror`, no manual CLI-reference mirror. (The `h`-form tolerance is an input-acceptance widening, not a surface change — but DO note it in the CHANGELOG + verify no manual prose claims the parsers are apostrophe-only.) No new error variant (reuses `ImportWalletParse`). No sibling-codec change.

## 7. Phased plan
- **Phase 1 (RED):** the `h`-form import-wallet cell (asserts a previously-refused `h`-form descriptor now parses). Verify RED-for-the-right-reason.
- **Phase 2 (GREEN):** §2 shared `extract_origin_components`/`finalize_slot_fields` in `pipeline.rs` + rewrite the 6 `build_slot_fields` as thin wrappers + delete the 4 `extract_origin_components` + 4 `origin_capture_regex` + 2 inline regexes + prune unused imports. Workspace test + `make audit` + clippy GREEN. Per-phase opus review → persist.
- **Phase 3 (release):** CHANGELOG `[0.46.3]` (note the `h`-form widening + convergent messages); version v0.46.2 → **v0.46.3** (Cargo.toml/lock + 2 READMEs + install.sh self-pin); flip `descriptor-origin-extraction-dedup` + `import-parser-hform-origin-tolerance` → resolved. Per-phase review.
- **Phase 4 (ship):** clean tree → ff-merge → tag `mnemonic-toolkit-v0.46.3` → push → watch CI (rust, install/sibling-pin-check; manual fires only if a manual file changed).

## 8. Risk
Low-moderate. The inner-helper lift is behavior-preserving for apostrophe inputs; the `h`-form widening is a deliberate superset (FOLLOWUP-resolving). The bitcoin_core `entry_idx` shape + coldcard single-key shape are preserved as thin wrappers (NOT forced into one signature — that is the trap). R0 must confirm: (i) the corrected 6/4/4 file sets + current line numbers; (ii) the bitcoin_core `entry_idx` selection logic is preserved unchanged through the wrapper; (iii) no test/manual pins the convergent internal messages OR the apostrophe-only behavior; (iv) the canonical `key_regex` capture-group indices (1/2/3) match what all 6 parsers expect (they do — both regexes share the group structure).
