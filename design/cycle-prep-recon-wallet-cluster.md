# cycle-prep recon — 2026-05-22 — wallet cluster (BSMS / BIP-129 / wallet-import)

**Origin/master SHA at recon time:** `9b94a7d`
**Local branch:** `master`
**Sync state:** up-to-date (HEAD == origin/master)
**Untracked:** `.claude/`

Slug(s) verified: `wallet-import-bsms-round-1`, `wallet-import-bsms-encrypted`, `bsms-bip129-full-cutover`, `bsms-taproot-emit`, `wallet-import-signet-regtest-disambiguation`, `wallet-import-format-mismatch-matrix-completion-discovered-gaps`, `bsms-extract-threshold-defense-in-depth-direct-unit-test`, `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification`. **Heavy drift expected — the Cycle-7 (v0.31.0) BIP-129 encryption envelope + Cycles 15–17 (v0.32.x) BSMS work post-date most of these v0.26/v0.27-era slugs.**

---

## Per-slug verification

### `wallet-import-bsms-encrypted` — **STALE / SHIPPED → CLOSE**
- **WHAT:** decrypt BSMS encrypted-envelope per BIP-129 §Encryption (AES-CTR over Round-2 payload, token-derived key), route plaintext through Round-2 parser; needs a CLI key flag (slug speculated `--bsms-key`).
- **Citations:**
  - `wallet_import/bsms.rs` "current parser handles unencrypted Round-2 only" — **STRUCTURALLY-WRONG.** Encrypted Round-2 decrypt **shipped v0.31.0**.
  - Live `import-wallet --help`: `--bsms-encryption-token <FILE|->` — *"v0.31.0 — BIP-129 encryption-envelope Round-2 decrypt. … PBKDF2-SHA512 key derivation + AES-256-CTR decrypt + HMAC-SHA256 verify per BIP-129 §Encryption. Combine with --format bsms to decrypt encrypted Round-2 wallet shares."* This **is** the slug's deliverable (flag named `--bsms-encryption-token`, not the speculated `--bsms-key`). v0.32.2 made it repeatable (per-Signer).
- **Action:** **Close** the FOLLOWUP as resolved-at-v0.31.0 (encryption envelope) + v0.32.1/v0.32.2 (encrypted Round-1 + per-Signer tokens). No work.

### `bsms-bip129-full-cutover` — **REVISE: only sub-item (d) remains**
- **WHAT:** complete BIP-129 conformance — (a) deprecate 6-line lenient parser, (b) 4-line Round-2 parser, (c) encryption envelope, (d) final removal of the 6-line lenient parser.
- **Citations:**
  - `wallet_import/bsms.rs:105-127` "6-line lenient parser" (SHA `176443e`) — **DRIFTED.** Line-count match arms now: `2 =>` L108, `4 =>` L116, **`6 =>` L146**, `other =>` L188. The 6-line arm still parses + fires the DEPRECATION NOTICE (L159-169).
  - Sub-item (c) "encryption-envelope STANDARD/EXTENDED (PBKDF2-SHA512 c=2048 → AES-256-CTR + HMAC-SHA256)" — **STALE/SHIPPED v0.31.0** (`--bsms-encryption-token`; same as the slug above).
  - (a)/(b) already marked shipped v0.28.0 (`1444c51`) — confirmed (4-line is the canonical ingest; 6-line deprecated).
- **Action:** Rewrite the slug down to **(d) only**: remove the deprecated 6-line lenient parser arm (`bsms.rs:146`) + `ImportProvenance::BsmsSixLine`. This is a behavior removal (the path still works-with-warning today) → SemVer **MINOR** (pre-1.0 breaking-ish). Also collapse the **duplicate stub** entry (the `v0.28+` `bsms-bip129-full-cutover` points at the canonical `v0.27-cycle-close` one). Cite SHA `9b94a7d`.

### `wallet-import-bsms-round-1` — **PARTIALLY STALE + scope question**
- **WHAT:** ingest BSMS Round-1 *share* files and **collate N shares into a Round-2-equivalent bundle** (coordinator-side descriptor assembly); proposed `--shares` repeating flag + threshold-consistency invariants.
- **Citations:**
  - `wallet_import/bsms.rs` "current parser handles Round-2 only" — **STRUCTURALLY-WRONG.** `--bsms-round1 <FILE>` shipped v0.27.0 (repeating; BIP-322 ECDSA Round-1 record **verify**; `--bsms-verify-strict`; encrypted-record variant v0.32.1).
  - BUT what shipped is Round-1 *verification*, NOT *coordinator assembly* (the slug's "collate N shares → assembled descriptor"). The `--bsms-round1` help is explicit: *"for BIP-322 ECDSA signature verification … each record is verified independently."* No descriptor synthesis from shares.
- **Action:** **Re-scope or close.** The verify subset shipped; the remaining piece (toolkit acts as BSMS **coordinator**, assembling the multisig descriptor from N Round-1 shares) is arguably **out of scope** (the toolkit imports/verifies, it is not a coordinator). Recommend **close as superseded** (verify shipped) unless the user wants coordinator-assembly as a deliberate new feature. **Needs user direction.**

### `bsms-taproot-emit` — **GENUINELY OPEN, UPSTREAM-BLOCKED (not actionable)**
- **WHAT:** BSMS Round-2 emit for `tr()` descriptors; blocked on a BIP-129 update adding BIP-386 to §1 prerequisites.
- **Citations:**
  - `wallet_export/bsms.rs:69-76` taproot refusal in `emit()` (SHA `1abd9d1`) — **DRIFTED.** `fn emit` now L64; refusal comment L70-76; the `P2tr | P2trMulti` match is L79; FOLLOWUP cite at L76 + doc-comment L24-26. Refusal is intact + per-script-type discriminated (v0.28.0 P8A/P8B).
  - "BIP-129 §1 does NOT include BIP-386" — still true (no evidence of an upstream BIP-129 update). Soft-blocker stands.
- **Action:** Keep open; **not actionable** without an upstream BIP-129 canonicalization. Refresh the line cite to `wallet_export/bsms.rs:64-79`, SHA `9b94a7d`. No cycle.

### `wallet-import-signet-regtest-disambiguation` — **GENUINELY OPEN, actionable (needs user direction)**
- **WHAT:** coin-type-1 collapses signet/regtest→testnet; optionally add a `--network signet|regtest` override (post-parse re-binding) on `import-wallet`.
- **Citations:**
  - `wallet_import/bsms.rs:14-15` module doc comment citing the FOLLOWUP — **DRIFTED-by-~10.** The doc comment is at **L24-26**: *"Signet/regtest are not distinguishable from testnet … as testnet (FOLLOWUP `wallet-import-signet-regtest-disambiguation`)."*
  - `SPEC_wallet_import_v0_26_0.md §4.2 step 8` — testnet-collapse normative text (not re-read; behavior confirmed live).
- **Action:** Genuinely open. Small feature: a `--network signet|regtest` override on `import-wallet`. SemVer **PATCH** if additive flag (mandatory GUI `schema_mirror` + manual lockstep on the new flag NAME). **Needs user direction** (the slug itself says so — 99% of users never set it). Cite SHA `9b94a7d`.

### `bsms-extract-threshold-defense-in-depth-direct-unit-test` — **GENUINELY OPEN, trivial test-hygiene**
- **WHAT:** add a `#[cfg(test)]` unit test directly invoking `extract_threshold` on a `sortedmulti_a(`/`multi_a(` body, asserting `Err(BsmsTaprootImportRefused)`.
- **Citations:**
  - `wallet_import/bsms.rs:~493` guard — **DRIFTED-by-3.** Guard is at **L496-497**: `if descriptor_body.contains("sortedmulti_a(") || descriptor_body.contains("multi_a(") { return Err(ToolkitError::BsmsTaprootImportRefused); }`.
  - `bsms.rs:215` parse-entry refusal (`tr(` substring) — **ACCURATE** (L215-216: `if descriptor_body_no_csum.contains("tr(") { return Err(BsmsTaprootImportRefused) }`).
  - Existing test mod (L530+) has `extract_threshold_u8_overflow_is_typed_error` (L537) — does NOT cover the taproot guard. The direct unit test is **still missing**.
- **Action:** Genuinely open + **trivial**. Add one unit test to `bsms.rs::tests` calling `extract_threshold("tr(NUMS,sortedmulti_a(2,@0,@1))")` (or bare `sortedmulti_a(...)`) → `Err(BsmsTaprootImportRefused)`. Test-only PATCH. Cite SHA `9b94a7d`.

### `wallet-import-format-mismatch-matrix-completion-discovered-gaps` — **INDETERMINATE (arm count grew; needs per-pair audit)**
- **WHAT:** extend Coldcard/Sparrow/Specter/Electrum dispatch arms to refuse all wrong-format sniffs symmetrically (~10 additional `ImportWalletFormatMismatch` arms; full 8×7=56-cell off-diagonal matrix).
- **Citations:**
  - `cmd/import_wallet.rs` dispatch arms — there are now **9+ `ImportWalletFormatMismatch` return sites** (L481,487,493,499,505,511,517,533,539…; symbol appears 51× incl. tests) vs the parent's original 3 narrow arms. **Cannot confirm the full 56-cell matrix from arm-count alone** — needs a per-(override,sniff)-pair audit to know which of the 10 discovered gaps remain.
- **Action:** Lower-priority test-hygiene. Before any work, do a **targeted matrix audit** (enumerate the 8×7 off-diagonal and check each refuses). Likely partially-addressed since filing. Cite SHA `9b94a7d`.

### `schema-mirror-flag-name-vs-wire-shape-conceptual-clarification` — **GENUINELY OPEN (docs), low priority**
- **WHAT:** document that the GUI `schema_mirror` gate enforces **clap flag-NAME parity only**, NOT runtime `--json` wire-shape; runtime `--json` consumers have no automated drift gate. Recommended: (c) document in CLAUDE.md (v0.29.x); (b) per-consumer GUI tests at v0.30+ for high-traffic subcommands.
- **Citations:**
  - `mnemonic-gui/tests/schema_mirror.rs:91-121` (`assert_schema_matches_help`, flag-NAME set comparison) — **ACCURATE** (verified by direct read this session).
  - CLAUDE.md — grep for `flag-name|wire-shape|JSON wire|only gates` → **EMPTY**. The explicit clarification (option c) was **NOT** added to CLAUDE.md; only the CHANGELOG note + this FOLLOWUP exist. Option (b) per-consumer tests not done.
- **Action:** Genuinely open (docs). Cheap: add the flag-NAME-vs-wire-shape clarification to CLAUDE.md's "GUI schema-mirror coverage" section (option c). Option (b) is a larger cross-repo test pass — defer. Cite SHA `9b94a7d`.

---

## Cross-cutting observations

1. **Two slugs are dead-on-arrival as written** (`wallet-import-bsms-encrypted` fully shipped; `bsms-bip129-full-cutover` (c) shipped) — both resolved by the **v0.31.0** `--bsms-encryption-token` envelope. Their "Where" citations ("parser handles unencrypted Round-2 only") are now factually false. This is the same stale-status pattern the v0.34.2 cycle-prep flushed (FOLLOWUP bodies are snapshots; the BSMS arc closed across Cycles 7/15/16/17 without retiring these v0.26/v0.27-era parents).
2. **`wallet-import-bsms-round-1` conflates two things:** Round-1 record *verify* (shipped) vs coordinator *assembly* (not done, likely OOS). The slug reads as "open" but the actionable/intended subset shipped.
3. **`bsms-bip129-full-cutover` has a duplicate stub** (`v0.28+` entry → "DUPLICATE STUB → see canonical entry above"). Collapse on edit.
4. **Pervasive line-number drift** (every BSMS citation moved 3–40 lines since filing) — expected; substance mostly intact except the two shipped-feature claims.
5. **Only ONE slug citation was perfectly ACCURATE** (`schema_mirror.rs:91-121`) — the rest drifted or are structurally wrong.
6. No new cross-pin/version staleness surfaced beyond the known GUI-`pinned-upstream` mk pin (v0.4.0 vs install.sh v0.4.1), already noted last cycle.

---

## Recommended brainstorm-session scope

**Cycle A — "wallet-cluster FOLLOWUP hygiene" (docs/test-only, PATCH, ~1 small cycle, NO new CLI surface):**
- **Close** `wallet-import-bsms-encrypted` (shipped v0.31.0).
- **Close** `wallet-import-bsms-round-1` as superseded (verify shipped) — OR re-scope to coordinator-assembly pending user call.
- **Rewrite** `bsms-bip129-full-cutover` → (d)-only; collapse the duplicate stub.
- **Refresh** line cites on `bsms-taproot-emit` (upstream-blocked; keep open) + `wallet-import-signet-regtest-disambiguation`.
- **Ship** the two trivial closes: slug 7 (`extract_threshold` direct unit test, test-only) + slug 8 (CLAUDE.md flag-NAME-vs-wire-shape clarification, docs-only).
- This is a clean, low-risk PATCH (`v0.34.3`) — toolkit-only, no GUI/manual lockstep (no flag-NAME change). Mandatory opus R0 still applies (plan-doc).

**Deferred to their own decisions (NOT in Cycle A):**
- `wallet-import-signet-regtest-disambiguation` — small `--network` feature; **needs user direction** (would be a flag-NAME add → GUI+manual lockstep, PATCH).
- `wallet-import-format-mismatch-matrix-completion-discovered-gaps` — do a per-pair matrix audit first; low priority.
- `bsms-taproot-emit` — externally blocked (BIP-129 §1 + BIP-386); revisit on upstream movement.
- `wallet-import-bsms-round-1` coordinator-assembly — only if user wants the toolkit to assume the coordinator role.
