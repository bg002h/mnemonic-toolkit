# R0 Review — BIP-388 policy-name round-trip — ROUND 2 (GREEN)

**Source SHA:** `cea1da5`. Re-review after folding all round-1 findings.

**Verdict: 🟢 GREEN — 0 Critical / 0 Important.** Implementation may proceed.

## Fold confirmations
- **I1** — template-path carve-out correct: `bip388.rs:33-44` (`format_bip388_wallet_policy`) hardcodes `"name": template.human_name()` (`:133`); fix targets only `:47` passthrough; FOLLOWUP `bip388-template-path-wallet-name` is the right disposition.
- **I2** — one-step RED cell correct: the two-step path drops the name at the intermediate concrete step (no policy metadata); T1's direct `--descriptor <named-policy> --format bip388` is the only canonical RED cell. Two-step test stays a no-regression cell (stale comment updated).
- **I3** — general lift confirmed; T5 accurate: `specter.rs:51` `label` IS the Specter wallet-name output key; `collect_missing` (`specter.rs:34`) checks `!wallet_name_is_non_default` → a named policy now flips it true → exit 0 with `"label":"test-vault"` (pre-fix exit 2 `MissingField::WalletName`).
- **m1** — None-on-malformed contract documented; real parse error surfaces in `expand_bip388_policy`.
- **m2** — `bip388_policy_name` local declared `None` unconditionally before the `is_bip388_policy_shape` block → the `||` safe on the --template path.
- **m4** — `DEFAULT_BIP388_POLICY_NAME` (new const, grep-confirmed zero current matches) defined in `pipeline.rs`, re-exported from `mod.rs:33`, consumed by `build_descriptor.rs:402`. The `--from-import-json` path (`:347`) short-circuits before the var is declared — no scope conflict.

## Observation (no action)
The existing `cli_export_wallet.rs:885` `descriptor_to_bip388_wallet_policy_round_trip` doesn't assert `json["name"]`; post-fix it stays green (unnamed → "imported-descriptor" default) as an implicit T2 no-regression cell.

All folds accurate + complete; no introduced drift. Implementation may proceed.
