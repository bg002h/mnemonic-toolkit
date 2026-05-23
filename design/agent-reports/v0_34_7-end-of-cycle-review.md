# v0.34.7 — End-of-cycle architect review (opus) — MANDATORY pre-release-train gate (cross-repo)

**Date:** 2026-05-23
**Cycle:** v0.34.7 argv-hardening (`PR_SET_DUMPABLE`) across 4 m-format CLIs (+ GUI pin bump deferred to the release train)
**Reviewer:** opus (feature-dev:code-reviewer), end-of-cycle gate
**Scope reviewed:** combined cross-repo diff `/tmp/v0_34_7_crossrepo.diff` + live branch state in all 4 code repos

---

## Critical
(none)

## Important
(none)

## Minor (all documentation-only; none breaks build/CI/release-train)

- **M1 — broken rustdoc intra-doc link.** `mnemonic-toolkit/crates/mnemonic-toolkit/src/process_hardening.rs:12` links `[crate::secret_advisory]`, but `secret_advisory` is a BIN module (`main.rs:21`) while `process_hardening` is in the LIB → `crate::secret_advisory` doesn't resolve from the lib root (rustdoc warning; not caught by CI/clippy). The 3 siblings dropped the link. **Fix:** plain prose (match siblings). **[FOLDED.]**
- **M2 — dangling companion-FOLLOWUP slug.** The 3 sibling CHANGELOGs cite "Companion FOLLOWUP `argv-hardening-pr-set-dumpable`" which exists nowhere (siblings have no `design/FOLLOWUPS.md`; the toolkit closed `argv-overwrite-after-parse`). **Fix:** redirect to the toolkit's `argv-overwrite-after-parse` closure. **[FOLDED.]**
- **M3 — mk-cli `[0.4.2]` entry in the wrong file.** `mnemonic-key/crates/mk-cli/CHANGELOG.md` (its self-declared source of truth) lacks the entry; it was added to the root `mnemonic-key/CHANGELOG.md` (mk-codec's). **Fix:** add to `crates/mk-cli/CHANGELOG.md`; remove the misplaced root entry. **[FOLDED.]**

---

## Verification summary (all 9 areas)
1. **Module sound + identical** — all 4 `set_non_dumpable()` = `#[cfg(target_os="linux")] unsafe { let _ = libc::prctl(libc::PR_SET_DUMPABLE,0); }` (no-op elsewhere); `unsafe` sound (integer args only); Linux unit test asserts `PR_GET_DUMPABLE==0`; bodies byte-identical (toolkit has extra doc lines — M1).
2. **Hook placement** — FIRST statement in every main() before clap parse: toolkit (lib-path call), md-cli, ms-cli, mk-cli. `mod`/`pub mod` decls confirmed.
3. **libc dep** — added to md-cli + mk-cli; present in toolkit + ms-cli; no other churn.
4. **Versions** — toolkit 0.34.7 (Cargo.toml+lock), md-cli 0.6.1, ms-cli 0.4.1, mk-cli 0.4.2; CHANGELOG entries present (mk mis-placed — M3).
5. **install.sh pins** — self `v0.34.7` + siblings `md-cli-v0.6.1`/`ms-cli-v0.4.1`/`mk-cli-v0.4.2` match the cut-to-be tags.
6. **FOLLOWUP closure** — `argv-overwrite-after-parse` resolved-v0.34.7, accurate, cites all 4 versions.
7. **No CLI surface change** — diff touches only Cargo.toml/lib.rs/main.rs/process_hardening.rs/install.sh; no flag → no schema_mirror/manual lockstep; gui-schema unchanged.
8. **Unit-test side effect** — safe; no ptrace/`/proc/self`/coverage/dumpability dependency in any repo's suite; mlock harnesses orthogonal.
9. **clippy/test status** — new test passes + clippy `-D warnings` clean + suites green (toolkit 123, md 20, ms 37, mk 5) — consistent; M1 invisible to clippy/CI.

VERDICT: GREEN (0C/0I)

---

## Fold disposition (controller)
GREEN (0C/0I) → gate satisfied. Folded all 3 doc-only Minors (M1 toolkit doc-link → prose; M2 sibling CHANGELOG dangling-slug → redirect to `argv-overwrite-after-parse`; M3 mk-cli entry → moved to `crates/mk-cli/CHANGELOG.md`). Doc-only, zero build/test impact → no R2 re-dispatch (no Critical/Important). Cleared for the release train (GATED — user go-ahead + crates.io re-confirm).
