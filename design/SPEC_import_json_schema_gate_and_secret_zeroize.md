# SPEC — import-json schema-version gate (A) + silent-payment/nostr secret zeroize (B)

**Cycle:** toolkit PATCH · **Source SHA:** `91a2b20` · **Recon:** `cycle-prep-recon-import-json-schema-version+silentpayment-nostr-zeroize.md`.
**Resolves:** `import-json-schema-version-unchecked` (Part A) + `silentpayment-nostr-priv-not-zeroizing` (Part B). Two independent `[minor]` audit-backlog items, one PATCH release. **Parts A and B are fully separable — R0 may SPLIT into two cycles if it judges the combined scope too large (B carries the design depth).**

No CLI flag/help/subcommand change in either part → **no `schema_mirror` / manual / GUI / sibling-codec lockstep.** SemVer **PATCH**.

---

## PART A — gate the import-json `schema_version`

### Problem (verified @ `91a2b20`)
The import-json envelope deserializes `schema_version` into `String` fields — outer `ImportJsonEnvelope.schema_version` (`json_envelope.rs:64`, emitted `"1"`) and inner `BundleJsonView.schema_version` (`:150`, emitted `"4"`) — but **no consumer reads or version-gates them.** Both consumers (`export_wallet.rs:619 run_from_import_json`, `bundle.rs:1716 bundle_run_from_import_json`) route through the shared `parse_import_json_envelopes` (`json_envelope.rs:256`) → `envelope_to_resolved_slots`, neither of which checks the version. serde drops unknown fields by default, so a FUTURE incompatible envelope (e.g. a hypothetical schema "2"/"5" that repurposes a field) would be silently mis-parsed by an older toolkit instead of cleanly rejected.

### Design
Gate in the single shared chokepoint `parse_import_json_envelopes` (`json_envelope.rs:256-290`) so BOTH consumers inherit it. Refactor the two inline `Ok(...)` returns to select into a local, validate, then return:
```rust
const SUPPORTED_ENVELOPE_SCHEMA: &str = "1";
const SUPPORTED_BUNDLE_SCHEMA: &str = "4";

fn validate_schema_versions(env: &ImportJsonEnvelope, flag_label: &str) -> Result<(), ToolkitError> {
    if env.schema_version != SUPPORTED_ENVELOPE_SCHEMA {
        return Err(ToolkitError::BadInput(format!(
            "{flag_label}: unsupported import-json envelope schema_version {:?} \
             (this toolkit supports {SUPPORTED_ENVELOPE_SCHEMA:?}); upgrade the toolkit",
            env.schema_version)));
    }
    if env.bundle.schema_version != SUPPORTED_BUNDLE_SCHEMA {
        return Err(ToolkitError::BadInput(format!(
            "{flag_label}: unsupported import-json bundle schema_version {:?} \
             (this toolkit supports {SUPPORTED_BUNDLE_SCHEMA:?}); upgrade the toolkit",
            env.bundle.schema_version)));
    }
    Ok(())
}
```
`let selected = match index { … };  validate_schema_versions(&selected, flag_label)?;  Ok(selected)`.

**Strict-equal, fail-closed** (R0 design Q — confirm policy): an unrecognised version is REJECTED with a clear upgrade message, NOT silently parsed. Current valid envelopes (`"1"`/`"4"`) are byte-exactly accepted → zero regression. Strict-equal (not `<=`) because the risk is a FUTURE version with changed semantics; a forward-compat `<=` policy would defeat the gate. The constants tie to the emit side (`format.rs:120/149` BundleJson="4"; outer "1" at import_wallet.rs/json_envelope.rs fixtures) — add a comment cross-citing them so an emit-side bump updates here in lockstep.

### Part A tests
- **T-A1:** an envelope with outer `schema_version:"2"` → `parse_import_json_envelopes` returns `Err(BadInput)` naming the unsupported version. (RED: without the gate it parses Ok and proceeds.)
- **T-A2:** an envelope with `bundle.schema_version:"5"` → `Err(BadInput)`.
- **T-A3 (no-regression):** a SINGLE-entry valid `"1"`/`"4"` envelope parsed with `index=None` still returns `Ok` (R0-r1 I2 — use a single-entry raw like the valid envelope structure at `json_envelope.rs:673-679`, wrapped as a 1-element array; do NOT reuse the `:566-567` multi-entry fixture, which errors before the gate for the wrong reason). Confirms the gate ACCEPTS current valid envelopes.

---

## PART B — zeroize the silent-payment + nostr secret strings

### Problem (verified @ `91a2b20`)
Derived PRIVATE-key material is hex/WIF-encoded into plain `String`s that linger un-scrubbed:
- **silent-payment** (`cmd/silent_payment.rs`): `scan_priv`/`spend_priv` = `hex::encode(b_scan.secret_bytes())` / `…b_spend…` (`:260-261`), carried into `SilentPaymentJson{ scan_priv, spend_priv }` (`:97-98` fields, `:275-276` assign) AND printed on the text path (`:294-295`).
- **nostr** (`cmd/nostr.rs`): `wif = wif_for(&norm, …)` (`:212`) propagates into THREE places — each `OutputRow.electrum = format!("{p}{wif}")` (`:221`), `NostrJson.wif = Some(wif.clone())` (`:231`), and the text print (`:249`). (The `import_recipe` (`build_import_recipe`, `:33-40`) clones only `r.descriptor` = x-only PUBLIC key — it does NOT embed the WIF, so no `serde_json::Value` secret-embedding concern.)
`zeroize 1.8` is a dep but has no serde feature → `Zeroizing<String>` does NOT impl `Serialize`, so a serde field can't simply become `Zeroizing<String>`.

### Design — a serialize-transparent `SecretString` newtype (auto-scrub via Drop)
New `crate::secret_string::SecretString(Zeroizing<String>)` (R0 may relocate into an existing util module):
```rust
#[derive(Clone)]
pub struct SecretString(zeroize::Zeroizing<String>);
impl SecretString { pub fn new(s: String) -> Self { Self(zeroize::Zeroizing::new(s)) } }
impl std::ops::Deref for SecretString { type Target = str; fn deref(&self) -> &str { &self.0 } }
impl std::fmt::Display for SecretString { fn fmt(&self, f) -> … { f.write_str(&self.0) } }
impl serde::Serialize for SecretString {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> { s.serialize_str(&self.0) }
}
```
The inner `Zeroizing<String>` scrubs on drop — so EVERY copy (struct field, clone, propagated `electrum`) auto-zeroizes, regardless of where it travelled. `Serialize` is transparent → **the `--json` wire-shape is byte-identical** (no SemVer/wire change). `Display`/`Deref` keep the text-path `writeln!` identical.

Field/local changes:
- silent-payment: `SilentPaymentJson.scan_priv: SecretString`, `spend_priv: SecretString` (`:97-98`); build with `SecretString::new(hex::encode(b_scan.secret_bytes()))` etc. Text path `writeln!(… "{scan_priv}" …)` works via `Display`.
- nostr: `OutputRow.electrum: Option<SecretString>` (`:119` field, `:221` build = `SecretString::new(format!("{p}{wif}"))`), `NostrJson.wif: Option<SecretString>` (`:128` field, `:231` build = `Some(wif.clone())`). **The `wif` local (`:212`) is a `SecretString` DIRECTLY (R0-r1 m3, option (c)): `let wif = SecretString::new(crate::nostr::wif_for(&norm, args.network));`** — then `wif.clone()` (a `SecretString`) and `format!("{p}{wif}")` (via `Display`) and the text-path `writeln!("  wif: {wif}")` all work with no `Zeroizing→SecretString` conversion ambiguity.
- Non-secret fields (`scan_pubkey`/`spend_pubkey`, `descriptor`, `address`, x-only) stay `String` — do NOT over-wrap.

**ACCEPTED LIMITATION (document, do NOT chase):** the secret bytes are written to `stdout` (the command's PURPOSE — `nostr --secret --json` emits the WIF); zeroize only scrubs the in-memory heap copies after use, not the emitted bytes or the OS pipe/terminal buffer. Same best-effort allocator-residue caveat the codebase already documents (`gui-secret-buffer-allocator-residue` analogue). The secp256k1 source `SecretKey`s (`b_scan`/`b_spend`/`norm`) are a separate concern — out of scope here (R0: confirm they're not regressed; they were never the finding's target).

### Part B tests
- **T-B1 (Serialize transparency / no wire change):** `serde_json::to_value(SilentPaymentJson{…})` / `NostrJson{…}` produces the SAME JSON as today for a known fixture (the `scan_priv`/`wif` string values appear verbatim). Pins that the newtype did not change the `--json` output.
- **T-B2 (Display transparency):** `format!("{}", SecretString::new("abc".into())) == "abc"` (the text-path render is unchanged).
- **T-B3 — DROPPED (R0-r1 m2):** zeroize-on-drop isn't directly assertable (reading freed memory is UB). The intent is fully covered by T-B4's ZEROIZE_ROWS literal anchors (`: SecretString`) + compile-time type enforcement (a mistyped field won't compile). No separate test.
- **T-B4 (lint regression guard):** add rows to `tests/lint_zeroize_discipline.rs` ZEROIZE_ROWS for the new secret sites (`silent_payment.rs` scan_priv/spend_priv, `nostr.rs` wif/electrum — 4 rows) with `: SecretString` source anchors, so a future regression that un-wraps them trips the lint. **Bump the row-count range upper bound (R0-r1 m4): `lint_zeroize_discipline.rs:262` is `(18..=35)`; the current count (~31-32) + 4 new rows lands at/over the boundary → widen to `(18..=42)` for headroom** (confirm the exact resulting `ZEROIZE_ROWS.len()` at impl time via `cargo test`). (The lint currently does NOT list these — recon-confirmed.)

---

## Ritual
CHANGELOG `[<next patch>]` (two bullets, A + B); version bump (Cargo.toml + Cargo.lock). **Self-pin sites (R0-r1 I1 — they DO exist, verified @ `91a2b20`): bump `<!-- toolkit-version: 0.53.5 -->` in BOTH `README.md:13` and `crates/mnemonic-toolkit/README.md:9`, AND the tag string `mnemonic-toolkit-v0.53.5` in `scripts/install.sh:32`** (tag-gated lockstep site missed at v0.53.0 per MEMORY). FOLLOWUPS resolve both slugs. Bump the `lint_zeroize_discipline.rs:262` row-count range upper bound (R0-r1 m4, see Part B). No manual/schema_mirror/GUI/sibling lockstep. Mandatory R0 gate to 0C/0I before any code; persist reviews to `design/agent-reports/` before fold-and-commit.

## Non-goals
The secp256k1 source-key zeroize (separate, not the finding); mlock-pinning the OUTPUT WIF (`nostr.rs:205` already pins the INPUT — adjacent, optional, defer); the emitted-bytes / OS-pipe residue (inherent); any wire-shape change.
