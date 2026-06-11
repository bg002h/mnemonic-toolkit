# cycle-prep recon — 2026-06-11 — import-json-schema-version-unchecked + silentpayment-nostr-priv-not-zeroizing

**Origin/master SHA at recon time:** `91a2b20`
**Local branch:** `master`
**Sync state:** up-to-date (0 ahead / 0 behind)
**Untracked:** the GUI-cluster + this recon scratch files (none in the toolkit src tree).

Two independent `[minor]` audit-backlog slugs. Expect ACCURATE-with-minor-drift.

---

## Per-slug verification

### `import-json-schema-version-unchecked`
- **WHAT (from FOLLOWUPS.md):** the import-json envelope carries `schema_version` (outer `"1"`, inner bundle `"4"`) into deserialized `String` fields, but no consumer reads or version-gates it; serde drops unknown fields, so a future incompatible envelope could be silently mis-parsed by an older toolkit.
- **Citations:**
  - `json_envelope.rs:62-90 (ImportJsonEnvelope)` — **ACCURATE.** Struct at `:60-90`; `schema_version: String` at `:64` (comment: `"1"` for v0.27.0).
  - `json_envelope.rs:149-170 (BundleJsonView)` — **ACCURATE.** Struct at `:147-170`; `schema_version: String` at `:150`.
  - consumers `export_wallet.rs:619 run_from_import_json` — **ACCURATE** (`fn run_from_import_json` at `:619`; routes through `parse_import_json_envelopes` + `envelope_to_resolved_slots`, never reads schema_version).
  - consumers `bundle.rs:1716 bundle_run_from_import_json` — **ACCURATE** (`fn bundle_run_from_import_json` at `:1716`; same shared parse, no version check).
  - **Confirming grep:** `schema_version` is read ONLY on the EMIT side (`format.rs:120/149` `&'static str`; `synthesize.rs:1754-1776` pins BundleJson="4"; `seed_xor.rs` own "1"); the CONSUME side never gates it. Emit constants: outer `"1"` (import_wallet.rs:1748 + json_envelope.rs:482/506 fixtures), inner bundle `"4"`.
- **Action for brainstorm spec:** gate in the shared chokepoint `parse_import_json_envelopes` (`json_envelope.rs:256`) so BOTH consumers inherit it. Validate outer `schema_version == "1"` AND each `bundle.schema_version == "4"`; reject an unrecognized/future version with `ToolkitError::BadInput` ("unsupported import-json schema_version <v>; upgrade the toolkit" — fail-closed, not silent mis-parse). Accept the CURRENT values byte-exact (no regression for valid envelopes). Cite source SHA `91a2b20`. **Design Q for R0:** strict-equal vs `<=` (forward-compat policy); whether to gate the inner per-bundle version at parse or at `envelope_to_resolved_slots`.

### `silentpayment-nostr-priv-not-zeroizing`
- **WHAT (from FOLLOWUPS.md):** `scan_priv`/`spend_priv` (silent-payment) and the nostr WIF are plain `String`s (hex/WIF of derived secret bytes) carried into JSON-envelope structs, never zeroized.
- **Citations:**
  - `silent_payment.rs:97-98` (`scan_priv: String`, `spend_priv: String` struct fields) — **ACCURATE.**
  - `silent_payment.rs:256-257` ("hex::encode of the derived scan/spend secret bytes") — **DRIFTED:** the actual `scan_priv = hex::encode(b_scan.secret_bytes())` / `spend_priv = hex::encode(b_spend.secret_bytes())` is at `:260-261`. (`:256-257` is the change_address_warning.) Struct-literal assignment at `:275-276`.
  - `silent_payment.rs:271-272` — **DRIFTED:** those are `scan_pubkey`/`spend_pubkey` (PUBLIC); the secret fields are assigned at `:275-276`.
  - `nostr.rs:212` (`let wif = crate::nostr::wif_for(&norm, ...)`) — **ACCURATE.**
  - `nostr.rs:221` (`electrum: ...format!("{p}{wif}")`) — **ACCURATE** (the WIF propagates INTO each `OutputRow.electrum` string).
  - `nostr.rs:231` (`wif: Some(wif.clone())` into `NostrJson`) — **ACCURATE.**
  - `nostr.rs:249` (`writeln!(stdout, "  wif: {wif}")`) — **ACCURATE.**
- **Multi-copy propagation (LOAD-BEARING for the fix):** the nostr WIF exists in: the `wif` local (`:212`), every `OutputRow.electrum` (`{p}{wif}`, `:221`), `NostrJson.wif` (`:231`), and (via `rows` → `build_import_recipe(args, &rows)` at `:225`) potentially the import recipe. A complete zeroize must cover ALL copies, not just the `wif` local. silent-payment is simpler: `scan_priv`/`spend_priv` live only in the struct + the (text-path) printed values.
- **Existing pattern:** the codebase wraps TRANSIENT input secrets in `zeroize::Zeroizing<String>` (final_word.rs:69, ms_shares.rs:250/284). NONE of these are serde-serialized OUTPUT fields. `lint_zeroize_discipline.rs` ZEROIZE_ROWS does NOT list silent_payment/nostr → adding the fix should add lint rows to prevent regression.
- **Action for brainstorm spec:** wrap/scrub the secret strings. **Design Q for R0 (the crux):** `zeroize::Zeroizing<String>` does NOT implement `serde::Serialize` — so a serde field can't just become `Zeroizing<String>`. Options: (a) keep `String` fields, `zeroize()` them after `to_writer` returns (struct must be `mut`; covers all struct copies but NOT the `electrum`/import_recipe propagations); (b) `#[serde(serialize_with=...)]` shim that serializes the deref'd str while the field is `Zeroizing<String>`; (c) a newtype `SecretString(Zeroizing<String>)` with `Serialize` + `Drop`. R0 must choose AND decide how to scrub the propagated copies (electrum rows, import_recipe, the text-path locals). Also note `nostr.rs:205` already `mlock::pin_pages_for`s the INPUT secret — the OUTPUT WIF is unpinned (adjacent, optional). Cite source SHA `91a2b20`.

---

## Cross-cutting observations
1. Both slugs are independent (input-validation vs secret-hygiene), both internal (no CLI flag/help change) → **no `schema_mirror` / manual / GUI / sibling-codec lockstep.**
2. Slug-2 line citations DRIFTED (hex at :260-261 not :256-257; pubkeys at :271-272). Slug-1 citations all ACCURATE.
3. Slug 1 is a clean, well-bounded parse-gate. Slug 2 has real design depth (serde+Zeroizing interplay + multi-copy propagation + lint integration) — the larger of the two.
4. No incidental cross-pin/version staleness surfaced.

---

## Recommended brainstorm-session scope
**One cycle, one PATCH release, two clearly-separated parts** (A = schema-version gate; B = secret-zeroize), each independently tested + R0-reviewed; SPEC must keep them separable so R0 can SPLIT if it judges the combined scope too large (slug 2's serde/multi-copy depth is the risk). SemVer **PATCH** for both: slug 1 = fail-closed input-validation tightening (no regression for valid current envelopes); slug 2 = pure internal hygiene (no behavior/wire change). No lockstep. Tests: A = reject-unsupported-version cells (outer + inner) + accept-current; B = post-serialize/drop scrub assertions where testable + a `lint_zeroize_discipline` ZEROIZE_ROWS addition for the new secret sites. Mandatory R0 gate to 0C/0I before any code.
