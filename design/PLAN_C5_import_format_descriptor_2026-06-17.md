# PLAN — C5: `import-wallet --format descriptor` (commented-descriptor intake), 2026-06-17

> Tier-2 item C5 from `design/PLAN_remaining_open_items_tiered_2026-06-16.md`, **re-scoped by the
> user** from "import `--format green`" to a GENERIC commented-descriptor import format (covers
> green's 3-line export + any hand-written/foreign commented descriptor). **Source SHA: toolkit
> `b15f5e6`** (HEAD == origin/master == tag mnemonic-toolkit-v0.57.1; citations grep-verified at
> write time). **MINOR → v0.57.1 → v0.58.0.** **Cross-repo:** toolkit + mnemonic-gui (schema dropdown
> value + pin bump). Toolkit git-tag only, no publish. R0 gate: **no code until R0 → GREEN (0C/0I).**

---

## Gap (precondition outcome)

There is no `import-wallet` door for a bare/commented concrete-descriptor text file. The toolkit's
own `export-wallet --format green` emits a 3-line file (2 leading `#`-comments + a
`<descriptor>#csum` line, `wallet_export/green.rs:41-44`), and `export-wallet --format descriptor`
emits a bare `<descriptor>#csum` — but neither can be re-imported: `import-wallet --format
bitcoin-core` expects JSON, not commented text (precondition checked — NOT doc-only). A user who
has a watch-only descriptor in a text file ("here is a `wsh(sortedmulti(...))` from my hardware
wallet, make cards") has no `import-wallet` entry point.

**Fix (user's choice — generic, not green-specific):** a new `import-wallet --format descriptor`
that reads a watch-only descriptor from text, tolerating leading `#`-comment lines + blank lines
(so it subsumes the green export AND hand-written/foreign commented descriptors), then flows the
descriptor through the EXISTING import pipeline → md1 bundle, watch-only. Supports **both singlesig
and multisig** (a descriptor string carries everything; the pipeline already handles both — unlike
green-*export*, which is singlesig-only by Blockstream's server-mediated multisig constraint).

## Citations (grep-verified @ `b15f5e6`)

| Surface | Location |
|---|---|
| import `--format` clap value list (GUI dropdown surface) | `cmd/import_wallet.rs:143-152` (`PossibleValuesParser` of 8 values; add `descriptor` alphabetically after `coldcard-multisig`) |
| parser dispatch | `cmd/import_wallet.rs:1157-1172` (`match format_str { … }`; add `"descriptor" => DescriptorParser::parse(&blob, stderr)?`) |
| `--blob` read | `cmd/import_wallet.rs:read_blob` (`-`→stdin, else `fs::read`) called at `:424` |
| parser trait | `wallet_import/mod.rs:44-53` `WalletFormatParser { fn sniff(&[u8])->bool; fn parse(&[u8], &mut dyn Write)->Result<Vec<ParsedImport>,_> }` |
| return type | `wallet_import/mod.rs:298-317` `ParsedImport { descriptor: md_codec::Descriptor :303, original_descriptor: String :309, cosigners, network, threshold, provenance }` |
| provenance enum (exhaustive, alphabetical) | `wallet_import/mod.rs:70-140`; ~8 accessor `match self` blocks `:146-285` |
| descriptor-string → placeholder + keys (the import-parser glue) | `wallet_import/pipeline.rs:235-305` `concrete_keys_to_placeholders` + `extract_origin_components :52-84` (the specter path — R0-r1 M2). NOTE: `descriptor_concrete_to_resolved_slots :311-350` is NOT the path (yields neither network nor threshold; see impl step 3). |
| clone template (simplest parser) | `wallet_import/specter.rs:156-317` (`parse`: checksum-verify → `concrete_keys_to_placeholders` → `parse_descriptor` → slots → `ParsedImport`); threshold `extract_threshold_local :417-433`; network `network_from_origins :370-397` |
| sniffer (explicit-only model) | `wallet_import/sniff.rs:84-111` (`votes` array `:94-103`); BSMS-encrypted explicit-only precedent `import_wallet.rs:220-223` |
| comment/blank strip precedent | `wallet_import/descriptor_intake.rs:216-220` (`.lines().map(trim).filter(!empty)`) |
| GUI dropdown values | `mnemonic-gui/src/schema/mnemonic.rs:2333-2342` `IMPORT_WALLET_FORMATS` (add `"descriptor"`); `EXPORT_FORMATS` already has it `:108` |
| GUI schema_mirror gate | `mnemonic-gui/tests/schema_mirror.rs:91-121` — **flag-NAMES only, NOT dropdown values** (so the new value is discipline-enforced, not gate-caught) |
| GUI toolkit pin | `mnemonic-gui/Cargo.toml:42` + `pinned-upstream.toml:22` (both `mnemonic-toolkit-v0.56.0` → bump to `v0.58.0`) |
| manual import `--format` row (STALE — lists 2 of 8) | `docs/manual/src/40-cli-reference/41-mnemonic.md:1087` |
| manual foreign-formats chapter | `docs/manual/src/45-foreign-formats.md` |
| manual lint (flag-NAMES only, not values) | `docs/manual/tests/lint.sh:84-96` |
| per-format test template | `tests/cli_import_wallet_sparrow.rs` (clone); green fixture shape `tests/export_wallet/green_descriptor.txt` |

## Design decisions (locked; R0 to vet)

1. **Format value name = `descriptor`** — symmetric with `export-wallet --format descriptor`. No clap
   collision (the `--descriptor` FLAG on bundle/export/verify is a different surface). Subsumes the
   original `green` framing (green's output is just comments + a descriptor).
2. **Singlesig AND multisig** — the pipeline (`concrete_keys_to_placeholders` + `parse_descriptor` +
   `extract_origin_components`) already parses `wsh(sortedmulti(...))` (Specter multisig path). A
   descriptor carries threshold/cosigners; no Blockstream-style constraint. (Watch-only out.)
3. **Explicit-only (no auto-sniff)** — mirror BSMS-encrypted. `DescriptorParser::sniff` returns
   `false` always and is NOT added to `sniff_format`'s `votes` array (`sniff.rs:94-103`); a bare
   descriptor sniffs as `NoMatch` today (no false-positive). `--format descriptor` is REQUIRED.
4. **`ImportProvenance::Descriptor` — UNIT variant** (no source metadata; a bare descriptor has no
   wallet name). Alphabetical slot between `ColdcardMultisig` and `Electrum` (`mod.rs:99↔100`).
   Add `Self::Descriptor => None` to each of the ~8 exhaustive accessor blocks (`:146-285`) — NO new
   accessor, NO new metadata type. (CLAUDE.md alphabetical discipline makes this mechanical.)
5. **Comment-strip semantics:** strip FULL-LINE leading `#`-comments + blank lines; require EXACTLY
   ONE remaining non-comment line = the descriptor (refuse 0 with "no descriptor line", refuse 2+
   with "expected a single descriptor"). Inline trailing `# …` on the descriptor line is NOT
   stripped (the `#` after the descriptor body is its BIP-380 checksum — do not corrupt).
6. **Checksum is TOLERANT, not required (R0-r1 M1).** Mirror `bundle --descriptor`: call
   `miniscript::descriptor::checksum::verify_checksum` which **validates-if-present, tolerates-absence**
   (BIP-380 tolerant mode — NOT "requires one"; the plan's earlier claim that specter requires a
   checksum was WRONG). A bad checksum → refuse; a missing checksum → accept (consistent with
   `bundle --descriptor`/`verify-bundle`, and necessary to subsume hand-written/foreign commented
   descriptors). The toolkit's own green/descriptor export always emits a `#csum`, so the round-trip
   case carries one regardless. (Add a checksum-LESS accept TDD cell.)
7. **Canonicalize/roundtrip = OUT of scope (graceful skip).** The `canonicalize_<fmt>` dispatch
   (`import_wallet.rs:1420-1432`) has a `_ => None` fallback → the new format simply has no
   byte-exact/semantic roundtrip section in `--json`. (A trivial `canonicalize_descriptor` is a
   cheap fast-follow, not v1.)
8. **Error variant (R0-r1):** parser internal parse errors use `ToolkitError::ImportWalletParse`
   (specter's convention); `ToolkitError::BadInput` for the strip/arity refusals. NEITHER is new → no
   alphabetical-sort obligation.

## Implementation

**New `crates/mnemonic-toolkit/src/wallet_import/descriptor.rs`** (mod decl in `wallet_import/mod.rs`
alphabetically). `DescriptorParser` impl `WalletFormatParser`:
- `sniff(_) -> false` (explicit-only).
- `parse(blob, stderr)`:
  1. `strip_comments(blob)` → the single descriptor line (full-line `#`/blank strip; arity check; `#`-strip is NEW — `descriptor_intake.rs:216-220` is a blank-strip-only precedent, R0-r1 M3).
  2. `verify_checksum` (tolerant: validate-if-present, strip suffix; mirror `specter.rs:217-222`).
  3. **The explicit specter sequence (R0-r1 M2 — locked; do NOT use `descriptor_concrete_to_resolved_slots`, which yields neither network nor threshold and carries a `--descriptor`-flavored error prefix):** `concrete_keys_to_placeholders(body)` → `parse_descriptor::parse_descriptor(...)` → `extract_origin_components(&body, "descriptor")` → per-slot `finalize_slot_fields` → `validate_watch_only_resolved`.
  4. `network` (mirror `specter.rs::network_from_origins :370-397`), `threshold` (mirror `specter.rs::extract_threshold_local :417-433`).
  5. Return `vec![ParsedImport { descriptor, original_descriptor: <raw w/ checksum>, cosigners, network, threshold, provenance: ImportProvenance::Descriptor }]`.

**Wire (toolkit):** clap value (`:143-152`), dispatch arm (`:1157-1172`), the explicit-format block
(`:512`+, no sniff-mismatch needed since explicit-only — confirm the `Some("descriptor")` arm just
returns the literal), `ImportProvenance::Descriptor` + accessor arms. NO new ToolkitError variant —
parser internal errors use `ImportWalletParse` (specter's convention), strip/arity refusals use
`BadInput` (Decision 8). The C1 unrestorable + older() advisories already fire on every
ParsedImport (`import_wallet.rs:1291/1295`) → the new format inherits them for free.

## TDD — tests are the deliverable

New `tests/cli_import_wallet_descriptor.rs` (clone `cli_import_wallet_sparrow.rs`):
1. **Singlesig round-trip parity.** `export-wallet --format descriptor` (or `green`) a singlesig
   wallet → feed the output to `import-wallet --format descriptor --blob -` → asserts a watch-only
   bundle (md1 emitted, `cosigners=1`, correct network). Confirms the green 3-line shape imports.
2. **Multisig.** A `wsh(sortedmulti(2,[fp/path]xpub…,[fp/path]xpub…))#csum` text file (with leading
   `#`-comments) → `import-wallet --format descriptor` → 2-cosigner watch-only bundle, threshold 2.
   (The key "more general than green" proof — green-export refuses multisig; descriptor-import accepts.)
3. **Explicit-only negative.** A bare descriptor file WITHOUT `--format descriptor` → auto-sniff →
   exit 1 "could not detect format" (NoMatch). Proves no greedy auto-sniff.
4. **Malformed negatives:** a file with NO descriptor line (only comments) → refuse "no descriptor
   line"; a file with TWO descriptor lines → refuse "expected a single descriptor"; a bad-checksum
   descriptor → refuse (checksum error). Each loud, exit non-zero.
5. **`--json` envelope:** `source_format: "descriptor"`, watch-only, no secret material on stdout.

**Module unit tests** in `descriptor.rs`: `strip_comments` over crafted inputs (leading comments +
blanks → the one line; 0 lines → Err; 2 lines → Err). Non-vacuity: revert the strip → the
green-3-line fixture (2 comment lines) fails to parse.

## Lockstep / SemVer — the FULL checklist (cross-repo)

- **MINOR → v0.57.1 → v0.58.0.** New `--format` value = new capability/dropdown-value (clap surface
  change). **This DOES have a GUI schema_mirror surface (dropdown value) + manual** — unlike C1/C2.
- **Toolkit version sites (C2-impl-review lesson):** `Cargo.toml`, BOTH READMEs (`README.md:13` +
  `crates/mnemonic-toolkit/README.md:9` — `readme_version_current` enforces both),
  `scripts/install.sh:32`, `fuzz/Cargo.lock` (`cargo update -p mnemonic-toolkit --precise 0.58.0`),
  main `Cargo.lock`, CHANGELOG `[0.58.0]`.
- **GUI paired update (mnemonic-gui) — SAME cycle (paired-PR rule; schema_mirror does NOT gate the
  value, so this is discipline not gate):** add `"descriptor"` to `IMPORT_WALLET_FORMATS`
  (`src/schema/mnemonic.rs:2336`-block, alphabetically after `coldcard-multisig`); bump toolkit pin
  `mnemonic-toolkit-v0.56.0 → v0.58.0` in `Cargo.toml:42` + `pinned-upstream.toml:22`. **The GUI pin
  is TWO toolkit releases behind (R0-r1 M4): v0.56.0, with v0.57.0 (C2) + v0.57.1 (C1) intervening.**
  Run the GUI `schema_mirror` against the v0.58.0 `MNEMONIC_BIN` — **MEASURE first:** diffing clap
  flag NAMES v0.56.0..v0.58.0 shows ZERO new flag names / subcommands (C2/C1 added only behavior +
  the new C5 VALUE) → the pin bump surfaces only the new `descriptor` VALUE, which schema_mirror's
  flag-NAME check ignores → expect schema_mirror PASS. Confirm the GUI builds + full GUI suite green.
  **GUI version bump = SemVer-MINOR (R0-r1 M5): v0.41.0 → v0.42.0** (a new user-visible dropdown value
  + pin bump is a feature surface, matching the v0.38-0.41 MINOR precedents). Tag the GUI per its
  ritual (the toolkit ships first; GUI paired-PR after the toolkit tag, mirroring prior cycles).
- **Manual:** `41-mnemonic.md:1087` import `--format` row — add `descriptor` (and backfill the 6
  missing values while here: the row is stale at 2/8). New `### descriptor` subsection in
  `45-foreign-formats.md`. Run `make -C docs/manual lint` (4 bins) — flag-coverage unaffected (value
  set not gated); confirm markdownlint/cspell/lychee/index pass.
- **fmt gate:** `cargo +1.95.0 fmt --all` then REVERT `mlock.rs` (g6).
- **FOLLOWUP:** there is NO filed FOLLOWUP slug (untracked-recon item). FILE one
  `import-wallet-format-descriptor` and mark it `resolved` in the shipping commit (record the
  re-scope from green + the singlesig+multisig generality).

## Execution

1. R0 architect review of THIS plan → GREEN (0C/0I), persist to
   `design/agent-reports/c5-import-format-descriptor-plan-r0-round{N}-review.md`. **R0 must vet:**
   checksum-tolerant-vs-required (decision 6); `descriptor_concrete_to_resolved_slots` vs explicit
   form (impl step 3); the unit-provenance accessor mechanics; the GUI version-bump (PATCH vs MINOR).
2. TDD (toolkit): write `cli_import_wallet_descriptor.rs` + module tests; confirm RED.
3. Implement parser + wiring + provenance. GREEN + full suite + clippy + manual lint + fuzz build.
4. Per-phase impl review → 0C/0I, persist.
5. Toolkit version bump v0.58.0 + ALL lockstep + manual + CHANGELOG + FOLLOWUP. fmt gate. Commit,
   tag `mnemonic-toolkit-v0.58.0`, push master, CI green.
6. GUI paired PR: dropdown value + pin bump v0.58.0 + GUI version bump + schema_mirror MEASURE + full
   GUI suite. Commit, tag (GUI), push, CI green.
