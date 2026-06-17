# cycle-prep recon — 2026-05-30 — Theme B read-only derivation arc (pieces 2/3/4, NOT sign-message)

**Scope (user-chosen):** build the "see it / use it after you recover it" surface — **batch address derivation (#2, toolkit), `ms derive` (#3, mnemonic-secret), `mk derive`/`mk address` (#4, mnemonic-key)** — but **NOT `sign-message` (#1)**. Firm product boundary: read-only PUBLIC derivation only (addresses / fingerprints / child xpubs — no nonce, no signing). See memory `feedback_no_signing_read_only_derivation_boundary`.

All three are **NET-NEW** (no filed FOLLOWUP slugs). Three independent subsystems / three repos → **three separate cycles**, each with its own R0 gate + version bump + lockstep. Unified by shared conventions (below).

---

## Piece #4 — `mk derive` / `mk address` (mnemonic-key, mk-cli)
- **Lowest friction:** mk-cli ALREADY deps `bitcoin = "0.32"` and imports `bip32::{DerivationPath,Fingerprint,Xpub}`. `Xpub::derive_pub`, `Address::p2pkh/p2wpkh/p2shwpkh/p2tr` all available. No new deps.
- A decoded `KeyCard` (`mk-codec/src/key_card.rs:24`) carries `xpub: Xpub`, `origin_path: DerivationPath`, `origin_fingerprint: Option<Fingerprint>`. `card.xpub.fingerprint()` computes the xpub's own fp (shown in `mk inspect`).
- **KEY DECISION — script type:** an mk1 card carries **NO script-type hint** (no SLIP-0132 prefix, no template). `mk address` MUST take `--script-type`/`--address-type` OR heuristic-default from origin_path prefix (44'→p2pkh, 49'→p2sh-p2wpkh, 84'→p2wpkh, 86'→p2tr). Multisig paths (48'/87') → single-key addresses are MEANINGLESS → refuse-with-advisory vs warn-and-proceed.
- **KEY DECISION:** `mk derive` (child xpub at relative path) and `mk address` (N addresses over chain/index) — two subcommands or one? `mk derive` only does UNHARDENED (xpub can't do hardened → refuse hardened in `--path`).
- House JSON: `schema_version` is **integer 1** (note: ms-cli uses string "1"); decode envelope `{schema_version, xpub, origin_fingerprint, origin_path, policy_id_stubs, chunks, code_variant}`.
- **SemVer:** mk-cli MINOR (→0.6.0), mk-codec unchanged (0.4.0). **Lockstep:** manual `docs/manual/src/40-cli-reference/44-mk-cli.md` (toolkit repo) + GUI `mnemonic-gui/src/schema/mk.rs` (pinned mk 0.3.1) + toolkit install.sh/manual.yml pin bump (sibling-pin-check.yml).

## Piece #2 — batch address derivation (mnemonic-toolkit, `convert`)
- `convert --to address` exists (single; `convert.rs:1272` phrase/entropy via mandatory full `--path`, `:1331` xpub via `derive_pub`). Script-type via `--script-type` OR `--template` inference (`resolve_script_type` :1577).
- **export-wallet `--range` is NOT reusable** — it's just Bitcoin-Core `importdescriptors` JSON metadata, no address loop (`wallet_export/bitcoin_core.rs`).
- The real address-range loop is `xpub_search/address_search.rs:55 scan_xpub_for_addresses` (chain∈{0,1}×index, `xpub.derive_pub(m/chain/i)` + `render_address`). **`render_address` (:35) is a PRIVATE DUPLICATE of convert's `build_address_from_xpub` (:1593).** → **pre-req refactor:** lift `pub(crate) render_address_from_xpub(secp, child, script_type, network)`; both callers use it (low-risk dedup).
- **KEY DECISION:** new flags on `convert` (`--count`/`--index`/`--change` or `--address-range A,B`) vs a dedicated `addresses` subcommand. convert's contract is single-in→single-out-per-`--to`; batch breaks it. Dedicated subcommand = cleaner scope, heavier lockstep.
- **KEY DECISION:** phrase/entropy batch needs `--template` (derive account xpub, then iterate chain/index) — different from today's mandatory-full-`--path` single-address flow; xpub batch is trivial. Single-sig only for v1 (multisig descriptor ranges out).
- **SemVer:** toolkit MINOR. `convert --json` wire-shape NOT gated by schema_mirror, BUT new clap FLAGS DO trigger GUI schema-mirror (`mnemonic-gui/src/schema/mnemonic.rs` CONVERT_FLAGS) + manual `41-mnemonic.md`.

## Piece #3 — `ms derive` (mnemonic-secret, ms-cli)
- **Highest-value gap, most friction:** ms-cli has `bip39 = "2"` but **NO `bitcoin` dep** → must add `bitcoin = "0.32"`. No fingerprint/xpub/xprv computation exists anywhere in ms-*. The canonical chain to mirror is toolkit `derive_slot.rs:32` (`to_seed`→`Xpriv::new_master`→`fingerprint`→`derive_priv`→`Xpub::from_priv`).
- **KEY DECISION (biggest) — output set & secret exposure:** master FINGERPRINT (4 bytes, non-secret, the "cheapest oracle") + account XPUB (non-secret, watch-only) are the read-only-public outputs. Master SEED (64B) + root XPRV are **secret-bearing** (equivalent to the entropy `ms decode` already emits — so NOT a new hazard class beyond key material, but a new stdout-secret surface). Options: (a) refuse seed/xprv entirely (fingerprint+xpub only), (b) gate behind explicit `--include-xprv`, (c) emit with the existing D9 "secret on stdout" stderr advisory. Note ms-cli's btcrecover footer currently says "`ms` does not perform derivation" — `ms derive` changes that framing.
- **KEY DECISION:** input DSL (`--from ms1=/phrase=/entropy=` toolkit-style vs ms-cli's existing positional+`--phrase`/`--hex`); path UX (`--path` vs `--account`+`--template`); template-surface width (single-sig subset vs full enum). New `--network` + `--passphrase`/`--passphrase-stdin` (ms-cli's first passphrase + dual-stdin guard). zeroize/mlock infra already present.
- **SemVer:** ms-cli MINOR (→0.4.3?), ms-codec unchanged. **Lockstep:** manual `43-ms.md` + GUI ms schema + toolkit pin.

---

## Cross-cutting conventions to SET in the brainstorm (consistency across all three)
1. **address-type flag name** — toolkit already inconsistent: `convert --script-type` vs `xpub-search --address-type`. PICK ONE for the new surfaces (recommend `--address-type` mk-side / keep `--script-type` toolkit-side? or unify). 
2. **batch count idiom** — `--count N` (enumerate) vs `--range A,B` (export-wallet style) vs `--gap-limit` (scan; WRONG semantics — these are enumeration not search).
3. **chain model** — `--change` bool / `--chain 0|1` / `--external-only` (xpub-search style); emit both chains or select.
4. **address JSON shape** — array of `{chain, index, address}` (consistent mk/toolkit).
5. **fingerprint terms** — master fp (`origin_fingerprint`) vs xpub-own fp vs parent fp — name them unambiguously (ms/mk both expose).
6. **No cross-repo code share** — codecs are upstream of the toolkit; each repo carries its own address-render copy (the sibling rule forbids depending on the toolkit). Accept the small duplication.

## Recommended sequence
**#4 mk (warm + zero new deps + sets address conventions) → #2 toolkit convert (mirrors mk conventions + dedup refactor) → #3 ms derive (new bitcoin dep + thorniest secret-exposure decision, settle xpub/fp conventions first).** Each is an independent MINOR; ship one, then the next. Value-order alternative: #3 first (fingerprint oracle is the starkest gap).

**SHAs at recon:** toolkit master `8b6bdf4`; mnemonic-secret + mnemonic-key at their current checkouts (verify at spec-write time).
