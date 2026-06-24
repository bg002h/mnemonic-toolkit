# P2 (aarch64-musl reproducible) — independent adversarial execution review

**Commit reviewed:** `c3e6e790` (P2; parent `ba10817d` = proven P1) on branch
`feat/repro-p2-aarch64`. (Reviewed at `ef525d33`, pre the two trivial Minor
folds below which were applied as `c3e6e790` — the folds are doc/cosmetic and do
not change the GREEN verdict.)

**Dispatch note.** TWO independent fresh-context reviewer agents were dispatched
(both general-purpose; both fetched + quoted authoritative cross-rs v0.2.5
source). BOTH converged independently to **GREEN — 0 Critical / 0 Important**.
(The first run's verdict arrived after a transcript-tail lag — it initially
appeared truncated mid-reasoning, then the full result landed via the task
completion event; the second run was dispatched in parallel as insurance and
also returned GREEN. No inline self-review was substituted — both are genuine
independent reviews; the first additionally raised the `CARGO_BUILD_RUSTFLAGS`
precision Minor folded below.)

## VERDICT (both reviewers, independently)

**GREEN — 0 Critical / 0 Important.**

All eight residual correctness questions resolved correctly:
- **A** — `BUILDER` is set (cc-validate.sh:66) before the `REMAP_SRC` block;
  cross→`/project`, cargo→`$ROOT`. Never empty/wrong; ordering sound.
- **B** — `/project`/`/build-a`/`$ROOT` are not substrings of the legit
  `/build`/`/cargo`; clean builds pass, a `/project` leak is flagged, `/build`
  (the remap target) is not false-caught.
- **C** — `--unset=SOURCE_DATE_EPOCH` → bare `docker -e` passthrough forwards
  nothing → genuinely unset in container. benign/blocker disambiguation sound.
- **D** — cross's `CARGO_TARGET_DIR=/target` maps to host `$real_root/target`
  (`metadata.target_directory = workspace_root.join("target")`, mounted
  `-v <target>:/target`); the `find`/`cp` hit the right host location.
- **E** — `sudo mkdir`+`chown` cover every write/read path; QEMU binfmt
  (docker/setup-qemu-action) auto-translates the bare aarch64 exec.
- **F** — the 3-block `--config` quoting is balanced (verified: 10 args = 5
  flags + 5 intended `source."git+…".key="value"` values); `CROSS_IMG`
  grep/sed extracts the full digest-pinned image; `run_aarch64: false` on the
  release `repro` call skips only the slow gate while the man-pages
  `musl-binaries` aarch64 leg re-homes the release artifact reproducibly.
- **G** — `type: boolean` inputs evaluate as real booleans in `if:`; no
  `"false"`-truthy string-coercion gotcha (that applies to string inputs only).
- **H** — the x86_64/P1 path is behaviorally byte-identical to the proven P1
  recipe (BUILDER defaults to cargo → REMAP_SRC=$ROOT → same flags); the
  `PATHS_RE` additions (`/project`, the former `${REMAP_SRC}`) are harmless on
  x86_64. No P1 regression.

### Minor (non-blocking)

1. **cc-validate.sh `PATHS_RE` — redundant `${REMAP_SRC}` alternative.**
   `${REMAP_SRC}` is always already a member (`$ROOT` on cargo, `/project` on
   cross). **FOLDED** (`c3e6e790`): dropped the duplicate; comment added.
2. **cc-validate.sh `DIFF_EPOCH` ↔ `'Jan  1 1980'` allowlist coupling** is
   implicit. **FOLDED** (`c3e6e790`): a ⚠ COUPLING comment ties the two
   constants. (Pre-existing from P1.)
3. **`find … -print -quit` takes the first secp `.o`.** Acceptable for the
   single-TU libsecp build; pre-existing from P1, not introduced by P2. Left as
   a P4-hardening note, NOT folded (out of P2 scope).
4. **(first reviewer) Cross.toml comment imprecise re `CARGO_BUILD_RUSTFLAGS`.**
   The var is REDUNDANT in the passthrough list — `cross` auto-forwards every
   `CARGO_`-prefixed var (add_cargo_configuration_envvars skips only a fixed
   non-RUSTFLAGS set), so the remap reaches the container even without the entry.
   Redundant ≠ wrong. **FOLDED**: a PRECISION NOTE in Cross.toml distinguishes
   the genuinely-required NON-`CARGO_`-prefixed vars from the kept-for-legibility
   redundant one.

## Externals confirmed against authoritative source (cross-rs v0.2.5)

- `src/docker/local.rs`: `mount_volumes==false` ⇒ `-v <host_root>:/project:z`,
  `-v <cargo>:/cargo:z`, `-v <target>:/target:z`.
- `src/docker/shared.rs`: `CARGO_HOME=/cargo`, `CARGO_TARGET_DIR=/target`;
  `metadata.target_directory = workspace_root.join("target")`;
  `add_cargo_configuration_envvars` forwards `CARGO_*` except a skip set that
  does NOT include `CARGO_BUILD_RUSTFLAGS`; `[build.env] passthrough` emits
  `docker -e <VAR>` (bare = forward-if-set).
- `src/lib.rs`: forwards `args.all` (the 5 `--config` flags + `--locked
  --offline`) to the inner cargo.
- `docker/Dockerfile.aarch64-unknown-linux-musl`: `CC_aarch64_unknown_linux_musl
  = aarch64-linux-musl-gcc` (GCC) ⇒ `.comment` carries `GCC:`.

**Ship.** (CI verification — the aarch64 cross/QEMU build + byte-identical/cc
proofs — runs after push; Docker/cross/QEMU absent on the dev box.)
