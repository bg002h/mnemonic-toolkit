# R0 ARCHITECT REVIEW — `SPEC_path_raw_bracketed_bare_unification.md`

**Reviewed at:** `origin/master = dd7c228`. Opus feature-dev:code-reviewer. Persisted verbatim before fold per CLAUDE.md audit-trail convention.

All findings independently grep/read-verified against live source (not the SPEC's line numbers). The SPEC's central byte-identity claim (#4) was scrutinized hardest and **survives**; the approach is sound, but the **producer/consumer enumeration is incomplete in two ways that would ship behavior changes the SPEC does not acknowledge**, and the **C4 prescription is wrong** as written.

## Verification summary of the four §2.1 pillars

- **Pillar 1 (no SemVer cost):** CONFIRMED. `ResolvedSlot` is binary-private (`mod synthesize;` private at `main.rs:27`; no re-export in `lib.rs`). No `tests/` integration test constructs or field-accesses `ResolvedSlot.path_raw` — the only `tests/` hits (`lint_zeroize_discipline.rs:120`, `cli_export_wallet_bsms.rs:478`) are comment/label strings, not field access. PATCH classification holds.
- **Pillar 2 (round-trip fidelity is stale):** CONFIRMED. `check_key_vector_distinctness` (`parse_descriptor.rs:1211`) compares typed `.path`; the pinning test exists (`:1931`); `cinfo_raw` (`:1843`) exists solely to prove non-consultation. The 129 grep count is **exact**. No production consumer reads `path_raw` for source bytes (see exceptions below).
- **Pillar 3 (`path_raw.is_empty()` ⟺ `path == DerivationPath::default()`):** CONFIRMED for the producers. `DerivationPath::default().to_string() == ""` is correct for `bitcoin 0.32.8` (`DerivationPath` Display writes nothing for an empty path — verified against the rust-bitcoin 0.32 source). The only empty-`path_raw` producer is the WIF slot (`bundle.rs:678`) which also has `path == default`.
- **Pillar 4 (bracket-fp == `slot.fingerprint`):** CONFIRMED for `json_envelope.rs:337-360`. One residual casing edge (uppercase-hex foreign fp) noted as Minor — benign because no consumer reads the rebuilt bracket's fp casing.

**Bracket byte-identity (#4, highest risk):** `DerivationPath` Display in `bitcoin 0.32.8` writes **no leading `m`/`/`**, `/`-separated, hardened as `'` (apostrophe, non-alternate). All six tuple parsers use the identical regex `\[([0-9a-fA-F]{8})((?:/\d+'?)+)\]` — capture group 2 carries a **leading `/`** and only `'`-form. So `bracketed_origin()` = `[fp/{path}]` reproduces `format!("[{fp_hex}{path_raw_inner}]")` byte-for-byte (except uppercase-fp casing, irrelevant to consumers). **The claim holds.**

---

## CRITICAL

### C-1. Two `CosignerKeyInfo` producer sites are missing from §4 — one ships an unflagged JSON wire-shape behavior change.
**`crates/mnemonic-toolkit/src/cmd/bundle.rs:1382`** and **`crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:830`** are both `CosignerKeyInfo { … path_raw, … }` field-init construction sites. Neither appears in the §4 producer list (§4 lists `bundle.rs:505/581/630/674` + `:1442` and the tuple parsers, but not the descriptor-mode push at `:1382`, nor verify-bundle at all). The SPEC's safety net ("Phase 3 field-deletion is the forcing function; every missed site is a compile error") will *catch* these as compile errors — but the plan as written does not enumerate them, so the per-phase work is under-scoped, and more importantly:

At `bundle.rs:1322-1332` (descriptor mode, `Xpub` slot with a user-supplied `--slot @N.path=`), `path_raw = p.value.clone()` — the **user's raw path bytes** (e.g. `48h/0h/0h/2h` or any non-canonical form). This vector is re-wrapped into `resolved_slots` (`bundle.rs:1434-1448`) and flows through `emit_unified`, where C5/C6/C7 read `.path_raw` to emit the `bundle --json` `origin_path`. After deletion → `origin_path_bare()` renders the **canonical** `m/48'/0'/0'/2'`, dropping the user's raw bytes. This is the **same class of behavior change as A1/A2 but on the `bundle --json` descriptor-mode wire-shape**, and it is not declared anywhere in §6/§10.

**Fix:** (1) Add `bundle.rs:1382` and `verify_bundle.rs:830` to the §4 producer list. (2) Add an explicit **Amendment A3** covering descriptor-mode `--slot @N.path=<raw>` → canonical `origin_path` in `bundle --json`, with a test asserting current behavior is either untested or intentionally changed. (Verify `cli_compare_cost.rs:770`'s `48h/…/2h` is a `--descriptor`-string input — which is canonicalized through `parse_descriptor` and therefore unaffected — and confirm no test pins a `--slot @N.path=`-injected non-canonical `origin_path`. I found none, but the amendment must state this explicitly.) `verify_bundle.rs:830`'s `path_raw` is never read for emission (`synthesize_descriptor` uses typed `.path` only; distinctness uses typed `.path`), so that site is a pure mechanical field-init drop.

### C-2. §5 row C4 prescription (`pipeline.rs::key_origin_str`) is wrong — it drops the BIP-32 origin path on the empty/fallback case, corrupting exported descriptor keys.
**`crates/mnemonic-toolkit/src/wallet_export/pipeline.rs:33-44`.** Current `key_origin_str` branches on `slot.path_raw.is_empty()`: when empty it builds the bracket from a **path-bearing `fallback_path`** (the template-derived path) → `[fp/<fallback>]`; when non-empty it uses `path_raw`. The §5 table's concrete fix for C4 says *"replace whole body with `slot.bracketed_origin()`"* — but `bracketed_origin()` returns `[fp]` (no path) for the default-path/empty case, **not** `[fp/<fallback>]`. For any single-sig export slot that hits the fallback (e.g. the WIF degenerate slot at `bundle.rs:674`, `path == default`), the exported descriptor key would become `[fp]xpub…` instead of `[fp/84'/0'/0']xpub…` — a silently corrupted/invalid descriptor. The §3 `bracketed_origin()` signature has no `fallback_path` parameter, so it **cannot** reproduce this branch.

The SPEC half-acknowledges this in the §5 prose note ("Verify whether any caller relies on the fallback producing a path-bearing bracket … Resolve at Phase 0") — but it is filed as a Phase-0 *verification* while the table row states the wrong fix as settled. This is a **design hole that must be closed before code**, not a Phase-0 spot-check: the resolution (keep `fallback_path` handling, e.g. `if slot.origin_path_bare().is_empty() { format!("[{fp}/{}]", fallback.trim_start_matches('/')) } else { slot.bracketed_origin() }`) changes the method-surface contract and must be in the SPEC's §3/§5 before implementation. As written, an implementer following C4 literally ships a broken single-sig export descriptor.

**Fix:** Rewrite C4 to preserve the fallback-path branch explicitly (the bracket must remain path-bearing). Add a unit test (extend T5) pinning `key_origin_str(wif_slot, "84'/0'/0'") == "[fp/84'/0'/0']"` both before and after. Confirm whether miniscript accepts `[fp]xpub` at all (the change may be a hard parse failure, not just a cosmetic one).

---

## IMPORTANT

### I-1. §1.1 root-cause claim is right but the consumer that fixes it (C6) shares `emit_unified` with the descriptor path — T1 must exercise the import-json path specifically.
The §1.1 bug (`bundle --import-json --json` → `"m/[fp/path]"`) is confirmed live: `bundle.rs:767` `normalize_origin_path(&s.path_raw)` where `s` comes from `envelope_to_resolved_slots` (bracketed `path_raw`). Cell 2 of `cli_bundle_import_json.rs:67` genuinely does not assert `origin_path` — confirmed. But because `emit_unified` is shared by all three entry paths (`bundle_run_unified`, `…_descriptor`, `…_from_import_json`), T1 must run via **`bundle --import-json`** (the `bundle_run_from_import_json` route at `bundle.rs:1687`), not the native or descriptor route, to actually guard the §1.1 regression. The SPEC's T1 says "multisig fixture `envelope_v0_27_0.json`" which implies the right route — make it explicit in the test matrix that T1 invokes `bundle --import-json <envelope>`.

### I-2. A1 framing mischaracterizes the *current* C9/C10 value as canonical-ish; it is actually **bracketed**.
**`bundle.rs:1630`** (`resolved_slots[i].path_raw` from `envelope_to_resolved_slots`, no band-aid) and **`overlay.rs:179`** (`bundle.cosigners[i].path_raw` from the import-wallet decode) both carry the **bracketed `[fp/48'/0'/0'/2']`** form today, so `ImportWalletSeedMismatch`'s `at path {path}` clause currently prints `[fp/…]`, not a bare path. A1 describes this as echoing "source `path_raw` bytes" and implies a minor `'`-notation tweak; the real change is bracketed → bare `m/…`. The conclusion (display-only, no test pins it) is **correct** — `cli_import_wallet_seed_overlay.rs:136` asserts only the substring `"supplied seed produces xpub"`, before the `path` clause, so it survives. But fold the accurate characterization into A1 so the reviewer/implementer isn't surprised by the magnitude of the string change.

### I-3. C11 empty-path divergence (`"m"` vs `""`) is unstated.
**`import_wallet.rs:1945`** `origin_path_from_bracket("[fp]")` (no inner slash) returns `"m"`; the replacement `origin_path_bare()` returns `""` for a default-path slot. These feed `import-wallet --json`'s `origin_path`. In practice the import regex `(?:/\d+'?)+` guarantees a path is always captured, so a pathless cosigner bracket is unreachable from the foreign-format parsers — but the SPEC asserts a clean swap without noting the edge. State the reachability argument (regex requires ≥1 path component ⟹ `bracketed_origin()` always path-bearing for these slots ⟹ `origin_path_bare()` always non-empty) so the `"m"`→`""` divergence is provably dead.

### I-4. §4(a) mislabels three test-only synthesize.rs sites as production.
`synthesize.rs:1141`, `:1262`, and `:1375` are **all** inside `#[cfg(test)] mod tests` (begins at `synthesize.rs:778`). The SPEC tags only `:1375` as a `#[cfg(test)]` fixture and presents `:1141`/`:1262` as production ("bare `path.to_string()`"). No functional impact (all need the field-init dropped regardless), but the enumeration's production/test split is inaccurate — correct it so the phase-3 "full build compiles" expectation correctly attributes these to the test target.

---

## MINOR

### M-1. Pillar-4 "byte-for-byte" is technically false for uppercase-hex foreign fingerprints.
The capture regex permits `[0-9a-fA-F]{8}`; `bracketed_origin()` lowercases via `fingerprint.to_string().to_lowercase()`. A foreign descriptor with an uppercase-hex `[ABCD1234/…]` would have its rebuilt bracket lowercased. Benign — C4 (after fix) and C11 both already discard/lowercase the bracket fp — but the absolute "byte-for-byte" wording in §2.1 pillar 4 should be softened to "byte-for-byte for all path-sensitive consumers; fingerprint casing is normalized to lowercase (already the case for every emit consumer)."

### M-2. §3 implementation-note Display verification can be marked DONE now.
The Phase-0 "confirm `DerivationPath` Display renders no leading `m`/`/`" is already settled by this review against `bitcoin 0.32.8` source (Display writes first component then `/`-joined, hardened `'`). Promote it from a Phase-0 TODO to a stated fact with the source citation, and keep T5(d) as the regression pin.

### M-3. §4(b) tuple-4th-element "re-verify per file" can be closed for sparrow now.
Verified at `sparrow.rs:431` that the 4th tuple element feeds only the `ResolvedSlot` push; the descriptor body is built from the separate `substituted` string. The pattern is identical across the other five parsers (all `out.push((fp, path, path_raw, xpub_str))` → consumed only into `ResolvedSlot`). Safe to assert in the SPEC rather than defer.

### M-4. §10 GUI note is correct but should commit to a decision.
The "File a heads-up note in the GUI repo? — value-only, low-impact" is left as an open question. Per the project's paired-PR / `--json` wire-shape self-update convention (CLAUDE.md GUI schema-mirror section), the `origin_path` *value* change in `bundle --json` (both the §1.1 fix AND the C-1/A3 descriptor-mode change) is not schema_mirror-gated; commit to "CHANGELOG note + GUI self-updates via paired-PR rule, no hard lockstep" rather than posing it as a question.

---

## §9 phase-ordering adjudication
"Delete the field last (Phase 3)" is **sound** as a forcing function — it correctly turns every missed producer into a compile error, which is how C-1's two omitted sites would surface. No earlier phase references the deleted field or a not-yet-added method out of order: Phase 0 adds the methods, Phases 1-2 swap consumers to the new methods (field still present), Phase 3 deletes producers + field together. The ordering does **not** sink the plan. The gap is enumeration completeness (C-1) and the C4 prescription (C-2), not sequencing.

---

## VERDICT: RED (2C / 4I)

Counts: **Critical 2, Important 4, Minor 4.**

The core approach (delete the denormalized field, derive from typed `fingerprint`+`path`) is correct and the bracket byte-identity claim survives scrutiny. But two Criticals must be folded before any code: **C-1** (two missing producer sites, one of which silently changes the `bundle --json` descriptor-mode `origin_path` wire value — needs an A3 amendment + tests) and **C-2** (the C4 prescription as written drops the origin path on the fallback case and corrupts exported descriptor keys — the `bracketed_origin()` contract must gain fallback handling in §3/§5 before implementation, not as a Phase-0 spot-check). Re-dispatch after folding per the after-every-fold reviewer-loop discipline.
