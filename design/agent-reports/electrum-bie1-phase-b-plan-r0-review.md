# Electrum BIE1 Phase-B plan — opus R0 + R1 review (verbatim)

Reviews of the PHASE B section of `design/PLAN_electrum_bie1_storage.md` (Cycle 19, feature-dev:code-reviewer, opus). Persisted per CLAUDE.md. All findings folded; R1 GREEN.

## R0 — VERDICT: YELLOW (0 Critical / 2 Important / 5 Minor)

Architecture sound (decrypt-before-sniff correct; detection non-false-positiving — every existing sniff requires JSON `{`/text prefixes, a BIE1 `QklFM…` blob sniffs NoMatch; secret hygiene well-reasoned).

**I1 — stdin guard underspecified vs the hoisted token read.** Password resolves at ~:336, AFTER `read_and_validate_bsms_token` (:299-302) and `read_blob` (:336) drain stdin. THREE potential stdin consumers (blob=-, bsms-token=-, decrypt-password-stdin). Fix: fold `--decrypt-password-stdin` into the existing `token_stdin_count` arithmetic at :267-294 as a single ">1 total" guard (hoisted, not local). Add test `--bsms-encryption-token=- --decrypt-password-stdin`.

**I2 — `--format`/decrypt precedence unspecified.** Decrypt-before-sniff is right, but lock: detection+decrypt run UNCONDITIONALLY regardless of `--format`; after blob replacement the normal `--format`/sniff logic runs on the recovered JSON. `--format bsms`+BIE1 → decrypt notice THEN `ImportWalletFormatMismatch{bsms,electrum}`. Document + test.

**M1** SemVer PATCH v0.33.2 CORRECT (reject architect MINOR): CLAUDE.md + Cycle-13 `--from` precedent — new flag names on existing subcommand = additive PATCH + mandatory GUI lockstep; MINOR reserved for new top-level subcommands / breaking.
**M2** `blob = decrypted.to_vec()` drops the `Zeroizing` wrapper; ship-acceptable (mlock-pin + watch-only output + read_blob is plain Vec for all formats) but sharpen the `import-wallet-blob-zeroizing` FOLLOWUP to "load-bearing."
**M3** Fixture: Phase A already proves crypto vs Electrum's own KATs, so the Phase-B test is CLI-wiring; PRIMARY fixture = independent pure-Python `ecdsa`; toolkit-self-gen only as documented fallback.
**M4** `detect_storage_magic` + `ecies_decrypt_storage` must share ONE trimmed input + same `base64::STANDARD` engine.
**M5** Lock exact notice strings so test cells assert fixed text; `ElectrumStorageMagic` in `electrum_crypto.rs`.

## R1 — VERDICT: GREEN

(a) 3-way hoisted stdin guard internally consistent with `token_stdin_count` (:267-294) — single ">1 total" subsumes the old pairwise checks; hoist rationale sound (token + blob drain stdin before password resolves). (b) `--format`-independent decrypt precedence unambiguous — detect+decrypt before sniff (:339), blob replaced, existing `--format bsms` arm (:341-363) fires mismatch on recovered JSON; cell #7 asserts notice-then-mismatch (both stderr; mismatch is correct verdict). I2/M2/M4/M5/M1/M3 consistent with cited source lines.
