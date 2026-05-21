# mnemonic-toolkit-v0.32.3 Implementation Plan (Cycle 17 — bsms-encryption-cross-impl-coinkite-python-smoke)

> **For agentic workers:** REQUIRED SUB-SKILL: `superpowers:subagent-driven-development` or direct execution.

**Goal:** Ship `mnemonic-toolkit-v0.32.3` (SemVer-PATCH; test/fixture-only). Closes `bsms-encryption-cross-impl-coinkite-python-smoke` FOLLOWUP — the third + final BIP-129-BSMS arc step. Pin the toolkit's BIP-129 §Encryption implementation against the independent Coinkite Python reference via VENDORED Coinkite-generated cross-impl fixtures + a regeneration script. **No live clone / pip / network in CI** (user-locked vendored-only scope).

**Architecture:** The Coinkite reference (`coinkite/bsms-bitcoin-secure-multisig-setup`, pinned SHA `c30abe3a6d9823b6a3003e89acd66b9f38e11f1c`, frozen 2023-01-24) implements BIP-129 §Encryption byte-identically to the toolkit (recon-verified: PBKDF2-SHA512 `b"No SPOF"` salt=raw-token-bytes 2048/32 → SHA256 hmac-key → HMAC-SHA256 over `hex_token+plaintext` → Ctr128BE → `mac.hex()+ciphertext`; sole dep `pyaes`). We generate cross-impl wires by running Coinkite's `encrypt()` (in a one-off local venv) over a REAL importable BSMS Round-2 descriptor, commit the resulting wire + token as fixtures, and add a default-on integration test that feeds the vendored Coinkite-generated wire into `import-wallet --format bsms --bsms-encryption-token` and asserts a successful cross-impl decrypt + import. The generation is deterministic (IV = MAC[:16] = f(token, plaintext)), so the fixture is reproducible — a committed `regen_coinkite_vectors.py` documents the exact recipe + pinned SHA for future refresh.

**Tech Stack:** Rust (test) + a committed Python regen script (not run in CI). Zero new crate deps; zero source-code changes (test/fixture/doc-only); zero new clap flags; zero GUI lockstep.

**Recon (verified 2026-05-21):**
- Coinkite `bsms/encryption.py` matches the toolkit recipe exactly (cross-checked function-by-function).
- A Coinkite-encrypted Round-2 descriptor (real `bsms-2line-multi-2of3.txt` plaintext, EXTENDED 16-byte token `00112233445566778899aabbccddeeff`) decrypts + MAC-verifies + parses + imports in the toolkit, producing the correct `sh(multi(2,...))` descriptor (token-width-32 NOTICE confirms EXTENDED mode).
- The existing `bsms-encrypted-standard-tv3.dat` is ALREADY a Coinkite-generated TV-3 Round-1 wire (STANDARD 8-byte token; Cycle 7a). So the Round-1 direction is already cross-validated; this cycle adds the Round-2 descriptor direction + a self-describing regen recipe.
- `tests/external/` does not yet exist (will create).

**SemVer rationale (v0.32.2 → v0.32.3 PATCH):** test/fixture/doc-only; no source, no CLI surface, no behavior change.

## File structure

### Fixtures (toolkit)
- `crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-coinkite-xref-round2-2of3.dat` — the Coinkite-`encrypt()`-generated hex wire (984 hex chars) for the `bsms-2line-multi-2of3.txt` plaintext + EXTENDED token.
- `crates/mnemonic-toolkit/tests/fixtures/wallet_import/bsms-coinkite-xref-round2-2of3-token.hex` — `00112233445566778899aabbccddeeff`.

### Regen tooling (toolkit; committed, NOT run in CI)
- `crates/mnemonic-toolkit/tests/external/regen_coinkite_vectors.py` — the generator: clones/uses the pinned Coinkite ref, reads the plaintext fixture as EXACT bytes (NO `.strip()`/`.rstrip()` — preserves the trailing `\n`; R0 M2), reads the TOKEN with `.strip()` before `bytes.fromhex`, runs `encrypt(KDF(token), token, plaintext)`, **self-verifies by re-decrypting its own output + asserting byte-equality with the plaintext BEFORE writing** (R0 M2), then writes the `.dat` + token fixtures. Self-contained recipe (documents the pinned SHA + `pip install pyaes` venv step).
- `crates/mnemonic-toolkit/tests/external/README.md` — documents: the pinned Coinkite SHA, the `pyaes` dependency, the venv recipe, WHY the fixtures are vendored (zero-CI-fragility cross-impl pin), and HOW to refresh them (run the script) if Coinkite ever changes.

### Test files (toolkit)
- `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms_encrypted.rs`:
  - **`coinkite_xref_round2_full_plaintext_byte_equal` (R0 I1 — the strong pin)** — decrypt the vendored Coinkite wire via `mnemonic_toolkit::bsms_crypto::decrypt` (deriving the key from the EXTENDED token, IV=MAC[:16], as `tv3_decrypted_plaintext()` does) and assert the decrypted bytes EQUAL the `bsms-2line-multi-2of3.txt` fixture bytes EXACTLY. This exercises the keystream over the FULL ~460-byte plaintext (not just the descriptor line), proving the independent Coinkite ciphertext decrypts byte-for-byte.
  - `coinkite_xref_round2_descriptor_imports` — end-to-end CLI check: feed the vendored Coinkite Round-2 wire + token into `import-wallet --format bsms --json`; assert (a) the decrypt NOTICE (token width 32 hex chars; MAC verified) + the expected 2-line-excerpt WARNING, (b) the imported `descriptor` byte-equals the descriptor produced by importing the PLAINTEXT `bsms-2line-multi-2of3.txt` directly. This is also the FIRST EXTENDED-token wire that actually DECRYPTS via the CLI (the existing `extended_mode_32_hex_token_passes_width_check` only exercised width-acceptance against a MAC-failing wire).
  - `coinkite_xref_round2_wrong_token_mac_mismatch` — flipped token → `BsmsMacMismatch` (exit 2) on the Coinkite-generated wire (confirms the MAC binds the Coinkite ciphertext to the token).

### Documentation (toolkit)
- `docs/manual/src/45-foreign-formats.md` (or the BSMS encryption section) — a short note that the toolkit's BIP-129 encryption is cross-validated against the Coinkite Python reference via vendored fixtures (pinned SHA), with the regen recipe pointer.

### Release tooling
- `Cargo.toml:3` — `0.32.2` → `0.32.3`.
- `CHANGELOG.md` — `## [0.32.3]`.
- `scripts/install.sh:32` — pin → `v0.32.3`.
- `design/FOLLOWUPS.md` — close `bsms-encryption-cross-impl-coinkite-python-smoke`.

## Tasks

### Task 1: Phase 2 — vendor fixtures + regen script + README
- [ ] Copy the generated `.dat` + token into `tests/fixtures/wallet_import/`.
- [ ] Write `tests/external/regen_coinkite_vectors.py` + `tests/external/README.md` (pinned SHA `c30abe3a6d9823b6a3003e89acd66b9f38e11f1c`, pyaes venv recipe).
- [ ] Commit Phase 2.

### Task 2: Phase 3 — cross-impl integration test
- [ ] Add the 2 cells (descriptor-byte-equal + wrong-token-MAC-mismatch).
- [ ] Build + run; confirm cross-impl agreement.
- [ ] Commit Phase 3.

### Task 3: Phase 4 — Manual note
- [ ] Cross-impl-validation note + regen pointer.
- [ ] Manual lint.
- [ ] Commit Phase 4.

### Task 4: Phase 5 — Cycle close
- [ ] Version bump + install.sh + CHANGELOG.
- [ ] Pre-tag audit (test + clippy + manual lint).
- [ ] Opus end-of-cycle review.
- [ ] Commit + tag mnemonic-toolkit-v0.32.3 + push + GH Release.
- [ ] install-pin-check CI green.
- [ ] Close FOLLOWUP + memory. **(Closes the entire BIP-129-BSMS arc.)** The closure note MUST explicitly record (R0 Q10) that the FOLLOWUP-body's (b) run-`python3 test.py` + (c) live-CI-gating were intentionally DROPPED per the user-locked vendored-only scope (not an oversight) — and explicitly WAIVE the live-CI residual (rationale: frozen Coinkite repo + vendored-output byte-pin + existing TV-3 byte-exact cross-validation make automated drift-gating low-value). Confirm no sibling-repo companion entry needs lockstep (toolkit-internal → none).

## Cross-phase invariants
- Opus R0 review on plan-doc BEFORE Phase 2.
- Opus end-of-cycle review BEFORE tag.
- No `cargo fmt --all`.
- No GUI lockstep (test-only).
- install-pin-check CI gate.

## Risk register
- **Fixture reproducibility** — the regen is deterministic (IV derived from token+plaintext), so a future `regen_coinkite_vectors.py` run with the same pinned SHA + plaintext + token yields a byte-identical `.dat`. If the plaintext fixture (`bsms-2line-multi-2of3.txt`) ever changes, the cross-impl wire must be regenerated (the descriptor-byte-equal test would catch a stale wire by failing to import the right descriptor, but the regen script is the source of truth).
- **No CI external dependency** — by vendoring, the default `cargo test` + CI need NO Coinkite clone, NO pyaes, NO network. The regen script is developer-run-only (documented in the README; NOT wired into any CI workflow).
- **EXTENDED-token coverage** — the new fixture uses a 16-byte EXTENDED token (the existing TV-3 is 8-byte STANDARD), so the two cross-impl fixtures together cover both token widths.
- **Plaintext-vs-ciphertext descriptor equality** — the test asserts the Coinkite-decrypted import equals the plaintext import. This is the meaningful cross-impl invariant (independent ciphertext → identical toolkit output). The 2-line excerpt WARNING is expected (the plaintext fixture is a 2-line excerpt) — assert it's present, not absent.

## Self-review (pre-R0 dispatch)
- ✓ Recon confirmed Coinkite matches + the Round-2 wire round-trips through the toolkit.
- ✓ Vendored-only scope (user-locked) — zero CI external dep.
- ✓ Deterministic regen → reproducible fixture + committed recipe.
- ✓ Both token widths covered (STANDARD via existing TV-3 + EXTENDED via the new fixture).
- ✓ SemVer PATCH (test/fixture/doc-only).
- ✓ Test surface: 2 cells (descriptor-byte-equal + wrong-token).
