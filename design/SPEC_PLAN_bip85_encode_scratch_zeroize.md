# SPEC + PLAN (combined) — bip85 encode-helper internal-scratch Zeroizing

**Slug:** `bip85-encode-helper-internal-scratch-zeroizing` (filed by cycle-15 Group A, v0.69.0). **Repo:** mnemonic-toolkit.
**Base:** `origin/master` = `d6e8757d` (v0.69.0). **Branch:** `feature/bip85-encode-scratch-zeroize` (worktree `wt-cycle15t`). **Target:** toolkit **PATCH 0.69.1**.
**Why combined doc:** the change is 3 mechanical `Zeroizing` wraps in ONE file with no ripple/signature/lint/wire change — proportionate to one R0'd spec+plan. (Still R0-gated before code + mandatory whole-diff review after, per the hard gate.)
**Coordination:** own-account instance owns the MAIN checkout (`feature/own-account-subset-search`); all work in `wt-cycle15t`; version coordinated at ship (own-account builds lower, no collision).

---

## Problem (the residue, cites re-verified vs `d6e8757d` — bip85.rs untouched by Group A)
The 7 `format_*` BIP-85 fns already return `SecretString` (Lane T, v0.68.0). But three INTERNAL locals materialize the secret in a bare heap allocation BEFORE the wrapped return, lingering un-scrubbed until function exit:
- `bip85.rs:189` `let encoded = base64_standard(&entropy[..]);` — full base64 of the 64-byte entropy (password material); only `encoded[..length]` is wrapped into the `SecretString` return (`:190`), the full `encoded` String lingers.
- `bip85.rs:204` `let encoded = base85_btc(&entropy[..]);` — same, base85 (`:205`).
- `bip85.rs:252` `let mut out: Vec<String> = Vec::with_capacity(rolls as usize);` — dice per-roll digit strings; `out.join(",")` is wrapped at `:274`, but the `Vec<String>` aggregate (the dice secret) is bare.

This is the same residue class as the (out-of-scope) `compute_outputs` bare-`String` output Vec; defense-in-depth, MED/LOW severity (the final SecretString return IS wrapped; these are transient pre-return copies of derived secret material).

## Fix (3 wraps, bip85.rs only — no ripple)
1. `:189` → `let encoded = Zeroizing::new(base64_standard(&entropy[..]));` (`encoded[..length as usize].to_string()` at `:190` indexes through `Deref<Target=String>` → str slice; unchanged).
2. `:204` → `let encoded = Zeroizing::new(base85_btc(&entropy[..]));` (`:205` likewise).
3. `:252` → `let mut out: Zeroizing<Vec<String>> = Zeroizing::new(Vec::with_capacity(rolls as usize));` (`out.push(...)` via DerefMut, `out.len()` / `out.join(",")` via Deref to `[String]`; all unchanged).

`Vec<String>: Zeroize` (blanket `Vec<T: Zeroize>` + `String: Zeroize`) so `Zeroizing<Vec<String>>` is valid and scrubs each String on drop. `Zeroizing` is already imported in `bip85.rs` (`:53`).

## Explicitly OUT of scope (with rationale)
- The per-roll `buf` (`:253` `let mut buf = vec![0u8; bytes_per_roll]`, raw SHAKE bytes) — NOT wrapped: it's a 1-4 byte transient overwritten every iteration, and wrapping it would break `for &b in &buf` (`&Zeroizing<Vec<u8>>` is not `IntoIterator`) + `reader.read(&mut buf)` — a deref ripple disproportionate to a per-roll byte scratch the slug did not name. Leave bare.
- The encode-helper internal `out: String` buffers (`base64_standard:288`, `base85_btc:333`) — NOT wrapped: they are **moved out** by return into the caller's `encoded` (now `Zeroizing`), so that allocation is scrubbed on the caller's drop; no separate lingering copy. Wrapping inside the helpers would be redundant.

## Non-goals / invariants
- **No signature change** (`format_*` and the helpers keep their types) → PATCH, not MINOR.
- **No lint change:** bip85.rs is already a `ZEROIZE_ROWS.source_file` (3 rows: `derive_entropy` Zeroizing return, entropy locals, `format_*` SecretString) — adding allocations needs NO new row, NO count-guard change, NO partition/floor change.
- **Output byte-identical:** the `SecretString` returns are built from the same bytes (`encoded[..length]` / `out.join(",")`); Deref renders verbatim. The existing BIP-85 vector tests must stay GREEN unchanged.
- **No clap/CLI/wire/schema/manual/GUI surface** → no mirror gates.

## SemVer + release
- **PATCH 0.69.0 → 0.69.1.** Version sites (R0-verify at write-time): `crates/mnemonic-toolkit/Cargo.toml:3`, root `README.md` + crate `README.md` (`<!-- toolkit-version: -->`), `scripts/install.sh` self-pin, root `Cargo.lock` + `fuzz/Cargo.lock` (toolkit pkg), `CHANGELOG.md` new `[0.69.1]` entry.
- FOLLOWUPS: flip `bip85-encode-helper-internal-scratch-zeroizing` `open → resolved` in the shipping commit (note `buf` + helper-internal-`out` dispositioned out-of-scope as above).

## Plan (TDD, single subagent, 2 phases)
**P1 — the wraps (`src/bip85.rs`):**
- RED: add a source-grep fence test (mirrors Lane T's seedqr T8 pattern) to `#[cfg(test)] mod tests` asserting the three wraps are present — e.g. assert the file source contains `Zeroizing::new(base64_standard(`, `Zeroizing::new(base85_btc(`, and `out: Zeroizing<Vec<String>>` (assemble the needle at runtime via `format!`/concat to avoid the test's own source self-matching the grep). RED before the wraps, GREEN after.
- GREEN: apply the 3 wraps.
- Verify: the existing BIP-85 vector/output tests stay GREEN unchanged (byte-identical); `cargo test -p mnemonic-toolkit` full suite + `cargo clippy --workspace --all-targets -- -D warnings` clean. (No caller edits — these are function-local; confirm no `clippy::redundant_clone`/type errors.)
- Commit P1: `feat(bip85-scratch): wrap encode/dice internal scratch in Zeroizing (derived-secret hygiene)`.

**P2 — release 0.69.1 + FOLLOWUP flip:**
- Version sweep 0.69.0 → 0.69.1 (6 sites + CHANGELOG `[0.69.1]`).
- `design/FOLLOWUPS.md`: flip the slug `open → resolved` (v0.69.1 note; `buf`/helper-`out` out-of-scope dispositioned).
- Final gate: full suite (incl. `readme_version_current` @ 0.69.1) + clippy clean.
- Commit P2: `release(bip85-scratch): toolkit v0.69.1 — bip85 encode/dice scratch Zeroizing`.

**Whole-diff review (orchestrator, mandatory):** independent adversarial review of the diff; verify the 3 wraps scrub correctly with no ripple, output byte-identical, no lint/version drift; then FF-push + tag `mnemonic-toolkit-v0.69.1`.

## R0 questions
1. Are the 3 wraps genuinely ripple-free (esp. `encoded[..length].to_string()` through Deref and `out.join(",")` on `Zeroizing<Vec<String>>`)? Confirm by applying+building.
2. Is leaving `buf` + the helper-internal `out` bare the right scope call, or does the first-class-hygiene bar demand `buf` too (accepting the `for &b in &buf` ripple)?
3. Is the source-grep fence the right TDD shape (no cleaner observable RED for a drop-scrub), and is the self-match trap handled?
4. Anything missed (a 4th residue local in these fns; a version site; PATCH-vs-MINOR)?
