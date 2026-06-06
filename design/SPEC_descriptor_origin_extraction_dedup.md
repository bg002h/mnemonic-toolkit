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

The inner logic is byte-identical modulo (i) error-message prefix, (ii) regex, **(iii) per-slot/entry context carried in the xpub-decode + out-of-range messages** (see §4): captures (1=fp-hex, 2=path, 3=xpub) → 4-byte fp parse → `DerivationPath::from_str("m"+path)` → `slip0132::normalize_xpub_prefix` → `Xpub::from_str`.

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

**(R0 M3) The per-parser SELECTION (out-of-range) message STAYS in the wrapper** — it lives in the `.nth(slot_idx).ok_or_else(...)` step the wrapper keeps, so its `entry_idx`/`slot_idx` context is retained for free (bitcoin_core's `descriptors[{entry_idx}]: slot index {slot_idx} out of range`; bsms/sparrow/specter `slot index {slot_idx} out of range`; electrum's `…in synthesized descriptor`). ONLY `finalize_slot_fields`'s xpub-decode message converges (§4). Do NOT flatten the wrapper's out-of-range message.

## 3. The `h`-form widening (resolves `import-parser-hform-origin-tolerance`)

Routing all parsers through `key_regex()` (`(?:/\d+(?:'|h)?)+`) means a Core/Sparrow wallet-file descriptor using `h`-form hardened markers (`84h/0h/0h`) now parses where the apostrophe-only copies refused it. This is a **superset** — every apostrophe-form input still matches identically — so no existing behavior changes; it only ADDS `h`-form acceptance. This is the intended resolution of `import-parser-hform-origin-tolerance` (dissolved, not separately fixed). Document in the CHANGELOG.

## 4. Convergent error-message strings (decision: accept)
The ONLY messages that converge are those produced INSIDE `finalize_slot_fields` (the xpub-decode branch) — the per-parser SELECTION (out-of-range) messages stay in the wrappers (§2, R0 M3). Convergences:
- **xpub-decode (R0 M1):** the per-slot/entry context in `bitcoin_core.rs:463` (`descriptors[{entry_idx}]: xpub decode for slot {slot_idx}: {e}`), `electrum.rs:949` / `sparrow.rs:631` / `specter.rs:410` (`xpub decode for slot {slot_idx}: {e}`) flattens to the shared `"import-wallet: {fmt}: parse error: xpub decode: {e}"`.
- coldcard's `"no origin annotation in synthesized descriptor (internal bug)"` (single-key `captures()` path) folds into the shared empty-result `"no origin annotations in descriptor"`.

**(R0 M2) Why the convergence is invisible — proven can't-happen guard.** Every parser calls `pipeline::concrete_keys_to_placeholders` BEFORE `build_slot_fields` (bitcoin_core:267, bsms:222, sparrow:406, specter:224, coldcard:313, electrum:373); that fn (pipeline.rs:116-121) already decodes each `[fp/path]xpub` via the same `key_regex` → `normalize_xpub_prefix` → `Xpub::from_str`, erroring on a bad xpub. So by the time `build_slot_fields` re-lexes the SAME key the decode provably already succeeded (encoded by `debug_assert_eq!` at bitcoin_core:293 / pipeline:199). The `xpub decode for slot` branch is the same defensive "(internal bug)" class — the per-slot context is never user-observable. **No test pins these** (grep of `tests/` + `docs/manual/` for `"internal bug)"` / `"synthesized descriptor"` / `"no origin annotation"` / `"xpub decode for slot"` returns only an unrelated comment at `cli_xpub_search_account_of_descriptor.rs:328`). Accept + note in CHANGELOG. (R0 confirmed.)

**(R0 M4) The eager PATH-parse in the shared `extract_origin_components` is likewise unreachable as a new error.** The shared helper parses every capture's path eagerly (before selection), whereas `electrum.rs` is lazy today (parses only the selected slot). A regex-valid path can still fail `DerivationPath::from_str` (index ≥ 2³¹), so in principle a malformed path in a non-selected electrum slot could flip skipped→error. But this is provably can't-happen — NOT via M2 (`concrete_keys_to_placeholders` only COPIES the path string, never `from_str`s it), but via **`parse_descriptor` → `lex_placeholders`** (`parse_descriptor.rs:90-105`), which runs BEFORE every parser's `build_slot_fields` (bsms 227→251, bitcoin_core 279→292, coldcard 321→334, electrum 380→395, + specter/sparrow) and `DerivationPath::from_str`-validates EVERY `@N[fp/path]` placeholder's path. Any malformed path errors there first → the loop is never reached → electrum's lazy→eager shift is behavior-preserving. (R0 confirmed.)

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
