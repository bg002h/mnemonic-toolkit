# R0 Review — import-json schema-gate (A) + sp/nostr secret-zeroize (B) — ROUND 1

**Source SHA:** `91a2b20`. **Verdict: 🔴 RED — 0 Critical / 2 Important / 4 Minor.** Design holds; all findings are SPEC-prose fixes.

## Critical
None.

## Important

**I1 — README + install.sh self-pin omitted from the Ritual.** `README.md:13` and `crates/mnemonic-toolkit/README.md:9` both carry `<!-- toolkit-version: 0.53.5 -->`; `scripts/install.sh:32` carries `mnemonic-toolkit-v0.53.5`. The SPEC's ritual said "verify" but never concluded to update them. A version bump without these leaves stale markers (the exact class MEMORY documents as missed at v0.53.0). **Fix:** add all three to the Ritual.

**I2 — T-A3 "no-regression" fixture cites an error-path test.** `json_envelope.rs:566-567` is inside `parse_import_json_envelopes_multi_entry_without_index_errors` (a multi-entry *rejection* test) — it errors before the schema gate, so it would pass for the wrong reason / not exercise the gate accepting a valid envelope. **Fix:** use a SINGLE-entry valid `"1"/"4"` raw (the structure at `json_envelope.rs:673-679` is a single valid envelope) parsed with `index=None` → `Ok`; assert it still parses after the gate.

## Minor
**m1 — citation drift:** SP text-path prints are at `silent_payment.rs:294-295`, not the SPEC's `:296-297`. (Recon fixed the `:260-261` drift but the SPEC introduced this one.)

**m2 — T-B3 left open ("R0: pick the assertable form").** DROP T-B3 as a separate test: it is subsumed by T-B4 (ZEROIZE_ROWS lint rows requiring the `: SecretString` literal anchors) + compile-time type enforcement (mistyped fields don't compile). State this.

**m3 — `Zeroizing<String>` → `SecretString` conversion unspecified at nostr.rs:231.** With `wif: Zeroizing<String>`, `Some(wif.clone())` into `Option<SecretString>` won't compile. **Adopt option (c):** make the `wif` local a `SecretString` directly (`let wif = SecretString::new(crate::nostr::wif_for(&norm, args.network));`) — then `wif.clone()` and `format!("{p}{wif}")` (via Display/Deref) both work. Commit to one path.

**m4 — ZEROIZE_ROWS count hits the upper bound.** `lint_zeroize_discipline.rs:262` is `(18..=35).contains(&n)`; current count 31 + 4 new rows = 35 (boundary). Bump the upper bound (e.g. `18..=42`) in the same commit so the next addition doesn't break the count.

## Confirmations
- **Part A chokepoint** correct: both consumers share `parse_import_json_envelopes`; gating the SELECTED (post-index) envelope is right (other array entries are not consumed). Strict-equal fail-closed is correct; "1"/"4" match the emit side (format.rs:120/149); no regression for current valid envelopes.
- **Part B** `SecretString` transparent `Serialize` keeps the `--json` wire byte-identical; multi-copy coverage is COMPLETE for nostr (wif local + each `OutputRow.electrum` + `NostrJson.wif`; `build_import_recipe` uses descriptor=x-only PUBLIC, no WIF) and SP (scan_priv/spend_priv locals + struct). Non-secret fields correctly left `String`. The "accepted limitation" (secret intentionally on stdout; heap-residue-after-return is the real, marginal-but-honest benefit; secp source keys out of scope) is honestly framed — Part B is NOT vacuous.
- **Split:** A+B coherent as one PATCH cycle (both internal, no wire/CLI change); keep combined unless implementation drifts.
- **SemVer PATCH** correct; no schema_mirror/manual/GUI/sibling lockstep.
