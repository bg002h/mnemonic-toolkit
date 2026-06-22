# R0 REVIEW (verbatim) — bip85 encode-scratch Zeroizing spec+plan

**Dispatched:** 2026-06-22. **Base:** `d6e8757d` (v0.69.0). **Doc:** `design/SPEC_PLAN_bip85_encode_scratch_zeroize.md`.
**Verdict:** GREEN (0C/0I) in one round — ready for implementation.

---

Tree is clean (only the untracked SPEC_PLAN doc remains; the harness note refers to my now-reverted edit, and `git diff --stat` confirmed empty). All review questions are resolved. Here is the verdict.

---

## VERDICT: GREEN (0C / 0I)

**What I ran (in worktree `wt-cycle15t`, base `d6e8757d` = v0.69.0):** applied all 3 wraps, then:
- `cargo build -p mnemonic-toolkit --tests` → **clean** (9.3s). Confirms the Deref/DerefMut ripples: `encoded[..length as usize].to_string()` (`:190`/`:205`) compiles through `Deref<Target=String>→str`, and `out.push/.len/.join` (`:254`,`:269`,`:274`) compile through DerefMut/Deref. No other line in the 3 fns breaks. Also proves `Vec<String>: Zeroize` holds (zeroize **1.8.2**, blanket `Vec<T: Zeroize>` + `String: Zeroize`).
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` → **clean** (no `redundant_clone`, no `type_complexity`).
- bip85 KATs (13 tests, bin target) → **GREEN**, incl. `pwd_base64_matches_spec`, `pwd_base85_matches_spec`, `dice_d6_10_rolls_matches_spec` — output byte-identical.
- `lint_zeroize_discipline` (6 tests) → **GREEN** — confirms no new row / no count-guard / no floor change needed (file-level + row-count lint; bip85.rs already a `source_file`).
- Full `cargo test -p mnemonic-toolkit` (all targets) → **0 failures**; only pre-existing env-gated/bitcoind tests ignored.
- **Reverted** `git checkout -- crates/mnemonic-toolkit/src/bip85.rs`; `git status` clean (only the untracked SPEC_PLAN doc, as expected).

**Cite re-verification vs `d6e8757d`:** all 3 target lines, the `Zeroizing` import (`:18`), the FOLLOWUPS slug body (`FOLLOWUPS.md:4502-4512`, status `open`), and all 6 version sites (`Cargo.toml:3`, both READMEs `toolkit-version: 0.69.0`, `install.sh:32`) match the doc.

**Q4 (buf scope) — clear call: leaving `buf` bare is CORRECT and in-scope-excluded.** `buf` is a 1-4 byte SHAKE scratch *overwritten every loop iteration* via `reader.read(&mut buf)` — it never accumulates and is the raw DRNG stream, not the rendered roll value. The rendered secret aggregate (`out`) is what's being scrubbed. Wrapping `buf` would force `for &b in &*buf` + `reader.read(&mut buf[..])` deref edits for a per-iteration byte scratch the slug never named, with no residue benefit over the already-Zeroized 64-byte `entropy` it derives from. The helper-internal `out: String` (`:288`,`:333`) is likewise correctly excluded — it's *moved* into the caller's now-`Zeroizing` `encoded`, so it scrubs on the caller's drop; wrapping inside the helper would be a redundant double-wrap. The first-class-hygiene bar does **not** demand either.

**Q7 (completeness):** No 4th residue local. The `format_*` returns are already `SecretString` (Lane T); `entropy` is already `Zeroizing<Vec<u8>>`; the per-fn `SecretKey`/`Xpriv` stack locals are separately tracked as third-party-blocked FOLLOWUPs. PATCH (0.69.1) is correct — no signature change. Version sites enumerated correctly.

**Minor (non-blocking, 0 of them gating):** none. The source-grep fence TDD shape (Q3/Q6) is the right RED-then-GREEN for a drop-scrub with no observable behavior — there is no cleaner observable RED (Drop-time zeroize leaves no testable artifact); assembling the needle at runtime via `concat!`/`format!` correctly defuses the self-match trap (proven by Lane T's T1 precedent in the same file). The byte-identical KATs are the behavioral guard.

**Ready for implementation.** The plan is GREEN — proceed to P1 (RED fence test → 3 wraps → GREEN) then P2 (version sweep + FOLLOWUP flip), with the mandatory whole-diff review before FF-push + tag.
