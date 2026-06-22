# R0 REVIEW — cycle-14 PLAN-DOC (close L22 — stdin-secret zeroize via SecretString) — Round 1

Verified against `origin/master = 82c61e76` (v0.66.0).

## VERDICT: GREEN — 0 Critical / 0 Important / 2 Minor

The plan operationalizes the GREEN spec faithfully, re-pins every citation live, and its census is now COMPLETE. Both priority axes (census completeness, `phrase_overlays` deferral) clear.

### Axis 1 — Census completeness (PRIORITY): CONFIRMED COMPLETE
Independently enumerated every `SlotInput` construction + `.value =`/`.value.clone()` against `82c61e76`:
- **3 struct-literal constructions** (`git grep 'SlotInput \{'`): `slot_input.rs:182` (`parse_slot_input`, value `:185`), `bundle_unified.rs:124` (`s()` helper, value `:127`), `slot_input.rs:380` (`slot()` helper, value `:383`). No 4th hidden constructor; no `..` struct-update. The plan's added set (`parse_slot_input` + `bundle_unified.rs` helper, atop the spec's named set) is exactly the complete remainder.
- `parse_slot_input:182-185` is genuinely THE production constructor (only `-> Result<SlotInput>` fn; the other two are `#[cfg(test)]`). "Hard compile-fence" framing correct.
- **4 `.value =` writes** to a `SlotInput`: `slot_input.rs:225` (stdin), `bundle.rs:2629`, `import_wallet.rs:1396`, `verify_bundle.rs:1883` (all `@env:`, `for s in owned.slot.iter_mut()`, gated `s.subkey.is_secret_bearing()`) — all in the plan. Other `.value =` (`convert.rs:1871` `f.value`, `seed_xor.rs:150` `sh.value`, `owned.from.value`) are `FromInput`/share — out of scope.
- **1 `SlotInput` `.value.clone()`**: `import_wallet.rs:1233` (overlay, in plan). The other 10 clones are `FromInput`/share (`seedqr.rs:172` `fi.value` is `FromInput`).

### Axis 2 — Literal correction: CONFIRMED
`slot_input.rs:225` = `slots[stdin_idxs[0]].value = buf;` (not `slots[i]`).

### Axis 3 — SecretString extension: CONFIRMED COMPILES
`secret_string.rs:22` `#[derive(Clone)]` only; `Deref<Target=str>` (`:32`), redacting Debug (`:46`), Serialize (`:52`). Adding plain PartialEq/Eq is additive. `SlotInput`'s other fields (`index:u8`, `subkey:SlotSubkey`) satisfy `#[derive(Debug,Clone,PartialEq,Eq)]` → derive compiles with `value: SecretString` once impls land. `secret_string.rs` PRIMITIVE-allowlisted (`lint:442`) → no lint row for the trait additions.

### Axis 4 — Lint gate: CONFIRMED EXACT
`SECRET_PATTERNS` includes `": SecretString"` (`lint:430`) → `slot_input.rs` (0 matches today) newly matches → SOURCE→declared gate (`lint:482`) requires the `slot_input.rs` row. **Ran the actual partition scan: true partition = exactly 35 files; +slot_input.rs → 36** makes `SECRET_FILE_FLOOR 35→36` (`lint:452`) exact. D4 OMIT of the redundant `convert.rs` doc row is sound (already 2 rows `:172,177`, gates satisfied). Row-count bound `(18..=60)` (`lint:375`): 53+1=54, in range.

### Axis 5 — `phrase_overlays` deferral (PRIORITY judgment): ACCEPTABLE
`SlotInput.value` → `.to_string()` → `phrase_overlays: Vec<(u8,String)>` (`import_wallet.rs:1229`) → `Source::Phrase(String)` (`overlay.rs:97`). The phrase lingers in 2 bare String copies downstream — but these are bare String TODAY (`.to_string()` is status-quo-preserving, NO NEW residue); L22's named scope is `apply_slot_stdin` + the `SlotInput.value` field; the import-wallet `--slot @N.phrase` path is `@env:`/inline only (no stdin `=-` channel, confirmed `import_wallet.rs:286-289`). Plan files FOLLOWUP `phrase-overlay-secretstring` + `OOS-phrase-overlay-deep-wrap`; the FIXED-note does NOT over-claim the overlay is scrubbed. Sound deferral.

### Axis 6 — SemVer / version sweep / scope: CONFIRMED
MINOR v0.67.0 (`main.rs:30 mod slot_input;` private, `lib.rs:177-178 #[cfg(fuzzing)] pub mod`, no `pub use SlotInput` → MINOR by the v0.10.1 `cfg(fuzzing)` precedent). All 6 version sites at 0.66.0 (`Cargo.toml:3`, `README.md:13`, `crates/mnemonic-toolkit/README.md:9`, `install.sh:32`, `fuzz/Cargo.lock:575`, `Cargo.lock:727`) + CHANGELOG. L22 report anchor `:850` + FOLLOWUPS `:1207`/`:1211` live (Site-1's cited `convert.rs:668+` drifted to live `:853/:861/:868` — plan flags it). `resolve_env_var_sentinel -> Result<String,_>` (`env_sentinel.rs:56-59`) → `@env:` re-wrap needs `SecretString::new(...)`. mlock pins (`convert.rs:880-886`) use `.as_ref()`/`.as_bytes()` → deref-absorb under the wrap. No `{slot:?}` of a populated `SlotInput` in non-test source → redacting Debug only improves test output.

## MINOR (fold during TDD — non-blocking)
- **m-1:** the `bundle_unified.rs` test-mod `s()` helper edit needs `use crate::secret_string::SecretString;` added (`:120` has only `use super::*;` + `use crate::slot_input::SlotSubkey;`). Compile-fenced by RED.
- **m-2:** citation precision (the `value:` lines are `:127`/`:185`/`:383`); implementer re-greps in-worktree anyway. Decay-safe.

## Disposition
GREEN. Census complete, `phrase_overlays` deferral leaves no new un-scrubbed secret, lint floor/row math exact (true partition 35→36), all anchors live. The lane may proceed to P1 TDD.
