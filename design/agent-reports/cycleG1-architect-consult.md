# Fable architect consult — g4_a mlock miri disposition (Cycle G.1)

**Reviewer:** Fable architect (read-only), per user directive "ask fable architect."
**Dispatched:** 2026-07-09, after the v0.82.0 (Cycle G) release turned the `rust`→`miri (mlock unsafe)` job red on `g4_a_pin_and_zeroize_compose_without_panic`. Persisted verbatim per CLAUDE.md.

## VERDICT: **B** — layout-robust assertion fix in both repos via an ms-cli patch tag (`ms-cli-v0.14.1`), run as its own small cross-repo micro-cycle, promptly but not as an emergency. v0.82.0 stands as shipped — no re-cut, no defect.

## Fact-check (all claims verified against live repo + CI)

1. **Verified.** `g4_a_pin_and_zeroize_compose_without_panic` at `crates/mnemonic-toolkit/src/mlock.rs:424-433`; log shows `assertion left == right failed: 64-byte buf pins exactly one page / left: 2 / right: 1` at `:429`.
2. **Verified.** `round_to_pages` (`:129-138`) rounds addr down + `addr+len` up — a 64-byte buffer at page-offset >4032 legitimately touches 2 pages. `page_count==2` is correct, fail-safe behavior; the `==1` assert encodes an allocator-placement assumption, not a safety property.
3. **Verified.** `git diff --name-only 818df179 8cf8bbe9` — `mlock.rs` absent; Cycle G touched `secret_string.rs`, `repair.rs`, `cost/strip.rs`, etc. The `--lib` miri test binary compiles the whole lib, so the layout-shift mechanism is consistent with miri's deterministic allocator.
4. **Verified.** No `error: Undefined Behavior` anywhere in the log — only a dead-code warning, miri's informational int-to-ptr-cast note, and the plain assertion panic. 6 passed / 1 failed / 3 ignored.
5. **Verified.** Same run: ubuntu, macos, windows-check, freebsd-check, aarch64-check, musl×2, release+einval, g6-invariant, fmt, clippy — all green. Only `miri (mlock unsafe)` red.
6. **Verified.** `required_status_checks.contexts == ["examples"]`.
7. **Verified.** f4461c07 (v0.81.0) and the six prior release runs back to v0.75.0 all green; miri toolchain is unpinned `dtolnay/rust-toolchain@nightly` (rust.yml:183), no `MIRIFLAGS`/seed. The FOLLOWUP at `design/FOLLOWUPS.md:45-47` is accurate on every substantive point, with **one overstatement — see below**.

**g6 constraint confirmed:** `mlock_g6_invariant.rs` normalizes the **whole file** (strips comment-only lines, preserves code incl. the `#[cfg(test)]` block) and asserts byte-equality against `mnemonic-secret`'s `crates/ms-cli/src/mlock.rs` checked out at the tag parsed from install.sh (`ms-cli-v0.14.0`, install.sh:38). The assert is a code line, so normalization gives no escape — any test edit is cross-repo.

## Reasoning

**(i) Does the deterministic red degrade the signal? Yes, materially — and this is what breaks the "leave tracked" precedent.** The prior architect ruling covered an *intermittent* flake; this is deterministic red on **every future master push** at this layout — the whole `rust` workflow now concludes `failure` on green code, indefinitely, pending nightly-allocator roulette. That trains both humans and this project's own polling agents (which use run-conclusion as ground truth per the standing ledger discipline) to pattern-match "rust red = known g4_a thing," which is exactly the blindness that would mask a *real* future miri finding. One calibration, though: the miri job's signal is real but modest — under `cfg(miri)` the syscalls are **stubbed to no-ops** (mlock.rs:256-264; rust.yml:190 says so explicitly), and the int-to-ptr cast puts miri in permissive-provenance mode. What miri actually checks is the pointer arithmetic, `Vec::zeroize` composition, and Drop ordering around the stubs. Modest — but it is the only UB-checker on the constellation's most safety-sensitive shared module, so it's worth keeping green rather than teaching everyone to ignore it.

**(ii) Is a cross-repo ms-cli patch proportionate? Yes — and it's lighter than framed.** The FOLLOWUP (and the framing above) claims the fix requires a **re-vendor: it does not.** The toolkit's Cargo dependency is on `ms-codec`, not `ms-cli`; the g6 job only *checks out ms-cli source* for byte-comparison. A test-only mlock.rs edit in ms-cli changes no toolkit Cargo dep, so no `Cargo.lock`/vendor ripple. Real cost: one-line edit ×2 repos, `ms-cli-v0.14.1` patch tag (+crates.io publish per the established codec ritual), install.sh ms-cli arm bump, and a sweep of any `--tag ms-cli-v0.14.0` references in workflows/manual prose (sibling-pin-check itself gates that sweep — the prior "frozen baseline" gotcha was about bumping the pin *without* the sweep, not a prohibition on bumping). No toolkit re-tag strictly required; the g6 job resolves the pin from the checked-out install.sh, so one atomic toolkit commit goes green once the ms-cli tag exists. **One execution caution:** the Cycle-A gotcha — `examples.yml` re-runs `gen.sh` on install.sh changes; watch the required `examples` check on that commit.

**(iii) Minimal correct change.** The prompt's "over-allocate + align a slice" idea is strictly worse: more diff in a frozen module, same cross-repo cost. The right fix is to stop asserting an allocator accident and assert the actual contract:

```rust
let expected = round_to_pages(v.as_ptr() as usize, v.len(), page_size()).1;
assert_eq!(pin.page_count, expected, "pin covers exactly the pages the buffer touches");
assert!((1..=2).contains(&pin.page_count), "a 64-byte buf touches 1 or 2 pages");
```

(The floor is just the second line; the pair is preferred — the range bound is the non-tautological invariant, the computed-equality catches pin/round divergence.) Identical text in both repos' mlock.rs.

**Process: mechanical-scale, but with ONE scoped R0 review round — not the full spec/plan pipeline, and not zero review.** It's a two-line test-only diff, but it pierces a deliberately frozen funds-safety-adjacent file and drives a cross-repo release; the project's standing rule ("'mechanical fix' is the rationalization the standard overrides") applies. One scoped architect pass over the diff + release-site checklist (install.sh arm, prose/workflow pin sweep, examples-gate behavior), then ship. **Own micro-cycle** — it cannot fold into anything queued (nothing ms-cli-touching is in flight), and folding it into an unrelated cycle would couple an ms-cli tag to foreign scope. Do *not* add an interim `--skip g4_a` to rust.yml — at a one-day fix horizon the bridge is pure added state; it only becomes worth considering if B is deferred beyond a couple of weeks, and then only with the FOLLOWUP slug cited in the workflow comment and a revert obligation attached.

**(iv) v0.82.0 soundness: sound and correctly shipped.** Zero mlock.rs lines changed; miri found no UB; all real-platform suites and all release-critical gates green; the failure is a pre-existing latent test fragility (an over-strict allocator-placement assumption) exposed by a benign, unavoidable lib-layout shift. Correctly filed as LOW severity. No re-cut, no hotfix to the release itself.

**One fold-back when the fix ships:** correct the FOLLOWUP's "re-vendor" claim (`design/FOLLOWUPS.md:47`) — no re-vendor is required since the toolkit depends on ms-codec, not ms-cli; keeping that overstatement would make the fix look heavier than it is to a future prioritization pass.
