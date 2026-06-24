## R0 ARCHITECT REVIEW — IMPLEMENTATION_PLAN_reproducible_builds_musl.md (round 2, post round-1 fold)

**Verdict: NOT GREEN — 0 Critical / 2 Important / 2 Minor.** Re-dispatch after fold.

Scope: reviewed the PLAN for executability + fidelity to the LOCKED R0-GREEN brainstorm; did NOT re-litigate the design (F1-F6, the BS fold log). Verified all load-bearing live facts against `origin/master` @ `43298598`.

### Round-1 fold (release-rehome) — VERIFIED clean
The round-1 Important is correctly folded into the phase table, P1, P3, P4, and the cross-phase invariants, with no drift introduced. The provenance-gap reasoning is sound: re-homing only the gate jobs while the published `musl-binaries` build stayed on bare `ubuntu-latest` would have made P4's "== published SHA256SUMS" assertion vacuous/false. The fold closes the loop (published hash == gate hash == P4-asserted hash). P4's added PRECONDITION (do not enable until the re-home lands+GREENs) is the right ordering guard.

### Brainstorm fidelity — PASS
Spot-checked every locked constraint; all carried faithfully:
- Remap via `CARGO_BUILD_RUSTFLAGS` ENV channel at fixed `/build/src`, two separate `--remap-path-prefix` flags, NOT `-C`, NOT committed config (R0-C1/C2). ✓
- FULL three-stanza `cargo vendor` `[source]` block as the ONE committed-config exception; "copy stdout verbatim". ✓
- Container distributed BY DIGEST (`docker pull @sha256`), Dockerfile-from-apt as fallback with snapshot.debian.org + exact-version pin. ✓
- `SOURCE_DATE_EPOCH=$(git show -s --format=%ct <commit-SHA>)` off the COMMIT SHA. ✓
- gzip `-n -9` piped form at BOTH live tar sites (man-pages.yml:50 + :133, line numbers verified live); mtime-zero + OS-byte residue checks. ✓
- `--locked --offline`; committed `vendor/` (not CI-time vendor) as canonical. ✓
- P4 = two-DISTINCT-path remap-EXERCISING gate + remap-off NEGATIVE test (RED). ✓ Correctly distinguished from a same-path container-determinism check.
- cc-validation: epoch-unset-→-differs probe + `__DATE__`/host-path residue grep = 0 — both present (P1 cc-validate.sh; aarch64 passthrough probe in P2). De-risks the un-measured musl-cc residual per recon §1.2. ✓
- md FOLLOWUP: do-not-re-edit status + add remap + flip RESOLVED in shipping commit (R0-I1). ✓
- NO-BUMP correct; manual-mirror / schema_mirror / changelog-check NOT tripped (CI/infra + docs + vendor/ only). ✓

### Live-fact verification (against origin/master @ 43298598)
- `musl-binaries` job: `runs-on: ubuntu-latest`, NO `container:` anywhere, host `apt-get install -y musl-tools` (117), `CC_x86_64_unknown_linux_musl: musl-gcc` (123), `tar czf` (133), `sha256sum` (135), gh upload (144-145). All plan citations match. ✓
- miniscript `[patch.crates-io]` rev `95fdd1c5` at **workspace-root** `Cargo.toml:28-29` (the plan/BS "`Cargo.toml:28-29`" is the ROOT file — correct; NOT crates/mnemonic-toolkit/Cargo.toml). Cargo.lock:700 resolves to the git source. ✓
- `.cargo/config.toml` + `Cross.toml` confirmed ABSENT (plan's "verified absent" holds). ✓
- `vendor/` NOT gitignored; `/target` anchored — committing vendor/ is unobstructed. ✓
- Sibling workflow filenames live-correct: md `man-pages.yml`, ms `man-release.yml`, mk `musl-binaries.yml`. ✓

### Blocking findings (2 Important)
The design is correct; these are execution-precision gaps in the load-bearing re-home edit that would mislead an implementer following the plan verbatim:
1. **Matrix `container:` cannot scope to one leg** — the "preferred `container:`" instruction collides with the single matrix job covering both arches; per-step `docker run` (or splitting the job) is actually required, not co-equal.
2. **Network boundary vs `gh release upload`** — `--network=none` must wrap ONLY build+package; checkout + upload stay host+network. Undelineated.

Both are localized to the P1/P3 man-pages.yml edit text + the corresponding cross-phase invariant; the fix is wording precision, not a design change.

### Recommendation
Fold the 2 Important (and ideally the 2 Minor), then re-dispatch this architect on the folded revision per the reviewer-loop discipline. No design re-litigation needed; convergence should be one round.