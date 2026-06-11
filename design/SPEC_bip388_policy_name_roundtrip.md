# SPEC — preserve the BIP-388 wallet-policy `name` across a `--format bip388` round-trip

**Cycle:** toolkit PATCH · **Source SHA:** `cea1da5` (v0.53.7) · **Recon:** `cycle-prep-recon-silentpayment-phrase-lang+bip388-name-roundtrip.md`.
**Resolves:** `bip388-policy-name-lossy-roundtrip`.

## Problem (verified @ `cea1da5`)
`export-wallet --descriptor <bip388-policy-json> --format bip388` loses the policy `name`:
- **Expand** (`cmd/export_wallet.rs:419-422`, `run`): a leading-`{` `--descriptor` is detected as a BIP-388 policy and expanded via `expand_bip388_policy(desc)` → concrete descriptor. The policy `name` is DROPPED (`expand_bip388_policy` (`wallet_import/pipeline.rs:187`) returns only the descriptor `String`; its `BipPolicyJson._name` field (`:162`, `#[serde(rename="name")]`) is deserialized-but-unread).
- **Emit** (`wallet_export/pipeline.rs:207`, `descriptor_to_bip388_wallet_policy`): HARDCODES `"name": "imported-descriptor"`.

So any named policy round-trips to `"imported-descriptor"`. (BIP-388 `name` is a human label, not funds-relevant, but a backup/restore tool should preserve it.)

## Design — lift the policy name into the existing `wallet_name` channel (elegant: the default already IS "imported-descriptor")
The descriptor-path default `wallet_name_resolved` is ALREADY `"imported-descriptor"` (`export_wallet.rs:566`) — the exact hardcoded emit value. So routing the policy name through `EmitInputs.wallet_name` + making the emit read it preserves current behavior for unnamed inputs by construction, and fixes named ones. This mirrors the `resolved_wallet_name` import-json lift precedent (`:746-781`).

1. **New sibling extractor** `wallet_import/pipeline.rs`: `pub(crate) fn bip388_policy_name(json: &str) -> Option<String>` — parse `BipPolicyJson`, return `Some(name)` if non-empty, else `None`; **malformed JSON → `None` (R0-r1 m1): the real parse error surfaces in the immediately-following `expand_bip388_policy` call — document this None-on-malformed contract on the fn so a caller doesn't error-check its result.** Minimal blast: `expand_bip388_policy`'s signature + its 3 callers (`export_wallet:420`, `descriptor_intake:195`, `bundle:314`) stay UNCHANGED. (The policy JSON is parsed twice — here + in `expand` — acceptable at CLI-time for small JSON.)
2. **Capture at expand** (`export_wallet.rs:419-422`): **declare `let mut bip388_policy_name: Option<String> = None;` UNCONDITIONALLY before the `if is_bip388_policy_shape(desc)` block (R0-r1 m2)**, then set it inside that arm via `pipeline::bip388_policy_name(desc)`. (Unconditional-None init keeps the `||` in step 4 safe for the --template path.)
3. **Thread into `wallet_name_resolved`** (`:562-568`): the `None => match template_opt { … None => "imported-descriptor" }` leaf becomes `None => bip388_policy_name.clone().unwrap_or_else(|| "imported-descriptor".to_string())`. Precedence: `--wallet-name` flag > bip388 policy name > `"imported-descriptor"`. (bip388 ⇒ `--descriptor` ⇒ `template_opt` is None, so the policy name lands in the None→None leaf — no conflict with the template-name branch.)
4. **`wallet_name_is_non_default`** (`:586`): becomes `args.wallet_name.is_some() || bip388_policy_name.is_some()` — a lifted policy name counts as non-default (mirrors the import-json lift at `:781`), so the Specter emitter (which rejects the silent `"imported-descriptor"` default) accepts a named policy.
5. **Emit reads the name** (`wallet_export/pipeline.rs:166,207`): `descriptor_to_bip388_wallet_policy(descriptor: &str, name: &str)` — `:207` emits `"name": name`. Define `pub(crate) const DEFAULT_BIP388_POLICY_NAME: &str = "imported-descriptor"` in `wallet_export/pipeline.rs` **and RE-EXPORT it from `wallet_export/mod.rs` (R0-r1 m4)** alongside the `descriptor_to_bip388_wallet_policy` re-export (`mod.rs:33`) so `build_descriptor.rs` imports it cleanly. Callers: `wallet_export/bip388.rs:47` passes `inputs.wallet_name`; `cmd/build_descriptor.rs:402` passes `DEFAULT_BIP388_POLICY_NAME` (build-descriptor has no wallet-name context — preserves its current output byte-for-byte).

No CLI flag/help/subcommand change → **no `schema_mirror` / manual / GUI / sibling lockstep.** The `--format bip388` output `name` field already exists (no wire-schema add/remove). SemVer **PATCH** (metadata-fidelity fix; unnamed inputs + build-descriptor unchanged).

## Tests (TDD)
Prefer pipeline-level unit tests (the fns are `pub(crate)`); add an integration cell only if a pure-unit assertion can't reach the round-trip.
- **T1 (DIRECT one-step round-trip preserves the name — the fix, RED-proven; R0-r1 I2):** the canonical RED cell is the CLI ONE-STEP path: `export-wallet --descriptor <policy_2of2 (name="test-vault")> --format bip388` → parse stdout JSON → `v["name"] == "test-vault"`. **RED pre-fix:** `"imported-descriptor"`. (Do NOT use the two-step `--format descriptor`→`--format bip388` path — its intermediate concrete step legitimately drops the policy context, so it never exercises the fix.) A pipeline-unit cell may ALSO assert `descriptor_to_bip388_wallet_policy(expand_bip388_policy(policy)?, &bip388_policy_name(policy).unwrap())` emits `"name":"test-vault"`.
- **T2 (unnamed → default, no-regression):** `descriptor_to_bip388_wallet_policy(desc, DEFAULT_BIP388_POLICY_NAME)` emits `"name":"imported-descriptor"`; `bip388_policy_name(<malformed / empty-name JSON>)` → `None` → the default flows through.
- **T3 (`--wallet-name` overrides the policy name — integration, in `run`):** `--descriptor <policy_2of2> --wallet-name "Override" --format bip388` → `v["name"]=="Override"` (precedence flag > policy-name).
- **T4 (extractor unit):** `bip388_policy_name` returns `Some("test-vault")` for the named policy, `None` for malformed JSON / empty name.
- **T5 (Specter unblock — R0-r1 I3, a deliberate behavior change):** `--descriptor <policy_2of2 (name="test-vault")> --format specter` now EXITS 0 and emits `"label":"test-vault"` — pre-fix it exited 2 (`MissingField::WalletName`, since the default name is non-default-rejected by Specter). Pins the general-lift consequence.
- **Existing-test update (R0-r1 I2):** `tests/cli_bip388_policy_intake.rs` `export_wallet_descriptor_bip388_policy_roundtrips` (two-step) — correct the stale "modulo the dropped name" comment: the name is now preserved on the ONE-STEP path but still lost in this TWO-STEP path because the intermediate `--format descriptor` step discards the policy context. Keep it as a no-regression cell. Watch for any other `--format {sparrow,specter,coldcard}` test of a NAMED policy that pinned the old "imported-descriptor"/refusal behavior; update to the corrected behavior.

## Ritual
CHANGELOG `[0.53.8]`; version bump (Cargo.toml + Cargo.lock); self-pins (`README.md:13` + `crates/mnemonic-toolkit/README.md:9` + `scripts/install.sh:32`); FOLLOWUPS resolve `bip388-policy-name-lossy-roundtrip` + FILE `bip388-template-path-wallet-name` (the carved-out template-path gap). No manual/schema_mirror/GUI/sibling lockstep. Mandatory R0 gate to 0C/0I; persist reviews to `design/agent-reports/`.

## Resolved: GENERAL lift (R0-r1 I3)
The name lift is GENERAL (into `wallet_name`, flowing to any `--format`) — CONFIRMED correct, consistent with the `resolved_wallet_name` import-json precedent (a named bip388 policy is as authoritative a name source as an import-json envelope). Consequence pinned by T5: a named-policy `--format specter` export, previously refused (default name), now succeeds (lifted name → non-default).

## Scope carve-out (R0-r1 I1) — the TEMPLATE path is NOT touched
`wallet_export/bip388.rs::emit` has two branches: the descriptor passthrough (`:47` `descriptor_to_bip388_wallet_policy` — the fix target) and the TEMPLATE path (`:33-44` `format_bip388_wallet_policy`, which hardcodes `"name": template.human_name()` at `bip388.rs:133`). This cycle changes ONLY the descriptor passthrough. The template path's name (and its independent ignoring of `--wallet-name`) is a PRE-EXISTING, separate gap — **do NOT touch it.** File FOLLOWUP `bip388-template-path-wallet-name` (`--format bip388` on the `--template` path ignores `--wallet-name`; emits `template.human_name()`).

## Non-goals
Changing build-descriptor's default policy name; renaming the `"imported-descriptor"` default; any wire-schema change; the TEMPLATE-path name (carved out above); the silent-payment cycle (shipped v0.53.7).
