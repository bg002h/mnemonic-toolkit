# v0.34.3 Wallet-Cluster Hygiene — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Retire the stale BSMS FOLLOWUPs the wallet-cluster cycle-prep recon surfaced, refresh decayed citations, and ship two trivial closes — leaving `design/FOLLOWUPS.md` honest. No behavior or CLI-surface change.

**Architecture:** A docs/test-only PATCH. The "implementation" is (1) FOLLOWUP-registry text edits, (2) one `#[cfg(test)]` unit test, (3) one CLAUDE.md paragraph, (4) version/release artifacts. Single branch `v0.34.3-wallet-cluster-hygiene`. No new code paths; no GUI/manual lockstep (no clap flag-NAME change).

**Tech Stack:** Rust (`mnemonic-toolkit` binary crate); `design/FOLLOWUPS.md` markdown; `CLAUDE.md`; `CHANGELOG.md`; `scripts/install.sh`; `Cargo.toml`/`Cargo.lock`.

**Source SHA for citations:** `9b94a7d` (origin/master at recon time, 2026-05-22).

**SemVer:** PATCH (`v0.34.2 → v0.34.3`). No behavior change, no CLI surface change, no GUI/manual lockstep.

**Approved design:** The scope below was presented and user-approved 2026-05-22 ("pure hygiene only"; `wallet-import-bsms-round-1` disposition = close-as-superseded per opus architect verdict A). This plan-doc embeds the approved spec.

---

## Recon basis (what's stale and why)

The wallet-cluster recon (`cycle-prep-recon-wallet-cluster.md`, SHA `9b94a7d`) found the BIP-129 encryption envelope shipped v0.31.0 (`--bsms-encryption-token`) + Cycles 15–17 (v0.32.x) BSMS work, post-dating these v0.26/v0.27-era slugs. Confirmed live:
- `import-wallet --bsms-encryption-token <FILE|->` exists (v0.31.0): PBKDF2-SHA512 + AES-256-CTR + HMAC-SHA256 per BIP-129 §Encryption.
- `import-wallet --bsms-round1 <FILE>` exists (v0.27.0): repeating, BIP-322 Round-1 record **verify** (not assembly).
- `wallet_import/bsms.rs`: line-count match arms `2=>`L108, `4=>`L116, **`6=>`L146**, `other=>`L188; the `extract_threshold` taproot guard at **L496-497**; parse-entry `tr(` refusal at **L215-216**.
- `wallet_export/bsms.rs`: `fn emit` L64, taproot refusal match `P2tr|P2trMulti` at **L79** (comment L70-76).
- `bsms.rs` signet doc comment at **L24-26**.
- Siblings already resolved: `bsms-bip129-encryption-envelope` (Cycle 7/v0.31.0), `bsms-verify-signatures` (v0.27.0), `wallet-export-bsms-emitter` (v0.27.0).

---

## Task 1: FOLLOWUP-registry hygiene (`design/FOLLOWUPS.md`)

**Files:** Modify `design/FOLLOWUPS.md`.

All edits are exact-`old_string` replacements resolved against current source at edit time. New text below.

- [ ] **Step 1a — Close `wallet-import-bsms-encrypted`.** Replace its `- **Status:** open` line with:

```
- **Status:** resolved — shipped v0.31.0 (`import-wallet --bsms-encryption-token <FILE|->`: BIP-129 §Encryption envelope = PBKDF2-SHA512 + AES-256-CTR + HMAC-SHA256 verify-before-decrypt). The CLI flag the body speculated as `--bsms-key` shipped as `--bsms-encryption-token`; encrypted Round-1 records landed v0.32.1, per-Signer tokens v0.32.2. The "current parser handles unencrypted Round-2 only" framing above is superseded. Resolved alongside sibling `bsms-bip129-encryption-envelope` (Cycle 7). Closed 2026-05-22 via wallet-cluster cycle-prep recon (SHA `9b94a7d`).
```

- [ ] **Step 1b — Close `wallet-import-bsms-round-1`.** Replace its `- **Status:** open` line with:

```
- **Status:** resolved — superseded by v0.27.0 `import-wallet --bsms-round1 <FILE>` (repeating; BIP-129 Round-1 record BIP-322 verify; `--bsms-verify-strict`). The in-scope subset (Round-1 record ingest + verify) shipped; the body's remaining intent — coordinator-side *assembly* of a multisig descriptor from N Round-1 shares (the proposed `--shares` collation) — is OUT OF SCOPE for an import/verify/backup tool (same category as the deliberately-excluded signing/PSBT; opus architect disposition 2026-05-22: DISPOSITION A). Users coordinate in Sparrow/Specter/Coldcard, then `import-wallet` the resulting Round-2 blob (supported, plaintext or encrypted). If a concrete user wants coordinator mode, file a fresh, deliberately-scoped slug with its own brainstorm/R0. Cross-ref `bsms-verify-signatures` (v0.27.0 Round-1 SIG closure) + sibling `bsms-encryption-round1-decrypt-then-verify`. Closed 2026-05-22 via wallet-cluster cycle-prep recon (SHA `9b94a7d`).
```

- [ ] **Step 1c — Rewrite `bsms-bip129-full-cutover` → (d)-only.** Three edits within the entry:
  - (i) In the `**Where:**` block, replace `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:105-127` with `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:146` and append ` (cite refreshed against SHA \`9b94a7d\` 2026-05-22; the \`6 =>\` arm of the line-count match)`.
  - (ii) Replace the sub-item (c) paragraph (`(c) **Add encryption-envelope … carved out into the dedicated sibling FOLLOWUP \`bsms-bip129-encryption-envelope\` (v0.28+) for tracking.`) with:

```
  - (c) **Add encryption-envelope (STANDARD/EXTENDED) support.** *Shipped v0.31.0 (Cycle 7)*: `import-wallet --bsms-encryption-token <FILE|->` — PBKDF2-SHA512 + AES-256-CTR decrypt + HMAC-SHA256 verify per BIP-129 §Encryption (repeatable per-Signer at v0.32.2). Resolved as the dedicated sibling `bsms-bip129-encryption-envelope`.
```

  - (iii) Replace the `- **Status:** open (sub-items (c) + (d) remain; (a)/(b)/(e) shipped at v0.28.0).` line with:

```
- **Status:** open — ONLY sub-item (d) remains: final removal of the deprecated 6-line lenient parser arm (`wallet_import/bsms.rs:146`) + `ImportProvenance::BsmsSixLine`. (a)/(b)/(e) shipped v0.28.0; (c) shipped v0.31.0 (sibling `bsms-bip129-encryption-envelope`). (d) is a behavior change (the 6-line path still parses-with-deprecation-notice today) → future SemVer **MINOR**, not bundled into v0.34.3 hygiene. Sub-item scope corrected 2026-05-22 via wallet-cluster cycle-prep recon (SHA `9b94a7d`).
```

- [ ] **Step 1d — Collapse the duplicate stub.** Delete the entire `### \`bsms-bip129-full-cutover\` — DUPLICATE STUB → see canonical entry above (line ~2207)` entry (at `FOLLOWUPS.md:2480`) and its body, leaving the single canonical entry as the source of truth.

- [ ] **Step 1e — Refresh decayed cites (keep both open).**
  - `bsms-taproot-emit` `**Where:**`: replace `crates/mnemonic-toolkit/src/wallet_export/bsms.rs:69-76` with `crates/mnemonic-toolkit/src/wallet_export/bsms.rs:64-79` and update the trailing `Citation verified against origin/master SHA \`1abd9d1\` 2026-05-19.` → `Citation refreshed against origin/master SHA \`9b94a7d\` 2026-05-22.` (Substance unchanged: still upstream-blocked on BIP-129 §1 + BIP-386.)
  - `wallet-import-signet-regtest-disambiguation` `**Where:**`: replace `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:14-15` with `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:24-26`. (Still open; the `--network signet|regtest` override remains a deferred feature awaiting user direction.)

- [ ] **Step 1f — Narrow `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` to its (b) residual.** Replace its `- **Status:** \`open\`` line with:

```
- **Status:** open — option (c) [document the gap in CLAUDE.md] **shipped v0.34.3** (CLAUDE.md "GUI schema-mirror coverage" section now states the gate enforces clap flag-NAME parity only, NOT runtime `--json` wire-shape). Residual = option (b): per-consumer `--json` wire-shape regression tests on the GUI side for high-traffic subcommands (`xpub-search`/`import-wallet`/`export-wallet`), v0.30+. Narrowed 2026-05-22 (SHA `9b94a7d`).
```

- [ ] **Step 1g — Commit.**

```bash
git add design/FOLLOWUPS.md
git commit -m "docs(followups): wallet-cluster hygiene — close 2 stale BSMS slugs, rewrite cutover→(d), refresh cites"
```

---

## Task 2: `extract_threshold` taproot-guard direct unit test (closes `bsms-extract-threshold-defense-in-depth-direct-unit-test`)

**Files:** Modify `crates/mnemonic-toolkit/src/wallet_import/bsms.rs` (the `#[cfg(test)] mod tests`, near `extract_threshold_u8_overflow_is_typed_error`).

- [ ] **Step 1 — Write the test.** Add to the test module, mirroring the existing `extract_threshold_u8_overflow_is_typed_error` error-matching idiom (use the same `ToolkitError` path that test uses — `super::*` is already in scope):

```rust
#[test]
fn extract_threshold_refuses_taproot_multi_a_directly() {
    // v0.34.3: direct unit coverage for the v0.28.7 defense-in-depth guard
    // at extract_threshold (bsms.rs:496). The integration path can't reach
    // it — parse-entry refuses the `tr(` substring first (bsms.rs:215) — so
    // this asserts the guard directly on the multi_a / sortedmulti_a bodies.
    // Closes FOLLOWUP `bsms-extract-threshold-defense-in-depth-direct-unit-test`.
    assert!(matches!(
        extract_threshold("tr(NUMS,sortedmulti_a(2,@0,@1))"),
        Err(ToolkitError::BsmsTaprootImportRefused)
    ));
    assert!(matches!(
        extract_threshold("tr(NUMS,multi_a(2,@0,@1))"),
        Err(ToolkitError::BsmsTaprootImportRefused)
    ));
}
```

- [ ] **Step 2 — Run it (it passes; the guard already exists at L496-497).**

Run: `cargo test -p mnemonic-toolkit --lib extract_threshold_refuses_taproot_multi_a_directly`
Expected: PASS (1 passed).

*Note: if `ToolkitError` is not directly in scope in the test module, fully-qualify as `crate::error::ToolkitError` to match whatever the sibling overflow test uses — confirm at write time.*

- [ ] **Step 3 — Close the slug in `design/FOLLOWUPS.md`.** Replace `bsms-extract-threshold-defense-in-depth-direct-unit-test`'s `- **Status:** \`open\`` with:

```
- **Status:** resolved — v0.34.3. Added `extract_threshold_refuses_taproot_multi_a_directly` to `wallet_import/bsms.rs::tests` directly asserting `extract_threshold("tr(NUMS,{sortedmulti_a,multi_a}(...))") == Err(BsmsTaprootImportRefused)` (guard at `bsms.rs:496-497`; parse-entry refusal at `:215`). Cite drift fixed (was `~493`). Closed via wallet-cluster cycle-prep recon (SHA `9b94a7d`).
```

- [ ] **Step 4 — Commit.**

```bash
git add crates/mnemonic-toolkit/src/wallet_import/bsms.rs design/FOLLOWUPS.md
git commit -m "test(bsms): direct unit test for extract_threshold taproot guard (closes followup)"
```

---

## Task 3: CLAUDE.md flag-NAME-vs-wire-shape clarification (satisfies slug 8 option (c))

**Files:** Modify `CLAUDE.md` ("GUI schema-mirror coverage" section, after the description paragraph at line 28).

- [ ] **Step 1 — Insert the clarification paragraph** immediately after the existing line 28 paragraph (before the line-32 "lagging indicator" paragraph):

```
**Scope of the gate — clap flag-NAME parity, NOT JSON wire-shape.** `schema_mirror` enforces that the hand-maintained `SubcommandSchema`'s clap **flag-name set** (plus dropdown value enums) matches `gui-schema`'s output. It does NOT gate the runtime **`--json` wire-shape** of any subcommand (`xpub-search`, `import-wallet`, `export-wallet`, `bundle`, …). GUI consumers of those `--json` payloads have no automated drift gate — they must self-update when a wire-shape changes, coordinated manually via the paired-PR rule. (Extending the gate to per-subcommand `--json` output-shape declarations is FOLLOWUP `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` option (b), v0.30+.)
```

- [ ] **Step 2 — Commit.** (Slug 8 narrowing already done in Task 1f.)

```bash
git add CLAUDE.md
git commit -m "docs(claude): clarify schema_mirror gates clap flag-NAME parity, not --json wire-shape"
```

---

## Task 4: Release artifacts + ship

**Files:** `crates/mnemonic-toolkit/Cargo.toml`, `Cargo.lock`, `scripts/install.sh`, `CHANGELOG.md`, move `cycle-prep-recon-wallet-cluster.md` → `design/`.

- [ ] **Step 1 — Move the recon artifact into `design/`.**

```bash
git mv cycle-prep-recon-wallet-cluster.md design/cycle-prep-recon-wallet-cluster.md 2>/dev/null || (mkdir -p design && mv cycle-prep-recon-wallet-cluster.md design/ && git add design/cycle-prep-recon-wallet-cluster.md)
```

- [ ] **Step 2 — Version bump.** `crates/mnemonic-toolkit/Cargo.toml` `version = "0.34.2"` → `"0.34.3"`. Then regenerate the lock (the `cargo-lock-version-bump-lockstep` lesson):

Run: `cargo build -p mnemonic-toolkit` → confirm `Cargo.lock` `mnemonic-toolkit` entry = `0.34.3`.

- [ ] **Step 3 — install.sh self-pin.** `scripts/install.sh:32` `mnemonic-toolkit-v0.34.2` → `mnemonic-toolkit-v0.34.3`.

- [ ] **Step 4 — CHANGELOG.** Add above `[0.34.2]`:

```
## mnemonic-toolkit [0.34.3] — 2026-05-22

**SemVer-PATCH — wallet-cluster FOLLOWUP hygiene.** No behavior or CLI-surface change. Retires stale BSMS/BIP-129 FOLLOWUPs surfaced by a cycle-prep recon: closes `wallet-import-bsms-encrypted` (the BIP-129 §Encryption envelope shipped v0.31.0 as `--bsms-encryption-token`) and `wallet-import-bsms-round-1` (Round-1 *verify* shipped v0.27.0 as `--bsms-round1`; coordinator descriptor-assembly is out-of-scope per opus architect disposition); rewrites `bsms-bip129-full-cutover` down to its sole remaining sub-item (d) (6-line lenient-parser removal, a future MINOR) + collapses a duplicate stub; refreshes decayed line-citations (`bsms-taproot-emit`, `wallet-import-signet-regtest-disambiguation`). Ships two trivial closes: a direct unit test for the `extract_threshold` taproot defense-in-depth guard (`bsms-extract-threshold-defense-in-depth-direct-unit-test`), and a CLAUDE.md clarification that `schema_mirror` gates clap flag-NAME parity only, not runtime `--json` wire-shape (`schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` option (c); residual (b) deferred to v0.30+). Also corrects the lock-regen discipline from `cargo-lock-version-bump-lockstep`.
```

- [ ] **Step 5 — Full regression + clippy + manual lint.**

Run: `cargo test -p mnemonic-toolkit` → all green.
Run: `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` → clean.
Run: `make -C docs/manual lint MNEMONIC_BIN=$PWD/target/debug/mnemonic MD_BIN=md MS_BIN=ms MK_BIN=mk` → 6/6 OK (no manual surface change expected, but confirm no regression).

- [ ] **Step 6 — Commit release artifacts.**

```bash
git add crates/mnemonic-toolkit/Cargo.toml Cargo.lock scripts/install.sh CHANGELOG.md design/cycle-prep-recon-wallet-cluster.md design/IMPLEMENTATION_PLAN_v0_34_3_wallet_cluster_hygiene.md
git commit -m "release(toolkit): mnemonic-toolkit v0.34.3 — wallet-cluster FOLLOWUP hygiene"
```

- [ ] **Step 7 — End-of-cycle opus review → GREEN (0C/0I).** Persist verbatim to `design/agent-reports/v0_34_3-end-of-cycle-review.md`. Fold any Critical/Important before tag (mandatory gate).

- [ ] **Step 8 — Ship (after user go-ahead, per the outward-facing pattern).** Merge → master (ff), push, tag `mnemonic-toolkit-v0.34.3` (install-pin-check passes — self-pin already bumped), GH release. **No GUI/manual lockstep** (no flag-NAME change).

---

## Self-review (writing-plans)

- **Spec coverage:** all 8 recon slugs accounted for — closed (encrypted, round-1, extract-threshold-test), rewritten (cutover→d + dedup), cite-refreshed (taproot-emit, signet), narrowed (schema-mirror-wire-shape), + the slug-8 doc (c). ✓
- **No placeholders:** every FOLLOWUP edit, the test code, and the CLAUDE.md paragraph are written out verbatim. ✓
- **Type consistency:** test uses `ToolkitError::BsmsTaprootImportRefused` (confirmed variant at `bsms.rs:497`) + `extract_threshold` (confirmed `pub(super) fn` at `bsms.rs:489`). ✓
- **SemVer / lockstep:** PATCH; no flag-NAME change → no GUI `schema_mirror` / manual lockstep. ✓
- **Risk:** near-zero — no production code path changes; the one test asserts already-shipped behavior; the lock-regen avoids the v0.34.1-class stale-lock defect.
