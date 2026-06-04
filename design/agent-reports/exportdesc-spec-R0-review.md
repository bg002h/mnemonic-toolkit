# export-wallet --format descriptor — SPEC R0 Review (round 0)
**Verdict:** RED (1C / 1I)

Architecture sound + the load-bearing claim verified, but two compile-stoppers + 4 Minors. Folded; re-dispatching.

## Critical (1)
**C1 — `DescriptorEmitter` (SPEC §2) does not compile + double-newlines.**
- The `WalletFormatEmitter` trait has THREE required methods (`wallet_export/mod.rs:395-399`): `collect_missing`, `emit`, AND `extension() -> &'static str`. Every emitter implements all three (green.rs:47, jade.rs:66, bitcoin_core.rs:35, …). The §2 block omitted `extension()` → E0046. Fix: add `fn extension() -> &'static str { "txt" }`.
- `emit` returned `format!("{}\n", …)` but BOTH dispatch tails already append `\n` (writeln! `:562/:816`; file `format!("{emitted}\n")` `:569/:823`) → `…\n\n`. green (`:41-44`) returns `{}` (no `\n`). Fix: `Ok(inputs.canonical_descriptor.to_string())`.

## Important (1)
**I1 — FIVE exhaustive `match CliExportFormat`-class sites, not 4; the SPEC missed `format_requires_template` (`:53`).** It's exhaustive (no `_`), called on the from-import-json path (`:721`); adding `Descriptor` without an arm = E0004. Correct arm: `Descriptor => false` (passthrough/template-agnostic; group with `BitcoinCore | Bip388 | Bsms | Green | Specter` `:55`). The 5 sites: `:53` format_requires_template, `:504` run collect_missing, `:523` run emit, `:756` from-import-json collect_missing, `:777` from-import-json emit.

## Minor (4)
- **M1** citation drift: enum `:21-43` (10 variants); trait `:395-399`; green `:22-50`; checksum `:418-437`. Corrected in SPEC §10.
- **M2** `format_requires_template_tests::partition_is_exact` (`:838-846`) is a LOGIC test (not compile-forced) → must add `Descriptor` to the passthrough array or it silently leaves it uncovered.
- **M3** round-trip recipe must EXCLUDE taproot from the from-import-json leg (`run_from_import_json` refuses taproot `:672-682`); taproot reaches `--format descriptor` only via direct `--descriptor` passthrough.
- **M4** the §2 comment said "auto-derefs to &str" but `format!("{}")` uses `Display` (`mod.rs:453-457`) — both work; aligned the comment.

## Verification ledger (confirmed TRUE)
- **Load-bearing:** `CheckedDescriptor` has an explicit `Display` (`mod.rs:453-457`) → `format!("{}", canonical_descriptor)` yields the canonical multipath `<descriptor>#<checksum>` (fixture `mod.rs:529` = `wpkh([…/84'/0'/0']xpub…/<0;1>/*)#tk4vnxy8`). HOLDS.
- **Input-path coverage:** all three populate `canonical_descriptor` (`--descriptor` `:328-342`; `--template`+`--slot` `:418-427`; from-import-json `:692,728`). from-import-json reaches the same emit dispatch (`:777`) → DescriptorEmitter. Single-sig + wsh-multisig both reach it.
- **Multisig:** dropping green's refusal is correct — multisig builds a populated canonical_descriptor (not an error) before any emitter runs.
- **collect_missing empty + ignored flags:** valid (`if !missing.is_empty()` skips refusal); `--range`/`--timestamp` only consumed by BitcoinCoreEmitter; no top-level error on unused flags (green precedent). `--output`/stdout format-agnostic.
- **Lockstep:** GUI `EXPORT_FORMATS += "descriptor"` is the ONLY schema change; import sniff list `:1989-1998` correctly untouched (export-only). Manual flag-coverage (`lint.sh:84`) is flag-NAME-only → `--format` VALUES don't gate it (manual value-list update is mirror-discipline, not a hard gate). MINOR/v0.42.0 justified; one-cycle scope right.

## Verdict rationale
RED only on the two mechanical compile-stoppers (C1 missing `extension()` + double-`\n`; I1 missed 5th match site). Folded both + M1-M4 → persist → re-dispatch (folds can drift). Expect GREEN round 1.
