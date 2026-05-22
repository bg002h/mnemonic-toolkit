# v0.34.1 import-wallet hygiene — end-of-cycle opus review (verbatim)

**Date:** 2026-05-22
**Reviewer:** opus `feature-dev:code-reviewer` (agent `ab6a9a96e76c10448`)
**Scope:** implemented diff `6576cbf..HEAD` — `import_wallet.rs` (`acabdc5` mlock + `16b5946` zeroize) + release commit `683d73d`, vs the R0→R1 GREEN plan.
**Verdict:** **GREEN** — 0 Critical / 0 Important. Cleared to tag `mnemonic-toolkit-v0.34.1`.

## Critical / Important — None

## Verification
1. **mlock (C1 fix) as-built — CONFIRMED.** Single `let mut _pin_blob = …pin_pages_for(&blob);` at `:398` (right after the `:390` blob binding, unconditional). Re-pinned via `drop(std::mem::replace(&mut _pin_blob, …pin_pages_for(&blob)))` at the BIE1 reassign (`:444-447`) + Round-2 reassign (`:1057-1060`). Old `_pin_pt` fully removed; `_pin_pw` untouched. Pin-new-before-munlock-old ordering holds (mem::replace arg evaluates before the old guard drops; new buffer allocated before old freed → no page reuse; munlock never dereferences). Exactly one live guard; no stale end-of-`run()` munlock.
2. **zeroize as-built — CONFIRMED.** `decrypt_bsms_record -> Result<Zeroizing<String>, ToolkitError>` (`:2178`), wrapped via `.map(Zeroizing::new)` (`:2207`). Round-2: `blob = Zeroizing::new(plaintext.as_bytes().to_vec());` (`:1055`). Round-1 else: `Zeroizing::new(raw_text)` (`:2335`). `parse_round1(&text)` unchanged via deref. Exactly two consumers, both handled; no un-scrubbed copy persists.
3. **No behavior change — CONFIRMED.** Only the `_pin_blob` guard lifetime + `decrypt_bsms_record` return type + its two consumers change. No control-flow/output/error-path change. (122 ok / 0 fail + clippy clean, consistent with type-only hardening.)
4. **Release hygiene — CONFIRMED.** `Cargo.toml` 0.34.1; `install.sh:32` self-pin `mnemonic-toolkit-v0.34.1` (matches the tag → install-pin-check WILL pass); CHANGELOG `[0.34.1]`; both FOLLOWUPs resolved v0.34.1. SemVer PATCH; no CLI surface change → no GUI/manual/sibling lockstep.

## Minor (out-of-scope observation, confidence ~30 — NOT this cycle)
`BsmsToken.raw: Vec<u8>` (`:2135`) + `read_bsms_token`'s returned `String` (`:2125`) are neither mlock-pinned nor zeroized — pre-existing, outside the two-FOLLOWUP scope, lower sensitivity (BIP-129 session token, not a seed). File as a future hygiene-sweep FOLLOWUP.

## Verdict: GREEN — cleared to tag v0.34.1.
