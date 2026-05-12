# tech-manual v0.3 — Phase 3.2 reviewer report (r1)

| Field | Value |
|---|---|
| Cut | `tech-manual-v0.3.0` (in progress) |
| Phase | 3.2 (Part IV §IV.2 — Anti-Collision Invariants) |
| Commit under review | `4248c89` |
| Date | 2026-05-11 |
| Reviewer | `feature-dev:code-reviewer` |
| Files in scope | `docs/technical-manual/src/40-bundle-formation/42-anti-collision-invariants.md` · `docs/technical-manual/src/60-back-matter/62-index-table.md` (+14 rows) · `docs/technical-manual/transcripts/mnemonic-bundle-bip388-collision.{cmd,out}` · `docs/technical-manual/.cspell.json` (+1 word "multiset") |

## Findings: 0 Critical / 2 Important / 1 Low / 1 Nit

---

## Important

**I-1. `cs[i].path` is typed `DerivationPath`, not `Option<DerivationPath>` — chapter misstates the type (confidence: 95)**

`42-anti-collision-invariants.md:109`:

> `cs[i].path: Option<DerivationPath>` compares via the typed `PartialEq` derived for `DerivationPath`, which is what folds `h` ↔ `'`.

Actual struct at `synthesize.rs:568-576`:

```rust
pub struct ResolvedSlot {
    pub xpub: Xpub,
    pub fingerprint: Fingerprint,
    pub path: DerivationPath,
    pub path_raw: String,
    pub entropy: Option<Vec<u8>>,
}
```

`CosignerKeyInfo = ResolvedSlot` (type alias, `synthesize.rs:190`), so `cs[i].path` is `DerivationPath`, not `Option<DerivationPath>`. The comparison at `parse_descriptor.rs:1108` is `cs[i].path == cs[j].path` — direct `DerivationPath == DerivationPath`, not `Option`-wrapped. The PartialEq folding claim is still correct (typed DerivationPath comparison does fold `h` ↔ `'`), but the stated type is wrong. A reader following the prose would search for an `Option<DerivationPath>` field and not find it.

Fix: Change `cs[i].path: Option<DerivationPath>` to `cs[i].path: DerivationPath`.

---

**I-2. "Unreachable from real CLI usage" claim is incorrect for xpub slots in template-mode — the bifurcation is reachable (confidence: 90)**

`42-anti-collision-invariants.md:115`:

> In practice this is unreachable from real CLI usage — the template-mode bundle never accepts a user-supplied `--slot @N.path=...` for paths it derives itself (the path is computed from `--template` + `--account` + family), so `path_raw` always comes out as the canonical `'`-notation string written by `template.rs`.

This is only true for phrase/entropy slots. Xpub slots in template-mode DO accept user-supplied `--slot @N.path=...`. At `bundle.rs:355-363`:

```rust
let (path, path_raw) = match slot_inputs
    .iter()
    .find(|s| s.subkey == SlotSubkey::Path)
{
    Some(p) => {
        let parsed = DerivationPath::from_str(&p.value).map_err(...)?;
        (parsed, p.value.clone())
    }
```

When a user supplies `--slot @0.xpub=<X> --slot @0.path=48h/0h/0h/2h --slot @1.xpub=<X> --slot @1.path=48'/0'/0'/2'` in template-mode, `path_raw` preserves the raw user string. Because `check_resolved_slots_distinctness` compares `path_raw` strings (which differ: `48h/0h/0h/2h` ≠ `48'/0'/0'/2'`), it does NOT fire. The bundle is created. A subsequent `verify-bundle` call DOES fire (typed equality folds the two forms to equal). The bifurcation is reachable in practice.

Fix: Narrow the claim to phrase/entropy slots only, and acknowledge xpub slots as the exception where the bifurcation is live.

---

## Low

**L-1. Stale doc-comment at `error.rs:69-71` — same v0.4 raw-string framing as `bundle.rs:259-260`, not flagged by the chapter (confidence: 80)**

The chapter correctly flags the stale doc-comment at `bundle.rs:259-260`. But `error.rs:69-71` has the same v0.4 residue:

```rust
/// `i` and `j` are the colliding slot indices (i < j) under
/// (xpub, derivation_path_string)` raw-string equality per §4.11.b
/// normalization domain.
```

v0.5 changed `check_key_vector_distinctness` to typed `DerivationPath` equality, making this doc-comment stale in exactly the same way. The chapter describes the stale doc-comment problem at `bundle.rs` but doesn't mention `error.rs`, leaving the same residue silently undocumented.

Resolution: parenthetical added near the `error.rs:68-76` source pointer flagging both `bundle.rs:259-260` and `error.rs:69-71` as v0.4 doc-comment residue (folded inline at phase close).

---

## Nit

**N-1. ms1 Case 3 `ms1_decode[i]` table cell `decode_error` column says `<e>` — imprecise; actual value is `format!("{:?}", e)` Debug representation (confidence: 80)**

`42-anti-collision-invariants.md:52`:

> Case 3 `ms1_decode[i]`: `passed: false`, `decode_error: <e>`

At `verify_bundle.rs:1004-1009`:
```rust
let err_msg = format!("{:?}", e);
checks.push(VerifyCheck { ...
    decode_error: Some(err_msg),
```

The `<e>` shorthand is used consistently in the table for both ms1 Case 3 and the mk1 `DecodeFailed` row. This is technically imprecise (the actual value is the Debug representation of the error, not a Display representation).

Resolution: footnote added clarifying `<e>` and `<mk_codec error message>` are the `{:?}` Debug representations (folded inline at phase close).

---

## Resolution (Phase 3.2 close)

All four findings folded inline at the closing commit. None deferred — fix sizes are uniformly small one-line edits, and inline folding keeps `FOLLOWUPS.md` lean.

---

## Verified-correct items (no action needed)

- `emit_multisig_checks` range `verify_bundle.rs:838-1277` — confirmed.
- `MappingFailure` enum at `verify_bundle.rs:831-836` — confirmed exact.
- Two-pass algorithm range `verify_bundle.rs:895-947` — confirmed.
- `md1_xpub_match` multiset at `verify_bundle.rs:1194-1232` — confirmed exact. Detail text `"md1 pubkeys differ from expected set"` at line 1226 matches chapter.
- `check_key_vector_distinctness` at `parse_descriptor.rs:1104-1117` — confirmed exact.
- `check_resolved_slots_distinctness` at `bundle.rs:261-275` — confirmed exact.
- md1 4-hex format at `bundle.rs:707` — confirmed: `format!("{:02x}{:02x}", bytes[0], bytes[1])`.
- mk1/ms1 5-hex format at `bundle.rs:724` — confirmed: `format!("{:05x}", derive_mk1_chunk_set_id(...))`.
- `derive_mk1_chunk_set_id` packing at `synthesize.rs:42-44` — confirmed.
- `Bip388Distinctness` message at `error.rs:325` and `Bip388VerifyDistinctness` message at `error.rs:328` — confirmed byte-exact.
- `error.rs:68-76` range for variant declarations — confirmed.
- Re-wrap at `verify_bundle.rs:470-471` — confirmed.
- Descriptor-mode bundle synthesis calls typed check at `bundle.rs:982` — confirmed.
- `xpub_to_65` 65-byte form at `synthesize.rs:69-74` — confirmed.
- `chunk_set_id_extract` at `format.rs:379-395` — confirmed.
- Four-case ms1 table Case 1 `decode_error: "skipped: watch-only slot"` — confirmed at `verify_bundle.rs:962-970`.
- Case 4 `ms1_decode[i]` `decode_error: "error: ms1[{}] expected (full-mode bundle) but not supplied"` — confirmed at `verify_bundle.rs:1027-1030`. Case 4 `ms1_entropy_match[i]` `decode_error: "skipped: ms1[{}] not supplied"` — confirmed at line 1037.
- Case 3 `ms1_entropy_match[i]` `decode_error: "skipped: ms1 decode failed"` — confirmed at line 1016.
- `NotSupplied` detail `"cosigner[i] mk1 not supplied"`, decode_error `"skipped: mk1[i] not supplied"` — confirmed at `verify_bundle.rs:1121-1122`.
- `DecodeFailed` detail `"cosigner[i] mk1 decode failed"`, decode_error `msg.clone()` — confirmed at `verify_bundle.rs:1124-1126`.
- `XpubNotInPolicy` detail `"cosigner[i] supplied mk1 card xpub absent from descriptor policy"`, decode_error `"supplied mk1 card xpub absent from descriptor policy"` — confirmed at `verify_bundle.rs:1128-1131`.
- Cascade-skip at `verify_bundle.rs:1141-1149`: `decode_error: "skipped: mk1[i] decode failed"` — confirmed at line 1146.
- Stale doc-comment claim at `bundle.rs:259-260` — confirmed.
- Transcript `.out` file: single line `error: BIP-388 distinct-key violation: slot @0 and slot @1 resolve to identical (xpub, path)` — confirmed byte-exact match to `error.rs:325` message text.
- All 14 new `\index{}` terms have matching rows in `62-index-table.md` pointing to `Anti-Collision Invariants`.
- `verify_bundle.rs:98` — `pub fn run` dispatch entry confirmed.
- "multiset" added to `.cspell.json` — confirmed present.
- ms1 `is_empty()` discriminator in multisig path at `verify_bundle.rs:952` — confirmed.
