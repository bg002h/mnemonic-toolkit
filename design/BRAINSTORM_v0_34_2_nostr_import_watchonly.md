# BRAINSTORM / DESIGN — `mnemonic nostr --import` (read-only Bitcoin Core importdescriptors) — v0.34.2

**Date:** 2026-05-22
**Source baseline:** branch `v0.34.2-toolkit-hygiene` tip (`d6b24af`); code identical to `origin/master` `1d6436d` (the only delta is `A`'s docs-only FOLLOWUPS closes). Citations verified against this state.
**Status:** design approved-in-direction (2026-05-22); pre-spec-review. Next: spec self-review → user review → writing-plans → opus R0 (mandatory) → implement.
**SemVer:** PATCH → **v0.34.2** (additive flags on the existing `nostr` subcommand). Likely ships alongside `A` (the 4 FOLLOWUPS hygiene closes already on this branch).

---

## §0 — Motivation & scope

Continuation of the v0.34.0 `mnemonic nostr` key-wrapper. Today, for a nostr key we emit the x-only hex, a watch-only descriptor string, an address, and (for `nsec`) a WIF — but **no ready-to-paste Bitcoin Core import recipe**. `export-wallet --format bitcoin-core` already emits a read-only `importdescriptors` recipe for HD/xpub wallets (`wallet_export/bitcoin_core.rs:42`), but nothing does it for a raw nostr key.

**Scope (this cycle): READ-ONLY (watch-only) importdescriptors on `mnemonic nostr` only.**

**Explicitly deferred to a future cycle (FOLLOWUPs filed, see §6):**
- **Spending** importdescriptors (private `wpkh(<WIF>)`/`tr(<WIF>)` descriptors). Requires a secret-bearing surface; `export-wallet` is **watch-only by definition** (`wallet_export/mod.rs:58` `REFUSAL_SECRET_INPUT`; refuses phrase/entropy/xprv/wif slots) so it CANNOT host spending — the spending surface will be `nostr` (nsec) and/or `convert` (wif/xprv).
- `export-wallet` spending mode (infeasible there — see above).

---

## §1 — CLI surface

```
mnemonic nostr <KEY-INPUT> [--script-type T | --all-script-types]
                           [--import readonly] [--timestamp <now|UNIX>]
                           [--network N] [--json]
```

- **`--import <readonly>`** — value-valued. Accepts `readonly` now. `spending` and `both` are **reserved**: the value parser recognizes them but returns a clean error (`"--import: spending/both is deferred to a future cycle; only 'readonly' is supported in v0.34.2"`). This keeps the flag **forward-compatible** — the future spending cycle enables those values with NO flag-shape break.
- **`--timestamp <now|UNIX_SECONDS>`** — Bitcoin Core `importdescriptors` rescan anchor. **Default `0`** (rescan from genesis — the right default for importing an *existing* key to discover its funds). Reuses `export-wallet`'s `parse_timestamp` (`export_wallet.rs:216`) + `TimestampArg` type. Only meaningful with `--import`.
- Both flags are inert without `--import` (a bare `--timestamp` with no `--import` is accepted but unused — or warn; decide in plan).

---

## §2 — Behavior

When `--import readonly` is set, append an `importdescriptors` recipe built from the **watch-only** descriptor(s) (`descriptor_for(xonly, script_type)` — the pubkey-based `wpkh(02…)`/`tr(<xonly>)`/etc., NOT the WIF). Works identically for `--pubkey` and `--secret` (both derive the same x-only pubkey; the WIF, if any, is untouched).

- **Single `--script-type`:** one import entry.
- **`--all-script-types`:** ONE `importdescriptors` array containing all four watch-only descriptors (paste once to watch all four address types).
- Each entry: `{"desc":"<descriptor>#csum","active":false,"internal":false,"timestamp":<ts>}`.
  - `active:false` — a single watched address, not a ranged receiving descriptor Core hands out from.
  - `internal:false` — single key, not change.
  - **no `range`** — raw single key is non-ranged (unlike export-wallet's HD `<0;1>/*`).

Example:
```
$ mnemonic nostr --pubkey npub10elf… --script-type p2wpkh --import readonly
  x-only:      7e7e9c42…df4e
  script-type: p2wpkh
  descriptor:  wpkh(02…)#csum
  address:     bc1q…
  import:      importdescriptors '[{"desc":"wpkh(02…)#csum","active":false,"internal":false,"timestamp":0}]'
```

---

## §3 — Architecture (shared helper)

Generalize the existing `wallet_export::bitcoin_core::format_bitcoin_core_importdescriptors` into a shared helper usable by BOTH `export-wallet` (ranged HD) and `nostr` (non-ranged single-key):
- Today it always emits `"range":[lo,hi]` + splits multipath `<0;1>` into receive/change. Generalize so a **non-ranged, single descriptor** entry omits `range` and uses caller-supplied `active`/`internal`.
- Likely shape: a small `core_import` module (or extend `wallet_export::bitcoin_core`) exposing `import_entry(desc: &str, active: bool, internal: bool, range: Option<(u32,u32)>, timestamp: TimestampArg) -> Value` and an array builder. `export-wallet` keeps its current ranged/multipath path; `nostr` calls the single-key path. **One JSON shape, no drift.**
- `nostr` builds the descriptor strings via the existing `nostr::descriptor_for` (already produces `#csum`-checked descriptors), then wraps each in an import entry.

---

## §4 — Output & secret handling

- Human-readable: an `import:` line (per the example). With `--all-script-types`, a single `import:` line whose array carries all four.
- `--json`: add an `import` field to the existing `NostrJson` (the importdescriptors array as structured JSON, not a string), present only when `--import` is set.
- **Secret-handling: NONE new.** The read-only recipe contains only the **public** descriptor — no WIF/secret on stdout beyond what `--secret` already emits. So no new argv/stdout advisory. (The deferred SPENDING recipe WOULD embed the WIF → that future cycle adds the secret-on-stdout handling.)

---

## §5 — SemVer & lockstep
- **SemVer PATCH → v0.34.2** (two additive flags `--import`, `--timestamp` on `nostr`).
- **MANDATORY GUI `schema_mirror`** — add `--import` (dropdown: `readonly`; or text) + `--timestamp` (text) to the `nostr` SubcommandSchema in `mnemonic-gui/src/schema/mnemonic.rs` + pin bump + paired GUI release. New flag NAMEs trip the gate.
- **MANDATORY manual** — `docs/manual/src/40-cli-reference/41-mnemonic.md` `nostr` section: document `--import readonly` + `--timestamp` + the import-line output + the importdescriptors paste-into-Core recipe.
- Neither flag is secret → no `flag_is_secret` change.
- New deps: none.

---

## §6 — Testing
- `--import readonly` → stdout contains an `importdescriptors '[…]'` line; the JSON parses; the array's `desc` equals the watch-only descriptor; `active:false`/`internal:false`/`timestamp:0`.
- `--all-script-types --import readonly` → one array with 4 entries (one per type).
- `--import spending` / `--import both` → clean refusal (exit 1, "deferred to a future cycle").
- `--timestamp now` / `--timestamp 1700000000` → reflected in the entry; default (no flag) → `0`.
- `--json --import readonly` → `import` field present + valid; absent without `--import`.
- export-wallet's existing bitcoin-core tests stay green after the shared-helper extraction (no behavior change to the ranged path).

---

## §7 — FOLLOWUPs to file (at implementation)
1. **`nostr-import-spending-descriptors`** (`v0.34+`): the deferred SPENDING importdescriptors on `nostr` (nsec → `wpkh(<WIF>)`/`tr(<WIF>)`) + the secret-on-stdout handling; enables `--import=spending|both`. Companion: `convert` spending import.
2. **`export-wallet-timestamp-default-zero`** (`v0.34+`): change `export-wallet`'s `--timestamp` default from `"now"` (`export_wallet.rs:117`) to `0`, for consistency with `nostr`'s `0` default — a **behavior change** to the emitted recipe (default rescan-from-genesis), so it needs its own SemVer consideration + a deliberate call.
3. **`timestamp-zero-default-docs-sweep`** (`v0.34+`): update all documentation that states/implies `--timestamp` defaults to `now` (manual + any SPEC) to reflect the `0` default once #2 lands.

---

## §8 — Open items for the plan phase
1. Confirm `parse_timestamp`/`TimestampArg` are reusable from `nostr` (pub-visibility from `cmd/export_wallet.rs`), or relocate to a shared module.
2. Whether a bare `--timestamp` without `--import` warns or is silently inert.
3. Exact `--import` dropdown representation in gui-schema (value enum `[readonly]` vs plain text — mirrors how `parse_*_arg` custom parsers surface in gui-schema; cf. `--script-type` emits as `text`).
