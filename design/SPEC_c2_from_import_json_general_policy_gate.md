# SPEC — C2: `export-wallet --from-import-json` refuses general policies for template-requiring formats

**Cycle:** toolkit **PATCH** (v0.54.1 → **v0.54.2**) · **Source SHA:** `9533fba` · **Recon:** `design/agent-reports/fragment-backup-restore-review-2026-06-11.md` (C2) + `design/RECON_faithful_general_policy_restore.md`.
**Resolves:** `export-wallet-from-import-json-template-collapse` (C2 — the same `template_from_descriptor` collapse as the restore C1 bug, on the export `--from-import-json` door).

## The bug
`export-wallet --from-import-json <env> --format <template-requiring>` for a GENERAL-policy descriptor silently collapses it to plain multisig. At `cmd/export_wallet.rs:777-781`:
```rust
let derived_template: Option<CliTemplate> = if format_requires_template(args.format) {
    Some(crate::wallet_export::template_from_descriptor(&parsed_ms)?)   // <-- Wsh(_) => WshMulti collapse
} else { None };
```
`format_requires_template` is true for `sparrow`/`coldcard`/`coldcard-multisig`/`jade`/`electrum` (`:54-62`). For a general policy (e.g. an imported `wsh(and_v(v:multi(2,…),older(1000)))`), `template_from_descriptor` returns `WshMulti` (top-level-wrapper-only), and the emitter produces a plain `wsh(multi(2,…))` payload — the `older` silently dropped. (The descriptor-passthrough formats — `bitcoin-core`/`descriptor`/`bip388`/`bsms`/`green`/`specter` — keep `template: None` and emit the descriptor faithfully, so they are NOT affected.)

The DIRECT `--descriptor` path already refuses template-requiring formats ("requires --template; descriptor passthrough is not supported"), so this bug is specific to `--from-import-json` (which auto-derives the template).

## Fix — structural gate (refuse, don't collapse)
A template-requiring format genuinely cannot represent a general miniscript policy (they are k-of-n multisig wallet formats). So the correct behavior is a CLEAR refusal, NOT a silent collapse. **The predicate is "is this a GENERAL POLICY" (refuse), NOT "is this plain multisig" (R0-r1 C-1 — the latter wrongly refuses SINGLESIG `pkh`/`wpkh`/`sh(wpkh)` envelopes, which legitimately flow through `template_from_descriptor` → `Bip44`/`Bip84`/`Bip49` and have existing green cells).** Refuse IFF the descriptor is a script-hash family (`Wsh` / `Sh(Wsh)` / `Sh(Ms)`) whose root miniscript is NOT a plain `multi`/`sortedmulti`; singlesig falls through unchanged.

```rust
// C2: template-requiring formats can only represent a plain k-of-n multisig.
// A GENERAL policy (timelocks/hashlocks/andor/decay) must NOT be silently
// collapsed to plain multi — refuse loudly. Singlesig (pkh/wpkh/sh-wpkh) and
// plain multisig fall through unchanged. Descriptor-passthrough formats keep
// `None` and emit the descriptor faithfully (unaffected).
let derived_template: Option<CliTemplate> = if format_requires_template(args.format) {
    if descriptor_is_general_policy(&parsed_ms) {
        return Err(ToolkitError::BadInput(format!(
            "--from-import-json: --format {format_name} cannot represent a general wallet \
             policy (timelocks/hashlocks/non-multisig miniscript); it is a plain k-of-n \
             multisig format. Use --format descriptor / bitcoin-core / bip388 for faithful \
             descriptor passthrough."
        )));
    }
    Some(crate::wallet_export::template_from_descriptor(&parsed_ms)?)
} else { None };
```
(`format_name` — derive from the same per-format string literals `emit_payload` already uses (`export_wallet.rs:82-95`), NOT a nonexistent `args.format.as_str()` — R0-r1 M-1.)

New `fn descriptor_is_general_policy(d: &MsDescriptor<DescriptorPublicKey>) -> bool` (in `wallet_export/mod.rs`, beside `template_from_descriptor`/`script_type_from_descriptor`). **There is NO `WshInner` enum at rev `95fdd1c` (removed by #915) — check the root `Terminal` of the inner `Miniscript` directly, generic over `Ctx` (R0-r1 I-1):**
```rust
fn root_is_plain_multi<Ctx: miniscript::ScriptContext>(
    ms: &miniscript::Miniscript<DescriptorPublicKey, Ctx>,
) -> bool {
    matches!(ms.node, miniscript::Terminal::Multi(_) | miniscript::Terminal::SortedMulti(_))
}
fn descriptor_is_general_policy(d: &MsDescriptor<DescriptorPublicKey>) -> bool {
    use miniscript::descriptor::ShInner;
    use miniscript::Descriptor::*;
    match d {
        Wsh(w) => !root_is_plain_multi(w.as_inner()),                 // Segwitv0 inner
        Sh(s) => match s.as_inner() {
            ShInner::Wsh(w) => !root_is_plain_multi(w.as_inner()),    // Segwitv0
            ShInner::Ms(ms) => !root_is_plain_multi(ms),              // Legacy
            ShInner::Wpkh(_) => false,                                // singlesig → fall through
        },
        // Pkh / Wpkh (singlesig) → not general; Tr refused upstream; Bare → fall
        // through to template_from_descriptor's existing handling.
        _ => false,
    }
}
```
(`Tr` is refused upstream by the script-type gate at `export_wallet.rs:728-738` (R0-r1 M-2, corrected citation) — taproot never reaches here. `Wsh(w).as_inner()` is `Miniscript<_, Segwitv0>`; `Sh(Ms)` is `Miniscript<_, Legacy>` — hence the generic helper.)

**R0-r1 M-3 — `Sh(Ms(multi))` interaction:** `descriptor_is_general_policy` returns `false` for a legacy `sh(multi|sortedmulti)` (correct — it IS plain multisig, not a general policy), so it falls through to `template_from_descriptor`, whose `ShInner::Ms` arm still REFUSES it with its OWN specific message ("legacy bare P2SH multisig … has no export-wallet template; use --format bitcoin-core", `mod.rs:279-281`). This is DELIBERATE: the C2 gate must not pre-empt that better-targeted message, and a `descriptor_is_general_policy`-true there would mislabel plain legacy multisig as "general policy". Pin with a cell.

## Tests (RED-first, `tests/cli_export_wallet*.rs` or `cli_from_import_json*.rs`)
- **`from_import_json_general_policy_refuses_template_format`** — import a general policy (`wsh(and_v(v:multi(2,…),older(1000)))`) to an envelope; `export-wallet --from-import-json <env> --format sparrow` exits non-zero with the clear "cannot represent a general wallet policy" message. RED pre-fix (today it emits a collapsed `wsh(multi(2,…))`). Repeat for `coldcard`/`jade`/`electrum`/`coldcard-multisig` (parametrized).
- **`from_import_json_general_policy_passthrough_faithful`** — same envelope, `--format descriptor` (and `bip388`/`bitcoin-core`) emits the FAITHFUL descriptor (contains `older(1000)`), unchanged by the gate.
- **`from_import_json_plain_multisig_template_unchanged`** — a plain `wsh(sortedmulti(2,…))` envelope still maps to its template + emits for `--format sparrow` byte-for-byte as before (the gate must not regress plain multisig). Existing `--from-import-json` goldens stay green.
- **`from_import_json_singlesig_template_unchanged` (R0-r1 C-1, load-bearing)** — `wpkh(…)`/`pkh(…)`/`sh(wpkh(…))` envelopes + `--format sparrow`/`coldcard`/`electrum` still succeed (fall through to `Bip84`/`Bip44`/`Bip49`) — the gate must NOT refuse singlesig. Plus: the existing SINGLESIG_SOURCES happy-path matrix (`tests/cli_export_wallet_from_import_json.rs` ~:946-957) stays green.
- Unit cells for `descriptor_is_general_policy`: `wsh(multi)`/`wsh(sortedmulti)`/`sh(wsh(multi))`/`wpkh`/`pkh`/`sh(wpkh)` → **false** (not general); `wsh(and_v(...))`/`wsh(andor(...))`/`wsh(or_d(...))`/`sh(wsh(and_v(...)))` → **true** (general).
- **`from_import_json_legacy_sh_multi_keeps_specific_message` (R0-r1 M-3)** — a legacy `sh(multi(2,…))` envelope + `--format sparrow` falls through to `template_from_descriptor` and refuses with its OWN "legacy bare P2SH multisig … use --format bitcoin-core" message (NOT the C2 general-policy message).

## SemVer / lockstep / ritual
- **PATCH** v0.54.1 → **v0.54.2** (refuses a previously-silently-wrong input; no flag/wire/schema change). 4 version sites + CHANGELOG.
- **NO `schema_mirror`** (no clap surface change). No GUI/manual flag delta (a new refusal on an existing flag combo — optionally note in the export-wallet manual section). No sibling-codec lockstep.
- FOLLOWUPS: resolve `export-wallet-from-import-json-template-collapse`. Stage paths explicitly. Mandatory R0 gate to 0C/0I; persist reviews to `design/agent-reports/`.

## Non-goals
Faithful re-emit of a general policy to a template-requiring format (impossible — those formats are k-of-n multisig only; refusal is the correct end-state). The direct `--descriptor` path (already refuses). Taproot (already refused upstream).
