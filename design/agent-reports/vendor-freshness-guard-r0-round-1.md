# R0 review — `vendor/` freshness CI guard (round 1)

**Reviewer:** opus architect (adversarial R0, pre-implementation gate)
**Spec:** `design/SPEC_vendor_freshness_ci_guard.md` (Source SHA master @ `45be1ec1`)
**Repo state at review:** HEAD `45be1ec1` (post-v0.74.0 re-cut), toolchain `cargo 1.85.0`,
host `x86_64-unknown-linux-gnu`. All claims below were **empirically tested** against the
live repo (isolated copies in scratch; the real `vendor/` was not mutated — verified clean).

---

## Verdict

**GREEN — 0 Critical / 0 Important.**

The load-bearing design choice (a **host-target**, offline, vendored-source-replacement
resolution check) is **empirically proven to catch the exact v0.74.0 bug class with zero
false negatives** — including the worst-case concern raised (a *non-host-target-only*
vendored crate going stale). The guard correctly fills a real PR-time gap that no existing
workflow covers. Findings below are **Minor/Nit** only — none gate implementation.

The single highest-value (still non-blocking) refinement: **use `cargo metadata` instead of
`cargo check`** as the check command, and **derive the rev from `Cargo.lock`, not
`Cargo.toml`**. Reasoning is settled in the "Recommended check command" section. The spec is
free to adopt `cargo check` instead if it wants the strictly-larger checksum-drift coverage;
both are correct. Either way this is GREEN.

---

## The decisive experiment (Question 1 — false-negative risk)

The spec's central bet: a **host** resolution check catches a **musl**-release vendor-staleness
because the failure is a *resolution* (not compile) failure and resolution is target-agnostic.
I tried hard to break this. It holds:

| Scenario (on Linux **gnu** host, musl never built) | `cargo check` (host) | `cargo metadata` |
|---|---|---|
| FRESH `vendor/` (current master) | **exit 0** (compiles, 15s cold) | **exit 0** (<1s) |
| md-codec vendored as 0.39.0, Cargo.lock wants 0.39.1 (the **exact v0.74.0 bug**) | **exit 101** — `failed to select a version for md-codec = "^0.39.1" … directory source vendor` | **exit 101** (same error) |
| `windows-sys` (a **Windows-only**, non-host `cfg(windows)` dep) vendored at wrong version | **exit 101** | **exit 101** |
| `windows-sys` vendor dir **entirely deleted** | **exit 101** — `no matching package named windows-sys` | **exit 101** |

**Why the false-negative concern does not materialise:** with `source.crates-io.replace-with
= vendored-sources` active, cargo's *resolution* phase validates **every `Cargo.lock` entry
against the vendor directory regardless of target cfg**. The directory source must contain a
satisfying version for each locked package, and a `[target.'cfg(windows)']` dep faults on
Linux just the same. I verified the dep graph genuinely contains non-host-target deps
(`windows-sys`, `wasi`, `wasip2`, `r-efi`, plus `getrandom`'s freebsd/netbsd/openbsd/solaris
arms) — and the host check REDs on their staleness anyway. The "musl-only staleness slips
through" hypothesis is **disproven** for the realistic bug class (forgotten re-vendor →
wrong-version or missing crate). **No musl toolchain / Docker / QEMU is required** — the
spec's cost-saving claim is correct.

This also vindicates the spec's "verified locally @ the fix" assertion; I reproduced it
independently.

---

## Why this is genuinely the missing *leading* gate (Question 6 — right layer)

Not redundant with anything. Confirmed by inspection + experiment:

- **`repro-drift.yml`** triggers only `schedule:` (weekly) + `workflow_dispatch` — no PR/push.
  Docker-based, full bit-for-bit. Lagging.
- **`man-pages.yml` / `reproducible-musl-build.yml`** — tag-triggered (`mnemonic-toolkit-v*`)
  or `workflow_call`. This is the leg that **failed at v0.74.0**. Lagging.
- **`rust.yml`** *does* run `cargo metadata --locked` on PR+push (step "Verify Cargo.lock is
  up to date", line ~94) — **but WITHOUT `--offline` and WITHOUT the vendored-source config.**
  I confirmed it returns **exit 0 on the stale vendor** (it resolves against live crates.io —
  "Updating crates.io index / Downloading crates" — and never consults `vendor/`). *That is
  exactly why v0.74.0 slipped every PR gate.* The proposed guard is precisely the
  `--offline` + `SRC_CONFIG` variant of an already-established house pattern. The gap is real
  and this is the correct, minimal closure.

(Worth a one-line note in the spec/impl: the new guard is the offline-vendor sibling of
`rust.yml`'s existing online `cargo metadata --locked` step — readers will otherwise wonder
why there are two metadata steps.)

---

## False positives (Question 2)

No realistic false positive found.

- Re-ran FRESH master through both commands → clean exit 0.
- **Checksum handling under `--offline`:** distinguishes the two candidate commands.
  - `cargo metadata` does **pure resolution** — it does *not* read `.cargo-checksum.json`. A
    vendor tree whose *source files* were hand-edited but whose version + checksum file are
    intact passes metadata (exit 0). That is **not** a false positive; it's simply outside
    metadata's scope (and outside the v0.74.0 bug class).
  - `cargo check` additionally verifies `.cargo-checksum.json` at compile time. I confirmed it
    REDs (`the listed checksum of …/md-codec/src/lib.rs has changed`) on an edited-but-
    version-intact tree. This is a **true positive** for a corrupt/partial re-vendor, not a
    false one. So `cargo check` REDs on a *strictly larger* set than metadata, and every
    member of that larger set is a genuine vendor defect. No false positive either way.
- The `--config source.…` three-block form is the same one already proven on cargo 1.85.0 in
  `reproducible-musl-build.yml` (POSITIVE 3-block → exit 0). Pinning the guard's toolchain to
  1.85.0 (as the spec says) keeps it aligned with the repro leg and avoids cargo-version
  config-schema drift.

---

## Minor / Nit

**M1 (Minor) — `cargo metadata` is the better default command than `cargo check`.**
Both catch the v0.74.0 bug; the difference is cost vs. coverage:
- `cargo metadata --locked --offline <SRC_CONFIG>` = **<1s**, pure resolution, **no compile,
  no `--target`, no `--bin`/`-p` needed** (it resolves the whole workspace graph). Catches
  the entire v0.74.0 class (wrong version / missing crate / forgotten re-vendor).
- `cargo check --locked --offline <SRC_CONFIG> -p … --bin …` = **15s cold**, and *additionally*
  catches `.cargo-checksum.json` drift / a partial-corrupt re-vendor.

Recommendation: settled below. (Not blocking — `cargo check` is a defensible "catch more"
choice; just make the choice deliberately, and if you keep `check`, note in the spec that the
extra 15s buys checksum-drift coverage the bug-of-record didn't need.)

**M2 (Minor) — derive `MINISCRIPT_REV` from `Cargo.lock`, not `Cargo.toml`.**
The spec greps the rev from `Cargo.toml [patch.crates-io]`. That works *today* **only if** the
grep is anchored — note `Cargo.toml` line 12 contains the human comment `…master rev 95fdd1c…`
and line 17 `This rev PREDATES…`; a bare `grep rev` matches both. A `rev = "[0-9a-f]{40}"`-
anchored grep is safe today (the comment forms have no `= "…"`). **But** `Cargo.lock` carries
the *authoritative resolved* rev in a machine-generated, comment-free line:
`source = "git+https://github.com/rust-bitcoin/rust-miniscript?rev=<REV>#…"`. Deriving from
Cargo.lock is strictly more robust because (a) no human prose to trip on, (b) it is the value
the vendored source-key must literally match (the `[source]` key cargo expects is
`source."git+…?rev=<REV>"`, mirroring the lock string), (c) it auto-tracks a future 2nd
patched-git dep without an ambiguous multi-match. Either derivation is acceptable; Cargo.lock
is the more durable choice and removes the "grep robustness / multiple [patch] entries" risk
the prompt flags. Document the live SHA in the script comment regardless.

**N1 (Nit) — push-to-master coverage is already in the design; keep it.** The spec's trigger
(`pull_request` + `push: [main, master]`) matches the established house style
(`rust.yml`, `gui-pin-drift-check.yml`, etc.) and correctly covers a **direct FF that skips a
PR** — important because the m-format release model does direct FF + tag for toolkit/codec
(per CLAUDE.md). Good as specified; just don't drop the push leg.

**N2 (Nit) — path-filter completeness is sound, with one belt-and-suspenders option.**
The realistic bug (transitive/codec bump) lands in **`Cargo.lock`**, which is in the filter →
covered (this is literally the v0.74.0 shape). A `[patch]` rev change lands in **`Cargo.toml`**
→ covered. A re-vendor lands in **`vendor/**`** → covered. I could not construct a
re-vendor-needing dep change that touches *none* of `{Cargo.lock, **/Cargo.toml, vendor/**}` —
a resolved dep change always rewrites `Cargo.lock`. So the filter has no realistic blind spot.
(Optional hardening if you want zero chance of a filter mistake silently skipping the gate:
run this guard **unfiltered** — it's <1s with `cargo metadata` — or fold it into `rust.yml`
which already fires on `crates/**`/`Cargo.toml`/`Cargo.lock`. Not required; the filter as
specified is correct.)

**N3 (Nit) — scope: codecs are an honest FOLLOWUP, not in-scope creep.** Confirmed all three
codec repos commit `vendor/` (`descriptor-mnemonic` 125 files, `mnemonic-secret` 118,
`mnemonic-key` 135), consume the same reusable repro recipe, and have **no PR-time vendor
guard** — i.e. the **identical latent bug**. They use the **two-block** form (no miniscript
fork; verified `descriptor-mnemonic` has no `[patch.crates-io]`), so the same script ports
verbatim with `MINISCRIPT_REV=""` — exactly the parameterization
`reproducible-musl-build.yml` already supports. Leaving them to a catalog FOLLOWUP is
acceptable for this cycle **provided** the FOLLOWUP is filed with the companion cross-cite
discipline (CLAUDE.md "Cross-repo follow-ups"): a `vendor-freshness-pr-gate` entry in each
codec's `design/FOLLOWUPS.md` plus the toolkit's, since this is a known shared exposure, not a
hypothetical. (The bug already bit toolkit; a codec dep bump could bite md/ms/mk identically.)
Don't force them into this cycle, but file them loudly.

**N4 (Nit) — error-message UX.** The spec's `::error::` text ("run `cargo vendor vendor/`…")
is good. Cargo's own native message already says `note: perhaps a crate was updated and
forgotten to be re-vendored?` — quote/preserve cargo's stderr in the failure output so the
maintainer sees the offending crate + version, then append the remediation line.

---

## Recommended check command

**Primary recommendation — `cargo metadata` (resolution-only), rev derived from `Cargo.lock`:**

```sh
# Derive the locked miniscript fork rev from Cargo.lock (authoritative, comment-free).
MINISCRIPT_REV="$(grep -A2 'name = "miniscript"' Cargo.lock \
                  | grep -oE 'rev=[0-9a-f]{40}' | head -1 | cut -d= -f2)"

SRC_CONFIG=( --config 'source.crates-io.replace-with="vendored-sources"' )
if [ -n "$MINISCRIPT_REV" ]; then
  SRC_CONFIG+=(
    --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".git=\"https://github.com/rust-bitcoin/rust-miniscript\""
    --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".rev=\"${MINISCRIPT_REV}\""
    --config "source.\"git+https://github.com/rust-bitcoin/rust-miniscript?rev=${MINISCRIPT_REV}\".replace-with=\"vendored-sources\""
  )
fi
SRC_CONFIG+=( --config 'source.vendored-sources.directory="vendor"' )

cargo metadata --format-version 1 --locked --offline "${SRC_CONFIG[@]}" >/dev/null
```

**metadata vs. check vs. --target — the reasoning, settled:**

- **`cargo metadata` wins on the bug-of-record.** Empirically it REDs on every member of the
  v0.74.0 class (wrong version, missing crate) — including non-host-target deps — at **<1s**,
  **no compiler invocation**, no `--target`/`--bin` needed. It is the cheapest command with
  **no false negatives for the realistic bug class**.
- **`cargo check` (host)** is the only other contender and is **also correct** — it REDs on the
  same class *plus* `.cargo-checksum.json` drift / partial-corrupt re-vendor, at the cost of a
  ~15s cold host compile. If the maintainer wants the guard to also catch a hand-edited /
  half-re-vendored tree (a plausible human error during a manual `cargo vendor`), choose
  `cargo check`. Keep it on the **host target** — adding `--target x86_64-unknown-linux-musl`
  buys **nothing** here (resolution is target-agnostic, proven above) and would force a musl
  toolchain onto the runner. Do **not** use `--target musl`.
- **Bottom line:** default to `cargo metadata` (cheapest, zero false-negative for the bug it
  exists to prevent). Upgrade to `cargo check` *only if* you deliberately want checksum-drift
  coverage and accept +15s. Both pass R0. The prompt asked for "the cheapest command with no
  false negatives for the realistic bug class" → that is **`cargo metadata`**.

A defensible belt-and-suspenders is to run `cargo metadata` (the freshness gate, <1s,
unfiltered) **and** keep relying on the existing tag-time repro build for the full compile —
no need to pay 15s on every PR for a class the metadata resolve already covers.

---

## Conformance / gate notes for implementation (non-blocking)

- Pin the workflow toolchain to **1.85.0** (matches the repro leg; the `--config [source]`
  schema is version-sensitive). Spec already says this.
- The guard runs on the **host** — no `actions/setup-qemu`, no Docker, no `container:`. Keep it
  a plain `ubuntu-latest` job.
- Keep the `push: [main, master]` leg (direct-FF coverage) — do not reduce to PR-only.
- File the codec FOLLOWUPs with companion cross-cites per CLAUDE.md before this cycle ships.
- Per CLAUDE.md reviewer-loop discipline: this verdict is GREEN, but if the spec is amended to
  fold M1/M2 (command + rev-source change), **re-dispatch a scoped convergence review** of the
  amended spec before implementation — folds can introduce drift.
