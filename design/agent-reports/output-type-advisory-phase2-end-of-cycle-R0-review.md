# End-of-cycle R0 review — output-type advisory Phase 2 (mk + md) + Tier-0

> Opus architect end-of-cycle review across all 3 repos (mk `e5620ce..fc2341b`, md `c599292..0599c23`, toolkit `64943f2..520ed16`), branch `output-class-advisory-phase2`. RED (0C/2I) — both Important findings are audit-trail/process, NOT code; code is fully green. Folded → R1 (see the R1 review file).

## Verdict: RED (0C / 2I)

The code across all three repos is correct, complete, green, and ships sound behavior — every functional, version, pin, byte-parity, Tier-0, transcript, and SemVer check passes. The RED is **process/audit-trail only**: the cycle's design artifacts + all architect reviews are uncommitted (CLAUDE.md hard-convention violation), and a shipped FOLLOWUP closure asserts those files are "persisted" when they are not in git. Both are `git add`/prose fixes, no code change.

## Critical
None.

## Important
- **I1 — Cycle design artifacts + ALL architect reviews are uncommitted; the audit trail is not in git.** Toolkit `git status` shows untracked (not gitignored): `design/SPEC_output_type_advisory_phase2_mk_md.md`, `design/IMPLEMENTATION_PLAN_output_type_advisory_phase2_mk_md.md`, and the 8 `design/agent-reports/output-type-advisory-phase2-*` review files; md repo carries an untracked phase-B mirror. `git log --all` returns empty for those paths. Violates CLAUDE.md ("Design artifacts in design/: SPEC_*, IMPLEMENTATION_PLAN_*" + "Per-phase architect-review agent outputs persist verbatim to design/agent-reports/ … Transcript-only review text is unrecoverable from outside the session"). The `520ed16` release commit touched only `design/FOLLOWUPS.md`. **Fix:** stage + commit the cycle's design/review files before tagging.
- **I2 — Shipped FOLLOWUP prose has two dangling cross-citations.** (a) The new toolkit FOLLOWUP `output-class-advisory-byte-parity-test-tautological` Companion line claims mk/md/ms mirror entries that do not exist (`grep -c` = 0 in all three siblings). (b) The sweep closure's Status claims "Per-phase reviews persisted in mnemonic-toolkit design/agent-reports/…" but those files are uncommitted (dissolved by fixing I1). **Fix:** reword the Companion line to drop non-existent mirrors (or add the mirrors); fixing I1 makes the "persisted" claim true.

## Minor
- **M1** — `byte_parity_advisory_lines` tests are tautological (const == inline copy, same file); correctly already filed as the new FOLLOWUP. Within-repo anchoring provided by the positive `.contains(...)` runtime cells. No action beyond I2's reword.
- **M2** — mk CHANGELOG.md not updated for v0.6.1 — pre-existing pattern (mk CHANGELOG abandoned for mk-cli releases since 0.3.x; v0.6.0 also added none). Not a cycle regression. md `[0.6.2]` + toolkit `[0.38.3]` CHANGELOGs are accurate + current.

## Cross-repo integration checks
- **Version ↔ pin** ✓ — mk-cli 0.6.1, md-cli 0.6.2, toolkit 0.38.3; 5 sibling-pin sites point at `mk-cli-v0.6.1` + `descriptor-mnemonic-md-cli-v0.6.2` (ms-cli-v0.5.0 unchanged); `sibling-pin-check.yml` body run verbatim → exit 0.
- **Byte-parity** ✓ — 3 lines byte-identical across toolkit `secret_advisory.rs` (literal `—`), ms-cli, mk-cli, md-cli `output_advisory.rs` (`\u{2014}` == `—` == `\xe2\x80\x94`).
- **Tier-0** ✓ — pin `0.35`, lock `0.35.0`; smoke test real guard (exit 5 recover on 0.35; exit 2 on 0.34); FOLLOWUP correction truthful, no fabricated SHA.
- **FOLLOWUP closures** ✗ (I2) — sweep-closure resolved in all 3 repos w/ consistent wording + intact Companion lines; the NEW tautological FOLLOWUP has the dangling Companion + the persisted-reviews claim.
- **Transcript** ✓ — `24-recover-md1.out` adds exactly the one template note; no other transcript changed.

## Per-crate green (all 3)
- **mk-cli**: `cargo test -p mk-cli` 60/0 (cli_output_class 15); clippy `--all-targets -D warnings` clean.
- **md-cli**: default 21 binaries/0 (cli_output_class 16); `--features cli-compiler` 21 binaries/0 (cli_output_class 18); clippy clean both.
- **mnemonic-toolkit**: `cargo test` **2576/0** (incl. cli_repair_md1_non_chunked + readme_version_current at 0.38.3); clippy clean.
- **Coverage** ✓ — mk 6 emit / 3 inert; md 7 emit / 3 inert; all emits unconditional post-success pre-`Ok`, no `--json` bypass; no gap, no inert mis-emit.
- **SemVer** ✓ — no new flags/subcommands/wire-shape in any repo; all PATCH; toolkit Tier-0 touches no `src/` (no schema_mirror/manual flag lockstep).

## Ship-readiness notes
- **Blocking before tag (resolve the RED):** I1 — commit the toolkit SPEC + plan + 9 agent-reports reviews (+ handle the md mirror); I2 — reword the new FOLLOWUP's Companion line.
- **After GREEN (gated on user authorization):** tag/push mk-cli v0.6.1 (`fc2341b`), md-cli v0.6.2 (`0599c23`), toolkit v0.38.3 (`520ed16`); crates.io publish mk-cli + md-cli (toolkit tag-only). No GUI lockstep (no flag-name change).
