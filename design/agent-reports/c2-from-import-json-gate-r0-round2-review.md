# R0 Review — C2 from-import-json general-policy gate — ROUND 2

**Source SHA:** `9533fba`; pinned miniscript `95fdd1c` (verified against live checkout). **Verdict: 🟢 — 0 Critical / 0 Important / 0 Minor.**

## Round-1 findings — all RESOLVED
- **C-1 RESOLVED.** Gate refuses IFF `descriptor_is_general_policy` is TRUE; detector returns `false` for singlesig (`ShInner::Wpkh(_) => false` + `_ => false` covers Pkh/Wpkh) → they fall through to `template_from_descriptor` (`Pkh→Bip44`/`Wpkh→Bip84`/`Sh(Wpkh)→Bip49`, `mod.rs:270-273`). §Tests adds `from_import_json_singlesig_template_unchanged` + pins SINGLESIG_SOURCES matrix green.
- **I-1 RESOLVED.** No `WshInner`; detector is `root_is_plain_multi<Ctx: ScriptContext>` matching `ms.node` vs `Terminal::Multi(_)|SortedMulti(_)`. Verified at 95fdd1c: `Wsh{ms}` `as_inner()->&Miniscript<_,Segwitv0>` (`segwitv0.rs:28-38`); `ShInner={Wsh,Wpkh,Ms(Miniscript<_,Legacy>)}` (`sh.rs:40-47`); `Miniscript.node` pub; `Terminal::Multi/SortedMulti` tuple variants; `Terminal`/`ScriptContext` crate-root re-exports → paths compile; matches exhaustive. No API error.
- **M-1 RESOLVED.** `format_name` from `emit_payload`'s per-format literals (`:82-95`), not `as_str()`.
- **M-2 RESOLVED.** Taproot citation `:728-738` (`matches!(script_type, P2tr|P2trMulti)`), gate before `:777`.
- **M-3 RESOLVED.** Legacy `sh(multi)` → detector false → falls through to `template_from_descriptor`'s own `ShInner::Ms` refusal (`mod.rs:279-281`), pinned with a cell.

## No-new-drift
`wsh(sortedmulti)` = `Terminal::SortedMulti` at Ms root → not-general → falls through → `WshSortedMulti` (verified parse + `Wsh::new_sortedmulti`). `wsh(multi)` likewise. Gate inherits `format_requires_template`'s exhaustiveness (no `_` arm + partition test `:831-846`). `parsed_ms: &MsDescriptor<DescriptorPublicKey>` matches the detector signature; `matches!` borrows, no move. Passthrough `else { None }` untouched. PATCH v0.54.2, no schema_mirror/GUI/manual/sibling.

**GREEN — ready for implementation (TDD RED-first).**
