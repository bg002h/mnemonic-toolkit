# SPEC: mnemonic-toolkit v0.3 — user-supplied descriptor passthrough

**Date:** 2026-05-05
**Convention:** delta over `SPEC_mnemonic_toolkit_v0_2.md` (which becomes canonical for all v0.2 invocations under a v0.3 binary). v0.1 SPEC remains canonical for v0.1 single-sig invocations under a v0.3 binary.
**Status:** approved 0C / 0I after iterative architect-review (rounds 1–6); see §11 revision history.

## §1. Scope (delta)

v0.3 adds **one** feature: user-supplied BIP-388 descriptor passthrough.

**IN-SCOPE for v0.3:**
1. `--descriptor "<string>"` and `--descriptor-file <path>` flags accept any BIP-388 descriptor whose miniscript AST is supported by toolkit's extended walker (defined in §4.9). The extended walker covers the full `md_codec::tag::Tag` enum (36 variants) modulo the deferred items in §8. Specifically: hash terminals (`sha256`, `hash160`, `hash256`, `ripemd160`), timelocks (`after`, `older`), miniscript wrappers (`c:`, `v:`, `s:`, `a:`, `d:`, `j:`, `n:`), boolean operators (`and_v`, `and_b`, `andor`, `or_b`, `or_c`, `or_d`, `or_i`), and `thresh()`.
2. Mode-dispatch refactor: `BundleArgs::template` becomes `Option<CliTemplate>` with clap `required_unless_present_any = ["descriptor", "descriptor_file"]`; runtime mode-violation pre-check enforces exactly-one of `--template` / `--descriptor` / `--descriptor-file`.
3. Bundle synthesis honors the parsed descriptor's structure (single-sig if n=1, multisig if n≥2 with `@N` placeholders).
4. `verify-bundle` extension: descriptor-derived bundles round-trip via the original descriptor string preserved in the bundle JSON.
5. `--privacy-preserving` and `--self-check` carry from v0.2 unchanged.
6. `--threshold` / `--cosigner-count` / `--multisig-path-family` are mode-violation errors when `--descriptor` is set (descriptor encodes structure directly). `--account` is accepted as `--account 0` (no-op) for backwards-compatible script ergonomics; `--account != 0` is a mode-violation error (descriptor encodes account index in `@N` origin path, not via `--account`). Exact rules in §2.1.13 and §6.9.
7. Cosigner xpub source for `@N≥1`: reuse v0.2's `--cosigner` / `--cosigners-file`. `@0` is bound to the local seed (full mode) or to a `--xpub` (single-sig watch-only `@0`-only descriptors), per §4.11.

**Still-in-scope from v0.1 + v0.2** (carry unchanged): all template-mode invocations remain byte-identical.

**EXPLICITLY DEFERRED to v0.4+:**
- K-of-N share encoding (gates on `ms-codec v0.2`).
- `--output <dir>` (per-card files).
- Recovery flow (3 strings → wallet artifact).
- Multi-leaf taproot trees (`tr(@0, { leaf1, leaf2, ... })` with N≥2 leaves) — Merkle-root computation out of scope; walker rejects with mode-violation error per §6.8.
- Color / interactive prompts (forbidden indefinitely per v0.1 §3.3).
- Backport toolkit's expanded walker to `md-cli` (cross-repo FOLLOWUP at v0.4-cross-repo tier; described in §10).

## §2. Command surface (delta)

### §2.1.10 (NEW) `--descriptor "<string>"` / `--descriptor-file <path>`

Two mutually-exclusive ways to supply a BIP-388 descriptor:
- `--descriptor "wsh(and_v(v:pk(@0/<0;1>/*),sha256(<32B-hex>)))"` — direct argument.
- `--descriptor-file path/to/desc.txt` — single-line UTF-8 file containing the descriptor; trailing newline tolerated.

Mutual exclusivity: `--descriptor` XOR `--descriptor-file` (clap-level `conflicts_with`); either of these is mutually-required-one-of with `--template` (runtime pre-check, exit 2).

The descriptor MUST contain ≥1 `@N` placeholder. `@N` indices MUST be dense over `0..n` (matches md-cli's `resolve_placeholders` invariant).

### §2.1.11 (NEW) Mode dispatch under `--descriptor`

`bundle.rs:51` declaration changes:
```rust
pub template: Option<CliTemplate>,  // was: pub template: CliTemplate
```
with clap attribute `#[arg(required_unless_present_any = ["descriptor", "descriptor_file"])]`. The `--descriptor` flag itself uses `#[arg(conflicts_with = "descriptor_file")]` to enforce its XOR with `--descriptor-file`. Runtime pre-check (in addition to clap-level): exactly one of `--template` / `--descriptor` / `--descriptor-file` MUST be present after clap parsing; violations exit 2 with the message in §6.9.

Mode determination from descriptor (single-sig vs multisig vs watch-only) follows §4.10.

### §2.1.12 Cosigner xpub source for `@N`

Reuses v0.2's flag matrix verbatim:
- `@0`: bound to `--phrase`-derived xpub (full mode) or `--xpub` (single-sig watch-only when descriptor has only `@0` and no other placeholder).
- `@N` for N≥1: bound to the N-th cosigner from `--cosigner` (repeatable) or `--cosigners-file`.

If the descriptor uses `@N` for N≥1 and no `--cosigner` / `--cosigners-file` is supplied, exit 2 with message: `descriptor uses @{N} but no cosigner xpub provided; supply --cosigner or --cosigners-file with at least {N} entries.`

### §2.1.13 Mode-violation deltas under `--descriptor`

The following flag combinations are mode-violation errors (exit 2):
- `--descriptor` with `--template`
- `--descriptor` with `--threshold`
- `--descriptor` with `--cosigner-count`
- `--descriptor` with `--multisig-path-family`
- `--descriptor` with `--account` and `--account != 0`

Rationale: descriptor encodes structure (template, K, N, paths, account index) directly; these flags would conflict.

Exact pre-check messages: see §6.9.

### §2.1.14 `--privacy-preserving` / `--self-check` / `--passphrase` / `--language` carry-forward

All four flags carry from v0.2 unchanged. `--privacy-preserving` strips mk1 fingerprints under descriptor mode the same way as multisig watch-only mode. `--self-check` runs verify-bundle's check ladder against the synthesized bundle.

`--passphrase` and `--language` apply to `--phrase`-derived seeds the same way.

### §2.2.3 (NEW) `verify-bundle` for descriptor-derived bundles

Descriptor-derived bundles include a new top-level `descriptor` field in their JSON output (§5.6). `verify-bundle` re-parses this field through the same walker, recomputes md1/mk1 expectations, and runs the existing 9 / 3+6N check schema. The check schema is **unchanged** for descriptor mode; only the *source* of the wallet-policy-id is the preserved descriptor string instead of a `--template` enum.

### §2.3 `--help` text delta

Add help text for `--descriptor` and `--descriptor-file` that names a small example and references the BIP-388 spec.

## §3. Input/output discipline

No delta from v0.2.

## §4. Bundle synthesis rules (delta)

### §4.9 (NEW) Descriptor parsing pipeline

Toolkit-local file `crates/mnemonic-toolkit/src/parse_descriptor.rs` implements:

1. **Read**: `--descriptor "<string>"` (UTF-8) or `--descriptor-file <path>` (read + trim trailing newline).
2. **Lex placeholders**: identical regex to md-cli `parse/template.rs:19-27`: `@(\d+)((?:/\d+'?)*)(?:/<([0-9;]+)>)?(/\*(?:'|h)?)?`.
3. **Resolve placeholders**: density check `0..n`; build `PathDecl` (Shared vs Divergent).
4. **Substitute synthetic xpubs**: deterministic synthetic xpubs computed identically to md-cli's `synthetic_xpub_for` at `parse/template.rs:246-269`, with one normative change: the seed prefix is `b"toolkit-v0.3"` (not md-cli's `b"md-v0.15"`). Specifically: `seed = sha256(&[b"toolkit-v0.3", i, depth].concat())`; `chain_code = sha256(&[b"cc", i, depth].concat())`; pubkey is derived from `seed` as a secp256k1 secret key, serialized compressed; xpub assembled as a BIP-32 base58check string. Depth byte is 3 for `ScriptCtx::SingleSig` and 4 for `ScriptCtx::MultiSig` (matches md-cli). The prefix CHOICE is normative for fixture stability (test vectors must reproduce); the prefix VALUE does not affect encoded card strings (synthetic xpubs are looked up by string equality in `key_map` and replaced by real user-supplied keys before TLV encoding — verified by reading `lookup_key()` and TLV-population code in md-cli's `parse_template`).
5. **`miniscript::Descriptor::from_str()`**: invoke rust-miniscript v13 parser.
6. **`walk_root` + extended `walk_miniscript_node`**: traverse miniscript AST, emit `md_codec::tree::Node`. Toolkit's walker MUST cover all `Terminal` variants in the rust-miniscript v13 surface that map to a `md_codec::tag::Tag` variant in the supported set (see §4.9.a). Variants outside the supported set produce `descriptor parse error: unsupported miniscript fragment: <fragment>` (exit 2; per §6.7).
7. **Build `Descriptor`**: identical to md-cli `parse/template.rs:637-664`.

#### §4.9.a Supported `Terminal` variants

The toolkit walker handles every variant from md-cli's existing walker PLUS the v0.3-NEW arms. Full list (referencing rust-miniscript v13 `miniscript::miniscript::decode::Terminal`):

**Layer 1 — Descriptor-wrapper level (handled in `walk_root` / `walk_wsh_inner` / `walk_sh` / `walk_tr`, NOT inside `walk_miniscript_node`):**

These are routed by the wrapper-level walker and never appear as `miniscript::miniscript::decode::Terminal` arms:

- `Wpkh(@0)` → `Tag::Wpkh` — already in md-cli; carried.
- `Pkh(@0)` → `Tag::Pkh` — already in md-cli; carried.
- `Wsh(WshInner::Ms(...))` → recurses into `walk_miniscript_node`.
- `Wsh(WshInner::SortedMulti(sm))` → `Tag::SortedMulti` (already in md-cli; carried; reuses `build_multi_node`).
- `Sh(ShInner::Wsh(...))` → `Tag::Sh` wrapping `Tag::Wsh`; recurses.
- `Sh(ShInner::Wpkh(...))` → `Tag::Sh` wrapping `Tag::Wpkh`.
- `Sh(ShInner::SortedMulti(sm))` → `Tag::Sh` wrapping `Tag::SortedMulti`.
- `Sh(ShInner::Ms(...))` → `Tag::Sh` wrapping `walk_miniscript_node` result. (NEW — md-cli currently rejects.)
- `Tr(t)` with 0 leaves → `Tag::Tr` keypath-only (already in md-cli; carried).
- `Tr(t)` with single-leaf miniscript → `Tag::Tr` + `Tag::TapTree` wrapping `walk_miniscript_node` result (already in md-cli for limited subset; carried).
- `Tr(t)` with single-leaf `sortedmulti_a(k, @0, @1, ...)` → **deferred to v0.4 pending upstream parser support**; rust-miniscript v13.0.0 has no parser for `sortedmulti_a` in tap-leaves (no `Terminal::SortedMultiA` arm; no Layer-1 routing in `descriptor/tr.rs`). The wire-format opcode `Tag::SortedMultiA` remains reserved in md-codec and is reachable in v0.4 once upstream lands the fragment. Pre-Phase-A SPIKE resolved this; see `design/agent-reports/spike-toolkit-v0_3-pre-phaseA.md` §1 and FOLLOWUP `tr-sortedmulti-a-via-upstream`. Users wanting sorted-key tap multisig in v0.3 can pre-sort cosigner keys lexicographically and use plain `multi_a(...)` (script-equivalent for new wallets; lossy for backing up an existing `sortedmulti_a` wallet whose keys are not already sorted).
- `Tr(t)` with ≥2 leaves → exit 2 per §6.8 (deferred to v0.4).

**Layer 2 — `Terminal` arms inside `walk_miniscript_node` (rust-miniscript v13's `miniscript::miniscript::decode::Terminal` enum):**

Already handled by md-cli's walker (carried verbatim):
- `Terminal::PkK(pk)` → `Tag::PkK`
- `Terminal::PkH(pk)` → `Tag::PkH`
- `Terminal::Multi(thresh)` → `Tag::Multi` (via `build_multi_node`)
- `Terminal::MultiA(thresh)` → `Tag::MultiA` (via `build_multi_node`). In v0.3 the walker emits `Tag::MultiA` unconditionally because rust-miniscript v13.0.0 has no `sortedmulti_a` parser (sortedness disambiguation is moot until v0.4; see Layer 1 above and FOLLOWUP `tr-sortedmulti-a-via-upstream`).
- `Terminal::Check(inner)` → `Tag::Check` (with tap-context collapse to `PkK`/`PkH`)

v0.3-NEW arms:
- `Terminal::After(n)` → `Tag::After` (u32 body)
- `Terminal::Older(n)` → `Tag::Older` (u32 body)
- `Terminal::Sha256(h)` → `Tag::Sha256` (32-byte body)
- `Terminal::Hash256(h)` → `Tag::Hash256` (32-byte body; extension)
- `Terminal::Hash160(h)` → `Tag::Hash160` (20-byte body)
- `Terminal::Ripemd160(h)` → `Tag::Ripemd160` (20-byte body; extension)
- `Terminal::RawPkH(h)` → `Tag::RawPkH` (20-byte body; extension)
- `Terminal::False` → `Tag::False` (extension)
- `Terminal::True` → `Tag::True` (extension)
- `Terminal::Verify(inner)` → `Tag::Verify`
- `Terminal::Swap(inner)` → `Tag::Swap`
- `Terminal::Alt(inner)` → `Tag::Alt`
- `Terminal::DupIf(inner)` → `Tag::DupIf`
- `Terminal::NonZero(inner)` → `Tag::NonZero`
- `Terminal::ZeroNotEqual(inner)` → `Tag::ZeroNotEqual`
- `Terminal::AndV(a, b)` → `Tag::AndV`
- `Terminal::AndB(a, b)` → `Tag::AndB`
- `Terminal::AndOr(a, b, c)` → `Tag::AndOr`
- `Terminal::OrB(a, b)` → `Tag::OrB`
- `Terminal::OrC(a, b)` → `Tag::OrC`
- `Terminal::OrD(a, b)` → `Tag::OrD`
- `Terminal::OrI(a, b)` → `Tag::OrI`
- `Terminal::Thresh(k, subs)` → `Tag::Thresh`

`SortedMulti` does not appear as a `Terminal` arm; it lives as `WshInner::SortedMulti` / `ShInner::SortedMulti` and is handled at Layer 1 above. `SortedMultiA` is BIP-388 grammar but unreachable in v0.3 (deferred to v0.4 — see Layer 1 above).

**Out-of-scope (rejected with mode-violation per §6.8):**
- Multi-leaf taproot trees (`tr(@0, { leaf1, leaf2 })` with ≥2 leaves) — Merkle-root logic deferred to v0.4.

### §4.10 Mode determination from descriptor

After parsing yields `Descriptor { n, .. }`:
- **`n == 1` (descriptor has only `@0`) → single-sig mode**, regardless of outer wrapper. This rule is uniform: `wpkh(@0/...)`, `pkh(@0/...)`, `tr(@0)`, AND `wsh(pk(@0))` / `wsh(multi(1,@0))` / `wsh(and_v(v:pk(@0),sha256(...)))` / `sh(wpkh(@0/...))` are ALL single-sig. Rationale: a 1-of-1 multisig is degenerate; tooling matches the user's likely mental model (`@0` = my key) over the wrapper-implied script context. Verify-bundle's check schema for n=1 reduces to the 9-element single-sig form irrespective of outer wrapper.
- **`n ≥ 2` → multisig mode.** Always. Includes wrappers like `tr(@0, sortedmulti_a(...))` (deferred to v0.4 — see §4.9.a), `wsh(multi(2,@0,@1,@2))`, etc.

**Tree-faithfulness invariant (load-bearing):** the "mode" label (single-sig vs multisig) controls (a) key-sourcing — which flags supply `@0` vs `@N≥1` xpubs, (b) the number of mk1 cards emitted (one for single-sig, n for multisig), and (c) the verify-bundle check schema's element count (9 vs 3+6N). It does NOT control the `Descriptor.tree` structure: the parsed tree is ALWAYS faithful to the user-supplied descriptor. For a degenerate `wsh(multi(1,@0))`, the tree is `Tag::Wsh → Tag::Multi(k=1, n_keys=1)` and the encoder emits exactly that — implementations MUST NOT collapse `multi(1,@0)` to `pk(@0)` in the tree.

Mode → seed/key sourcing rules:
- **Single-sig + `--phrase`** → full single-sig synthesis. `@0` xpub derived from the seed at the descriptor's annotated origin path (see §4.11).
- **Single-sig + `--xpub`** → watch-only single-sig synthesis. `@0` bound to `--xpub`.
- **Multisig + `--phrase` + `--cosigner`/`--cosigners-file`** → full multisig synthesis. `@0` from seed (at `@0`'s annotated origin path); `@N≥1` from cosigner triples.
- **Multisig + `--xpub`** → exit 2 (`--xpub` is single-sig watch-only; multisig watch-only uses `--cosigner`).
- **Multisig + `--cosigner`/`--cosigners-file` covering ALL `@N` (including `@0`) + no `--phrase` no `--xpub`** → watch-only multisig synthesis. The cosigner at `@0` slot is the "self" cosigner.
- **Multisig with no key source for `@0`** (no `--phrase`, no cosigner at `@0` index) → exit 2 (`descriptor uses @0 but no key source provided; supply --phrase OR a cosigner triple bound to @0`).

### §4.11 `@N` xpub binding data flow

For each placeholder `@N` in the parsed descriptor, the toolkit assembles a `ParsedKey { i: N, payload: xpub_to_65(&xpub) }` and a matching `ParsedFingerprint { i: N, fp: <4-byte-fp> }`, then calls `parse_descriptor(input, &keys, &fingerprints)`. The `parse_descriptor` function populates `tlv.pubkeys` (sorted by `i`) and `tlv.fingerprints` (sorted by `i`) verbatim from these slices — identical to md-cli's `parse_template` final assembly at `parse/template.rs:637-664`.

**Origin-path annotations.** The toolkit-supported descriptor syntax accepts the BIP-388 `@N[fp/path]/<multipath>/*` form. The `[fp/path]` annotation is REQUIRED for `@N` slots that need derivation (full mode) and OPTIONAL but allowed for watch-only slots (informational; cross-checked against the cosigner triple). `path` MUST be an absolute path starting from master (e.g., `48'/0'/0'/2'`), with hardened steps suffixed `'`.

**Divergence from md-cli (deliberate).** md-cli's `parse_template` accepts an `@i` placeholder with no `[fp/path]` annotation; in that case `PathDecl` is built with an empty origin path. Toolkit v0.3 TIGHTENS this for full-mode `@0` (and any full-mode `@N≥1` derived from the same seed under a self-multisig configuration): the annotation MUST be present, and `fp` is cross-checked against the seed-derived master fingerprint. Rationale: full-mode key derivation needs an explicit BIP-32 path; absent annotation would force the toolkit to invent a default (e.g. m/84'/0'/0') and silently produce different bundles for users who haven't read the path-default policy. Watch-only modes do NOT have this constraint (xpub already supplies the key); annotation if present is informational and cross-checked, but absence is permitted.

**Per-slot rules:**

- **`@0` in full single-sig mode (`--phrase` set):**
  - The descriptor's `@0[fp/path]` annotation MUST be present and supply both `fp` and `path`.
  - Toolkit derives the xpub from the seed at `path` → builds `ParsedKey { i: 0, payload: xpub_to_65(&derived_xpub) }`.
  - The derived xpub's master fingerprint MUST match the annotation's `fp`; mismatch → exit 2 (`@0 origin fingerprint annotation does not match seed master fingerprint`).
  - Origin path annotation missing → exit 2 (`@0 in full single-sig descriptor mode requires explicit [fp/path] origin annotation`).

- **`@0` in watch-only single-sig mode (`--xpub` set, n==1):**
  - The user-supplied `--xpub` is parsed and converted via `xpub_to_65` → `ParsedKey { i: 0, payload }`.
  - `--master-fingerprint` provides `fp` → `ParsedFingerprint { i: 0, fp }`.
  - If the descriptor has `@0[fp_anno/path]` annotation and `fp_anno != --master-fingerprint`, exit 2 (`@0 origin fingerprint annotation does not match --master-fingerprint`).
  - Path annotation present but doesn't match `--master-fingerprint`'s implied derivation? Toolkit cannot verify path consistency from xpub-only input; the annotation is informational and accepted as-is.

- **`@0` in full multisig mode (`--phrase` set + cosigners cover `@N≥1`):**
  - Same rule as full single-sig: `@0[fp/path]` annotation REQUIRED; toolkit derives at `path`; fingerprint cross-check.

- **`@0` in watch-only multisig mode (no `--phrase`, no `--xpub`, cosigners cover ALL `@N` including `@0`):**
  - The cosigner triple at `@0` index supplies xpub + fp + path.
  - If the descriptor has `@0[fp/path]` annotation, MUST agree with the cosigner triple's fp and path; mismatch → exit 2 (`@0 origin annotation does not match cosigner-triple at index 0`).

- **`@N≥1` (always cosigner-sourced):**
  - The cosigner triple `<xpub>:<fp>:<path>` (from `--cosigner` or `--cosigners-file`) at index N supplies xpub, fingerprint, and origin path.
  - If the descriptor has `@N[fp_anno/path_anno]` annotation, MUST agree with the cosigner triple; mismatch → exit 2 (`@{N} origin annotation does not match cosigner-triple at index {N}`).
  - Annotation may be elided; cosigner triple is then the source of truth.

**Cosigner-list size invariants:**
- In full multisig mode (`--phrase` set + cosigners cover `@N≥1`): `@0` is supplied by `--phrase`; required `--cosigner` count is `n - 1`. Mismatch → exit 2.
- In watch-only multisig mode (no `--phrase`, no `--xpub`, cosigners cover ALL `@N` including `@0`): required `--cosigner` count is `n`. Mismatch → exit 2.
- In full single-sig mode (`n == 1` + `--phrase`): no cosigners required; supplying any `--cosigner` is a mode-violation (covered by §6.9 cosigner-count-mismatch row).
- In watch-only single-sig mode (`n == 1` + `--xpub`): no cosigners required; same constraint.
- `--cosigner` indexing is positional: in full multisig, the first `--cosigner` → `@1`, the next → `@2`, etc. In watch-only multisig, the first `--cosigner` → `@0`, the next → `@1`, etc.

**SELF-MULTISIG WARNING applicability:** unchanged from v0.2 §4.7 — fires when `n≥2` AND `--phrase` is set AND any cosigner triple's xpub equals a derivation from the same seed (toolkit-detected via xpub equality after derivation at the cosigner's path).

### §4.12 SELF-MULTISIG WARNING applicability

(See §4.11 closing paragraph — applicability rule lives there for proximity to the `@N` binding logic.)

## §5. Output format (delta)

### §5.6 (NEW) `bundle --json` schema delta

`schema_version` bumps from `"2"` to `"3"`. The new top-level field `descriptor` is added; the existing `template` field becomes nullable.

```json
{
  "schema_version": "3",
  "mode": "full" | "watch-only",
  "template": "bip84" | "wsh-sortedmulti" | ... | null,
  "descriptor": "<original user-supplied descriptor string, verbatim>" | null,
  ...
}
```

**Concrete struct change (`crates/mnemonic-toolkit/src/format.rs`):**
- `pub template: &'static str` → `pub template: Option<&'static str>` (currently at `format.rs:111`).
- New field: `pub descriptor: Option<String>` — emitted always; `None` for template-mode invocations.
- Default serde `Option` serialization: `Some(x)` emits `x`; `None` emits `null`. Do NOT use `#[serde(skip_serializing_if = ...)]` on either field — both fields MUST always be present in the JSON output (the schema discriminates by which is null, not by presence).
- All existing emit call sites that wrote `template: args.template.human_name()` change to `template: args.template.as_ref().map(|t| t.human_name())`. Descriptor-mode emit sets `template: None` and `descriptor: Some(<user-supplied string>)`. Template-mode emit sets `descriptor: None`.

**Wire-bit-identical guarantee for v0.2 invocations:** any v0.2 invocation under v0.3 binary emits identical card strings (ms1/mk1/md1) AND nearly identical JSON (only `schema_version` changes from `"2"` to `"3"` and a new `"descriptor": null` field appears; existing fields including `"template": "bip84"` are byte-identical). Per v0.2 §9.4.1 precedent, JSON envelope deltas are explicitly outside the wire-bit-identical claim — only encoded card strings count.

**Wire-bit-identical guarantee for descriptor-mode equivalents (conditional):** when the user supplies a descriptor that exactly expresses a v0.2 template AND identical `@N[fp/path]` annotations matching the v0.2 path-derivation rules AND identical cosigner triples (e.g., `--descriptor "wsh(sortedmulti(2,@0[fp/48'/0'/0'/2']/<0;1>/*,@1[fp/48'/0'/0'/2']/<0;1>/*))"` matching `--template wsh-sortedmulti --threshold 2 --cosigner-count 2 --cosigner ...`), the emitted ms1/mk1/md1 card strings MUST be byte-identical between descriptor-mode and template-mode invocations. SPEC fixture row D.2 (§10) tests representative inputs. The guarantee is conditional on user-controlled inputs; the toolkit does NOT enforce that descriptor-mode reproduces template-mode output for arbitrary descriptors that don't satisfy the conditions.

**`MultisigInfo` struct fields under descriptor mode (delta over v0.2 §5.3):** v0.2's `MultisigInfo` has `template: &'static str` and `path_family: &'static str`. For descriptor-mode multisig bundles (n≥2 from descriptor), these are populated with the literal static string `"descriptor"` for both fields (no canonical template / path family applies). `threshold` is derived from the descriptor: for `Tag::Multi(k,...)` / `Tag::SortedMulti(k,...)` / `Tag::MultiA(k,...)` / `Tag::SortedMultiA(k,...)` and `Tag::Thresh(k,...)`, threshold = `k`; for compositions where there is no clean K (e.g., `or_d`, `andor` with no top-level `thresh`), threshold = `n` (placeholder count). `cosigner_count` = `n`. `cosigners` array shape unchanged from v0.2 (per-`@N` index, sorted, with xpub/fp/path from §4.11 binding).

`"engraving_card"`, `"ms1"`, `"mk1"`, `"md1"`, `"network"`, `"origin_path"`, `"master_fingerprint"`, `"account"`, `"privacy_preserving"` all carry from v0.2 unchanged.

### §5.7 (NEW) `verify-bundle --json` schema delta

`verify-bundle` accepts schema_version "3" bundles. When `descriptor != null`, the verifier:
1. Re-parses the preserved descriptor string through the same `parse_descriptor.rs` pipeline.
2. Recomputes the expected md1/mk1 contents.
3. Runs the existing 9 / 3+6N check schema; `md1_wallet_policy` check passes iff the recomputed wallet-policy-id matches the bundle's md1 decoded value.

The check array remains structurally identical to v0.2 (3+6N for multisig). New error variant `descriptor_reparse_failed` (exit 4) for when the preserved descriptor no longer round-trips (e.g. user manually edited the JSON).

### §5.5 Error JSON envelope

No delta.

## §6. Errors and exit codes (delta)

### §6.7 (NEW) Descriptor parse error (exit 2)

`descriptor parse failed: <miniscript-error-text>` — wraps any error from `MsDescriptor::from_str()` or the lex/resolve placeholder steps.

### §6.8 (NEW) Unsupported-fragment error (exit 2)

`unsupported miniscript fragment: <fragment-string>; v0.3 walker covers BIP-388 surface modulo multi-leaf tap trees (deferred to v0.4)` — for `Terminal` variants outside the v0.3 supported set (currently: only multi-leaf tap trees).

### §6.9 (NEW) Mode-violation deltas

| Trigger | Message |
|---|---|
| `--descriptor` with `--template` | `--descriptor and --template are mutually exclusive; pick descriptor passthrough or template, not both.` |
| `--descriptor` with `--descriptor-file` | `--descriptor and --descriptor-file are mutually exclusive; supply the descriptor inline or via file, not both.` |
| `--descriptor` with `--threshold` | `--threshold is meaningful only with a multisig --template; descriptor mode encodes K directly.` |
| `--descriptor` with `--cosigner-count` | `--cosigner-count is meaningful only with --template; descriptor mode encodes N from @i placeholder count.` |
| `--descriptor` with `--multisig-path-family` | `--multisig-path-family is meaningful only with --template; descriptor mode encodes paths directly via @i/path syntax.` |
| `--descriptor` with `--account` and `--account != 0` | `--account != 0 is meaningful only with --template; descriptor mode encodes account index in the @i origin path.` |
| Descriptor uses `@N` for N≥1 with no `--cosigner`/`--cosigners-file` | `descriptor uses @{N} but no cosigner xpub provided; supply --cosigner or --cosigners-file with at least {N} entries.` |
| `--descriptor` is empty / no `@N` placeholders | `descriptor must contain at least one @N placeholder.` |
| Multisig descriptor (n≥2) with `--xpub` | `--xpub is single-sig watch-only; for multisig watch-only, use --cosigner / --cosigners-file with no --phrase / --xpub.` |
| Multisig descriptor with no key source for `@0` | `descriptor uses @0 but no key source provided; supply --phrase OR a cosigner triple bound to @0.` |
| `@N` origin annotation contradicts cosigner triple | `@{N} origin annotation does not match cosigner-triple at index {N}.` |
| `@0` origin fingerprint annotation contradicts seed-derived fp (full mode) | `@0 origin fingerprint annotation does not match seed master fingerprint.` |
| `@0` origin annotation missing in full single-sig descriptor mode | `@0 in full single-sig descriptor mode requires explicit [fp/path] origin annotation.` |
| Full multisig mode: `--cosigner` count ≠ n - 1 | `full multisig descriptor mode requires {n}-1 = {n_minus_1} cosigner triples (--phrase supplies @0); got {actual} --cosigner triple(s).` |
| Watch-only multisig mode: `--cosigner` count ≠ n | `watch-only multisig descriptor mode requires {n} cosigner triples (one per @N); got {actual} --cosigner triple(s).` |

All enforced as **runtime pre-checks (exit 2)**, NOT as clap `conflicts_with`. Matches v0.2 §6.6 convention.

**Pre-check evaluation order:** the runtime ladder evaluates rows TOP-TO-BOTTOM in the order listed in the table. The first triggered row fires; subsequent rows short-circuit. This pins error-message determinism for cases where multiple rows could match (e.g. user supplies `--descriptor "wsh(multi(2,@0,@1))"` with no key sources at all → the "no cosigner for `@N≥1`" row triggers before the "no key source for `@0`" row, yielding the cosigner-list-required message).

## §7. Engraving guidance (no delta)

The engraving card emits metadata only (md1 not embedded; per format.rs analysis). Long descriptor-derived md1 strings render in the JSON output's `md1` chunked array via `md_codec::encode::render_codex32_grouped(s, 5)`. No card-width guard added.

## §8. Out-of-scope / deferred to v0.4+ (delta over v0.2 §8)

| Feature | v0.2 tier | v0.3 tier | v0.4+ tier |
|---|---|---|---|
| K-of-N share encoding | v0.3 (gates on ms-codec v0.2) | **v0.4+** (still gates) | unchanged |
| `--output <dir>` | v0.3 | **v0.4** | unchanged |
| Recovery flow | v0.3+ | **v0.4+** | unchanged |
| User-supplied descriptor | v0.3+ | **DELIVERED in v0.3** | — |
| Hash-locked / hybrid-miniscript descriptors | v0.3+ | **DELIVERED in v0.3** (subset of user-supplied) | — |
| Multi-leaf taproot trees | implicit v0.3+ | **v0.4** (Merkle-root logic out of scope for v0.3) | — |
| md-cli walker backport | n/a | **v0.4-cross-repo** (toolkit's expanded walker → md-cli or shared crate) | — |

## §9. Closures from brainstorm

- **Q1 (parser source-of-truth):** rust-miniscript v13 + extended AST walker in toolkit-local `parse_descriptor.rs`. Inline BIP-388 parser path rejected (much larger effort, no benefit).
- **Q2 (PR #935 contingency):** moot — v0.3 implements its own walker arms for hash terminals; does not depend on rust-miniscript's `WalletPolicy::translate_pk` fix. Hash-terminal handling is local to toolkit's walker. Pre-Phase-A SPIKE confirmed all four hash terminals (`sha256`/`hash256`/`hash160`/`ripemd160`) round-trip via `MsDescriptor::<DescriptorPublicKey>::from_str()` against rust-miniscript v13.0.0; see `design/agent-reports/spike-toolkit-v0_3-pre-phaseA.md` §2.
- **Q3 (flag shape):** both `--descriptor "<string>"` and `--descriptor-file <path>` shipped; mutually exclusive (clap `conflicts_with`).
- **Q4 (cosigner xpub source for @N):** reuse `--cosigner` / `--cosigners-file` from v0.2 verbatim.
- **Q5 (mode coexistence):** `--descriptor` is mutually-required-one-of with `--template`; `--threshold`/`--cosigner-count`/`--multisig-path-family`/`--account` are mode-violation errors when `--descriptor` is set.
- **Q6 (verify-bundle schema):** preserve descriptor string in JSON; verifier re-parses to recompute wallet-policy-id; check ladder structurally unchanged.
- **Q7 (wire-bit-identical):** every v0.2 invocation under v0.3 binary remains byte-identical to v0.2 output. SHA pin v0.2 corpus: `a381761656fd165e8e5af28574a5baaa55973e562c610254ae6f31d6b1f06171`.
- **Q8 (self-multisig vs production-multisig):** SELF-MULTISIG WARNING fires when `--descriptor` + `--phrase` + `n≥2` placeholders + cosigner xpub equals seed-derived xpub.
- **Q9 (descriptor length / engraving-card line-width):** no max-length guard. Engraving card emits metadata only; md1 chunking lives in md_codec's `render_codex32_grouped` (no toolkit-local width invariant). Per format.rs analysis.
- **Q10 (`--template` optionality):** `pub template: Option<CliTemplate>` with clap `required_unless_present_any = ["descriptor", "descriptor_file"]` (`--descriptor` and `--descriptor-file` themselves carry `conflicts_with` for their XOR). Runtime mode-violation pre-check for the overall mutually-required-one-of constraint. Integration tests: every existing `--template` test continues to work; v0.3 adds new tests for `--descriptor` mode.

## §10. Reference implementation (delta)

New files:
- `crates/mnemonic-toolkit/src/parse_descriptor.rs` — parsing pipeline; walker; `pub fn parse_descriptor(input: &str, keys: &[ParsedKey], fingerprints: &[ParsedFingerprint]) -> Result<md_codec::Descriptor, ToolkitError>`. Body mirrors md-cli's `parse_template.rs` with the extended walker arms (§4.9.a).

Modified files:
- `crates/mnemonic-toolkit/src/cmd/bundle.rs`:
  - `BundleArgs::template`: `CliTemplate` → `Option<CliTemplate>`.
  - New flags: `--descriptor`, `--descriptor-file`.
  - Mode-dispatch ladder gains a new `--descriptor` branch (calls `synthesize_descriptor`).
  - Pre-check ladder gains the §6.9 mode-violation cases.
- `crates/mnemonic-toolkit/src/synthesize.rs`:
  - New `synthesize_descriptor(parsed_descriptor, mode, key_sources, options)` (single entry point; dispatches single-sig vs multisig internally per §4.10).
  - Existing `synthesize_full` / `synthesize_multisig_full` / `synthesize_multisig_watch_only` factor a shared helper that takes a typed `Descriptor` and emits the bundle (descriptor mode reuses this helper after parsing).
- `crates/mnemonic-toolkit/src/format.rs`:
  - `BundleJson` struct (currently at `format.rs:111`) — `template: &'static str` becomes `Option<&'static str>`; new field `descriptor: Option<String>` added; `schema_version` bumps from `"2"` to `"3"`.
  - `verify_bundle` JSON intake reads schema `"2"` or `"3"`; for schema `"3"` with `descriptor != null`, re-parses via `parse_descriptor`.
- `crates/mnemonic-toolkit/Cargo.toml`:
  - New dep: `miniscript = { version = "13", default-features = false, features = ["std"] }`. Pinned to the same major version md-cli uses.
  - Existing `bip39` / `hex` / `bitcoin` / `clap` / `serde` / `serde_json` deps unchanged.

Tests (categories; cell counts pinned at IMPLEMENTATION_PLAN-time, not SPEC-time):

**A. Unit tests in `parse_descriptor.rs::tests`:**
1. One round-trip test (descriptor string → parse → encode → md1 → decode → typed `Descriptor` equality) per v0.3-NEW `Terminal` arm in §4.9.a Layer 2 list (24 arms in v0.3-NEW, ~24 unit tests). Each test fixes a small descriptor invoking that arm.
2. One round-trip test per Layer 1 wrapper (Wpkh/Pkh/Wsh-Ms/Wsh-SortedMulti/Sh-Wsh/Sh-Wpkh/Sh-SortedMulti/Sh-Ms/Tr-keypath/Tr-singleleaf, ~10 tests).
3. One placeholder-resolution edge-case test per §4.11 binding rule (≥6 tests covering `@0`-full-single-sig path, `@0`-watch-only-single-sig, `@0`-full-multisig, `@0`-watch-only-multisig, `@N≥1`-cosigner-bound, `@N≥1` annotation mismatch).
4. Synthetic-xpub determinism test: assert `synthetic_xpub_for(0, ScriptCtx::MultiSig)` produces a fixed expected base58check string (locks the `b"toolkit-v0.3"` prefix per §4.9 step 4).

**B. Mode-violation unit tests in `cmd/bundle.rs::tests` (or equivalent):**
- One test per §6.9 row (15 rows: 13 original + 2 from the cosigner-count split = 15 mode-violation tests). Each asserts exit code 2 + exact error message.

**C. Integration tests in `tests/cli/`:**
1. Descriptor-mode bundle output for each representative descriptor category (≥6 categories): hash-lock single-sig, timelock single-sig, threshold mix, AND/OR composition, hybrid hash+timelock, single-leaf taproot with miniscript.
2. Multi-cosigner via descriptor (descriptor with `@0`/`@1`/`@2`, three cosigner triples) — full and watch-only paths (2 tests).
3. Verify-bundle round-trip for each descriptor category in C.1 + C.2 (8 tests).

**D. Wire-bit-identical regression tests:**
1. v0.2 fixture matrix runs against v0.3 binary; expect 34/34 passing (v0.2 SHA pin: `a381761656fd165e8e5af28574a5baaa55973e562c610254ae6f31d6b1f06171`).
2. Descriptor-mode equivalents of v0.2 templates produce byte-identical card strings (≥3 fixtures: a single-sig BIP-84 expressed as descriptor, a wsh-sortedmulti expressed as descriptor, a tr-multi-a expressed as descriptor — each compared byte-for-byte against the template-mode emission for matching keys/cosigners).

**E. Fixture matrix extension:**
Total target: ≥40 v0.3-mode fixtures (covering A.1 + A.2 + C above plus mode-violation goldens). Plus 34 v0.2 carries from §D.1. New v0.3 SHA pin to be locked at v0.3 release prep phase.

## §11. Revision history

- 2026-05-05: Round 1 draft (in plan-mode review).
- 2026-05-05: Round 2 — addressed architect r1 verdict (3C / 6I / 6L); §4.9.a split into Layer 1 / Layer 2 (C-1); §4.10 mode-determination simplified to `n==1 → single-sig` (C-2); §4.11 expanded with explicit `@N` binding data flow (C-3); §5.6 pinned `Option<&'static str>` + serde defaults (I-1, I-3); §6.9 reconciled `--xpub` ban with §4.10 (I-2); `descriptor_source` dropped (I-4); §4.9 step 4 normatively pins synthetic xpub seed prefix (I-5); §10 test categorization expanded (I-6); L-1 / L-6 patched in-line.
- 2026-05-05: Round 3 — addressed architect r2 verdict (2C / 1I / 3L); §4.10 added tree-faithfulness invariant (N-1); §4.11 added md-cli divergence note for `@0` annotation requirement (N-2); §6.9 pinned pre-check evaluation order (N-3); §5.6 added `MultisigInfo` field-population rules for descriptor mode (N-4); §9 Q10 closure aligned to `required_unless_present_any` (N-5); §5.6 wire-bit-identical guarantee for descriptor-mode equivalents reworded as conditional (N-6 wording).
- 2026-05-05: Round 4 — addressed architect r3 verdict (0C / 1I / 4L); §1 item 6 narrowed to allow `--account 0` (NF-3); §4.9.a Layer 1 now explicitly names the `Tr(t) single-leaf sortedmulti_a → Tag::SortedMultiA` mapping (NF-1); §4.11 cosigner-list size invariants rewritten with explicit per-mode counts (NF-2); NF-4 / NF-5 routed to §12 follow-ups.
- 2026-05-05: Round 5 — addressed architect r4 verdict (0C / 1I / 2L); §6.9 row 14 split into two mode-specific rows with corrected user-facing messages (NF-R4-1 — "seed-self" eliminated from byte-exact error text); §4.9.a Layer 1 sortedmulti_a routing claim hedged as SPIKE-dependent with both branches described (NF-R4-2); Layer 2 `Terminal::MultiA` arm clarified to dispatch on sortedness flag with explicit warning against collapsing to `Tag::MultiA` (NF-R4-3).
- 2026-05-05: Round 6 — final cleanup. §10 category B mode-violation test count corrected from 13 to 15 (architect r5 NF-R5-1). SPEC at 0C / 0I.
- 2026-05-05: Round 7 (post-SPIKE) — pre-Phase-A SPIKE found rust-miniscript v13.0.0 has no parser for `sortedmulti_a` in tap-leaves; user approved option (c) "scope sortedmulti_a out of v0.3" with soft-deferral framing. Patches: §4.9.a Layer 1 `Tr-singleleaf-sortedmulti_a` bullet softened to "deferred to v0.4 pending upstream parser"; §4.9.a Layer 2 `Terminal::MultiA` paragraph narrowed to "emit `Tag::MultiA` unconditionally"; §4.9.a Layer 2 final note updated; §4.10 multisig-mode example keeps `tr(@0, sortedmulti_a(...))` with `(deferred to v0.4)` parenthetical (per user direction); §9 Q2 cites SPIKE report (closes FOLLOWUP `spike-report-citation`); §12 marks `spike-report-citation` resolved. New FOLLOWUP `tr-sortedmulti-a-via-upstream` at v0.4-cross-repo tier with two action items (file upstream issue + v0.4 kickoff gate). Plan A.4 round-trip count `≥11` → `≥10`; total exit subcount `≥58` → `≥57`. Architect r1 review of SPIKE report verdict 0C/2I/3L; all patches applied.

## §12. v0.3-tier FOLLOWUPS (open)

The following Low/nit items from architect review are filed in `mnemonic-toolkit/design/FOLLOWUPS.md` at appropriate tiers:

- `parse_template-regex-line-ref` (`v0.3-nice-to-have`): §4.9 step 2 cites `parse/template.rs:19-27` for the placeholder regex; the actual `Regex::new` call is at `:25-27`. Docs-only nit.
- `unsupported-fragment-error-style` (`v0.3-nice-to-have`): §6.8 error message text is a bit long for a CLI error; consider tightening at implementation time.
- `walker-backport-to-md-cli` (`v0.4-cross-repo`): toolkit's expanded walker should be backported to `md-cli` (or both should consume a shared crate).
- `spike-report-citation` (`v0.3`): RESOLVED — §9 Q2 now cites `design/agent-reports/spike-toolkit-v0_3-pre-phaseA.md` §2 (closed pre-Phase-A 2026-05-05).
- `synthesize-descriptor-fn-naming` (`v0.3`): the implementation plan resolved this to a single `synthesize_descriptor` entry point that dispatches single-sig vs multisig internally; see IMPLEMENTATION_PLAN_v0_3 Phase C.1.
- `v0.2-spec-§8-tier-citation` (`v0.3-nice-to-have`): §8 K-of-N tier citation against v0.2 SPEC §8 should be verified at implementation time for citation accuracy.
