# P4 (reproducibility drift gate) — independent adversarial execution review

**Commit reviewed:** `0a26852b` ("ci(repro): P4 — remap-off negative + scheduled
aarch64 drift gate (NO-BUMP) (#23)") on branch `feat/repro-p4-drift` (parent
`6e37b18e`).

**Scope reviewed:** the P4 drift-gate diff — `ci/repro/remap-off-negative.sh`
(the arch-aware remap-off negative probe), the scheduled re-proof workflow
`.github/workflows/repro-drift.yml`, the `remap-off NEGATIVE` steps wired into
both gate jobs of `.github/workflows/reproducible-musl-build.yml`
(`repro-x86_64-musl` ~L499, `repro-aarch64-musl` ~L677), and the §8.1 doc body in
`docs/verify-reproducibility.md`.

## VERDICT

**GREEN on the positive gate; 0 Critical / 1 Important / 2 Minor.**

The implemented P4 does NOT regress the positive gate (`double-build.sh`
byte-identity) or the scheduled gate (weekly + `workflow_dispatch` re-proof with
`run_aarch64: true`). The remap-off negative is arch-aware and correctly
asserts the load-bearing property per builder:

- **x86_64 / `cargo`:** the two no-remap binaries MUST DIFFER (each leg's own
  absolute build path leaks into `.rodata` once `--remap-path-prefix` is dropped).
  Byte-identity without the remap ⇒ the remap is hollow ⇒ RED.
- **aarch64 / `cross`:** `cross` collapses both legs to its fixed `/project`
  mount, so an A/B-difference is structurally unsatisfiable (the same reason A/B
  is weak aarch64 evidence, R0-I2). The negative instead asserts the no-remap
  build LEAKS `/project` (or `$CARGO_HOME`) residue the remap would strip. Zero
  residue ⇒ hollow remap ⇒ RED.

The negative probe holds every OTHER variable identical to the positive build
(epoch, `LC_ALL=C`, `TZ=UTC`, offline vendor resolution, the job-scoped `[source]`
`--config` activation reconstructed identically to `double-build.sh` /
`cc-validate.sh`, the MINISCRIPT_REV three-block-vs-two-block default) — the ONLY
removed variable is the remap (`CARGO_BUILD_RUSTFLAGS=""`, no `-ffile-prefix-map`),
so a divergence can be attributed to the remap and not an unrelated source. The
TWO-DISTINCT-PATH precondition is enforced (`PATH_A == PATH_B ⇒ exit 2`).

## Findings

### Important

**I1 — the explicit `== published SHA256SUMS.<arch>` assertion the R0 plan
specified for P4's positive gate is NOT implemented.** The R0 plan defined P4's
positive gate as also asserting each rebuilt `.tar.gz` SHA-256 equals the
published `SHA256SUMS.<arch>`. The implemented P4 (negative probe + scheduled
re-proof) adds no explicit published-hash comparison.

**ADJUDICATION — DESCOPE (architecture call; no user escalation).** The explicit
comparison is intentionally descoped because:
(a) the scheduled gate runs at `github.sha` (HEAD), which has NO published release
    artifact to compare against — a literal `== published` would false-RED every
    schedule (and every non-release commit), because there is nothing to compare
    the rebuild to;
(b) the closed loop is satisfied STRUCTURALLY — the P1/P3 release re-home
    PUBLISHES the exact double-build canonical output, so publish and gate share
    an identical *(commit SHA, container digest, build recipe)* tuple, and the
    recipe is reproducible by construction ⇒ the published artifact IS the
    gate-verified binary. The "rebuilt hash == published hash" loop holds not
    because a check asserts it but because the same byte-for-byte recipe produced
    both.

**Resolution (both legs applied in this fold):**
1. `docs/verify-reproducibility.md` §8.1 gains an explicit subsection ("Why the
   drift gate does NOT explicitly assert `== published SHA256SUMS.<arch>`") stating
   the assertion is satisfied structurally by the re-home (publish + gate share
   the identical commit/container/recipe) and the explicit hash-comparison is
   deferred because a HEAD-scheduled gate has no release peer.
2. A catalog FOLLOWUP `repro-explicit-published-hash-gate` is filed in
   `design/FOLLOWUPS.md` (status OPEN/catalog, severity MINOR, tier `ci`,
   cross-citing this cycle) for a future `release:`-published-triggered job that
   downloads the just-uploaded `SHA256SUMS.<arch>` and asserts equality against a
   fresh distinct-path rebuild — the explicit per-release closed-loop check
   (catches a hypothetical upload-path corruption the structural argument assumes
   away).

### Minor

**m1 — the x86_64 cargo branch asserts DIFFER but does not PIN the cause.** After
`cmp` confirms the two no-remap binaries differ, the script proved only that
*some* non-determinism exists between the legs — not that the divergence is
SPECIFICALLY the build-path leak `--remap-path-prefix` guards. An unrelated source
of non-determinism would also make `cmp` differ and would falsely GREEN the
probe. **Folded:** after the DIFFER assertion, the cargo branch now greps the
differing binaries for their own per-leg `/build-a` / `/build-b` build-root path
substring and REDs unless at least one leg leaks its own build-root path —
proving the divergence is the build-path leak. Kept cheap (one `grep -aq` per leg).

**m2 — the cross `/project` residue anchor needs a digest-bump re-verify note.**
The `/project` literal in the cross residue regex is cross v0.2.5's fixed bind
mount; a future cross-image digest bump must re-verify cross still mounts the
source to `/project`. **Folded:** a one-line comment was added near the
`RESIDUE_RE` anchor noting the digest-bump re-verify obligation, matching the
existing pin convention the positive scripts (`double-build.sh` / `cc-validate.sh`)
already carry.

## Gate verification

- `bash -n ci/repro/remap-off-negative.sh` — clean (no syntax errors).
- `actionlint` over the repro workflows — clean.
- Positive gate (`double-build.sh` byte-identity) and scheduled gate (weekly +
  `workflow_dispatch`, `run_aarch64: true`) — UNCHANGED by this fold; no regression.

**Post-fold status: 0 Critical / 0 Important** (I1 adjudicated as a documented
descope + catalog FOLLOWUP; m1 + m2 folded). NO-BUMP (CI/docs only).
