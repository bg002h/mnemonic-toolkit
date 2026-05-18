# PLAN — `mnemonic xpub-search` umbrella (v0.26.0 toolkit + v0.11.0 GUI lockstep)

**Working title:** `xpub-search-cycle`
**Worktree:** `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/xpub-search-brainstorm` (branch `worktree-xpub-search-brainstorm`)
**Plan-doc revision:** R1 (R0 folded: 2C + 9I + 8m + 2 user-direction expansions on input scope)

---

## Context

The toolkit currently consumes seed material (ms1/phrase) and emits xpubs (mk1), descriptors (md1), and bundles, but offers no surface that **runs the derivation in reverse**: given an xpub, descriptor, address, or passphrase, identify which derivation path / account / cosigner index / passphrase / child index under a seed produces that target. Four common Bitcoin recovery workflows want exactly this:

1. **"I have an xpub written down but I don't remember the path"** — given seed + xpub, find the path.
2. **"This wallet-import descriptor — is it from my seed, and at which account?"** — given seed + descriptor, find the cosigner role + account index.
3. **"I have an old address — which child of my xpub created it?"** — given xpub + address(es), scan child indices to a gap limit.
4. **"Did I use a BIP-39 passphrase with this xpub? Test it."** — given seed + passphrase + target xpub, verify match.

All four are deterministic searches over the same BIP-32 derivation graph. Existing primitives cover the cryptographic work; the new surface is mostly intake polymorphism + enumeration + comparison + UX.

**Target tags:** `mnemonic-toolkit-v0.26.0` + `mnemonic-gui-v0.11.0` (lockstep, same cycle).

---

## §1 — Locked decisions (from brainstorm dialogue + R0 folds)

| # | Decision | Source |
|---|----------|--------|
| L1 | CLI shape: **one umbrella subcommand** `mnemonic xpub-search <mode>` with 4 modes. Mirrors `seed-xor split\|combine` and `slip39 split\|combine` precedents (`cmd/seed_xor.rs:26-38`, `cmd/slip39.rs:66-78`). | brainstorm Q1 |
| L2 | Phase order: **foundation → composition**. P1 path-of-xpub → P2 account-of-descriptor → P3 address-of-xpub → P4 passphrase-of-xpub. Each is its own commit. | brainstorm Q2 |
| L3 | P1/P2/P4 default candidate path set: **BIP-44 / BIP-49 / BIP-84 / BIP-86 single-sig + BIP-48 multisig at script-type 1' / 2' / 3'** across account range `[min, max(min+N, max-account.unwrap_or(min+N))]`. Default `--min-account 0`, `--number-of-accounts 20`. Repeatable `--add-path <template>` extends the set. | brainstorm Q3 |
| L4 | P3 defaults: **gap-limit 20, both external (`0/*`) + internal (`1/*`) chains**. `--external-only` disables internal; address type inferred from SLIP-0132 prefix, `--address-type` overrides. **P3 adds P2PKH (BIP-44)** to `build_address_from_xpub` to round out the four standard address types. | brainstorm Q4 |
| L5 | P4 scope: **single-passphrase verification only**. `--passphrase`/`--passphrase-stdin` mandatory. No brute-force / candidate-list mode in MVP — file `xpub-search-passphrase-bruteforce` FOLLOWUP for v0.27+ if user demand surfaces. | brainstorm Q5 |
| L6 | GUI lockstep: **toolkit + GUI same cycle** — `mnemonic-gui-v0.11.0` ships 4 new panes + hub consuming `xpub-search` gui-schema entries. | brainstorm Q6 |
| L7 | **Seed intake (P1/P2/P4):** `--phrase` / `--phrase-stdin` + `--ms1` / `--ms1-stdin` mutex + **positional HRP-autodetect restricted to ms1 only** (mirrors v0.24.0 `repair::classify_hrp_prefix` at `repair.rs:84-108`). Plain BIP-39 phrase text has no HRP and is NOT positional-detectable; must be supplied via `--phrase`/`--phrase-stdin`. Auto-fire BCH repair gated to the `--ms1` decode-failure path only (BIP-39 phrase parse failure routes direct exit 1, no BCH primitive applies). | post-R0 user-Q1 + R1 C-1 fold |
| L8 | **P1/P4 target intake:** `--target-xpub <value>` accepts a bare xpub (any SLIP-0132 prefix) OR an mk1 card. Positional alternative permitted via HRP-autodetect. | post-R0 user-Q2 |
| L9 | **P2 descriptor intake:** `--descriptor <value>` accepts (a) external wallet-import literal-xpub descriptors (Sparrow/Specter/Core/Electrum/Liana/Caravan/Coldcard), (b) toolkit `@N[fp/path]` placeholder descriptors, (c) md1 card(s) (bech32, chunkable), (d) BIP-388 wallet-policy JSON. Shape auto-detected; explicit `--descriptor-from <node>=<value>` form available if auto-detect ambiguates. BSMS / Specter-JSON / Sparrow-JSON wrappers deferred to the wallet-import-multiformat cycle. | post-R0 user-Q2 |

---

## §2 — Surface map (R0-verified citations)

### 2.1 Existing primitives — reuse without modification

| Primitive | Location | Used by |
|-----------|----------|---------|
| `derive_master_seed(&Mnemonic, &str) -> Zeroizing<[u8; 64]>` | `derive_slot.rs:32-34` | P1, P2, P4 (BIP-39 passphrase plumbed) |
| `derive_bip32_from_entropy(entropy, passphrase, language, network, template, account) -> DerivedAccount` | `derive_slot.rs:42-91` | reference impl for candidate-iteration loop |
| `Xpriv::new_master`, `derive_priv`, `Xpub::from_priv`, `Xpub::derive_pub`, `Xpub::fingerprint` | `bitcoin::bip32` (Cargo dep) | All phases |
| `slip0132::normalize_xpub_prefix(s) -> Result<(String, Option<&'static str>), ToolkitError>` | `slip0132.rs:66-101` | All phases (target-xpub intake; variant signal is `&'static str` like `"zpub"`, not a typed enum) |
| `network_from_xpub(&Xpub) -> CliNetwork` | `cmd/convert.rs:1507-1512` | P3 (network inference from target xpub) |
| `MultisigPathFamily::default_origin_path(network, account, script_type) -> String` | `parse.rs:86-101` | **P1, P2, P4 BIP-48 multisig candidate paths** — `m/48'/coin'/account'/{1'\|2'\|3'}` (NOT `CliTemplate::derivation_path` which falls through to BIP-87 for multisig variants — R0 C1 fold) |
| `CliTemplate::derivation_path(network, account) -> DerivationPath` | `template.rs:75-78` | P1, P2, P4 single-sig (BIP-44/49/84/86) candidate paths ONLY |
| `parse_descriptor(input, keys, fingerprints) -> Result<MdDescriptor, ToolkitError>` | `parse_descriptor.rs:747-813` | P2 — toolkit `@N`-placeholder **shape detection only** (refused as non-searchable); plan does NOT call its full pipeline |
| `rust_miniscript::Descriptor::<DescriptorPublicKey>::from_str` + `for_each_key` / `iter_pk` | rust-miniscript crate (see precedent `wallet_export/pipeline.rs:163-204` `descriptor_to_bip388_wallet_policy`) | P2 — **the unified extraction primitive for all 3 searchable descriptor shapes** (R0 C2 + R1 I-1 + m-1 fold) |
| `md_codec::chunk::reassemble(&[&str]) -> Result<md_codec::Descriptor, _>` | `md_codec` dep (call shape per `synthesize.rs:894-895`) | P2 — md1 card(s) → `md_codec::Descriptor`; toolkit then walks the struct's fields (tree + TLV: fingerprints + xpubs) **directly** to extract cosigner `(idx, fingerprint, derivation_path, xpub)` tuples WITHOUT re-serializing through a canonical descriptor string (R2 C-R2-1 lock — `md_codec::Descriptor → String` is NOT a confirmed-existing API; tree-walk extraction is the fallback path that avoids the missing serializer entirely). |
| `CliTemplate::origin_path_str(network, account) -> String` | `template.rs:61-72` | P1/P2 template-name reporting |
| `read_stdin_to_string`, `read_stdin_passphrase` (re-exported) | `cmd/convert.rs:663-688` | All phases (stdin secret intake) |
| `secret_in_argv_warning(stderr, flag, alternative)` | `secret_advisory.rs:34-39` | P1, P2, P4 (argv-leak advisory) |
| `crate::repair::resolve_no_auto_repair(no_auto_repair) -> bool` | `repair.rs:355` | P1, P2, P4 — TTY-gate for `--ms1` decode-failure auto-fire only (R0 I2 fold) |
| `crate::repair::try_repair_and_short_circuit(kind, chunks, stdout, stderr, json) -> Result<(), ToolkitError>` | `repair.rs:962` (public + reusable helper) | P1, P2, P4 — BCH auto-fire on `--ms1` decode failure (R0 I2 fold) |
| `cmd/bundle.rs:1367-1388` `emit_default_path_notice` (private `fn`) | `cmd/bundle.rs` | P2 — **mirrored inline** (~6 lines); no extraction this cycle (R0 I7 lock) |
| `mlock::pin_pages_for(&[u8]) -> PinnedPageRange` | `mlock.rs:90-100` | P1/P2/P4 candidate-iteration loops |

### 2.2 Net-new primitives (this cycle)

| New surface | Location | Phase | Notes |
|-------------|----------|-------|-------|
| `XpubSearchArgs { command: XpubSearchCommand }` umbrella | `cmd/xpub_search/mod.rs` (new) | C1 (scaffolding in path-of-xpub commit) | Mirrors `cmd/seed_xor.rs:26-38` |
| `cmd/xpub_search/path_of_xpub.rs` (per-mode file — R0 I8 fold for parallel-disjoint commits) | new | P1 | `PathOfXpubArgs` + `run_path_of_xpub` |
| `cmd/xpub_search/account_of_descriptor.rs` | new | P2 | `AccountOfDescriptorArgs` + `run_account_of_descriptor` |
| `cmd/xpub_search/address_of_xpub.rs` | new | P3 | `AddressOfXpubArgs` + `run_address_of_xpub` |
| `cmd/xpub_search/passphrase_of_xpub.rs` | new | P4 | `PassphraseOfXpubArgs` + `run_passphrase_of_xpub` |
| `cmd/xpub_search/candidate_paths.rs` | new | P1 | `CandidatePathSet::build(min, n, max, add_paths) -> Vec<(template_name, path)>` — synthesizes BIP-44/49/84/86 + BIP-48 1'/2'/3' × accounts via `MultisigPathFamily::default_origin_path` for multisig, `CliTemplate::derivation_path` for single-sig, + `--add-path` substitution |
| `cmd/xpub_search/path_search.rs` | new | P1 | `match_xpub_against_paths(seed, candidates, target_xpub) -> Option<Match>` |
| `cmd/xpub_search/account_search.rs` | new | P2 | `match_descriptor_against_seed(seed, descriptor, candidates) -> Vec<CosignerMatch>` |
| `cmd/xpub_search/address_search.rs` | new | P3 | `scan_xpub_for_addresses(xpub, targets, gap_limit, chains, script_type) -> Vec<AddressMatch>` |
| `cmd/xpub_search/target_intake.rs` | new | P1 (extended in P2) | Polymorphic target shape parser: xpub \| mk1 \| descriptor \| md1 \| BIP-388 JSON |
| `cmd/xpub_search/seed_intake.rs` | new | P1 | Polymorphic seed shape parser: `--phrase` / `--phrase-stdin` / `--ms1` / `--ms1-stdin` / positional HRP-autodetect |
| `cmd/xpub_search/passphrase_verify.rs` | new | P4 | `verify_passphrase_for_xpub` (thin wrapper over `match_xpub_against_paths`) |
| `XpubSearchJson` serde-tagged enum (`tag = "mode"`) | `cmd/xpub_search/mod.rs` | P1 (extended per-phase) | Top-level `schema_version: "1"`; per-mode variant carries result fields |
| `build_address_from_xpub` extended with `ScriptType::P2pkh` arm + 5-site enum extension | `cmd/convert.rs` (lines: 343 enum, 349 parse, 366-369 template→script-type, 1485-1500 match arm) | P3 | R0 I3 fold — explicitly enumerated edit sites |
| `ToolkitError::XpubSearchNoMatch { mode: &'static str, searched: usize }` variant | `error.rs` (~line 10 enum + 321 exit_code + 366 kind + 373 message) | P1 (introduced; reused P2/P3/P4) | R0 I4 fold — dedicated variant routes to exit 4 |
| Inline-mirrored default-path-notice emission (~6 LOC) at `cmd/xpub_search/account_of_descriptor.rs` | new | P2 | R0 I7 lock — no extraction this cycle; FOLLOWUP `non-canonical-notice-helper-extract` filed |

---

## §3 — SPEC: P1 `xpub-search path-of-xpub`

### 3.1 Synopsis

```
mnemonic xpub-search path-of-xpub \
    {--phrase <bip39> | --phrase-stdin | --ms1 <bech32> | --ms1-stdin | <positional bare-arg HRP-autodetect>} \
    [--passphrase <p> | --passphrase-stdin] \
    --target-xpub <xpub|ypub|zpub|Ypub|Zpub|tpub|upub|vpub|Upub|Vpub|mk1...> \
    [--language <english|...>] [--network <mainnet|testnet|signet|regtest>] \
    [--min-account 0] [--number-of-accounts 20] [--max-account <N>] \
    [--add-path <template>]... \
    [--json]
```

Seed-intake mutex: exactly one of `{--phrase, --phrase-stdin, --ms1, --ms1-stdin, positional}`. **Positional accepts ms1-HRP bech32 only** (mirrors `repair::classify_hrp_prefix` v0.24.0 pattern at `repair.rs:84-108`); positional input that doesn't start with `ms1` HRP fails with a clear "BIP-39 phrase must be supplied via --phrase/--phrase-stdin (no HRP for positional autodetect)" error (R1 C-1 lock).

**Clap-derive shape (R2 I-R2-4 lock):** Each per-mode `*Args` struct (in its per-mode file) defines `--phrase`, `--phrase-stdin`, `--ms1`, `--ms1-stdin`, and `extra_strings: Vec<String>` (positional) fields directly. No `#[command(flatten)]` shared `SeedIntake` struct — clap-derive `flatten` doesn't compose cleanly with per-mode `Args` derives (v0.24.0 D35 cycle pulled cross-HRP `conflicts_with_all` precisely because of this — see precedent at `repair.rs:23-72` + `verify_bundle.rs:131-133`). Each `--*-stdin` carries `conflicts_with` against `--*` (inline-form); `extra_strings` carries `conflicts_with_all = ["phrase", "phrase_stdin", "ms1", "ms1_stdin"]` AND `required_unless_present_any = ["phrase", "phrase_stdin", "ms1", "ms1_stdin"]`. Helper `seed_intake::resolve(...)` in shared `cmd/xpub_search/seed_intake.rs` consumes the parsed fields uniformly across modes. C1 verifies via `--help` byte-exact + clap-error-on-double-supply cells.

Target intake polymorphism (R1 I-6 lock): `--target-xpub` accepts either a SLIP-0132 xpub string OR an `mk1...` bech32 card containing an xpub. Discrimination strategy: starts-with `mk1` (with `1` separator per bech32) → mk1 card; else → SLIP-0132 prefix-set check (`xpub|tpub|ypub|Ypub|zpub|Zpub|upub|Upub|vpub|Vpub`). Implementation in `cmd/xpub_search/target_intake.rs` calls `crate::repair::classify_hrp_prefix` for the bech32 branch (`pub(crate)` — same-crate access works per `verify_bundle.rs:983` precedent — R2 m-R2-2 note); base58-decode for the SLIP-0132 branch. mk1 decoded via the existing toolkit primitive; resulting xpub feeds the search.

Multisig variants (`Ypub`/`Zpub`/`Upub`/`Vpub`) are **accepted** in P1 — they represent a cosigner xpub at a multisig path. P1 searches both single-sig templates (irrelevant for these prefixes — won't match) and BIP-48 multisig templates. The variant signal is preserved in output. (R0 I-list item 8 fold: P1 does NOT refuse multisig prefixes; the search just runs against all candidate path families.)

### 3.2 Behavior

1. Resolve seed intake (mutex check); parse `--phrase` (or stdin/positional/ms1) → `bip39::Mnemonic`. On `--ms1` decode failure, route through `try_repair_and_short_circuit` (TTY-gated via `resolve_no_auto_repair`).
2. Resolve target intake; if mk1, decode to xpub; normalize SLIP-0132 alt-prefix via `slip0132::normalize_xpub_prefix`. Capture original variant string.
3. Compute master xprv via `derive_master_seed` + `Xpriv::new_master`. Network defaults to xpub-prefix inferred; `--network` overrides.
4. Build candidate path set:
   - Single-sig templates: for each of `[Bip44, Bip49, Bip84, Bip86]`, for each account `a` in range: `template.derivation_path(network, a)`.
   - Multisig templates: for each `script_type ∈ {1, 2, 3}`, for each account `a` in range: `DerivationPath::from_str(MultisigPathFamily::Bip48.default_origin_path(network, a, script_type))`.
   - `--add-path` templates: for each user-supplied template, for each account `a` in the range, produce one candidate path by substituting `a` for the **first** `account'` token in the template (then for the first `account` token if no `account'` found); if neither token present, search the path exactly once as-is (R3 m-R3-1 lock). **Substitution applies ONLY to `account'`/`account` token** — `coin'`, `script_type'`, and other tokens pass through literally; user supplies multiple `--add-path` flags for per-network coverage AND for multi-occurrence-within-one-template (R0 I9 lock).
   - Range: `a ∈ [min-account, max(min-account + number-of-accounts, max-account.unwrap_or(min-account + number-of-accounts)))` (half-open).
5. For each candidate path: derive child xpub. Byte-equal compare to target xpub (after normalization). First match wins.
6. On match: emit path, account, template name, target-xpub original variant. Exit 0.
7. On no match: return `ToolkitError::XpubSearchNoMatch { mode: "path-of-xpub", searched: N }`; exit 4. Includes JSON envelope under `--json`.

### 3.3 Output (text, match)

```
match: m/86'/0'/0'  (template=bip86, account=0)
target-xpub: xpub6r... (normalized from zpub; variant=zpub)
searched: 7 templates × 20 accounts = 140 paths
```

### 3.4 Output (`--json`)

```json
{
  "schema_version": "1",
  "mode": "path-of-xpub",
  "result": "match",
  "path": "m/86'/0'/0'",
  "template": "bip86",
  "account": 0,
  "target_xpub_canonical": "xpub6...",
  "target_xpub_variant": "zpub",
  "searched_count": 140
}
```

`target_xpub_variant` is `null` when the target was supplied as a canonical xpub/tpub (no SLIP-0132 alt-prefix swap occurred) — serialized via `Option<&'static str>` with `#[serde(skip_serializing_if = "Option::is_none")]` **disabled** (the field is always emitted with `null` when empty, to keep the JSON envelope structurally stable across runs) (R1 m-2 lock).

Top-level `tag = "mode"` deviates from project's `tag = "kind"` convention (used in `InspectJson` at `cmd/inspect.rs:245` and `gui_schema.rs:111`). Justification: "mode" is the natural domain term for xpub-search's four sub-modes; "kind" would conflict with `RepairJson`'s `kind: "ms1"|"mk1"|"md1"` per-card-type semantic. Documented in CHANGELOG `### Added` for grep-discoverability (R0 m2 fold).

`schema_version: "1"` introduces a new top-level field not present on `InspectJson` (R0 m3 acknowledged); justification: forward-compat for `XpubSearchJson` is high-value because the modes will accrue fields. Parallel `schema_version` addition to `InspectJson` filed as FOLLOWUP `inspect-json-schema-version-backfill` for v0.27+.

### 3.5 Exit codes

| Code | Meaning | Source |
|------|---------|--------|
| 0 | Match found | normal success |
| 1 | Bad input (parse failure on phrase / ms1 / xpub / mk1) | `error.rs:298` BadInput / Bip39 / etc. |
| 4 | No match in searched set | `ToolkitError::XpubSearchNoMatch` (new variant, R0 I4 fold) |
| 5 | Auto-fire BCH short-circuit on `--ms1` decode failure | `repair.rs:962` |
| 64 | Clap arg-parse error | `main.rs:86` |

### 3.6 Secret hygiene

- Phrase / ms1 secret: `Zeroizing<String>`; argv-leak advisory if `--phrase <inline>` / `--ms1 <inline>` used.
- Passphrase secret: `Zeroizing<String>` via `read_stdin_passphrase`.
- Master seed: `Zeroizing<[u8;64]>` via `derive_master_seed`.
- mlock pinning: per-iteration `mlock::pin_pages_for(&entropy_bytes[..])` mirroring `derive_slot.rs:82`. Pin lifetime bounded by loop iteration.
- Intermediate `Xpriv` lacks upstream Drop+Zeroize (tracked at `rust-bitcoin-xpriv-zeroize-upstream` FOLLOWUP). Mitigation unchanged this cycle.

---

## §4 — SPEC: P2 `xpub-search account-of-descriptor`

### 4.1 Synopsis

```
mnemonic xpub-search account-of-descriptor \
    {--phrase ... | --ms1 ... | positional} \
    [--passphrase ... | --passphrase-stdin] \
    --descriptor <value-OR-->  OR  --descriptor-from <node>=<value> \
    [--language <english|...>] [--network <mainnet|testnet|signet|regtest>] \
    [--min-account 0] [--number-of-accounts 20] [--max-account <N>] \
    [--add-path <template>]... \
    [--json]
```

### 4.2 Descriptor-shape auto-detect (L9)

`--descriptor <value>` is the polymorphic intake. **Tie-break order (R1 I-5 lock)** — checked top-to-bottom; first match wins:

1. **Starts-with `{` (after `trim_start`)** → BIP-388 wallet-policy JSON.
2. **All-bech32 tokens with `md1` HRP** (single token or comma+whitespace-separated multi-chunk) → md1 card(s).
3. **Contains `@N` token** (regex `@\d+` outside string-literal context per `parse_descriptor.rs:60-127` lex rules) → toolkit-@N → **refuse** (synthetic xpubs, non-searchable).
4. **Else** → external literal-xpub descriptor.

Per-shape parsing (all 3 searchable shapes funnel through the same unified primitive — R1 I-1 lock):

| Shape | Detection rule | Per-shape preprocessing | Final parser (unified) |
|-------|----------------|--------------------------|------------------------|
| BIP-388 wallet-policy JSON | Tie-break rule 1 | Parse JSON via `serde_json::from_str::<BipPolicyJson>` with `#[serde(deny_unknown_fields)]` strict struct (fields: `name: String`, `description_template: String`, `keys_info: Vec<String>` — per `wallet_export/pipeline.rs:192-198` emitter, NOT `"description"` and NOT bare `@N`). Reconstruct descriptor string by replacing each `@N/**` token in `description_template` with `keys_info[N] + "/<0;1>/*"` (exact inverse of the emitter at `wallet_export/pipeline.rs:192-198`) (R2 I-R2-2 lock). | → unified miniscript parser (string path) |
| md1 card(s) | Tie-break rule 2 | Stdin/multi-chunk **only** via `--descriptor-from md1=-` (one chunk per line; mirrors `repair.rs:7` / `verify_bundle.rs:88-90` / `inspect.rs:46-48` precedent) (R2 I-R2-1 + R3 I-R3-2 lock — **NO whitespace+comma split anywhere**). Single-chunk inline: `--descriptor <md1...>` works with auto-detect rule 2. Collect `Vec<&str>`; call `md_codec::chunk::reassemble(&chunks) -> md_codec::Descriptor`. Walk tree + TLV fields directly to extract cosigner xpubs (NO re-serialization through canonical descriptor string; the `md_codec::Descriptor → String` serializer is NOT confirmed-existing — R2 C-R2-1 lock). | → md_codec-tree-walk path (NOT the string-funnel path) |
| Toolkit @N-placeholder | Tie-break rule 3 | (none — refused with `ToolkitError::BadInput("toolkit @N descriptors carry synthetic xpubs; supply a literal-xpub descriptor, md1 card, or BIP-388 wallet-policy JSON instead")`) | — |
| External literal-xpub descriptor | Tie-break rule 4 (default) | (none — passes through directly) | `rust_miniscript::Descriptor::<DescriptorPublicKey>::from_str(s)` → walk via `iter_pk` collecting `(cosigner_index, fingerprint_anno, derivation_path_anno, xpub)` tuples (R2 I-R2-3 lock — use `iter_pk` not `for_each_key`; precedent `wallet_export/pipeline.rs:177`) |

Explicit override: `--descriptor-from <node>=<value>` disambiguates when auto-detect picks the wrong shape:
- `--descriptor-from literal=<...>`
- `--descriptor-from md1=<...>` — single inline chunk OR `--descriptor-from md1=-` for stdin multi-chunk (one chunk per line, mirroring `repair.rs:7` precedent) (R3 I-R3-2 lock — no whitespace+comma split)
- `--descriptor-from bip388=<...>` (JSON via inline or `=-` stdin)

**Tie-break rule 2 sentinel handling (R3 I-R3-3 lock):** Rule 2 fires either (a) the all-bech32-tokens-with-`md1`-HRP heuristic on inline `--descriptor`, OR (b) the explicit-form `--descriptor-from md1=<value>` (with `-` as stdin sentinel handled identically to `repair.rs:7` — read stdin lines, blank-line skip, each line a chunk).

### 4.2.1 Normalized intermediate form (R1 I-1 + R2 C-R2-1 lock)

Two-funnel approach (because `md_codec::Descriptor → String` is not a confirmed-existing API):

- **String funnel** (BIP-388 JSON + literal-xpub shapes): canonical descriptor string → `rust_miniscript::Descriptor::<DescriptorPublicKey>::from_str` → `iter_pk` walk → `Vec<CosignerExtract { idx, fingerprint_anno, derivation_path_anno, xpub }>`. Mirrors `wallet_export/pipeline.rs:163-204` inverse-direction.
- **Tree-walk funnel** (md1 shape): `md_codec::chunk::reassemble(&chunks) -> md_codec::Descriptor`, then iterate `desc.n` cosigner slots and for each index `idx ∈ [0, n)` extract (R3 C-R3-1 lock — source-verified field names per `parse_descriptor.rs:806-812` + `verify_bundle.rs:1925-1985` + `synthesize.rs:69-75`):
  - **xpub material** from `desc.tlv.pubkeys: Option<Vec<(u8 slot, [u8; 65] payload)>>` — payload layout `[0..32] = chain_code, [32..65] = compressed pubkey` per `synthesize.rs:71-75`.
  - **origin fingerprint** from `desc.tlv.fingerprints: Option<Vec<(u8 slot, [u8; 4] fp)>>`.
  - **derivation path** by mirroring `cmd/verify_bundle.rs:1934-1944` `md_path_for` closure: prefer per-slot override from `desc.tlv.origin_path_overrides: Option<Vec<(u8, OriginPath)>>`; else fall back to `desc.path_decl.paths` (`PathDeclPaths::Shared(OriginPath)` returns the same path for every idx; `PathDeclPaths::Divergent(Vec<OriginPath>)` indexes per slot).

**Comparison strategy:** derived child xpubs from `match_xpub_against_paths` use `synthesize::xpub_to_65(&derived) -> [u8; 65]` to project to the same 65-byte form; byte-equal compare to `desc.tlv.pubkeys[idx].1`. No full BIP-32 xpub reconstruction required — the 65-byte payload (chain code + compressed pubkey) is the unique cryptographic identity. Precedent: `cmd/verify_bundle.rs:1971-1985`.

Both funnels feed the same `match_descriptor_against_seed` downstream consumer; the helper boundary is `cosigner_extract::from_string_descriptor(s) -> Vec<CosignerExtract>` and `cosigner_extract::from_md_descriptor(d: &md_codec::Descriptor) -> Vec<CosignerExtract>`. The `CosignerExtract` struct carries the 65-byte payload form for both funnels (string-funnel projects via `xpub_to_65` too) so the downstream comparator is funnel-agnostic.

**Phase 0 reconnaissance task in C2 commit:** Re-read `md_codec::Descriptor` (the `tree` + `path_decl` + `tlv` field shapes) end-to-end; confirm field-name + type citations above remain accurate against `md-codec` crate at the v0.34.0 pin. If a public `to_string`/`Display`/`render` API exists, file `md-codec-descriptor-string-serializer-discovered` FOLLOWUP and KEEP tree-walk path (no rework — funnel uniformity is not load-bearing for v0.26.0).

**Contingency FOLLOWUP (file at C2 cycle close if applicable):** `md-codec-descriptor-string-serializer` — sibling repo `mnemonic-descriptor` request for public `Display` impl or `fn to_descriptor_string(&self) -> String` to unify the two funnels in v0.27+ if a third caller surfaces.

### 4.3 Behavior

1. Seed intake (mutex) → master xprv (identical to P1).
2. Descriptor intake auto-detect → unified `Vec<CosignerExtract>` via §4.2.1 two-funnel pipeline.
3. **Zero-xpub guard (R2 m-R2-1 + R3 I-R3-4 lock):** After unified extraction, count xpub-shaped entries:
   - **String funnel:** `parsed.iter_pk().filter(|k| matches!(k, DescriptorPublicKey::XPub(_) | DescriptorPublicKey::MultiXPub(_))).count()`.
   - **Tree-walk funnel:** `desc.tlv.pubkeys.as_ref().map_or(0, Vec::len)` (handles `None` case where md1 carries no xpub material — pre-binding state per `parse_descriptor.rs:781-787` — explicitly returns 0 rather than panicking).
   
   If zero → refuse with `BadInput("descriptor contains no extended keys; xpub-search requires xpub-shaped cosigners")` exit 1. Guards the auto-detect rule-4 fall-through edge case (e.g. `wpkh(03abc…)` raw pubkey) AND the template-only md1 edge case.
4. For v0.19.0 silent-default descriptors (literal-xpub with missing key-origin `[fp/path]` annotations): mirror `cmd/bundle.rs:1367-1388` inline (~6 LOC) — assign BIP-48 default `m/48'/coin'/account'/2'`, emit the same stderr notice as `bundle`.
5. For each cosigner xpub: run `match_xpub_against_paths(seed, candidates, cosigner_xpub)`. Collect matches.
6. Emit list of matched cosigners with `(cosigner_index, path, account, template)` per match. Multiple cosigners matching the same seed (unusual but possible) → report all.
7. NUMS sentinel cosigner (per v0.19.0 `tr(NUMS, …)`): skip with output `"unspendable_internal_key": true` for that cosigner; do not search.
8. Bare-`tr()` refusal: surfaces as `parse_descriptor.rs:296-307` `BARE_TR_NO_KEY_MSG` exit 1.

### 4.4 Output (text, multisig match)

```
match: cosigner @0  m/48'/0'/0'/2'  (template=bip48-wsh, account=0)
descriptor: wsh(sortedmulti(2, [fp1/48h/0h/0h/2h]xpub1.../0/*, [fp2/.../...]xpub2.../0/*, [fp3/.../...]xpub3.../0/*))
cosigners total: 3
matched cosigner indices: [0]
searched: 7 templates × 20 accounts × 3 cosigners = 420 paths
```

### 4.5 Output (`--json`)

```json
{
  "schema_version": "1",
  "mode": "account-of-descriptor",
  "result": "match",
  "matched_cosigners": [
    {"cosigner_index": 0, "path": "m/48'/0'/0'/2'", "template": "bip48-wsh", "account": 0}
  ],
  "cosigners_total": 3,
  "searched_count_per_cosigner": 140,
  "descriptor_shape": "literal_xpub"
}
```

### 4.6 Exit codes

Same as P1. `XpubSearchNoMatch` variant carries `mode: "account-of-descriptor"`.

---

## §5 — SPEC: P3 `xpub-search address-of-xpub`

### 5.1 Synopsis

```
mnemonic xpub-search address-of-xpub \
    --xpub <value> | --xpub-stdin \
    --target-address <addr>... \
    [--gap-limit 20] \
    [--external-only] \
    [--address-type p2pkh|p2sh-p2wpkh|p2wpkh|p2tr] \
    [--network mainnet|testnet|signet|regtest] \
    [--json]
```

`--xpub` accepts either a bare xpub (any SLIP-0132 prefix) OR an mk1 card. `--external-only` disables internal-change scanning (default is both chains). The plan drops the redundant `--include-change` flag (R0 I-list item 7 fold — `--external-only` is the sole toggle).

### 5.2 Behavior

1. Parse `--xpub`; if mk1, decode; SLIP-0132 normalize.
2. Address-type inference (priority): explicit `--address-type` → SLIP-0132 prefix mapping (`xpub`/`tpub` with no other signal → require `--address-type` explicit; `ypub`/`upub` → P2SH-P2WPKH; `zpub`/`vpub` → P2WPKH; no native P2TR prefix in SLIP-0132 → require `--address-type p2tr` explicit). Multisig SLIP-0132 prefixes (`Ypub`/`Zpub`/`Upub`/`Vpub`) → **refuse** with clear error "address-of-xpub is single-sig only; multisig address derivation requires the full descriptor (use `account-of-descriptor` to find the matching account)".
3. Network inference: from xpub version byte via `network_from_xpub`; `--network` overrides for signet/regtest disambiguation.
4. For each target address: scan `chain/i` for `chain ∈ {0}` (external) + (default) `{1}` (internal change) + `i ∈ [0, gap_limit)`. Derive child via `xpub.derive_pub`, render via extended `build_address_from_xpub(secp, &child, script_type, network)`, byte-equal compare.
5. Per-target first-match wins. Emit per-target results.

### 5.3 P2PKH gap-fix (in C3 commit) — 5-site edit (R0 I3 fold)

1. **`cmd/convert.rs:343` `enum ScriptType`** — add `P2pkh` variant.
2. **`cmd/convert.rs:349` `parse_script_type_arg`** — add `"p2pkh" => Ok(ScriptType::P2pkh)` arm.
3. **`cmd/convert.rs:366-369` `script_type_from_template`** — add `CliTemplate::Bip44 => Some(ScriptType::P2pkh)` arm.
4. **`cmd/convert.rs:1485-1500` `build_address_from_xpub`** — add `ScriptType::P2pkh => Address::p2pkh(&child.to_pub(), network.network_kind()).to_string()` arm.
5. **Clap value-parser / possible-values** — verify `cmd/convert.rs` `--address-type` flag's `value_parser` lists `p2pkh`; extend if missing. Verify with `mnemonic convert --address-type --help` byte-exact.

P3 phase-0 reconnaissance: grep `ScriptType::` outside `cmd/convert.rs` for exhaustive matches; if found, extend those sites too.

### 5.4 Output (text)

```
match: bc1qabc... → 0/5  (script_type=p2wpkh, chain=external, index=5)
no match: bc1qxyz... (searched 0/0..19 + 1/0..19)
scanned: 1 target × 40 children = 40 derivations
```

### 5.5 Output (`--json`)

```json
{
  "schema_version": "1",
  "mode": "address-of-xpub",
  "results": [
    {"target": "bc1qabc...", "result": "match", "chain": "external", "index": 5, "script_type": "p2wpkh"},
    {"target": "bc1qxyz...", "result": "no_match", "scanned_external": 20, "scanned_internal": 20}
  ],
  "xpub_canonical": "xpub6...",
  "xpub_variant": "zpub",
  "gap_limit": 20
}
```

### 5.6 Exit codes

| Code | Meaning |
|------|---------|
| 0 | All targets matched (1+ targets, all found) |
| 1 | Bad input (xpub parse error, address parse error, multisig SLIP-0132 prefix) |
| 4 | At least one target unmatched (`XpubSearchNoMatch` with `mode: "address-of-xpub"`) |
| 64 | Clap arg-parse error |

P3 takes no secret material; auto-fire BCH repair does NOT apply (xpub parse failure is direct exit 1).

---

## §6 — SPEC: P4 `xpub-search passphrase-of-xpub`

### 6.1 Synopsis

```
mnemonic xpub-search passphrase-of-xpub \
    {--phrase ... | --ms1 ... | positional} \
    {--passphrase <p> | --passphrase-stdin}   (mandatory) \
    --target-xpub <xpub|mk1...> \
    [--language <english|...>] [--network <mainnet|testnet|signet|regtest>] \
    [--min-account 0] [--number-of-accounts 20] [--max-account <N>] \
    [--add-path <template>]... \
    [--json]
```

### 6.2 Behavior

P4 is **P1 + a fixed mandatory passphrase**. Re-derive master via `derive_master_seed(mnemonic, passphrase)`, then invoke `match_xpub_against_paths`. The semantic difference from P1: P1 asks "what path produced this xpub?"; P4 asks "does this specific passphrase produce this xpub (at some standard path)?". Clap requires the passphrase group (`required = true`).

### 6.3 What P4 deliberately does NOT do (MVP)

- No `--passphrases-file <path>` brute-force.
- No streaming candidates from stdin.
- No generated passphrase wordlists.

Filed as FOLLOWUP `xpub-search-passphrase-bruteforce` for v0.27+.

### 6.4 Stderr advisory (always emit)

```
note: passphrase verification searches the standard BIP-44/49/84/86 + BIP-48 templates × account range; if the wallet uses a non-standard path, supply --add-path or use `xpub-search path-of-xpub` to find the path first.
```

### 6.5 Output / exit codes

Identical envelope shape to P1, with `"mode": "passphrase-of-xpub"`. `XpubSearchNoMatch.mode = "passphrase-of-xpub"`.

---

## §7 — SPEC: GUI lockstep (`mnemonic-gui v0.11.0`)

### 7.1 Schema contract

Toolkit emits `gui-schema` JSON with 4 new subcommand entries (flattened):

- `xpub-search-path-of-xpub`
- `xpub-search-account-of-descriptor`
- `xpub-search-address-of-xpub`
- `xpub-search-passphrase-of-xpub`

Each carries clap-derive's flag inventory. v0.25.0 global-vs-local-id disjointness `debug_assert!` at `cmd/gui_schema.rs:1106` (signature) / 1111 (assert site) is the gate. Schema version remains **v5** — no grammar extension this cycle, only new subcommand entries. Snapshot file: `tests/cli_gui_schema_v5_extensions.rs` (R0 I6 fold — was previously stale `_v3_`).

### 7.2 GUI panes (4 + hub)

| Pane | Toolkit mode | Inputs | Output rendering |
|------|--------------|--------|------------------|
| xpub-search hub | — | nav-only | 4 cards link to the 4 panes |
| Path of xpub | path-of-xpub | seed-intake widget (phrase/ms1 tabs), passphrase, target xpub/mk1, min/N/max account, add-path repeater | `XpubSearchJson` match/no-match render |
| Account of descriptor | account-of-descriptor | seed-intake widget, passphrase, descriptor textarea (auto-detect shape; show detected shape badge), account range | per-cosigner match table |
| Address of xpub | address-of-xpub | xpub/mk1 field, target-address multi-line, gap limit, chain selector | per-target match table |
| Passphrase of xpub | passphrase-of-xpub | seed-intake widget, **mandatory** passphrase, target xpub/mk1, account range | yes/no big-text + path detail on match |

### 7.3 Widget reuse + net-new

- Reuse: `PhraseField`, `PassphraseField`, `Ms1Field` (v0.6+ / v0.9+).
- Net-new: `SeedIntakeWidget` (tabs: phrase / ms1 / positional), `TargetXpubField` (xpub-or-mk1 with prefix-normalize-on-paste), `DescriptorIntakeField` (textarea + shape-detect badge), `TargetAddressField` (multi-line), `AddPathRepeater` (Vec<String> with +/− buttons), `XpubSearchResultRenderer`.

### 7.4 GUI kittest cells (~28 net-new)

- P1 path-of-xpub: 8 cells (match phrase, match ms1, positional, mk1 target, alt-prefix normalize, add-path, account-range, passphrase, error refusal — pick 8)
- P2 account-of-descriptor: 9 cells (single-sig literal-xpub match, multisig 2-of-3 cosigner match, multi-cosigner-match, md1 intake, BIP-388 JSON intake, toolkit-@N refusal, NUMS sentinel skip, descriptor parse error, default-path-inference notice)
- P3 address-of-xpub: 6 cells (match external, match internal, no-match, alt-prefix normalize, multisig-prefix refusal, gap-limit override)
- P4 passphrase-of-xpub: 4 cells (match, no-match-wrong-passphrase, mandatory-passphrase clap-error, stderr advisory)
- Hub navigation: ~2 cells

Total: ~29 cells.

---

## §8 — SPEC: Manual lockstep

### 8.1 Coverage gate

`docs/manual/tests/lint.sh` requires each CLI flag in `tests/cli-subcommands.list` to appear in `40-cli-reference/41-mnemonic.md`. C1..C4 each adds the relevant subcommand-mode entry + full flag table + `--json` envelope schema.

### 8.2 GUI manual

`docs/manual-gui/` cells documenting each pane + hub. Track via `manual-gui` lockstep gate.

---

## §9 — SPEC: Cross-cutting concerns

### 9.1 Argv leakage

- Phrase / ms1 / passphrase: `--*-stdin` preferred; inline emits `secret_in_argv_warning`.
- Descriptor / xpub / mk1 (containing xpub only — NOT secret per project taxonomy) / address: argv intake fine. No advisories.
- `--add-path` templates: NOT secret; argv intake fine.

### 9.2 mlock pinning

Per-iteration `mlock::pin_pages_for(&entropy_bytes[..])` in P1/P2/P4 derivation loops, mirroring `derive_slot.rs:82`. Pin lifetimes bounded by loop scope; no cumulative pinning.

### 9.3 Auto-fire BCH repair (R0 I1 fold — narrowed to `--ms1` decode path)

P1/P2/P4 auto-fire applies **only** when seed intake is via `--ms1` / `--ms1-stdin` / positional-detected-as-ms1 AND the decode fails. The auto-fire path threads through `crate::repair::try_repair_and_short_circuit` at `repair.rs:962`, TTY-gated via `crate::repair::resolve_no_auto_repair(no_auto_repair)` at `repair.rs:355`. `--phrase` BIP-39 parse failure routes directly to exit 1 (no BCH primitive for plain text).

P3 has no seed intake; auto-fire does NOT apply.

### 9.4 Exit-code contract + `XpubSearchNoMatch` (R0 I4 fold — 4-site edit)

1. **`error.rs:10ff` `enum ToolkitError`** — add variant `XpubSearchNoMatch { mode: &'static str, searched: usize }`.
2. **`error.rs:296-322` `exit_code()`** — add arm `XpubSearchNoMatch{..} => 4`.
3. **`error.rs:325ff` `kind()`** — add arm.
4. **`error.rs:373ff` `message()` / `Display`** — add arm: `"no match in searched set: mode={mode}, paths searched={searched}"`.

No `friendly.rs` entry needed (direct message).

### 9.5 Per-mode JSON envelope

```rust
#[derive(Serialize)]
struct XpubSearchEnvelope {
    schema_version: &'static str, // "1"
    #[serde(flatten)]
    body: XpubSearchJson,
}

#[derive(Serialize)]
#[serde(tag = "mode", rename_all = "kebab-case")]
enum XpubSearchJson {
    PathOfXpub(PathOfXpubResult),
    AccountOfDescriptor(AccountOfDescriptorResult),
    AddressOfXpub(AddressOfXpubResult),
    PassphraseOfXpub(PassphraseOfXpubResult),
}
```

Each variant carries its own per-mode fields. `tag = "mode"` deviation from project's `tag = "kind"` documented in CHANGELOG (R0 m2).

### 9.6 Iteration determinism

Candidate iteration order: **fixed `Vec<&'static str>` template ordering** = `["bip44", "bip49", "bip84", "bip86", "bip48-sh-wsh", "bip48-wsh", "bip48-tr-multi-a"]` × accounts ascending × `--add-path` templates in user-supplied order × accounts ascending. First match wins. No `HashMap`/`HashSet` for iteration (R0 R4 lock). Snapshot test asserts byte-exact JSON across two consecutive runs.

### 9.7 No sibling-codec lockstep this cycle

Toolkit-only cycle. No `ms-codec` / `mk-codec` / `md-codec` consumption changes (uses existing `md_codec::chunk::reassemble` for P2 md1 intake; no API additions). Sibling crate-version pins unchanged. Mirrors v0.21.0 / v0.22.1 / v0.25.0 toolkit-only precedent (R0 m8 fold).

---

## §10 — PLAN: 6 commits (C1..C6 — commit numbering aligned with phase numbering; R3 I-R3-1 lock)

**Commit ↔ phase mapping** (per-phase agents dispatched by commit number):
- **C1** = phase P1 (path-of-xpub) + umbrella scaffolding
- **C2** = phase P2 (account-of-descriptor)
- **C3** = phase P3 (address-of-xpub) + P2PKH gap-fix
- **C4** = phase P4 (passphrase-of-xpub)
- **C5** = GUI lockstep `mnemonic-gui v0.11.0` (separate repo)
- **C6** = cycle close + release on toolkit

### C1 — Umbrella scaffold + phase P1 path-of-xpub (one commit)

**Commit:** `feat(xpub-search): P1 path-of-xpub mode + umbrella scaffolding`

**Files (new):**
- `crates/mnemonic-toolkit/src/cmd/xpub_search/mod.rs`
- `crates/mnemonic-toolkit/src/cmd/xpub_search/path_of_xpub.rs`
- `crates/mnemonic-toolkit/src/cmd/xpub_search/candidate_paths.rs`
- `crates/mnemonic-toolkit/src/cmd/xpub_search/path_search.rs`
- `crates/mnemonic-toolkit/src/cmd/xpub_search/seed_intake.rs`
- `crates/mnemonic-toolkit/src/cmd/xpub_search/target_intake.rs`
- `tests/cli_xpub_search_path_of_xpub.rs`

**Files (edit):**
- `crates/mnemonic-toolkit/src/cmd/mod.rs` — register `xpub_search` module
- `crates/mnemonic-toolkit/src/main.rs` — add `XpubSearch(cmd::xpub_search::XpubSearchArgs)` Command variant + dispatch arm
- `crates/mnemonic-toolkit/src/error.rs` — add `XpubSearchNoMatch` variant + 4 edit sites (§9.4)
- `tests/cli_gui_schema_v5_extensions.rs` — extend snapshot with `xpub-search-path-of-xpub`
- `docs/manual/src/40-cli-reference/41-mnemonic.md` — add `xpub-search path-of-xpub` section
- `docs/manual/tests/cli-subcommands.list` — add `mnemonic xpub-search path-of-xpub`
- `CHANGELOG.md` — `### Added` entry citing `tag = "mode"` deviation + `schema_version: "1"` introduction

**Tests (TDD; written before impl):** ~19 integration cells + 1 unit cell covering happy paths (xpub target, mk1 target, BIP-44/49/84/86/48 hits at varied accounts), no-match exit 4, SLIP-0132 normalization, account-range bounds, `--add-path` substitution (`account'` + `account` tokens), `--phrase-stdin` / `--ms1-stdin` / positional intakes (positional restricted to ms1 HRP), auto-fire on `--ms1` decode failure, `--no-auto-repair` honor, `MNEMONIC_FORCE_TTY` env-var, `--json` envelope exact-match, argv-leak advisory, gui-schema snapshot, **unit cell: `XpubSearchEnvelope` serde round-trip asserting `{"schema_version":"1","mode":"path-of-xpub", ...}` byte-exact** (R1 I-4 lock — `#[serde(flatten)]` + inner `tag = "mode"` is a known-tricky serde shape; round-trip cell pins behavior).

**Reviewer-loop:** Opus architect R0 → R1 → ... until 0C/0I per-phase.

**Acceptance gate:**
- `cargo test -p mnemonic-toolkit --test cli_xpub_search_path_of_xpub` green
- `cargo test -p mnemonic-toolkit` green
- `make -C docs/manual lint MNEMONIC_BIN=... MD_BIN=... MS_BIN=... MK_BIN=...` green
- `mnemonic xpub-search path-of-xpub --help` byte-exact against manual chapter
- **CHANGELOG `### Resolved` entries AND `Status:` flips in `design/FOLLOWUPS.md`** for every cited FOLLOWUP (R3 m-R3-2 lock; per memory `feedback_per_phase_agents_forget_followup_status_flip`)

### C2 — phase P2 account-of-descriptor (one commit)

**Commit:** `feat(xpub-search): P2 account-of-descriptor mode (literal-xpub / @N / md1 / BIP-388-JSON)`

**Files (new):**
- `crates/mnemonic-toolkit/src/cmd/xpub_search/account_of_descriptor.rs`
- `crates/mnemonic-toolkit/src/cmd/xpub_search/account_search.rs`
- `crates/mnemonic-toolkit/src/cmd/xpub_search/descriptor_intake.rs` — shape auto-detect + the 4 parsers (literal-xpub via `rust_miniscript::Descriptor::from_str`, toolkit-@N refusal, md1 via `md_codec::chunk::reassemble`, BIP-388 JSON)
- `tests/cli_xpub_search_account_of_descriptor.rs`

**Files (edit):**
- `crates/mnemonic-toolkit/src/cmd/xpub_search/mod.rs` — extend enum + dispatch
- Manual / `cli-subcommands.list` / `cli_gui_schema_v5_extensions.rs` / CHANGELOG

**Tests:** ~14 cells — single-sig literal-xpub match, multisig sortedmulti match-at-cosigner-1, multi-cosigner-match (same seed twice via --add-path), md1 chunked intake match, BIP-388 JSON intake match, toolkit-@N refusal, NUMS sentinel skip, bare-`tr()` refusal, v0.19.0 default-path stderr notice emission, descriptor parse error → exit 1, account range honored, `--add-path` per-cosigner, `--passphrase`, JSON envelope.

**Acceptance gate:** same as C1 (including FOLLOWUPS `Status:` flip + CHANGELOG `### Resolved` entries for any cited FOLLOWUP).

### C3 — phase P3 address-of-xpub + P2PKH gap-fix (one commit)

**Commit:** `feat(xpub-search): P3 address-of-xpub mode + P2PKH support in build_address_from_xpub`

**Files (new):**
- `crates/mnemonic-toolkit/src/cmd/xpub_search/address_of_xpub.rs`
- `crates/mnemonic-toolkit/src/cmd/xpub_search/address_search.rs`
- `tests/cli_xpub_search_address_of_xpub.rs`

**Files (edit):**
- `crates/mnemonic-toolkit/src/cmd/convert.rs` — 5-site P2PKH gap-fix (§5.3)
- `crates/mnemonic-toolkit/src/cmd/xpub_search/mod.rs` — extend enum + dispatch
- `tests/cli_convert_address.rs` — add P2PKH happy + refusal-relaxation cells
- Manual / `cli-subcommands.list` / `cli_gui_schema_v5_extensions.rs` / CHANGELOG

**Tests:** ~14 cells — zpub + bech32 P2WPKH at `0/5`, zpub + change-chain at `1/3`, ypub + P2SH, xpub + explicit P2PKH (gap-fix exercised), xpub + explicit P2TR, no-match → exit 4, `--external-only`, `--gap-limit 50`, multi-target all-match exit 0, multi-target partial → exit 4, multisig prefix refusal, network inference, `--network signet` override, invalid input → exit 1, regression `cli_convert_address` P2PKH happy path.

**Acceptance gate:** same as C1 (including FOLLOWUPS `Status:` flip + CHANGELOG `### Resolved` entries for any cited FOLLOWUP).

### C4 — phase P4 passphrase-of-xpub (one commit)

**Commit:** `feat(xpub-search): P4 passphrase-of-xpub mode`

**Files (new):**
- `crates/mnemonic-toolkit/src/cmd/xpub_search/passphrase_of_xpub.rs`
- `crates/mnemonic-toolkit/src/cmd/xpub_search/passphrase_verify.rs`
- `tests/cli_xpub_search_passphrase_of_xpub.rs`

**Files (edit):**
- `crates/mnemonic-toolkit/src/cmd/xpub_search/mod.rs` — extend enum + dispatch
- Manual / `cli-subcommands.list` / `cli_gui_schema_v5_extensions.rs` / CHANGELOG

**Tests:** ~10 cells.

**Acceptance gate:** same as C1 (including FOLLOWUPS `Status:` flip + CHANGELOG `### Resolved` entries for any cited FOLLOWUP).

### C5 — GUI lockstep `mnemonic-gui v0.11.0` (separate repo)

Plan-doc in GUI repo (thin follow-on); ~28 cells per §7.4; pinned-upstream.toml bump to `mnemonic-toolkit-v0.26.0`.

### C6 — Cycle close + release (one commit on toolkit)

**Commit:** `release(toolkit): mnemonic-toolkit v0.26.0 — xpub-search umbrella (4 modes)`

**Files (edit):**
- `crates/mnemonic-toolkit/Cargo.toml` — version bump
- `CHANGELOG.md` — finalize entries
- `design/PLAN_v0_26_0_xpub_search.md` (new, copied from this plan-mode plan-file) — canonical record
- `design/FOLLOWUPS.md` — file new entries per §15; flip Status for any closed during cycle
- `install.sh` — pinned toolkit tag → `v0.26.0` (R0 m7 fold)
- Tag: `mnemonic-toolkit-v0.26.0`
- GitHub Release with manual PDF attached

**Acceptance:**
- Holistic end-of-cycle architect review (opus) GREEN 0C/0I
- All FOLLOWUPS `Status:` flips applied (per `feedback_per_phase_agents_forget_followup_status_flip` memory)
- `cargo publish --dry-run` for known-blocked miniscript[patch.crates-io] caveat unchanged (toolkit publish remains blocked as in v0.24.x/v0.25.x; not gating)

---

## §11 — Parallelization (R0 I8 + R1 I-3 folds)

**Split-file architecture enables true parallel-disjoint phases.** With each mode in its own file under `cmd/xpub_search/`, C2/C3/C4 commits do NOT three-way-conflict — they each:
- ADD a per-mode file (with `pub struct <Mode>Args` and `pub fn run_<mode>` as the only public exports — every other type/helper is module-private)
- EXTEND `cmd/xpub_search/mod.rs` enum + dispatch + `use` declarations (small mechanical merge):
  ```rust
  // mod.rs
  mod path_of_xpub;
  mod account_of_descriptor;
  mod address_of_xpub;
  mod passphrase_of_xpub;
  // shared helpers
  mod candidate_paths;
  mod path_search;
  // (P2+) mod descriptor_intake;
  // (P3+) mod address_search;
  // (P4+) mod passphrase_verify;
  // (P1+) mod seed_intake; mod target_intake;

  #[derive(Subcommand)]
  pub enum XpubSearchCommand {
      PathOfXpub(path_of_xpub::PathOfXpubArgs),
      AccountOfDescriptor(account_of_descriptor::AccountOfDescriptorArgs),
      AddressOfXpub(address_of_xpub::AddressOfXpubArgs),
      PassphraseOfXpub(passphrase_of_xpub::PassphraseOfXpubArgs),
  }
  ```
  Per-mode commits ADD their own `mod <name>;` line + enum variant + dispatch arm (3-line addition). Conflict surface = 3 lines per phase; trivial 3-way merge.
- ADD a test file (disjoint)
- EXTEND manual / `cli-subcommands.list` / `cli_gui_schema_v5_extensions.rs` snapshot / CHANGELOG (mechanical merge)

Recommended fanout:
- `worktree-xpub-search-brainstorm` (this) — plan-mode + C1
- After C1 lands on master:
  - `worktree-xpub-search-c2-account-of-descriptor` — C2
  - `worktree-xpub-search-c3-address-of-xpub` — C3
  - `worktree-xpub-search-c4-passphrase-of-xpub` — C4
- These three rebase against C1's master; per-phase merge resolves the small mechanical conflicts in `cmd/xpub_search/mod.rs` / shared snapshots.
- After toolkit C4 lands:
  - `mnemonic-gui` repo worktree — C5 GUI lockstep
- After C5 GUI lands:
  - C6 release commit on toolkit (this same worktree or a fresh one)

---

## §12 — Verification

### 12.1 Per-phase

```
cargo test -p mnemonic-toolkit --test cli_xpub_search_path_of_xpub
cargo test -p mnemonic-toolkit --test cli_xpub_search_account_of_descriptor
cargo test -p mnemonic-toolkit --test cli_xpub_search_address_of_xpub
cargo test -p mnemonic-toolkit --test cli_xpub_search_passphrase_of_xpub
cargo test -p mnemonic-toolkit  # full
make -C docs/manual lint MNEMONIC_BIN=... MD_BIN=... MS_BIN=... MK_BIN=...
```

### 12.2 End-to-end smokes (C6 cycle close)

- BIP-32 §Test Vectors seed → known xpubs at known paths → P1 round-trip
- Trezor 24-word vector → known wpkh descriptor → P2 round-trip (literal-xpub)
- Sparrow-exported multisig descriptor (test fixture) → P2 round-trip (cosigner match)
- md1 card → P2 round-trip
- BIP-388 wallet-policy JSON → P2 round-trip
- Known zpub + known bech32 address → P3 round-trip
- Known phrase + passphrase → known xpub → P4 round-trip
- `MNEMONIC_FORCE_TTY=1` / `=0` env-var smokes for auto-fire-on-ms1 path

### 12.3 GUI smokes (C5)

`cargo test -p mnemonic-gui` full kittest run; manual interactive run.

---

## §13 — Acceptance gates (cycle close)

| Gate | Verification |
|------|--------------|
| All 4 toolkit modes green | `cargo test -p mnemonic-toolkit` 0 failures |
| Manual lint green | `make -C docs/manual lint ...` 0 failures |
| gui-schema snapshot stable | `cli_gui_schema_v5_extensions.rs` cells pass |
| Auto-fire applies only on `--ms1` path | Per-phase cells assert |
| `XpubSearchNoMatch` routes to exit 4 | Per-phase cells assert |
| Argv-leak advisory present on inline secrets | Per-phase cells assert |
| mlock pin lifetimes bounded | Code review confirms per-iteration scope |
| Iteration determinism | Snapshot cell asserts byte-exact JSON across runs |
| GUI 4 panes + hub + ~29 cells | `cargo test -p mnemonic-gui` 0 failures |
| End-of-cycle architect review | opus GREEN 0C/0I; folds inline |
| FOLLOWUPS Status flips applied | grep `Status: open` vs CHANGELOG citations all reconciled |
| install.sh pin bumped | `install.sh` carries `v0.26.0` |
| Tag created | `mnemonic-toolkit-v0.26.0` + `mnemonic-gui-v0.11.0` |
| GH Releases live | Manual PDF asset attached |

---

## §14 — Risks (post-R0 register)

| # | Risk | Mitigation |
|---|------|-----------|
| R1 | Account-range default `[0, 20)` misses wallets at account > 20. | `--help` + manual document `--max-account` and `--number-of-accounts`. |
| R2 | P3 P2PKH 5-site gap-fix may touch exhaustive `ScriptType` matches outside `cmd/convert.rs`. | P2 Phase 0 recon: grep `ScriptType::` across the crate; extend any extra sites. |
| R3 | Wallet-import-multiformat cycle (`.wallet-import-export-multiformat-kickoff.md`) may overlap P2 surface. | P2 SPEC scopes to "descriptor-string + md1 + BIP-388 JSON intake only"; BSMS/Specter-JSON/Sparrow-JSON wrappers remain wallet-import's job. Coordination clause in plan §1 L9. |
| R4 | `rust-bitcoin` `Xpriv` lacks Drop+Zeroize → per-iteration xprvs in candidate loop linger. | Existing FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`; no new mitigation. |
| R5 | `--add-path` substitution edge cases (whitespace, multi-`account` tokens, escaped `'`). | Lock-in §3.2 step 4: substitute first occurrence of `account'` then `account`; multi-occurrence requires repeat `--add-path` flags. Tests cover. |
| R6 | Descriptor auto-detect ambiguity (e.g., a string containing both `@0` and literal `xpub6...`). | `--descriptor-from <node>=<value>` explicit-form available. Cells exercise the disambiguation. |
| R7 | Toolkit crates.io publish remains blocked on miniscript `[patch.crates-io]`. | Not gating; git-tag only this cycle as prior cycles. |
| R8 | BIP-388 JSON parser is net-new; potential security-sensitive parser (arbitrary JSON intake). | Parse via `serde_json::from_str::<BipPolicyJson>` with a strictly-typed struct (no `serde_json::Value` catch-alls); reject unknown keys via `#[serde(deny_unknown_fields)]`. Phase 0 recon cell. |
| R9 | C2 commit (phase P2 account-of-descriptor) is materially larger than C3/C4 commits (4 descriptor shapes + auto-detect + BIP-388 JSON struct + unified funnel + ~14 test cells). | R1 I-7 acknowledged. Mitigation: per-phase reviewer-loop allows more rounds (R0→R1→R2→...) for C2; intake parsers + integration discipline (parsers first, search dispatch second within the same commit; TDD cells for parsers ship as compile-time scaffold before integration). No commit split (C2a + C2b is impractical given TDD-per-commit discipline — parser-only commit is non-shippable intermediate). |
| R10 | `XpubSearchNoMatch.searched: usize` JSON serialization (R1 m-3). | `usize` serializes as JSON number; safe up to 2^53 on 64-bit. Account ranges × templates × cosigners won't approach this; no explicit cap. One-line note. |

---

## §15 — FOLLOWUPs to file (anticipated)

| ID | Tier | Topic |
|----|------|-------|
| `xpub-search-passphrase-bruteforce` | v0.27 | File-based passphrase brute-force mode |
| `xpub-search-generated-passphrase-templates` | v0.27 | Generated candidate sets (year/birthday/custom) |
| `xpub-search-output-watchonly-bundle` | v0.27 | After match, emit watch-only bundle for matched account |
| `xpub-search-multi-target-batch-json` | v0.27 | Batch multiple targets per invocation (already partially supported in P3; extend to P1/P2/P4) |
| `xpub-search-bsms-specter-sparrow-wallet-import` | wallet-import-multiformat cycle | Wrapping wallet-file-format intakes around P2 |
| `inspect-json-schema-version-backfill` | v0.27 | Add `schema_version` field to `InspectJson` for parity with `XpubSearchJson` |
| `non-canonical-notice-helper-extract` | v0.27 | Extract `cmd/bundle.rs:1367-1388` notice emitter into shared module if a 3rd caller surfaces |
| `descriptor-literal-xpub-extraction-primitive` | v0.27 | Promote `for_each_key`-extraction helper used in P2 to shared module if 2nd caller surfaces |
| Per-phase R0..Rn surfaced items | per-phase | TBD |

---

## §16 — Plan-mode artifact disposition

This plan file (`/home/bcg/.claude/plans/woolly-spinning-honey.md`) is the **plan-mode source of truth**. On `ExitPlanMode` approval, C6 commits a verbatim copy to `design/PLAN_v0_26_0_xpub_search.md` as the canonical project record (matching `design/PLAN_v0_19_0_non_canonical_descriptors.md` precedent).

End of plan.
