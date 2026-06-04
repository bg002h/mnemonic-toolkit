# export-wallet --format descriptor — SPEC R0 Review (round 1)
**Verdict:** GREEN (0C/0I)

Both round-0 blockers confirmed-fixed against source; no new C/I drift. 1 non-blocking Minor.

## C1 — CONFIRMED-FIXED
- Trait `WalletFormatEmitter` (`mod.rs:395-399`) requires exactly 3 methods (collect_missing, emit, extension); §2 now implements all 3; `extension()->"txt"` matches green/jade. Closes E0046.
- emit returns `Ok(inputs.canonical_descriptor.to_string())` (no trailing `\n`); the 4 dispatch tails each add exactly one `\n` (run stdout `:562`/file `:569`; from-import-json stdout `:816`/file `:823`) → single-newline output. Matches green.rs:41-44.
- `CheckedDescriptor` is `Copy`+explicit `Display` (`mod.rs:411,453-457`); `.to_string()` yields the canonical multipath `<descriptor>#<checksum>` (fixture `:529`). Imports mirror green.rs:19-20; `MissingField` (`:303`) used in signature. COMPILES.

## I1 — CONFIRMED-FIXED
- Grep proves EXACTLY 5 `match`-class sites: `:53` format_requires_template, `:504` run collect_missing, `:523` run emit, `:756` from-import-json collect_missing, `:777` from-import-json emit. §3 lists all 5.
- `Descriptor => false` (passthrough group `:55`) is correct: `true` at call site `:721` triggers `template_from_descriptor` (inject template); `false` keeps `template: None` (descriptor accepted without `--template`). Semantically right.

## M1-M4 — CONFIRMED-FIXED
M1 citation ranges accurate (≤1-line). M2 `partition_is_exact` (`:838-846`) is a logic test (literal arrays, not compile-forced) → adding `Descriptor` to the false array is genuinely needed. M3 taproot refusal `:672-682` fires for all formats on the from-import-json leg (incl descriptor); direct `--descriptor` passthrough (`run()` `:328-342→:523`) has no taproot refusal → §7.3/§9 correctly route taproot via passthrough only. M4 Display-not-Deref comment aligned.

## new-drift scan: clean
§2 compiles; `Descriptor => false` at `:55` syntactically/semantically right; GUI EXPORT_FORMATS the only schema change (import sniff `:1989-1998` untouched); both EmitInputs construction sites wrap `CheckedDescriptor::new(...)?` and both descriptor sources carry the `#csum` → construction succeeds.

## Minor (1, non-blocking)
- m5: inserting `Descriptor` into the enum shifts all downstream `:NNN` citations +1 — ordinary line-decay, already covered by §10's "re-grep at impl" mandate. Implementer re-greps post-insert.

## Verdict rationale
Round-0's two blockers mechanically + completely closed, verified against the live trait, green precedent, and CheckedDescriptor's Copy+Display. 5-site dispatch + the `false` passthrough classification confirmed by grep. M1-M4 verify. No new C/I. GREEN (0C/0I) — implementation may proceed.
