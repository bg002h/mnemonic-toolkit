# v0.31.4 end-of-cycle architect review

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** end-of-cycle
**Cycle:** Cycle 11 (sparrow-import-detection-regex-defensive-widening)
**Date:** 2026-05-21
**Pre-tag SHA:** `847672a` (Phase 2 commit; Phase 5 uncommitted on disk)

## Verdict

**GREEN.** All 8 verification items PASS.

## Verification matrix

1. **Regex literal correctness** (`sparrow.rs:349-351`): literal `r"@\d+/\*\*"`, called with `is_match(&script_template)`, `.expect("at-placeholder regex is a fixed string literal")` text meaningful + matches CHANGELOG byte-for-byte.
2. **Inline pattern conformance**: `Regex::new(...)` inline (not `LazyLock`), mirrors I1 fold direction. Precedent: `sparrow.rs:555/566/678`, `bsms.rs:501/520`, `bitcoin_core.rs:530/553/561`.
3. **Test coverage**: both cells exist at lines 1099 + 1139 with meaningful assertions. Regex-unit has 7 positive + 5 negative cases (CHANGELOG cosmetic counting drift was 6 — fixed to 5 inline). Backward-compat parses `sparrow-singlesig-p2wpkh.json` + asserts `84'` survives substitution.
4. **No-behavior-change claim**: under current Sparrow emit (`wallet_export/sparrow.rs:230` indexes `(0..n)`), `@0/**` always present in template-mode ⇒ substring + regex agree. Regex strictly supersets substring. Backward-compat cell locks the claim.
5. **Comment block** (`sparrow.rs:334-347`): documents v0.31.4 rationale + `@\d+(?:/\*\*)?` precedent cite + closes 3 FOLLOWUPs.
6. **Version bumps**: `Cargo.toml:3` = `0.31.4`; `Cargo.lock` = `0.31.4`; `install.sh:32` = `mnemonic-toolkit-v0.31.4`; CHANGELOG header `[0.31.4] — 2026-05-21`.
7. **SemVer PATCH**: defensive hardening + no observable behavior change + no CLI/clap surface change ⇒ correct.
8. **Test totals**: 2152 (+2 vs v0.31.3 baseline 2150).

## Cleared for tag.
