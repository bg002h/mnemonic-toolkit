# R0 REVIEW — cycle-10 md-codec cluster — PLAN-DOC, Round 2

**Plan-doc:** `design/IMPLEMENTATION_PLAN_cycle10_mdcodec_lib.md`
Verified against toolkit `origin/master` + `descriptor-mnemonic origin/main = 1a4b322`.

## VERDICT: GREEN (0 Critical / 0 Important)

Both round-1 Important findings are correctly folded and re-verified; the four Minor folds are clean; no new drift introduced (the fold touched only §2/§6/§7 bookkeeping + 3 Minor lines — the funds-critical M3 logic, L14/L15/L17/L6 phase logic, and the P4 publish→pin chain are UNCHANGED from round 1).

### Important-1 (toolkit pin from-version 0.37 → 0.38) — VERIFIED
`git show origin/master:crates/mnemonic-toolkit/Cargo.toml | grep md-codec` → `36:md-codec = "0.38"`. Plan §2 row + §6 now read `"0.38" → "0.39"`. Root `Cargo.lock` lines 676-677 = `md-codec` / `0.38.0` — plan's `Cargo.lock:677 → 0.38.0` lands exactly on the version line. (The toolkit lockfile is the repo-ROOT `Cargo.lock`, not `crates/mnemonic-toolkit/Cargo.lock`; plan citation is correct as written.)

### Important-2 (bughunt tick lines → 245/368/593/603/621) — VERIFIED
All five land on the correct `### - [ ]` checkbox headers on current toolkit origin/master: M3@245, L6@368, L14@593, L15@603, L17@621; slug text matches each. The plan's "re-grep again at ship time as the report grows" hedge is correct.

### identity.rs:572 preserved — VERIFIED
§2 row "L17 vacuous test" still cites `identity.rs:572` for `fn walletpolicyid_stable_across_origin_elision` (source-file test fn — confirmed at md-codec source line 572). The fold correctly moved ONLY the bughunt-REPORT L17 tick (→621), leaving the source-file `:572` untouched. The `:593` sibling (`walletpolicyid_stable_across_use_site_elision`, keep-as-is) confirmed at source line 593.

### Minor folds — all clean
- Clippy: both command occurrences + the P1-gate prose now `--workspace --all-targets`; no bare `--all-targets` remains.
- §7 FOLLOWUP: now states the five slugs don't exist in `FOLLOWUPS.md` in either repo; bughunt report is system-of-record; "do NOT flip a non-existent slug."
- L14 edit-boundary: §2 + P2 say REPLACE `identity.rs:192-193` (declaration + write call), warning against insert-before-192 (duplicate `path_scratch`).

### Unperturbed round-1-verified items (spot-check)
M3 gate block (`derive.rs:110-122`, `max_alts` fail-closed form), L14/L15/L17 loci, L6 guard mirror (`canonicalize.rs:206`, `n_keys = d.n` borrow note), and the P4 publish→pin chain (md-codec 0.38.0→0.39.0 publish FIRST → md-cli `Cargo.toml:28` `=0.38.0`→`=0.39.0` pin-edit → md-cli 0.9.0→0.9.1 → publish AFTER; tags `md-codec-v0.39.0` / `descriptor-mnemonic-md-cli-v0.9.1`) — all UNCHANGED.

## Disposition
GREEN. The lane may proceed to TDD implementation (single implementer, worktree off `descriptor-mnemonic origin/main = 1a4b322`, RED-first, full `-p md-codec`/`-p md-cli` suites + clippy + `cargo fmt --all --check` gates).
