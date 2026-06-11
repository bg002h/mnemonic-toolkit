# PLAN — friendly mapper: redact codex32 `InvalidChecksum`'s embedded full input (leak-hardening)

**Cycle:** toolkit v0.53.4 (PATCH, can ride the next release if one is imminent — standalone otherwise) · **Source SHA:** `f6023b1` · **Resolves:** `friendly-ms1-invalidchecksum-echoes-full-input` (design/FOLLOWUPS.md, spawned by v0.53.3 R0-r1 M1).

## Problem (verified at f6023b1)

`friendly.rs:79` catch-all `ms_codec::Error::Codex32(c) => format!("ms1 codex32: {:?}", c)` Debug-prints `codex32::Error::InvalidChecksum { checksum: &'static str, string: String }` (verified at codex32-0.1.0 lib.rs:58-62) — the `string` field is the FULL input. An uncorrectable lowercase ms1 (known HRP, bad checksum) therefore echoes the full near-secret on stderr — on BOTH the piped/`--no-auto-repair` path AND the TTY/auto-repair path when corruption exceeds the BCH bound (repair fails → same Debug dump; R0-r1 M1). Confirmed live at f6023b1: piped `convert --from ms1=<1-char-corrupt>` → `error: ms1 codex32: InvalidChecksum { checksum: "short", string: "ms10entrs…34v7f" }`, exit 1. The v0.53.3 `UnknownHrp` truncation does not cover this (known-HRP path). Variant sweep (R0-r1 I2, complete): codex32-0.1.0 Error has 16 variants; friendly handles 6 explicitly (friendly.rs:58-78) → TEN reach the catch-all — `Field(field::Error)`, `IdNotLength4`, `IncompleteGroup`, `InvalidLength`, `InvalidChar`, `InvalidCase`, `InvalidThreshold`, `InvalidThresholdN`, `InvalidShareIndex`, `InvalidChecksum`. Verified ONLY `InvalidChecksum` embeds a String (the others carry char/usize/Fe; `field::Error` carries ExtraChar(char)/NotAByte/InvalidByte) — one explicit arm suffices.

## Fix

Add an explicit arm ABOVE the catch-all:
`Codex32(codex32::Error::InvalidChecksum { checksum, string }) => format!("ms1 codex32: invalid {checksum} checksum ({} chars; input withheld)", string.chars().count())` — names the checksum kind + length (actionable: user can spot truncation) without the bytes. FULL withholding (not the v0.53.3 UnknownHrp 12-char-head truncation) is DELIBERATE (R0-r1 M3): chars 9+ of an ms1 are payload, so a head-echo would leak ~3 payload chars; do not "harmonize" to a head-echo. Catch-all stays for the other variants (none carry the input).

## Tests (TDD red-first)

- **T1:** a valid-length lowercase ms1 with a corrupted data char, driven through a decode path that does NOT auto-repair — pass global `--no-auto-repair` OR set `env("MNEMONIC_FORCE_TTY","0")` on the Command (R0-r1 I1: the bare ambient-non-TTY default is unreliable — the repo suite sets FORCE_TTY=1, which would flip T1 onto the repair path → exit 5, friendly render never reached); use `convert --from ms1=<bad> --to phrase` (verified to render `friendly_ms_codec`), assert exit 1, stderr does NOT contain the input string, DOES contain `invalid` + `checksum` + `input withheld`.
- **T2 (guard):** a friendly unit cell pinning the new InvalidChecksum message — and CRUCIALLY asserting the constructed `string` payload is ABSENT from the rendered message (unit-level redaction pin, not just message needles) + `!contains("InvalidChecksum")`. The structural-arms table `ms_codec_structural_arms_render_prose` (friendly.rs:880-942) is typed `[(E,&str,&str);7]` — add a STANDALONE cell rather than bump the arity (cleaner). The `!contains("unhandled")` wildcard discipline is preserved.

## Ritual

CHANGELOG `[0.53.4]` (or fold into the next release's entry); version sites (R0-r1 M4: Cargo.toml:3 + CHANGELOG.md + README.md:13 marker + crates/mnemonic-toolkit/README.md + scripts/install.sh:32; Cargo.lock follows) IF released standalone; FOLLOWUPS resolve. No clap/schema/manual-flag impact; manual advisory tables unaffected (no documented message quotes this string — verify with a grep).

## Non-goals

Upstream codex32 changes; other friendly arms (sweep says none embed input); UnknownHrp (done v0.53.3).
