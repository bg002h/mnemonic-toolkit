# R0 Review — C2 from-import-json general-policy gate — ROUND 1

**Source SHA:** `9533fba`. **Verdict: 🔴 — 1 Critical / 1 Important / 3 Minor.**

## Detector + format-partition INDEPENDENTLY VERIFIED (against pinned miniscript rev 95fdd1c)
- **NO `WshInner` enum at rev 95fdd1c** (removed by #915). `Wsh<Pk>` is `struct Wsh { ms }` with `as_inner() -> &Miniscript`; `wsh(sortedmulti)` is `Terminal::SortedMulti` at the Ms root, NOT a variant. (`descriptor/segwitv0.rs:28-49`; toolkit `parse_descriptor.rs:431` confirms.)
- **`ShInner = { Wsh(Wsh), Wpkh(Wpkh), Ms(Miniscript<_,Legacy>) }`** (`sh.rs:40-47`) — no SortedMulti variant; `sh(sortedmulti)` = `ShInner::Ms` root `Terminal::SortedMulti`. SPEC's Sh arm correct.
- **`Terminal::Multi`/`SortedMulti`** exist (`decode.rs:155-157`); `Miniscript.node` pub. `MultiA` is Tap-only (Tr refused before the gate). Wsh inner is `Segwitv0`, `Sh(Ms)` is `Legacy` → root-check helper must be generic over `Ctx`.
- **Format partition confirmed** (`export_wallet.rs:54-60`): template-requiring = Sparrow|Coldcard|ColdcardMultisig|Jade|Electrum; passthrough = BitcoinCore|Bip388|Bsms|Green|Specter|Descriptor. No second C2 surface on passthrough (descriptor/bsms/specter verbatim; bitcoin-core re-parses+emits; bip388 structural key-extract; green refuses multisig loudly).
- **Single call site** of `template_from_descriptor` (`export_wallet.rs:778`; restore uses tree-side `plain_template_from_tree`). Taproot refused upstream at `:728-738`. RED-first feasible (audit demonstrated exit-0 wrong payload; envelope via `import-wallet` bitcoin-core blob).

## Critical
**C-1 — the gate REGRESSES singlesig `--from-import-json` exports.** The detector returns `false` for singlesig (`pkh`/`wpkh`/`sh(wpkh)`), and the gate refuses on `false` — but singlesig legitimately flows through `template_from_descriptor` (`Pkh→Bip44`, `Wpkh→Bip84`, `Sh(Wpkh)→Bip49`, `mod.rs:270-273`); existing green cells (`tests/cli_export_wallet_from_import_json.rs:582/:584` + the SINGLESIG_SOURCES happy-path matrix `:946-957`) would break. **Fix:** refuse IFF the descriptor is `Wsh(w)`/`Sh(Wsh(w))`/`Sh(Ms(ms))` whose root `Terminal` is NOT `Multi|SortedMulti`; `Pkh`/`Wpkh`/`Sh(Wpkh)` fall through unchanged. Add singlesig not-refused cells + matrix-stays-green.

## Important
**I-1 — detector `Wsh` arms are API-invalid (no `WshInner` at rev 95fdd1c).** Use `Descriptor::Wsh(w) => matches!(w.as_inner().node, Terminal::Multi(_) | Terminal::SortedMulti(_))`. Generic helper `fn root_is_multi<Ctx: ScriptContext>(ms: &Miniscript<DescriptorPublicKey, Ctx>) -> bool` (Wsh/Sh(Wsh) = Segwitv0, Sh(Ms) = Legacy).

## Minor
- **M-1** `args.format.as_str()` does not exist — use the `format_name` literals in `emit_payload` (`:82-95`) or `{:?}`.
- **M-2** stale citation: taproot refusal is `export_wallet.rs:728-738` (script-type gate), NOT `:94-100`.
- **M-3** document that `Sh(Ms(multi))` detector-true is correct but `template_from_descriptor`'s `ShInner::Ms` arm still refuses legacy P2SH multisig with its specific message (`mod.rs:279-281`) — pin with a cell; detector-false there would mislabel it "general policy".

## Verified-correct
Refusal IS the right end-state (template formats are k-of-n multisig only — can't carry timelocks/hashlocks/branches). Gate placement before `template_from_descriptor`, passthrough `else` untouched. PATCH v0.54.2, no schema_mirror/GUI/manual/sibling. RED-first genuinely RED.

**Gate status: NOT GREEN.** Fold C-1 + I-1 + 3 minors, re-dispatch Round 2.
