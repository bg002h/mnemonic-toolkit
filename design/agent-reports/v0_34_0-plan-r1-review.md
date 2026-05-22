# v0.34.0 nostr-key-wrappers — plan-doc opus R1 verification review (verbatim)

**Date:** 2026-05-22
**Reviewer:** opus `feature-dev:code-reviewer` (fresh agent `a29a7b996d3c85e6d`)
**Target:** revised plan `design/IMPLEMENTATION_PLAN_v0_34_0_nostr_key_wrappers.md` (commit `5857213`) vs R0 report + spec + live source `f501ec3`
**Verdict:** **YELLOW** — 0 Critical, 0 Important, 2 Minor (both doc-internal stale cross-references; folded immediately after).

---

## R0 fold verification summary (all confirmed against live source)

- **C1 resolved ✓** — `nostr.rs` is a binary-crate module (`mod nostr;` in `main.rs`, A0.2 line 106; file-structure line 21; architecture line 7). `main.rs:5,12,16` declares `mod cmd; mod error; mod network;`; `lib.rs:67-90` does NOT export them. So from `src/nostr.rs`, `crate::error::ToolkitError`, `crate::cmd::convert::ScriptType` (`convert.rs:357`), `crate::network::CliNetwork` (`network.rs:12`) resolve. In `cmd/nostr.rs`, `crate::nostr::*` resolves, `mlock` is `mnemonic_toolkit::mlock` (lines 751, 772).
- **C2 resolved ✓** — A0.3 adds `impl ScriptType { pub fn as_str(self) }`. `ScriptType` (`convert.rs:356`) derives `Copy, PartialEq, Eq` → round-trip test compiles.
- **C3 resolved ✓** — `flag_is_secret("--secret")` single-arg matches `secrets.rs:49`.
- **C4 resolved ✓** — A0.1 adds enum + `kind()` (`error.rs:489-538`) + `message()` (`error.rs:543-713`) + `exit_code()` (`error.rs:436-483`) arms; removed phantom Display arm. `message()` returns `String` and surrounding arms use `format!`/`.clone()` → plan's `NostrKeyParse(msg) => format!("nostr: {msg}")` is correct. `details()` has `_ => None` (`error.rs:742`); `impl Display` = `write!(f, "error: {}", self.message())` (`error.rs:749`).
- **I1 resolved ✓** — no `crate::mlock` in code blocks; only `mnemonic_toolkit::mlock::pin_pages_for`.
- **I2 resolved ✓** — B1 stub (577), B2 `run` (648), dispatch (600) all `&NostrArgs -> Result<u8>` / `Ok(0)`; dispatch has no `.map(|_|0)`. Matches `electrum_decrypt.rs:85-90` + `main.rs:121-123`. `args.script_type.unwrap_or(...)` (Option<ScriptType>: Copy) + `args.pubkey.as_deref()` valid.
- **I3 resolved ✓** — valid NIP-19 set verified verbatim: npub `npub10elf…qzvjptg` ↔ `7e7e9c42…86addf4e`; nsec `…fe5` ↔ `67dea2ed…92ffa`. Comment notes DISTINCT keys; no test asserts a keypair; "Same key" mislabel gone.
- **I4 resolved ✓** (with a §8 straggler — Minor 2) — plan O1 (1038) + spec §4 (97) say emit WIF plainly, no redaction pathway; `[SECRET]` gone from spec example (86-94); B3 emits plainly (767). Validated vs `secret_taxonomy.rs:72`.
- **I5/M1/M3 resolved ✓** — `default_value_t = CliNetwork::Mainnet` kept + "do NOT add Default" note (565-566); `CliNetwork` has no Default (`network.rs:10`); enum-placement note matches `main.rs:59-92`; `[SECRET]` removed.

## Critical
None.

## Important
None.

## Minor (both folded in commit following this review)

**Minor 1 — Stale `ScriptType::as_str()` prose at plan line 682** (Task B2 Step 3 trailing note) said "exists at `convert.rs:67`", contradicting the C2 fold (A0.3 CREATES it). Code blocks were correct; only the parenthetical was stale. **Fixed** → now points to Task A0.3.

**Minor 2 — Spec §8 reuse-map (line 134)** still listed "secret-on-stdout redaction pathway" — the premise R0 I4 falsified. **Fixed** → struck, with a pointer to §4 / R0 I4.

## New-issue scan — none introduced
- No `library`/`lib` mislabel for `nostr` in code-bearing text.
- Two `nostr` modules (`crate::nostr` crypto + `crate::cmd::nostr` CLI) referenced unambiguously; `cmd/nostr.rs` registered via `pub mod nostr;` in `cmd/mod.rs`.
- No `.map(|_|0)` / `Result<()>` / stray `Ok(())` stragglers (line 597 is prose saying NOT to use it).
- O2 (Electrum prefix strings) correctly OPEN + non-blocking (1039).
- `address_for` call forms match `convert.rs:1558-1565`; gui-schema auto-walk confirmed (`gui_schema.rs:990`) → C2 verify-only.

## Verdict: YELLOW → GREEN after Minor fold
0 Critical / 0 Important (the repo's stop bar). 2 doc-internal Minors folded immediately. Plan ready to execute.
