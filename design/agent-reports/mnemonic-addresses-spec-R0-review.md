# R0 Architect Review — SPEC_mnemonic_addresses.md (toolkit 0.38.0)

Reviewer: feature-dev:code-reviewer (opus). Reviewed against real source @ branch
`mnemonic-addresses-subcommand` (master `a9b30ac`). Version confirmed `0.37.11`. Verified every §2
citation, derivation/render/network APIs, dedup callers, panic class, advisory boundary, lockstep.

Core derivation spine (`ScriptType→CliTemplate`→`derive_bip32_from_entropy`→
`account_xpub.derive_pub(m/c/i)`→render) is correct; dedup behavior-preserving; panic-guard +
advisory-non-fire reasoning hold.

## Critical — None.

## Important

**I1 — §3.2/§2 dedup omits the THIRD `network_from_xpub` duplicate; reuse needs a visibility decision.**
The render-fn dedup count (3 callers: convert.rs:1291/:1343, address_search.rs:87) is correct. But
`network_from_xpub` exists in THREE places: `convert.rs:1616` (SPEC cites `:1611` — wrong) AND a
verbatim copy at `xpub_search/address_of_xpub.rs:359` (its doc-comment says "Mirror of …
convert.rs::network_from_xpub (private there)"). convert's is **private** (`fn`), so `addresses`
cannot call it as §2 "Reuse" implies. **Fix:** §3.2 also dedups `network_from_xpub` into the new
shared module as `pub(crate)` (re-point 3 sites: convert.rs:1616, address_of_xpub.rs:359, new
addresses.rs), OR explicitly widen convert's to `pub(crate)`. Fix §2 line cite to `:1616`.

**I2 — §3.1 "reuse the parser" for `--from` understates the work: secret resolution is NOT reusable.**
Reusable (`pub`/`pub(crate)`): `parse_from_input`/`FromInput`/`NodeType` (convert.rs:136/31),
`read_stdin_to_string`/`read_stdin_passphrase` (`:706/:719`), `seedqr::decode`,
`env_sentinel::resolve_env_var_sentinel`. NOT reusable: `resolve_env_sentinels`/
`needs_env_sentinel_resolution` (convert.rs:1689/1668, **private**, typed on `ConvertArgs`), the
single-stdin mutual-exclusion locks, seedqr inline substitution, `is_side_input_only`.
`parse_from_input` does ZERO resolution (only splits `<node>=<value>`). So `addresses` must
re-implement env-sentinel resolution + stdin-`-` + seedqr→phrase decode over the reusable primitives.
**Fix:** §3.1 replace "reuse the parser … per existing convert hygiene" with an explicit enumeration
(reuse `parse_from_input`/`NodeType` for parsing; re-implement the resolution loop over
`resolve_env_var_sentinel` / `read_stdin_to_string` / `seedqr::decode`); add §5 cells for `@env:`,
stdin `-`, and `seedqr=` (currently no `@env:`/stdin cell despite §3.1 promising the channels).

## Minor
- **M1** — §2 line drift: `ScriptType` at `convert.rs:357` (SPEC says `:356`); `network_from_xpub` at
  `:1616` (SPEC says `:1611`). Re-grep, use live lines.
- **M2** — §3.1 main-vs-test kind guard "(the mk pattern)" has NO in-repo precedent — convert's
  xpub→address (convert.rs:1342) silently honors a mismatched `--network` with no check. The stricter
  `BadInput` is defensible but §3.1 should note it diverges from convert. For SEED sources the guard
  is unneeded (network drives coin-type; `derive_bip32_from_entropy` self-checks at derive_slot.rs:91).
- **M3** — §4 README count: both READMEs need the count bump (twenty→twenty-one, 20 Command arms
  confirmed); `readme_version_current.rs` gates only the version STRING, not the prose count. Note the
  crate README line 10 already reads "v0.36.x" (pre-existing drift the implementer will touch).
- **M4** — §3.1 ceiling-guard framing: `ChildNumber::from_normal_idx` returns `Result` (panics via
  `.unwrap()`, not bare). Guard is correct regardless; pin the EXACT inequality: `--count N` → last
  index `N-1`, valid iff `N ≤ 2^31` (so `--count 2147483648` is VALID, `2147483649` rejected);
  `--range A,B` → `B < 2^31` && `A ≤ B`. Add the `2147483648` boundary edge to §5.

## Confirmed correct (no action)
ScriptType→CliTemplate inverse; `derivation_path` depth-3; `derive_bip32_from_entropy` returns
`DerivedAccount.account_xpub` composing to BIP-44/84 addresses; dedup byte-identical (p2pkh by-value,
others by-`&`); xpub-`--account`/`--passphrase` hard `BadInput`; non-English advisory non-fire (convert
comment confirms derived targets don't fire); SemVer 0.38.0; no new ToolkitError variant (BadInput);
gui-schema reflective; test oracle (`convert --to address` leaf-from-master) genuinely independent.

**VERDICT: RED (0C/2I)**

---

## Fold applied (controller, verified @ a9b30ac)
- **I1:** confirmed `network_from_xpub` private (convert.rs:1616) + 3rd copy (address_of_xpub.rs:359).
  §3.2 extended to dedup `network_from_xpub` (3 sites) into the shared `address_render` module as
  `pub(crate)`. §2 line fixed to `:1616`.
- **I2:** confirmed resolution machinery private/ConvertArgs-typed. §3.1 `--from` rewritten: parse via
  `parse_from_input`/`NodeType`; re-implement resolution over `resolve_env_var_sentinel` /
  `read_stdin_to_string` / `seedqr::decode`. §5 adds `@env:`/stdin/`seedqr=` cells.
- **M1** line cites fixed (ScriptType :357, network_from_xpub :1616). **M2** divergence-from-convert
  note added. **M3** crate-README-stale note + count in both. **M4** exact ceiling inequality pinned +
  boundary edge cell.
