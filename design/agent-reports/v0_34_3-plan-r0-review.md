# v0.34.3 — Plan-doc R0 architect review (opus) — MANDATORY pre-implementation gate

**Date:** 2026-05-22
**Cycle:** mnemonic-toolkit v0.34.3 — wallet-cluster FOLLOWUP hygiene
**Branch:** `v0.34.3-wallet-cluster-hygiene`
**Reviewer:** opus (feature-dev:code-reviewer), R0 (pre-implementation)
**Target:** `design/IMPLEMENTATION_PLAN_v0_34_3_wallet_cluster_hygiene.md` (verified against live source + `cycle-prep-recon-wallet-cluster.md`)

---

## Verification matrix (every plan claim → live source)

| Plan claim | Live source | Verdict |
|---|---|---|
| `6 =>` arm at L146 | `wallet_import/bsms.rs:146` | ✓ |
| `extract_threshold` is `pub(super) fn` at L489 | `bsms.rs:489` | ✓ |
| taproot guard `sortedmulti_a(`/`multi_a(` → `BsmsTaprootImportRefused` at L496-497 | `bsms.rs:496-497` | ✓ |
| parse-entry `tr(` refusal at L215-216 | `bsms.rs:215-216` | ✓ |
| test mod with `extract_threshold_u8_overflow_is_typed_error`, `use super::*;` | `bsms.rs:526-527,536-537` | ✓ |
| `ToolkitError::BsmsTaprootImportRefused` variant exists | `error.rs:51` | ✓ |
| `import-wallet --bsms-encryption-token` (v0.31.0) | `cmd/import_wallet.rs:227-228` | ✓ |
| `import-wallet --bsms-round1` (v0.27.0) | `cmd/import_wallet.rs:208-209` | ✓ |
| `bsms-bip129-encryption-envelope` resolved Cycle 7/v0.31.0 | `FOLLOWUPS.md:2558` | ✓ |
| `bsms-verify-signatures` resolved v0.27.0 | `FOLLOWUPS.md:2204` | ✓ |
| `wallet-export-bsms-emitter` resolved v0.27.0 | `FOLLOWUPS.md:2237` | ✓ |
| `wallet_export/bsms.rs` `fn emit` L64, `P2tr\|P2trMulti` refusal ~L79 | `wallet_export/bsms.rs:64,77-80` | ✓ |
| `bsms-taproot-emit` cite `:69-76`, SHA `1abd9d1` | `FOLLOWUPS.md:2470` | ✓ |
| signet doc comment cite `:14-15` → `:24-26` | `bsms.rs:24-26`; FOLLOWUP cite at `FOLLOWUPS.md:2174` | ✓ |
| duplicate stub heading at L2480, canonical at L2208 | `FOLLOWUPS.md:2480,2208` | ✓ |
| `bsms-bip129-full-cutover` `**Where:**` cite `:105-127` | `FOLLOWUPS.md:2212` | ✓ |
| canonical Status line `open (sub-items (c) + (d) remain...)` | `FOLLOWUPS.md:2223` | ✓ |
| `wallet-import-bsms-encrypted` Status open, body speculated `--bsms-key` | `FOLLOWUPS.md:2386,2384` | ✓ |
| `wallet-import-bsms-round-1` Status open | `FOLLOWUPS.md:2374` | ✓ |
| `bsms-extract-threshold...test` Status open, cite `~493` | `FOLLOWUPS.md:2781,2778` | ✓ |
| `schema-mirror-flag-name...` Status open, option (c)=CLAUDE.md | `FOLLOWUPS.md:2793,2791` | ✓ |
| `bsms-encryption-round1-decrypt-then-verify` exists (Step 1b cross-ref) | `FOLLOWUPS.md:3023,3033` | ✓ |
| CLAUDE.md "GUI schema-mirror coverage" §L26, desc para L28, no wire-shape clarification yet | `CLAUDE.md:26,28,32` | ✓ |
| Cargo.toml version `0.34.2` | `Cargo.toml:3` | ✓ |
| install.sh self-pin `mnemonic-toolkit-v0.34.2` at L32 | `install.sh:32` | ✓ |
| CHANGELOG top entry `[0.34.2]` | `CHANGELOG.md:9` | ✓ |

## Disposition correctness
- **Closing `wallet-import-bsms-encrypted`**: justified by shipped code — `--bsms-encryption-token` (`import_wallet.rs:227`) implements PBKDF2-SHA512 + AES-256-CTR + HMAC-SHA256 verify-before-decrypt; the body's speculated `--bsms-key` shipped under the real name. ✓
- **Closing `wallet-import-bsms-round-1` as superseded**: the verify subset (`--bsms-round1`, repeatable, BIP-322 verify, incl. encrypted Round-1 via v0.32.1) shipped; the body's coordinator-side *assembly* intent is genuinely distinct and out-of-scope per user-approved DISPOSITION A. ✓
- **Cutover → (d)-only**: (a)/(b)/(e) shipped v0.28.0 (`FOLLOWUPS.md:2216-2220`), (c) shipped v0.31.0 (`bsms-bip129-encryption-envelope` resolved). Only (d) (6-line lenient arm removal + `ImportProvenance::BsmsSixLine`) remains, and that arm still actively parses-with-deprecation-notice (`bsms.rs:146-187`) → correctly framed as a future MINOR, not bundled. ✓
- **Duplicate-stub deletion**: the stub (L2480) explicitly defers all tracking to canonical (L2208); deleting it does not orphan the canonical entry. ✓
- **Unit test**: `extract_threshold("tr(NUMS,sortedmulti_a(2,@0,@1))")` hits the L496 substring guard directly (called outside `parse`, so the L215 `tr(` refusal is bypassed) → `Err(BsmsTaprootImportRefused)`. `ToolkitError` resolves in the test module via `use super::*`. Reachable, compiles, asserts exactly the slug's ask. ✓
- **SemVer/lockstep**: no clap flag-NAME added/removed/renamed; PATCH + no GUI/manual lockstep is correct. ✓
- **No false "shipped" claim**: every "resolved/shipped" assertion the plan makes to close a slug is backed by live resolved-status siblings or live code. No still-open slug is wrongly closed.

---

## Critical
(none)

## Important
(none)

## Minor

- **Self-review slug accounting is imprecise — `wallet-import-format-mismatch-matrix-completion-discovered-gaps` is the 8th recon slug and is silently dropped, not "accounted for".** `IMPLEMENTATION_PLAN...:195` claims "all 8 recon slugs accounted for" then enumerates only 7 dispositions + the doc. The 8th (`FOLLOWUPS.md:2763`, recon slug 8, INDETERMINATE) is correctly *deferred* per the recon's "Deferred to their own decisions" section, but the prose implies it received a disposition this cycle. No edit is wrong and no slug is left in a contradictory state — prose-accuracy nit. Fix: reword to "7 slugs dispositioned this cycle; the 8th (`...-discovered-gaps`) deferred per recon."

- **Plan's incidental "comment L70-76" for `bsms-taproot-emit` is slightly narrow.** `IMPLEMENTATION_PLAN...:25` says the refusal comment is "L70-76"; the live comment block runs `wallet_export/bsms.rs:65-76`. Does not affect the actual edit (Step 1e inserts cite `:64-79`, correctly spanning `fn emit` L64 → enum line L79). Cite-fidelity note only.

- **`extract_threshold` "L496-497" vs slug body's "L493" / test comment's "bsms.rs:496".** All live values (guard L496-497, parse-entry L215-216) are correct; the drift-fix is itself accurate. No action.

---

VERDICT: GREEN (0C/0I)

Clean hygiene plan. Every factual claim verified against live source at the branch tip. No slug wrongly closed; the two closures backed by genuinely shipped code; cutover→(d) correctly leaves only the still-live 6-line arm open; duplicate-stub deletion does not orphan the canonical entry. Unit test compiles + reachable + asserts exactly the slug's ask. PATCH + no-lockstep correct. The three Minor items are prose/cite-fidelity nits below the blocking threshold. Implementation may proceed.

---

## Fold disposition (controller)
GREEN (0C/0I) → gate satisfied. Folded Minor #1 (self-review reworded to "7 dispositioned + 1 deferred") and Minor #2 (recon-basis cite `L70-76` → `L65-76`) — both doc-only, zero spec impact, so no R0 re-dispatch (the re-dispatch convention guards against drift from folding Critical/Important findings; there were none). Minor #3 needs no action.
