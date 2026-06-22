# R0 REVIEW — cycle-15 Lane M PLAN-DOC (ms-codec/ms-cli secret zeroize) — Round 1

Verified against `mnemonic-secret origin/master = 6f9f60b`. Lens: ms-codec IS the BIP-39-entropy codec.

## VERDICT: BLOCKING — 0 Critical / 1 Important + 2 Minor

One Important: a missing compile-breaking consumer edit in the P1 reader-set. Everything else verified GREEN.

## AXIS 1 — InspectReport Design A — VERIFIED CORRECT (with the I-1 gap)
- **Deref keeps the 4 ms-cli readers compiling — CONFIRMED:** `inspect.rs:160 .len()`, `:166 .len().saturating_sub(1)`, `:217`/`:247 hex::encode(&report.payload_bytes)` — all Deref-coerce under `Zeroizing<Vec<u8>>` (`zeroize-1.8.2:660 impl Deref, Target=Z`).
- **Hand-rolled redacting Debug mandatory — CONFIRMED:** `Zeroizing<Z>` `zeroize-1.8.2:622 #[derive(Debug,Default,Eq,PartialEq)]` forwards to inner Vec, no redacting impl. RULE Z-DEBUG correct.
- **Marquee RED test** genuinely RED pre-fix (derived Debug leaks `payload_bytes`); must be in-crate `#[cfg(test)]` or via `inspect()` because `#[non_exhaustive]` (`:35`) blocks struct-literal from `tests/`. Plan accounts for both. `Clone` retained, `PartialEq` underived — correct.

### IMPORTANT I-1 — the reader-set OMITS a compile-breaking existing test
Beyond the 4 ms-cli readers there are 2 more repo-wide `.payload_bytes` consumers; one BREAKS the build:
- **`ms-codec/src/inspect.rs:125`** `assert_eq!(r.payload_bytes, entropy)` where `entropy: Vec<u8>` (`:117`). Under Design A LHS becomes `Zeroizing<Vec<u8>>`, which has NO `PartialEq<Vec<u8>>` (derived only `PartialEq<Self>`, `zeroize-1.8.2:622`) → WON'T COMPILE. Fix: deref the field (`*r.payload_bytes` / `.as_slice()`). **FENCE the wrong fix: do NOT derive `PartialEq` on `InspectReport`** (unnecessary public-API growth).
- **`ms-codec/tests/uppercase_envelope.rs:65`** `assert_eq!(ru.payload_bytes, rl.payload_bytes)` — both sides `Zeroizing<Vec<u8>>`, derived `PartialEq<Self>` applies → compiles unchanged (leaks bytes on failure-path only; note as deliberately-left — Minor-B).
The full-suite gate would surface `:125`, but P1 test #3 currently says "Deref keeps everything green" — FALSE for `:125`. **Required fold:** add `inspect.rs:125` to the P1 impl checklist with the deref fix + the "don't derive PartialEq" guard; correct test #3's claim.

## AXIS 2 — Payload/decode wire-stability — VERIFIED CORRECT
`decode.rs:82-83`(Entr)/`:89-90`(Mnem) clone pattern confirmed. Moving `data`/`entropy` straight into `Payload::Entr(data)`/`Payload::Mnem{language,entropy}` is fewer copies + keeps the `Payload` enum shape (`payload.rs:44/:51`) BYTE-IDENTICAL — internal copy-count, not a shape change. Downstream toolkit stability holds. `use zeroize::Zeroizing` at `decode.rs:77` becomes unused → plan flags the `-D warnings` risk. Correct.

## AXIS 3 — shares + Minor-2 — VERIFIED
`shares.rs:130/137`(`secret_s` MOVED into `defining`)/`139`/`148`/`195`/`210` confirmed. `Codex32String` is a foreign String-newtype in dormant `codex32-0.1.0`, no Drop/Zeroize → enumerate-and-defer (Q2 HOLD, FOLLOWUP open) is honest. Minor-2 confirmed (lifetime-min targets `defining`/`parsed`, not `secret_s`).

## P2 RepairDetail (Minor-1) + lint re-anchor (Minor-3) — VERIFIED
- Minor-1 MUST-FIX confirmed: `repair.rs:62 #[derive(Debug, Clone)]` over `original_chunk`/`corrected_chunk: String` (`:65/:66`). NO `{:?}` consumer exists (emit_text uses index/positions; emit_json borrows `&str`) → "drop Debug, keep Clone" is viable + cleanest.
- Minor-3 confirmed: ms-cli verify lint row (`lint:87-89`) anchors `derived_str` at `verify.rs:117` (in `run()`) → FALSE-GREEN while `verify.rs:170 _mnemonic.to_string()` (in `emit_round_trip_ok:169`) is unwrapped. Plan re-points the EXISTING row (lower-blast-radius "wrap the temp" alt is fine).

## RULE Z-DEBUG sweep (4b) — COMPLETE
Applied to InspectReport + RepairDetail. Swept: `format.rs`/`repair.rs` `*Json` derive only `Serialize` (no Debug). The clap `RepairArgs` (`:46 Args, Debug` + `ms1: String`) is pre-existing convention (wrapped via `mem::take`+`Zeroizing` at run() entry), out of scope. Complete.

## P3 publish + version sweep (axis 5) — VERIFIED
Sites EXACTLY: `ms-codec/Cargo.toml:3`(0.5.0→0.6.0), `ms-cli/Cargo.toml:3`(0.9.0→0.10.0) + `:20` pin `=0.5.0`→`=0.6.0`, `Cargo.lock`, root `CHANGELOG.md`, `ms-codec/CHANGELOG.md`. No README/install.sh version pins (READMEs exist but carry no version literal). Publish order ms-codec→ms-cli. CI = clippy `-D warnings` only (`rust.yml:153`, no fmt). Correct.

## Phasing / gates / wire / FOLLOWUPs (axis 6) — VERIFIED
RED-first; FULL `-p ms-codec`+`-p ms-cli` per phase (catches the lint re-anchors); wire guard (P1 #6). Lint counts ms-codec 5→4, ms-cli 10→bump (confirmed vs live arrays). FOLLOWUP flips (6 resolved + #3/#7 open + #4 deferred) verified; new `rust-bitcoin-xpriv-zeroize-upstream` to-be-filed. Mandatory whole-diff review present.

## MINOR (non-blocking)
- **Minor-A:** the precedent cite "`error.rs:257-383` redacting Debug" — the `impl fmt::Debug for Error` actually starts at `error.rs:242`. Precedent real; line range stale. Update to `:242+`.
- **Minor-B:** `uppercase_envelope.rs:65` compiles under Design A (derived `PartialEq<Self>`) but prints raw bytes on failure-path. Note as deliberately-left, or compare `*ru.payload_bytes`.

## Required fold (to reach GREEN)
I-1: add `ms-codec/src/inspect.rs:125` to the P1 checklist (deref the field; fence "don't derive PartialEq on InspectReport"); note `uppercase_envelope.rs:65` compiles as-is; correct P1 test #3's "Deref keeps everything green". Fold Minor-A/B. Persist, re-dispatch.
