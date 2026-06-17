# R0 round-1 architect review — PLAN_friendly_invalidchecksum_redaction (2026-06-10)

Reviewer: Fable 5 architect agent (R0, pre-implementation). Plan @ design/PLAN_friendly_invalidchecksum_redaction.md, master @ f6023b1. Verdict: YELLOW (0 Critical / 2 Important / 5 Minor). Review verbatim below.

---

## Critical

None.

## Important

**I1 — T1's decode route is not deterministically repair-free as written; the plan must pin the gate explicitly.** `resolve_no_auto_repair` (repair.rs:401-407) = `no_auto_repair || !tty`, tty = `MNEMONIC_FORCE_TTY` override else `stdout().is_terminal()`. Empirically at f6023b1: piped `convert --from ms1=<1-char-corrupted>` → `error: ms1 codex32: InvalidChecksum { checksum: "short", string: "ms10entrsqqq…34v7f" }` exit 1 — **full input on stderr, leak confirmed live**; with `MNEMONIC_FORCE_TTY=1` auto-repair fires (exit 5) and the friendly render is NEVER reached. The repo's own suite sets `MNEMONIC_FORCE_TTY=1` (repair.rs:379-400 names the suite as a consumer) — an inherited env var silently flips T1 onto the repair path. **Fix:** T1 passes global `--no-auto-repair` (verified: leak still renders) or sets `env("MNEMONIC_FORCE_TTY","0")`, and asserts exit 1.

**I2 — The variant sweep enumeration is incomplete (conclusion holds, claim wrong as stated).** codex32 Error has 16 variants; friendly.rs handles 6 explicitly → **TEN** reach the catch-all, not six (plan omits `InvalidThreshold(char)`, `InvalidThresholdN(usize)`, `InvalidShareIndex(Fe)`). All three verified char/usize/Fe-only; `Field(field::Error)` carries only char/TryFromIntError/u8 (field.rs:71-80). "ONLY InvalidChecksum embeds the input" is **verified TRUE** — but the sweep sentence must enumerate all 10 (enumerate-and-verify-ALL discipline).

## Minor

**M1 — The leak ALSO fires on the TTY/auto-repair path when corruption exceeds the BCH bound** (verified: FORCE_TTY=1 + 9 corrupted chars → repair fails → same Debug dump). Broader than "piped/--no-auto-repair"; update the Problem paragraph.
**M2 — T2 placement:** the table `ms_codec_structural_arms_render_prose` (friendly.rs:880-942) is typed `[(E,&str,&str);7]` — bump arity or standalone cell; assert the constructed `string` payload ABSENT (unit-level redaction pin) + `!contains("InvalidChecksum")`. No existing test pins the Debug output (the :456-461 cell uses InvalidChar, prefix-only).
**M3 — Redaction shape diverges from the v0.53.3 UnknownHrp truncation precedent (head+…) — DELIBERATE:** chars 9+ of an ms1 are payload, a 12-char head would leak ~3 payload chars; full withholding + char-count is right here. Say so to prevent future "harmonization".
**M4 — Version sites named:** Cargo.toml:3, CHANGELOG.md, README.md:13 + crates/mnemonic-toolkit/README.md:9 markers, scripts/install.sh:32 (+ Cargo.lock via build).
**M5 — Latent footgun, no action:** repair.rs:862-865 Debug-dumps codex32 errors but InvalidChecksum is unreachable there (residue==0 ⇒ checksum valid; corrected strings re-verified). Note in the FOLLOWUP resolution.

**Verified clean:** no manual/manual-gui quoted message; single render channel (error.rs:766); no clap/schema impact; release shape standalone v0.53.4 right (pin bump still publish-blocked).

## Verdict

**YELLOW — 0 Critical / 2 Important.** Fold I1 + I2 (+ minors), re-dispatch.
