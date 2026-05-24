# v0.36.1 Implementation Plan — `silent-payment` `--passphrase` + `--change-address`

> **For agentic workers:** per-phase TDD; tests before impl; per-phase opus reviewer-loop until 0C/0I; persist reviews to `design/agent-reports/` before the fold. NO parallel code-gen. Steps use `- [ ]`.

**Goal:** Two additive flags on `mnemonic silent-payment`: `--passphrase`/`--passphrase-stdin` (derive the SP address for a BIP-39-passphrase-protected wallet) and `--change-address` (emit the BIP-352 m=0 change address, footgun-guarded).

**Architecture:** Both are additive surface on the existing `cmd/silent_payment.rs` (`SilentPaymentArgs` + `run` + `resolve_master_xpriv`). The crypto already exists: `derive_master_seed(mnemonic, passphrase)` takes a passphrase (`derive_slot.rs:32`); `labeled_spend_key(secp, &b_scan, b_spend_pub, m)` handles any m incl. 0 (`silent_payment.rs:45`). Mirror the `convert`/`derive-child` `--passphrase`/`--passphrase-stdin` pattern.

**Tech Stack:** Rust; clap-derive; `bip39`, `bitcoin::bip32`, `secp256k1`; `zeroize`.

**Source SHA:** citations grep-verified vs `origin/master` @ `6100d85` (2026-05-23).

---

## SemVer + lockstep
- **PATCH → v0.36.1** (additive flags on an existing subcommand). Three net-new flag NAMES: `--passphrase`, `--passphrase-stdin`, `--change-address`.
- **GUI lockstep (MANDATORY — flag-NAME change):** add the 3 flags to `mnemonic-gui/src/schema/mnemonic.rs::SILENT_PAYMENT_FLAGS` (`--passphrase`/`--passphrase-stdin` `secret:true` — already covered by `flag_is_secret`; `--change-address` bool, non-secret) + toolkit pin bump → paired GUI release.
- **Manual lockstep:** extend `docs/manual/src/40-cli-reference/41-mnemonic.md` `## mnemonic silent-payment` flag table. `cli-subcommands.list` already lists `mnemonic silent-payment` (no new line). No `gui-schema` subcommand-count change (same subcommand). No sibling-codec companions.
- **NO `secrets.rs`/taxonomy change:** `flag_is_secret` already matches `--passphrase`/`--passphrase-stdin` (`secrets.rs:52-53`).

## Design decisions (R0 open-questions resolved; architect to confirm)
1. **`--passphrase` + xprv/tprv input → warn-and-ignore** (mirror `convert`'s "ignored on this edge" advisory). An xprv IS the master; BIP-39 passphrase applies only to phrase/ms1/entropy inputs.
2. **Flag shapes:** `--passphrase: Option<String>` (secret), `--passphrase-stdin: bool` with `conflicts_with = "passphrase"` (derive-child precedent `:97-98`); `--change-address: bool`.
3. **Dual-stdin guard:** refuse `--passphrase-stdin` together with `--secret-stdin` (single stdin per invocation — derive_child.rs:128-131 precedent).
4. **`--change-address` is ADDITIVE:** it appends the m=0 change address to the normal output (base + labels + keys); it does NOT replace the base address. Human: a clearly-tagged line; JSON: a `change_address` string field (present only when `--change-address`). Footgun guard: the human line is unmistakable ("m=0 CHANGE — internal use; never hand out").
5. **`--label 0` stays refused** (`:138`); `--change-address` is the deliberate, guarded route to m=0.
6. **Empty-passphrase default preserves v0.35.0 output** (no behavior change when the flag is absent).

---

## Phase 1 — `--passphrase` / `--passphrase-stdin`

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/silent_payment.rs` (`SilentPaymentArgs` + `run` + `resolve_master_xpriv` signature)
- Test: `crates/mnemonic-toolkit/tests/cli_silent_payment.rs`

**Context:** `resolve_master_xpriv(secret, network)` (`:81`) calls `derive_master_seed(&mnemonic, "")` at `:86` (ms1/entropy via `to_master` closure) and `:112` (BIP-39 phrase). Both must thread a passphrase. `derive_master_seed(mnemonic, passphrase)` @ `derive_slot.rs:32`. The xprv branch (`:92`) is passphrase-independent.

- [ ] **Step 1 — RED test:** a passphrase changes the derived address (vs the no-passphrase base).

```rust
#[test]
fn passphrase_changes_derived_address() {
    let no_pass = sp_base(&["silent-payment", "--secret", PHRASE]);
    let with_pass = sp_base(&["silent-payment", "--secret", PHRASE, "--passphrase", "TREZOR"]);
    assert_ne!(no_pass, with_pass, "passphrase must change the SP address");
    assert!(with_pass.starts_with("sp1q"));
}
// sp_base() runs the cmd and extracts the `address:` line value.
```

- [ ] **Step 2** — run; FAIL (`--passphrase` unknown flag).
- [ ] **Step 3 — args:** add to `SilentPaymentArgs`:
```rust
/// BIP-39 mnemonic-extension passphrase ("25th word"). Applies to phrase/
/// ms1/entropy inputs; ignored (with a warning) for an xprv input. SECRET —
/// leaks via argv; prefer --passphrase-stdin.
#[arg(long)]
pub passphrase: Option<String>,
/// Read the BIP-39 passphrase from stdin. Mutually exclusive with --passphrase.
#[arg(long = "passphrase-stdin", conflicts_with = "passphrase")]
pub passphrase_stdin: bool,
```
- [ ] **Step 4 — resolve + thread.** In `run` (after the secret is resolved): guard `if args.passphrase_stdin && args.secret_stdin { return Err(ToolkitError::SilentPayment("--passphrase-stdin cannot be combined with --secret-stdin (single stdin per invocation)".into())); }`. Resolve `passphrase: Zeroizing<String>` from `--passphrase` (argv-leak warning via `secret_in_argv_warning(stderr, "--passphrase", "--passphrase-stdin")`) / `--passphrase-stdin` (read stdin) / else `""`. mlock-pin it. Change `resolve_master_xpriv(secret, network)` → `resolve_master_xpriv(secret, &passphrase, network)`; pass `&passphrase` into both `derive_master_seed` calls (`:86`,`:112`). On the xprv branch, if `!passphrase.is_empty()` emit `writeln!(stderr, "warning: --passphrase ignored — an xprv/tprv input is already the master key")`.
- [ ] **Step 5** — run Step-1 test → PASS. Add: `--passphrase-stdin` works (stdin); `--passphrase` + `--secret-stdin` allowed (passphrase inline); `--passphrase-stdin` + `--secret-stdin` refused; xprv + `--passphrase` warns + ignores (address == no-passphrase xprv address).
- [ ] **Step 6** — full suite + clippy; commit.

**Test vector note:** the BIP-39 spec's canonical `TREZOR` passphrase over the `abandon…about` phrase is the standard cross-impl vector; assert the SP address differs and is well-formed (the SP-encoding crypto is already byte-exact-validated in the lib unit tests, so a difference + `sp1q` prefix suffices).

---

## Phase 2 — `--change-address` (BIP-352 m=0)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/silent_payment.rs` (`SilentPaymentArgs` + `SilentPaymentJson` + `run` output)
- Test: `crates/mnemonic-toolkit/tests/cli_silent_payment.rs`

**Context:** `labeled_spend_key(secp, &b_scan, b_spend_pub, 0)` (`silent_payment.rs:45`) computes the m=0 key; `encode_sp_address(hrp, &b_scan_pub, &b_m)` encodes it. The `--label 0` refusal (`:138`) is unchanged.

- [ ] **Step 1 — RED test:** `--change-address` emits a distinct m=0 address tagged as change, alongside the base.

```rust
#[test]
fn change_address_emits_tagged_m0_distinct_from_base() {
    let out = run_str(&["silent-payment", "--secret", PHRASE, "--change-address"]);
    assert!(out.contains(BASE_SP), "base address still emitted");
    // change address present, tagged, and != base
    assert!(out.to_lowercase().contains("change"));
    let change = extract_field(&out, "change");
    assert!(change.starts_with("sp1q") && change != BASE_SP);
}

#[test]
fn change_address_json_field() {
    let v = run_json(&["silent-payment", "--secret", PHRASE, "--change-address", "--json"]);
    assert!(v["change_address"].as_str().unwrap().starts_with("sp1q"));
    assert_ne!(v["change_address"], v["address"]);
}

#[test]
fn change_address_absent_by_default() {
    let v = run_json(&["silent-payment", "--secret", PHRASE, "--json"]);
    assert!(v.get("change_address").is_none());
}
```

- [ ] **Step 2** — run; FAIL (`--change-address` unknown).
- [ ] **Step 3 — arg:**
```rust
/// Also emit the BIP-352 m=0 CHANGE address (B_scan ‖ B_m=0). For the wallet's
/// OWN change detection only — never hand it out as a receiving address.
#[arg(long = "change-address")]
pub change_address: bool,
```
- [ ] **Step 4 — JSON field:** add to `SilentPaymentJson` (`:63`): `#[serde(skip_serializing_if = "Option::is_none")] change_address: Option<String>,`.
- [ ] **Step 5 — emit.** In `run`, after computing `labeled`: if `args.change_address`, compute `let change = encode_sp_address(hrp, &b_scan_pub, &labeled_spend_key(&secp, &b_scan, b_spend_pub, 0)?);`. JSON: set `change_address: Some(change)`. Human: after the labeled lines, `writeln!(stdout, "  change_addr:  {change}   (BIP-352 m=0 CHANGE — internal change detection ONLY; never hand out as a receiving address)")`.
- [ ] **Step 6** — run tests → PASS; full suite + clippy; commit.

---

## Phase 3 — GUI lockstep (mnemonic-gui PATCH/MINOR)

**Files (mnemonic-gui):** `src/schema/mnemonic.rs::SILENT_PAYMENT_FLAGS`, `Cargo.toml` (version + pin), `pinned-upstream.toml`, `Cargo.lock`, `CHANGELOG.md`.

- [ ] **Step 1** — add 3 `FlagSchema` entries to `SILENT_PAYMENT_FLAGS` (alphabetical insert): `--change-address` (Boolean, non-secret); `--passphrase` (Text, **secret:true**); `--passphrase-stdin` (Boolean, **secret:true**). Re-dump `mnemonic gui-schema` for silent-payment and match the flag-NAME set + `secret` bits exactly.
- [ ] **Step 2** — pin `mnemonic-toolkit-v0.36.0 → v0.36.1` (Cargo.toml git-dep + pinned-upstream.toml); GUI version bump (`0.21.0 → 0.21.1` PATCH, or `0.22.0` — match repo convention for "flags added to existing subcommand"; check prior precedent e.g. v0.19.2 import-wallet `--network`); `cargo update -p mnemonic-toolkit`; bump `pinned_version` display string.
- [ ] **Step 3** — `MNEMONIC_BIN=<v0.36.1 binary> cargo test`: `mnemonic_schema_flag_names_match_help_text` + `secret_drift_gate_*` (the 2 new secret flags MUST register) + conditional-drift GREEN; full suite + clippy.
- [ ] **Step 4** — CHANGELOG entry; commit.

---

## Phase 4 — manual + release + end-of-cycle

- [ ] **Step 1 — manual:** add the 3 flags to the `## mnemonic silent-payment` flag table in `41-mnemonic.md`; document the m=0 change-address footgun + passphrase-applies-to-seed-inputs-only. Run `make -C docs/manual lint MNEMONIC_BIN=<v0.36.1>` → flag-coverage GREEN.
- [ ] **Step 2 — toolkit release-prep:** `Cargo.toml` 0.36.1; `Cargo.lock` regen; `scripts/install.sh:32` self-pin `mnemonic-toolkit-v0.36.1`; `CHANGELOG.md` [0.36.1] PATCH entry.
- [ ] **Step 3 — FOLLOWUPS:** close `silent-payment-passphrase` + `silent-payment-change-address-m0`.
- [ ] **Step 4 — end-of-cycle opus review** → persist `design/agent-reports/v0_36_1-end-of-cycle-review.md` → fold → GREEN.
- [ ] **Step 5 — ship toolkit:** merge→master (ff), tag `mnemonic-toolkit-v0.36.1`, push, GH release; verify rust + manual + install-pin-check CI.
- [ ] **Step 6 — ship GUI:** tag `mnemonic-gui-v0.21.x`, push, GH release; verify schema-mirror + build CI.

---

## Self-review
- **Spec coverage:** passphrase (P1), change-address (P2), GUI (P3), manual+release (P4). ✓
- **Type consistency:** `resolve_master_xpriv(secret, &passphrase, network)` signature change is the one cross-cutting edit — both call sites in `run` + the fn def. `SilentPaymentJson.change_address: Option<String>`. ✓
- **Open R0 items:** (a) GUI SemVer for added-flags-to-existing-subcommand (PATCH vs MINOR — check precedent); (b) `--change-address` JSON footgun (bare `change_address` field could be misused by a JSON consumer — should it carry a sibling note/flag, or is the field name + docs enough?); (c) `--passphrase` + xprv warn-vs-error (chose warn per convert precedent); (d) dual-stdin guard wording; (e) whether `--passphrase` should also accept `@env:` sentinels like convert (`convert.rs:117`) — likely out of scope for v1.
