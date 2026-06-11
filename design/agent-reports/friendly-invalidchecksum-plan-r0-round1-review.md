# R0 round-1 architect review — PLAN_friendly_invalidchecksum_redaction (2026-06-10)

Reviewer: Fable 5 architect agent (R0, pre-implementation). Plan @ design/PLAN_friendly_invalidchecksum_redaction.md, master @ f6023b1. Verdict: YELLOW (0 Critical / 2 Important / 5 Minor). Review verbatim below.

---

## Critical
None.

## Important

**I1 — T1's decode route is not deterministically repair-free as written; the plan must pin the gate explicitly.**
`resolve_no_auto_repair` (repair.rs:401-407) = `no_auto_repair || !tty`, tty = `MNEMONIC_FORCE_TTY` override else `is_terminal()`. Empirically at f6023b1: piped `convert --from ms1=<1-char-corrupted>` → `error: ms1 codex32: InvalidChecksum { checksum: "short", string: "ms10entrsqqq…34v7f" }`, exit 1 — **full input on stderr, leak confirmed live**; `MNEMONIC_FORCE_TTY=1` + same input → auto-repair fires, exit 5, friendly render NEVER reached → T1 can never go green. The repo suite sets `MNEMONIC_FORCE_TTY=1` per the documented contract (repair.rs:379-400), so an ambient env var silently flips T1 onto the repair path. **Fix:** T1 passes global `--no-auto-repair` (leak still renders) OR `env("MNEMONIC_FORCE_TTY","0")`, assert exit 1.

**I2 — The variant sweep enumeration is incomplete (conclusion holds, claim wrong as stated).**
codex32-0.1.0 Error (lib.rs:42-83) has 16 variants; friendly handles 6 explicitly (friendly.rs:58-78) → TEN reach the catch-all, not six. Plan omits `InvalidThreshold(char)`, `InvalidThresholdN(usize)`, `InvalidShareIndex(Fe)`. Verified all three carry only char/usize/Fe, and `Field(field::Error)` carries only ExtraChar(char)/NotAByte(TryFromIntError)/InvalidByte(u8) — nothing stringy. So "ONLY InvalidChecksum embeds the input; one arm suffices" is TRUE — but correct the sweep to the full 10-variant enumeration (enumerate-and-verify-ALL discipline; a leak plan's central claim cannot under-enumerate).

## Minor

**M1 — Leak ALSO fires on the TTY/auto-repair path when corruption exceeds the BCH bound** (FORCE_TTY=1 + 9 corrupted chars → repair fails → same Debug dump). Broader than "piped only"; fix point unchanged. Update the Problem paragraph.
**M2 — T2 placement:** table `ms_codec_structural_arms_render_prose` (friendly.rs:880-942, typed `[(E,&str,&str);7]` — bump arity or add a standalone cell); no existing test pins InvalidChecksum Debug (the fall-through cell friendly.rs:456-461 uses InvalidChar, pins only the prefix). T2 must ALSO assert the constructed `string` payload is ABSENT from the message (unit-level redaction pin) + the per-table `!contains` discipline.
**M3 — Redaction shape diverges from v0.53.3's UnknownHrp head-truncation (full withholding here)** — stronger (chars 9+ are payload; a 12-char head would leak ~3 payload chars); note the deliberate divergence so nobody "harmonizes" it back.
**M4 — Version sites ×4:** Cargo.toml:3, CHANGELOG.md, README.md:13 marker + crates/mnemonic-toolkit/README.md, scripts/install.sh:32 (Cargo.lock follows). Name them.
**M5 — Latent (no action):** repair.rs:862-865 also Debug-dumps codex32 errors but InvalidChecksum is unreachable there (residue==0 ⇒ valid checksum before decode). Optional one-line comment / FOLLOWUP note.
**Verified clean:** no manual quotes the message; single render channel (error.rs:766 → friendly mapper); no clap/schema impact; standalone v0.53.4 right (nothing else toolkit-side unblocked — the pin bump is publish-blocked).

## Verdict
**YELLOW — 0 Critical / 2 Important.** Fold I1 (deterministic repair-disable in T1) + I2 (10-variant sweep), re-dispatch.
