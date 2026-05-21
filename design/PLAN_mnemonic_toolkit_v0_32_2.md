# mnemonic-toolkit-v0.32.2 Implementation Plan (Cycle 16 — bsms-encryption-per-signer-tokens)

> **For agentic workers:** REQUIRED SUB-SKILL: `superpowers:subagent-driven-development` or direct execution.

**Goal:** Ship `mnemonic-toolkit-v0.32.2` (SemVer-PATCH; additive — single-token usage unchanged). Closes `bsms-encryption-per-signer-tokens` FOLLOWUP. Make `--bsms-encryption-token` repeatable (`Option<PathBuf>` → `Vec<PathBuf>` via `ArgAction::Append`) so a Coordinator can supply one TOKEN per Signer (BIP-129 line 74: "one common TOKEN for all Signers, or one per Signer"), paired positionally with `--bsms-round1` encrypted Round-1 records.

**Architecture:** The Cycle-15 `BsmsToken` read-once refactor is the prerequisite. `--bsms-encryption-token` becomes `Vec<PathBuf>`; each entry is read+width-validated into a `Vec<BsmsToken>`. Token-to-record pairing:
- **0 tokens** → no decrypt path (encrypted record still → existing no-token `BadInput`).
- **1 token (SHARED, backward-compatible — current v0.31.0/v0.32.1 behavior)** → decrypts ALL encrypted `--bsms-round1` records AND the Round-2 `--blob`.
- **N>1 tokens (PER-SIGNER positional)** → requires: (a) at least one `--bsms-round1` record present (R0 I1 gap-h fold — N>1 tokens with ZERO records → `BadInput("per-Signer tokens (N>1 --bsms-encryption-token) require N matching --bsms-round1 records; none supplied")`, guarded EARLY since `verify_bsms_round1_files` is skipped when records is empty); (b) every `--bsms-round1` record is encrypted (mixing plaintext + per-Signer tokens is refused — index alignment ambiguity); (c) `N == --bsms-round1` record count; (d) NO encrypted Round-2 `--blob` in the same invocation (a single Round-2 share carries a single token; multi-token + encrypted blob → refuse as ambiguous). Then `token[i]` decrypts `record[i]`.
- **Error precedence (R0 I2 fold):** `verify_bsms_round1_files` (L277) runs BEFORE the Round-2 block (L860). So in N>1 + encrypted-blob, the Round-1 positional verify (incl. any per-record MAC failure) fires FIRST; the multi-token-Round-2 refusal is reached only after all Round-1 records verify. The gap-h guard (N>1 + 0 records) runs earliest of all (before either path).
- **stdin**: at most one token entry may be `-`; the existing `--blob=- AND token=-` dual-stdin refusal is retained + generalized (any token `-` vs blob `-`).

**Tech Stack:** Rust; reuses Cycle-15 helpers (`BsmsToken`, `read_and_validate_bsms_token`, `decrypt_bsms_record`, `is_encrypted_bsms_record`); zero new deps; zero new clap flag NAMES; zero new `ToolkitError` variants (reuses `BadInput`, `BsmsMacMismatch`).

**SemVer rationale (v0.32.1 → v0.32.2 PATCH):** purely additive — `Option<PathBuf>` → `Vec<PathBuf>` with `Append` means a single `--bsms-encryption-token X` parses unchanged; supplying it twice (previously a clap error) now succeeds. No flag-name change, no removal, no behavior change for existing single-token invocations. Per the v0.32.1 (#1) precedent (PATCH; no new flag). The FOLLOWUP body's "MINOR CLI break" framing is loose — there is no break (strictly more permissive).

**GUI lockstep — OPTIONAL, not gate-forced:** `schema_mirror.rs:52-53` compares flag NAMES only (`sub.flags.iter().map(|f| f.name)`); the `--bsms-encryption-token` name is unchanged, so flipping the GUI's `repeating: false → true` (mnemonic-gui `src/schema/mnemonic.rs:1736`) does NOT trip the gate. File a follow-on `gui-bsms-encryption-token-repeating-mirror` (GUI v0.17.1; lets the GUI's SlotEditor-style flag-repeat UI add multiple token rows). Non-blocking.

**P0 STRICT-GATE recon (verified at master HEAD `c25b272`):**
- `cmd/import_wallet.rs:204-205` — `--bsms-encryption-token: Option<PathBuf>`.
- `cmd/import_wallet.rs:258-264` — dual-stdin guard (`args.blob` + `args.bsms_encryption_token`).
- `cmd/import_wallet.rs:266-272` — hoisted single-token read → `Option<BsmsToken>`.
- `cmd/import_wallet.rs:277-283` — `verify_bsms_round1_files(&args.bsms_round1, strict, bsms_token.as_ref(), stderr)`.
- `cmd/import_wallet.rs:861-879` — Round-2 block consumes `bsms_token`.
- `cmd/import_wallet.rs:1908-1930` — `read_and_validate_bsms_token`.
- `cmd/import_wallet.rs::verify_bsms_round1_files` — currently `token: Option<&BsmsToken>`; change to `tokens: &[BsmsToken]` + positional logic.
- GUI `src/schema/mnemonic.rs:1733-1745` — `--bsms-encryption-token` FlagSchema (`repeating: false`).
- `schema_mirror.rs:52-53` — flag-NAME-only parity (confirms GUI lockstep optional).

## File structure

### Source files modified (toolkit)
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`:
  - L204-205: `pub bsms_encryption_token: Vec<PathBuf>` with `#[arg(long = "bsms-encryption-token", value_name = "FILE|-")]` — NO explicit `action` (R0 M1 fold: mirror the sibling `--bsms-round1: Vec<PathBuf>` at L191-192, which relies on clap-derive's auto-inferred Append for `Vec` fields).
  - **Gap-h guard (R0 I1)**: add an early check — if `args.bsms_encryption_token.len() > 1` AND `args.bsms_round1.is_empty()` → `BadInput` (per-Signer tokens require matching records). Place before the token read + `verify_bsms_round1_files`.
  - **Dual-stdin guard (L258-264)**: generalize to "at most one stdin consumer". Refuse if `args.blob == Some("-")` AND any token entry is `"-"`; refuse if MORE THAN ONE token entry is `"-"`.
  - **Hoisted token read (L266-272)**: read ALL token entries → `Vec<BsmsToken>` (each via `read_and_validate_bsms_token`). NOTE: `read_and_validate_bsms_token` reads stdin for `-`; with a Vec, only one entry may be `-` (enforced by the guard above) — read that one from stdin, the rest from files.
  - **`verify_bsms_round1_files` signature**: `tokens: &[BsmsToken]`. Upfront, compute `encrypted_count` over the records. Pairing rules:
    - If `tokens.len() <= 1`: shared (use `tokens.first()` for every encrypted record; no-token error if empty + an encrypted record present).
    - If `tokens.len() > 1`: require ALL records encrypted AND `tokens.len() == records.len()` (else `BadInput` with a precise message); `token[i]` decrypts `record[i]`.
  - **Round-2 block (L861)**: if `tokens.len() > 1` AND the blob is encrypted (i.e. this block is entered) → `BadInput("Round-2 --blob decrypt requires exactly one --bsms-encryption-token; got N (per-Signer tokens pair with --bsms-round1 records only)")`. Else use `tokens[0]` (the single shared token).
  - Plumb `bsms_token: Option<BsmsToken>` → `bsms_tokens: Vec<BsmsToken>` through the call sites.

### Test files modified (toolkit)
- `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms_encrypted.rs`:
  - `per_signer_two_tokens_two_records_positional` — 2 encrypted Round-1 records + 2 tokens (positional) → both verify. (Build a 2nd encrypted record via test-time re-encryption with a 2nd token, mirroring the Cycle-15 helper.)
  - `per_signer_token_count_mismatch_refused` — 2 records + 3 tokens (or 2 records + 1... no, 1 is shared) → N>1 ≠ record-count → `BadInput`.
  - `per_signer_mixed_plaintext_encrypted_refused` — N>1 tokens but one record plaintext → refuse.
  - `per_signer_multi_token_with_encrypted_blob_refused` — N>1 tokens + encrypted `--blob` → refuse (ambiguous).
  - `single_token_shared_still_decrypts_all` — 1 token + 2 encrypted records → both decrypt (backward-compat).
  - `single_token_backward_compat_round2_blob` — existing 1-token Round-2 path unchanged (regression).
  - `two_token_stdin_refused` — two `--bsms-encryption-token -` → refuse (single stdin).
  - `multi_token_zero_records_refused` (R0 I1/M2) — N>1 tokens + NO `--bsms-round1` records → `BadInput` (gap-h guard).
  - `per_signer_token_i_mac_mismatch_cites_index` (R0 M2) — 2 records + 2 tokens where token[1] is wrong for record[1] → `BsmsMacMismatch` / error citing record index 1.
- All Cycle-15 + earlier encrypted-suite cells must still pass (single-token paths unchanged).

### Documentation (toolkit)
- `docs/manual/src/40-cli-reference/41-mnemonic.md`: `--bsms-encryption-token` documents repeatability + the 1-token-shared vs N-token-positional rules + the multi-token-Round-2 refusal.

### Cross-repo (optional follow-on, NOT this cycle)
- File `gui-bsms-encryption-token-repeating-mirror` FOLLOWUP (GUI v0.17.1: flip `repeating: true`).

### Release tooling
- `Cargo.toml:3` — `0.32.1` → `0.32.2`.
- `CHANGELOG.md` — `## [0.32.2]`.
- `scripts/install.sh:32` — pin → `v0.32.2`.
- `design/FOLLOWUPS.md` — close `bsms-encryption-per-signer-tokens` + file `gui-bsms-encryption-token-repeating-mirror`.

## Tasks

### Task 1: Phase 2 — orchestrator: Vec tokens + positional pairing
- [ ] Flag → Vec + Append; generalize dual-stdin guard; hoist multi-token read → `Vec<BsmsToken>`.
- [ ] `verify_bsms_round1_files(tokens: &[BsmsToken], ...)` + pairing rules (shared vs positional + all-encrypted + count checks).
- [ ] Round-2 block: multi-token + encrypted-blob refusal; single-token path unchanged.
- [ ] Build.
- [ ] Commit Phase 2.

### Task 2: Phase 3 — Integration tests
- [ ] 7 cells per the plan (positional, count-mismatch, mixed-refusal, multi-token-blob-refusal, single-token-shared, backward-compat, two-stdin-refusal).
- [ ] Build + run; confirm all prior encrypted-suite cells green.
- [ ] Commit Phase 3.

### Task 3: Phase 4 — Manual
- [ ] Document repeatable token + pairing rules.
- [ ] Manual lint.
- [ ] Commit Phase 4.

### Task 4: Phase 5 — Cycle close
- [ ] Version bump + install.sh + CHANGELOG.
- [ ] Pre-tag audit (test + clippy + manual lint).
- [ ] Opus end-of-cycle review.
- [ ] Commit + tag mnemonic-toolkit-v0.32.2 + push + GH Release.
- [ ] install-pin-check CI green.
- [ ] Close FOLLOWUP + file GUI follow-on + memory.

## Cross-phase invariants
- Opus R0 review on plan-doc BEFORE Phase 2 (stress-test the pairing semantics + edge cases).
- Opus end-of-cycle review BEFORE tag (security-adjacent: per-token MAC verify).
- No `cargo fmt --all`.
- GUI lockstep OPTIONAL (follow-on FOLLOWUP).
- install-pin-check CI gate.

## Risk register
- **Pairing-semantics complexity** — the 0/1/N-token rules + all-encrypted + count + no-Round-2-blob constraints are the design core. R0 must stress-test: what if N>1 tokens + 0 records? (refuse: per-Signer tokens require records). What if 1 token + plaintext-only records? (token unused; fine). What if N tokens + records where some decrypt-MAC-fail? (per-record `BsmsMacMismatch` at index i).
- **stdin with Vec tokens** — only ONE token entry may be `-`. The read loop must read the `-` entry from stdin exactly once; refuse 2+ `-` entries. Combined with `--blob=-`, refuse.
- **Backward-compat** — the SINGLE-token paths (Round-1 shared + Round-2 blob) must be byte-identical to v0.32.1. Locked by regression cells.
- **Positional index vs encrypted-record skipping** — by requiring ALL records encrypted in N>1 mode, `token[i] ↔ record[i]` is unambiguous (no plaintext-skipping). Mixed mode is refused.
- **Per-record decrypt error attribution** — a MAC mismatch on record i must cite index i (the `decrypt_bsms_record` ctx already includes "encrypted record {i}").

## Self-review (pre-R0 dispatch)
- ✓ P0 recon + Cycle-15 prerequisite confirmed.
- ✓ Pairing semantics fully specified incl. edge cases.
- ✓ SemVer PATCH (additive; single-token unchanged) + GUI lockstep classified OPTIONAL.
- ✓ stdin-with-Vec single-`-` rule specified.
- ✓ Backward-compat regression cells planned.
- ✓ Test surface: 7 integration cells.
