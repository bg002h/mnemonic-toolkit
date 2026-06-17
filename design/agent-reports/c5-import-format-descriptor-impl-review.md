# C5 (import --format descriptor) impl review — code reviewer (verbatim)

> Reviewer: opus code reviewer (general-purpose, full tools — byte-diffed helpers vs specter, ran
> suite/clippy, probed edge cases live). Branch `feature/import-format-descriptor` @ v0.58.0.
> Verdict GREEN (0C/0I). The one cosmetic Minor was folded post-review (see footer).

---

**Verdict: GREEN (0C/0I)**

I read the actual code, diffed the duplicated helpers byte-for-byte against the specter originals, built the binary, ran the full test suite (990 passed / 0 failed), ran clippy clean, and probed edge cases live via the CLI. The implementation faithfully mirrors the R0-GREEN plan.

**Critical:** none.
**Important:** none.

**Minor (non-blocking; FYI):**
- `cmd/import_wallet.rs:1178-1184` — the `other =>` unsupported-format error message string still lists only 8 formats (omits `descriptor`). This arm is **unreachable** for `descriptor` (clap's `PossibleValuesParser` includes it, so dispatch always matches the `"descriptor"` arm). Stale list only surfaces in a never-taken internal-error branch. Cosmetic; pre-existing pattern. [FOLDED post-review: added `descriptor` to the list.]

**What I verified (highest-value items):**

1. **Parser correctness (`wallet_import/descriptor.rs`)** — all confirmed:
   - `strip_comments`: filters `!l.is_empty() && !l.starts_with('#')` on trimmed lines → drops only FULL-LINE `#`-comments + blanks; a `wsh(...)#csum` line is kept (starts with `wsh`, not `#`). Arity: `[]`→Err "no descriptor line", `[one]`→Ok, `_`→Err "expected a single descriptor". Mid-line `#<checksum>` preserved.
   - `verify_checksum`: tolerant (validate-if-present, accept-if-absent). Live-proven: checksum-less singlesig accepted; `#deadbeef` refused with "checksum".
   - Parse flow faithfully mirrors `specter.rs:156-317`. The `bsms:`→`descriptor:` error-prefix `replacen` is correct.
   - `sniff` returns `false`; absent from sniff votes (live-proven: bare descriptor w/o `--format` → "could not detect").
   - The 5 duplicated helpers are **byte-for-byte identical** to the specter originals modulo the `"descriptor"` format-name string (AST-extract diff).

2. **Wiring** — clap value list, dispatch arm, explicit-format `Some("descriptor")` arm (reaches the parser, not shadowed), mod decl, all correct + alphabetical.

3. **Accessor completeness** — all **8** exhaustive `match self` blocks over `ImportProvenance` (`mod.rs:154-301`) have `Self::Descriptor => None` in the correct alphabetical slot (between `ColdcardMultisig` and `Electrum`). Unit variant; `--json` confirms no field silently dropped; `roundtrip: {}` degrades gracefully via the `_ => None` canonicalize fallback.

4. **Tests non-vacuous** — 8 integration + 5 module pass. Multisig cell asserts `cosigners=2` AND `threshold=2`. Explicit-only negative runs without `--format`. `bad_checksum_refused` uses a real wrong checksum (proves validation). Singlesig cell is checksum-LESS (proves tolerant path).

5. **SemVer/lockstep complete** — all toolkit sites at 0.58.0 (Cargo.toml, both READMEs, install.sh:32, fuzz/Cargo.lock, Cargo.lock, CHANGELOG). `git grep 0.57.1` stragglers all unrelated. Manual `--format` row backfilled 9/9 + foreign-formats subsection. FOLLOWUP filed-resolved. GUI v0.42.0 paired-PR correctly noted as pending (not gated).

6. **No bugs / leaks** — `--json` watch-only, no secret material. Network inference graceful on all edge cases (short path / missing origin / non-hardened coin-type / unsupported coin-type → clean error, no panic; testnet coin-type 1 → correct). The 4 non-test `unwrap`/`expect`/`[0]`/`.nth` sites are guarded or compile-time constants (identical to audited specter originals).

The hard gate is satisfied: 0 Critical / 0 Important. Clear to ship toolkit v0.58.0.

---

## FOLD (post-review)
- Cosmetic Minor fixed: `import_wallet.rs:1181` unsupported-format list now includes `descriptor`. Re-verified clippy + the new tests still green.
