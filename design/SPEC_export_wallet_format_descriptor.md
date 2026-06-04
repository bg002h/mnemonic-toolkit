# SPEC — `mnemonic export-wallet --format descriptor` + concrete↔bundle round-trip recipe

**Status:** draft (pre-R0). **Target:** mnemonic-toolkit **v0.42.0** (SemVer MINOR) + paired **mnemonic-gui v0.23.0**.
**Source SHA (re-grep at impl):** toolkit `a26377e` (master, v0.41.0). Recon: `cycle-prep-recon-export-wallet-format-descriptor.md`.
**Decisions (autonomous, per recon):** one-line multipath descriptor; format supports single-sig AND multisig; all input paths; name `descriptor`; SemVer MINOR. No crates.io component (toolkit + GUI both tag-only; no sibling-codec change).

---

## §1. Feature

Add a `descriptor` value to `mnemonic export-wallet --format` that emits the **bare loadable canonical output descriptor string with its BIP-380 `#checksum`** on stdout (or `--output <file>`) — no `importdescriptors`/`bip388`/wallet-file JSON wrapper. This is the unwrapped sibling of the descriptor the toolkit already binds + emits inside `--format bitcoin-core` (importdescriptors) / `bip388` / `green`.

This completes the constellation's concrete-descriptor in/out at the toolkit/full-bundle layer: the **IN** direction shipped as A1 (`bundle --descriptor`); the **OUT** direction emitted the concrete descriptor only wrapped — `--format descriptor` adds the bare emit. (`md1` is keyless-template by design, so a concrete descriptor is inherently a bundle-level artifact = md1 template + mk1 xpubs; this is a toolkit feature, never an `md`-cli one.)

## §2. The `DescriptorEmitter`

`EmitInputs.canonical_descriptor` (`wallet_export/mod.rs:464,470`) is a `CheckedDescriptor` that ALREADY carries the validated BIP-380 8-char `#checksum` (`mod.rs:427-433`) and is the canonical **multipath `<0;1>` form** (e.g. `wpkh([5436d724/84'/0'/0']xpub…/<0;1>/*)#tk4vnxy8`). So the emitter is trivial — modeled on `green.rs:26-44` MINUS green's multisig-refusal + text-wrapper:

```rust
// crates/mnemonic-toolkit/src/wallet_export/descriptor.rs (NEW)
//! Bare canonical descriptor emitter: `<descriptor>#<checksum>` on one line,
//! no wallet-file wrapper. Works for single-sig AND multisig (unlike `green`,
//! which is Green-wallet-targeted and refuses multisig). The descriptor + its
//! BIP-380 checksum are already computed in `EmitInputs.canonical_descriptor`.
use super::{EmitInputs, MissingField, WalletFormatEmitter};
use crate::error::ToolkitError;

pub(crate) struct DescriptorEmitter;

impl WalletFormatEmitter for DescriptorEmitter {
    fn collect_missing(_inputs: &EmitInputs) -> Vec<MissingField> { Vec::new() }
    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
        // CheckedDescriptor auto-derefs to &str and carries the #checksum.
        Ok(format!("{}\n", inputs.canonical_descriptor))
    }
}
```
Re-grep `green.rs` + `mod.rs` at impl to match the exact trait/`MissingField`/`CheckedDescriptor` shapes (the block above is field-accurate vs the recon but confirm). Register `mod descriptor;` + `pub(crate) use descriptor::DescriptorEmitter;` in `wallet_export/mod.rs` alongside the other emitters.

## §3. Dispatch + enum (exhaustive `match`, no `_` — each arm forced)

- `cmd/export_wallet.rs:22-41` `enum CliExportFormat`: add `#[value(name = "descriptor")] Descriptor,`.
- `run()` collect_missing `match args.format` (`:504-514`): add `CliExportFormat::Descriptor => (DescriptorEmitter::collect_missing(&inputs), "descriptor"),`.
- `run()` emit `match args.format` (`:523-555`): add `CliExportFormat::Descriptor => DescriptorEmitter::emit(&inputs),`.
- `run_from_import_json()` collect_missing `match args.format` (`:756-765`) + its emit `match`: add the same two arms. (Re-grep — there are ~4 `match args.format` sites total; the compiler's non-exhaustive error enumerates any missed one.)
- Import the emitter at the dispatch sites (mirror `GreenEmitter` imports).

## §4. Behavior decisions (settled)

- **One multipath line** (`<descriptor>#<checksum>\n`) — the canonical form already computed + what `green` emits; modern Core 26+/Sparrow accept multipath. NOT two single-path lines (a future `--split-multipath` could add that; out of scope).
- **Single-sig AND multisig** — `DescriptorEmitter` does NOT refuse multisig (unlike green). The `canonical_descriptor` is built for both.
- **All input paths** — `--template`+`--slot @0.xpub=`, `--descriptor` passthrough, and `--from-import-json` all populate `canonical_descriptor` → `--format descriptor` works from any. (This makes the round-trip recipe realizable.)
- **`collect_missing` empty** — a bare descriptor needs no `--wallet-name`/`--range`/`--timestamp`. Those flags are silently ignored for this format (like green). Confirm the run() path tolerates the empty missing-set + doesn't error on unused range/timestamp.
- **Output** — stdout by default; `--output <file>` honored via the existing write path (no special-casing).

## §5. Lockstep (paired GUI v0.23.0)

- **GUI `schema_mirror` (value-enum):** `mnemonic-gui/src/schema/mnemonic.rs` `EXPORT_FORMATS` const (`:61-72`, consumed by `FlagKind::Dropdown(EXPORT_FORMATS)` `:803`) → add `"descriptor"`. **Do NOT** add it to the inbound/import sniff list (`:1990`) — `descriptor` is export-only. Confirm with `mnemonic gui-schema` against the new binary that the export-wallet `--format` dropdown values match.
- Paired GUI cycle bumps the toolkit pin to the new tag (the `tests/pin_coherence.rs` guard enforces Cargo↔pinned-upstream lockstep) + the schema const → GUI **v0.23.0**.
- **Manual:** `docs/manual/src/40-cli-reference/41-mnemonic.md` export-wallet `--format` value list; `docs/manual/src/30-workflows/37-wallet-export.md` round-trip recipe (§9).
- No sibling-codec (ms/md/mk) change; no crates.io publish.

## §6. Phasing (mandatory opus R0: SPEC + plan + per-phase + end-of-cycle; 0C/0I; re-dispatch after every fold; persist to design/agent-reports/)

- **P1 (code):** enum value + `DescriptorEmitter` + the dispatch arms + tests. TDD.
- **P2 (docs + GUI lockstep + release):** manual recipe (§9) + `--format` value list; paired GUI v0.23.0; toolkit v0.42.0 version bump (Cargo.toml + 2 README markers + CHANGELOG + install.sh self-pin + Cargo.lock relock + readme_version_current); `make audit` EXIT 0.

## §7. Tests (P1)

1. **single-sig:** `export-wallet --template bip84 --slot @0.xpub=<acct-xpub> --format descriptor` → exactly `wpkh([fp/84'/0'/0']xpub…/<0;1>/*)#<8char>\n`; assert the line parses as a valid BIP-380 descriptor + the checksum is present + correct (round-trip via miniscript parse, or assert `#` + 8 alnum).
2. **multisig:** `--template wsh-sortedmulti --threshold 2 --slot @0.xpub= --slot @1.xpub= --format descriptor` → `wsh(sortedmulti(2,[..]xpub…/<0;1>/*,[..]…))#<8char>` (NOT refused, unlike green).
3. **round-trip (the headline):** `bundle --descriptor '<concrete>' …` → import-json envelope (or cards) → `export-wallet --from-import-json <env> --format descriptor` emits a descriptor whose canonical form == the original `<concrete>` (modulo checksum recompute). Cover single-sig + multisig.
4. **flags ignored:** `--range`/`--timestamp`/`--wallet-name` with `--format descriptor` don't error (silently ignored).
5. **`--output`:** writes the one-line descriptor to the file.
6. **GUI parity (in the GUI cycle):** `schema_mirror` EXPORT_FORMATS includes `descriptor` matching `gui-schema`.
Gate per phase: `cargo test -p mnemonic-toolkit --no-fail-fast` (0 fail) + `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`. No `cargo fmt`.

## §8. SemVer

MINOR → toolkit **v0.42.0** (new user-facing `--format` value + schema_mirror lockstep). GUI **v0.23.0** (paired schema bump + pin to v0.42.0).

## §9. Round-trip recipe (B — docs, authored in P2 after A lands)

`docs/manual/src/30-workflows/37-wallet-export.md` — a "Concrete descriptor ↔ bundle round-trip" section:
- **md1 is keyless by design** → a concrete descriptor is a bundle-level artifact (md1 template + mk1 xpubs); this is why it's a toolkit (not `md`-cli) feature.
- **IN:** `mnemonic bundle --descriptor 'wsh(sortedmulti(2,@0,@1))' --slot @0.xpub=… --slot @1.xpub=… …` (A1, descriptor → cards).
- **OUT:** `mnemonic export-wallet --from-import-json <envelope> --format descriptor` → the bare `<descriptor>#<checksum>` (bundle → concrete descriptor).
- Distinguish `--format descriptor` (raw, any policy) from `--format green` (Green-wallet text, single-sig only).
- All commands verified end-to-end + `make audit` EXIT 0 (authored after A so they run).

## §10. Citations (re-grep at impl, SHA a26377e)

`cmd/export_wallet.rs:22-41` (CliExportFormat), `:504-514`/`:523-555`/`:756-765`+ (dispatch sites); `wallet_export/mod.rs:395-397` (WalletFormatEmitter trait), `:464,470` (EmitInputs.canonical_descriptor: CheckedDescriptor), `:427-433` (checksum validation); `wallet_export/green.rs:26-44` (emitter model); `mnemonic-gui/src/schema/mnemonic.rs:61-72` (EXPORT_FORMATS), `:803` (Dropdown), `:1990` (import sniff — NOT touched); manual `30-workflows/37-wallet-export.md` + `40-cli-reference/41-mnemonic.md`.
