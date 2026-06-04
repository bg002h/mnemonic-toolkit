# cycle-prep recon — 2026-06-03 — export-wallet-format-descriptor

**Origin/master SHA at recon time:** `a26377e` (v0.41.0 + the all-single-sig manual recipe)
**Local branch:** `master`
**Sync state:** up-to-date (0 ahead / 0 behind)
**Untracked:** pre-existing scratch only (`cycle-prep-recon-*.md`, `CONTINUITY.md`, `feature-coverage-survey-*.md`, `.claude/`, `stderr*.txt`).

Slug verified: **none exists** — NET-NEW feature (proposed slug `export-wallet-format-descriptor`). Feature recon. **Headline:** the OUT half is a ~5-line emitter (the canonical descriptor + checksum is already computed); `green` is a near-exact model. The whole thing is SMALL; the only design question (multipath vs two-line) resolves cleanly to multipath.

---

## Feature recon — (A) `export-wallet --format descriptor` + (B) concrete↔bundle round-trip recipe

### (A) the bare `<descriptor>#<checksum>` emit — SMALL

- **`CliExportFormat` enum (`cmd/export_wallet.rs:22-41`)** — 10 values (bitcoin-core/bip388/coldcard/coldcard-multisig/jade/sparrow/specter/electrum/green/bsms). Add `#[value(name = "descriptor")] Descriptor`. — ACCURATE.
- **`WalletFormatEmitter` trait (`wallet_export/mod.rs:395-397)`** — `collect_missing(&EmitInputs) -> Vec<MissingField>` + `emit(&EmitInputs) -> Result<String, ToolkitError>`. A new `DescriptorEmitter` implements exactly these two. — ACCURATE.
- **`EmitInputs.canonical_descriptor: CheckedDescriptor` (`mod.rs:464,470`)** — a compile-time-checked descriptor that ALREADY carries the BIP-380 8-char `#checksum` (validated at `CheckedDescriptor::new`, `mod.rs:427-433`) and is the **multipath `<0;1>` form** (e.g. `wpkh([5436d724/84'/0'/0']xpub…/<0;1>/*)#tk4vnxy8`, `mod.rs:529`). So `DescriptorEmitter::emit` = `Ok(inputs.canonical_descriptor.to_string())` (+ a trailing newline to match green) and `collect_missing` = `vec![]`. **The computation is 100% done; the emitter is ~5 lines.** — ACCURATE.
- **`green` is the model (`wallet_export/green.rs:26-44`)** — `collect_missing(_inputs) -> vec![]`; `emit` refuses multisig then `Ok(format!("…{}", inputs.canonical_descriptor))`. The new `DescriptorEmitter` is `green` MINUS the multisig refusal + the green-specific text framing — i.e. just the bare descriptor. — ACCURATE.
- **Dispatch is an exhaustive `match args.format` (no `_` arm) at ~4 sites** — `run()` collect_missing (`:504-514`), `run()` emit (`:523-555`), `run_from_import_json()` collect_missing (`:756-765`) + its emit dispatch. Adding `Descriptor` FORCES an arm at each (the project's "no `_` → forces a decision" property). — ACCURATE.
- **Action:** add the enum value + `DescriptorEmitter` (mirror green, drop the multisig refusal) + the 4 dispatch arms. Cite SHA `a26377e`.

### Design questions for the brainstorm

1. **Multipath vs two single-path lines — RESOLVES to multipath.** `canonical_descriptor` is already the BIP-380 multipath `<0;1>` form, checksummed, and `green` emits exactly that. So `--format descriptor` emits ONE multipath `<descriptor>#<checksum>` line. (Modern Core 26+/Sparrow accept multipath. A future `--split-multipath` option could emit receive `#c1` + change `#c2` two-line, but default + simplest = the canonical multipath string. Brainstorm to confirm one-line-multipath as the v1 shape.)
2. **Single-sig AND multisig — YES both.** Unlike `green` (which refuses multisig), the generic `descriptor` format should emit the canonical descriptor for BOTH single-sig templates and multisig (the canonical_descriptor is built for both). So `DescriptorEmitter` does NOT refuse multisig. This makes the round-trip recipe (B) work for both.
3. **Input paths — all work.** `--template`+`--slot @0.xpub=`, `--descriptor` passthrough, AND `--from-import-json` (a full bundle) all populate `canonical_descriptor`, so `--format descriptor` works from any of them. The round-trip `bundle --descriptor` IN ↔ `export-wallet --from-import-json --format descriptor` OUT is realizable.
4. **`collect_missing` = empty** (a bare descriptor needs no `--wallet-name`/`--range`/`--timestamp`). Confirm the run() path tolerates an empty missing-set + ignores the range/timestamp flags for this format (a minor: should supplying `--range`/`--timestamp` with `--format descriptor` warn/ignore? Recommend silently ignore, like green).
5. **Naming:** `descriptor` (clearest). Output to stdout (or `--output`), one line.

### (B) the concrete↔bundle round-trip recipe — DOCS

- Home: `docs/manual/src/30-workflows/37-wallet-export.md` (extend, next to the all-single-sig recipe just added at `a26377e`) + the `export-wallet --format` value list in `40-cli-reference/41-mnemonic.md`.
- Content: the toolkit-only / full-bundle concrete↔template story — **IN:** `bundle --descriptor 'wsh(sortedmulti(2,@0,@1))' --slot @0.xpub=… …` (A1, descriptor→cards); **OUT:** `export-wallet --from-import-json <envelope> --format descriptor` (bundle→bare concrete descriptor) — emphasizing md1 is keyless-by-design so the concrete descriptor is a bundle-level artifact (md1 template + mk1 xpubs), which is why this lives in the toolkit, not `md`-cli.
- The recipe's commands must be verified end-to-end + pass `make audit` (verify-examples is transcript-replay; a prose recipe with the new flag is fine once the flag exists — author B in the SAME cycle AFTER A lands so the commands run).

---

## Cross-cutting observations

1. **No slug exists** — net-new; the survey's "md concrete in/out" headline is the parent class (resolved-as-toolkit-feature; this is the last thin slice). File `export-wallet-format-descriptor` if not implemented immediately.
2. **A1 already shipped the IN direction** (`bundle --descriptor`, v0.38.1) + the OUT direction already emits the concrete descriptor WITH checksum, only WRAPPED (bitcoin-core importdescriptors / bip388 / green). This cycle adds the UNWRAPPED emit + documents the round-trip. So it completes a half-built capability, not a greenfield one.
3. **Lockstep — real schema_mirror (value-enum):** GUI `mnemonic-gui/src/schema/mnemonic.rs` `EXPORT_FORMATS` const (`:61-72`, consumed by the `FlagKind::Dropdown(EXPORT_FORMATS)` at `:803`) must gain `"descriptor"`. (The SEPARATE inbound/import list at `:1990` is for `import-wallet` sniff formats — `descriptor` is export-only, so it does NOT go there; confirm at impl.) Paired GUI cycle → **GUI v0.23.0** pinning the new toolkit tag; the new `tests/pin_coherence.rs` guard (GUI v0.22.0) enforces the Cargo/pinned-upstream lockstep. Manual `40-cli-reference/41-mnemonic.md` export-wallet `--format` values + `37-wallet-export.md`. No sibling-codec change.
4. **`green` overlap note:** `green` already emits essentially-bare descriptor text (with a 3-line green wrapper + multisig refusal). The new `descriptor` format is the un-wrappered, multisig-allowing generalization. Worth a one-line manual note distinguishing them (green = Green-wallet-targeted; descriptor = raw, any policy).

---

## Recommended brainstorm-session scope

- **SemVer:** new `--format` VALUE on an existing subcommand (additive value-enum). Per the cycle-prep rule that's PATCH, but it's a user-facing capability + schema_mirror lockstep → recommend **MINOR (toolkit v0.42.0)** (consistent with the ms1-slot / recent user-facing-value additions); PATCH (v0.41.1) defensible.
- **Decomposition — ONE small cycle, two phases:**
  - **P1 (code):** `CliExportFormat::Descriptor` + `DescriptorEmitter` (mirror green, drop multisig-refusal, emit `canonical_descriptor.to_string()`) + the ~4 dispatch arms + tests (single-sig + multisig + from-import-json round-trip + the BIP-380 checksum present). ~80-150 LOC incl tests.
  - **P2 (docs + lockstep):** manual recipe B (37-wallet-export.md + 41-mnemonic.md `--format` values) + paired GUI v0.23.0 (`EXPORT_FORMATS += "descriptor"`) + version bump + CHANGELOG. `make audit` EXIT 0.
  Both halves (A code + B docs) the user asked for fit one cycle; B's recipe is authored after A so its commands run + verify.
- **Lockstep flags:** GUI `schema_mirror` (EXPORT_FORMATS dropdown), manual mirror (40-cli-reference + 30-workflows). The leading discipline is the paired GUI PR; the lagging gate fires at the GUI pin bump (now guarded by pin_coherence).
- **Reference implementations to cite:** `wallet_export/green.rs:26-44` (the emitter model), `wallet_export/mod.rs:464-470` (`EmitInputs.canonical_descriptor: CheckedDescriptor`), `cmd/export_wallet.rs:504-555,756-765` (the dispatch sites). Source SHA `a26377e`.
- Mandatory opus R0 on the brainstorm spec + plan + per-phase + end-of-cycle (0C/0I before code; re-dispatch after every fold). The one open design question is the multipath-vs-two-line shape (recon recommends one-line multipath); single-sig+multisig + input-path coverage are settled.
