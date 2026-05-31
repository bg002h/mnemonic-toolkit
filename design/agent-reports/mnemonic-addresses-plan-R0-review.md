# R0 Architect Review — IMPLEMENTATION_PLAN_mnemonic_addresses.md (toolkit 0.38.0)

Reviewer: feature-dev:code-reviewer (opus). Reviewed against the GREEN spec + live source on branch
`mnemonic-addresses-subcommand`. Read the real crate structure, cited helpers, bitcoin/clap APIs.

## Critical

**C1 — `address_render.rs` placed in `lib.rs` will not compile (module-crate mismatch).**
The plan's Task 0.1 + file-structure say "Modify `src/lib.rs` — `mod address_render;`", and the module
body does `use crate::cmd::convert::ScriptType;` + `use crate::network::CliNetwork;`. But `src/lib.rs`
declares only a public set (`mlock`, `seedqr`, `slip39`, …) with **no `mod cmd`/`mod network`/`mod
language`** — those are bin-only (`main.rs:5/17/18`). A lib-level `address_render` cannot name
`crate::cmd::convert::ScriptType`/`crate::network::CliNetwork`, nor be referenced as
`crate::address_render` by its bin consumers. **Fix:** make it a **bin module** — `mod address_render;`
in `src/main.rs` (not lib.rs); delete the "(or main.rs)" ambiguity. Then all reachability holds.

## Important

**I1 — `--count 2147483648` "SUCCEEDS" is not a runnable CLI test (2^31 derivations / 8 GB Vec).**
The ceiling LOGIC is correct, but `resolve_indices` eagerly `(0..2^31).collect()` (8 GB) + 2.1B derive
iterations → OOM/hang. **Fix:** test the boundary at UNIT level (`resolve_indices(Some(2_147_483_648),
None).is_ok()`, `…(2_147_483_649) → Err`), NOT a CLI derive. CLI tests only the rejection cases
(2147483649, range 0,2147483648). Update spec §5 cell 5 wording from "command SUCCEEDS" to
"ceiling guard accepts (unit-level)".

**I2 — Seed-flow snippet won't compile: `Mnemonic::parse_in` arity + Option-typed args to value params.**
`bip39::Mnemonic::parse_in` takes `(Language, &str)` — snippet passes one arg. Canonical form:
`Mnemonic::parse_in(language.into(), phrase).map_err(ToolkitError::Bip39)?.to_entropy()`. `language` is
`Option<CliLanguage>` (needs `unwrap_or_default().into()`); `derive_bip32_from_entropy` needs
`passphrase: &str` + `language: CliLanguage` (not Options) + network defaulted to Mainnet for seeds.
**Fix:** resolve the Options to concrete `&str`/`CliLanguage`/`CliNetwork` before `parse_in`/
`derive_bip32_from_entropy`; show the compiling form.

## Minor
- **M1** — `is_json_mode` does not exist (verified); JSON is read internally via `args.json`. Drop the
  mention; dispatch arm is `Command::Addresses(args) => cmd::addresses::run(args, stdin, stdout, stderr)`.
- **M2** — "run mirrors convert's" imprecise: convert is 5-arg (`…, no_auto_repair`). Mirror the
  non-repair subcommands — `decode_address::run(args, stdin, stdout, stderr)` (`main.rs:151`, generic
  `<R: Read, W: Write, E: Write>`).
- **M3** — `parse_xpub` is shorthand → `Xpub::from_str(&value).map_err(…Bip32…)?` (decide on SLIP-0132
  `normalize_xpub_prefix` or document omission). `seedqr::decode` is a LIB module →
  `mnemonic_toolkit::seedqr::decode(value).map_err(|e| crate::cmd::seedqr::map_seedqr_error(e,&action))`
  (pattern at bundle.rs:481), NOT `crate::seedqr::decode`.
- **M4** — no task/cell for the inline-`--passphrase`/`--from <secret>=` argv-leak advisory (spec §3.1
  promises it). Reusable: `crate::secret_advisory::secret_in_argv_warning(stderr, flag, alt)`. Add an
  emission step + test cell, or scope out explicitly.

## Verified-clean
ScriptType/parse_script_type_arg pub; `#[arg(value_parser=parse_script_type_arg)]` on required
ScriptType valid clap; ChainSel/conflicts_with/passphrase-stdin valid; dedup call-sites enumerated +
correct (render convert:1291/:1343, address_search:87; network convert:1342, address_of_xpub:359);
`derive_bip32_from_entropy` arg order (derive_slot.rs:43); `DerivedAccount.account_xpub` (derive.rs:26);
`template_for` correct inverse; ceiling arithmetic exact (MAX_PLUS1=2^31, no overflow); resolution
primitives pub(crate); `ToolkitError::BadInput` exists exit 1, no new variant; per-phase TDD + review
gates present; §5 cells 1-12 mapped.

**VERDICT: RED (1C/2I)**

---

## Fold applied (controller, verified @ branch)
- **C1:** confirmed lib.rs has no mod cmd/network/language. Plan → `mod address_render;` in **main.rs**
  (bin module); lib.rs instruction removed.
- **I1:** spec §5 cell 5 + plan Task 3.1 → 2^31 boundary tested at UNIT level (resolve_indices), CLI
  tests only the reject cases.
- **I2:** Task 2.1 seed snippet rewritten with Option-resolution + correct `parse_in(language.into(),
  phrase).to_entropy()`.
- **M1** is_json_mode dropped. **M2** run mirrors decode_address (4-arg generic). **M3** Xpub::from_str
  + mnemonic_toolkit::seedqr::decode/map_seedqr_error. **M4** argv-leak advisory task + cell added.
