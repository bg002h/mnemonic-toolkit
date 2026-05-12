# tech-manual-v1.0 Phase 5.5 review — r1

Date: 2026-05-12
Reviewer: feature-dev:code-reviewer (r1)

## Summary

- Troubleshooting (65-troubleshooting.md): 0C / 0I / 0L / 0N
- Bibliography (66-bibliography.md): 2C / 0I / 0L / 0N
- cspell additions (.cspell.json): 0C / 0I / 0L / 0N

Total: 2C / 0I / 0L / 0N

---

## Critical

### C-1 — BIP-86 author attribution fabricated

**File:** `docs/technical-manual/src/60-back-matter/66-bibliography.md:18`

**Evidence.** The entry reads: `**BIP-86.** Pieter Wuille, Greg Maxwell. *Key Derivation for Single Key P2TR Outputs.*`. The live BIP at `github.com/bitcoin/bips/blob/master/bip-0086.mediawiki` confirms the sole author is **Ava Chow**. Wuille is an author on BIP-340/341/342 and BIP-173; Maxwell on BIP-340/341; neither on BIP-86.

**Fix.** Replace `Pieter Wuille, Greg Maxwell` with `Ava Chow`.

### C-2 — `docs.rs/rust-codex32` URL returns HTTP 404

**File:** `docs/technical-manual/src/60-back-matter/66-bibliography.md:43`

**Evidence.** The entry at line 43 reads `**\`rust-codex32\`** v0.1.0 (Andrew Poelstra, CC0). [docs.rs/rust-codex32](https://docs.rs/rust-codex32)`. Fetching that URL returns HTTP 404 — the crates.io package is `codex32` (the GitHub repository name is `rust-codex32`). Line 47 already has a correct `codex32` crate entry pointing at `docs.rs/codex32`.

**Fix.** Merge the two entries (line 43 + line 47) into one — both refer to the same crate.

---

## Passing checks

**Variant-count integrity.** 43 (md) + 22 (mk) + 10 (ms) + 26 (toolkit) = 101 total. All section counts match HEAD enums exactly. PASS.

**Variant-name spot-check (4 per section, 16 total).** All match `error.rs` declarations: `BitStreamTruncated`, `OperatorContextViolation`, `NUMSSentinelConflict`, `ChunkSetIdMismatch`, `MalformedPayloadPadding`, `CrossChunkHashMismatch`, `CardPayloadTooLarge`, `InvalidXpubVersion`, `ReservedTagNotEmittedInV01`, `UnexpectedStringLength`, `PayloadLengthMismatch`, `ExportWalletMissingFields`, `DeriveChildLengthOutOfRange`, `Bip388VerifyDistinctness`. PASS.

**Remediation pointer plausibility.** All cited sections (§II.1, §II.2, §II.3, §III.1, §III.2, §IV.1, §IV.2, §V.1.4, §V.2.4, §V.3.4, §V.4.4) exist. PASS.

**Bibliography "Cited in" accuracy** (5 BIPs spot-checked: BIP-38, BIP-44, BIP-85, BIP-87, SLIP-0132). All citation lists confirmed against chapter content. PASS.

**BIP author spot-check (non-BIP-86).** BIP-45, BIP-48, BIP-49, BIP-84, BIP-85, BIP-87 — all confirmed against live mediawiki. PASS.

**URL validity** (5 URLs spot-checked, excluding C-2): `bip-0038`, `bip-0085`, `bip-0049`, `docs.rs/miniscript`, `docs.rs/bip39` — all well-formed and valid. PASS.

**cspell additions** (17 entries verified): all real proper names or technical terms (`Araoz`, `Matias`, `Alejo`, `Fontaine`, `Weigl`, `Kosakovsky`, `Spigler`, `Riccardo`, `Casatta`, `Tolnay`, `satoshilabs`, `Satoshi`, `unconstructed`, `varints`, `formedness`, `multipaths`). PASS.

**Style alignment.** Troubleshooting uses 3-column `| Variant | Likely cause | Remediation pointer |` format throughout; bibliography rows follow pre-existing pattern. PASS.

---

## Verdict

- [ ] 0 C / 0 I — Phase 5.5 ready to close
- [x] Findings present — iterate r2

Two Critical, both in `66-bibliography.md`. Local one-paragraph and one-block-merge fixes. Troubleshooting is clean.
