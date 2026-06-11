# R0 Review — BIP-388 policy-name round-trip — ROUND 1

**Source SHA:** `cea1da5`. **Verdict: 🟡 YELLOW — 0 Critical / 3 Important / 4 Minor.** All SPEC line citations verified accurate @ cea1da5 (m3). Findings are SPEC + test-plan fixes; the core design (lift into `wallet_name`) is sound.

## Critical
None.

## Important

**I1 — the TEMPLATE-path emit is untouched and now diverges.** `bip388.rs:32-48` has two branches: template (`format_bip388_wallet_policy` → hardcodes `"name": template.human_name()` at `:133`) and descriptor passthrough (`descriptor_to_bip388_wallet_policy` at `:47`, the fix target). The fix only touches the passthrough; the template path keeps ignoring `inputs.wallet_name` (so `--wallet-name` is silently dropped on `--format bip388` + `--template`). PRE-EXISTING, out of scope. **Fix:** explicitly scope-carve the template path in the SPEC + file FOLLOWUP `bip388-template-path-wallet-name` so the implementer doesn't touch it / isn't surprised.

**I2 — the canonical RED cell must be the DIRECT one-step path.** The existing `tests/cli_bip388_policy_intake.rs:35` `export_wallet_descriptor_bip388_policy_roundtrips` is a TWO-STEP path (policy → `--format descriptor` → concrete → `--format bip388`); the intermediate concrete step legitimately discards the policy context, so the name is `None` there even post-fix — it never exercises the fixed path and its "modulo the dropped name" comment is now stale. **Fix:** T1 must use the DIRECT `--descriptor <policy_2of2 (name="test-vault")> --format bip388` → assert `v["name"]=="test-vault"` (RED pre-fix = "imported-descriptor"). Update the two-step test's stale comment (name preserved one-step; still lost two-step because the intermediate step drops policy context).

**I3 — general lift CONFIRMED correct, but the Specter unblock needs a positive test.** General (into `wallet_name`, flowing to any `--format`) is right — consistent with the `resolved_wallet_name` import-json precedent (a named policy is as authoritative a name source). Consequence: `--descriptor <named-policy> --format specter` previously exited 2 (`MissingField::WalletName`, default name); post-fix it exits 0 (lifted name → `wallet_name_is_non_default=true`). A deliberate behavior change with NO existing coverage. **Fix:** add T5 — `--descriptor <policy_2of2> --format specter` succeeds + emits `"label":"test-vault"`.

## Minor
- **m1** — `bip388_policy_name(json)` returns `None` on malformed JSON (the real parse error surfaces in the immediately-following `expand_bip388_policy` call); document this contract on the fn.
- **m2** — initialize the `bip388_policy_name` local to `None` UNCONDITIONALLY before the `if is_bip388_policy_shape` block (so the `||` in `wallet_name_is_non_default` is safe for the --template path).
- **m3** — all citations verified live @ cea1da5 (no action).
- **m4** — re-export `DEFAULT_BIP388_POLICY_NAME` from `wallet_export/mod.rs` (alongside the `descriptor_to_bip388_wallet_policy` re-export at `:33`) so `build_descriptor.rs:402` imports it cleanly.

## Confirmations
- Precedence (`--wallet-name` > policy-name > "imported-descriptor") + the None→None leaf placement correct (bip388 ⇒ --descriptor ⇒ template_opt None).
- Minimal-blast extractor (expand signature + 3 callers unchanged) is the right choice; double-parse acceptable (small JSON, CLI-time).
- PATCH; `--format bip388` wire-shape unchanged (name field already exists); no schema_mirror/manual/GUI/sibling lockstep; 3 self-pins correct.
