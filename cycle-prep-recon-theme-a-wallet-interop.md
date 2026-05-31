# cycle-prep recon — 2026-05-31 — Theme A "wallet interop"

**Repos at recon time:** descriptor-mnemonic `main` (md-codec 0.35.0 / md-cli 0.6.1, in sync); mnemonic-key `main`; mnemonic-toolkit `master ea8ba88`.
**Scope:** decompose Theme A; pick + scope the first sub-cycle.

Theme A = "round-trip the wallet you actually have" (interop / one-directionality). Decomposes into three INDEPENDENT sub-cycles:

- **A1 — concrete-descriptor round-trip.** md-cli (and possibly toolkit) ingest/emit of a *bare* concrete descriptor.
- **A2 — mk SLIP-0132 preservation.** mk1 normalizes ypub/zpub/Ypub/Zpub/upub/vpub → xpub/tpub, losing the variant.
- **A3 — `import-wallet --format green`.** export writes `green`, import can't read it (read↔write asymmetry).

---

## Key recon findings (correcting the 2026-05-30 survey)

1. **There is NO `WalletPolicy::from_descriptor` / concrete-descriptor parser in md-codec or md-cli.** (Survey claimed it exists — WRONG.) Only the outbound `to_miniscript_descriptor` exists (`descriptor-mnemonic/crates/md-codec/src/to_miniscript.rs:53`). `md encode` rejects anything lacking `@N` placeholders at `md-cli/src/parse/template.rs:86-89` (`lex_placeholders` → `"template contains no @i placeholders"`).

2. **`to_miniscript_descriptor(d, chain)` already produces a CONCRETE miniscript descriptor** (real `[fp/path]xpub/chain/*` keys from `d.tlv.pubkeys`, single-chain). `.to_string()` appends the BIP-380 `#checksum` for free. BUT `md decode` only calls `descriptor_to_template` (`md-cli/src/cmd/decode.rs:28`) → template-only output. No md-cli command runs the concrete pipeline end-to-end.

3. **The TOOLKIT already synthesizes loadable, checksummed concrete descriptors** in two paths:
   - `export-wallet --template` → `wallet_export/pipeline.rs:18-31` builds the string, `MsDescriptor::from_str(&s).to_string()` adds the `#checksum`; `CheckedDescriptor::new()` (`wallet_export/mod.rs:418`) enforces the suffix.
   - `export-wallet --descriptor` passthrough → re-parses + re-canonicalizes (`cmd/export_wallet.rs:643-679`).
   - `--format bitcoin-core` → `wallet_export/bitcoin_core.rs:42-86` splits `<0;1>` multipath + emits valid `importdescriptors` JSON with checksummed `desc`.

4. **`import-wallet` already parses concrete descriptors → `@N` + cosigners** via per-format parsers (`sparrow.rs`, `coldcard_multisig.rs`, `electrum.rs`, …) → `wallet_import::pipeline` (`[fp/path]xpub` → `@N`). `ParsedImport.original_descriptor` preserves the wire string incl. checksum.
   - **The narrow remaining gap:** no entry point accepts a **bare concrete-descriptor STRING**. `import-wallet` needs a recognized wallet-FILE format; `export-wallet/bundle --descriptor` needs explicit `@N` annotation (toolkit `parse_descriptor.rs:60-139` `lex_placeholders` for the `@N[fp/path]` variant). So "here is a raw `wsh(sortedmulti(...))` from my hardware wallet, make cards" has no door.

5. **mk SLIP-0132 (A2):** guard at `mnemonic-key/crates/mk-codec/src/bytecode/xpub_compact.rs:63-68` (`version_to_network` → `Error::InvalidXpubVersion` for non-xpub/tpub version bytes). The toolkit normalizes ypub/zpub → xpub *before* mk1 (`mnemonic-toolkit/src/slip0132.rs:66` `normalize_xpub_prefix`), so mk1 cards carry only `xpub`/`tpub`. Preserving the variant on-card needs either a new mk1 wire field (script-type hint → mk-codec MINOR + new test vectors) OR re-derive-prefix-on-emit from the descriptor's script type. Toolkit already has full normalize-in + `--xpub-prefix` re-emit-out infra (shipped v0.6.1/v0.7). **Open question: is on-card preservation actually wanted, or is normalize-in/re-emit-out sufficient?**

6. **green (A3):** `export-wallet` `CliExportFormat::Green` (`cmd/export_wallet.rs:39`), singlesig-only (FOLLOWUP `green-native-multisig-pending-server-support`). `import-wallet` dispatch (`cmd/import_wallet.rs:1128-1143`) handles 8 formats, `green` ABSENT. Green's singlesig export is a plain descriptor → may already be readable via `--format bitcoin-core` (could be doc-only/thin, needs a real Green export file to confirm).

---

## FOLLOWUPS / SemVer / lockstep

- No filed FOLLOWUP for "md concrete-descriptor ingest", "bare-descriptor ingest door", "mk1 SLIP-0132 preservation", or "import-wallet --format green". (SLIP-0132 *toolkit* normalize/re-emit entries are all SHIPPED/RESOLVED.)
- **A1 (md-cli ingest+emit):** md-codec wire UNCHANGED (front-end only in md-cli). md-cli MINOR. md-cli NOT in GUI schema → no schema-mirror. Manual mirror (`docs/manual/src/40-cli-reference/42-md.md`) + cli-subcommands.list required. md-cli on crates.io → publish.
- **A1-toolkit (bare-descriptor door), if in scope:** toolkit MINOR (new `--format descriptor` value or new flag). GUI schema-mirror + manual + sibling-pin lockstep.
- **A2:** mk-codec MINOR if new wire field (+ test vectors + toolkit re-pin + crates.io); or toolkit-only if re-emit-from-script-type.
- **A3:** toolkit MINOR (new `--format` value) + GUI schema-mirror + manual; possibly doc-only if bitcoin-core path already reads it.

---

## Recommended first sub-cycle

**A1 — concrete-descriptor round-trip**, the survey headline and the only truly one-directional flow. Cleanest leverage: md-codec wire is untouched; the work is an md-cli front-end (ingest: detect `[fp/path]xpub` → assign `@N` + extract triples → existing pipeline; emit: `md decode --as-descriptor --chain N` → `to_miniscript_descriptor(d,chain).to_string()`). Open scope question for the user: **standalone-`md` round-trip only, or also add the toolkit bare-descriptor ingest door** (the higher-value but larger, lockstep-heavy piece). A2/A3 are smaller, independent follow-ons.
