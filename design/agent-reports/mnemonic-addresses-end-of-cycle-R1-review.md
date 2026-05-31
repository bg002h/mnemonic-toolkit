# End-of-cycle R1 re-review — v0.38.0 `mnemonic addresses`

Reviewer: feature-dev:code-reviewer (opus). Re-review after the R0 0C/2I/2M fold. Verified all four
folds against current source + drift sweep + release hygiene.

## Fold verification — all correct
- **I1 (Zeroizing entropy):** `addresses.rs` — seed-branch `entropy` now
  `zeroize::Zeroizing<Vec<u8>>` via `Zeroizing::new(match …)`, mirroring convert.rs:1147. `&entropy`
  auto-derefs to `&[u8]` for `derive_bip32_from_entropy` (compiles, behavior unchanged). Only
  load-bearing master-secret intermediate; `from_value` plain == convert parity.
- **I2 (manual lockstep):** `41-mnemonic.md` has `## mnemonic addresses` documenting all 11 flags +
  `--help` (flag-coverage lint passes); `cli-subcommands.list` has `mnemonic addresses`; worked-example
  addresses are oracle-verified. Paired GUI schema-mirror landed (mnemonic-gui e0db08d; schema_mirror +
  secret-drift pass vs the v0.38.0 binary — 21/21).
- **M1:** cli_gui_schema doc-comment + assert vector (26, addresses first) + message all say 26.
- **M2:** `--passphrase` argv advisory gated `from.node != Xpub`; still fires for inline seed sources,
  suppressed only for xpub (which rejects passphrase). No over-suppression.

## Drift sweep — clean
argv-coverage evidence anchors intact; release hygiene (Cargo.toml/lock 0.38.0, CHANGELOG, README
markers); main.rs registers/dispatches Addresses.

## Critical / Important — None.

## Minor
- **M3 (R0-overlooked) — README subcommand NARRATIVE undercount in BOTH READMEs.** `README.md:44`
  "Twenty subcommands" (separate from the already-fixed Status line) + `addresses` absent from the
  grouped bullets; crate `README.md:32` inline list likewise. Non-gating (the readme_version gate only
  checks the version marker, not the narrative; not a CLAUDE.md hard mirror invariant). **[Folded
  post-R1: both → "Twenty-one" + `addresses` added to the Convert/derive group / inline list.]**

**VERDICT: GREEN (0C/0I)** — four R0 folds correct, no drift; the one residual (M3 README narrative)
is non-gating and now folded. The 0C/0I tag gate is satisfied.
