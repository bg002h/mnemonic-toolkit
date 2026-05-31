# R1 Re-Review — SPEC_mnemonic_addresses.md (toolkit 0.38.0)

Reviewer: feature-dev:code-reviewer (opus). Re-review after the R0 0C/2I/4M fold. Verified every R0
finding's fold against real source @ branch `mnemonic-addresses-subcommand` (version 0.37.11).

## Critical — None.
## Important — None.

### R0 fold verification
- **I1 RESOLVED** — §3.2 lifts BOTH `render_address_from_xpub` + `network_from_xpub` into new
  `src/address_render.rs` as `pub(crate)`. Confirmed: `network_from_xpub` private bare `fn` at
  `convert.rs:1616` + verbatim mirror at `address_of_xpub.rs:359`. §2 cite fixed to `:1616`. Re-point
  sites complete (render: convert :1291/:1343 callers + :1593 def, address_search.rs:35 dup + :87
  caller, addresses.rs; network: convert :1342 caller, address_of_xpub.rs:359 dup, addresses.rs).
- **I2 RESOLVED** — §3.1 states parsing reuses `parse_from_input`/`NodeType` (pub, split-only,
  confirmed :137-139 zero resolution) and `addresses::run` re-implements resolution over
  `resolve_env_var_sentinel` / `read_stdin_to_string` (pub(crate) :706) / `seedqr::decode`. Confirmed
  `resolve_env_sentinels`/`needs_env_sentinel_resolution` (:1689/:1668) private + ConvertArgs-typed.
  §5 cell #12 added (@env: / stdin-`-` + single-stdin guard / seedqr= parity).
- **M1** line cites fixed (ScriptType :357, network_from_xpub :1616). **M2** kind-guard-diverges note
  added (convert.rs:1342 silently honors mismatch; seed path self-checks derive_slot.rs:91). **M3**
  both READMEs + crate-README-stale ("v0.36.x") note; 20→21 count (20 Command arms confirmed). **M4**
  exact inequality pinned (`--count N` valid iff N ≤ 2^31; 2147483648 succeeds / 2147483649 rejected;
  `--range` B < 2^31) + §5 boundary cell.

### Minor (non-blocking)
- §2 "dispatch at :150" — the match opens at main.rs:147; :150 is the Convert arm inside it.
  Accurate enough (the add-an-Addresses-arm intent is unambiguous). Implementer awareness only.

### Drift sweep — clean
§3.2 two-fn dedup consistent with §2/§5; §3.1 `--from` rewrite consistent with §6 + the seed flow
(ScriptType→CliTemplate→derive_bip32_from_entropy); no new contradiction; every cite re-grepped;
SemVer/lockstep coherent; core spine + R0 confirmed-correct items undisturbed.

**VERDICT: GREEN (0C/0I)**
