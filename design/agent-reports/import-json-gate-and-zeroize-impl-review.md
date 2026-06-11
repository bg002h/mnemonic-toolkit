# Implementation Review — import-json schema-gate (A) + sp/nostr secret-zeroize (B) — before commit

Reviewed the uncommitted v0.53.6 working tree against the R0-GREEN spec.

**Verdict: 0 Critical / 0 Important / 0 Minor.** Clean and spec-conformant.

## Confirmations
- **Wire-shape transparency (load-bearing):** `SecretString::serialize` = `serialize_str(&self.0)`, byte-identical to `String::serialize`. `Option<SecretString>` + `#[serde(skip_serializing_if = "Option::is_none")]` behaves identically to `Option<String>` (the skip is an `Option`-level property). T-B1 pins the `Some` case byte-exact; the full suite incl. `tests/cli_nostr.rs` + `tests/cli_silent_payment.rs` `--json` cells passed.
- **Secret-copy completeness:** every WIF/scan_priv/spend_priv copy is a `SecretString` or scrubbing local — `hex::encode(...)`/`wif_for(...)`/`format!("{p}{wif}")` are each immediately wrapped with no intermediate plain-`String` binding; `NostrJson.wif = Some(wif.clone())` deep-clones the zeroizing buffer; `build_import_recipe` only touches `r.descriptor` (x-only PUBLIC). No escape found.
- **Part A gate:** `validate_schema_versions` called AFTER index selection (`json_envelope.rs:294`); constants "1"/"4" match the emit side (import_wallet.rs ENVELOPE const + synthesize/bundle BundleJson="4"); rejects "2"/"5", accepts "1"/"4"; both consumers route through the chokepoint; `BadInput` = exit 2.
- **Newtype soundness:** length-only Debug (no leak, test-pinned); `#[derive(Clone)]` deep-clones `Zeroizing<String>`; Deref/Display transparent; no Deserialize (output-only). 
- **Lint:** 4 evidence anchors match source substrings exactly (non-vacuous); count 35 ∈ widened `(18..=42)`.
- **Module registration:** `pub mod` in lib.rs + `mod` in main.rs (correct for the shared lib+bin tree).
- **Ritual:** all three self-pins at 0.53.6 (README.md:13, crates README:9, install.sh:32); Cargo.toml/lock bumped; CHANGELOG `[0.53.6]`; both FOLLOWUP slugs resolved.

Faithful to the R0-GREEN spec. No findings.
